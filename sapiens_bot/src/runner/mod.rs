use std::fmt::Debug;

use sapiens::context::{ChatEntry, ChatEntryFormatter, ChatHistory};
use sapiens::runner::Chain;
use sapiens::tools::TerminationMessage;
use sapiens::{Config, Error, StepObserver, StepOrStop};
use serenity::futures::channel::mpsc;
use serenity::futures::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};

use crate::runner::utils::{sanitize_msgs_for_discord, Formatter};

/// Formatting utilities
pub mod utils;

/// Sapiens bot
pub struct SapiensBot {
    chain: Chain,
}

impl SapiensBot {
    /// Create a new bot from the environment variables: OPENAI_API_KEY, ...
    pub async fn new_from_env() -> Self {
        let toolbox = sapiens_tools::setup::toolbox_from_env().await;

        let openai_client = sapiens::openai::Client::new().with_api_key(
            std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in configuration file"),
        );

        let config = Config::default();
        let chain = Chain::new(toolbox, config, openai_client).await;

        Self { chain }
    }

    /// Start a new task
    pub async fn start_task<T>(&mut self, task: String, handler: T) -> Result<StepOrStop<T>, Error>
    where
        T: StepObserver,
    {
        StepOrStop::with_handler(self.chain.clone(), task, handler).await
    }
}

/// Handler for task progress updates
pub struct ProgressObserver {
    /// Whether to show the warm-up prompt
    pub show_warmup_prompt: bool,
    pub job_tx: mpsc::Sender<JobUpdate>,
    entry_format: Box<dyn ChatEntryFormatter + 'static + Send + Sync>,
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
impl StepObserver for ProgressObserver {
    async fn on_start(&mut self, chat_history: &ChatHistory) {
        let format = self.entry_format.as_ref();
        let msgs = chat_history.format(format);
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

    async fn on_model_update(&mut self, model_message: ChatEntry) {
        let msg = self.entry_format.format(&model_message);
        debug!(msg = ?model_message, "on_model_update");

        let msgs = sanitize_msgs_for_discord(vec![msg]);
        self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
    }

    async fn on_invocation_success(&mut self, res: Result<ChatEntry, Error>) {
        debug!(res = ?res, "on_invocation_success");

        match res {
            Ok(tool_output) => {
                let msg = self.entry_format.format(&tool_output);

                let msgs = sanitize_msgs_for_discord(vec![msg]);

                self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
            }
            Err(error) => {
                let msg = format!("*Error*: {}", error);

                let msgs = sanitize_msgs_for_discord(vec![msg]);

                self.job_tx.send(JobUpdate::ToolError(msgs)).await.unwrap();
            }
        }
    }

    async fn on_invocation_failure(&mut self, res: Result<ChatEntry, Error>) {
        debug!(res = ?res, "on_invocation_failure");

        match res {
            Ok(tool_output) => {
                let msg = self.entry_format.format(&tool_output);

                let msgs = sanitize_msgs_for_discord(vec![msg]);

                self.job_tx.send(JobUpdate::Vec(msgs)).await.unwrap();
            }
            Err(error) => {
                let msg = format!("*Error*: {}", error);

                let msgs = sanitize_msgs_for_discord(vec![msg]);

                self.job_tx.send(JobUpdate::ToolError(msgs)).await.unwrap();
            }
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

            let handler = ProgressObserver {
                show_warmup_prompt: job.show_warmup_prompt,
                job_tx: job.tx,
                entry_format: Box::new(Formatter {}),
            };

            let max_steps = job.max_steps;

            let mut current_step = 0;

            match self.sapiens.start_task(job.task, handler).await {
                Ok(step) => {
                    let mut step = step;
                    loop {
                        match step.step().await {
                            Ok(s @ StepOrStop::Step { .. }) => {
                                step = s;
                                // update is going to come through the handler
                                debug!("Step for: {}", task);
                            }
                            Ok(StepOrStop::Stop { stop }) => {
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
