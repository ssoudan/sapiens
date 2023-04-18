//! Tools for sapiens

/// Hue tools
pub mod hue;

/// Tool to conclude a chain
pub mod conclude;

/// Tool to run some (limited) python
pub mod python;

#[cfg(test)]
/// Tool to test stuffs
pub(crate) mod dummy;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use indoc::indoc;
    use sapiens::invoke_tool;
    use sapiens::tools::Toolbox;

    use crate::dummy::DummyTool;
    use crate::python::PythonTool;

    #[test]
    fn test_tool_invocation() {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world!")          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default());

        let toolbox = Rc::new(toolbox);

        let output = invoke_tool(toolbox, data).unwrap();
        assert_eq!(output, "stdout: |\n  Hello world!\nstderr: ''\n");
    }

    #[test]
    fn test_tool_invocation_in_python() {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world!")
                rooms = toolbox.invoke("Dummy", {"blah": "blah"})
                print(rooms)
                rooms = tools.dummy(blah="blah")
                print(rooms)          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default());
        toolbox.add_tool(DummyTool::default());

        let toolbox = Rc::new(toolbox);

        let output = invoke_tool(toolbox, data).unwrap();
        assert_eq!(
            output,
            "stdout: |\n  Hello world!\n  {'something': 'blah and something else'}\n  {'something': 'blah and something else'}\nstderr: ''\n"
        );
    }

    #[test]
    fn test_multiple_tool_invocations() {
        let data = indoc! {r#"
        # Action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 1!")          
        ```
        
        # And another action
        ```yaml        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 2!")          
        ```
        
        # And yet another action
        ```        
        command: SandboxedPython
        input:
            code: |
                print("Hello world 3!")          
        ```
        "#};

        let mut toolbox = Toolbox::default();
        toolbox.add_advanced_tool(PythonTool::default());

        let toolbox = Rc::new(toolbox);

        let output = invoke_tool(toolbox, data).unwrap();
        assert_eq!(output, "stdout: |\n  Hello world 1!\nstderr: ''\n");
    }
}
