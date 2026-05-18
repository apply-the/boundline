use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::adapters::trace_store::FileTraceStore;
use crate::domain::distribution::{
    CanonInstallStatus, CompanionState, SUPPORTED_CANON_VERSION, evaluate_canon_install,
    supported_distribution_channels,
};
use crate::domain::project_memory::read_project_memory;
use crate::fixture::FixtureRuntimeError;
use crate::fixture::load_workspace_execution_profile;

const ADVANCED_CONTEXT_INDEX_RELATIVE: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3";
const CANON_GUIDANCE_DIR_RELATIVE: &str = ".canon/boundline/guidance";
const SESSION_RECORD_RELATIVE: &str = ".boundline/session.json";
const WORKSPACE_CONFIG_RELATIVE: &str = ".boundline/config.toml";
const WORKSPACE_GUIDANCE_DIR_RELATIVE: &str = ".boundline/guidance";
const WORKSPACE_GUARDIAN_DIR_RELATIVE: &str = ".boundline/guardians";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsStatus {
    Passed,
    Advisory,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsSubject {
    Workspace,
    Install,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsCheck {
    pub name: String,
    pub status: DiagnosticsStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub subject: DiagnosticsSubject,
    pub workspace_ref: Option<String>,
    pub installation_ref: Option<String>,
    pub checks: Vec<DiagnosticsCheck>,
    pub ready: bool,
    pub missing_prerequisites: Vec<String>,
    pub suggested_actions: Vec<String>,
    pub boundline_version: Option<String>,
    pub supported_canon_version: Option<String>,
    pub companion_state: Option<CompanionState>,
    pub channel_candidates: Vec<String>,
}

struct DiagnosticsReportContext {
    subject: DiagnosticsSubject,
    workspace_ref: Option<String>,
    installation_ref: Option<String>,
    boundline_version: Option<String>,
    supported_canon_version: Option<String>,
    companion_state: Option<CompanionState>,
    channel_candidates: Vec<String>,
}

pub fn diagnose_workspace(workspace_ref: impl AsRef<Path>) -> DiagnosticsReport {
    diagnose_workspace_with_profile_requirement(workspace_ref, true)
}

pub fn diagnose_workspace_context(workspace_ref: impl AsRef<Path>) -> DiagnosticsReport {
    let workspace = workspace_ref.as_ref();
    let mut report = diagnose_workspace_with_profile_requirement(workspace, true);
    extend_workspace_context_diagnostics(&mut report, workspace);
    report
}

pub fn diagnose_installation() -> DiagnosticsReport {
    diagnose_installation_from_current_exe(std::env::current_exe())
}

fn diagnose_installation_from_current_exe(
    current_exe: Result<std::path::PathBuf, std::io::Error>,
) -> DiagnosticsReport {
    let mut checks = Vec::new();
    let channel_candidates = supported_distribution_channels()
        .into_iter()
        .map(|channel| channel.to_string())
        .collect::<Vec<_>>();
    let mut suggested_actions = Vec::new();

    let installation_ref = match current_exe {
        Ok(executable) => {
            checks.push(DiagnosticsCheck {
                name: "boundline_binary".to_string(),
                status: DiagnosticsStatus::Passed,
                message: format!("Boundline binary is available at {}", executable.display()),
            });
            checks.push(DiagnosticsCheck {
                name: "distribution_channel".to_string(),
                status: DiagnosticsStatus::Passed,
                message: distribution_channel_message(&channel_candidates),
            });

            let canon_status = evaluate_canon_install(&executable);
            if matches!(canon_status.state, CompanionState::Blocked | CompanionState::RepairNeeded)
            {
                suggested_actions.extend(canon_status.suggested_actions.clone());
            }
            if let Some(location) = canon_status.location.as_ref() {
                checks.push(DiagnosticsCheck {
                    name: "canon_path".to_string(),
                    status: DiagnosticsStatus::Passed,
                    message: format!("authoritative Canon binary path: {}", location.display()),
                });
            } else {
                checks.push(DiagnosticsCheck {
                    name: "canon_path".to_string(),
                    status: DiagnosticsStatus::Failed,
                    message: "Canon binary path could not be resolved".to_string(),
                });
            }
            if let Some(surface) = canon_status.surface_verification.as_ref() {
                checks.push(DiagnosticsCheck {
                    name: "canon_governance_surface".to_string(),
                    status: if surface.operations_verified {
                        DiagnosticsStatus::Passed
                    } else {
                        DiagnosticsStatus::Failed
                    },
                    message: if surface.operations_verified {
                        "Canon governance operations are available".to_string()
                    } else {
                        format!(
                            "Canon governance operations missing: {}",
                            surface.missing_operations.join(", ")
                        )
                    },
                });
                checks.push(DiagnosticsCheck {
                    name: "canon_modes".to_string(),
                    status: if surface.modes_verified {
                        DiagnosticsStatus::Passed
                    } else {
                        DiagnosticsStatus::Failed
                    },
                    message: if surface.modes_verified {
                        "Canon exposes all canonical modes".to_string()
                    } else {
                        format!(
                            "Canon modes missing: {}",
                            surface
                                .missing_modes
                                .iter()
                                .map(|mode| mode
                                    .primary_document_name()
                                    .trim_end_matches(".md")
                                    .to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    },
                });
            } else {
                checks.push(DiagnosticsCheck {
                    name: "canon_governance_surface".to_string(),
                    status: DiagnosticsStatus::Failed,
                    message: "Canon governance capabilities could not be queried".to_string(),
                });
                checks.push(DiagnosticsCheck {
                    name: "canon_modes".to_string(),
                    status: DiagnosticsStatus::Failed,
                    message: "Canon mode capabilities could not be queried".to_string(),
                });
            }
            checks.push(DiagnosticsCheck {
                name: "canon_companion".to_string(),
                status: if matches!(
                    canon_status.state,
                    CompanionState::Ready | CompanionState::AlreadySatisfied
                ) {
                    DiagnosticsStatus::Passed
                } else {
                    DiagnosticsStatus::Failed
                },
                message: canon_status.message,
            });

            return finalize_report(
                checks,
                suggested_actions,
                DiagnosticsReportContext {
                    subject: DiagnosticsSubject::Install,
                    workspace_ref: None,
                    installation_ref: Some(executable.display().to_string()),
                    boundline_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    supported_canon_version: Some(SUPPORTED_CANON_VERSION.to_string()),
                    companion_state: Some(canon_status.state),
                    channel_candidates,
                },
            );
        }
        Err(error) => {
            checks.push(DiagnosticsCheck {
                name: "boundline_binary".to_string(),
                status: DiagnosticsStatus::Failed,
                message: format!(
                    "resolve the current Boundline executable location before rerunning install diagnostics: {error}"
                ),
            });
            checks.push(DiagnosticsCheck {
                name: "distribution_channel".to_string(),
                status: DiagnosticsStatus::Passed,
                message: distribution_channel_message(&channel_candidates),
            });
            suggested_actions.push(
                "rerun `boundline doctor --install` from the installed Boundline executable in a normal shell"
                    .to_string(),
            );
            None
        }
    };

    finalize_report(
        checks,
        suggested_actions,
        DiagnosticsReportContext {
            subject: DiagnosticsSubject::Install,
            workspace_ref: None,
            installation_ref,
            boundline_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            supported_canon_version: Some(SUPPORTED_CANON_VERSION.to_string()),
            companion_state: Some(CompanionState::Blocked),
            channel_candidates,
        },
    )
}

pub fn diagnose_native_direct_run_workspace(workspace_ref: impl AsRef<Path>) -> DiagnosticsReport {
    diagnose_workspace_with_profile_requirement(workspace_ref, false)
}

fn diagnose_workspace_with_profile_requirement(
    workspace_ref: impl AsRef<Path>,
    require_execution_profile: bool,
) -> DiagnosticsReport {
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
                "clear or fix the trace path at {} so Boundline can persist run traces",
                trace_root.display()
            ),
        }
    });

    checks.push(match load_workspace_execution_profile(workspace) {
        Ok(profile) => DiagnosticsCheck {
            name: "workspace_execution_profile".to_string(),
            status: DiagnosticsStatus::Passed,
            message: format!(
                "execution profile '{}' is ready at {}",
                profile.name,
                workspace.join(".boundline/execution.json").display(),
            ),
        },
        Err(FixtureRuntimeError::MissingExecutionProfile(_)) if require_execution_profile => {
            DiagnosticsCheck {
                name: "workspace_execution_profile".to_string(),
                status: DiagnosticsStatus::Failed,
                message: format!(
                    "run `boundline init --workspace {}` to create the workspace profile",
                    workspace.display()
                ),
            }
        }
        Err(FixtureRuntimeError::MissingExecutionProfile(_)) => DiagnosticsCheck {
            name: "workspace_execution_profile".to_string(),
            status: DiagnosticsStatus::Passed,
            message: "execution profile is optional for native direct run".to_string(),
        },
        Err(error) if require_execution_profile => DiagnosticsCheck {
            name: "workspace_execution_profile".to_string(),
            status: DiagnosticsStatus::Failed,
            message: format!("workspace execution profile is unavailable: {error}"),
        },
        Err(error) => DiagnosticsCheck {
            name: "workspace_execution_profile".to_string(),
            status: DiagnosticsStatus::Passed,
            message: format!(
                "execution profile is optional for native direct run; current profile state is ignored: {error}"
            ),
        },
    });

    finalize_report(
        checks,
        Vec::new(),
        DiagnosticsReportContext {
            subject: DiagnosticsSubject::Workspace,
            workspace_ref: Some(workspace_ref),
            installation_ref: None,
            boundline_version: None,
            supported_canon_version: None,
            companion_state: None,
            channel_candidates: Vec::new(),
        },
    )
}

fn extend_workspace_context_diagnostics(report: &mut DiagnosticsReport, workspace: &Path) {
    if !(workspace.exists() && workspace.is_dir()) {
        return;
    }

    let workspace_ref = workspace.to_string_lossy().into_owned();
    let config_path = workspace.join(WORKSPACE_CONFIG_RELATIVE);
    push_context_check(
        report,
        DiagnosticsCheck {
            name: "boundline_config".to_string(),
            status: if config_path.is_file() {
                DiagnosticsStatus::Passed
            } else {
                DiagnosticsStatus::Advisory
            },
            message: if config_path.is_file() {
                format!("workspace config is available at {}", config_path.display())
            } else {
                format!(
                    "workspace config is missing; inspect or create it with `boundline config show --workspace {} --scope effective`",
                    workspace.display()
                )
            },
        },
        (!config_path.is_file()).then(|| {
            format!("boundline config show --workspace {workspace_ref} --scope effective")
        }),
    );

    let project_memory = read_project_memory(workspace);
    let project_memory_available = project_memory.has_credible_memory();
    let project_memory_message = project_memory
        .condition()
        .map(|condition| condition.headline().to_string())
        .unwrap_or_else(|| {
            "Canon project memory is missing; govern a stage or refresh project memory after Canon promotes a stable docs/project surface"
                .to_string()
        });
    push_context_check(
        report,
        DiagnosticsCheck {
            name: "canon_project_memory".to_string(),
            status: if project_memory_available {
                DiagnosticsStatus::Passed
            } else {
                DiagnosticsStatus::Advisory
            },
            message: project_memory_message,
        },
        (!project_memory_available)
            .then(|| format!("boundline govern --workspace {workspace_ref}")),
    );

    let expert_pack_ready = [
        workspace.join(WORKSPACE_GUIDANCE_DIR_RELATIVE),
        workspace.join(WORKSPACE_GUARDIAN_DIR_RELATIVE),
        workspace.join(CANON_GUIDANCE_DIR_RELATIVE),
    ]
    .iter()
    .any(|path| path_has_entries(path));
    push_context_check(
        report,
        DiagnosticsCheck {
            name: "expert_pack_inputs".to_string(),
            status: if expert_pack_ready {
                DiagnosticsStatus::Passed
            } else {
                DiagnosticsStatus::Advisory
            },
            message: if expert_pack_ready {
                "workspace or Canon guidance inputs are available for expert-pack calibration"
                    .to_string()
            } else {
                "expert-pack inputs are missing; add workspace domain guidance or bind a required context before relying on deeper S7 routing".to_string()
            },
        },
        (!expert_pack_ready).then(|| {
            "boundline config set-domain --scope workspace --family <family> --enable --standards \"<standards>\""
                .to_string()
        }),
    );

    let (provider_status, provider_message, provider_actions) =
        provider_readiness_context(workspace);
    push_context_check(
        report,
        DiagnosticsCheck {
            name: "provider_readiness".to_string(),
            status: provider_status,
            message: provider_message,
        },
        None,
    );
    for action in provider_actions {
        push_suggested_action(report, action);
    }

    let advanced_context_index = workspace.join(ADVANCED_CONTEXT_INDEX_RELATIVE);
    let advanced_context_ready = advanced_context_index.is_file();
    push_context_check(
        report,
        DiagnosticsCheck {
            name: "advanced_context_index".to_string(),
            status: if advanced_context_ready {
                DiagnosticsStatus::Passed
            } else {
                DiagnosticsStatus::Advisory
            },
            message: if advanced_context_ready {
                format!(
                    "advanced-context index is available at {}",
                    advanced_context_index.display()
                )
            } else {
                "advanced-context index is missing; enable local semantic acceleration before expecting higher-order impact inference"
                    .to_string()
            },
        },
        (!advanced_context_ready).then(|| {
            "boundline config set-semantic-acceleration --scope workspace --policy local"
                .to_string()
        }),
    );

    let trace_root = FileTraceStore::for_workspace(workspace).root().to_path_buf();
    let has_session_evidence =
        workspace.join(SESSION_RECORD_RELATIVE).is_file() || path_has_entries(&trace_root);
    push_context_check(
        report,
        DiagnosticsCheck {
            name: "session_evidence".to_string(),
            status: if has_session_evidence {
                DiagnosticsStatus::Passed
            } else {
                DiagnosticsStatus::Advisory
            },
            message: if has_session_evidence {
                "session or trace evidence is available for S7 explanations".to_string()
            } else {
                "session evidence is missing; start a session before expecting `why`, `risk`, or `next-best` to cite live runtime context".to_string()
            },
        },
        (!has_session_evidence).then(|| format!("boundline start --workspace {workspace_ref}")),
    );
}

fn path_has_entries(path: &Path) -> bool {
    path.read_dir().ok().and_then(|mut entries| entries.next()).is_some()
}

fn provider_readiness_context(workspace: &Path) -> (DiagnosticsStatus, String, Vec<String>) {
    let canon_status =
        std::env::current_exe().ok().map(|current_exe| evaluate_canon_install(&current_exe));
    provider_readiness_context_from_status(workspace, canon_status)
}

fn provider_readiness_context_from_status(
    workspace: &Path,
    canon_status: Option<CanonInstallStatus>,
) -> (DiagnosticsStatus, String, Vec<String>) {
    let fallback_action = "boundline doctor --install".to_string();
    let Some(canon_status) = canon_status else {
        return (
            DiagnosticsStatus::Advisory,
            "provider readiness is unknown because the current Boundline executable could not be resolved"
                .to_string(),
            vec![fallback_action],
        );
    };

    if matches!(canon_status.state, CompanionState::Ready | CompanionState::AlreadySatisfied) {
        return (
            DiagnosticsStatus::Passed,
            format!(
                "provider readiness is confirmed for workspace diagnostics from {}",
                workspace.display()
            ),
            Vec::new(),
        );
    }

    let mut actions = canon_status.suggested_actions;
    if !actions.iter().any(|action| action == &fallback_action) {
        actions.push(fallback_action);
    }
    (
        DiagnosticsStatus::Advisory,
        format!("provider readiness is not confirmed for this machine: {}", canon_status.message),
        actions,
    )
}

fn push_context_check(
    report: &mut DiagnosticsReport,
    check: DiagnosticsCheck,
    suggested_action: Option<String>,
) {
    report.checks.push(check);
    if let Some(action) = suggested_action {
        push_suggested_action(report, action);
    }
}

fn push_suggested_action(report: &mut DiagnosticsReport, action: String) {
    if !report.suggested_actions.iter().any(|existing| existing == &action) {
        report.suggested_actions.push(action);
    }
}

fn distribution_channel_message(channel_candidates: &[String]) -> String {
    match channel_candidates {
        [] => "no supported install channels are available on this machine".to_string(),
        [single] if single == "source" => {
            "official bundled channels are unavailable on this machine; source fallback remains supported"
                .to_string()
        }
        _ => format!(
            "supported install paths on this machine: {}",
            channel_candidates.join(", ")
        ),
    }
}

fn finalize_report(
    checks: Vec<DiagnosticsCheck>,
    additional_actions: Vec<String>,
    context: DiagnosticsReportContext,
) -> DiagnosticsReport {
    let missing_prerequisites = checks
        .iter()
        .filter(|check| check.status == DiagnosticsStatus::Failed)
        .map(|check| check.name.clone())
        .collect::<Vec<_>>();
    let mut suggested_actions = checks
        .iter()
        .filter(|check| check.status == DiagnosticsStatus::Failed)
        .map(|check| check.message.clone())
        .collect::<Vec<_>>();
    for action in additional_actions {
        if !suggested_actions.iter().any(|existing| existing == &action) {
            suggested_actions.push(action);
        }
    }

    DiagnosticsReport {
        subject: context.subject,
        workspace_ref: context.workspace_ref,
        installation_ref: context.installation_ref,
        ready: missing_prerequisites.is_empty(),
        checks,
        missing_prerequisites,
        suggested_actions,
        boundline_version: context.boundline_version,
        supported_canon_version: context.supported_canon_version,
        companion_state: context.companion_state,
        channel_candidates: context.channel_candidates,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use super::{
        ADVANCED_CONTEXT_INDEX_RELATIVE, CANON_GUIDANCE_DIR_RELATIVE, DiagnosticsCheck,
        DiagnosticsReportContext, DiagnosticsStatus, DiagnosticsSubject, SESSION_RECORD_RELATIVE,
        WORKSPACE_CONFIG_RELATIVE, WORKSPACE_GUIDANCE_DIR_RELATIVE, diagnose_installation,
        diagnose_installation_from_current_exe, diagnose_native_direct_run_workspace,
        diagnose_workspace, diagnose_workspace_context, distribution_channel_message,
        finalize_report, provider_readiness_context_from_status,
    };
    use crate::domain::distribution::{CanonInstallStatus, CompanionState};

    fn temp_workspace() -> PathBuf {
        let workspace =
            std::env::temp_dir().join(format!("boundline-diagnostics-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"boundline\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        fs::write(
            workspace.join(".boundline").join("execution.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "name": "diagnostics-execution",
                "read_targets": ["src/lib.rs"],
                "validation_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "attempts": [
                    {
                        "attempt_id": "fix-add",
                        "summary": "Replace subtraction with addition",
                        "failure_mode": "terminal",
                        "changes": [
                            {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn temp_stack_neutral_workspace() -> PathBuf {
        let workspace =
            std::env::temp_dir().join(format!("boundline-neutral-diagnostics-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(
            workspace.join(".boundline").join("execution.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "name": "diagnostics-execution",
                "read_targets": ["README.md"],
                "validation_command": {"program": "echo", "args": ["ok"]},
                "attempts": [
                    {
                        "attempt_id": "workspace-bootstrap",
                        "summary": "Prepare an empty workspace for planning",
                        "failure_mode": "terminal",
                        "changes": [
                            {"path": "README.md", "find": "before", "replace": "after"}
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn write_project_memory_surface(workspace: &Path) {
        let project_dir = workspace.join("docs/project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("architecture-map.md"), "# Architecture Map\n\nContent here.")
            .unwrap();
        fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": "run-123",
                "mode": "architecture",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "architecture-map",
                "destination": "docs/project/architecture-map.md",
                "source_artifacts": ["architecture-overview.md"],
                "profile": "project-memory",
                "promotion_state": "auto",
                "update_strategy": "managed-blocks",
                "lineage": {
                    "contract_version": "v1",
                    "producer": "canon",
                    "source_ref": "canon-run:run-123",
                    "source_artifacts": ["architecture-overview.md"],
                    "mode": "architecture",
                    "promotion_state": "auto-if-approved",
                    "approval_state": "Completed",
                    "stage": "architecture",
                    "owner": "Owner <owner@example.com>",
                    "risk": "bounded-impact",
                    "zone": "yellow",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:abc123",
                    "packet_readiness": "complete",
                    "promotion_profile": "project-memory"
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn context_check<'a>(report: &'a super::DiagnosticsReport, name: &str) -> &'a DiagnosticsCheck {
        report.checks.iter().find(|check| check.name == name).unwrap()
    }

    #[test]
    fn diagnostics_report_marks_a_writable_workspace_as_ready() {
        let workspace = temp_workspace();
        let report = diagnose_workspace(&workspace);

        assert!(report.ready);
        assert_eq!(report.subject, DiagnosticsSubject::Workspace);
        assert!(report.missing_prerequisites.is_empty());
        assert!(report.checks.iter().all(|check| check.status == DiagnosticsStatus::Passed));
    }

    #[test]
    fn diagnostics_report_accepts_stack_neutral_workspace_with_profile() {
        let workspace = temp_stack_neutral_workspace();
        let report = diagnose_workspace(&workspace);

        assert!(report.ready, "{report:#?}");
        assert!(!report.missing_prerequisites.contains(&"repository_root".to_string()));
    }

    #[test]
    fn diagnostics_report_flags_a_missing_workspace() {
        let workspace = std::env::temp_dir().join(format!("boundline-missing-{}", Uuid::new_v4()));
        let report = diagnose_workspace(&workspace);

        assert!(!report.ready);
        assert!(report.missing_prerequisites.contains(&"workspace_exists".to_string()));
    }

    #[test]
    fn diagnostics_report_flags_an_invalid_trace_root() {
        let workspace = temp_workspace();
        let trace_root = workspace.join(".boundline").join("traces");
        fs::create_dir_all(trace_root.parent().unwrap()).unwrap();
        fs::write(&trace_root, "not-a-directory").unwrap();

        let report = diagnose_workspace(&workspace);

        assert!(!report.ready);
        assert!(report.missing_prerequisites.contains(&"trace_store".to_string()));
    }

    #[test]
    fn installation_diagnostics_report_exposes_install_metadata() {
        let report = diagnose_installation();

        assert_eq!(report.subject, DiagnosticsSubject::Install);
        assert!(report.boundline_version.is_some());
        assert!(report.supported_canon_version.is_some());
        assert!(!report.channel_candidates.is_empty());
        assert!(report.companion_state.is_some());
    }

    #[test]
    fn native_direct_run_diagnostics_ignore_missing_and_invalid_profiles() {
        let missing_profile_workspace = temp_workspace();
        fs::remove_file(missing_profile_workspace.join(".boundline").join("execution.json")).ok();

        let missing_profile_report =
            diagnose_native_direct_run_workspace(&missing_profile_workspace);
        let missing_profile_check = missing_profile_report
            .checks
            .iter()
            .find(|check| check.name == "workspace_execution_profile")
            .unwrap();
        assert_eq!(missing_profile_check.status, DiagnosticsStatus::Passed);
        assert!(missing_profile_check.message.contains("optional for native direct run"));

        let invalid_profile_workspace = temp_workspace();
        fs::write(invalid_profile_workspace.join(".boundline").join("execution.json"), "{not-json")
            .unwrap();

        let invalid_profile_report =
            diagnose_native_direct_run_workspace(&invalid_profile_workspace);
        let invalid_profile_check = invalid_profile_report
            .checks
            .iter()
            .find(|check| check.name == "workspace_execution_profile")
            .unwrap();
        assert_eq!(invalid_profile_check.status, DiagnosticsStatus::Passed);
        assert!(invalid_profile_check.message.contains("ignored"));
        assert!(invalid_profile_report.ready);
    }

    #[test]
    fn diagnostics_report_flags_missing_profile_when_required() {
        let workspace = temp_workspace();
        fs::remove_file(workspace.join(".boundline").join("execution.json")).ok();

        let report = diagnose_workspace(&workspace);
        let profile_check =
            report.checks.iter().find(|check| check.name == "workspace_execution_profile").unwrap();

        assert_eq!(profile_check.status, DiagnosticsStatus::Failed);
        assert!(profile_check.message.contains("boundline init --workspace"));
        assert!(report.missing_prerequisites.contains(&"workspace_execution_profile".to_string()));
    }

    #[test]
    fn diagnostics_report_flags_invalid_profile_when_required() {
        let workspace = temp_workspace();
        fs::write(workspace.join(".boundline").join("execution.json"), "{not-json").unwrap();

        let report = diagnose_workspace(&workspace);
        let profile_check =
            report.checks.iter().find(|check| check.name == "workspace_execution_profile").unwrap();

        assert_eq!(profile_check.status, DiagnosticsStatus::Failed);
        assert!(profile_check.message.contains("workspace execution profile is unavailable"));
        assert!(report.missing_prerequisites.contains(&"workspace_execution_profile".to_string()));
    }

    #[test]
    fn workspace_context_diagnostics_surface_advisory_follow_ups_for_missing_context_inputs() {
        let workspace = temp_workspace();

        let report = diagnose_workspace_context(&workspace);

        assert!(report.ready, "context advisories should not fail workspace doctor: {report:#?}");
        assert_eq!(context_check(&report, "boundline_config").status, DiagnosticsStatus::Advisory);
        assert_eq!(
            context_check(&report, "canon_project_memory").status,
            DiagnosticsStatus::Advisory
        );
        assert_eq!(
            context_check(&report, "expert_pack_inputs").status,
            DiagnosticsStatus::Advisory
        );
        assert_eq!(
            context_check(&report, "advanced_context_index").status,
            DiagnosticsStatus::Advisory
        );
        assert_eq!(context_check(&report, "session_evidence").status, DiagnosticsStatus::Advisory);
        assert!(report.suggested_actions.iter().any(|action| {
            action.contains("boundline config show --workspace")
                && action.contains(workspace.to_string_lossy().as_ref())
        }));
        assert!(
            report
                .suggested_actions
                .iter()
                .any(|action| action.contains("boundline govern --workspace"))
        );
        assert!(
            report
                .suggested_actions
                .iter()
                .any(|action| action.contains("boundline start --workspace"))
        );
    }

    #[test]
    fn workspace_context_diagnostics_skip_context_checks_for_missing_workspace() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-missing-context-{}", Uuid::new_v4()));

        let report = diagnose_workspace_context(&workspace);

        assert!(!report.ready);
        assert!(report.missing_prerequisites.contains(&"workspace_exists".to_string()));
        assert!(report.checks.iter().all(|check| check.name != "boundline_config"));
    }

    #[test]
    fn workspace_context_diagnostics_mark_local_context_inputs_as_ready_when_present() {
        let workspace = temp_workspace();
        fs::write(workspace.join(WORKSPACE_CONFIG_RELATIVE), "[routing]\nmode = \"balanced\"\n")
            .unwrap();
        write_project_memory_surface(&workspace);
        let guidance_dir = workspace.join(WORKSPACE_GUIDANCE_DIR_RELATIVE);
        fs::create_dir_all(&guidance_dir).unwrap();
        fs::write(guidance_dir.join("domain.md"), "# Domain guidance\n").unwrap();
        let canon_guidance_dir = workspace.join(CANON_GUIDANCE_DIR_RELATIVE);
        fs::create_dir_all(&canon_guidance_dir).unwrap();
        fs::write(canon_guidance_dir.join("governance.md"), "# Canon guidance\n").unwrap();
        let advanced_context_index = workspace.join(ADVANCED_CONTEXT_INDEX_RELATIVE);
        fs::create_dir_all(advanced_context_index.parent().unwrap()).unwrap();
        fs::write(&advanced_context_index, b"sqlite-index").unwrap();
        fs::write(workspace.join(SESSION_RECORD_RELATIVE), "{}\n").unwrap();

        let report = diagnose_workspace_context(&workspace);

        assert_eq!(context_check(&report, "boundline_config").status, DiagnosticsStatus::Passed);
        assert_eq!(
            context_check(&report, "canon_project_memory").status,
            DiagnosticsStatus::Passed
        );
        assert_eq!(context_check(&report, "expert_pack_inputs").status, DiagnosticsStatus::Passed);
        assert_eq!(
            context_check(&report, "advanced_context_index").status,
            DiagnosticsStatus::Passed
        );
        assert_eq!(context_check(&report, "session_evidence").status, DiagnosticsStatus::Passed);
    }

    #[test]
    fn installation_diagnostics_handle_missing_current_executable() {
        let report = diagnose_installation_from_current_exe(Err(std::io::Error::other(
            "no executable available",
        )));

        assert_eq!(report.subject, DiagnosticsSubject::Install);
        assert!(!report.ready);
        assert!(report.installation_ref.is_none());
        assert_eq!(
            report.companion_state,
            Some(crate::domain::distribution::CompanionState::Blocked)
        );
        assert!(report.missing_prerequisites.contains(&"boundline_binary".to_string()));
        assert!(report.suggested_actions.iter().any(|action| action.contains("doctor --install")));
    }

    #[test]
    fn diagnostics_helpers_cover_channel_messages_and_deduplicated_actions() {
        assert_eq!(
            distribution_channel_message(&[]),
            "no supported install channels are available on this machine"
        );
        assert!(distribution_channel_message(&["source".to_string()]).contains("source fallback"));
        assert!(
            distribution_channel_message(&["homebrew".to_string(), "source".to_string()])
                .contains("homebrew, source")
        );

        let report = finalize_report(
            vec![DiagnosticsCheck {
                name: "workspace_exists".to_string(),
                status: DiagnosticsStatus::Failed,
                message: "create the workspace".to_string(),
            }],
            vec!["create the workspace".to_string(), "rerun doctor".to_string()],
            DiagnosticsReportContext {
                subject: DiagnosticsSubject::Workspace,
                workspace_ref: Some("/tmp/workspace".to_string()),
                installation_ref: None,
                boundline_version: None,
                supported_canon_version: None,
                companion_state: None,
                channel_candidates: Vec::new(),
            },
        );

        assert!(!report.ready);
        assert_eq!(report.missing_prerequisites, vec!["workspace_exists".to_string()]);
        assert_eq!(
            report.suggested_actions,
            vec!["create the workspace".to_string(), "rerun doctor".to_string()]
        );
    }

    #[test]
    fn provider_readiness_context_reports_ready_status_without_actions() {
        let (status, message, actions) = provider_readiness_context_from_status(
            Path::new("/tmp/workspace"),
            Some(CanonInstallStatus {
                state: CompanionState::Ready,
                version: Some("0.53.0".to_string()),
                location: None,
                bundled_with_boundline: true,
                message: "Canon is ready".to_string(),
                suggested_actions: Vec::new(),
                surface_verification: None,
            }),
        );

        assert_eq!(status, DiagnosticsStatus::Passed);
        assert!(message.contains("provider readiness is confirmed"));
        assert!(actions.is_empty());
    }

    #[test]
    fn provider_readiness_context_reports_advisory_with_fallback_actions() {
        let (status, message, actions) = provider_readiness_context_from_status(
            Path::new("/tmp/workspace"),
            Some(CanonInstallStatus {
                state: CompanionState::RepairNeeded,
                version: Some("0.52.0".to_string()),
                location: None,
                bundled_with_boundline: false,
                message: "Canon needs repair".to_string(),
                suggested_actions: vec!["brew upgrade canon".to_string()],
                surface_verification: None,
            }),
        );

        assert_eq!(status, DiagnosticsStatus::Advisory);
        assert!(message.contains("Canon needs repair"));
        assert!(actions.iter().any(|action| action == "brew upgrade canon"));
        assert!(actions.iter().any(|action| action == "boundline doctor --install"));
    }

    #[test]
    fn provider_readiness_context_reports_unknown_when_install_state_is_unavailable() {
        let (status, message, actions) =
            provider_readiness_context_from_status(Path::new("/tmp/workspace"), None);

        assert_eq!(status, DiagnosticsStatus::Advisory);
        assert!(message.contains("could not be resolved"));
        assert_eq!(actions, vec!["boundline doctor --install".to_string()]);
    }
}
