//! Coverage
use std::env;
use std::process::Command;

use xshell::Shell;

use crate::flags;
use crate::utils::Project;

impl flags::Coverage {
    /// Run the linting command
    #[allow(clippy::unused_self)]
    pub(crate) fn run(&self, _shell: &Shell, project: &Project) -> anyhow::Result<()> {
        let root = project.root_path();
        let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .current_dir(root)
            .args([
                "tarpaulin",
                "--engine",
                "llvm",
                "--workspace",
                "--all-features",
                "--no-dead-code",
                "--out",
                "Xml",
            ])
            .status()?;

        if !status.success() {
            println!("Coverage failed");
            std::process::exit(1);
        }

        Ok(())
    }
}
