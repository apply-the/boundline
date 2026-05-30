#![cfg(feature = "sqlite-vec")]

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use boundline::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use boundline::domain::context_intelligence::{DerivedIndexManifest, RetrievalIndexState};
use boundline::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use boundline::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection, build_index_status_report,
};

use crate::workspace_fixture::{
    SEMANTIC_VECTOR_STATE_READY_VALUE, force_semantic_vector_state_override, temp_empty_workspace,
};

type TestResult = Result<(), Box<dyn Error>>;

const MANIFEST_RELATIVE: &str = ".boundline/context-intelligence/manifest.json";
const REBUILD_SOURCE_REF: &str = "src/lib.rs";
const LEGACY_CONFIG_FINGERPRINT: &str = "legacy-config-fingerprint";

fn write_index_rebuild_workspace(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let workspace = temp_empty_workspace(prefix);
    fs::create_dir_all(workspace.join("src"))?;
    fs::write(workspace.join(REBUILD_SOURCE_REF), "pub fn rebuild_candidate() -> bool { true }\n")?;
    Ok(workspace)
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_status_requires_rebuild_when_manifest_fingerprint_changes() -> TestResult {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = write_index_rebuild_workspace("boundline-context-index-rebuild")?;

    run_refresh_cycle(&workspace)?;
    let mut manifest = read_manifest(&workspace)?;
    manifest.config_fingerprint = LEGACY_CONFIG_FINGERPRINT.to_string();
    write_manifest(&workspace, &manifest)?;

    run_refresh_cycle(&workspace)?;
    let updated_manifest = read_manifest(&workspace)?;
    let report = build_index_status_report(&workspace).map_err(string_error)?;

    if updated_manifest.index_status != RetrievalIndexState::Incompatible {
        return Err("expected mismatched manifest fingerprint to require rebuild".into());
    }
    if report.pre_state != RetrievalIndexState::Incompatible
        || report.post_state != RetrievalIndexState::Incompatible
    {
        return Err(
            "expected lifecycle status report to preserve incompatible rebuild-required state"
                .into(),
        );
    }
    if !report.recommended_action.contains("boundline index rebuild --workspace") {
        return Err("expected rebuild-required lifecycle report to recommend rebuild".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
fn run_refresh_cycle(workspace: &Path) -> TestResult {
    let projection = build_advanced_context_projection(
        "refresh semantic rebuild evidence",
        workspace,
        &[workspace_input(REBUILD_SOURCE_REF)],
        &[],
        AdvancedContextBuildState {
            credibility: ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    if projection.semantic_engine().as_str() != "sqlite_vec" {
        return Err("expected sqlite_vec semantic engine during rebuild integration".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
fn workspace_input(reference: &str) -> ContextInput {
    ContextInput {
        kind: ContextInputKind::WorkspaceFile,
        reference: reference.to_string(),
        source: "workspace_scan".to_string(),
        rationale: "rebuild integration fixture".to_string(),
        primary: true,
    }
}

#[cfg(feature = "sqlite-vec")]
fn read_manifest(workspace: &Path) -> Result<DerivedIndexManifest, Box<dyn Error>> {
    let manifest_json = fs::read_to_string(workspace.join(MANIFEST_RELATIVE))?;
    let manifest = serde_json::from_str::<DerivedIndexManifest>(&manifest_json)?;
    manifest.validate().map_err(context_error)?;
    Ok(manifest)
}

#[cfg(feature = "sqlite-vec")]
fn write_manifest(workspace: &Path, manifest: &DerivedIndexManifest) -> TestResult {
    manifest.validate().map_err(context_error)?;
    let manifest_json = serde_json::to_string_pretty(manifest)?;
    fs::write(workspace.join(MANIFEST_RELATIVE), manifest_json)?;
    Ok(())
}

#[cfg(feature = "sqlite-vec")]
fn context_error(error: impl ToString) -> Box<dyn Error> {
    io::Error::other(error.to_string()).into()
}

#[cfg(feature = "sqlite-vec")]
fn string_error(error: String) -> Box<dyn Error> {
    io::Error::other(error).into()
}
