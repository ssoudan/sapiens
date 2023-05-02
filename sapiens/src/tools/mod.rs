use std::collections::HashMap;
use std::fmt::Debug;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use toolbox::Toolbox;
use tracing::warn;

use crate::tools::invocation::InvocationError;

/// Tools to extract Tool invocations from a messages
pub mod invocation;

/// Collection of tools
pub mod toolbox;

/// Part of a [`Format`]
#[derive(Debug, Clone)]
pub struct FieldFormat {
    /// Name of the field
    pub name: String,
    /// Type of the field
    pub r#type: String,
    /// True if the field is optional
    pub optional: bool,
    /// Description of the field
    pub description: String,
}

/// Input or output format of a tool
pub trait Describe {
    /// Describe the format
    fn describe() -> Format;
}

/// Format of [`Tools`] input and output
#[derive(Debug, Clone)]
pub struct Format {
    /// Fields of the format
    pub fields: Vec<FieldFormat>,
}

impl Serialize for Format {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let n = self.fields.len();
        let mut map = serializer.serialize_map(Some(n))?;
        for field in &self.fields {
            let description = if field.optional {
                format!("<{}> {} (optional)", field.r#type, field.description)
            } else {
                format!("<{}> {}", field.r#type, field.description)
            };
            map.serialize_entry(&field.name, &description)?;
        }
        map.end()
    }
}

impl From<Vec<FieldFormat>> for Format {
    fn from(fields: Vec<FieldFormat>) -> Self {
        Format { fields }
    }
}

/// Tool description
#[derive(Debug, Serialize, Clone)]
pub struct ToolDescription {
    /// Name of the tool
    pub name: String,
    /// Description of the tool
    pub description: String,
    /// Input format
    pub input_format: Format,
    /// Output format
    pub output_format: Format,
}

impl ToolDescription {
    /// Create a new tool description
    pub fn new(name: &str, description: &str, input_format: Format, output_format: Format) -> Self {
        ToolDescription {
            name: name.to_string(),
            description: description.to_string(),
            input_format,
            output_format,
        }
    }
}

/// Error while using a tool
#[derive(Debug, thiserror::Error, Clone)]
pub enum ToolUseError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    /// Tool invocation failed
    #[error("Tool invocation failed: {0}")]
    InvocationFailed(String),
    /// Failed to serialize the output
    #[error("Failed to serialize the output: {0}")]
    InvalidOutput(String),
    /// Failed to deserialize the input
    #[error("Failed to deserialize the input: {0}")]
    InvalidInput(String),
    /// Invalid input
    #[error("Invalid invocation: {0}")]
    InvalidInvocation(#[from] InvocationError),
    /// Too many invocation found
    #[error("Too many invocation found")]
    TooManyInvocationFound,
    /// No action found
    #[error("No action found")]
    NoActionFound,
}

/// A tool invocation input
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ToolInvocationInput {
    /// The tool to invoke
    tool_name: String,
    // FUTURE(ssoudan) should this be flattened?
    // FUTURE(ssoudan) should this be called `spec` or `argumens` or `parameters`?
    /// The input to the tool
    input: serde_yaml::Value,
    /// The junk
    #[serde(skip_serializing_if = "HashMap::is_empty", flatten)]
    junk: HashMap<String, serde_yaml::Value>,
}

/// Something meant to become a [`Tool`] - description
pub trait ProtoToolDescribe {
    /// the description of the tool
    fn description(&self) -> ToolDescription;
}

/// Something meant to become a [`Tool`] - invocation
#[async_trait::async_trait]
pub trait ProtoToolInvoke {
    /// Invoke the tool
    async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError>;
}

/// A Tool - the most basic kind of tools. See [`AdvancedTool`] and
/// [`TerminalTool`] for more.
#[async_trait::async_trait]
pub trait Tool: Sync + Send {
    /// the description of the tool
    fn description(&self) -> ToolDescription;

    /// Invoke the tool
    // FUTURE(ssoudan) Box<Deserialize>?
    async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError>;
}

#[async_trait::async_trait]
impl<T: Sync + Send> Tool for T
where
    T: ProtoToolDescribe + ProtoToolInvoke,
{
    fn description(&self) -> ToolDescription {
        self.description()
    }

    async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        self.invoke(input).await
    }
}

/// A termination message
///
/// This is the message that is sent to the user when a chain of exchanges
/// terminates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationMessage {
    /// The final textual answer for this task.
    pub conclusion: String,
    /// The original question that was asked to the user.
    pub original_question: String,
}

/// A [`Tool`] that wraps a chain of exchanges
#[async_trait::async_trait]
pub trait TerminalTool: Tool + Sync + Send {
    /// done flag.
    async fn is_done(&self) -> bool {
        false
    }

    /// Take the done flag.
    async fn take_done(&self) -> Option<TerminationMessage> {
        None
    }
}

/// A [`Tool`]  that can benefit from a [`Toolbox`]
#[async_trait::async_trait]
pub trait AdvancedTool: Tool {
    /// Invoke the tool with a [`Toolbox`]
    async fn invoke_with_toolbox(
        &self,
        toolbox: Toolbox,
        input: serde_yaml::Value,
    ) -> Result<serde_yaml::Value, ToolUseError>;
}

async fn choose_invocation(
    tool_invocations: Vec<ToolInvocationInput>,
) -> Result<ToolInvocationInput, InvocationError> {
    // if no tool_invocations are found, we return an error
    if tool_invocations.is_empty() {
        return Err(InvocationError::NoInvocationFound);
    }

    // We just take the first one
    let mut invocation = tool_invocations.into_iter().next().unwrap();

    // FUTURE(ssoudan) clean up the object and return this one instead
    // if any tool_invocations have an 'output' field, we return an error
    if !invocation.junk.is_empty() {
        let junk_keys = invocation
            .junk
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");

        // FUTURE(ssoudan) they should not reach the ChatHistory
        warn!(
            ?junk_keys,
            "The Action should not have fields: {}.", junk_keys
        );

        // We just remove them for now
        invocation.junk.clear();

        // return Err(InvocationError::NoValidInvocationFound(format!(
        //     "The Action cannot have fields: {}. Only `command` and `input`
        // are allowed.",     junk_keys
        // )));
    }

    Ok(invocation)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use insta::assert_display_snapshot;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct FakeToolInput {
        q: String,
        excluded_terms: Option<String>,
        num_results: Option<u32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct FakeToolOutput {
        items: Vec<String>,
    }

    #[tokio::test]
    async fn test_serializing_tool_invocation() {
        let input = FakeToolInput {
            q: "Marcel Deneuve".to_string(),
            excluded_terms: Some("Resident Evil".to_string()),
            num_results: Some(10),
        };

        let output = FakeToolOutput {
        items: vec![
            "Marcel Deneuve is a character in the Resident Evil film series,".to_string(), 
            "playing a minor role in Resident Evil: Apocalypse and a much larger".to_string(),
            " role in Resident Evil: Extinction. Explore historical records and ".to_string(),
            "family tree profiles about Marcel Deneuve on MyHeritage, the world's largest family network.".to_string()
        ]

        };

        let junk = vec![("output".to_string(), serde_yaml::to_value(output).unwrap())];

        let invocation = super::ToolInvocationInput {
            tool_name: "Search".to_string(),
            input: serde_yaml::to_value(input).unwrap(),
            junk: HashMap::from_iter(junk.into_iter()),
        };

        let serialized = serde_yaml::to_string(&invocation).unwrap();

        assert_display_snapshot!(serialized);
    }
}
