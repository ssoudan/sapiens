/// OODA agents
pub mod ooda;

use crate::chains::Outcome;
use crate::context;
use crate::prompt::Task;
use crate::tools::ToolUseError;

/// Error from the agent
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to add to the chat history
    #[error("Failed to add to the chat history: {0}")]
    ChatHistoryError(#[from] context::Error),
    /// Error from the model
    #[error("Error from the model: {0}")]
    ModelError(#[from] crate::models::Error),
}

/// Format the outcome of a task
#[allow(clippy::ref_option)]
pub(crate) fn format_outcome(
    task: &Task,
    invocation_count: usize,
    tool_name: &Option<String>,
    outcome: &Outcome,
) -> String {
    /// Maximum number of characters in the response
    const MAX_RESPONSE_CHAR: usize = 2048;

    match outcome {
        Outcome::Success { result } => {
            let msg = Task::action_success_prompt(
                tool_name.clone().unwrap_or_else(|| "unknown".to_string()),
                invocation_count,
                result,
            );

            // if the response is too long, we add an error message to the chat
            // history instead
            if msg.len() > MAX_RESPONSE_CHAR {
                let msg = format!("The response is too long ({}B). Max allowed is {}B. Ask for a shorter response or use SandboxedPython Tool to process the response the data.",
                                      msg.len(), MAX_RESPONSE_CHAR);
                let e = ToolUseError::InvocationFailed(msg);
                let msg = Task::action_failed_prompt(
                    tool_name.clone().unwrap_or_else(|| "unknown".to_string()),
                    &e,
                );

                format!("{}\n{}", msg, task.to_prompt())
            } else {
                format!("{}\n{}", msg, task.to_prompt())
            }
        }
        Outcome::NoValidInvocationsFound { e } | Outcome::NoInvocationsFound { e } => {
            let msg = Task::invalid_action_prompt(e);
            format!("{}\n{}", msg, task.to_prompt())
        }
        Outcome::ToolUseError { e } => {
            let msg = Task::action_failed_prompt(
                tool_name.clone().unwrap_or_else(|| "unknown".to_string()),
                e,
            );
            format!("{}\n{}", msg, task.to_prompt())
        }
    }
}
