use std::fmt::Debug;

use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

/// A tool that is called to test stuffs
pub struct DummyTool {}

impl Default for DummyTool {
    fn default() -> Self {
        Self {}
    }
}

/// A tool that is called to test stuffs
#[derive(Debug, Serialize, Deserialize)]
pub struct DummyToolInput {
    /// Well. MANDATORY.
    pub blah: String,
}

/// DummyToolOutput not very significant
#[derive(Serialize, Deserialize)]
pub struct DummyToolOutput {
    /// No much.
    pub something: String,
}

impl Describe for DummyToolInput {
    fn describe() -> Format {
        vec![("blah", "Well. MANDATORY.").into()].into()
    }
}

impl Describe for DummyToolOutput {
    fn describe() -> Format {
        vec![("something", "No much.").into()].into()
    }
}

impl DummyTool {
    fn invoke_typed(&self, input: &DummyToolInput) -> Result<DummyToolOutput, ToolUseError> {
        Ok(DummyToolOutput {
            something: input.blah.clone() + " and something else",
        })
    }
}

impl Tool for DummyTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "Dummy",
            "A tool to test stuffs.",
            "Use this to test stuffs.",
            DummyToolInput::describe(),
            DummyToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(&input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}
