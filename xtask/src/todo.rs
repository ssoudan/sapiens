//! Collect todo comments from the project

use std::path::PathBuf;
use std::str::FromStr;

use color_print::cprintln;
use regex::Regex;
use xshell::Shell;

use crate::flags;
use crate::utils::{Project, Visitor};

impl flags::Todo {
    /// Run the todo command
    #[allow(clippy::unused_self)]
    pub(crate) fn run(self, _sh: &Shell, project: &Project) -> anyhow::Result<()> {
        let mut todo = TodoVisitor::default();
        todo.visit_project(project)
    }
}

/// Todo visitor
#[derive(Default)]
pub(crate) struct TodoVisitor {
    /// Files to validate.
    files: Vec<PathBuf>,
    /// Todo comments
    todo: Vec<TodoComment>,
}

/// Task verb
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verb {
    /// NOW: a task being worked on
    Now,
    /// TODO: a task to be done before things are considered done
    ToDo,
    /// FIXME: a task that needs to be fixed ASAP
    FixMe,
    /// FUTURE: a task that will be done in the future
    Future,
    /// NOFUTURE: a task that may be done in the future
    NoFuture,
    /// NOTE: a note
    Note,
    /// Unknown verb
    Unknown(String),
}

impl std::fmt::Display for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Now => write!(f, "NOW"),
            Self::ToDo => write!(f, "TODO"),
            Self::FixMe => write!(f, "FIXME"),
            Self::Future => write!(f, "FUTURE"),
            Self::NoFuture => write!(f, "NOFUTURE"),
            Self::Note => write!(f, "NOTE"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

impl FromStr for Verb {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NOW" => Ok(Self::Now),
            "TODO" => Ok(Self::ToDo),
            "FIXME" => Ok(Self::FixMe),
            "FUTURE" => Ok(Self::Future),
            "NOFUTURE" => Ok(Self::NoFuture),
            "NOTE" => Ok(Self::Note),
            _ => Ok(Self::Unknown(s.to_string())),
        }
    }
}

/// A todo comment
pub(crate) struct TodoComment {
    /// File where the comment is
    pub(crate) file: PathBuf,
    /// Line number
    pub(crate) line: usize,
    /// Verb
    pub(crate) verb: Verb,
    /// Username
    pub(crate) username: String,
    /// Task
    pub(crate) task: String,
}

impl TodoComment {
    /// Print the todo comment
    pub(crate) fn print(&self, offset: usize) {
        let file_line_str = format!("{}:{}", self.file.display(), self.line);
        let spacer = " ".repeat(offset);
        match &self.verb {
            Verb::Now => cprintln!(
                "{}:{}<green>NOW</green>({}) {}",
                file_line_str,
                spacer,
                self.username,
                self.task
            ),
            Verb::ToDo => cprintln!(
                "{}:{}<blue>TODO</blue>({}) {}",
                file_line_str,
                spacer,
                self.username,
                self.task
            ),
            Verb::FixMe => cprintln!(
                "{}:{}<red>FIXME</red>({}) {}",
                file_line_str,
                spacer,
                self.username,
                self.task
            ),
            Verb::Future => cprintln!(
                "{}:{}<magenta>FUTURE</magenta>({}) {}",
                file_line_str,
                spacer,
                self.username,
                self.task
            ),
            Verb::NoFuture => cprintln!(
                "{}:{}<cyan>NOFUTURE</cyan>({}) {}",
                file_line_str,
                spacer,
                self.username,
                self.task
            ),
            Verb::Note => cprintln!(
                "{}:{}<yellow>NOTE</yellow>({}) {}",
                file_line_str,
                spacer,
                self.username,
                self.task
            ),
            Verb::Unknown(s) => cprintln!(
                "{}:{}{}({}) {}",
                file_line_str,
                spacer,
                s,
                self.username,
                self.task
            ),
        }
    }
}

impl std::fmt::Display for TodoComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}({}) - {}",
            self.file.display(),
            self.line,
            self.verb,
            self.username,
            self.task
        )
    }
}

impl Visitor for TodoVisitor {
    fn visit_project(&mut self, project: &crate::utils::Project) -> anyhow::Result<()> {
        for crate_ in project.workspace_members() {
            self.visit_crate(&crate_)?;
        }

        self.collect_todos(project)?;

        let pos = self
            .todo
            .iter()
            .map(|todo| format!("{}:{}", todo.file.display(), todo.line).len())
            .max()
            .unwrap_or(0)
            + 1;

        for todo in &self.todo {
            let file_line_str = format!("{}:{}", todo.file.display(), todo.line);
            todo.print(pos - file_line_str.len());
        }

        Ok(())
    }

    fn visit_crate(&mut self, crate_: &crate::utils::Crate) -> anyhow::Result<()> {
        // collect the files from the crate
        self.files.extend_from_slice(&crate_.files()?);

        Ok(())
    }
}

impl TodoVisitor {
    /// Collect todo comments from the project
    fn collect_todos(&mut self, project: &Project) -> anyhow::Result<()> {
        let project_root = project.root_path();

        self.todo = collect_todos(&self.files, project_root)?;

        Ok(())
    }
}

/// Collect todo comments from the files
pub(crate) fn collect_todos(
    files: &[PathBuf],
    project_root: &PathBuf,
) -> anyhow::Result<Vec<TodoComment>> {
    let set = Regex::new(
        r"(?x)(?P<verb>NOW|TODO|FIXME|FUTURE|NOFUTURE|NOTE)\((?P<username>[^)]+)\)([:\s]+)(?P<task>.*)$",
    )?;

    let mut todo = Vec::new();
    for file in files {
        let contents = std::fs::read(file)?;
        let contents = String::from_utf8_lossy(&contents);
        for (ln, line) in contents.lines().enumerate() {
            if let Some(captures) = set.captures(line) {
                if let Some((verb, username, task)) = captures.name("verb").and_then(|verb| {
                    captures.name("username").and_then(|username| {
                        captures.name("task").map(|task| (verb, username, task))
                    })
                }) {
                    let verb = Verb::from_str(verb.as_str()).expect("Verb");
                    let username = username.as_str();
                    let task = task.as_str();

                    let file = file.strip_prefix(project_root)?;

                    todo.push(TodoComment {
                        file: file.to_owned(),
                        line: ln + 1,
                        verb,
                        username: username.to_string(),
                        task: task.to_string(),
                    });
                }
            }
        }
    }
    Ok(todo)
}
