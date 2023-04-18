use std::fmt::Debug;

use sapiens::tools::{
    Describe, Format, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError,
};
use sapiens_derive::{Describe, ProtoToolDescribe};
use serde::{Deserialize, Serialize};

/// A tool that is called to test stuffs
#[derive(Default, ProtoToolDescribe)]
#[tool(name = "Dummy", input = "DummyToolInput", output = "DummyToolOutput")]
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

impl ProtoToolInvoke for DummyTool {
    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(&input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}
