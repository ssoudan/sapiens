use std::cell::RefCell;
use std::fmt::Debug;

use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

use crate::tools::{TerminalTool, TerminationMessage};

/// A tool that is called to wrap the task.
#[derive(Default)]
pub struct ConcludeTool {
    done: RefCell<Option<ConcludeToolInput>>,
}

impl TerminalTool for ConcludeTool {
    fn is_done(&self) -> bool {
        self.done.borrow().is_some()
    }

    fn take_done(&self) -> Option<TerminationMessage> {
        self.done.borrow_mut().take().map(|input| input.into())
    }
}

/// A tool that is called to wrap the task.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConcludeToolInput {
    /// The final textual answer for this task.
    pub conclusion: String,
    /// The original question that was asked to the user.
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
#[derive(Serialize, Deserialize)]
pub struct ConcludeToolOutput {}

impl Describe for ConcludeToolInput {
    fn describe() -> Format {
        vec![(
            "conclusion",
            "The final textual answer for this task. No string interpolation supported. Plain text ONLY. MANDATORY.",
            )
            .into(),
            (
                "original_question",
                "The original question that was asked to the user. No string interpolation supported, only plain text. MANDATORY.",
            ).into(),]
        .into()
    }
}

impl Describe for ConcludeToolOutput {
    fn describe() -> Format {
        vec![].into()
    }
}

impl ConcludeTool {
    fn invoke_typed(&self, input: ConcludeToolInput) -> Result<ConcludeToolOutput, ToolUseError> {
        if self.done.borrow().is_some() {
            return Err(ToolUseError::ToolInvocationFailed(
                "This task is already done.".to_string(),
            ));
        }

        *self.done.borrow_mut() = Some(input);

        Ok(ConcludeToolOutput {})
    }
}

impl Tool for ConcludeTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "Conclude",
            "A tool to conclude a task.",
            "You have to use this to once you have the answer to the task with your conclusion.",
            ConcludeToolInput::describe(),
            ConcludeToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}
