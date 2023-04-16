use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// A tool that executes Python code.
pub struct PythonTool {}

impl PythonTool {
    pub fn new() -> Self {
        PythonTool {}
    }
}

impl Default for PythonTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PythonToolInput {
    code: String,
}

#[derive(Serialize, Deserialize)]
pub struct PythonToolOutput {
    stdout: String,
    stderr: String,
}

impl Describe for PythonToolInput {
    fn describe() -> Format {
        vec![
            ("code", "The Python code to execute. For example: `data = [...]; <...>; output = <...> ; print(output)`. MANDATORY").into(),
        ]
        .into()
    }
}

impl Describe for PythonToolOutput {
    fn describe() -> Format {
        vec![
            ("stdout", "The stdout of the executed Python code.").into(),
            ("stderr", "The stderr output of the Python code execution.").into(),
        ]
        .into()
    }
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

impl PythonTool {
    fn invoke_typed(&self, input: &PythonToolInput) -> Result<PythonToolOutput, ToolUseError> {
        let code = &input.code;

        let re = regex::Regex::new(r"import|open|exec|eval|__import__").unwrap();
        if re.is_match(code) {
            return Err(ToolUseError::ToolInvocationFailed(
                "Python code contains forbidden keywords such as import|open|exec|eval|__import__"
                    .to_string(),
            ));
        }

        // TODO(ssoudan) expose tools there

        let res: PyResult<(String, String)> = Python::with_gil(|py| {
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
            Python::run(py, code, None, None)?;

            // NOFUTURE(ssoudan) get something out

            let stdout = py_stdout_cell.borrow().output.clone();
            let stderr = py_stderr_cell.borrow().output.clone();

            Ok((stdout, stderr))
        });

        let (stdout, stderr) = res.map_err(|e| {
            ToolUseError::ToolInvocationFailed(format!("Python code execution failed: {}", e))
        })?;

        Ok(PythonToolOutput { stdout, stderr })
    }
}

impl Tool for PythonTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "SandboxedPythonTool",
            "A tool that executes sandboxed Python code. Only stdout and stderr are captured and made available. ",
            "Use this to transform data. This is not a Tool to retrieve information. Except `print()`, no interactions with the world. No input. No `import`. No library. No API access. It has no access to other Tools. Just plain Python. import|open|exec|eval|__import__ are forbidden.",
            PythonToolInput::describe(),
            PythonToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(&input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}

#[cfg(test)]
mod tests {
    use pyo3::indoc::indoc;
    use pyo3::types::PyDict;

    use super::*;

    #[test]
    fn test_python_tool() {
        let tool = PythonTool::new();
        let input = PythonToolInput {
            code: "print('hello')".to_string(),
        };
        let output = tool.invoke_typed(&input).unwrap();
        assert_eq!(output.stdout, "hello\n");
        assert_eq!(output.stderr, "");
    }

    #[pyfunction]
    fn add_one(x: i64) -> i64 {
        x + 1
    }

    #[pymodule]
    fn foo(_py: Python<'_>, foo_module: &PyModule) -> PyResult<()> {
        foo_module.add_function(wrap_pyfunction!(add_one, foo_module)?)?;
        Ok(())
    }

    #[test]
    fn test_run_with_pyo3() {
        pyo3::append_to_inittab!(foo);
        Python::with_gil(|py| {
            let locals = PyDict::new(py);

            // capture stdout
            let sys = py.import("sys")?;
            let stdout = Logging::default();
            let py_stdout_cell = PyCell::new(py, stdout).unwrap();
            let stderr = Logging::default();
            let py_stderr_cell = PyCell::new(py, stderr).unwrap();

            let py_stdout = py_stdout_cell.borrow_mut();
            let py_stderr = py_stderr_cell.borrow_mut();
            sys.setattr("stdout", py_stdout.into_py(py))?;
            sys.setattr("stderr", py_stderr.into_py(py))?;

            let res = Python::run(
                py,
                indoc! {
                r#"import foo;
                   a = 12
                   b = foo.add_one(a)
                   print("b=", b)                                                                                              
                  "#},
                None,
                locals.into(),
            );

            assert_eq!(locals.get_item("a").unwrap().extract::<i64>().unwrap(), 12);
            assert_eq!(locals.get_item("b").unwrap().extract::<i64>().unwrap(), 13);

            let stdout = py_stdout_cell.borrow();
            assert_eq!(stdout.output, "b= 13\n");

            let stderr = py_stderr_cell.borrow();
            assert_eq!(stderr.output, "");

            res
        }).unwrap();
    }
}
