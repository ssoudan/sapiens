//! Validate project configuration

use std::path::PathBuf;

use xshell::Shell;

use crate::flags;
use crate::todo::{collect_todos, Verb};
use crate::utils::{Project, Visitor};

/// Package type
enum PackageType {
    /// Normal package
    Normal,
    /// Tauri plugin
    TauriPlugin,
}

/// Get the package type
fn get_package_type(toml: &toml::Value) -> PackageType {
    if let Some(dev_dependencies) = toml.get("build-dependencies") {
        if dev_dependencies
            .get("tauri-plugin")
            .or_else(|| dev_dependencies.get("tauri-plugin"))
            .is_some()
        {
            return PackageType::TauriPlugin;
        }
    }

    PackageType::Normal
}

impl flags::Validate {
    /// Run the validate command
    pub(crate) fn run(self, _sh: &Shell, project: &Project) -> anyhow::Result<()> {
        let mut validate: Validate = self.into();
        validate.visit_project(project)
    }
}

/// Validate visitor
struct Validate {
    /// Files to validate.
    files: Vec<PathBuf>,
    /// Check pre commit conditions
    strict: bool,
}

impl From<flags::Validate> for Validate {
    fn from(flags: flags::Validate) -> Self {
        Self {
            files: vec![],
            strict: flags.strict,
        }
    }
}

impl Visitor for Validate {
    fn visit_project(&mut self, project: &crate::utils::Project) -> anyhow::Result<()> {
        /// Release forbidden verbs
        const RELEASE_FORBIDDEN_VERBS: &[Verb] = &[Verb::Now, Verb::ToDo, Verb::FixMe];

        // Project is a workspace
        if !project.is_workspace() {
            return Err(anyhow::anyhow!("Project is not a workspace"));
        }

        for crate_ in project.workspace_members() {
            self.visit_crate(&crate_)?;
        }

        let todos = collect_todos(&self.files, project.root_path())?;

        for todo in todos {
            if RELEASE_FORBIDDEN_VERBS.contains(&todo.verb) {
                if self.strict {
                    return Err(anyhow::anyhow!("Incomplete task found:\n{todo}"));
                }

                todo.print(1);
            }
        }

        Ok(())
    }

    fn visit_project_toml(&mut self, _toml: &crate::utils::CargoToml) -> anyhow::Result<()> {
        Ok(())
    }

    fn visit_crate(&mut self, crate_: &crate::utils::Crate) -> anyhow::Result<()> {
        /// Inherited package settings
        const INHERITED: &[&str] = &["edition", "authors", "license"];

        println!("Checking {crate_}");

        let toml = crate_.toml()?;

        let package_type = get_package_type(&toml);

        // Check the `package` section
        let package = toml
            .get("package")
            .ok_or_else(|| anyhow::anyhow!("Missing `package` section in {crate_}"))?;

        // Ensure the package name is present, is in snake_case (unless it is a Tauri
        // plugin), and is the same as the directory name
        let name = package
            .get("name")
            .ok_or_else(|| anyhow::anyhow!("Missing `package.name` in {crate_}"))?;

        let name = name
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("`package.name` is not a string in {crate_}"))?;

        if name != crate_.name {
            return Err(anyhow::anyhow!(
                "`package.name` does not match directory name in {crate_}"
            ));
        }

        match package_type {
            PackageType::Normal => {
                if !name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                    return Err(anyhow::anyhow!(
                        "`package.name` is not in snake_case in {crate_}"
                    ));
                }
            }
            PackageType::TauriPlugin => {
                if !name.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
                    return Err(anyhow::anyhow!(
                        "Tauri Plugin `package.name` is not in kebab-case in {crate_}"
                    ));
                }
            }
        }

        // ensure the description is present
        let _description = package
            .get("description")
            .ok_or_else(|| anyhow::anyhow!("Missing `package.description` in {crate_}"))?;

        // ensure inherited package settings have 'workspace' set to true
        for field in INHERITED {
            let value = package
                .get(field)
                .ok_or_else(|| anyhow::anyhow!("Missing `package.{field}` in {crate_}"))?;

            let value = value.get("workspace").ok_or_else(|| {
                anyhow::anyhow!("Missing `package.{field}.workspace` in {crate_}")
            })?;
            let value = value.as_bool().ok_or_else(|| {
                anyhow::anyhow!("`package.{field}.workspace` is not a boolean in {crate_}")
            })?;
            if !value {
                return Err(anyhow::anyhow!(
                    "`package.{field}.workspace` is not true in {crate_}"
                ));
            }
        }

        // Check for the presence of the `lints` section
        let lints = toml
            .get("lints")
            .ok_or_else(|| anyhow::anyhow!("Missing `lints` section in {crate_}"))?;
        let workspace = lints
            .get("workspace")
            .ok_or_else(|| anyhow::anyhow!("Missing `lints.workspace` in {crate_}"))?;
        let value = workspace
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("`lints.workspace` is not a boolean in {crate_}"))?;
        if !value {
            return Err(anyhow::anyhow!("`lints.workspace` is not true in {crate_}"));
        }

        // collect the files to validate
        self.files.extend_from_slice(&crate_.files()?);

        Ok(())
    }
}
