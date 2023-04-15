use std::process::Command;

use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
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
    status: Option<i32>,
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
            ("status", "The exit status of the Python code execution.").into(),
            ("stdout", "The stdout of the executed Python code.").into(),
            ("stderr", "The stderr output of the Python code execution.").into(),
        ]
        .into()
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

        // // Add tool bindings at the beginning of the code
        // let code = format!(
        //     r#"def RoomTool(room_filter=[]):
        //            return "nice try"
        //        {}"#,
        //     code
        // );

        // TODO(ssoudan) use pyo3

        // TODO(ssoudan) expose tools there

        // NOTE(ssoudan) capturing stdout: https://github.com/PyO3/pyo3/discussions/1918#discussioncomment-1473356

        let mut command = Command::new("python3");
        command
            .env_clear()
            .arg("-c")
            .arg(code)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        command.stdin(std::process::Stdio::null());
        let child = command.spawn().map_err(|_e| {
            ToolUseError::ToolInvocationFailed("failed to execute process".to_string())
        })?;

        // read stdout and stderr
        let output = child
            .wait_with_output()
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;
        Ok(PythonToolOutput {
            status: output.status.code(),
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        })
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

    use super::*;

    #[test]
    fn test_python_tool() {
        let tool = PythonTool::new();
        let input = PythonToolInput {
            code: "print('hello')".to_string(),
        };
        let output = tool.invoke_typed(&input).unwrap();
        assert_eq!(output.status, Some(0));
        assert_eq!(output.stdout, "hello\n");
        assert_eq!(output.stderr, "");
    }

    use pyo3::prelude::*;
    use pyo3::types::PyDict;

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

    #[pyfunction]
    fn add_one(x: i64) -> i64 {
        x + 1
    }

    #[pymodule]
    fn foo(_py: Python<'_>, foo_module: &PyModule) -> PyResult<()> {
        foo_module.add_function(wrap_pyfunction!(add_one, foo_module)?)?;
        Ok(())
    }

    fn run() -> PyResult<()> {
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
                   print("foo=", repr(foo))
                   print("foo=", dir(foo))                                                                           
                  "#},
                None,
                locals.into(),
            );

            for (key, value) in locals {
                println!("{}: {}", key, value);
            }

            let stdout = py_stdout_cell.borrow();
            print!("stdout: {}", stdout.output);

            let stderr = py_stderr_cell.borrow();
            print!("stderr: {}", stderr.output);

            res
        })
    }

    #[test]
    fn test_run_with_pyo3() {
        run().unwrap();
    }
}
