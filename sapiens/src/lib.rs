//! Botrs library
pub mod context;

/// Prompt generation logic
pub mod prompt;

/// Toolbox for sapiens
pub mod tools;

/// Runner for sapiens
pub mod runner;

/// OpenAI API client
pub mod openai;

use std::fmt::Debug;
use std::sync::{Arc, Weak};

use async_openai::types::Role;
use runner::Chain;
use tokio::sync::Mutex;

use crate::context::{ChatEntry, ChatHistory};
use crate::openai::{Client, OpenAIError};
use crate::runner::{ModelResponse, TaskChain, Usage};
use crate::tools::TerminationMessage;

/// The error type for the bot
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to add to the chat history
    #[error("Failed to add to the chat history")]
    ChatHistoryError(#[from] context::Error),
    /// No response from the model
    #[error("No response from the model")]
    NoResponseFromModel,
    /// The model returned an error
    #[error("Model invocation failed")]
    OpenAIError(#[from] OpenAIError),
    /// Reached the maximum number of steps
    #[error("Maximal number of steps reached")]
    MaxStepsReached,
    /// The response is too long
    #[error("The response is too long: {0}")]
    ActionResponseTooLong(String),
}

/// Configuration for the bot
#[derive(Debug, Clone)]
pub struct Config {
    /// The model to use
    pub model: String,
    /// The maximum number of steps
    pub max_steps: usize,
    /// The minimum number of tokens that need to be available for completion
    pub min_token_for_completion: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            max_steps: 10,
            min_token_for_completion: 512,
        }
    }
}

/// An update from the model
#[derive(Debug, Clone)]
pub struct ModelUpdateNotification {
    /// The message from the model
    pub chat_entry: ChatEntry,
    /// The number of tokens used by the model
    pub usage: Option<Usage>,
}

impl From<ModelResponse> for ModelUpdateNotification {
    fn from(res: ModelResponse) -> Self {
        Self {
            chat_entry: ChatEntry {
                role: Role::Assistant,
                msg: res.msg,
            },
            usage: res.usage,
        }
    }
}

/// Invocation success notification
pub struct InvocationSuccessNotification {
    /// The tool name
    pub tool_name: String,
    /// The input
    pub input: ChatEntry,
    /// The result
    pub res: Result<ChatEntry, Error>,
    /// The number of tokens used by the model
    pub usage: Option<Usage>,
}

/// Invocation failure notification
pub struct InvocationFailureNotification {
    /// The tool name
    pub tool_name: String,
    /// The input
    pub input: ChatEntry,
    /// The result
    pub res: Result<ChatEntry, Error>,
    /// The number of tokens used by the model
    pub usage: Option<Usage>,
}

/// Termination notification
pub struct TerminationNotification {
    /// The messages
    pub messages: Vec<TerminationMessage>,
    /// The number of tokens used by the model
    pub usage: Option<Usage>,
}

/// Observer for the step progresses
#[async_trait::async_trait]
pub trait StepObserver: Send {
    /// Called when the task is submitted
    async fn on_task(&mut self, _task: &str) {}

    /// Called when the task starts
    async fn on_start(&mut self, _chat_history: &ChatHistory) {}

    /// Called when the model updates the chat history
    async fn on_model_update(&mut self, _event: ModelUpdateNotification) {}

    /// Called when the tool invocation was successful
    async fn on_invocation_success(&mut self, _event: InvocationSuccessNotification) {}

    /// Called when the tool invocation failed
    async fn on_invocation_failure(&mut self, _event: InvocationFailureNotification) {}

    /// Called when the task is done
    async fn on_termination(&mut self, _event: TerminationNotification) {}
}

/// A step in the task
pub struct Step {
    task_chain: TaskChain,
    observer: WeakStepObserver,
}

impl Step {
    /// Run the task for a single step
    async fn step(mut self) -> Result<StepOrStop, Error> {
        let model_response = self.task_chain.query_model().await?;

        // Wrap the response as the chat history entry
        let model_update = ModelUpdateNotification::from(model_response);

        // Show the message from the assistant
        if let Some(observer) = self.observer.upgrade() {
            observer
                .lock()
                .await
                .on_model_update(model_update.clone())
                .await;
        }

        // pass the message to the tools and get the response
        let tool_input = model_update.chat_entry;
        let (tool_name, resp) = self.task_chain.invoke_tool(&tool_input.msg).await;
        match resp {
            Ok(response) => {
                // check if the task is done
                if let Some(termination_messages) = self.task_chain.is_terminal().await {
                    if let Some(observer) = self.observer.upgrade() {
                        observer
                            .lock()
                            .await
                            .on_termination(TerminationNotification {
                                messages: termination_messages.clone(),
                                usage: model_update.usage,
                            })
                            .await;
                    }

                    return Ok(StepOrStop::Stop {
                        stop: Stop {
                            termination_messages,
                        },
                    });
                }

                // Got a response from the tool but the task is not done yet
                let res = self
                    .task_chain
                    .on_tool_success(&tool_name, tool_input.clone(), response);
                if let Some(observer) = self.observer.upgrade() {
                    observer
                        .lock()
                        .await
                        .on_invocation_success(InvocationSuccessNotification {
                            input: tool_input,
                            tool_name,
                            res,
                            usage: model_update.usage,
                        })
                        .await;
                }
            }
            Err(e) => {
                // check if the task is done
                if let Some(termination_messages) = self.task_chain.is_terminal().await {
                    if let Some(observer) = self.observer.upgrade() {
                        observer
                            .lock()
                            .await
                            .on_termination(TerminationNotification {
                                messages: termination_messages.clone(),
                                usage: model_update.usage,
                            })
                            .await;
                    }

                    return Ok(StepOrStop::Stop {
                        stop: Stop {
                            termination_messages,
                        },
                    });
                }

                let res = self
                    .task_chain
                    .on_tool_failure(&tool_name, tool_input.clone(), e);
                if let Some(observer) = self.observer.upgrade() {
                    observer
                        .lock()
                        .await
                        .on_invocation_failure(InvocationFailureNotification {
                            input: tool_input,
                            tool_name,
                            res,
                            usage: model_update.usage,
                        })
                        .await;
                }
            }
        }

        Ok(StepOrStop::Step { step: self })
    }
}

/// The task is done
pub struct Stop {
    /// The termination messages
    pub termination_messages: Vec<TerminationMessage>,
}

/// A step or the task is done
pub enum StepOrStop {
    /// The task is not done yet
    Step {
        /// The actual step task
        step: Step,
    },
    /// The task is done
    Stop {
        /// the actual stopped task
        stop: Stop,
    },
}

/// Wrap an observer into the a [`StrongStepObserver<O>`] = [`Rc<Mutex<O>>`]
///
/// Use [`Arc::downgrade`] to get a [`Weak<Mutex<dyn StepObserver>>`] and pass
/// it to [`run_to_the_end`] for example.
pub fn wrap_observer<O: StepObserver + 'static>(observer: O) -> StrongStepObserver<O> {
    Arc::new(Mutex::new(observer))
}

/// A strong reference to the observer
pub type StrongStepObserver<O> = Arc<Mutex<O>>;

/// A weak reference to the observer
pub type WeakStepObserver = Weak<Mutex<dyn StepObserver>>;

/// A void observer
pub struct VoidTaskProgressUpdateObserver;

#[async_trait::async_trait]
impl StepObserver for VoidTaskProgressUpdateObserver {}

impl StepOrStop {
    /// Create a new [`StepOrStop`] for a `task`.
    pub fn new(chain: Chain, task: String) -> Result<Self, Error> {
        let task_chain = chain.start_task(task)?;

        let observer = wrap_observer(VoidTaskProgressUpdateObserver {});

        let observer = Arc::downgrade(&observer);

        Ok(StepOrStop::Step {
            step: Step {
                task_chain,
                observer,
            },
        })
    }
}

impl StepOrStop {
    /// Create a new [`StepOrStop`] for a `task`.
    ///
    /// The `observer` will be called when the task starts and when a step is
    /// completed - either successfully or not. The `observer` will be called
    /// with the latest chat history element. It is also called on error.
    pub async fn with_observer(
        chain: Chain,
        task: String,
        observer: WeakStepObserver,
    ) -> Result<Self, Error> {
        if let Some(observer) = observer.upgrade() {
            observer.lock().await.on_task(&task).await;
        }

        let task_chain = chain.start_task(task)?;

        // call the observer
        if let Some(observer) = observer.upgrade() {
            observer
                .lock()
                .await
                .on_start(task_chain.chat_history())
                .await;
        }

        Ok(StepOrStop::Step {
            step: Step {
                task_chain,
                observer,
            },
        })
    }

    /// Run the task for the given number of steps
    pub async fn run(mut self, max_steps: usize) -> Result<Stop, Error> {
        for _ in 0..max_steps {
            match self {
                StepOrStop::Step { step } => {
                    self = step.step().await?;
                }
                StepOrStop::Stop { stop } => {
                    return Ok(stop);
                }
            }
        }

        Err(Error::MaxStepsReached)
    }

    /// Run the task for a single step
    pub async fn step(self) -> Result<Self, Error> {
        match self {
            StepOrStop::Step { step } => step.step().await,
            StepOrStop::Stop { stop } => Ok(StepOrStop::Stop { stop }),
        }
    }

    /// is the task done?
    pub fn is_done(&self) -> Option<Vec<TerminationMessage>> {
        match self {
            StepOrStop::Step { step: _ } => None,
            StepOrStop::Stop { stop } => Some(stop.termination_messages.clone()),
        }
    }
}

/// Run until the task is done or the maximum number of steps is reached
///
/// See ['StepOrStop::new'], [`StepOrStop::step`] and ['StepOrStop::run] for
/// more flexible ways to run a task
#[tracing::instrument(skip(toolbox, openai_client, observer, config))]
pub async fn run_to_the_end(
    toolbox: tools::Toolbox,
    openai_client: Client,
    config: Config,
    task: String,
    observer: WeakStepObserver,
) -> Result<Vec<TerminationMessage>, Error> {
    let chain = Chain::new(toolbox, config.clone(), openai_client).await;

    let step_or_stop = StepOrStop::with_observer(chain, task, observer).await?;

    let stop = step_or_stop.run(config.max_steps).await?;

    Ok(stop.termination_messages)
}
