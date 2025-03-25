use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::LazyLock;

use convert_case::{Case, Casing};
use pyo3::indoc::{formatdoc, indoc};
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use sapiens::tools::toolbox::{invoke_simple_from_toolbox, Toolbox};
use sapiens::tools::{
    AdvancedTool, Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError,
};
use sapiens_derive::{Describe, ProtoToolDescribe};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tracing::trace;

/// Conversion tools
pub(crate) mod utils;

use crate::python::utils::SimpleToolDescription;

const MAX_OUTPUT_SIZE: usize = 512;

// FUTURE(ssoudan) install pySWIP

/// A tool that runs sandboxed Python code. Use this to transform data.
///
/// - To use another Tool with parameters `input_field_1` and `input_field_2`
///   and result fields `output_field_1` and `output_field_2` use:
/// ```python
/// result = tools.ToolName(input_field_1=..., input_field_2=...)
/// print(result['output_field_1'])
/// print(result['output_field_2'])
/// ```
/// - Only stdout and stderr are captured and made available (limited to 512B
///   total). If the output is larger, use `tools.Conclude` directly from the
///   code.
/// - List available tools with `tools.list()`. And returns a list of
///   `{'name':.., 'description':.., 'parameters':.., 'responses_content':..,
///   }`.
/// - `open`|`exec` are forbidden.
/// - Limited libraries available: urllib3, requests, sympy, numpy,
///   `BeautifulSoup4`, feedparser, arxiv.
/// - No PIP.
#[derive(Debug, Default, ProtoToolDescribe)]
#[tool(
    name = "SandboxedPython",
    input = "PythonToolInput",
    output = "PythonToolOutput"
)]
#[allow(clippy::module_name_repetitions)]
pub struct PythonTool {}

/// The input of the Python tool
#[derive(Debug, Serialize, Deserialize, Describe)]
#[allow(clippy::module_name_repetitions)]
pub struct PythonToolInput {
    /// The Python code to run. MANDATORY
    pub code: String,
}

/// The output of the Python tool
#[derive(Serialize, Deserialize, Describe)]
#[allow(clippy::module_name_repetitions)]
pub struct PythonToolOutput {
    /// The stdout output of the Python code.
    pub stdout: String,
    /// The stderr output of the Python code.
    pub stderr: String,
}

#[pyclass]
#[derive(Default)]
struct Logging {
    output: String,
}

#[pymethods]
impl Logging {
    fn write(&mut self, data: &str) {
        self.output.push_str(data);
    }
}

#[pyclass(unsendable)]
struct ToolsWrapper {
    toolbox: Toolbox,
    tool_list: Vec<SimpleToolDescription>,
}

impl ToolsWrapper {
    async fn new(toolbox: Toolbox) -> Self {
        let tools = toolbox.describe().await;
        let tool_list = tools
            .into_values()
            .map(SimpleToolDescription::from)
            .collect::<Vec<_>>();

        Self { toolbox, tool_list }
    }
}

#[pymethods]
impl ToolsWrapper {
    // list all tools
    #[allow(clippy::unnecessary_wraps)]
    #[pyo3(signature = ())]
    fn list(&self, py: Python<'_>) -> PyResult<PyObject> {
        let tools = self.tool_list.to_object(py);
        Ok(tools)
    }

    // invoke a tool
    #[pyo3(signature = (tool_name, input))]
    fn invoke(
        &self,
        py: Python<'_>,
        tool_name: &str,
        input: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        // convert PyDict to a serde_yaml::Value
        let input = if let Some(input) = input {
            let input: PyObject = input.into();

            utils::to_yaml(py, &input).map_err(|e| {
                pyo3::exceptions::PyException::new_err(format!("Invalid input: {e}"))
            })?
        } else {
            Value::default()
        };

        let (tx, mut rx) = tokio::sync::oneshot::channel::<Result<Value, ToolUseError>>();

        // release the GIL to allow the thread to run
        py.allow_threads(move || {
            let toolbox = self.toolbox.clone();

            let tool_name = tool_name.to_string();

            // Build the runtime for the new thread.
            //
            // The runtime is created before spawning the thread
            // to more cleanly forward errors if the `unwrap()`
            // panics.
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            std::thread::spawn(move || {
                rt.block_on(async move {
                    let output = invoke_simple_from_toolbox(toolbox, &tool_name, input).await;

                    match output {
                        Ok(output) => {
                            tx.send(Ok(output)).unwrap();
                        }
                        Err(e) => {
                            tx.send(Err(e)).unwrap();
                        }
                    }
                });
            });
        });

        // blockingly wait for the result
        loop {
            if let Ok(output) = rx.try_recv() {
                let output = output.map_err(|e| {
                    pyo3::exceptions::PyException::new_err(format!("Tool invocation failed: {e}"))
                })?;

                let output = utils::value_to_object(output, py);

                return Ok(output);
            }
        }
    }
}

impl PythonTool {
    #[tracing::instrument(skip(self, toolbox))]
    async fn invoke_typed(
        &self,
        toolbox: Toolbox,
        input: &PythonToolInput,
    ) -> Result<PythonToolOutput, ToolUseError> {
        let code = input.code.clone();

        let tools = toolbox.describe().await;

        let code = Self::transform_code(&code, tools)?;

        let toolwrapper = ToolsWrapper::new(toolbox).await;

        trace!("Running code:\n{}", code);

        // FIXME(ssoudan) got to set a limit on the execution time
        // https://docs.python.org/3/library/asyncio-task.html#timeouts
        // https://stackoverflow.com/questions/70142680/pyo3-prevent-user-submitted-code-from-looping-and-blocking-server-thread

        let res: PyResult<(String, String)> = Python::with_gil(|py| {
            // println!("Python version: {}", py.version());

            let tools_cell = PyCell::new(py, toolwrapper)?;
            let globals = [("toolbox", tools_cell)].into_py_dict(py);

            // capture stdout and stderr
            let sys = py.import("sys")?;

            let stdout = Logging::default();
            let py_stdout_cell = PyCell::new(py, stdout)?;
            let py_stdout = py_stdout_cell.borrow_mut();
            sys.setattr("stdout", py_stdout.into_py(py))?;

            let stderr = Logging::default();
            let py_stderr_cell = PyCell::new(py, stderr)?;
            let py_stderr = py_stderr_cell.borrow_mut();
            sys.setattr("stderr", py_stderr.into_py(py))?;

            // NOFUTURE(ssoudan) pass something in

            // run code
            Python::run(py, &code, globals.into(), None)?;

            // NOFUTURE(ssoudan) get something out

            let stdout = py_stdout_cell.borrow().output.clone();
            let stderr = py_stderr_cell.borrow().output.clone();

            Ok((stdout, stderr))
        });

        let (stdout, stderr) = res.map_err(|e| {
            ToolUseError::InvocationFailed(format!("Python code execution failed: {e}"))
        })?;

        Ok(PythonToolOutput { stdout, stderr })
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::format_push_string)]
    fn transform_code(
        code: &str,
        tools: HashMap<String, ToolDescription>,
    ) -> Result<String, ToolUseError> {
        static EXEC_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"(exec|pip)").unwrap());
        static IMPORT_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"(?x)import \s+ tools.*").unwrap());
        static FROM_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"(?x)from \s+ tools \s+ import .*").unwrap());

        // FUTURE(ssoudan) use PyModule::from_code ?

        // check for forbidden keywords - with capture
        if let Some(caps) = EXEC_RE.captures(code.as_ref()) {
            return Err(ToolUseError::InvocationFailed(format!(
                "Python code contains forbidden keywords such as {}",
                caps.get(0).unwrap().as_str()
            )));
        }

        // remove the `import tools` if present
        // remove the `from tools import xxx` if present
        let code = code
            .lines()
            .filter(|&l| !IMPORT_RE.is_match(l))
            .filter(|&l| !FROM_RE.is_match(l))
            // .map(|l| l.replace(r"^import tools.*$", ""))
            // .map(|l| l.replace(r"^from tools import.*", ""))
            .collect::<Vec<_>>()
            .join("\n");

        let mut tool_class_code = String::new();

        tool_class_code.push_str("class Tools:\n");
        tool_class_code.push_str(
            &indoc! {r#"
            """Wrapper for the tools."""
            def __init__(self, toolbox):
                self.toolbox = toolbox
            "#}
            .lines()
            .map(|s| format!("    {s}"))
            .collect::<Vec<_>>()
            .join("\n"),
        );
        tool_class_code.push('\n');

        for (name, description) in tools {
            let inputs_parts = description.parameters.fields;
            let output_parts = description.responses_content.fields;

            // FUTURE(ssoudan) might want to add None only for optional inputs
            let mut inputs = inputs_parts.clone();

            // sort with the optional inputs at the end
            inputs.sort_by(|a, b| {
                if a.optional && !b.optional {
                    Ordering::Greater
                } else if !a.optional && b.optional {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            });

            let inputs = inputs
                .into_iter()
                .map(|f| {
                    if f.optional {
                        format!("{}=None", f.name)
                    } else {
                        f.name
                    }
                })
                .collect::<Vec<_>>();

            let inputs = inputs.join(", ");
            let inputs = if inputs.is_empty() {
                "(self)".to_string()
            } else {
                format!("(self, {inputs})")
            };

            // Build the docstring
            // start got to be 4-spaces aligned
            //
            // Several descriptions are multi-line:
            // - description.description
            // - description.parameters.fields.description
            // - description.responses_content.fields.description
            let mut docstring = String::new();
            // Description
            description.description.lines().for_each(|l| {
                docstring.push_str(&format!("    {l}\n"));
            });
            // Arguments
            docstring.push_str("    Args:\n");
            for f in &inputs_parts {
                docstring.push_str(&format!("        {}: <{}> ", f.name, f.r#type));
                // first line goes with the name, the rest is indented
                f.description.lines().enumerate().for_each(|(i, l)| {
                    if i == 0 {
                        docstring.push_str(&format!("{l}\n"));
                    } else {
                        docstring.push_str(&format!("            {l}\n"));
                    }
                });
            }
            if !output_parts.is_empty() {
                // Return
                docstring.push_str("    Returns:\n");
                docstring.push_str("           A dictionary with the following keys:\n");
                for f in &output_parts {
                    if f.optional {
                        docstring
                            .push_str(&format!("        {}: <{}> (Optional) ", f.name, f.r#type));
                    } else {
                        docstring.push_str(&format!("        {}: <{}> ", f.name, f.r#type));
                    }

                    // first line goes with the name, the rest is indented
                    f.description.lines().enumerate().for_each(|(i, l)| {
                        if i == 0 {
                            docstring.push_str(&format!("{l}\n"));
                        } else {
                            docstring.push_str(&format!("            {l}\n"));
                        }
                    });
                }
            }

            let dict = inputs_parts
                .iter()
                .map(|f| {
                    let name = &f.name;
                    format!("\"{name}\": {name}")
                })
                .collect::<Vec<_>>()
                .join(", ");

            for cased_name in [name.to_case(Case::Snake), name.to_case(Case::Pascal)] {
                // in Pascal case
                tool_class_code.push_str(&indent(
                    4,
                    &formatdoc! {r#"
            def {}{}:
                """{}    """
                return self.toolbox.invoke("{}", {{{}}})
            "#, 
                        cased_name,
                        inputs,
                        docstring,
                        name,
                        dict
                    },
                ));
                tool_class_code.push('\n');
            }
        }

        // add list function
        tool_class_code.push_str(&indent(
            4,
            indoc! {r#"
            def list(self):
                """List the tools."""
                return self.toolbox.list()
            "#},
        ));
        tool_class_code.push('\n');

        // instantiate the class
        tool_class_code.push_str("tools = Tools(toolbox)\n");

        let code_to_prepend = tool_class_code;

        // prepend the code to the user code
        let code = format!("{code_to_prepend}\n# ======== user code\n{code}");

        Ok(code)
    }

    #[tracing::instrument(skip(self))]
    fn invoke_sync_typed(&self, input: &PythonToolInput) -> Result<PythonToolOutput, ToolUseError> {
        let code = input.code.clone();

        // check for forbidden keywords - with capture
        let re = regex::Regex::new(r"(exec|pip)").unwrap();
        if let Some(caps) = re.captures(&code) {
            return Err(ToolUseError::InvocationFailed(format!(
                "Python code contains forbidden keywords such as {}",
                caps.get(0).unwrap().as_str()
            )));
        }

        let res: PyResult<(String, String)> = Python::with_gil(|py| {
            // println!("Python version: {}", py.version());

            let globals = PyDict::new(py);

            // capture stdout and stderr
            let sys = py.import("sys")?;

            let stdout = Logging::default();
            let py_stdout_cell = PyCell::new(py, stdout)?;
            let py_stdout = py_stdout_cell.borrow_mut();
            sys.setattr("stdout", py_stdout.into_py(py))?;

            let stderr = Logging::default();
            let py_stderr_cell = PyCell::new(py, stderr)?;
            let py_stderr = py_stderr_cell.borrow_mut();
            sys.setattr("stderr", py_stderr.into_py(py))?;

            // run code
            Python::run(py, &code, globals.into(), None)?;

            let stdout = py_stdout_cell.borrow().output.clone();
            let stderr = py_stderr_cell.borrow().output.clone();

            Ok((stdout, stderr))
        });

        let (stdout, stderr) = res.map_err(|e| {
            ToolUseError::InvocationFailed(format!("Python code execution failed: {e}"))
        })?;

        Ok(PythonToolOutput { stdout, stderr })
    }
}

fn indent(offset: u32, s: &str) -> String {
    let mut indented = String::new();
    for _ in 0..offset {
        indented.push(' ');
    }

    s.lines()
        .map(|l| format!("{indented}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[async_trait::async_trait]
impl ProtoToolInvoke for PythonTool {
    async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input =
            serde_yaml::from_value(input).map_err(|e| ToolUseError::InvalidInput(e.to_string()))?;

        let output = self.invoke_sync_typed(&input)?;

        // check the size of the output (stdout and stderr)
        let l = output.stdout.len() + output.stderr.len();
        if l > MAX_OUTPUT_SIZE {
            return Err(ToolUseError::InvocationFailed(format!(
                "Python code produced too much output on stdout and stderr
        combined ({l} bytes) - max is {MAX_OUTPUT_SIZE}"
            )));
        }

        Ok(serde_yaml::to_value(output).map_err(|e| ToolUseError::InvalidOutput(e.to_string()))?)
    }
}

#[async_trait::async_trait]
impl AdvancedTool for PythonTool {
    async fn invoke_with_toolbox(
        &self,
        toolbox: Toolbox,
        input: Value,
    ) -> Result<Value, ToolUseError> {
        let input =
            serde_yaml::from_value(input).map_err(|e| ToolUseError::InvalidInput(e.to_string()))?;
        let output = self.invoke_typed(toolbox, &input).await?;
        Ok(serde_yaml::to_value(output).map_err(|e| ToolUseError::InvalidOutput(e.to_string()))?)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_snapshot;
    use sapiens::tools::toolbox::Toolbox;

    use crate::conclude::ConcludeTool;
    use crate::python::{PythonTool, PythonToolInput};

    #[tokio::test]
    async fn test_code_transformation() {
        let input = PythonToolInput {
            code: indoc! {
                r#"
                import tools
                from tools import Arxiv

                arxiv_results = Arxiv(
                  search_query='cat:cs.AI',
                  max_results=5,
                  sort_by='lastUpdatedDate',
                  sort_order='descending',
                  show_summary=True
                )
            
                formatted_results = []
                for result in arxiv_results['result']:
                    formatted_results.append(f"{result['title']} : {result['pdf_url']}")
                formatted_results = "\n".join(formatted_results)
            
                print({'formatted_results': formatted_results})
            "#}
            .to_string(),
        };

        let toolbox = Toolbox::default();
        // toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        // toolbox.add_advanced_tool(PythonTool::default()).await;

        let tools = toolbox.describe().await;

        let code = PythonTool::transform_code(&input.code, tools).unwrap();

        assert_snapshot!(code);
    }
}
