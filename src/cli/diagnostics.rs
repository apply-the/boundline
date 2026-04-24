use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::adapters::trace_store::FileTraceStore;
use crate::demo::endpoints::build_demo_runtime;
use crate::demo::profile::DemoRunProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsCheck {
    pub name: String,
    pub status: DiagnosticsStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub workspace_ref: String,
    pub checks: Vec<DiagnosticsCheck>,
    pub ready: bool,
    pub missing_prerequisites: Vec<String>,
    pub suggested_actions: Vec<String>,
}

pub fn diagnose_workspace(workspace_ref: impl AsRef<Path>) -> DiagnosticsReport {
    let workspace = workspace_ref.as_ref();
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let mut checks = Vec::new();

    let workspace_exists = workspace.exists() && workspace.is_dir();
    checks.push(if workspace_exists {
        DiagnosticsCheck {
            name: "workspace_exists".to_string(),
            status: DiagnosticsStatus::Passed,
            message: format!("workspace is available at {workspace_ref}"),
        }
    } else {
        DiagnosticsCheck {
            name: "workspace_exists".to_string(),
            status: DiagnosticsStatus::Failed,
            message: "point --workspace to an existing local directory".to_string(),
        }
    });

    let repository_root = workspace_exists && workspace.join("Cargo.toml").is_file();
    checks.push(if repository_root {
        DiagnosticsCheck {
            name: "repository_root".to_string(),
            status: DiagnosticsStatus::Passed,
            message: "workspace contains the Synod Cargo manifest".to_string(),
        }
    } else {
        DiagnosticsCheck {
            name: "repository_root".to_string(),
            status: DiagnosticsStatus::Failed,
            message: "run the command from the repository root containing Cargo.toml".to_string(),
        }
    });

    let workspace_writable = if workspace_exists {
        fs::metadata(workspace).map(|metadata| !metadata.permissions().readonly()).unwrap_or(false)
    } else {
        false
    };
    checks.push(if workspace_writable {
        DiagnosticsCheck {
            name: "workspace_writable".to_string(),
            status: DiagnosticsStatus::Passed,
            message: "workspace permissions allow local trace persistence".to_string(),
        }
    } else {
        DiagnosticsCheck {
            name: "workspace_writable".to_string(),
            status: DiagnosticsStatus::Failed,
            message: "ensure the workspace directory is writable before starting a run".to_string(),
        }
    });

    let trace_store = FileTraceStore::for_workspace(workspace);
    let trace_root = trace_store.root();
    let trace_root_ready =
        if trace_root.exists() { trace_root.is_dir() } else { workspace_writable };
    checks.push(if trace_root_ready {
        DiagnosticsCheck {
            name: "trace_store".to_string(),
            status: DiagnosticsStatus::Passed,
            message: if trace_root.exists() {
                format!("trace directory is ready at {}", trace_root.display())
            } else {
                format!("trace directory will be created on first run at {}", trace_root.display())
            },
        }
    } else {
        DiagnosticsCheck {
            name: "trace_store".to_string(),
            status: DiagnosticsStatus::Failed,
            message: format!(
                "clear or fix the trace path at {} so Synod can persist run traces",
                trace_root.display()
            ),
        }
    });

    checks.push(
        match (
            build_demo_runtime(DemoRunProfile::guided_demo()),
            build_demo_runtime(DemoRunProfile::default_run("Validate the default developer flow")),
        ) {
            (Ok(_), Ok(_)) => DiagnosticsCheck {
                name: "built_in_flow".to_string(),
                status: DiagnosticsStatus::Passed,
                message: "built-in demo and default developer flow are available".to_string(),
            },
            (Err(error), _) | (_, Err(error)) => DiagnosticsCheck {
                name: "built_in_flow".to_string(),
                status: DiagnosticsStatus::Failed,
                message: format!("built-in developer flow is unavailable: {error}"),
            },
        },
    );

    let missing_prerequisites = checks
        .iter()
        .filter(|check| check.status == DiagnosticsStatus::Failed)
        .map(|check| check.name.clone())
        .collect::<Vec<_>>();
    let suggested_actions = checks
        .iter()
        .filter(|check| check.status == DiagnosticsStatus::Failed)
        .map(|check| check.message.clone())
        .collect::<Vec<_>>();

    DiagnosticsReport {
        workspace_ref,
        ready: missing_prerequisites.is_empty(),
        checks,
        missing_prerequisites,
        suggested_actions,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{DiagnosticsStatus, diagnose_workspace};

    fn temp_workspace() -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("synod-diagnostics-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    #[test]
    fn diagnostics_report_marks_a_writable_workspace_as_ready() {
        let workspace = temp_workspace();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"synod\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        let report = diagnose_workspace(&workspace);

        assert!(report.ready);
        assert!(report.missing_prerequisites.is_empty());
        assert!(report.checks.iter().all(|check| check.status == DiagnosticsStatus::Passed));
    }

    #[test]
    fn diagnostics_report_flags_a_missing_workspace() {
        let workspace = std::env::temp_dir().join(format!("synod-missing-{}", Uuid::new_v4()));
        let report = diagnose_workspace(&workspace);

        assert!(!report.ready);
        assert!(report.missing_prerequisites.contains(&"workspace_exists".to_string()));
    }

    #[test]
    fn diagnostics_report_flags_an_invalid_trace_root() {
        let workspace = temp_workspace();
        let trace_root = workspace.join(".synod").join("traces");
        fs::create_dir_all(trace_root.parent().unwrap()).unwrap();
        fs::write(&trace_root, "not-a-directory").unwrap();

        let report = diagnose_workspace(&workspace);

        assert!(!report.ready);
        assert!(report.missing_prerequisites.contains(&"trace_store".to_string()));
    }
}
