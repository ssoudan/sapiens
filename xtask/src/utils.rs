//! Rust project exploration utils

use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Project Visitor trait
///
/// By defaule the visitor does nothing but visit the project toml, the crates
/// and their tomls in this order. Override the methods to implement custom
/// behavior and don't forget to call the other methods if you want to keep the
/// default behavior.
///
/// # Example
///
/// ```rust
/// use xtask::utils::{Crate, Project, Visitor};
///
/// struct MyVisitor;
///
/// impl Visitor for MyVisitor {
///     fn visit_project_toml(&self, toml: &toml::Value) -> anyhow::Result<()> {
///         println!("Visiting project toml");
///         Ok(())
///     }
///
///     fn visit_crate_toml(&self, toml: &toml::Value) -> anyhow::Result<()> {
///         println!("Visiting crate toml");
///         Ok(())
///     }
/// }
///
/// let project = Project::default();
/// let visitor = MyVisitor;
/// visitor.visit_project(&project).unwrap();
/// ```
pub(crate) trait Visitor {
    /// Visit a project
    ///
    /// Default implementation calls `visit_project_toml` and `visit_crate` for
    /// each crate in the workspace.
    ///
    /// Override to implement custom behavior.
    fn visit_project(&mut self, project: &Project) -> anyhow::Result<()> {
        self.visit_project_toml(&project.root_cargo_toml)?;

        for crate_ in project.workspace_members() {
            self.visit_crate(&crate_)?;
        }
        Ok(())
    }
    /// Visit the root `Cargo.toml`
    ///
    /// Default implementation does nothing.
    ///
    /// Override to implement custom behavior.
    #[allow(unused_variables)]
    fn visit_project_toml(&mut self, toml: &CargoToml) -> anyhow::Result<()> {
        Ok(())
    }
    /// Visit a crate
    ///
    /// Default implementation calls `visit_crate_toml` with the toml of the
    /// crate.
    ///
    /// Override to implement custom behavior.
    fn visit_crate(&mut self, crate_: &Crate) -> anyhow::Result<()> {
        self.visit_crate_toml(&crate_.toml()?)
    }
    /// Visit the `Cargo.toml` of a crate
    ///
    /// Default implementation does nothing.
    ///
    /// Override to implement custom behavior.
    #[allow(unused_variables)]
    fn visit_crate_toml(&mut self, toml: &CargoToml) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Returns the path to the root directory of the project.
#[allow(clippy::expect_used)]
#[must_use]
pub(crate) fn project_root() -> PathBuf {
    let dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());
    PathBuf::from(dir)
        .parent()
        .expect("no parent dir")
        .to_owned()
}

/// Returns the path to the root `Cargo.toml`.
fn root_cargo_path() -> PathBuf {
    project_root().join("Cargo.toml")
}

/// The `Cargo.toml` file as a `toml::Value`.
pub(crate) type CargoToml = toml::Value;

/// Project
pub(crate) struct Project {
    /// Root directory of the workspace
    root: PathBuf,
    /// `Cargo.toml` of the root
    root_cargo_toml: CargoToml,
}

impl Default for Project {
    fn default() -> Self {
        let root = project_root();
        let root_cargo_toml = root_cargo_toml();
        Self {
            root,
            root_cargo_toml,
        }
    }
}

/// Crate in the workspace
pub(crate) struct Crate {
    /// Root directory of the workspace
    root: PathBuf,
    /// Name of the crate - the directory name
    pub(crate) name: String,
    /// Absolute path to the crate directory
    pub(crate) path: PathBuf,
}

impl Display for Crate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rel_path = self
            .path
            .strip_prefix(&self.root)
            .unwrap_or_else(|_| &self.path);

        write!(f, "{} ({})", self.name, rel_path.display())
    }
}

impl Crate {
    /// Returns the `toml::Value` of the `Cargo.toml` of the crate.
    pub(crate) fn toml(&self) -> anyhow::Result<CargoToml> {
        let toml_str = std::fs::read_to_string(self.path.join("Cargo.toml"))?;
        Ok(toml_str.parse()?)
    }

    /// Returns all the files (recursively) in the crate
    pub(crate) fn files(&self) -> anyhow::Result<Vec<PathBuf>> {
        let dir = self.path.join("src/**/*");
        Ok(glob::glob(dir.to_str().expect("invalid pattern"))?
            .flatten()
            // keep only files
            .filter(|p| p.is_file())
            .collect())
    }
}

impl Project {
    /// Returns the root path of the project.
    #[must_use]
    pub(crate) const fn root_path(&self) -> &PathBuf {
        &self.root
    }

    /// True if project is using a workspace
    #[must_use]
    pub(crate) fn is_workspace(&self) -> bool {
        self.root_cargo_toml.get("workspace").is_some()
    }

    /// List of crates in the workspace
    #[must_use]
    pub(crate) fn workspace_members(&self) -> Vec<Crate> {
        let glob_patterns: Vec<String> = self
            .root_cargo_toml
            .get("workspace")
            .and_then(|t| t.get("members"))
            .map(|members| {
                members
                    .as_array()
                    .expect("workspace.members should be an array")
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .expect("workspace.members should be an array of strings")
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();

        // capture all the directories matching the glob with a Cargo.toml
        let mut members = Vec::new();
        for pattern in glob_patterns {
            let pattern = self.root.join(pattern).join("Cargo.toml");
            for path in glob::glob(pattern.to_str().expect("invalid pattern"))
                .expect("glob failed")
                .flatten()
            {
                if path.is_file() {
                    let dir_name = path
                        .parent()
                        .expect("Cargo.toml should be in a directory")
                        .file_name()
                        .expect("directory should have a name")
                        .to_string_lossy()
                        .to_string();
                    let member = Crate {
                        root: self.root.clone(),
                        name: dir_name,
                        path: path
                            .parent()
                            .expect("Cargo.toml should be in a directory")
                            .to_owned(),
                    };
                    members.push(member);
                }
            }
        }
        members
    }
}

/// Returns the `toml::Value` of the root `Cargo.toml`.
fn root_cargo_toml() -> CargoToml {
    let toml_str = std::fs::read_to_string(root_cargo_path()).expect("could not read Cargo.toml");
    toml_str.parse().expect("could not parse Cargo.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_root() {
        let root = project_root();
        assert!(root.join("Cargo.toml").exists());
    }

    #[test]
    fn test_root_cargo_path() {
        let path = root_cargo_path();
        assert!(path.exists());
    }

    #[test]
    fn test_project_project_root() {
        let project = Project::default();
        assert!(project.root.join("Cargo.toml").exists());
    }

    #[test]
    fn test_project_is_workspace() {
        let project = Project::default();
        assert!(project.is_workspace());
    }

    #[test]
    fn test_project_workspace_members() {
        let project = Project::default();
        let members = project.workspace_members();
        assert!(!members.is_empty());
        assert!(members
            .iter()
            .map(|c| c.name.clone())
            .collect::<String>()
            .contains("xtask"));

        let members_map = members
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect::<std::collections::HashMap<String, Crate>>();

        assert!(members_map.contains_key("xtask"));
        let xtask_crate = members_map.get("xtask").unwrap();

        #[allow(clippy::cmp_owned)]
        let files = xtask_crate
            .files()
            .unwrap()
            .iter()
            .map(|p| {
                p.strip_prefix(&xtask_crate.path)
                    .expect("strip_prefix failed")
                    .to_owned()
            })
            .any(|file| file == PathBuf::from("src/main.rs"));
        // println!("{:?}", files);
        assert!(files);
    }
}
