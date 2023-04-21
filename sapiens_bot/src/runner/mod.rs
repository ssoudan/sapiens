use std::cell::RefCell;

use sapiens::context::{ChatEntry, ChatEntryFormatter, ChatHistory};
use sapiens::runner::Chain;
use sapiens::{Config, Error, StepOrStop, TaskProgressUpdateHandler};
use serenity::futures::channel::mpsc;
use serenity::futures::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};

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
    pub fn start_task(
        &mut self,
        task: String,
        handler: Box<dyn TaskProgressUpdateHandler>,
    ) -> Result<StepOrStop, Error> {
        StepOrStop::with_handler(self.chain.clone(), task, handler)
    }
}

/// Handler for task progress updates
#[derive(Debug)]
pub struct Handler {
    /// Whether to show the warm-up prompt
    pub show_warmup_prompt: bool,
    pub job_tx: RefCell<mpsc::Sender<JobUpdate>>,
}

struct Formatter {}

impl ChatEntryFormatter for Formatter {
    fn format(&self, entry: &ChatEntry) -> String {
        let msg = entry.msg.clone();
        format!("{:?}:\n{}", entry.role, msg)
    }
}

impl TaskProgressUpdateHandler for Handler {
    fn on_start(&self, chat_history: &ChatHistory) {
        debug!("on_start {:?}", chat_history);
        let msgs = chat_history.format(Formatter {});

        self.job_tx
            .borrow_mut()
            .try_send(JobUpdate::Vec(msgs))
            .unwrap();
    }

    fn on_model_update(&self, model_message: ChatEntry) {
        let msg = model_message.msg.clone();
        debug!("on_model_update {:?}", model_message);
        self.job_tx
            .borrow_mut()
            .try_send(JobUpdate::Text(msg))
            .unwrap();
    }

    fn on_tool_update(&self, tool_output: ChatEntry, _success: bool) {
        // TODO(ssoudan) clean this
        debug!("on_tool_update {:?}", tool_output);
        let msg = tool_output.msg;
        self.job_tx
            .borrow_mut()
            .try_send(JobUpdate::Text(msg))
            .unwrap();
    }

    fn on_tool_error(&self, error: Error) {
        error!("on_tool_error {:?}", error);
        self.job_tx
            .borrow_mut()
            .try_send(JobUpdate::ToolError(error))
            .unwrap();
    }
}

/// A job update
#[derive(Debug)]
pub enum JobUpdate {
    Vec(Vec<String>),
    Text(String),
    FailedToStart(Error),
    ToolError(Error),
}

/// A job to run
pub struct NewJob {
    task: String,
    tx: mpsc::Sender<JobUpdate>,
}

impl NewJob {
    /// Create a new job
    pub fn new(task: String, tx: mpsc::Sender<JobUpdate>) -> Self {
        Self { task, tx }
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

            let handler = Box::new(Handler {
                show_warmup_prompt: true,
                job_tx: RefCell::new(job.tx),
            });

            match self.sapiens.start_task(job.task, handler) {
                Ok(step) => {
                    let mut step = step;
                    loop {
                        match step.step().await {
                            Ok(s @ StepOrStop::Step { .. }) => {
                                step = s;
                                // update is going to come through the handler
                                debug!("Step for: {}", task);
                            }
                            Ok(StepOrStop::Stop { .. }) => {
                                info!("Task finished: {}", task);
                                break;
                            }
                            Err(e) => {
                                error!("Error while running task: {}", e);
                                tx.send(JobUpdate::FailedToStart(e)).await.unwrap();
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error while starting task: {}", e);
                    tx.send(JobUpdate::FailedToStart(e)).await.unwrap()
                }
            }
        }
        warn!("Runner stopped");
    }
}
