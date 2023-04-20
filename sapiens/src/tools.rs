use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub use llm_chain::parsing::find_yaml;
pub use llm_chain::tools::{Describe, Format, FormatPart, ToolDescription, ToolUseError};

/// Something meant to become a [`Tool`] - description
pub trait ProtoToolDescribe {
    /// the description of the tool
    fn description(&self) -> ToolDescription;
}

/// Something meant to become a [`Tool`] - invocation
pub trait ProtoToolInvoke {
    /// Invoke the tool
    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError>;
}

/// A Tool - the most basic kind of tools. See [`AdvancedTool`] and
/// [`TerminalTool`] for more.
pub trait Tool {
    /// the description of the tool
    fn description(&self) -> ToolDescription;

    /// Invoke the tool
    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError>;
}

impl<T> Tool for T
where
    T: ProtoToolDescribe + ProtoToolInvoke,
{
    fn description(&self) -> ToolDescription {
        self.description()
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        self.invoke(input)
    }
}

/// A termination message
///
/// This is the message that is sent to the user when a chain of exchanges
/// terminates.
pub struct TerminationMessage {
    /// The final textual answer for this task.
    pub conclusion: String,
    /// The original question that was asked to the user.
    pub original_question: String,
}

/// A [`Tool`] that wraps a chain of exchanges
pub trait TerminalTool: Tool {
    /// done flag.
    fn is_done(&self) -> bool {
        false
    }

    /// Take the done flag.
    fn take_done(&self) -> Option<TerminationMessage> {
        None
    }
}

/// A [`Tool`]  that can benefit from a [`Toolbox`]
pub trait AdvancedTool: Tool {
    /// Invoke the tool with a [`Toolbox`]
    fn invoke_with_toolbox(
        &self,
        toolbox: Rc<Toolbox>,
        input: serde_yaml::Value,
    ) -> Result<serde_yaml::Value, ToolUseError>;
}

/// Toolbox
///
/// a [`Toolbox`] is a collection of [`Tool`], [`TerminalTool`] and
/// [`AdvancedTool`].
#[derive(Default)]
pub struct Toolbox {
    /// The terminal tools - the one that can terminate a chain of exchanges
    terminal_tools: HashMap<String, Box<dyn TerminalTool>>,

    /// The tools - the other tools
    tools: HashMap<String, Box<dyn Tool>>,

    /// The advanced tools - the one that can invoke another tool (not an
    /// advanced one)
    advanced_tools: HashMap<String, Box<dyn AdvancedTool>>,
}

impl Debug for Toolbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toolbox")
            .field("terminal_tools", &self.terminal_tools.keys())
            .field("tools", &self.tools.keys())
            .field("advanced_tools", &self.advanced_tools.keys())
            .finish()
    }
}

impl Toolbox {
    /// Collect the termination messages
    pub fn termination_messages(&self) -> Vec<TerminationMessage> {
        let mut messages = Vec::new();

        for tool in self.terminal_tools.values() {
            if let Some(message) = tool.take_done() {
                messages.push(message);
            }
        }

        messages
    }

    /// Add a terminal tool
    ///
    /// A [`TerminalTool`] can terminate a chain of exchanges.
    pub fn add_terminal_tool(&mut self, tool: impl TerminalTool + 'static) {
        let name = tool.description().name;
        self.terminal_tools.insert(name, Box::new(tool));
    }

    /// Add a tool
    ///
    /// A [`Tool`] can be invoked by an [`AdvancedTool`].
    pub fn add_tool(&mut self, tool: impl Tool + 'static) {
        let name = tool.description().name;
        self.tools.insert(name, Box::new(tool));
    }

    /// Add an advanced tool
    ///
    /// An [`AdvancedTool`] is a [`Tool`] that can invoke another tool.
    pub fn add_advanced_tool(&mut self, tool: impl AdvancedTool + 'static) {
        let name = tool.description().name;
        self.advanced_tools.insert(name, Box::new(tool));
    }

    /// Get the descriptions of the tools
    pub fn describe(&self) -> HashMap<String, ToolDescription> {
        let mut descriptions = HashMap::new();

        for (name, tool) in self.terminal_tools.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        for (name, tool) in self.tools.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        for (name, tool) in self.advanced_tools.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        descriptions
    }
}

/// Invoke a [`Tool`] or [`AdvancedTool`]  from a [`Toolbox`]
pub fn invoke_from_toolbox(
    toolbox: Rc<Toolbox>,
    name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is an advanced tool
    if let Some(tool) = toolbox.clone().advanced_tools.get(name) {
        return tool.invoke_with_toolbox(toolbox, input);
    }

    // if not, test if the tool is a terminal tool
    if let Some(tool) = toolbox.terminal_tools.get(name) {
        return tool.invoke(input);
    }

    // otherwise, use the normal tool
    let tool = toolbox.tools.get(name).ok_or(ToolUseError::ToolNotFound)?;

    tool.invoke(input)
}

/// Invoke a Tool from a [`Toolbox`]
pub fn invoke_simple_from_toolbox(
    toolbox: Rc<Toolbox>,
    name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is a terminal tool
    if let Some(tool) = toolbox.terminal_tools.get(name) {
        return tool.invoke(input);
    }

    // the normal tool only
    let tool = toolbox.tools.get(name).ok_or(ToolUseError::ToolNotFound)?;

    tool.invoke(input)
}
