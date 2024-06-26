//! Options for the `xtask` command.
#![allow(unreachable_pub, missing_docs, clippy::missing_docs_in_private_items)]

xflags::xflags! {
    src "./src/flags.rs"

    /// Run custom build command.
    cmd xtask {

        /// Validate the project configuration.
        cmd validate {
            /// Run optional checks - for example, pre-commit checks.
            optional --strict
        }

        /// Lint the project.
        cmd lint {}

        /// Collect todo comments.
        cmd todo {}

        /// Coverage
        cmd coverage {}
    }
}

// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build -p xtask` to regenerate.
#[derive(Debug)]
pub struct Xtask {
    pub subcommand: XtaskCmd,
}

#[derive(Debug)]
pub enum XtaskCmd {
    Validate(Validate),
    Lint(Lint),
    Todo(Todo),
    Coverage(Coverage),
}

#[derive(Debug)]
pub struct Validate {
    pub strict: bool,
}

#[derive(Debug)]
pub struct Lint;

#[derive(Debug)]
pub struct Todo;

#[derive(Debug)]
pub struct Coverage;

impl Xtask {
    #[allow(dead_code)]
    pub fn from_env_or_exit() -> Self {
        Self::from_env_or_exit_()
    }

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}
// generated end
