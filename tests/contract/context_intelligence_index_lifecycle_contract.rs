use std::error::Error;
use std::fs;

use boundline::domain::context_intelligence::{
    DerivedIndexManifest, IndexLifecycleReport, IndexMaintenanceCommand, IndexRefreshReason,
    IndexStaleReason, ManifestFtsState, RetrievalIndexState, SemanticEngine, VectorExtensionState,
};

#[cfg(feature = "sqlite-vec")]
use crate::workspace_fixture::run_boundline_in_with_env;
#[cfg(feature = "sqlite-vec")]
use crate::workspace_fixture::temp_git_workspace;
use crate::workspace_fixture::{
    run_boundline_in, stdout_json, temp_empty_workspace, terminal_text,
};

type TestResult = Result<(), Box<dyn Error>>;

const STATUS_COMMAND: &str = "boundline index status";
const REFRESH_COMMAND: &str = "boundline index refresh";
const REBUILD_COMMAND: &str = "boundline index rebuild";
const CLEAN_COMMAND: &str = "boundline index clean";
const REQUIRED_TOP_LEVEL_FIELDS: [&str; 7] = [
    "command",
    "workspace_root",
    "operation_id",
    "pre_state",
    "post_state",
    "recommended_action",
    "warnings",
];
#[cfg(feature = "sqlite-vec")]
const SEMANTIC_VECTOR_STATE_OVERRIDE_ENV: &str = "BOUNDLINE_SEMANTIC_VECTOR_STATE_OVERRIDE";
#[cfg(feature = "sqlite-vec")]
const INDEX_CONTRACT_SOURCE_REF: &str = "src/lib.rs";

#[test]
fn index_lifecycle_contract_serializes_expected_command_family() -> TestResult {
    let commands = [STATUS_COMMAND, REFRESH_COMMAND, REBUILD_COMMAND, CLEAN_COMMAND];
    if !commands.iter().all(|command| !command.trim().is_empty()) {
        return Err("expected every lifecycle command entry to be non-empty".into());
    }
    if !REQUIRED_TOP_LEVEL_FIELDS.iter().all(|field| !field.trim().is_empty()) {
        return Err("expected every lifecycle contract field to be non-empty".into());
    }

    let reports = [
        lifecycle_report_fixture(IndexMaintenanceCommand::Status),
        lifecycle_report_fixture(IndexMaintenanceCommand::Refresh),
        lifecycle_report_fixture(IndexMaintenanceCommand::Rebuild),
        lifecycle_report_fixture(IndexMaintenanceCommand::Clean),
    ];

    for report in reports {
        let serialized = serde_json::to_value(&report)?;
        let object = serialized
            .as_object()
            .ok_or("expected serialized lifecycle report to be a JSON object")?;

        for field in REQUIRED_TOP_LEVEL_FIELDS {
            if !object.contains_key(field) {
                return Err(format!("missing `{field}` in {serialized}").into());
            }
        }
        if object.get("workspace_root").and_then(|value| value.as_str()) != Some("workspace") {
            return Err(format!("unexpected workspace_root in {serialized}").into());
        }
        if object.get("command").and_then(|value| value.as_str()) != Some(report.command.as_str()) {
            return Err(format!("unexpected command in {serialized}").into());
        }
        if object.get("pre_state").and_then(|value| value.as_str())
            != Some(report.pre_state.as_str())
        {
            return Err(format!("unexpected pre_state in {serialized}").into());
        }
        if object.get("post_state").and_then(|value| value.as_str())
            != Some(report.post_state.as_str())
        {
            return Err(format!("unexpected post_state in {serialized}").into());
        }
        if object.get("operation_id").and_then(|value| value.as_str()).is_none() {
            return Err(format!("missing operation_id string in {serialized}").into());
        }
        if object.get("recommended_action").and_then(|value| value.as_str()).is_none() {
            return Err(format!("missing recommended_action string in {serialized}").into());
        }
        if object.get("warnings").and_then(|value| value.as_array()).is_none() {
            return Err(format!("missing warnings array in {serialized}").into());
        }
        if report.post_state == RetrievalIndexState::Stale
            && object.get("stale_reason").and_then(|value| value.as_str())
                != Some(IndexStaleReason::GitHeadChanged.as_str())
        {
            return Err(format!("missing stale_reason in {serialized}").into());
        }
    }
    Ok(())
}

#[test]
fn index_status_contract_preserves_manifest_details() -> TestResult {
    let report = lifecycle_report_fixture(IndexMaintenanceCommand::Status);
    let serialized = serde_json::to_value(&report)?;

    if serialized["command"] != "status" {
        return Err(format!("unexpected command payload: {serialized}").into());
    }
    if serialized["manifest"]["schema_version"] != "retrieval-index-v3" {
        return Err(format!("missing schema_version in {serialized}").into());
    }
    if serialized["manifest"]["index_status"] != "ready" {
        return Err(format!("missing index_status in {serialized}").into());
    }
    if serialized["manifest"]["last_refresh_reason"] != "manual_refresh" {
        return Err(format!("missing last_refresh_reason in {serialized}").into());
    }
    if serialized["manifest"]["semantic_engine"] != "sqlite_vec" {
        return Err(format!("missing semantic_engine in {serialized}").into());
    }
    if serialized["stale_reason"] != "git_head_changed" {
        return Err(format!("missing stale_reason in {serialized}").into());
    }
    if serialized["warnings"][0]
        != "git HEAD changed since the last successful derived-index refresh"
    {
        return Err(format!("missing stale-head warning in {serialized}").into());
    }
    Ok(())
}

#[test]
fn index_status_command_reports_missing_index_for_empty_workspace() -> TestResult {
    let workspace = temp_empty_workspace("boundline-index-status-contract-missing");
    let output = run_boundline_in(
        &workspace,
        &["index", "status", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&output);

    if output.status.code() != Some(0) {
        return Err(format!("expected success, got output: {text}").into());
    }

    let report: serde_json::Value = stdout_json(&output);
    let expected_workspace = canonical_display(&workspace);

    if report["command"] != "status" {
        return Err(format!("unexpected status command payload: {report}").into());
    }
    if report["workspace_root"] != expected_workspace {
        return Err(format!("unexpected workspace_root payload: {report}").into());
    }
    if report["pre_state"] != "missing" || report["post_state"] != "missing" {
        return Err(format!("unexpected missing-index states: {report}").into());
    }
    if report["warnings"][0] != "derived index manifest not found for this workspace" {
        return Err(format!("unexpected warnings payload: {report}").into());
    }
    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_refresh_command_builds_manifest_backed_report() -> TestResult {
    let workspace = write_index_contract_workspace("boundline-index-refresh-contract");
    let output = run_boundline_in_with_env(
        &workspace,
        &["index", "refresh", "--workspace", workspace.to_string_lossy().as_ref()],
        &[(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, "ready")],
    );
    let text = terminal_text(&output);
    if output.status.code() != Some(0) {
        return Err(format!("expected refresh success, got output: {text}").into());
    }

    let report: serde_json::Value = stdout_json(&output);
    if report["command"] != "refresh" {
        return Err(format!("unexpected refresh command payload: {report}").into());
    }
    if report["pre_state"] != "missing" || report["post_state"] != "ready" {
        return Err(format!("unexpected refresh lifecycle states: {report}").into());
    }
    if report["recommended_action"] != "none" {
        return Err(format!("unexpected refresh recommended_action: {report}").into());
    }
    if report["manifest"].is_null() {
        return Err(
            format!("expected refresh command to persist manifest details: {report}").into()
        );
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_clean_command_removes_derived_index_artifacts() -> TestResult {
    let workspace = write_index_contract_workspace("boundline-index-clean-contract");
    let refresh_output = run_boundline_in_with_env(
        &workspace,
        &["index", "refresh", "--workspace", workspace.to_string_lossy().as_ref()],
        &[(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, "ready")],
    );
    if refresh_output.status.code() != Some(0) {
        return Err(format!(
            "expected refresh setup success, got: {}",
            terminal_text(&refresh_output)
        )
        .into());
    }

    let output = run_boundline_in(
        &workspace,
        &["index", "clean", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&output);
    if output.status.code() != Some(0) {
        return Err(format!("expected clean success, got output: {text}").into());
    }

    let report: serde_json::Value = stdout_json(&output);
    if report["command"] != "clean" {
        return Err(format!("unexpected clean command payload: {report}").into());
    }
    if report["post_state"] != "missing" {
        return Err(format!("expected clean to leave the index missing: {report}").into());
    }
    if !report["manifest"].is_null() {
        return Err(format!("expected clean command to clear manifest details: {report}").into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_rebuild_command_recreates_manifest_backed_report() -> TestResult {
    let workspace = write_index_contract_workspace("boundline-index-rebuild-contract");
    let output = run_boundline_in_with_env(
        &workspace,
        &["index", "rebuild", "--workspace", workspace.to_string_lossy().as_ref()],
        &[(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, "ready")],
    );
    let text = terminal_text(&output);
    if output.status.code() != Some(0) {
        return Err(format!("expected rebuild success, got output: {text}").into());
    }

    let report: serde_json::Value = stdout_json(&output);
    if report["command"] != "rebuild" {
        return Err(format!("unexpected rebuild command payload: {report}").into());
    }
    if report["post_state"] != "ready" {
        return Err(format!("expected rebuild command to restore a ready index: {report}").into());
    }
    if report["manifest"].is_null() {
        return Err(format!("expected rebuild command to return manifest details: {report}").into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_status_command_records_post_checkout_hook_staleness() -> TestResult {
    let workspace = temp_git_workspace("boundline-index-hook-status-contract");
    let source_path = workspace.join(INDEX_CONTRACT_SOURCE_REF);
    if let Some(parent) = source_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&source_path, "pub fn hook_contract() -> bool { true }\n")?;

    let refresh_output = run_boundline_in_with_env(
        &workspace,
        &["index", "refresh", "--workspace", workspace.to_string_lossy().as_ref()],
        &[(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, "ready")],
    );
    if refresh_output.status.code() != Some(0) {
        return Err(format!(
            "expected refresh setup success, got: {}",
            terminal_text(&refresh_output)
        )
        .into());
    }

    let status_output = run_boundline_in_with_env(
        &workspace,
        &["index", "status", "--workspace", workspace.to_string_lossy().as_ref()],
        &[("BOUNDLINE_INDEX_HOOK_TRIGGER", "post_checkout")],
    );
    let status_text = terminal_text(&status_output);
    if status_output.status.code() != Some(0) {
        return Err(
            format!("expected hook-triggered status success, got output: {status_text}").into()
        );
    }

    let report: serde_json::Value = stdout_json(&status_output);
    if report["post_state"] != "stale" {
        return Err(format!(
            "expected hook-triggered status to surface stale post_state: {report}"
        )
        .into());
    }
    if report["stale_reason"] != "branch_checkout" {
        return Err(format!(
            "expected hook-triggered status to preserve branch_checkout stale_reason: {report}"
        )
        .into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
fn write_index_contract_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = temp_empty_workspace(prefix);
    let source_path = workspace.join(INDEX_CONTRACT_SOURCE_REF);
    if let Some(parent) = source_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(source_path, "pub fn contract_index_refresh() -> bool { true }\n");
    workspace
}

fn canonical_display(path: &std::path::Path) -> String {
    match fs::canonicalize(path) {
        Ok(canonical_path) => canonical_path.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn lifecycle_report_fixture(command: IndexMaintenanceCommand) -> IndexLifecycleReport {
    IndexLifecycleReport {
        command,
        workspace_root: "workspace".to_string(),
        operation_id: format!("index-{}-operation", command.as_str()),
        pre_state: RetrievalIndexState::Ready,
        post_state: match command {
            IndexMaintenanceCommand::Status => RetrievalIndexState::Stale,
            IndexMaintenanceCommand::Refresh => RetrievalIndexState::Ready,
            IndexMaintenanceCommand::Rebuild => RetrievalIndexState::Ready,
            IndexMaintenanceCommand::Clean => RetrievalIndexState::Missing,
            IndexMaintenanceCommand::Doctor => RetrievalIndexState::Degraded,
        },
        recommended_action: match command {
            IndexMaintenanceCommand::Status => {
                "boundline index refresh --workspace workspace".to_string()
            }
            IndexMaintenanceCommand::Refresh => "none".to_string(),
            IndexMaintenanceCommand::Rebuild => "none".to_string(),
            IndexMaintenanceCommand::Clean => {
                "boundline index refresh --workspace workspace".to_string()
            }
            IndexMaintenanceCommand::Doctor => {
                "boundline index rebuild --workspace workspace".to_string()
            }
        },
        stale_reason: if command == IndexMaintenanceCommand::Status {
            Some(IndexStaleReason::GitHeadChanged)
        } else {
            None
        },
        warnings: if command == IndexMaintenanceCommand::Status {
            vec!["git HEAD changed since the last successful derived-index refresh".to_string()]
        } else {
            Vec::new()
        },
        manifest: if command == IndexMaintenanceCommand::Status {
            Some(DerivedIndexManifest {
                schema_version: "retrieval-index-v3".to_string(),
                workspace_root: "workspace".to_string(),
                git_branch: Some("main".to_string()),
                git_head: Some("abc123".to_string()),
                last_seen_head: Some("def456".to_string()),
                index_status: RetrievalIndexState::Ready,
                last_refresh_at: Some("2026-05-30 12:00:00".to_string()),
                last_refresh_reason: Some(IndexRefreshReason::ManualRefresh),
                stale_reason: None,
                file_count: 2,
                chunk_count: 2,
                fts5_state: ManifestFtsState::Ready,
                sqlite_vec_state: VectorExtensionState::Ready,
                semantic_engine: SemanticEngine::SqliteVec,
                workspace_fingerprint: "fnv64:0000000000000001".to_string(),
                config_fingerprint: "fnv64:0000000000000002".to_string(),
                chunker_fingerprint: "fnv64:0000000000000003".to_string(),
                embedding_model_fingerprint: "fnv64:0000000000000004".to_string(),
            })
        } else {
            None
        },
    }
}
