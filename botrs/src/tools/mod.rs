use std::collections::HashMap;
use std::rc::Rc;

use llm_chain::tools::{Tool, ToolDescription, ToolUseError};

/// Hue tools
pub mod hue;

/// Tool to conclude a chain
pub(crate) mod conclude;

/// Tool to run some (limited) python
pub(crate) mod python;

#[cfg(test)]
/// Tool to test stuffs
pub(crate) mod dummy;

/// A tool that can benefit from a toolbox
pub trait AdvancedTool: Tool {
    /// Invoke the tool with a toolbox
    fn invoke_with_toolbox(
        &self,
        toolbox: Rc<Toolbox>,
        input: serde_yaml::Value,
    ) -> Result<serde_yaml::Value, ToolUseError>;
}

/// Toolbox
#[derive(Default)]
pub struct Toolbox {
    /// The tools
    tools: HashMap<String, Box<dyn Tool>>,

    /// The advanced tools - the one that can invoke another tool (not an
    /// advanced one)
    advanced_tools: HashMap<String, Box<dyn AdvancedTool>>,
}

impl Toolbox {
    /// Add a tool
    pub fn add_tool(&mut self, tool: impl Tool + 'static) {
        let name = tool.description().name;
        self.tools.insert(name, Box::new(tool));
    }

    /// Add an advanced tool
    pub fn add_advanced_tool(&mut self, tool: impl AdvancedTool + 'static) {
        let name = tool.description().name;
        self.advanced_tools.insert(name, Box::new(tool));
    }

    /// Get the descriptions
    pub fn describe(&self) -> HashMap<String, ToolDescription> {
        let mut descriptions = HashMap::new();

        for (name, tool) in self.tools.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        for (name, tool) in self.advanced_tools.iter() {
            descriptions.insert(name.clone(), tool.description());
        }

        descriptions
    }
}

/// Invoke a Tool or AdvancedTool  from a toolbox
pub fn invoke_from_toolbox(
    toolbox: Rc<Toolbox>,
    name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // test if the tool is an advanced tool
    if let Some(tool) = toolbox.clone().advanced_tools.get(name) {
        return tool.invoke_with_toolbox(toolbox, input);
    }

    // otherwise, use the normal tool
    let tool = toolbox.tools.get(name).ok_or(ToolUseError::ToolNotFound)?;

    tool.invoke(input)
}

/// Invoke a Tool from a toolbox
pub fn invoke_simple_from_toolbox(
    toolbox: Rc<Toolbox>,
    name: &str,
    input: serde_yaml::Value,
) -> Result<serde_yaml::Value, ToolUseError> {
    // the normal tool only
    let tool = toolbox.tools.get(name).ok_or(ToolUseError::ToolNotFound)?;

    tool.invoke(input)
}
