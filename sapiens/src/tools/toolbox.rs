use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::tools;
use crate::tools::invocation::Error;
use crate::tools::{
    AdvancedTool, TerminalTool, TerminationMessage, Tool, ToolDescription, ToolUseError,
};

/// Tool usage statistics
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    /// Number of times the tool has been invoked successfully
    pub success_count: HashMap<String, usize>,
    /// Number of times the tool has been invoked with an error
    pub error_count: HashMap<String, usize>,
    /// Number of times an inexistent tool has been invoked
    pub inexistent_count: HashMap<String, usize>,
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

    /// The tool usage statistics
    stats: Arc<RwLock<Stats>>,
}

impl Debug for Toolbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toolbox").finish()
    }
}

impl Toolbox {
    /// Collect the termination messages
    #[allow(clippy::significant_drop_tightening)]
    #[allow(clippy::significant_drop_in_scrutinee)]
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
    pub async fn add_terminal_tool(&self, tool: impl TerminalTool + 'static) {
        let name = tool.description().name;
        self.terminal_tools
            .write()
            .await
            .insert(name, Box::new(tool));
    }

    /// Check if the toolbox has at least one terminal tool
    pub async fn has_terminal_tools(&self) -> bool {
        !self.terminal_tools.read().await.is_empty()
    }

    /// Add a tool
    ///
    /// A [`Tool`] can be invoked by an [`AdvancedTool`].
    pub async fn add_tool(&self, tool: impl Tool + 'static) {
        let name = tool.description().name;
        self.tools.write().await.insert(name, Box::new(tool));
    }

    /// Add an advanced tool
    ///
    /// An [`AdvancedTool`] is a [`Tool`] that can invoke another tool.
    pub async fn add_advanced_tool(&self, tool: impl AdvancedTool + 'static) {
        let name = tool.description().name;
        self.advanced_tools
            .write()
            .await
            .insert(name, Box::new(tool));
    }

    /// Get the descriptions of the tools
    #[allow(clippy::significant_drop_tightening)]
    #[allow(clippy::significant_drop_in_scrutinee)]
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

    /// Reset stats
    pub async fn reset_stats(&self) {
        *self.stats.write().await = Stats::default();
    }

    /// Get the stats
    pub async fn stats(&self) -> Stats {
        self.stats.read().await.clone()
    }

    /// Report a successful invocation
    pub async fn report_success(&self, tool_name: &str) {
        let mut stats = self.stats.write().await;
        stats
            .success_count
            .entry(tool_name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    /// Report a failed invocation
    pub async fn report_error(&self, tool_name: &str) {
        let mut stats = self.stats.write().await;
        stats
            .error_count
            .entry(tool_name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    /// Report an inexistent tool invocation
    pub async fn report_inexistent(&self, tool_name: &str) {
        let mut stats = self.stats.write().await;
        stats
            .inexistent_count
            .entry(tool_name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }
}

/// Invoke a [`Tool`] or [`AdvancedTool`] or [`TerminalTool`] from a [`Toolbox`]
#[allow(clippy::significant_drop_tightening)]
#[allow(clippy::significant_drop_in_scrutinee)]
async fn invoke_from_toolbox(
    toolbox: Toolbox,
    tool_name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is an advanced tool
    if let Some(tool) = toolbox.clone().advanced_tools.read().await.get(tool_name) {
        let result = tool.invoke_with_toolbox(toolbox.clone(), input).await;

        if result.is_ok() {
            toolbox.report_success(tool_name).await;
        } else {
            toolbox.report_error(tool_name).await;
        }

        return result;
    }

    // if not, test if the tool is a terminal tool
    {
        let guard = toolbox.terminal_tools.read().await;
        if let Some(tool) = guard.get(tool_name) {
            let result = tool.invoke(input).await;
            if result.is_ok() {
                toolbox.report_success(tool_name).await;
            } else {
                toolbox.report_error(tool_name).await;
            }

            return result;
        }
    }

    // otherwise, use the normal tool
    let guard = toolbox.tools.read().await;
    let tool = guard.get(tool_name);

    if tool.is_none() {
        toolbox.report_inexistent(tool_name).await;
    }

    let tool = tool.ok_or_else(|| ToolUseError::ToolNotFound(tool_name.to_string()))?;

    let result = tool.invoke(input).await;
    if result.is_ok() {
        toolbox.report_success(tool_name).await;
    } else {
        toolbox.report_error(tool_name).await;
    }
    result
}

/// Invoke a [`Tool`] or [`TerminalTool`] from a [`Toolbox`].
///
/// This function is intended to be used by [`AdvancedTool`]s.
/// It will not invoke another [`AdvancedTool`].
///
/// If you want to invoke an [`AdvancedTool`], use [`invoke_tool`].
#[allow(clippy::significant_drop_tightening)]
#[allow(clippy::module_name_repetitions)]
pub async fn invoke_simple_from_toolbox(
    toolbox: Toolbox,
    tool_name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is a terminal tool
    {
        let guard = toolbox.terminal_tools.read().await;
        if let Some(tool) = guard.get(tool_name) {
            let result = tool.invoke(input).await;
            if result.is_ok() {
                toolbox.report_success(tool_name).await;
            } else {
                toolbox.report_error(tool_name).await;
            }

            return result;
        }
    }

    // the normal tool only
    let guard = toolbox.tools.read().await;
    let tool = guard.get(tool_name);

    if tool.is_none() {
        toolbox.report_inexistent(tool_name).await;
    }

    let tool = tool.ok_or_else(|| ToolUseError::ToolNotFound(tool_name.to_string()))?;

    let result = tool.invoke(input).await;
    if result.is_ok() {
        toolbox.report_success(tool_name).await;
    } else {
        toolbox.report_error(tool_name).await;
    }
    result
}

/// Result of invoking a tool with [`invoke_tool`].
#[derive(Debug, Clone)]
pub enum InvokeResult {
    /// No invocation found in the message
    NoInvocationsFound {
        /// The error that occurred
        e: Error,
    },
    /// No valid invocation found in the message
    NoValidInvocationsFound {
        /// The error that occurred
        e: Error,
        /// The number of invocations found in the message
        invocation_count: usize,
    },
    /// Successful invocation
    Success {
        /// The number of invocations found in the message
        invocation_count: usize,
        /// The name of the tool that was invoked
        tool_name: String,
        /// The extracted input for the tool
        extracted_input: String,
        /// The result of the invocation
        result: String,
    },
    /// Error during invocation
    Error {
        /// The number of invocations found in the message
        invocation_count: usize,
        /// The name of the tool that was invoked
        tool_name: String,
        /// The extracted input for the tool
        extracted_input: String,
        /// The error that occurred
        e: ToolUseError,
    },
}

/// Try to find the tool invocation from the chat message and invoke the
/// corresponding tool.
///
/// If multiple tool invocations are found, only the first one is used.
#[tracing::instrument(skip(toolbox, data))]
pub async fn invoke_tool(toolbox: Toolbox, data: &str) -> InvokeResult {
    let tool_invocations = match tools::invocation::find_all(data) {
        Ok(invocations) => invocations,
        Err(e) => return InvokeResult::NoInvocationsFound { e },
    };
    let invocation_count = tool_invocations.invocations.len();
    info!(
        "{} YAML blocks and {} Tool invocations found",
        tool_invocations.yaml_block_count, invocation_count
    );

    // FUTURE(ssoudan) feature to control this
    // if more than one tool_invocations are found, we return an error
    // if tool_invocations.len() > 1 {
    //     return Err(ToolUseError::TooManyInvocationFound);
    // }

    // FUTURE(ssoudan) invoke corresponding tools one by one. Fail on first error.
    // FUTURE(ssoudan) document this in the initial prompt

    let invocation = match tools::choose_invocation(tool_invocations) {
        Ok(invocation) => invocation,
        Err(e) => {
            return InvokeResult::NoValidInvocationsFound {
                e,
                invocation_count,
            }
        }
    };

    // We found an invocation, let's invoke the tool
    debug!(tool_name = invocation.tool_name, "Invocation found");

    let tool_name = invocation.tool_name.clone();
    let input = invocation.parameters;

    let extracted_input = serde_yaml::to_string(&input).unwrap_or_else(|_| {
        format!(
            "Failed to serialize input for tool {}",
            invocation.tool_name
        )
    });

    let result = invoke_from_toolbox(toolbox, &tool_name, input.clone()).await;

    match result {
        Ok(output) => {
            let result = serde_yaml::to_string(&output).unwrap_or_else(|_| {
                format!(
                    "Failed to serialize output for tool {}",
                    invocation.tool_name
                )
            });

            InvokeResult::Success {
                tool_name,
                extracted_input,
                invocation_count,
                result,
            }
        }
        Err(e) => InvokeResult::Error {
            tool_name,
            extracted_input,
            invocation_count,
            e,
        },
    }
}
