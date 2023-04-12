use std::process::Command;

use llm_chain_tools::{Describe, Format, Tool, ToolDescription};
use serde::{Deserialize, Serialize};

/// A tool that executes a bash command.
pub struct BashTool {}

impl BashTool {
    pub fn new() -> Self {
        BashTool {}
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BashToolInput {
    cmd: String,
}

#[derive(Serialize, Deserialize)]
pub struct BashToolOutput {
    stderr: String,
    stdout: String,
    status: isize,
}

impl Describe for BashToolInput {
    fn describe() -> Format {
        vec![("cmd", "The command to execute in the bash shell.").into()].into()
    }
}

impl Describe for BashToolOutput {
    fn describe() -> Format {
        vec![
            ("result", "Exit code 0 == success").into(),
            ("stderr", "The stderr output of the command").into(),
            ("stdout", "The stdout output of the command").into(),
        ]
        .into()
    }
}

impl BashTool {
    fn invoke_typed(&self, input: &BashToolInput) -> Result<BashToolOutput, String> {
        let output = Command::new("bash")
            .arg("-c")
            // .arg("echo")
            .arg(&input.cmd)
            .output()
            .map_err(|_e| "failed to execute process")?;
        Ok(BashToolOutput {
            status: output.status.code().unwrap() as isize,
            stderr: String::from_utf8(output.stderr).unwrap(),
            stdout: String::from_utf8(output.stdout).unwrap(),
        })
    }
}

impl Tool for BashTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "BashTool",
            "A tool that executes a bash command.",
            "Use this to execute local commands to solve your goals",
            BashToolInput::describe(),
            BashToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, String> {
        let input = serde_yaml::from_value(input).unwrap();
        let output = self.invoke_typed(&input).unwrap();
        Ok(serde_yaml::to_value(output).unwrap())
    }
}
