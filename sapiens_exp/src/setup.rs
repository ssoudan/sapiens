use sapiens::tools::toolbox;
use sapiens_tools::conclude::ConcludeTool;
use sapiens_tools::python::PythonTool;

/// Create the toolbox with the tools for Sapiens Exp
///
/// Initially, the toolbox contains the PythonTool and the ConcludeTool.
/// Scenario builders like [`crate::tools::scenario_0::build`] will add
/// their tools to the toolbox.
pub async fn basic_toolbox() -> toolbox::Toolbox {
    let mut toolbox = toolbox::Toolbox::default();

    toolbox.add_advanced_tool(PythonTool::default()).await;
    toolbox.add_terminal_tool(ConcludeTool::default()).await;

    toolbox
}
