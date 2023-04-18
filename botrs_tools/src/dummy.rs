use std::fmt::Debug;

use botrs_derive::Describe;
use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

/// A tool that is called to test stuffs
#[derive(Default)]
pub struct DummyTool {}

/// A tool that is called to test stuffs
#[derive(Debug, Serialize, Deserialize, Describe)]
pub struct DummyToolInput {
    /// Well. MANDATORY.
    pub blah: String,
}

/// DummyToolOutput not very significant
#[derive(Serialize, Deserialize, Describe)]
pub struct DummyToolOutput {
    /// Not much.
    pub something: String,
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
