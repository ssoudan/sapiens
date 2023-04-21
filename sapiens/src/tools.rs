use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub use llm_chain::parsing::{find_yaml, ExtractionError};
pub use llm_chain::tools::{Describe, Format, FormatPart, ToolDescription};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Error while using a tool
#[derive(Debug, thiserror::Error)]
pub enum ToolUseError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    /// Tool invocation failed
    #[error("Tool invocation failed: {0}")]
    ToolInvocationFailed(String),
    /// Invalid YAML
    #[error("Invalid YAML")]
    InvalidYaml(#[from] serde_yaml::Error),
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(#[from] ExtractionError),
}

#[derive(Serialize, Deserialize, Debug)]
struct ToolInvocationInput {
    command: String,
    input: serde_yaml::Value,
    output: Option<serde_yaml::Value>,
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
    let guard = toolbox.terminal_tools.read().await;
    if let Some(tool) = guard.get(name) {
        return tool.invoke(input).await;
    }

    // otherwise, use the normal tool
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
    let guard = toolbox.terminal_tools.read().await;

    if let Some(tool) = guard.get(name) {
        return tool.invoke(input).await;
    }

    // the normal tool only
    let tool = guard
        .get(name)
        .ok_or(ToolUseError::ToolNotFound(name.to_string()))?;

    tool.invoke(input).await
}

/// Try to find the tool invocation from the chat message and invoke the
/// corresponding tool.
///
/// If multiple tool invocations are found, only the first one is used.
#[tracing::instrument]
pub async fn invoke_tool(toolbox: Toolbox, data: &str) -> (String, Result<String, ToolUseError>) {
    let tool_invocations = find_yaml::<ToolInvocationInput>(data);

    match tool_invocations {
        Ok(tool_invocations) => {
            if tool_invocations.is_empty() {
                return (
                    "unknown".to_string(),
                    Err(ToolUseError::ToolInvocationFailed(
                        "No Action found".to_string(),
                    )),
                );
            }

            // if any tool_invocations have an 'output' field, we return an error
            for invocation in tool_invocations.iter() {
                if invocation.output.is_some() {
                    return (
                        "unknown".to_string(),
                        Err(ToolUseError::ToolInvocationFailed(
                            "The Action cannot have an `output` field. Only `command` and `input` are allowed.".to_string(),
                        )),
                    );
                }
            }

            // Take the first invocation - the list is reversed
            let invocation_input = &tool_invocations.last().unwrap();

            let tool_name = invocation_input.command.clone();

            let input = invocation_input.input.clone();

            match invoke_from_toolbox(toolbox, &invocation_input.command, input).await {
                Ok(o) => (tool_name, Ok(serde_yaml::to_string(&o).unwrap())),
                Err(e) => (tool_name, Err(e)),
            }
        }
        Err(e) => ("unknown".to_string(), Err(ToolUseError::InvalidInput(e))),
    }
}
