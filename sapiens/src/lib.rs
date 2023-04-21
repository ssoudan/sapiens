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

use runner::Chain;

use crate::context::{ChatEntry, ChatHistory};
use crate::openai::{Client, OpenAIError};
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
pub trait TaskProgressUpdateHandler: Send {
    // FIXME(ssoudan) async trait
    /// Called when the task starts
    fn on_start(&self, _chat_history: &ChatHistory) {}

    /// Called when the model updates the chat history
    fn on_model_update(&self, _model_message: ChatEntry) {}

    /// Called when the tool updates the chat history
    fn on_tool_update(&self, _tool_output: ChatEntry, _success: bool) {}

    /// Called when the tool returns an error
    fn on_tool_error(&self, _error: Error) {}
}

/// A step in the task
pub struct Step {
    task_chain: TaskChain,
    handler: Option<Box<dyn TaskProgressUpdateHandler>>,
}

impl Step {
    /// Run the task for a single step
    async fn step(mut self) -> Result<StepOrStop, Error> {
        let model_message = self.task_chain.query_model().await?;

        // show the message from the assistant
        if let Some(handler) = &self.handler {
            handler.on_model_update(model_message.clone());
        }

        // pass the message to the tools and get the response
        let (tool_name, resp) = self.task_chain.invoke_tool(&model_message.msg).await;
        match resp {
            Ok(response) => {
                // check if the task is done
                if let Some(termination_messages) = self.task_chain.is_terminal().await {
                    return Ok(StepOrStop::Stop {
                        stop: Stop {
                            termination_messages,
                        },
                    });
                }

                // Got a response from the tool but the task is not done yet
                match self.task_chain.on_tool_success(tool_name, response) {
                    Ok(tool_output) => {
                        if let Some(handler) = &self.handler {
                            handler.on_tool_update(tool_output, true);
                        }
                    }
                    Err(e) => {
                        if let Some(handler) = &self.handler {
                            handler.on_tool_error(e);
                        }
                    }
                }
            }
            Err(e) => {
                // check if the task is done
                if let Some(termination_messages) = self.task_chain.is_terminal().await {
                    return Ok(StepOrStop::Stop {
                        stop: Stop {
                            termination_messages,
                        },
                    });
                }

                match self.task_chain.on_tool_failure(tool_name, e) {
                    Ok(tool_output) => {
                        if let Some(handler) = &self.handler {
                            handler.on_tool_update(tool_output, false);
                        }
                    }
                    Err(e) => {
                        if let Some(handler) = &self.handler {
                            handler.on_tool_error(e);
                        }
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
    /// Create a new [`StepOrStop`] for a `task`.
    ///
    /// The `handler` will be called when the task starts and when a step is
    /// completed - either successfully or not. The `handler` will be called
    /// with the latest chat history element. It is also called on error.
    pub fn with_handler(
        chain: Chain,
        task: String,
        handler: Box<dyn TaskProgressUpdateHandler>,
    ) -> Result<Self, Error> {
        let task_chain = chain.start_task(task)?;

        // call the handler
        handler.on_start(task_chain.chat_history());

        Ok(StepOrStop::Step {
            step: Step {
                task_chain,
                handler: Some(handler),
            },
        })
    }

    /// Create a new [`StepOrStop`] for a `task`.
    pub fn new(chain: Chain, task: String) -> Result<Self, Error> {
        let task_chain = chain.start_task(task)?;

        Ok(StepOrStop::Step {
            step: Step {
                task_chain,
                handler: None,
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
    handler: impl TaskProgressUpdateHandler + 'static + Debug,
) -> Result<Vec<TerminationMessage>, Error> {
    let chain = Chain::new(toolbox, config.clone(), openai_client).await;

    let step_or_stop = StepOrStop::with_handler(chain, task, Box::new(handler))?;

    let stop = step_or_stop.run(config.max_steps).await?;

    Ok(stop.termination_messages)
}
