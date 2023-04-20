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
use serde::{Deserialize, Serialize};

use crate::context::{ChatEntry, ChatHistory};
use crate::tools::{find_yaml, invoke_from_toolbox, TerminationMessage, ToolUseError, Toolbox};

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
    pub max_steps: u32,
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

/// Try to find the tool invocation from the chat message and invoke the
/// corresponding tool.
///
/// If multiple tool invocations are found, only the first one is used.
#[tracing::instrument]
pub fn invoke_tool(toolbox: Rc<Toolbox>, data: &str) -> (String, Result<String, ToolUseError>) {
    let tool_invocations = find_yaml::<ToolInvocationInput>(data);

    match tool_invocations {
        Ok(tool_invocations) => {
            if tool_invocations.is_empty() {
                return (
                    "unknown".to_string(),
                    Err(ToolUseError::ToolInvocationFailed(
                        "No Action found".to_string(),
                    )),
                );
            }

            // if any tool_invocations have an 'output' field, we return an error
            for invocation in tool_invocations.iter() {
                if invocation.output.is_some() {
                    return (
                        "unknown".to_string(),
                        Err(ToolUseError::ToolInvocationFailed(
                            "The Action cannot have an `output` field. Only `command` and `input` are allowed.".to_string(),
                        )),
                    );
                }
            }

            // Take the first invocation - the list is reversed
            let invocation_input = &tool_invocations.last().unwrap();

            let tool_name = invocation_input.command.clone();

            let input = invocation_input.input.clone();

            match invoke_from_toolbox(toolbox, &invocation_input.command, input) {
                Ok(o) => (tool_name, Ok(serde_yaml::to_string(&o).unwrap())),
                Err(e) => (tool_name, Err(e)),
            }
        }
        Err(e) => ("unknown".to_string(), Err(ToolUseError::InvalidYaml(e))),
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

/// Run a task with a set of tools
#[tracing::instrument]
pub async fn work(
    toolbox: Toolbox,
    openai_client: Client,
    config: Config,
    task: String,
    handler: impl TaskProgressUpdateHandler,
) -> Result<Vec<TerminationMessage>, Error> {
    let toolbox = Rc::new(toolbox);

    let chain = Chain::new(toolbox, config.clone(), openai_client);

    // Now we are ready to start the task
    let mut task_chain = chain.start_task(task).await.unwrap();

    // show the chat history
    handler.on_start(task_chain.chat_history());

    for _ in 1..config.max_steps {
        let chat_msg = task_chain.query_model().await?;

        // show the message from the assistant
        handler.on_tool_update(chat_msg.clone(), true);

        // pass the message to the tools and get the response
        let (tool_name, resp) = task_chain.invoke_tool(&chat_msg.msg);
        match resp {
            Ok(response) => {
                // check if the task is done
                if let Some(termination_messages) = task_chain.is_terminal() {
                    return Ok(termination_messages);
                }

                // Got a response from the tool but the task is not done yet
                match task_chain.on_tool_success(tool_name, response) {
                    Ok(e) => {
                        handler.on_tool_update(e, true);
                    }
                    Err(e) => {
                        handler.on_tool_error(e);
                    }
                }
            }
            Err(e) => {
                // check if the task is done
                if let Some(termination_messages) = task_chain.is_terminal() {
                    return Ok(termination_messages);
                }

                match task_chain.on_tool_failure(tool_name, e) {
                    Ok(e) => {
                        handler.on_tool_update(e, false);
                    }
                    Err(e) => {
                        handler.on_tool_error(e);
                    }
                }
            }
        }
    }

    Err(Error::MaxStepsReached)
}

#[derive(Serialize, Deserialize, Debug)]
struct ToolInvocationInput {
    command: String,
    input: serde_yaml::Value,
    output: Option<serde_yaml::Value>,
}
