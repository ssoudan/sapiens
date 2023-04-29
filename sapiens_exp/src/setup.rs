use sapiens::tools;
use sapiens_tools::conclude::ConcludeTool;
use sapiens_tools::python::PythonTool;

/// Create the toolbox with the tools for Sapiens Exp
pub async fn toolbox() -> tools::Toolbox {
    let mut toolbox = tools::Toolbox::default();

    toolbox.add_advanced_tool(PythonTool::default()).await;
    toolbox.add_terminal_tool(ConcludeTool::default()).await;

    // TODO(ssoudan) add the other tools

    toolbox
}
