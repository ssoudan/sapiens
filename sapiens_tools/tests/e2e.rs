use std::rc::Rc;

use indoc::indoc;
use sapiens::invoke_tool;
use sapiens::tools::Toolbox;
use sapiens_tools::dummy::DummyTool;
use sapiens_tools::python;

#[test]
fn test_python() {
    let mut toolbox = Toolbox::default();
    toolbox.add_tool(DummyTool::default());
    toolbox.add_advanced_tool(python::PythonTool::default());

    let toolbox = Rc::new(toolbox);

    let input = indoc! {
    r#"```yaml
       command: SandboxedPython
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

    let res = invoke_tool(toolbox, input).unwrap();

    assert_eq!(
        res,
        "stdout: |\n  And the result is: hello and something else\nstderr: ''\n"
    );
}
