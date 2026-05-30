//! Workspace probe: lightweight preflight check for assistant hosts.

use std::path::Path;

use crate::adapters::config_store::FileConfigStore;
use crate::adapters::env_layer::provider_environment_status;
use crate::adapters::session_store::{FileSessionStore, SessionStore};
use crate::domain::configuration::SemanticIndexHookAction;
use crate::domain::distribution::{CompanionState, evaluate_canon_install};
use crate::domain::probe::{
    CanonState, CapabilitiesState, ProbeReport, ProviderState, RecommendedHandoff, RecommendedNext,
    SessionState, WorkspaceState,
};
use crate::domain::session::SessionStatus;
use crate::orchestrator::context_intelligence::build_index_doctor_report;

/// Relative path to the advanced context-intelligence retrieval index.
const ADVANCED_CONTEXT_INDEX_RELATIVE: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3";

/// Relative path to the Canon-managed guidance directory.
const CANON_GUIDANCE_DIR_RELATIVE: &str = ".canon/boundline/guidance";

/// Relative path to the workspace execution profile.
const EXECUTION_PROFILE_RELATIVE: &str = ".boundline/execution.json";

/// Relative path to the cluster configuration.
const CLUSTER_CONFIG_RELATIVE: &str = ".boundline/cluster.toml";

/// Relative path to the workspace configuration.
const WORKSPACE_CONFIG_RELATIVE: &str = ".boundline/config.toml";

/// Relative path to the Boundline state directory.
const BOUNDLINE_STATE_DIR: &str = ".boundline";

/// Execute the workspace probe, returning a structured report of workspace
/// readiness for assistant host consumption.
pub fn execute_probe(workspace: &Path) -> ProbeReport {
    let workspace_state = probe_workspace(workspace);
    let session_state = probe_session(workspace);
    let provider_state = probe_providers(workspace);
    let canon_state = probe_canon(workspace);
    let capabilities_state = probe_capabilities(workspace, &canon_state);
    let recommended_next =
        compute_recommended_next(&workspace_state, &session_state, &provider_state);
    let recommended_handoffs =
        compute_recommended_handoffs(&workspace_state, &session_state, &provider_state);

    ProbeReport {
        workspace: workspace_state,
        session: session_state,
        providers: provider_state,
        canon: canon_state,
        capabilities: capabilities_state,
        recommended_next,
        recommended_handoffs,
    }
}

fn probe_workspace(workspace: &Path) -> WorkspaceState {
    let initialized = workspace.join(BOUNDLINE_STATE_DIR).is_dir();
    let config_present = workspace.join(WORKSPACE_CONFIG_RELATIVE).is_file();
    let execution_profile_present = workspace.join(EXECUTION_PROFILE_RELATIVE).is_file();

    WorkspaceState {
        path: workspace.display().to_string(),
        initialized,
        config_present,
        execution_profile_present,
    }
}

fn probe_session(workspace: &Path) -> SessionState {
    let store = FileSessionStore::for_workspace(workspace);
    match store.load() {
        Ok(Some(record)) => {
            let status_str = session_status_label(record.latest_status).to_string();
            let goal_summary = record.goal.clone();
            let waiting_for_phase_request = matches!(record.latest_status, SessionStatus::Blocked);

            SessionState {
                active: true,
                session_id: Some(record.session_id.clone()),
                status: Some(status_str),
                goal_summary,
                waiting_for_phase_request,
            }
        }
        Ok(None) | Err(_) => SessionState {
            active: false,
            session_id: None,
            status: None,
            goal_summary: None,
            waiting_for_phase_request: false,
        },
    }
}

/// Maps `SessionStatus` to its snake_case label for probe output.
fn session_status_label(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn probe_providers(workspace: &Path) -> ProviderState {
    let env_status = provider_environment_status(Some(workspace));

    let has_env_credentials = env_status.workspace_env_present
        || env_status.workspace_env_local_present
        || env_status.global_env_present
        || !env_status.process_keys_present.is_empty();

    let has_env_template =
        env_status.workspace_env_template_present || env_status.global_env_template_present;

    let configured = has_env_credentials || has_env_template;
    let healthy = has_env_credentials;

    let recommended_action = if !configured {
        Some("init".to_string())
    } else if !healthy {
        Some("doctor".to_string())
    } else {
        None
    };

    // Determine active runtime from config if available.
    let active_runtime = resolve_active_runtime(workspace);

    ProviderState { configured, healthy, active_runtime, recommended_action }
}

fn resolve_active_runtime(workspace: &Path) -> Option<String> {
    let store = FileConfigStore::for_workspace(workspace);
    let config = store.load_local().ok().flatten()?;
    let planning_route = config.routing.planning.as_ref()?;
    Some(planning_route.runtime.as_str().to_string())
}

fn probe_canon(workspace: &Path) -> CanonState {
    let binary_available = match std::env::current_exe() {
        Ok(exe) => {
            let status = evaluate_canon_install(&exe);
            matches!(status.state, CompanionState::Ready | CompanionState::AlreadySatisfied)
        }
        Err(_) => false,
    };

    let project_memory_present = path_has_entries(&workspace.join(CANON_GUIDANCE_DIR_RELATIVE));
    let guidance_present = project_memory_present;

    CanonState { binary_available, project_memory_present, guidance_present }
}

fn probe_capabilities(workspace: &Path, canon: &CanonState) -> CapabilitiesState {
    let semantic_index = workspace.join(ADVANCED_CONTEXT_INDEX_RELATIVE).is_file();
    let semantic_index_health =
        build_index_doctor_report(workspace).ok().map(|report| report.status.as_str().to_string());
    let semantic_index_hooks = resolve_semantic_index_hook_probe_state(workspace);
    let cluster = workspace.join(CLUSTER_CONFIG_RELATIVE).is_file();

    CapabilitiesState {
        phase_request: true,
        json_stream: true,
        guidance_catalog: true,
        guardians: true,
        canon_governance: canon.binary_available,
        semantic_index,
        semantic_index_health,
        semantic_index_hooks,
        cluster,
    }
}

fn resolve_semantic_index_hook_probe_state(workspace: &Path) -> Option<String> {
    let config = FileConfigStore::for_workspace(workspace).load_local().ok().flatten()?;
    let hook_action = config
        .routing
        .semantic_acceleration
        .as_ref()
        .map(|policy| policy.index_hook_action)
        .unwrap_or(SemanticIndexHookAction::Disabled);
    let hook_paths = [
        workspace.join(".git/hooks/post-checkout"),
        workspace.join(".git/hooks/post-merge"),
        workspace.join(".git/hooks/post-rewrite"),
    ];
    let installed = hook_paths.iter().all(|path| path.is_file());
    Some(match hook_action {
        SemanticIndexHookAction::Disabled => "disabled".to_string(),
        SemanticIndexHookAction::MarkStale if installed => "mark_stale_installed".to_string(),
        SemanticIndexHookAction::MarkStale => "mark_stale_missing".to_string(),
    })
}

fn compute_recommended_next(
    workspace: &WorkspaceState,
    session: &SessionState,
    providers: &ProviderState,
) -> Option<RecommendedNext> {
    // Priority 1: workspace not initialized.
    if !workspace.initialized {
        return Some(RecommendedNext {
            command: "boundline init".to_string(),
            assistant_command: None,
            reason: "workspace is not initialized".to_string(),
        });
    }

    // Priority 2: providers not healthy.
    if !providers.healthy {
        return Some(RecommendedNext {
            command: "boundline doctor".to_string(),
            assistant_command: Some("/boundline-doctor".to_string()),
            reason: "provider credentials are not available".to_string(),
        });
    }

    // Priority 3: no active session.
    if !session.active {
        return Some(RecommendedNext {
            command: "boundline goal".to_string(),
            assistant_command: Some("/boundline-goal".to_string()),
            reason: "no active session".to_string(),
        });
    }

    // Priority 4: session status determines next action.
    let status = session.status.as_deref().unwrap_or_default();
    match status {
        "initialized" => Some(RecommendedNext {
            command: "boundline goal".to_string(),
            assistant_command: Some("/boundline-goal".to_string()),
            reason: "session needs a goal".to_string(),
        }),
        "goal_captured" => Some(RecommendedNext {
            command: "boundline plan".to_string(),
            assistant_command: Some("/boundline-plan".to_string()),
            reason: "goal captured; ready to plan".to_string(),
        }),
        "planned" => Some(RecommendedNext {
            command: "boundline run".to_string(),
            assistant_command: Some("/boundline-run".to_string()),
            reason: "plan ready; ready to execute".to_string(),
        }),
        "running" => Some(RecommendedNext {
            command: "boundline step".to_string(),
            assistant_command: Some("/boundline-step".to_string()),
            reason: "execution in progress".to_string(),
        }),
        "blocked" => Some(RecommendedNext {
            command: "boundline status".to_string(),
            assistant_command: Some("/boundline-status".to_string()),
            reason: "session is blocked; review status".to_string(),
        }),
        "succeeded" | "failed" | "exhausted" | "aborted" => Some(RecommendedNext {
            command: "boundline goal".to_string(),
            assistant_command: Some("/boundline-goal".to_string()),
            reason: format!("session is terminal ({status}); start a new goal"),
        }),
        _ => None,
    }
}

fn compute_recommended_handoffs(
    workspace: &WorkspaceState,
    session: &SessionState,
    providers: &ProviderState,
) -> Vec<RecommendedHandoff> {
    let mut handoffs = Vec::new();

    if !workspace.initialized {
        return handoffs;
    }

    if !providers.healthy {
        handoffs.push(RecommendedHandoff {
            label: "Run Doctor".to_string(),
            command: "/boundline-doctor".to_string(),
            reason: "provider credentials need configuration".to_string(),
        });
        return handoffs;
    }

    if !session.active {
        handoffs.push(RecommendedHandoff {
            label: "Set Goal".to_string(),
            command: "/boundline-goal".to_string(),
            reason: "no active session".to_string(),
        });
        return handoffs;
    }

    let status = session.status.as_deref().unwrap_or_default();
    match status {
        "goal_captured" => {
            handoffs.push(RecommendedHandoff {
                label: "Plan Workflow".to_string(),
                command: "/boundline-plan".to_string(),
                reason: "goal captured; ready to plan".to_string(),
            });
            handoffs.push(RecommendedHandoff {
                label: "Check Status".to_string(),
                command: "/boundline-status".to_string(),
                reason: "review current session state".to_string(),
            });
        }
        "planned" => {
            handoffs.push(RecommendedHandoff {
                label: "Execute Plan".to_string(),
                command: "/boundline-run".to_string(),
                reason: "plan ready to execute".to_string(),
            });
            handoffs.push(RecommendedHandoff {
                label: "Review Plan".to_string(),
                command: "/boundline-inspect".to_string(),
                reason: "verify plan details before execution".to_string(),
            });
        }
        "running" => {
            handoffs.push(RecommendedHandoff {
                label: "Continue Step".to_string(),
                command: "/boundline-step".to_string(),
                reason: "execution in progress".to_string(),
            });
            handoffs.push(RecommendedHandoff {
                label: "Check Status".to_string(),
                command: "/boundline-status".to_string(),
                reason: "review execution progress".to_string(),
            });
        }
        "blocked" => {
            handoffs.push(RecommendedHandoff {
                label: "Review Status".to_string(),
                command: "/boundline-status".to_string(),
                reason: "session is blocked".to_string(),
            });
            handoffs.push(RecommendedHandoff {
                label: "Recover".to_string(),
                command: "/boundline-recover".to_string(),
                reason: "attempt recovery from blocked state".to_string(),
            });
        }
        "succeeded" | "failed" | "exhausted" | "aborted" => {
            handoffs.push(RecommendedHandoff {
                label: "Start New Goal".to_string(),
                command: "/boundline-goal".to_string(),
                reason: format!("session is terminal ({status})"),
            });
            handoffs.push(RecommendedHandoff {
                label: "Inspect Results".to_string(),
                command: "/boundline-inspect".to_string(),
                reason: "review session outcome".to_string(),
            });
        }
        _ => {
            handoffs.push(RecommendedHandoff {
                label: "Check Status".to_string(),
                command: "/boundline-status".to_string(),
                reason: "review current state".to_string(),
            });
        }
    }

    handoffs
}

/// Returns `true` if the path exists and contains at least one entry.
fn path_has_entries(path: &Path) -> bool {
    path.is_dir()
        && std::fs::read_dir(path).ok().is_some_and(|mut entries| entries.next().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const SEMANTIC_HOOK_CONFIG_DISABLED: &str =
        "version = 1\n\n[routing.semantic_acceleration]\npolicy = \"local\"\n";
    const SEMANTIC_HOOK_CONFIG_MARK_STALE: &str = concat!(
        "version = 1\n\n",
        "[routing.semantic_acceleration]\n",
        "policy = \"local\"\n",
        "index_hook_action = \"mark_stale\"\n",
    );

    #[test]
    fn probe_uninitialized_workspace_recommends_init() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        let report = execute_probe(tmp.path());

        assert!(!report.workspace.initialized);
        assert!(!report.workspace.config_present);
        assert!(!report.session.active);
        let next = report.recommended_next.as_ref().unwrap();
        assert_eq!(next.command, "boundline init");
        assert_eq!(next.assistant_command, None);
        assert_eq!(next.reason, "workspace is not initialized");
        assert!(report.recommended_handoffs.is_empty());
    }

    #[test]
    fn probe_initialized_no_session_recommends_goal() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        fs::create_dir_all(tmp.path().join(".boundline")).map_err(|e| e.to_string()).unwrap();
        // Provide a workspace .env so provider health check passes.
        fs::write(tmp.path().join(".env"), b"PROVIDER_KEY=test\n")
            .map_err(|e| e.to_string())
            .unwrap();

        let report = execute_probe(tmp.path());

        assert!(report.workspace.initialized);
        assert!(!report.session.active);
        let next = report.recommended_next.as_ref().unwrap();
        assert_eq!(next.assistant_command.as_deref(), Some("/boundline-goal"));
    }

    #[test]
    fn probe_reports_config_and_execution_profile() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        let bl = tmp.path().join(".boundline");
        fs::create_dir_all(&bl).map_err(|e| e.to_string()).unwrap();
        fs::write(bl.join("config.toml"), b"[routing]\n").map_err(|e| e.to_string()).unwrap();
        fs::write(bl.join("execution.json"), b"{}").map_err(|e| e.to_string()).unwrap();

        let report = execute_probe(tmp.path());

        assert!(report.workspace.config_present);
        assert!(report.workspace.execution_profile_present);
    }

    #[test]
    fn probe_detects_semantic_index() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        let index_dir = tmp.path().join(".boundline/context-intelligence");
        fs::create_dir_all(&index_dir).map_err(|e| e.to_string()).unwrap();
        fs::write(index_dir.join("retrieval-index.sqlite3"), b"fake-db")
            .map_err(|e| e.to_string())
            .unwrap();

        let report = execute_probe(tmp.path());

        assert!(report.capabilities.semantic_index);
        assert_eq!(report.capabilities.semantic_index_health.as_deref(), Some("failed"));
    }

    #[test]
    fn probe_detects_cluster() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        let bl = tmp.path().join(".boundline");
        fs::create_dir_all(&bl).map_err(|e| e.to_string()).unwrap();
        fs::write(bl.join("cluster.toml"), b"[cluster]\n").map_err(|e| e.to_string()).unwrap();

        let report = execute_probe(tmp.path());

        assert!(report.capabilities.cluster);
    }

    #[test]
    fn probe_canon_guidance_absent() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();

        let report = execute_probe(tmp.path());

        assert!(!report.canon.project_memory_present);
        assert!(!report.canon.guidance_present);
    }

    #[test]
    fn probe_canon_guidance_present() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        let guidance = tmp.path().join(".canon/boundline/guidance");
        fs::create_dir_all(&guidance).map_err(|e| e.to_string()).unwrap();
        fs::write(guidance.join("some-rule.md"), b"# Rule").map_err(|e| e.to_string()).unwrap();

        let report = execute_probe(tmp.path());

        assert!(report.canon.project_memory_present);
        assert!(report.canon.guidance_present);
    }

    #[test]
    fn probe_capabilities_always_include_core_features() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();

        let report = execute_probe(tmp.path());

        assert!(report.capabilities.phase_request);
        assert!(report.capabilities.json_stream);
        assert!(report.capabilities.guidance_catalog);
        assert!(report.capabilities.guardians);
    }

    #[test]
    fn probe_reports_semantic_index_hook_states() {
        let tmp = TempDir::new().map_err(|e| e.to_string()).unwrap();
        let config_root = tmp.path().join(".boundline");
        fs::create_dir_all(&config_root).map_err(|e| e.to_string()).unwrap();

        fs::write(config_root.join("config.toml"), SEMANTIC_HOOK_CONFIG_DISABLED)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(
            resolve_semantic_index_hook_probe_state(tmp.path()).as_deref(),
            Some("disabled")
        );

        fs::write(config_root.join("config.toml"), SEMANTIC_HOOK_CONFIG_MARK_STALE)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(
            resolve_semantic_index_hook_probe_state(tmp.path()).as_deref(),
            Some("mark_stale_missing")
        );

        let hooks_root = tmp.path().join(".git/hooks");
        fs::create_dir_all(&hooks_root).map_err(|e| e.to_string()).unwrap();
        for hook_name in ["post-checkout", "post-merge", "post-rewrite"] {
            fs::write(hooks_root.join(hook_name), "#!/bin/sh\nexit 0\n")
                .map_err(|e| e.to_string())
                .unwrap();
        }

        assert_eq!(
            resolve_semantic_index_hook_probe_state(tmp.path()).as_deref(),
            Some("mark_stale_installed")
        );
    }
}
