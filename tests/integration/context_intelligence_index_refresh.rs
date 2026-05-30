#![cfg(feature = "sqlite-vec")]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use boundline::cli::index::{execute_clean, execute_refresh};
use boundline::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use boundline::domain::context_intelligence::{
    DerivedIndexManifest, IndexStaleReason, RetrievalIndexState,
};
use boundline::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use boundline::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection, build_index_status_report,
};
use rusqlite::Connection;

use crate::workspace_fixture::{
    SEMANTIC_VECTOR_STATE_READY_VALUE, force_semantic_vector_state_override, temp_empty_workspace,
};

type TestResult = Result<(), Box<dyn Error>>;

const MANIFEST_RELATIVE: &str = ".boundline/context-intelligence/manifest.json";
const RETRIEVAL_INDEX_RELATIVE: &str = ".boundline/context-intelligence/retrieval-index.sqlite3";
const LIB_SOURCE_REF: &str = "src/lib.rs";
const SEMANTIC_SOURCE_REF: &str = "src/semantic.rs";
const OBSOLETE_SOURCE_REF: &str = "src/obsolete.rs";
const LIB_CHUNK_ID: &str = "semantic:src/lib.rs:0";
const SEMANTIC_CHUNK_ID: &str = "semantic:src/semantic.rs:0";
const OBSOLETE_CHUNK_ID: &str = "semantic:src/obsolete.rs:0";
const SEMANTIC_VECTORS_TABLE_NAME: &str = "semantic_vectors";
const BRANCH_CHECKOUT_PREVIOUS_HEAD: &str = "1111111111111111111111111111111111111111";
const BRANCH_CHECKOUT_CURRENT_HEAD: &str = "2222222222222222222222222222222222222222";

#[derive(Clone)]
struct SemanticChunkSnapshot {
    rowid: i64,
    content_hash: String,
}

fn write_index_refresh_workspace(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let workspace = temp_empty_workspace(prefix);
    fs::create_dir_all(workspace.join("src"))?;
    fs::write(workspace.join(LIB_SOURCE_REF), "pub fn refresh_ready() -> bool { true }\n")?;
    fs::write(
        workspace.join(SEMANTIC_SOURCE_REF),
        "pub fn lifecycle_candidate() -> bool { true }\n",
    )?;
    fs::write(
        workspace.join(OBSOLETE_SOURCE_REF),
        "pub fn removable_candidate() -> bool { true }\n",
    )?;
    Ok(workspace)
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_refresh_preserves_unchanged_chunks_and_removes_disappeared_sources() -> TestResult {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = write_index_refresh_workspace("boundline-context-index-refresh")?;

    run_refresh_cycle(&workspace)?;
    let initial_chunks = semantic_chunk_snapshots(&workspace)?;
    let initial_vectors = semantic_vector_chunk_ids(&workspace)?;

    let initial_lib = initial_chunks
        .get(LIB_CHUNK_ID)
        .ok_or("expected initial lib semantic chunk snapshot")?
        .clone();
    let initial_semantic = initial_chunks
        .get(SEMANTIC_CHUNK_ID)
        .ok_or("expected initial semantic chunk snapshot")?
        .clone();

    if initial_chunks.len() != 3 {
        return Err(
            format!("expected 3 initial semantic chunks, got {}", initial_chunks.len()).into()
        );
    }
    if !initial_vectors.contains(LIB_CHUNK_ID)
        || !initial_vectors.contains(SEMANTIC_CHUNK_ID)
        || !initial_vectors.contains(OBSOLETE_CHUNK_ID)
    {
        return Err("expected initial vector rows for all indexed semantic chunks".into());
    }

    fs::write(
        workspace.join(SEMANTIC_SOURCE_REF),
        "pub fn lifecycle_candidate() -> bool { false }\n",
    )?;
    fs::remove_file(workspace.join(OBSOLETE_SOURCE_REF))?;

    run_refresh_cycle(&workspace)?;
    let updated_chunks = semantic_chunk_snapshots(&workspace)?;
    let updated_vectors = semantic_vector_chunk_ids(&workspace)?;

    let updated_lib = updated_chunks
        .get(LIB_CHUNK_ID)
        .ok_or("expected unchanged lib semantic chunk after refresh")?;
    let updated_semantic = updated_chunks
        .get(SEMANTIC_CHUNK_ID)
        .ok_or("expected changed semantic chunk after refresh")?;

    if updated_chunks.len() != 2 {
        return Err(format!(
            "expected 2 semantic chunks after deletion, got {}",
            updated_chunks.len()
        )
        .into());
    }
    if updated_lib.rowid != initial_lib.rowid {
        return Err("expected unchanged semantic chunk rowid to remain stable".into());
    }
    if updated_lib.content_hash != initial_lib.content_hash {
        return Err("expected unchanged semantic chunk hash to remain stable".into());
    }
    if updated_semantic.content_hash == initial_semantic.content_hash {
        return Err("expected changed semantic chunk hash to update after refresh".into());
    }
    if updated_chunks.contains_key(OBSOLETE_CHUNK_ID) {
        return Err("expected disappeared source semantic chunk to be removed".into());
    }
    if updated_vectors.contains(OBSOLETE_CHUNK_ID) {
        return Err("expected disappeared source vector row to be removed".into());
    }
    if !updated_vectors.contains(LIB_CHUNK_ID) || !updated_vectors.contains(SEMANTIC_CHUNK_ID) {
        return Err("expected surviving semantic chunks to keep synchronized vector rows".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_status_reports_branch_checkout_staleness() -> TestResult {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = write_index_refresh_workspace("boundline-context-index-stale")?;

    run_refresh_cycle(&workspace)?;
    let mut manifest = read_manifest(&workspace)?;
    manifest.index_status = RetrievalIndexState::Ready;
    manifest.stale_reason = Some(IndexStaleReason::BranchCheckout);
    manifest.git_head = Some(BRANCH_CHECKOUT_CURRENT_HEAD.to_string());
    manifest.last_seen_head = Some(BRANCH_CHECKOUT_PREVIOUS_HEAD.to_string());
    write_manifest(&workspace, &manifest)?;

    let report = build_index_status_report(&workspace).map_err(string_error)?;
    if report.pre_state != RetrievalIndexState::Ready {
        return Err("expected branch-checkout status report to preserve ready pre-state".into());
    }
    if report.post_state != RetrievalIndexState::Stale {
        return Err("expected branch-checkout status report to project stale post-state".into());
    }
    if report.stale_reason != Some(IndexStaleReason::BranchCheckout) {
        return Err("expected branch-checkout stale reason in lifecycle status report".into());
    }
    if !report.recommended_action.contains("boundline index refresh --workspace") {
        return Err("expected branch-checkout status report to recommend refresh".into());
    }
    if report.warnings.is_empty() {
        return Err("expected branch-checkout status report to emit stale warning text".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_refresh_on_empty_workspace_leaves_missing_index_state() -> TestResult {
    let workspace = temp_empty_workspace("boundline-context-index-refresh-empty");

    let report = execute_refresh(Some(workspace.as_path())).map_err(string_error)?;

    if report.pre_state != RetrievalIndexState::Missing
        || report.post_state != RetrievalIndexState::Missing
    {
        return Err(format!(
            "expected empty refresh to preserve missing index state, got {report:?}"
        )
        .into());
    }
    if report.manifest.is_some() {
        return Err("expected empty refresh workspace to keep manifest absent".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_clean_preserves_non_artifact_files_in_context_directory() -> TestResult {
    let workspace = temp_empty_workspace("boundline-context-index-clean-preserve");
    let index_directory = workspace.join(".boundline/context-intelligence");
    fs::create_dir_all(&index_directory)?;
    fs::write(index_directory.join("notes.txt"), "keep this note\n")?;
    fs::write(index_directory.join("retrieval-index.sqlite3"), b"sqlite-fixture")?;

    let report = execute_clean(Some(workspace.as_path())).map_err(string_error)?;

    if report.post_state != RetrievalIndexState::Missing {
        return Err(format!("expected clean to leave missing index state, got {report:?}").into());
    }
    if !index_directory.is_dir() {
        return Err("expected non-artifact context directory to remain on disk".into());
    }
    if !index_directory.join("notes.txt").is_file() {
        return Err("expected clean to preserve non-artifact files".into());
    }
    if index_directory.join("retrieval-index.sqlite3").exists() {
        return Err("expected clean to remove managed database artifact".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
#[test]
fn index_clean_reports_remove_file_errors_for_non_file_artifacts() {
    let workspace = temp_empty_workspace("boundline-context-index-clean-error");
    let artifact_directory =
        workspace.join(".boundline/context-intelligence/retrieval-index.sqlite3");
    let create = fs::create_dir_all(&artifact_directory);
    assert!(create.is_ok(), "failed to create non-file artifact fixture: {create:?}");

    let error = execute_clean(Some(workspace.as_path())).expect_err(
        "expected clean to fail when a managed artifact path is a directory instead of a file",
    );
    assert!(error.contains("failed to remove"), "unexpected clean error: {error}");
}

#[cfg(feature = "sqlite-vec")]
fn run_refresh_cycle(workspace: &Path) -> TestResult {
    let projection = build_advanced_context_projection(
        "refresh semantic lifecycle evidence",
        workspace,
        &workspace_inputs(),
        &[],
        AdvancedContextBuildState {
            credibility: ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    if projection.semantic_engine().as_str() != "sqlite_vec" {
        return Err("expected sqlite_vec semantic engine during refresh integration".into());
    }

    Ok(())
}

#[cfg(feature = "sqlite-vec")]
fn workspace_inputs() -> Vec<ContextInput> {
    vec![
        workspace_input(LIB_SOURCE_REF, true),
        workspace_input(SEMANTIC_SOURCE_REF, false),
        workspace_input(OBSOLETE_SOURCE_REF, false),
    ]
}

#[cfg(feature = "sqlite-vec")]
fn workspace_input(reference: &str, primary: bool) -> ContextInput {
    ContextInput {
        kind: ContextInputKind::WorkspaceFile,
        reference: reference.to_string(),
        source: "workspace_scan".to_string(),
        rationale: "refresh integration fixture".to_string(),
        primary,
    }
}

#[cfg(feature = "sqlite-vec")]
fn semantic_chunk_snapshots(
    workspace: &Path,
) -> Result<BTreeMap<String, SemanticChunkSnapshot>, Box<dyn Error>> {
    let connection = Connection::open(workspace.join(RETRIEVAL_INDEX_RELATIVE))?;
    let mut statement = connection
        .prepare("SELECT rowid, chunk_id, content_hash FROM semantic_chunks ORDER BY chunk_id")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    })?;

    let mut snapshots = BTreeMap::new();
    for row in rows {
        let (rowid, chunk_id, content_hash) = row?;
        snapshots.insert(chunk_id, SemanticChunkSnapshot { rowid, content_hash });
    }
    Ok(snapshots)
}

#[cfg(feature = "sqlite-vec")]
fn semantic_vector_chunk_ids(workspace: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let connection = Connection::open(workspace.join(RETRIEVAL_INDEX_RELATIVE))?;
    let table_count = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE name = ?1",
        [SEMANTIC_VECTORS_TABLE_NAME],
        |row| row.get::<_, i64>(0),
    )?;
    if table_count == 0 {
        return Err(
            "expected semantic_vectors table to exist during sqlite-vec refresh test".into()
        );
    }

    let mut statement = connection.prepare(&format!(
        "SELECT chunk_id FROM {SEMANTIC_VECTORS_TABLE_NAME} ORDER BY chunk_id"
    ))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut chunk_ids = BTreeSet::new();
    for row in rows {
        chunk_ids.insert(row?);
    }
    Ok(chunk_ids)
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
