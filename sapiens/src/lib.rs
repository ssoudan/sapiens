//! Botrs library
pub mod context;

/// Prompt generation logic
pub mod prompt;

/// Toolbox for sapiens
pub mod tools;

/// Runner for sapiens
pub mod runner;

use std::fmt::Debug;
use std::rc::Rc;

pub use async_openai::error::OpenAIError;
pub use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Role};
use async_openai::Client;
use runner::Chain;

use crate::context::{ChatEntry, ChatHistory};
use crate::runner::TaskChain;
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
            min_token_for_completion: 256,
        }
    }
}

/// Handler for the task progress updates
pub trait TaskProgressUpdateHandler: Debug {
    /// Called when the task starts
    fn on_start(&self, chat_history: &ChatHistory);

    /// Called when the tool updates the chat history
    fn on_tool_update(&self, chat_message: ChatEntry, success: bool);

    /// Called when the tool returns an error
    fn on_tool_error(&self, error: Error);
}

/// A step in the task
pub struct Step {
    task_chain: TaskChain,
    handler: Box<dyn TaskProgressUpdateHandler>,
}

impl Step {
    /// Run the task for a single step
    async fn step(mut self) -> Result<StepOrStop, Error> {
        let chat_msg = self.task_chain.query_model().await?;

        // show the message from the assistant
        self.handler.on_tool_update(chat_msg.clone(), true);

        // pass the message to the tools and get the response
        let (tool_name, resp) = self.task_chain.invoke_tool(&chat_msg.msg);
        match resp {
            Ok(response) => {
                // check if the task is done
                if let Some(termination_messages) = self.task_chain.is_terminal() {
                    return Ok(StepOrStop::Stop {
                        stop: Stop {
                            termination_messages,
                        },
                    });
                }

                // Got a response from the tool but the task is not done yet
                match self.task_chain.on_tool_success(tool_name, response) {
                    Ok(e) => {
                        self.handler.on_tool_update(e, true);
                    }
                    Err(e) => {
                        self.handler.on_tool_error(e);
                    }
                }
            }
            Err(e) => {
                // check if the task is done
                if let Some(termination_messages) = self.task_chain.is_terminal() {
                    return Ok(StepOrStop::Stop {
                        stop: Stop {
                            termination_messages,
                        },
                    });
                }

                match self.task_chain.on_tool_failure(tool_name, e) {
                    Ok(e) => {
                        self.handler.on_tool_update(e, false);
                    }
                    Err(e) => {
                        self.handler.on_tool_error(e);
                    }
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

impl StepOrStop {
    /// Create a new `StepOrStop`.
    pub fn new(
        chain: Chain,
        task: String,
        handler: Box<dyn TaskProgressUpdateHandler>,
    ) -> Result<Self, Error> {
        let task_chain = chain.start_task(task)?;
        Ok(StepOrStop::Step {
            step: Step {
                task_chain,
                handler,
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
#[tracing::instrument]
pub async fn run_to_the_end(
    toolbox: tools::Toolbox,
    openai_client: Client,
    config: Config,
    task: String,
    handler: impl TaskProgressUpdateHandler + 'static,
) -> Result<Vec<TerminationMessage>, Error> {
    let toolbox = Rc::new(toolbox);

    let chain = Chain::new(toolbox, config.clone(), openai_client);

    let step_or_stop = StepOrStop::new(chain, task, Box::new(handler))?;

    let stop = step_or_stop.run(config.max_steps).await?;

    Ok(stop.termination_messages)
}
