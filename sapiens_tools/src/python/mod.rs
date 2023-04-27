use std::cmp::Ordering;
use std::collections::HashMap;

use convert_case::{Case, Casing};
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use sapiens::tools::{
    invoke_simple_from_toolbox, AdvancedTool, Describe, ProtoToolDescribe, ProtoToolInvoke,
    ToolDescription, ToolUseError, Toolbox,
};
use sapiens_derive::{Describe, ProtoToolDescribe};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

/// Conversion tools
pub(crate) mod utils;

use crate::python::utils::SimpleToolDescription;

const MAX_OUTPUT_SIZE: usize = 512;

/// A tool that runs sandboxed Python code. Use this to transform data.
///
/// - To use another Tool with input format `input_field_1` and `input_field_2`
///   and output format with fields `output_field_1` and `output_field_2` use:
/// ```python
/// result = tools.ToolName(input_field_1=..., input_field_2=...)
/// print(result['output_field_1'])
/// print(result['output_field_2'])
/// ```
/// - Only stdout and stderr are captured and made available (limited to 512B
///   total). If the output is larger, use `tools.Conclude` directly from the
///   code.
/// - List available tools with `tools.list()`. And returns a list of
///   `{'name':.., 'description':.., 'input_format':.., 'output_format':.., }`.
/// - `open`|`exec` are forbidden.
/// - Limited libraries available: urllib3, requests, sympy, numpy,
/// BeautifulSoup4, feedparser, arxiv.
/// - No PIP.
#[derive(Debug, Default, ProtoToolDescribe)]
#[tool(
    name = "SandboxedPython",
    input = "PythonToolInput",
    output = "PythonToolOutput"
)]
pub struct PythonTool {}

/// The input of the Python tool
#[derive(Debug, Serialize, Deserialize, Describe)]
pub struct PythonToolInput {
    /// The Python code to run. MANDATORY
    pub code: String,
}

/// The output of the Python tool
#[derive(Serialize, Deserialize, Describe)]
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

        ToolsWrapper { toolbox, tool_list }
    }
}

#[pymethods]
impl ToolsWrapper {
    // list all tools
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
                pyo3::exceptions::PyException::new_err(format!("Invalid input: {}", e))
            })?
        } else {
            Value::default()
        };

        // println!("invoking tool {} with input {:?}", tool_name, input);

        // Build the runtime for the new thread.
        //
        // The runtime is created before spawning the thread
        // to more cleanly forward errors if the `unwrap()`
        // panics.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, mut rx) = tokio::sync::oneshot::channel::<Result<Value, ToolUseError>>();

        let toolbox = self.toolbox.clone();

        let tool_name = tool_name.to_string();

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

        // blockingly wait for the result
        loop {
            if let Ok(output) = rx.try_recv() {
                let output = output.map_err(|e| {
                    pyo3::exceptions::PyException::new_err(format!("Tool invocation failed: {}", e))
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

        let code = Self::transform_code(code, tools)?;

        let toolwrapper = ToolsWrapper::new(toolbox).await;

        // print!("{}", code);

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

            // FUTURE(ssoudan) pass something in

            // run code
            Python::run(py, &code, globals.into(), None)?;

            // NOFUTURE(ssoudan) get something out

            let stdout = py_stdout_cell.borrow().output.clone();
            let stderr = py_stderr_cell.borrow().output.clone();

            Ok((stdout, stderr))
        });

        let (stdout, stderr) = res.map_err(|e| {
            ToolUseError::InvocationFailed(format!("Python code execution failed: {}", e))
        })?;

        Ok(PythonToolOutput { stdout, stderr })
    }

    fn transform_code(
        code: String,
        tools: HashMap<String, ToolDescription>,
    ) -> Result<String, ToolUseError> {
        lazy_static::lazy_static! {
            static ref EXEC_RE: regex::Regex = regex::Regex::new(r"(exec|pip)").unwrap();
            static ref IMPORT_RE: regex::Regex = regex::Regex::new(r"(?x)import \s+ tools.*").unwrap();
            static ref FROM_RE: regex::Regex =
                regex::Regex::new(r"(?x)from \s+ tools \s+ import .*").unwrap();
        }

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
        tool_class_code.push_str("    def __init__(self, toolbox):\n");
        tool_class_code.push_str("        self.toolbox = toolbox\n");

        let mut binding_code = String::new();

        for (name, description) in tools {
            let inputs_parts = description.input_format.fields;
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
                format!("(self, {})", inputs)
            };

            let dict = inputs_parts
                .iter()
                .map(|f| {
                    let name = &f.name;
                    format!("\"{}\": {}", name, name)
                })
                .collect::<Vec<_>>()
                .join(", ");

            // in snake case
            // tool_class_code.push_str(&format!(
            //     "    def {}{}:\n        return self.toolbox.invoke(\"{}\", {{{}}})\n",
            //     name.to_case(Case::Snake),
            //     inputs,
            //     name,
            //     dict
            // ));

            // in Pascal case
            tool_class_code.push_str(&format!(
                "    def {}{}:\n        return self.toolbox.invoke(\"{}\", {{{}}})\n",
                name.to_case(Case::Pascal),
                inputs,
                name,
                dict
            ));

            // add ToolName(..) to the binding code
            binding_code.push_str(&format!(
                "def {}{}:\n        return tools.toolbox.invoke(\"{}\", {{{}}})\n",
                name, inputs, name, dict
            ));

            // FUTURE(ssoudan) set input_format and output_format
        }

        // add list function
        tool_class_code.push_str("    def list(self):\n");
        tool_class_code.push_str("        return self.toolbox.list()\n");

        tool_class_code.push_str("tools = Tools(toolbox)\n");

        let code_to_prepend = format!("{}{}", tool_class_code, binding_code);

        // prepend the code to the user code
        let code = format!("{}\n# ======== user code\n{}", code_to_prepend, code);

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
            ToolUseError::InvocationFailed(format!("Python code execution failed: {}", e))
        })?;

        Ok(PythonToolOutput { stdout, stderr })
    }
}

#[async_trait::async_trait]
impl ProtoToolInvoke for PythonTool {
    async fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;

        let output = self.invoke_sync_typed(&input)?;

        // check the size of the output (stdout and stderr)
        let l = output.stdout.len() + output.stderr.len();
        if l > MAX_OUTPUT_SIZE {
            return Err(ToolUseError::InvocationFailed(format!(
                "Python code produced too much output on stdout and stderr
        combined ({} bytes) - max is {}",
                l, MAX_OUTPUT_SIZE
            )));
        }

        Ok(serde_yaml::to_value(output)?)
    }
}

#[async_trait::async_trait]
impl AdvancedTool for PythonTool {
    async fn invoke_with_toolbox(
        &self,
        toolbox: Toolbox,
        input: Value,
    ) -> Result<Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(toolbox, &input).await?;
        Ok(serde_yaml::to_value(output)?)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_display_snapshot;
    use sapiens::tools::Toolbox;

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

        let mut toolbox = Toolbox::default();
        // toolbox.add_tool(ArxivTool::new().await).await;
        toolbox.add_terminal_tool(ConcludeTool::default()).await;
        // toolbox.add_advanced_tool(PythonTool::default()).await;

        let tools = toolbox.describe().await;

        let code = PythonTool::transform_code(input.code, tools).unwrap();

        assert_display_snapshot!(code);
    }
}
