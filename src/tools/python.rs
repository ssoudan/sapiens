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

        // send input to stdin if present
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
            "Use this to transform data. This is not a tool to retrieve information. Except `print()`, no interactions with the world. No input. No `import`. No library. No API access. Just plain Python. import|open|exec|eval|__import__ are forbidden.",
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
