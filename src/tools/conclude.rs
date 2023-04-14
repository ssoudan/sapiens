use std::fmt::Debug;

use colored::Colorize;
use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

/// A tool that is called to wrap the task.
pub struct ConcludeTool {}

impl ConcludeTool {
    pub fn new() -> Self {
        ConcludeTool {}
    }
}

impl Default for ConcludeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConcludeToolInput {
    conclusion: String,
    original_question: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConcludeToolOutput {}

impl Describe for ConcludeToolInput {
    fn describe() -> Format {
        vec![(
            "conclusion",
            "The final textual answer for this task. No string interpolation supported, only plain text. MANDATORY.",
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
    fn invoke_typed(&self, input: &ConcludeToolInput) -> Result<ConcludeToolOutput, ToolUseError> {
        println!(
            "The original question was: {} ",
            input.original_question.green()
        );
        println!("And the conclusion is: {} ", input.conclusion.blue());

        // TODO(ssoudan) lame
        std::process::exit(0);
    }
}

impl Tool for ConcludeTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "ConcludeTool",
            "A tool to terminate a task with a conclusion.",
            "Use this to terminate a task when it is complete.",
            ConcludeToolInput::describe(),
            ConcludeToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(&input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}
