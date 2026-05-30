use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROJECT_INDEX_FILE: &str = "project.boundline.toml";
pub const DEFAULT_PROJECT_MEMORY_ROOT: &str = "docs/project";
pub const DEFAULT_EVIDENCE_ROOT: &str = "docs/evidence";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIndex {
    pub project: ProjectIndexProject,
    #[serde(default)]
    pub docs: ProjectIndexDocs,
    #[serde(default)]
    pub systems: BTreeMap<String, ProjectIndexSystem>,
}

impl ProjectIndex {
    pub fn from_toml_str(contents: &str) -> Result<Self, ProjectIndexError> {
        let index: Self = toml::from_str(contents).map_err(ProjectIndexError::ParseProjectIndex)?;
        index.validate()?;
        Ok(index)
    }

    pub fn load(workspace_root: &Path) -> Result<Option<Self>, ProjectIndexError> {
        let path = workspace_root.join(PROJECT_INDEX_FILE);
        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path)
            .map_err(|source| ProjectIndexError::ReadProjectIndex { path: path.clone(), source })?;
        Self::from_toml_str(&contents).map(Some)
    }

    pub fn doc_roots(&self) -> ProjectDocRoots {
        self.docs.doc_roots()
    }

    pub fn system(&self, system_name: &str) -> Option<&ProjectIndexSystem> {
        self.systems.get(system_name)
    }

    pub fn validate(&self) -> Result<(), ProjectIndexError> {
        if self.project.name.trim().is_empty() {
            return Err(ProjectIndexError::MissingProjectName);
        }

        self.docs.validate()?;

        for (system_name, system) in &self.systems {
            system.validate(system_name)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIndexProject {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primary_domains: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIndexDocs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_memory: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<PathBuf>,
}

impl ProjectIndexDocs {
    pub fn doc_roots(&self) -> ProjectDocRoots {
        ProjectDocRoots {
            project_memory: self
                .project_memory
                .clone()
                .unwrap_or_else(|| PathBuf::from(DEFAULT_PROJECT_MEMORY_ROOT)),
            evidence: self.evidence.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_EVIDENCE_ROOT)),
        }
    }

    fn validate(&self) -> Result<(), ProjectIndexError> {
        validate_relative_path("docs.project_memory", self.project_memory.as_deref())?;
        validate_relative_path("docs.evidence", self.evidence.as_deref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectIndexSystem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality: Option<String>,
}

impl ProjectIndexSystem {
    fn validate(&self, system_name: &str) -> Result<(), ProjectIndexError> {
        for (index, path) in self.paths.iter().enumerate() {
            validate_relative_path(
                &format!("systems.{system_name}.paths[{index}]"),
                Some(path.as_path()),
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDocRoots {
    pub project_memory: PathBuf,
    pub evidence: PathBuf,
}

impl Default for ProjectDocRoots {
    fn default() -> Self {
        Self {
            project_memory: PathBuf::from(DEFAULT_PROJECT_MEMORY_ROOT),
            evidence: PathBuf::from(DEFAULT_EVIDENCE_ROOT),
        }
    }
}

impl ProjectDocRoots {
    pub fn project_memory_dir(&self, workspace_root: &Path) -> PathBuf {
        workspace_root.join(&self.project_memory)
    }

    pub fn evidence_dir(&self, workspace_root: &Path) -> PathBuf {
        workspace_root.join(&self.evidence)
    }
}

pub fn resolve_project_doc_roots(
    workspace_root: &Path,
) -> Result<ProjectDocRoots, ProjectIndexError> {
    Ok(ProjectIndex::load(workspace_root)?.map(|index| index.doc_roots()).unwrap_or_default())
}

fn validate_relative_path(field: &str, path: Option<&Path>) -> Result<(), ProjectIndexError> {
    let Some(path) = path else {
        return Ok(());
    };

    if path.as_os_str().is_empty() {
        return Err(ProjectIndexError::EmptyPath { field: field.to_string() });
    }

    if path.is_absolute() {
        return Err(ProjectIndexError::AbsolutePath { field: field.to_string() });
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum ProjectIndexError {
    #[error("project index could not be read from {path}: {source}")]
    ReadProjectIndex {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("project index could not be parsed: {0}")]
    ParseProjectIndex(toml::de::Error),
    #[error("project index project.name must not be empty")]
    MissingProjectName,
    #[error("project index field `{field}` must not be empty")]
    EmptyPath { field: String },
    #[error("project index field `{field}` must be relative to the repository root")]
    AbsolutePath { field: String },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ProjectDocRoots, ProjectIndex, ProjectIndexError};

    #[test]
    fn parses_project_index_contract_shape_and_docs_overrides() {
        let index = ProjectIndex::from_toml_str(
            r#"
[project]
name = "boundline"
primary_domains = ["delivery-control"]

[docs]
project_memory = "knowledge/project"
evidence = "knowledge/evidence"

[systems.checkout]
workspace = "web-app"
paths = ["apps/checkout", "packages/payment"]
owner = "checkout-team"
domain = "commerce"
criticality = "high"
"#,
        )
        .unwrap();

        assert_eq!(index.project.name, "boundline");
        assert_eq!(index.project.primary_domains, vec!["delivery-control"]);
        assert_eq!(
            index.doc_roots(),
            ProjectDocRoots {
                project_memory: PathBuf::from("knowledge/project"),
                evidence: PathBuf::from("knowledge/evidence"),
            }
        );

        let checkout = index.system("checkout").unwrap();
        assert_eq!(checkout.workspace.as_deref(), Some("web-app"));
        assert_eq!(
            checkout.paths,
            vec![PathBuf::from("apps/checkout"), PathBuf::from("packages/payment")]
        );
        assert_eq!(checkout.owner.as_deref(), Some("checkout-team"));
        assert_eq!(checkout.domain.as_deref(), Some("commerce"));
        assert_eq!(checkout.criticality.as_deref(), Some("high"));
    }

    #[test]
    fn defaults_doc_roots_when_docs_section_is_absent() {
        let index = ProjectIndex::from_toml_str(
            r#"
[project]
name = "boundline"
"#,
        )
        .unwrap();

        assert_eq!(index.doc_roots(), ProjectDocRoots::default());
    }

    #[test]
    fn rejects_absolute_project_index_paths() {
        let error = ProjectIndex::from_toml_str(
            r#"
[project]
name = "boundline"

[docs]
project_memory = "/tmp/project"
"#,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            ProjectIndexError::AbsolutePath { field } if field == "docs.project_memory"
        ));
    }
}
