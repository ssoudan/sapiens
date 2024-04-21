use std::env::VarError;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use sapiens::context::{ChatEntryFormatter, ContextDump, MessageFormatter};
use sapiens::models::SupportedModel;
use sapiens::tools::toolbox::Toolbox;
use sapiens::tools::TerminationMessage;
use sapiens::{
    models, wrap_observer, Error, InvalidInvocationNotification, InvocationFailureNotification,
    InvocationResultNotification, InvocationSuccessNotification, MessageNotification,
    ModelNotification, RuntimeObserver, SapiensConfig, TaskState, WeakRuntimeObserver,
};
use serenity::futures::channel::mpsc;
use serenity::futures::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};

use crate::runner::utils::{sanitize_msgs_for_discord, Formatter};

/// Formatting utilities
pub mod utils;

/// Sapiens bot
pub struct SapiensBot {
    toolbox: Toolbox,
    config: SapiensConfig,
}

impl SapiensBot {
    /// Create a new bot from the environment variables: OPENAI_API_KEY, ...
    pub async fn new_from_env() -> Self {
        let toolbox = sapiens_tools::setup::toolbox_from_env().await;

        let _ =
            std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in configuration file");

        let temperature = Some(0.);

        let model = match std::env::var("MODEL") {
            Ok(e) => SupportedModel::from_str(&e).expect("Invalid model"),
            Err(e) => {
                if e == VarError::NotPresent {
                    warn!("MODEL not specified: defaulting to chat-bison-001.");
                    SupportedModel::ChatBison001
                } else {
                    panic!("Invalid model: {}", e)
                }
            }
        };

        let model = match model {
            SupportedModel::ChatBison001 => {
                let google_api_key =
                    std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY is not set");

                models::vertex_ai::build(google_api_key, temperature)
                    .await
                    .expect("Failed to build model")
            }
            SupportedModel::OllamaMixtral
            | SupportedModel::OllamaLlamaPro
            | SupportedModel::OllamaLlama370BInstruct
            | SupportedModel::OllamaLlama3Instruct => {
                let host = std::env::var("OLLAMA_HOST").expect("OLLAMA_HOST is not set");
                let port = std::env::var("OLLAMA_PORT")
                    .expect("OLLAMA_PORT is not set")
                    .parse::<u16>()
                    .expect("OLLAMA_PORT is not a valid port");

                models::ollama::build(host, port, model)
                    .await
                    .expect("Failed to build model")
            }
            _ => {
                let api_key = std::env::var("OPENAI_API_KEY").ok();
                let api_base = std::env::var("OPENAI_API_BASE").ok();

                models::openai::build(model, api_key, api_base, temperature)
                    .await
                    .expect("Failed to build model")
            }
        };

        let config = SapiensConfig {
            model,
            ..SapiensConfig::default()
        };

        Self { toolbox, config }
    }

    /// Start a new task
    pub async fn start_task(
        &mut self,
        task: String,
        observer: WeakRuntimeObserver,
    ) -> Result<TaskState, Error>
where {
        TaskState::with_observer(self.config.clone(), self.toolbox.clone(), task, observer).await
    }
}

/// Handler for task progress updates
pub struct ProgressObserver {
    /// Whether to show the warm-up prompt
    pub show_warmup_prompt: bool,
    pub job_tx: mpsc::Sender<JobUpdate>,
    entry_format: Box<dyn ChatEntryFormatter + 'static + Send + Sync>,
    message_format: Box<dyn MessageFormatter + 'static + Send + Sync>,
}

impl Debug for ProgressObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressHandler")
            .field("show_warmup_prompt", &self.show_warmup_prompt)
            .field("job_tx", &"RefCell<mpsc::Sender<JobUpdate>>")
            // .field("entry_format", &"Box<dyn ChatEntryFormatter + 'static + Send>")
            .finish()
    }
}

#[async_trait::async_trait]
impl RuntimeObserver for ProgressObserver {
    async fn on_start(&mut self, context: ContextDump) {
        let format = self.message_format.as_ref();
        let msgs = context.format(format);
        let last_msg = msgs.last();
        debug!(last_msg = ?last_msg, "on_start");

        if self.show_warmup_prompt {
            let msgs = sanitize_msgs_for_discord(msgs);
            self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
        } else {
            // Show only the last message
            if let Some(last_msg) = last_msg {
                let msgs = sanitize_msgs_for_discord(vec![last_msg.to_string()]);
                self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
            } else {
                warn!("No messages to show - this should not happen");
            }
        }
    }

    async fn on_model_update(&mut self, event: ModelNotification) {
        let msg = self.entry_format.format(&event.chat_entry);
        debug!(msg = ?event.chat_entry, "on_model_update");

        let msgs = sanitize_msgs_for_discord(vec![msg]);
        self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
    }

    async fn on_message(&mut self, _event: MessageNotification) {
        // todo!()
    }

    async fn on_invocation_result(&mut self, event: InvocationResultNotification) {
        match event {
            InvocationResultNotification::InvocationSuccess(InvocationSuccessNotification {
                result,
                ..
            }) => {
                let msg = result;

                let msgs = sanitize_msgs_for_discord(vec![msg]);

                self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
            }
            InvocationResultNotification::InvocationFailure(InvocationFailureNotification {
                e,
                ..
            }) => {
                let msg = format!("*Error*: {}", e);

                let msgs = sanitize_msgs_for_discord(vec![msg]);

                self.job_tx.send(JobUpdate::ToolError(msgs)).await.unwrap();
            }
            InvocationResultNotification::InvalidInvocation(InvalidInvocationNotification {
                ..
            }) => {}
        }
    }
}

/// A job update
#[derive(Debug)]
pub enum JobUpdate {
    Completed(Vec<String>),
    Vec(Vec<String>),
    FailedToStart(Vec<String>),
    ToolError(Vec<String>),
    Over,
}

/// A job to run
pub struct NewJob {
    task: String,
    tx: mpsc::Sender<JobUpdate>,
    max_steps: usize,
    show_warmup_prompt: bool,
}

impl NewJob {
    /// Create a new job
    pub fn new(
        task: String,
        max_steps: usize,
        show_warmup_prompt: bool,
        tx: mpsc::Sender<JobUpdate>,
    ) -> Self {
        Self {
            task,
            tx,
            max_steps,
            show_warmup_prompt,
        }
    }
}

pub struct Runner {
    rx: mpsc::Receiver<NewJob>,
    sapiens: SapiensBot,
}

impl Runner {
    pub async fn new(rx: mpsc::Receiver<NewJob>) -> Self {
        let sapiens = SapiensBot::new_from_env().await;
        Self { rx, sapiens }
    }

    pub async fn run(&mut self) {
        while let Some(job) = self.rx.next().await {
            let task = job.task.clone();
            info!("Starting job: {}", task);

            let mut tx = job.tx.clone();

            let observer = ProgressObserver {
                show_warmup_prompt: job.show_warmup_prompt,
                job_tx: job.tx,
                entry_format: Box::new(Formatter {}),
                message_format: Box::new(Formatter {}),
            };

            let observer = wrap_observer(observer);

            let w_observer = Arc::downgrade(&observer);

            let max_steps = job.max_steps;

            let mut current_step = 0;

            match self.sapiens.start_task(job.task, w_observer).await {
                Ok(step) => {
                    let mut step = step;
                    loop {
                        match step.step().await {
                            Ok(s @ TaskState::Step { .. }) => {
                                step = s;
                                // update is going to come through the handler
                                debug!("Step for: {}", task);
                            }
                            Ok(TaskState::Stop { stop }) => {
                                info!("Task finished: {}", task);

                                let messages = stop
                                    .termination_messages.iter()
                                    .flat_map(|m: &TerminationMessage| {
                                        let msg = format!("# Termination message\n - original question: {}\n - conclusion: {}", m.original_question.trim(), m.conclusion.trim());
                                        sanitize_msgs_for_discord(vec![msg])
                                    }).collect();

                                tx.send(JobUpdate::Completed(messages)).await.unwrap();
                                break;
                            }
                            Err(e) => {
                                error!("Error while running task: {}", e);

                                let msg = format!("Error: {}", e);
                                let msgs = sanitize_msgs_for_discord(vec![msg]);

                                tx.send(JobUpdate::FailedToStart(msgs)).await.unwrap();
                                break;
                            }
                        }

                        current_step += 1;

                        if current_step >= max_steps {
                            info!("Task aborted: {}", task);

                            tx.send(JobUpdate::Over).await.unwrap();
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error while starting task: {}", e);
                    let msg = format!("Error: {}", e);
                    let msgs = sanitize_msgs_for_discord(vec![msg]);

                    tx.send(JobUpdate::FailedToStart(msgs)).await.unwrap();
                }
            }
        }
        warn!("Runner stopped");
    }
}
