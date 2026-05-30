//! Derived-index lifecycle CLI helpers.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use crate::domain::context_intelligence::{
    IndexDoctorReport, IndexLifecycleReport, IndexMaintenanceCommand, RetrievalState,
};
use crate::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use crate::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection, build_index_doctor_report,
    build_index_status_report,
};
use crate::orchestrator::goal_planner::collect_workspace_file_refs;

use super::workspace;

const INDEX_CONTEXT_DIRECTORY: &str = ".boundline/context-intelligence";
const INDEX_DATABASE_FILE_NAME: &str = "retrieval-index.sqlite3";
const INDEX_DATABASE_WAL_FILE_NAME: &str = "retrieval-index.sqlite3-wal";
const INDEX_DATABASE_SHM_FILE_NAME: &str = "retrieval-index.sqlite3-shm";
const INDEX_MANIFEST_FILE_NAME: &str = "manifest.json";
const INDEX_REFRESH_GOAL_TEXT: &str = "refresh the derived index for local workspace evidence";
const INDEX_WORKSPACE_SCAN_SOURCE: &str = "index_workspace_scan";
const INDEX_WORKSPACE_SCAN_RATIONALE: &str =
    "manual index lifecycle refresh across workspace files";

/// Resolves the workspace and returns the current manifest-backed index status report.
pub fn execute_status(workspace_ref: Option<&Path>) -> Result<IndexLifecycleReport, String> {
    let workspace =
        workspace::resolve_workspace(workspace_ref).map_err(|error| error.to_string())?;
    build_index_status_report(&workspace)
}

/// Resolves the workspace, refreshes the derived index, and returns the lifecycle report.
pub fn execute_refresh(workspace_ref: Option<&Path>) -> Result<IndexLifecycleReport, String> {
    run_index_operation(workspace_ref, IndexMaintenanceCommand::Refresh, |workspace| {
        refresh_workspace_index(workspace)
    })
}

/// Resolves the workspace, rebuilds the derived index, and returns the lifecycle report.
pub fn execute_rebuild(workspace_ref: Option<&Path>) -> Result<IndexLifecycleReport, String> {
    run_index_operation(workspace_ref, IndexMaintenanceCommand::Rebuild, |workspace| {
        remove_index_artifacts(workspace)?;
        refresh_workspace_index(workspace)
    })
}

/// Resolves the workspace, removes derived-index artifacts, and returns the lifecycle report.
pub fn execute_clean(workspace_ref: Option<&Path>) -> Result<IndexLifecycleReport, String> {
    run_index_operation(workspace_ref, IndexMaintenanceCommand::Clean, |workspace| {
        remove_index_artifacts(workspace)
    })
}

/// Resolves the workspace and returns the current derived-index doctor report.
pub fn execute_doctor(workspace_ref: Option<&Path>) -> Result<IndexDoctorReport, String> {
    let workspace =
        workspace::resolve_workspace(workspace_ref).map_err(|error| error.to_string())?;
    build_index_doctor_report(&workspace)
}

fn run_index_operation<F>(
    workspace_ref: Option<&Path>,
    command: IndexMaintenanceCommand,
    operation: F,
) -> Result<IndexLifecycleReport, String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    let workspace =
        workspace::resolve_workspace(workspace_ref).map_err(|error| error.to_string())?;
    let pre_state = build_index_status_report(&workspace)?.post_state;
    operation(&workspace)?;
    build_index_operation_report(command, &workspace, pre_state)
}

fn build_index_operation_report(
    command: IndexMaintenanceCommand,
    workspace_ref: &Path,
    pre_state: crate::domain::context_intelligence::RetrievalIndexState,
) -> Result<IndexLifecycleReport, String> {
    let status_report = build_index_status_report(workspace_ref)?;
    let report = IndexLifecycleReport {
        command,
        workspace_root: status_report.workspace_root,
        operation_id: format!("index-{}:{}", command.as_str(), Uuid::new_v4()),
        pre_state,
        post_state: status_report.post_state,
        recommended_action: status_report.recommended_action,
        stale_reason: status_report.stale_reason,
        warnings: status_report.warnings,
        manifest: status_report.manifest,
    };
    report.validate().map_err(|error| error.to_string())?;
    Ok(report)
}

fn refresh_workspace_index(workspace_ref: &Path) -> Result<(), String> {
    let file_refs = collect_workspace_file_refs(workspace_ref);
    if file_refs.is_empty() {
        return remove_index_artifacts(workspace_ref);
    }

    let inputs = workspace_file_inputs(&file_refs);
    let projection = build_advanced_context_projection(
        INDEX_REFRESH_GOAL_TEXT,
        workspace_ref,
        &inputs,
        &[],
        AdvancedContextBuildState {
            credibility: ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    finish_refresh_projection(projection)
}

fn finish_refresh_projection(
    projection: crate::domain::context_intelligence::AdvancedContextProjection,
) -> Result<(), String> {
    if projection.retrieval_state == RetrievalState::Unavailable {
        return Err(projection
            .terminal_reason
            .unwrap_or_else(|| "failed to refresh the derived index".to_string()));
    }

    Ok(())
}

fn workspace_file_inputs(file_refs: &[String]) -> Vec<ContextInput> {
    file_refs
        .iter()
        .enumerate()
        .map(|(index, reference)| ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: reference.clone(),
            source: INDEX_WORKSPACE_SCAN_SOURCE.to_string(),
            rationale: INDEX_WORKSPACE_SCAN_RATIONALE.to_string(),
            primary: index == 0,
        })
        .collect()
}

fn remove_index_artifacts(workspace_ref: &Path) -> Result<(), String> {
    let index_directory = workspace_ref.join(INDEX_CONTEXT_DIRECTORY);
    for artifact in index_artifact_paths(&index_directory) {
        match fs::remove_file(&artifact) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(format!("failed to remove {}: {error}", artifact.display())),
        }
    }

    match fs::remove_dir(&index_directory) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::DirectoryNotEmpty) =>
        {
            Ok(())
        }
        Err(error) => Err(format!("failed to remove {}: {error}", index_directory.display())),
    }
}

fn index_artifact_paths(index_directory: &Path) -> [PathBuf; 4] {
    [
        index_directory.join(INDEX_DATABASE_FILE_NAME),
        index_directory.join(INDEX_DATABASE_WAL_FILE_NAME),
        index_directory.join(INDEX_DATABASE_SHM_FILE_NAME),
        index_directory.join(INDEX_MANIFEST_FILE_NAME),
    ]
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{
        AdvancedContextBuildState, AdvancedContextConfig, ContextInput, ContextInputKind,
        ContextPackCredibility, INDEX_CONTEXT_DIRECTORY, INDEX_REFRESH_GOAL_TEXT,
        INDEX_WORKSPACE_SCAN_RATIONALE, INDEX_WORKSPACE_SCAN_SOURCE,
        SemanticAccelerationPolicyState, build_advanced_context_projection,
        finish_refresh_projection, index_artifact_paths, refresh_workspace_index,
        remove_index_artifacts, workspace_file_inputs,
    };
    use crate::domain::context_intelligence::RetrievalState;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn simple_projection_workspace() -> PathBuf {
        let workspace = temp_workspace("boundline-cli-index");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .unwrap();
        workspace
    }

    #[test]
    fn finish_refresh_projection_returns_terminal_reason_for_unavailable_index_state() {
        let workspace = simple_projection_workspace();
        let mut projection = build_advanced_context_projection(
            INDEX_REFRESH_GOAL_TEXT,
            &workspace,
            &[ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                source: "index_workspace_scan".to_string(),
                rationale: "selected workspace target".to_string(),
                primary: true,
            }],
            &[],
            AdvancedContextBuildState {
                credibility: ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Local,
            },
            &AdvancedContextConfig::default(),
        );
        projection.retrieval_state = RetrievalState::Unavailable;
        projection.terminal_reason = Some("semantic refresh failed".to_string());

        assert_eq!(finish_refresh_projection(projection).unwrap_err(), "semantic refresh failed");
    }

    #[test]
    fn finish_refresh_projection_falls_back_to_default_reason_when_unavailable() {
        let workspace = simple_projection_workspace();
        let mut projection = build_advanced_context_projection(
            INDEX_REFRESH_GOAL_TEXT,
            &workspace,
            &[ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                source: "index_workspace_scan".to_string(),
                rationale: "selected workspace target".to_string(),
                primary: true,
            }],
            &[],
            AdvancedContextBuildState {
                credibility: ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Local,
            },
            &AdvancedContextConfig::default(),
        );
        projection.retrieval_state = RetrievalState::Unavailable;
        projection.terminal_reason = None;

        assert_eq!(
            finish_refresh_projection(projection).unwrap_err(),
            "failed to refresh the derived index"
        );
    }

    #[test]
    fn remove_index_artifacts_treats_non_empty_directory_as_success() {
        let workspace = temp_workspace("boundline-cli-index-clean");
        let index_directory = workspace.join(INDEX_CONTEXT_DIRECTORY);
        let nested_directory = index_directory.join("preserved");
        fs::create_dir_all(&nested_directory).unwrap();
        fs::write(index_directory.join("retrieval-index.sqlite3"), "placeholder index").unwrap();
        fs::write(nested_directory.join("marker.txt"), "keep").unwrap();

        remove_index_artifacts(&workspace).unwrap();

        assert!(nested_directory.join("marker.txt").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn remove_index_artifacts_reports_directory_remove_errors() {
        use std::os::unix::fs::PermissionsExt;

        let workspace = temp_workspace("boundline-cli-index-clean-error");
        let state_directory = workspace.join(".boundline");
        let index_directory = workspace.join(INDEX_CONTEXT_DIRECTORY);
        fs::create_dir_all(&index_directory).unwrap();
        fs::set_permissions(&state_directory, fs::Permissions::from_mode(0o555)).unwrap();

        let error = remove_index_artifacts(&workspace).unwrap_err();

        fs::set_permissions(&state_directory, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(error.contains(&index_directory.display().to_string()));
    }

    #[test]
    fn refresh_workspace_index_removes_artifacts_when_workspace_has_no_files() {
        let workspace = temp_workspace("boundline-cli-index-empty-refresh");
        let index_directory = workspace.join(INDEX_CONTEXT_DIRECTORY);
        fs::create_dir_all(&index_directory).unwrap();
        for artifact in index_artifact_paths(&index_directory) {
            fs::write(artifact, "placeholder").unwrap();
        }

        refresh_workspace_index(&workspace).unwrap();

        assert!(!index_directory.join("retrieval-index.sqlite3").exists());
    }

    #[test]
    fn workspace_file_inputs_marks_the_first_ref_as_primary() {
        let inputs = workspace_file_inputs(&["src/lib.rs".to_string(), "README.md".to_string()]);

        assert_eq!(inputs.len(), 2);
        assert!(inputs[0].primary);
        assert!(!inputs[1].primary);
        assert_eq!(inputs[0].source, INDEX_WORKSPACE_SCAN_SOURCE);
        assert_eq!(inputs[1].rationale, INDEX_WORKSPACE_SCAN_RATIONALE);
    }
}
