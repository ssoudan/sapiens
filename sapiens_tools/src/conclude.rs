use std::fmt::Debug;

use sapiens::tools::{
    Describe, ProtoToolDescribe, ProtoToolInvoke, TerminalTool, TerminationMessage,
    ToolDescription, ToolUseError,
};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// A tool to conclude a task.
/// You have to use this to once you have the answer to the task with your
/// conclusion.
#[derive(Default, ProtoToolDescribe, ProtoToolInvoke)]
#[tool(
    name = "Conclude",
    input = "ConcludeToolInput",
    output = "ConcludeToolOutput"
)]
pub struct ConcludeTool {
    done: Mutex<Option<ConcludeToolInput>>,
}

impl Debug for ConcludeTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcludeTool").finish()
    }
}

#[async_trait::async_trait]
impl TerminalTool for ConcludeTool {
    async fn is_done(&self) -> bool {
        // lock
        let done = self.done.lock().await;
        done.is_some()
    }

    async fn take_done(&self) -> Option<TerminationMessage> {
        // lock
        {
            let mut done = self.done.lock().await;
            done.take().map(|input| input.into())
        }
    }
}

/// A tool that is called to wrap the task.
#[derive(Debug, Clone, Serialize, Deserialize, Describe)]
pub struct ConcludeToolInput {
    /// The final answer for this task. Plain text ONLY. No string interpolation
    /// supported. MANDATORY. Call directly from `SandboxPython` Tool for long
    /// answers.
    pub conclusion: String,
    /// The original question that was asked to the user. No string
    /// interpolation supported, only plain text. MANDATORY.
    pub original_question: String,
}

impl From<ConcludeToolInput> for TerminationMessage {
    fn from(input: ConcludeToolInput) -> Self {
        Self {
            conclusion: input.conclusion,
            original_question: input.original_question,
        }
    }
}

/// ConcludeToolOutput - empty
#[derive(Serialize, Deserialize, Describe)]
pub struct ConcludeToolOutput {}

impl ConcludeTool {
    #[tracing::instrument(skip(self))]
    async fn invoke_typed(
        &self,
        input: &ConcludeToolInput,
    ) -> Result<ConcludeToolOutput, ToolUseError> {
        // lock
        {
            let mut done = self.done.lock().await;

            if done.is_some() {
                return Err(ToolUseError::InvocationFailed(
                    "This task is already done.".to_string(),
                ));
            }

            // set done
            *done = Some(input.clone());
        }

        Ok(ConcludeToolOutput {})
    }
}
