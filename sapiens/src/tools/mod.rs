use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::tools::invocation::InvocationError;

/// Tools to extract Tool invocations from a messages
pub mod invocation;

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
#[derive(Debug, thiserror::Error)]
pub enum ToolUseError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    /// Tool invocation failed
    #[error("Tool invocation failed: {0}")]
    InvocationFailed(String),
    /// Failed to serialize the output
    #[error("Failed to serialize the output: {0}")]
    InvalidOutput(#[from] serde_yaml::Error),
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(#[from] InvocationError),
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

/// Toolbox
///
/// a [`Toolbox`] is a collection of [`Tool`], [`TerminalTool`] and
/// [`AdvancedTool`].
#[derive(Default, Clone)]
pub struct Toolbox {
    /// The terminal tools - the one that can terminate a chain of exchanges
    terminal_tools: Arc<RwLock<HashMap<String, Box<dyn TerminalTool>>>>,

    /// The tools - the other tools
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,

    /// The advanced tools - the one that can invoke another tool (not an
    /// advanced one)
    advanced_tools: Arc<RwLock<HashMap<String, Box<dyn AdvancedTool>>>>,
}

impl Debug for Toolbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toolbox").finish()
    }
}

impl Toolbox {
    /// Collect the termination messages
    pub async fn termination_messages(&self) -> Vec<TerminationMessage> {
        let mut messages = Vec::new();

        for tool in self.terminal_tools.read().await.values() {
            if let Some(message) = tool.take_done().await {
                messages.push(message);
            }
        }

        messages
    }

    /// Add a terminal tool
    ///
    /// A [`TerminalTool`] can terminate a chain of exchanges.
    pub async fn add_terminal_tool(&mut self, tool: impl TerminalTool + 'static) {
        let name = tool.description().name;
        self.terminal_tools
            .write()
            .await
            .insert(name, Box::new(tool));
    }

    /// Add a tool
    ///
    /// A [`Tool`] can be invoked by an [`AdvancedTool`].
    pub async fn add_tool(&mut self, tool: impl Tool + 'static) {
        let name = tool.description().name;
        self.tools.write().await.insert(name, Box::new(tool));
    }

    /// Add an advanced tool
    ///
    /// An [`AdvancedTool`] is a [`Tool`] that can invoke another tool.
    pub async fn add_advanced_tool(&mut self, tool: impl AdvancedTool + 'static) {
        let name = tool.description().name;
        self.advanced_tools
            .write()
            .await
            .insert(name, Box::new(tool));
    }

    /// Get the descriptions of the tools
    pub async fn describe(&self) -> HashMap<String, ToolDescription> {
        let mut descriptions = HashMap::new();

        for (name, tool) in self.terminal_tools.read().await.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        for (name, tool) in self.tools.read().await.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        for (name, tool) in self.advanced_tools.read().await.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        descriptions
    }
}

/// Invoke a [`Tool`] or [`AdvancedTool`]  from a [`Toolbox`]
pub async fn invoke_from_toolbox(
    toolbox: Toolbox,
    name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is an advanced tool
    if let Some(tool) = toolbox.clone().advanced_tools.read().await.get(name) {
        return tool.invoke_with_toolbox(toolbox, input).await;
    }

    // if not, test if the tool is a terminal tool
    {
        let guard = toolbox.terminal_tools.read().await;
        if let Some(tool) = guard.get(name) {
            return tool.invoke(input).await;
        }
    }

    // otherwise, use the normal tool
    let guard = toolbox.tools.read().await;
    let tool = guard
        .get(name)
        .ok_or(ToolUseError::ToolNotFound(name.to_string()))?;

    tool.invoke(input).await
}

/// Invoke a Tool from a [`Toolbox`]
pub async fn invoke_simple_from_toolbox(
    toolbox: Toolbox,
    name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is a terminal tool
    {
        let guard = toolbox.terminal_tools.read().await;
        if let Some(tool) = guard.get(name) {
            return tool.invoke(input).await;
        }
    }

    // the normal tool only
    let guard = toolbox.tools.read().await;
    let tool = guard
        .get(name)
        .ok_or(ToolUseError::ToolNotFound(name.to_string()))?;

    tool.invoke(input).await
}

/// Try to find the tool invocation from the chat message and invoke the
/// corresponding tool.
///
/// If multiple tool invocations are found, only the first one is used.
#[tracing::instrument(skip(toolbox, data))]
pub async fn invoke_tool(toolbox: Toolbox, data: &str) -> (String, Result<String, ToolUseError>) {
    let invocation = choose_invocation(data).await;

    match invocation {
        Ok(invocation) => {
            debug!(tool_name = invocation.tool_name, "Invocation found");

            let tool_name = invocation.tool_name.clone();
            let input = invocation.input;
            let result = invoke_from_toolbox(toolbox, &tool_name, input).await;

            match result {
                Ok(output) => (
                    tool_name,
                    serde_yaml::to_string(&output).map_err(ToolUseError::InvalidOutput),
                ),
                Err(e) => (tool_name, Err(e)),
            }
        }
        Err(e) => ("unknown".to_string(), Err(e)),
    }
}

async fn choose_invocation(data: &str) -> Result<ToolInvocationInput, ToolUseError> {
    match invocation::find_all(data) {
        Ok(tool_invocations) => {
            info!("{} Tool invocations found", tool_invocations.len());

            // if no tool_invocations are found, we return an error
            if tool_invocations.is_empty() {
                return Err(ToolUseError::NoActionFound);
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

                // return Err(ToolUseError::InvocationFailed(format!(
                //     "The Action cannot have fields: {}. Only `command` and
                // `input` are allowed.",     junk_keys
                // )));
            }

            Ok(invocation)
        }
        Err(e) => Err(ToolUseError::InvalidInput(e)),
    }
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
