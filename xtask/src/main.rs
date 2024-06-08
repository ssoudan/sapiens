//! xtask is a simple task runner for Rust projects.
#![allow(clippy::print_stderr, clippy::print_stdout)]

mod coverage;
mod flags;
mod lint;
mod todo;
mod utils;
mod validate;

use xshell::Shell;

fn main() -> anyhow::Result<()> {
    let flags = flags::Xtask::from_env_or_exit();

    let sh = &Shell::new()?;
    let project = utils::Project::default();
    sh.change_dir(project.root_path());

    match flags.subcommand {
        flags::XtaskCmd::Validate(cmd) => cmd.run(sh, &project),
        flags::XtaskCmd::Lint(cmd) => cmd.run(sh, &project),
        flags::XtaskCmd::Todo(cmd) => cmd.run(sh, &project),
        flags::XtaskCmd::Coverage(cmd) => cmd.run(sh, &project),
    }
}
