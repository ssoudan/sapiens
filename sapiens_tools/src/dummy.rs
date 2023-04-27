use std::fmt::Debug;

use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};

/// A tool that is called to test stuffs
#[derive(Debug, Default, ProtoToolDescribe, ProtoToolInvoke)]
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
    #[tracing::instrument(skip(self))]
    async fn invoke_typed(&self, input: &DummyToolInput) -> Result<DummyToolOutput, ToolUseError> {
        Ok(DummyToolOutput {
            something: input.blah.clone() + " and something else",
        })
    }
}
