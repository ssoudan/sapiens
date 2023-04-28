use indoc::indoc;
use insta::assert_display_snapshot;
use pyo3::PyResult;
use sapiens::tools::{invoke_tool, Toolbox};
use sapiens_tools::conclude::ConcludeTool;
use sapiens_tools::dummy::DummyTool;
use sapiens_tools::python::PythonTool;

#[pyo3_asyncio::tokio::test]
async fn test_tool_invocation() -> PyResult<()> {
    let data = indoc! {r#"
    # Action
    ```yaml        
    tool_name: SandboxedPython
    input:
        code: |
            print("Hello world!")          
    ```
    "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default()).await;

    let (tool_name, res) = invoke_tool(toolbox, data).await;
    assert_eq!(tool_name, "SandboxedPython");
    let output = res.unwrap();
    assert_eq!(output.result, "stdout: |\n  Hello world!\nstderr: ''\n");

    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn test_tool_simple_invocation() -> PyResult<()> {
    let data = indoc! {r#"
    # Action
    ```yaml        
    tool_name: Conclude
    input:
        original_question: |
            print("Hello world!")
        conclusion: |
            Hello world!          
    ```
    "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default()).await;
    toolbox.add_tool(ConcludeTool::default()).await;

    let (tool_name, res) = invoke_tool(toolbox, data).await;
    assert_eq!(tool_name, "Conclude");

    let output = res.unwrap();
    assert_display_snapshot!(output.result);

    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn test_tool_invocation_in_python() -> PyResult<()> {
    let data = indoc! {r#"
    # Action
    ```yaml        
    tool_name: SandboxedPython
    input:
        code: |
            print("Hello world!")
            rooms = toolbox.invoke("Dummy", {"blah": "blah"})
            print(rooms)
            rooms = tools.Dummy(blah="blah")
            print(rooms)          
    ```
    "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default()).await;
    toolbox.add_tool(DummyTool::default()).await;

    let (tool_name, res) = invoke_tool(toolbox, data).await;
    assert_eq!(tool_name, "SandboxedPython");
    let output = res.unwrap();
    assert_display_snapshot!(output.result);

    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn test_multiple_tool_invocations() -> PyResult<()> {
    let data = indoc! {r#"
    # Action
    ```yaml        
    tool_name: SandboxedPython
    input:
        code: |
            print("Hello world 1!")          
    ```
    
    # And another action
    ```yaml        
    tool_name: SandboxedPython
    input:
        code: |
            print("Hello world 2!")          
    ```
    
    # And yet another action
    ```        
    tool_name: SandboxedPython
    input:
        code: |
            print("Hello world 3!")          
    ```
    "#};

    let mut toolbox = Toolbox::default();
    toolbox.add_advanced_tool(PythonTool::default()).await;

    let (tool_name, res) = invoke_tool(toolbox, data).await;
    assert_eq!(tool_name, "SandboxedPython");

    let output = res.unwrap();
    assert_display_snapshot!(output.result);

    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn test_python() -> PyResult<()> {
    let mut toolbox = Toolbox::default();
    toolbox.add_tool(DummyTool::default()).await;
    toolbox.add_advanced_tool(PythonTool::default()).await;

    let data = indoc! {r#"```yaml
   tool_name: SandboxedPython
   input:
     code: |           
       args = {
           'blah': "hello"
       }
       output = tools.Dummy(**args)           
      
       something = output['something']                       

       print(f"And the result is: {something}")
   ```
"#};

    let (tool_name, res) = invoke_tool(toolbox, data).await;
    assert_eq!(tool_name, "SandboxedPython");

    let output = res.unwrap();
    assert_display_snapshot!(output.result);

    Ok(())
}

#[pyo3_asyncio::tokio::test]
async fn test_python_docstring() -> PyResult<()> {
    let mut toolbox = Toolbox::default();
    toolbox.add_tool(DummyTool::default()).await;
    toolbox.add_advanced_tool(PythonTool::default()).await;

    let data = indoc! {r#"```yaml
   tool_name: SandboxedPython
   input:
     code: |                  
       output = help(tools.Dummy)           
       print(f"And the docstring is: {output}")
   ```
"#};

    let (tool_name, res) = invoke_tool(toolbox, data).await;
    assert_eq!(tool_name, "SandboxedPython");

    let output = res.unwrap();
    assert_display_snapshot!(output.result);

    Ok(())
}
