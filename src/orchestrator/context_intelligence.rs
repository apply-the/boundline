//! Builds bounded advanced-context retrieval projections using a local
//! SQLite + FTS5 index with structured fallback ordering.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(feature = "sqlite-vec")]
use std::sync::OnceLock;

use rusqlite::{Connection, params};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::audit::format_audit_timestamp;
use crate::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use crate::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ContextFidelityTier,
    ContextInclusionMode, ContextOmissionFinding, ContextOmissionSeverity,
    ContextPackEntryProjection, DerivedIndexManifest, DigestBackedArtifactRef, HybridOutcome,
    ImpactAnalysisFinding, ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus,
    IndexDoctorCheck, IndexDoctorConsistencyState, IndexDoctorReport, IndexDoctorStatus,
    IndexLifecycleReport, IndexMaintenanceCommand, IndexRefreshReason, IndexStaleReason,
    ManifestFtsState, PatchSafeEditAttempt, PatchSafeEditResultState, RelationshipCredibilityState,
    RelationshipKind, RelationshipProjection, RepositoryMapState, RetrievalCompatibilityState,
    RetrievalIndexState, RetrievalMatchOrigin, RetrievalMode, RetrievalScore, RetrievalSourceKind,
    RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState,
    SemanticChunkRecord, SemanticChunkState, SemanticEngine, SemanticPolicyState,
    SemanticTraceEventKind, SemanticTraceRecord, SnapshotCacheState, VectorExtensionState,
};
use crate::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use crate::domain::governance::{CanonSemanticEligibilityState, CanonSemanticProvenanceBoundary};
use crate::domain::project_index::{PROJECT_INDEX_FILE, ProjectIndex};
use crate::domain::project_memory::{
    CompatibilityOutcome, PromotionStateView, read_canon_semantic_artifact_surface,
};
use crate::domain::trace::current_timestamp_millis;

const BOUNDLINE_STATE_DIRECTORY: &str = ".boundline";
const CONTEXT_INTELLIGENCE_DIRECTORY: &str = "context-intelligence";
const RETRIEVAL_INDEX_FILE_NAME: &str = "retrieval-index.sqlite3";
const RETRIEVAL_INDEX_WAL_FILE_NAME: &str = "retrieval-index.sqlite3-wal";
const RETRIEVAL_INDEX_SHM_FILE_NAME: &str = "retrieval-index.sqlite3-shm";
const RETRIEVAL_INDEX_MANIFEST_FILE_NAME: &str = "manifest.json";
const SEMANTIC_CHUNKS_TABLE_NAME: &str = "semantic_chunks";
const RETRIEVAL_INDEX_SCHEMA_VERSION: &str = "retrieval-index-v3";
const SEMANTIC_INDEX_MANIFEST_ID: &str = "semantic-index-manifest";
const SEMANTIC_INDEX_HOOK_TRIGGER_ENV: &str = "BOUNDLINE_INDEX_HOOK_TRIGGER";
const SEMANTIC_INDEX_HOOK_TRIGGER_POST_CHECKOUT: &str = "post_checkout";
const SEMANTIC_INDEX_HOOK_TRIGGER_POST_MERGE: &str = "post_merge";
const SEMANTIC_INDEX_HOOK_TRIGGER_POST_REWRITE: &str = "post_rewrite";
const SEMANTIC_VECTOR_STATE_OVERRIDE_ENV: &str = "BOUNDLINE_SEMANTIC_VECTOR_STATE_OVERRIDE";
const SEMANTIC_VECTOR_STATE_READY_VALUE: &str = "ready";
const SEMANTIC_VECTOR_STATE_MISSING_VALUE: &str = "missing";
const SEMANTIC_VECTOR_STATE_STALE_VALUE: &str = "stale";
const SEMANTIC_VECTOR_STATE_DEGRADED_VALUE: &str = "degraded";
const SEMANTIC_VECTOR_STATE_CORRUPT_VALUE: &str = "corrupt";
const SEMANTIC_VECTOR_STATE_UNSUPPORTED_VALUE: &str = "unsupported";
const SEMANTIC_REFRESH_PENDING_REASON: &str =
    "semantic acceleration scaffold initialized; no semantic refresh has completed yet";
const SQLITE_VEC_MODULE_NAME: &str = "vec0";
const SQLITE_VEC_EACH_MODULE_NAME: &str = "vec_each";
const SEMANTIC_ACCELERATION_MISSING_REASON: &str = "semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval";
const SEMANTIC_ACCELERATION_SKIPPED_REASON: &str = "semantic acceleration is configured locally but bounded retrieval did not reach semantic expansion";
const SEMANTIC_ACCELERATION_STALE_REASON: &str = "semantic acceleration is enabled but semantic vector state is stale; using baseline structured retrieval";
const SEMANTIC_ACCELERATION_DEGRADED_REASON: &str = "semantic acceleration is enabled but sqlite-vec capability is degraded; using baseline structured retrieval";
const SEMANTIC_ACCELERATION_CORRUPT_REASON: &str = "semantic acceleration is enabled but sqlite-vec state is corrupt; using baseline structured retrieval";
const SEMANTIC_ACCELERATION_UNSUPPORTED_REASON: &str = "semantic acceleration is enabled but SQLite vector capability inspection is unavailable; using baseline structured retrieval";
const SEMANTIC_BASELINE_ONLY_REASON: &str =
    "semantic acceleration evaluated the bounded query but kept the V1 candidate set unchanged";
const SEMANTIC_EXPANDED_REASON: &str =
    "semantic acceleration expanded the V1 candidate set with additional local evidence";
const SEMANTIC_RERANKED_REASON: &str =
    "semantic acceleration reranked the V1 candidate set within bounded authority order";
const SEMANTIC_EXPAND_SELECTION_REASON: &str =
    "semantic similarity expanded the V1 candidate set with bounded local evidence";
const SEMANTIC_RERANK_SELECTION_REASON: &str =
    "semantic similarity reranked the candidate within bounded authority order";
const SEMANTIC_REJECTED_LIMIT_REASON: &str = "semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged";
const SEMANTIC_SCHEMA_LINE_V1: &str = "boundline.semantic_chunk.v1";
const SEMANTIC_VECTORS_TABLE_NAME: &str = "semantic_vectors";
const SEMANTIC_VECTOR_TABLE_DEFINITION: &str =
    "chunk_id text primary key, embedding float[48] distance_metric=cosine";
const DERIVED_INDEX_CONFIG_FINGERPRINT_NAMESPACE: &str = "derived-index-config-v1";
const DERIVED_INDEX_CHUNKER_FINGERPRINT_NAMESPACE: &str = "derived-index-chunker-v1";
const DERIVED_INDEX_EMBEDDING_FINGERPRINT_NAMESPACE: &str = "derived-index-embedding-v1";
const DERIVED_INDEX_IGNORE_PATTERN_DATABASE: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3";
const DERIVED_INDEX_IGNORE_PATTERN_MANIFEST: &str = ".boundline/context-intelligence/manifest.json";
const DERIVED_INDEX_IGNORE_PATTERN_WAL: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3-wal";
const DERIVED_INDEX_IGNORE_PATTERN_SHM: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3-shm";
const MAX_SEMANTIC_CHUNK_BYTES: usize = 8 * 1024;
const SEMANTIC_EMBEDDING_DIMENSIONS: usize = 48;
const SEMANTIC_FEATURE_NGRAM_WIDTH: usize = 3;
const MIN_SEMANTIC_TOKEN_LENGTH: usize = 3;
const MIN_SEMANTIC_SIMILARITY_SCORE: f64 = 0.220;
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const MAX_INDEXED_BYTES: usize = 32 * 1024;
const MAX_QUERY_TERMS: usize = 8;
const LARGE_CONTEXT_EXCERPT_THRESHOLD_BYTES: u64 = 8 * 1024;
const LARGE_CONTEXT_DIGEST_THRESHOLD_BYTES: u64 = 16 * 1024;
const CONTEXT_REASON_UNSAFE_FULL_READ_REFUSED: &str = "unsafe_full_read_refused";
const CONTEXT_REASON_CRITICAL_UNAVAILABLE: &str = "critical_context_unavailable";
const CONTEXT_REASON_CRITICAL_DOWNGRADED: &str = "critical_context_downgraded";
const CONTEXT_REASON_SEARCH_REQUIRED_BEFORE_READ: &str = "search_required_before_read";
const CONTEXT_REASON_ARTIFACT_COMPACTED_TO_DIGEST: &str = "artifact_compacted_to_digest";
const CONTEXT_REASON_REPOSITORY_MAP_UNAVAILABLE: &str = "repository_map_unavailable";
const CONTEXT_REASON_SNAPSHOT_CACHE_STALE: &str = "snapshot_cache_stale";
const CONTEXT_REASON_TRACKED_CACHE_DETECTED: &str = "tracked_cache_detected";
const CONTEXT_REASON_ARCHIVED_CONTEXT_EXCLUDED: &str = "archived_context_excluded";

#[cfg(feature = "sqlite-vec")]
static SQLITE_VEC_AUTO_EXTENSION_REGISTRATION: OnceLock<Result<(), String>> = OnceLock::new();

#[cfg(feature = "sqlite-vec")]
type SqliteVecAutoExtensionInit = unsafe extern "C" fn(
    *mut rusqlite::ffi::sqlite3,
    *mut *const std::ffi::c_char,
    *const rusqlite::ffi::sqlite3_api_routines,
) -> std::ffi::c_int;
const CANON_PUBLICATION_TARGET_STABLE: &str = "stable";
const CANON_ARTIFACT_FILE_MISSING_REASON: &str =
    "Canon artifact file is missing from the workspace";
const CANON_SEMANTIC_SIDECAR_MISSING_REASON: &str = "Canon artifact metadata sidecar is missing";
const CANON_SEMANTIC_DESCRIPTOR_MISSING_REASON: &str =
    "Canon artifact metadata sidecar is missing a semantic descriptor";
const CANON_SEMANTIC_PROVENANCE_MISSING_REASON: &str =
    "Canon artifact semantic descriptor is missing required provenance metadata";
const CANON_SEMANTIC_SURFACE_BLOCKED_REASON: &str =
    "Canon artifact is not published to a stable Canon surface";
const CANON_SEMANTIC_EXCLUDED_REASON: &str =
    "Canon artifact semantic descriptor excluded the artifact from semantic retrieval";
const CANON_SEMANTIC_SKIP_SELECTION_PREFIX: &str =
    "Canon semantic compatibility skipped the artifact:";
const SEMANTIC_TRACE_CAPABILITY_REASON_PREFIX: &str =
    "semantic policy evaluated local capability as";
const SEMANTIC_TRACE_EXTENSION_LOAD_ATTEMPTED_PREFIX: &str =
    "trusted sqlite-vec extension load attempted";
const SEMANTIC_TRACE_INDEX_REFRESHED_PREFIX: &str =
    "semantic index refreshed eligible local evidence set";
const SEMANTIC_TRACE_CHUNK_BLOCKED_PREFIX: &str = "semantic chunk was blocked:";
const SEMANTIC_TRACE_VECTOR_QUERY_EXECUTED_PREFIX: &str =
    "vector query executed through semantic engine";
const SEMANTIC_TRACE_VECTOR_CANDIDATES_RETURNED_PREFIX: &str =
    "vector query returned chunk candidates before source collapse";
const SEMANTIC_TRACE_HYBRID_OUTCOME_PREFIX: &str = "semantic retrieval ended with retrieval state";

#[derive(Debug, Clone, Copy)]
pub struct AdvancedContextBuildState<'a> {
    pub credibility: ContextPackCredibility,
    pub staleness_reason: Option<&'a str>,
    pub semantic_policy: SemanticAccelerationPolicyState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitWorkspaceState {
    branch: String,
    head: String,
}

/// Builds the persisted advanced-context projection that planning, status,
/// and inspect surfaces share.
pub fn build_advanced_context_projection(
    goal_text: &str,
    workspace_ref: &Path,
    inputs: &[ContextInput],
    selected_targets: &[String],
    build_state: AdvancedContextBuildState<'_>,
    policy: &AdvancedContextConfig,
) -> AdvancedContextProjection {
    build_advanced_context_projection_with_vector_state(
        goal_text,
        workspace_ref,
        inputs,
        selected_targets,
        build_state,
        policy,
        vector_extension_state_override_from_env(),
    )
}

/// Builds the current manifest-backed lifecycle report for `boundline index status`.
pub fn build_index_status_report(workspace_ref: &Path) -> Result<IndexLifecycleReport, String> {
    let workspace_root = workspace_ref.to_string_lossy().into_owned();
    let manifest = read_derived_index_manifest(workspace_ref)
        .map_err(|error| error.to_string())?
        .map(|manifest| observe_manifest_git_freshness(workspace_ref, manifest))
        .transpose()
        .map_err(|error| error.to_string())?;

    let (pre_state, post_state, recommended_action, stale_reason, warnings, manifest) =
        match manifest {
            Some(manifest) => {
                let mut warnings = Vec::new();
                let pre_state = manifest.index_status;
                let stale_reason = manifest.effective_stale_reason();
                let post_state = if let Some(reason) = stale_reason {
                    warnings.push(index_stale_warning(reason).to_string());
                    RetrievalIndexState::Stale
                } else {
                    manifest.index_status
                };
                (
                    pre_state,
                    post_state,
                    recommended_action_for_index_state(post_state, workspace_ref),
                    stale_reason,
                    warnings,
                    Some(manifest),
                )
            }
            None => (
                RetrievalIndexState::Missing,
                RetrievalIndexState::Missing,
                recommended_action_for_index_state(RetrievalIndexState::Missing, workspace_ref),
                None,
                vec!["derived index manifest not found for this workspace".to_string()],
                None,
            ),
        };

    let report = IndexLifecycleReport {
        command: IndexMaintenanceCommand::Status,
        workspace_root,
        operation_id: format!("index-status:{}", Uuid::new_v4()),
        pre_state,
        post_state,
        recommended_action,
        stale_reason,
        warnings,
        manifest,
    };
    report.validate().map_err(|error| error.to_string())?;
    Ok(report)
}

/// Builds the current derived-index doctor report for `boundline index doctor`.
pub fn build_index_doctor_report(workspace_ref: &Path) -> Result<IndexDoctorReport, String> {
    let tracked_index_files = tracked_index_files(workspace_ref);
    let missing_ignore_rules = missing_ignore_rules(workspace_ref);
    let wal_sidecars_present = wal_sidecar_paths(workspace_ref).iter().any(|path| path.is_file());
    let (manifest_consistency, manifest_available) = inspect_manifest_consistency(workspace_ref);
    let vector_schema_consistency = inspect_vector_schema_consistency(workspace_ref);
    let checks = build_index_doctor_checks(
        workspace_ref,
        &tracked_index_files,
        &missing_ignore_rules,
        wal_sidecars_present,
        manifest_consistency,
        manifest_available,
        vector_schema_consistency,
    );
    let status = aggregate_index_doctor_status(&checks);
    let report = IndexDoctorReport {
        status,
        checks,
        tracked_index_files,
        missing_ignore_rules,
        wal_sidecars_present,
        manifest_consistency,
        vector_schema_consistency,
    };
    report.validate().map_err(|error| error.to_string())?;
    Ok(report)
}

fn build_index_doctor_checks(
    workspace_ref: &Path,
    tracked_index_files: &[String],
    missing_ignore_rules: &[String],
    wal_sidecars_present: bool,
    manifest_consistency: IndexDoctorConsistencyState,
    manifest_available: bool,
    vector_schema_consistency: IndexDoctorConsistencyState,
) -> Vec<IndexDoctorCheck> {
    vec![
        tracked_index_files_check(workspace_ref, tracked_index_files),
        managed_ignore_rules_check(workspace_ref, missing_ignore_rules),
        wal_sidecars_check(workspace_ref, wal_sidecars_present),
        manifest_consistency_check(workspace_ref, manifest_consistency, manifest_available),
        vector_schema_check(workspace_ref, vector_schema_consistency),
    ]
}

fn tracked_index_files_check(
    workspace_ref: &Path,
    tracked_index_files: &[String],
) -> IndexDoctorCheck {
    let workspace = workspace_ref.to_string_lossy();
    if tracked_index_files.is_empty() {
        return IndexDoctorCheck {
            check_name: "tracked_index_files".to_string(),
            result: IndexDoctorStatus::Passed,
            detail: "derived index artifacts are not tracked by Git".to_string(),
            suggested_fix: "none".to_string(),
        };
    }

    IndexDoctorCheck {
        check_name: "tracked_index_files".to_string(),
        result: IndexDoctorStatus::Failed,
        detail: format!("tracked derived index artifacts: {}", tracked_index_files.join(", ")),
        suggested_fix: format!(
            "git -C {workspace} rm --cached -- {} && boundline init --workspace {workspace} --force",
            tracked_index_files.join(" ")
        ),
    }
}

fn managed_ignore_rules_check(
    workspace_ref: &Path,
    missing_ignore_rules: &[String],
) -> IndexDoctorCheck {
    let workspace = workspace_ref.to_string_lossy();
    if missing_ignore_rules.is_empty() {
        return IndexDoctorCheck {
            check_name: "managed_ignore_rules".to_string(),
            result: IndexDoctorStatus::Passed,
            detail: "derived index ignore rules cover the database, manifest, and SQLite sidecars"
                .to_string(),
            suggested_fix: "none".to_string(),
        };
    }

    IndexDoctorCheck {
        check_name: "managed_ignore_rules".to_string(),
        result: IndexDoctorStatus::Advisory,
        detail: format!(
            "missing managed derived index ignore rules: {}",
            missing_ignore_rules.join(", ")
        ),
        suggested_fix: format!("boundline init --workspace {workspace} --force"),
    }
}

fn wal_sidecars_check(workspace_ref: &Path, wal_sidecars_present: bool) -> IndexDoctorCheck {
    let workspace = workspace_ref.to_string_lossy();
    if !wal_sidecars_present {
        return IndexDoctorCheck {
            check_name: "wal_sidecars".to_string(),
            result: IndexDoctorStatus::Passed,
            detail: "SQLite WAL and SHM sidecars are not present".to_string(),
            suggested_fix: "none".to_string(),
        };
    }

    IndexDoctorCheck {
        check_name: "wal_sidecars".to_string(),
        result: IndexDoctorStatus::Advisory,
        detail: "SQLite WAL or SHM sidecars are present next to the derived index".to_string(),
        suggested_fix: format!(
            "close any process holding the index open, then rerun boundline index refresh --workspace {workspace}"
        ),
    }
}

fn manifest_consistency_check(
    workspace_ref: &Path,
    manifest_consistency: IndexDoctorConsistencyState,
    manifest_available: bool,
) -> IndexDoctorCheck {
    let workspace = workspace_ref.to_string_lossy();
    let (result, detail, suggested_fix) = match manifest_consistency {
        IndexDoctorConsistencyState::Consistent => (
            IndexDoctorStatus::Passed,
            "derived index manifest is present and internally consistent".to_string(),
            "none".to_string(),
        ),
        IndexDoctorConsistencyState::Missing if !manifest_available => (
            IndexDoctorStatus::Advisory,
            "derived index manifest is missing".to_string(),
            format!("boundline index refresh --workspace {workspace}"),
        ),
        IndexDoctorConsistencyState::Missing => (
            IndexDoctorStatus::Advisory,
            "derived index manifest could not be observed".to_string(),
            format!("boundline index refresh --workspace {workspace}"),
        ),
        IndexDoctorConsistencyState::Corrupt => (
            IndexDoctorStatus::Failed,
            "derived index manifest is unreadable or corrupt".to_string(),
            format!("boundline index rebuild --workspace {workspace}"),
        ),
        IndexDoctorConsistencyState::Invalid => (
            IndexDoctorStatus::Failed,
            "derived index manifest does not match the current workspace or on-disk index"
                .to_string(),
            format!("boundline index rebuild --workspace {workspace}"),
        ),
    };

    IndexDoctorCheck {
        check_name: "manifest_consistency".to_string(),
        result,
        detail,
        suggested_fix,
    }
}

fn vector_schema_check(
    workspace_ref: &Path,
    vector_schema_consistency: IndexDoctorConsistencyState,
) -> IndexDoctorCheck {
    let workspace = workspace_ref.to_string_lossy();
    let (result, detail, suggested_fix) = match vector_schema_consistency {
        IndexDoctorConsistencyState::Consistent => (
            IndexDoctorStatus::Passed,
            "derived index schema and semantic vector tables are internally consistent"
                .to_string(),
            "none".to_string(),
        ),
        IndexDoctorConsistencyState::Missing => (
            IndexDoctorStatus::Advisory,
            "derived index database is missing".to_string(),
            format!("boundline index refresh --workspace {workspace}"),
        ),
        IndexDoctorConsistencyState::Corrupt => (
            IndexDoctorStatus::Failed,
            "derived index database could not be opened or queried".to_string(),
            format!("boundline index rebuild --workspace {workspace}"),
        ),
        IndexDoctorConsistencyState::Invalid => (
            IndexDoctorStatus::Failed,
            "derived index schema is missing required semantic tables for the current capability state"
                .to_string(),
            format!("boundline index rebuild --workspace {workspace}"),
        ),
    };

    IndexDoctorCheck {
        check_name: "vector_schema_consistency".to_string(),
        result,
        detail,
        suggested_fix,
    }
}

fn aggregate_index_doctor_status(checks: &[IndexDoctorCheck]) -> IndexDoctorStatus {
    if checks.iter().any(|check| check.result == IndexDoctorStatus::Failed) {
        IndexDoctorStatus::Failed
    } else if checks.iter().any(|check| check.result == IndexDoctorStatus::Advisory) {
        IndexDoctorStatus::Advisory
    } else {
        IndexDoctorStatus::Passed
    }
}

fn tracked_index_files(workspace_ref: &Path) -> Vec<String> {
    index_artifact_relative_paths()
        .into_iter()
        .filter_map(|relative_path| git_tracked_path(workspace_ref, relative_path))
        .collect()
}

fn git_tracked_path(workspace_ref: &Path, relative_path: &'static str) -> Option<String> {
    git_command_output(workspace_ref, ["ls-files", "--error-unmatch", "--", relative_path])
}

fn missing_ignore_rules(workspace_ref: &Path) -> Vec<String> {
    let gitignore_path = workspace_ref.join(".gitignore");
    let contents = fs::read_to_string(gitignore_path).unwrap_or_default();
    required_ignore_patterns()
        .into_iter()
        .filter(|pattern| !contents.lines().any(|line| line.trim() == *pattern))
        .map(str::to_string)
        .collect()
}

fn wal_sidecar_paths(workspace_ref: &Path) -> [PathBuf; 2] {
    let base = context_intelligence_state_directory(workspace_ref);
    [base.join(RETRIEVAL_INDEX_WAL_FILE_NAME), base.join(RETRIEVAL_INDEX_SHM_FILE_NAME)]
}

fn inspect_manifest_consistency(workspace_ref: &Path) -> (IndexDoctorConsistencyState, bool) {
    match read_derived_index_manifest(workspace_ref) {
        Ok(Some(manifest)) => {
            if manifest.workspace_root != workspace_ref.to_string_lossy()
                || !retrieval_index_path(workspace_ref).is_file()
            {
                (IndexDoctorConsistencyState::Invalid, true)
            } else {
                (IndexDoctorConsistencyState::Consistent, true)
            }
        }
        Ok(None) => (IndexDoctorConsistencyState::Missing, false),
        Err(_) => (IndexDoctorConsistencyState::Corrupt, true),
    }
}

fn inspect_vector_schema_consistency(workspace_ref: &Path) -> IndexDoctorConsistencyState {
    if !retrieval_index_path(workspace_ref).is_file() {
        return IndexDoctorConsistencyState::Missing;
    }

    let connection = match open_connection(workspace_ref) {
        Ok(connection) => connection,
        Err(_) => return IndexDoctorConsistencyState::Corrupt,
    };
    vector_schema_consistency_from_tables(
        table_exists(&connection, SEMANTIC_CHUNKS_TABLE_NAME),
        semantic_vector_table_exists(&connection),
        detect_vector_extension_state(&connection),
    )
}

fn vector_schema_consistency_from_tables(
    semantic_chunks_available: Result<bool, ContextIntelligenceBuildError>,
    vector_table_available: Result<bool, ContextIntelligenceBuildError>,
    vector_capability: VectorExtensionState,
) -> IndexDoctorConsistencyState {
    let semantic_chunks_available = match semantic_chunks_available {
        Ok(available) => available,
        Err(_) => return IndexDoctorConsistencyState::Corrupt,
    };
    if !semantic_chunks_available {
        return IndexDoctorConsistencyState::Invalid;
    }

    let vector_table_available = match vector_table_available {
        Ok(available) => available,
        Err(_) => return IndexDoctorConsistencyState::Corrupt,
    };
    if vector_capability == VectorExtensionState::Ready && !vector_table_available {
        return IndexDoctorConsistencyState::Invalid;
    }

    IndexDoctorConsistencyState::Consistent
}

fn table_exists(
    connection: &Connection,
    table_name: &str,
) -> Result<bool, ContextIntelligenceBuildError> {
    let mut statement = connection
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    let mut rows = statement
        .query(params![table_name])
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    rows.next()
        .map(|row| row.is_some())
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))
}

fn index_artifact_relative_paths() -> [&'static str; 4] {
    [
        DERIVED_INDEX_IGNORE_PATTERN_DATABASE,
        DERIVED_INDEX_IGNORE_PATTERN_MANIFEST,
        DERIVED_INDEX_IGNORE_PATTERN_WAL,
        DERIVED_INDEX_IGNORE_PATTERN_SHM,
    ]
}

fn required_ignore_patterns() -> [&'static str; 4] {
    index_artifact_relative_paths()
}

fn build_advanced_context_projection_with_vector_state(
    goal_text: &str,
    workspace_ref: &Path,
    inputs: &[ContextInput],
    selected_targets: &[String],
    build_state: AdvancedContextBuildState<'_>,
    policy: &AdvancedContextConfig,
    vector_extension_state_override: Option<VectorExtensionState>,
) -> AdvancedContextProjection {
    let query_id = Uuid::new_v4().to_string();
    let credibility = build_state.credibility;
    let staleness_reason = build_state.staleness_reason;
    let semantic_policy = build_state.semantic_policy;

    if policy.retrieval_mode == RetrievalMode::Disabled {
        return terminal_projection(TerminalProjectionInputs {
            query_id,
            workspace_ref,
            inputs,
            selected_targets,
            policy,
            semantic_policy,
            retrieval_state: RetrievalState::Insufficient,
            retrieval_index_state: RetrievalIndexState::Insufficient,
            terminal_reason: "advanced retrieval is disabled by configuration".to_string(),
        });
    }

    let documents = collect_retrieval_documents(
        workspace_ref,
        inputs,
        selected_targets,
        credibility,
        staleness_reason,
        semantic_policy,
    );

    if documents.is_empty() {
        return terminal_projection(TerminalProjectionInputs {
            query_id,
            workspace_ref,
            inputs,
            selected_targets,
            policy,
            semantic_policy,
            retrieval_state: RetrievalState::Insufficient,
            retrieval_index_state: RetrievalIndexState::Insufficient,
            terminal_reason: "no local documents were available for bounded advanced retrieval"
                .to_string(),
        });
    }

    let document_map = documents
        .iter()
        .map(|document| (document.source_ref.clone(), document))
        .collect::<BTreeMap<_, _>>();
    let canon_rejected_decisions = canon_rejected_candidate_decisions(&documents, semantic_policy);
    let queryable_documents = selectable_documents(&documents, semantic_policy);
    let default_index_state = default_index_state(credibility, &documents);
    let default_degraded_reason = credibility_degradation_reason(credibility, staleness_reason);

    if queryable_documents.is_empty() {
        let terminal_reason = Some(
            "no compatible local documents were available for bounded advanced retrieval"
                .to_string(),
        );
        let semantic_projection = semantic_projection_for_local_result(
            semantic_policy,
            VectorExtensionState::Unsupported,
        );
        let rejected_candidates = canon_rejected_decisions
            .iter()
            .enumerate()
            .filter_map(|(index, decision)| {
                document_map.get(&decision.source_ref).map(|document| {
                    candidate_from_decision(
                        format!("rejected-candidate-{}", index + 1),
                        decision,
                        document,
                    )
                })
            })
            .collect::<Vec<_>>();
        let semantic_trace_records = build_semantic_trace_records(SemanticTraceRecordInputs {
            query_id: &query_id,
            documents: &documents,
            semantic_policy_state: semantic_projection.policy_state,
            semantic_capability_state: semantic_projection.capability_state,
            hybrid_outcome: semantic_projection.hybrid_outcome,
            retrieval_state: RetrievalState::Insufficient,
            retrieval_index_state: default_index_state,
            execution_details: SemanticTraceExecutionDetails::default(),
            terminal_reason: terminal_reason.as_deref(),
            selected_evidence: &[],
            rejected_candidates: &rejected_candidates,
        });
        let substrate_fields = build_context_substrate_projection_fields(
            workspace_ref,
            inputs,
            selected_targets,
            &documents,
            &[],
            &rejected_candidates,
            default_index_state,
        );

        return AdvancedContextProjection {
            query_id,
            retrieval_mode: policy.retrieval_mode,
            retrieval_state: RetrievalState::Insufficient,
            retrieval_index_state: default_index_state,
            semantic_policy_state: semantic_projection.policy_state,
            semantic_capability_state: semantic_projection.capability_state,
            hybrid_outcome: semantic_projection.hybrid_outcome,
            budgets: policy.budgets.clone(),
            remote_policy_state: policy.remote_policy,
            used_remote: false,
            terminal_reason: combine_terminal_reasons(
                terminal_reason,
                semantic_projection.terminal_reason,
            ),
            selected_evidence: Vec::new(),
            rejected_candidates,
            semantic_trace_records,
            relationships: Vec::new(),
            impact_findings: Vec::new(),
            context_pack_entries: substrate_fields.context_pack_entries,
            omission_findings: substrate_fields.omission_findings,
            repository_map_state: substrate_fields.repository_map_state,
            snapshot_cache_state: substrate_fields.snapshot_cache_state,
            patch_safe_edit_attempts: substrate_fields.patch_safe_edit_attempts,
        };
    }

    let (
        base_selected_decisions,
        rejected_decisions,
        retrieval_state,
        retrieval_index_state,
        semantic_projection,
        base_terminal_reason,
        trace_execution_details,
    ) = match refresh_and_query_index(
        workspace_ref,
        goal_text,
        selected_targets,
        &queryable_documents,
        policy.budgets.evidence_limit,
        policy.budgets.expansion_limit,
        vector_extension_state_override,
    ) {
        Ok(result) if !result.lexical_matches.is_empty() => {
            let base_selected =
                candidate_decisions_from_lexical_matches(&promote_selected_ranked_refs(
                    result.lexical_matches,
                    selected_targets,
                    &queryable_documents,
                    policy.budgets.evidence_limit,
                ));
            let hybrid_result = apply_local_semantic_hybrid(
                HybridSelectionInputs {
                    semantic_policy,
                    selected_targets,
                    documents: &queryable_documents,
                    vector_extension_state: result.vector_extension_state,
                    evidence_limit: policy.budgets.evidence_limit,
                    expansion_limit: policy.budgets.expansion_limit,
                    base_retrieval_state: SelectionStrategy::Fts.retrieval_state(credibility),
                },
                base_selected,
                result.semantic_matches,
            );
            (
                hybrid_result.selected,
                merge_rejected_decisions(canon_rejected_decisions.clone(), hybrid_result.rejected),
                hybrid_result.retrieval_state,
                default_index_state,
                hybrid_result.semantic_projection,
                default_degraded_reason.clone(),
                SemanticTraceExecutionDetails {
                    extension_load_attempted: result.extension_load_attempted,
                    vector_query_attempted: result.vector_query_attempted,
                    vector_chunk_candidates_returned: result.vector_chunk_candidates_returned,
                },
            )
        }
        Ok(result) => {
            let fallback_refs = structured_fallback_refs(
                &queryable_documents,
                selected_targets,
                policy.budgets.evidence_limit,
            );
            if fallback_refs.is_empty() {
                return terminal_projection(TerminalProjectionInputs {
                    query_id,
                    workspace_ref,
                    inputs,
                    selected_targets,
                    policy,
                    semantic_policy,
                    retrieval_state: RetrievalState::Insufficient,
                    retrieval_index_state: default_index_state,
                    terminal_reason: "no indexed evidence matched the bounded goal".to_string(),
                });
            }
            let base_reason =
                    "SQLite retrieval returned no stronger local match; promoted structured bounded context evidence"
                        .to_string();
            let hybrid_result = apply_local_semantic_hybrid(
                HybridSelectionInputs {
                    semantic_policy,
                    selected_targets,
                    documents: &documents,
                    vector_extension_state: result.vector_extension_state,
                    evidence_limit: policy.budgets.evidence_limit,
                    expansion_limit: policy.budgets.expansion_limit,
                    base_retrieval_state: SelectionStrategy::StructuredFallback
                        .retrieval_state(credibility),
                },
                candidate_decisions_from_refs(
                    &fallback_refs,
                    RetrievalMatchOrigin::StructuredFallback,
                ),
                result.semantic_matches,
            );
            (
                hybrid_result.selected,
                merge_rejected_decisions(canon_rejected_decisions.clone(), hybrid_result.rejected),
                hybrid_result.retrieval_state,
                default_index_state,
                hybrid_result.semantic_projection,
                combine_terminal_reasons(Some(base_reason), default_degraded_reason.clone()),
                SemanticTraceExecutionDetails {
                    extension_load_attempted: result.extension_load_attempted,
                    vector_query_attempted: result.vector_query_attempted,
                    vector_chunk_candidates_returned: result.vector_chunk_candidates_returned,
                },
            )
        }
        Err(error) => {
            let fallback_refs = structured_fallback_refs(
                &documents,
                selected_targets,
                policy.budgets.evidence_limit,
            );
            if fallback_refs.is_empty() {
                return terminal_projection(TerminalProjectionInputs {
                    query_id,
                    workspace_ref,
                    inputs,
                    selected_targets,
                    policy,
                    semantic_policy,
                    retrieval_state: RetrievalState::Unavailable,
                    retrieval_index_state: RetrievalIndexState::Stale,
                    terminal_reason: error.to_string(),
                });
            }
            (
                candidate_decisions_from_refs(
                    &fallback_refs,
                    RetrievalMatchOrigin::StructuredFallback,
                ),
                canon_rejected_decisions,
                RetrievalState::Degraded,
                RetrievalIndexState::Stale,
                semantic_projection_for_local_error(semantic_policy, &error.to_string()),
                combine_terminal_reasons(
                    Some(format!("SQLite retrieval degraded to structured fallback: {error}")),
                    default_degraded_reason.clone(),
                ),
                SemanticTraceExecutionDetails {
                    extension_load_attempted: semantic_policy
                        == SemanticAccelerationPolicyState::Local,
                    vector_query_attempted: semantic_policy
                        == SemanticAccelerationPolicyState::Local,
                    vector_chunk_candidates_returned: 0,
                },
            )
        }
    };

    let selected_evidence = base_selected_decisions
        .iter()
        .enumerate()
        .filter_map(|(index, decision)| {
            document_map.get(&decision.source_ref).map(|document| {
                candidate_from_decision(format!("candidate-{}", index + 1), decision, document)
            })
        })
        .collect::<Vec<_>>();
    let rejected_candidates = rejected_decisions
        .iter()
        .enumerate()
        .filter_map(|(index, decision)| {
            document_map.get(&decision.source_ref).map(|document| {
                candidate_from_decision(
                    format!("rejected-candidate-{}", index + 1),
                    decision,
                    document,
                )
            })
        })
        .collect::<Vec<_>>();

    let (relationships, impact_findings) = derive_relationships_and_findings(
        workspace_ref,
        &selected_evidence,
        credibility,
        staleness_reason,
    );

    let terminal_reason =
        combine_terminal_reasons(base_terminal_reason, semantic_projection.terminal_reason);
    let semantic_trace_records = build_semantic_trace_records(SemanticTraceRecordInputs {
        query_id: &query_id,
        documents: &documents,
        semantic_policy_state: semantic_projection.policy_state,
        semantic_capability_state: semantic_projection.capability_state,
        hybrid_outcome: semantic_projection.hybrid_outcome,
        retrieval_state,
        retrieval_index_state,
        execution_details: trace_execution_details,
        terminal_reason: terminal_reason.as_deref(),
        selected_evidence: &selected_evidence,
        rejected_candidates: &rejected_candidates,
    });
    let substrate_fields = build_context_substrate_projection_fields(
        workspace_ref,
        inputs,
        selected_targets,
        &documents,
        &selected_evidence,
        &rejected_candidates,
        retrieval_index_state,
    );

    let mut projection = AdvancedContextProjection {
        query_id: query_id.clone(),
        retrieval_mode: policy.retrieval_mode,
        retrieval_state,
        retrieval_index_state,
        semantic_policy_state: semantic_projection.policy_state,
        semantic_capability_state: semantic_projection.capability_state,
        hybrid_outcome: semantic_projection.hybrid_outcome,
        budgets: policy.budgets.clone(),
        remote_policy_state: policy.remote_policy,
        used_remote: false,
        terminal_reason,
        selected_evidence,
        rejected_candidates,
        semantic_trace_records,
        relationships,
        impact_findings,
        context_pack_entries: substrate_fields.context_pack_entries,
        omission_findings: substrate_fields.omission_findings,
        repository_map_state: substrate_fields.repository_map_state,
        snapshot_cache_state: substrate_fields.snapshot_cache_state,
        patch_safe_edit_attempts: substrate_fields.patch_safe_edit_attempts,
    };

    if projection.validate().is_err() {
        projection = terminal_projection(TerminalProjectionInputs {
            query_id,
            workspace_ref,
            inputs,
            selected_targets,
            policy,
            semantic_policy,
            retrieval_state: RetrievalState::Unavailable,
            retrieval_index_state: RetrievalIndexState::Stale,
            terminal_reason: "advanced retrieval projection validation failed after local indexing"
                .to_string(),
        });
    }

    projection
}

fn candidate_from_decision(
    candidate_id: String,
    decision: &CandidateDecision,
    document: &RetrievalDocument,
) -> RetrievedEvidenceCandidate {
    RetrievedEvidenceCandidate {
        candidate_id,
        source_kind: document.source_kind,
        source_ref: document.source_ref.clone(),
        authority_rank: document.authority_rank,
        match_origin: decision.match_origin,
        selection_state: decision.selection_state,
        selection_reason: decision.selection_reason.clone(),
        provenance_summary: document.provenance_summary.clone(),
        compatibility_state: document.compatibility_state,
        staleness_state: document.staleness_state,
        lexical_score: decision.lexical_score,
        semantic_score: decision.semantic_score,
        canon_semantic_contract_line: decision
            .canon_semantic_contract_line
            .clone()
            .or_else(|| document.canon_semantic_contract_line.clone()),
        canon_semantic_provenance_ref: decision
            .canon_semantic_provenance_ref
            .clone()
            .or_else(|| document.canon_semantic_provenance_ref.clone()),
    }
}

#[derive(Debug, Clone)]
struct IndexQueryResult {
    lexical_matches: Vec<RankedDocumentRef>,
    semantic_matches: Vec<SemanticMatchResult>,
    vector_extension_state: VectorExtensionState,
    extension_load_attempted: bool,
    vector_query_attempted: bool,
    vector_chunk_candidates_returned: usize,
}

#[derive(Debug, Clone)]
struct RankedDocumentRef {
    source_ref: String,
    lexical_score: Option<RetrievalScore>,
}

#[derive(Debug, Clone)]
struct SemanticMatchResult {
    source_ref: String,
    semantic_score: RetrievalScore,
    canon_semantic_contract_line: Option<String>,
    canon_semantic_provenance_ref: Option<String>,
}

#[derive(Debug, Clone)]
struct SemanticQueryResult {
    matches: Vec<SemanticMatchResult>,
    vector_query_attempted: bool,
    vector_chunk_candidates_returned: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct SemanticTraceExecutionDetails {
    extension_load_attempted: bool,
    vector_query_attempted: bool,
    vector_chunk_candidates_returned: usize,
}

#[derive(Debug, Clone)]
struct CandidateDecision {
    source_ref: String,
    match_origin: RetrievalMatchOrigin,
    selection_state: CandidateSelectionState,
    selection_reason: String,
    lexical_score: Option<RetrievalScore>,
    semantic_score: Option<RetrievalScore>,
    canon_semantic_contract_line: Option<String>,
    canon_semantic_provenance_ref: Option<String>,
}

#[derive(Debug, Clone)]
struct HybridSelectionResult {
    selected: Vec<CandidateDecision>,
    rejected: Vec<CandidateDecision>,
    semantic_projection: SemanticProjectionState,
    retrieval_state: RetrievalState,
}

#[derive(Debug, Clone)]
struct SemanticProjectionState {
    policy_state: SemanticPolicyState,
    capability_state: SemanticCapabilityState,
    hybrid_outcome: HybridOutcome,
    terminal_reason: Option<String>,
}

#[derive(Debug, Clone)]
enum SelectionStrategy {
    Fts,
    StructuredFallback,
}

impl SelectionStrategy {
    fn selection_reason(&self) -> &'static str {
        match self {
            Self::Fts => "matched SQLite FTS evidence for the bounded goal",
            Self::StructuredFallback => {
                "promoted bounded context evidence through structured fallback ordering"
            }
        }
    }

    fn retrieval_state(&self, credibility: ContextPackCredibility) -> RetrievalState {
        match (self, credibility) {
            (Self::Fts, ContextPackCredibility::Credible | ContextPackCredibility::Stale) => {
                RetrievalState::Selected
            }
            _ => RetrievalState::Degraded,
        }
    }
}

fn semantic_projection_for_local_result(
    semantic_policy: SemanticAccelerationPolicyState,
    vector_extension_state: VectorExtensionState,
) -> SemanticProjectionState {
    match semantic_policy {
        SemanticAccelerationPolicyState::Disabled => SemanticProjectionState {
            policy_state: SemanticPolicyState::Disabled,
            capability_state: SemanticCapabilityState::Unsupported,
            hybrid_outcome: HybridOutcome::BaselineOnly,
            terminal_reason: None,
        },
        SemanticAccelerationPolicyState::Local => {
            let (capability_state, terminal_reason) = match vector_extension_state {
                VectorExtensionState::Ready => (
                    SemanticCapabilityState::Ready,
                    Some(SEMANTIC_REFRESH_PENDING_REASON.to_string()),
                ),
                VectorExtensionState::Missing => (
                    SemanticCapabilityState::Unavailable,
                    Some(SEMANTIC_ACCELERATION_MISSING_REASON.to_string()),
                ),
                VectorExtensionState::Unsupported => (
                    SemanticCapabilityState::Unsupported,
                    Some(SEMANTIC_ACCELERATION_UNSUPPORTED_REASON.to_string()),
                ),
                VectorExtensionState::Stale => (
                    SemanticCapabilityState::Degraded,
                    Some(SEMANTIC_ACCELERATION_STALE_REASON.to_string()),
                ),
                VectorExtensionState::Degraded => (
                    SemanticCapabilityState::Degraded,
                    Some(SEMANTIC_ACCELERATION_DEGRADED_REASON.to_string()),
                ),
                VectorExtensionState::Corrupt => (
                    SemanticCapabilityState::Corrupt,
                    Some(SEMANTIC_ACCELERATION_CORRUPT_REASON.to_string()),
                ),
            };

            SemanticProjectionState {
                policy_state: SemanticPolicyState::Local,
                capability_state,
                hybrid_outcome: HybridOutcome::Skipped,
                terminal_reason,
            }
        }
    }
}

fn semantic_projection_for_local_hybrid_outcome(
    hybrid_outcome: HybridOutcome,
    terminal_reason: Option<String>,
) -> SemanticProjectionState {
    SemanticProjectionState {
        policy_state: SemanticPolicyState::Local,
        capability_state: SemanticCapabilityState::Ready,
        hybrid_outcome,
        terminal_reason,
    }
}

fn semantic_projection_for_local_error(
    semantic_policy: SemanticAccelerationPolicyState,
    error: &str,
) -> SemanticProjectionState {
    match semantic_policy {
        SemanticAccelerationPolicyState::Disabled => {
            semantic_projection_for_local_result(semantic_policy, VectorExtensionState::Unsupported)
        }
        SemanticAccelerationPolicyState::Local => SemanticProjectionState {
            policy_state: SemanticPolicyState::Local,
            capability_state: SemanticCapabilityState::Degraded,
            hybrid_outcome: HybridOutcome::Fallback,
            terminal_reason: Some(format!(
                "semantic acceleration degraded with local retrieval index error: {error}"
            )),
        },
    }
}

fn combine_terminal_reasons(
    base_reason: Option<String>,
    semantic_reason: Option<String>,
) -> Option<String> {
    match (base_reason, semantic_reason) {
        (Some(base), Some(semantic)) if base == semantic => Some(base),
        (Some(base), Some(semantic)) => Some(format!("{base}; {semantic}")),
        (Some(base), None) => Some(base),
        (None, Some(semantic)) => Some(semantic),
        (None, None) => None,
    }
}

#[derive(Debug, Clone)]
struct RetrievalDocument {
    source_ref: String,
    source_kind: RetrievalSourceKind,
    authority_rank: AuthorityRank,
    provenance_summary: String,
    compatibility_state: RetrievalCompatibilityState,
    compatibility_reason: Option<String>,
    staleness_state: RetrievalStalenessState,
    canon_artifact_class: Option<String>,
    canon_semantic_contract_line: Option<String>,
    canon_semantic_provenance_boundary: Option<CanonSemanticProvenanceBoundary>,
    canon_semantic_provenance_ref: Option<String>,
    canon_semantic_labels: Vec<String>,
    metadata_json: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct RetrievalDocumentMetadata {
    source_kind: RetrievalSourceKind,
    authority_rank: AuthorityRank,
    source: String,
    primary: bool,
    selected_target: bool,
    relative_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canon_artifact_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canon_semantic_contract_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility_reason: Option<String>,
}

#[derive(Debug, Default)]
struct ContextSubstrateProjectionFields {
    context_pack_entries: Vec<ContextPackEntryProjection>,
    omission_findings: Vec<ContextOmissionFinding>,
    repository_map_state: Option<RepositoryMapState>,
    snapshot_cache_state: Option<SnapshotCacheState>,
    patch_safe_edit_attempts: Vec<PatchSafeEditAttempt>,
}

#[derive(Debug, Clone)]
struct CanonSemanticCompatibility {
    artifact_class: Option<String>,
    semantic_contract_line: Option<String>,
    semantic_provenance_boundary: Option<CanonSemanticProvenanceBoundary>,
    semantic_provenance_ref: Option<String>,
    semantic_labels: Vec<String>,
    compatibility_state: RetrievalCompatibilityState,
    compatibility_reason: Option<String>,
}

#[derive(Debug, Error)]
enum ContextIntelligenceBuildError {
    #[error("failed to create advanced retrieval state directory: {0}")]
    CreateStateDirectory(String),
    #[error("failed to open advanced retrieval index: {0}")]
    OpenIndex(String),
    #[error("failed to read advanced retrieval manifest: {0}")]
    ReadManifest(String),
    #[error("failed to write advanced retrieval manifest: {0}")]
    WriteManifest(String),
    #[error("failed to initialize advanced retrieval index: {0}")]
    InitializeIndex(String),
    #[error("failed to initialize semantic retrieval scaffold: {0}")]
    InitializeSemanticIndex(String),
    #[error("failed to refresh advanced retrieval index: {0}")]
    RefreshIndex(String),
    #[error("failed to query advanced retrieval index: {0}")]
    QueryIndex(String),
    #[error("failed to serialize advanced retrieval metadata: {0}")]
    SerializeMetadata(String),
}

fn collect_retrieval_documents(
    workspace_ref: &Path,
    inputs: &[ContextInput],
    selected_targets: &[String],
    credibility: ContextPackCredibility,
    staleness_reason: Option<&str>,
    semantic_policy: SemanticAccelerationPolicyState,
) -> Vec<RetrievalDocument> {
    let mut seen = BTreeSet::new();
    let mut documents = Vec::new();

    for input in inputs {
        if !seen.insert(input.reference.clone()) {
            continue;
        }

        let source_kind = retrieval_source_kind(input.kind);
        let authority_rank = authority_rank(input.kind);
        let relative_path = resolved_relative_path(workspace_ref, &input.reference);
        let has_file_backing =
            relative_path.as_ref().map(|path| workspace_ref.join(path).is_file()).unwrap_or(false);
        let canon_semantic = canon_semantic_compatibility(
            workspace_ref,
            input,
            relative_path.as_deref(),
            has_file_backing,
            semantic_policy,
        );
        let compatibility_state = canon_semantic
            .as_ref()
            .map_or(default_compatibility_state(input.kind, has_file_backing), |metadata| {
                metadata.compatibility_state
            });
        let staleness_state = staleness_state(input.kind, credibility, staleness_reason);
        let metadata = RetrievalDocumentMetadata {
            source_kind,
            authority_rank,
            source: input.source.clone(),
            primary: input.primary,
            selected_target: selected_targets.iter().any(|target| target == &input.reference),
            relative_path,
            canon_artifact_class: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.artifact_class.clone()),
            canon_semantic_contract_line: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.semantic_contract_line.clone()),
            compatibility_reason: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.compatibility_reason.clone()),
        };
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|error| ContextIntelligenceBuildError::SerializeMetadata(error.to_string()));
        let Ok(metadata_json) = metadata_json else {
            continue;
        };

        documents.push(RetrievalDocument {
            source_ref: input.reference.clone(),
            source_kind,
            authority_rank,
            provenance_summary: format!(
                "{} via {} ({})",
                input.reference, input.source, input.rationale
            ),
            compatibility_state,
            compatibility_reason: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.compatibility_reason.clone()),
            staleness_state,
            canon_artifact_class: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.artifact_class.clone()),
            canon_semantic_contract_line: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.semantic_contract_line.clone()),
            canon_semantic_provenance_boundary: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.semantic_provenance_boundary),
            canon_semantic_provenance_ref: canon_semantic
                .as_ref()
                .and_then(|metadata| metadata.semantic_provenance_ref.clone()),
            canon_semantic_labels: canon_semantic
                .as_ref()
                .map(|metadata| metadata.semantic_labels.clone())
                .unwrap_or_default(),
            metadata_json,
            content: document_content(workspace_ref, input),
        });
    }

    documents
}

fn document_content(workspace_ref: &Path, input: &ContextInput) -> String {
    let mut content = format!("{}\n{}\n{}", input.reference, input.rationale, input.source);

    if let Some(relative_path) = resolved_relative_path(workspace_ref, &input.reference) {
        let absolute_path = workspace_ref.join(relative_path);
        if let Ok(bytes) = fs::read(absolute_path) {
            content.push('\n');
            content.push_str(&truncate_utf8_lossy(&bytes, MAX_INDEXED_BYTES));
        }
    }

    truncate_string(content, MAX_INDEXED_BYTES)
}

fn truncate_utf8_lossy(bytes: &[u8], max_bytes: usize) -> String {
    truncate_string(String::from_utf8_lossy(bytes).into_owned(), max_bytes)
}

fn truncate_string(value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }

    let mut truncated = String::new();
    let mut used_bytes = 0;
    for character in value.chars() {
        let character_bytes = character.len_utf8();
        if used_bytes + character_bytes > max_bytes {
            break;
        }
        truncated.push(character);
        used_bytes += character_bytes;
    }
    truncated
}

fn default_index_state(
    credibility: ContextPackCredibility,
    documents: &[RetrievalDocument],
) -> RetrievalIndexState {
    if documents.is_empty() || credibility == ContextPackCredibility::Insufficient {
        return RetrievalIndexState::Insufficient;
    }
    if credibility == ContextPackCredibility::Stale
        || documents
            .iter()
            .any(|document| document.staleness_state == RetrievalStalenessState::Stale)
    {
        return RetrievalIndexState::Stale;
    }
    RetrievalIndexState::Ready
}

fn credibility_degradation_reason(
    credibility: ContextPackCredibility,
    staleness_reason: Option<&str>,
) -> Option<String> {
    match credibility {
        ContextPackCredibility::Credible => None,
        ContextPackCredibility::Insufficient => {
            Some("bounded context remains insufficient after local retrieval".to_string())
        }
        ContextPackCredibility::Stale => Some(format!(
            "bounded context remains stale after local retrieval: {}",
            staleness_reason.unwrap_or("refresh evidence before execution")
        )),
    }
}

fn build_context_substrate_projection_fields(
    workspace_ref: &Path,
    inputs: &[ContextInput],
    selected_targets: &[String],
    documents: &[RetrievalDocument],
    selected_evidence: &[RetrievedEvidenceCandidate],
    rejected_candidates: &[RetrievedEvidenceCandidate],
    retrieval_index_state: RetrievalIndexState,
) -> ContextSubstrateProjectionFields {
    let document_map = documents
        .iter()
        .map(|document| (document.source_ref.as_str(), document))
        .collect::<BTreeMap<_, _>>();
    let critical_refs = critical_context_refs(inputs, selected_targets);
    let repository_map_state = Some(repository_map_state_for_workspace(
        workspace_ref,
        retrieval_index_state,
        selected_targets,
    ));
    let snapshot_cache_state =
        Some(snapshot_cache_state_for_workspace(workspace_ref, retrieval_index_state));

    let mut fields = ContextSubstrateProjectionFields {
        repository_map_state,
        snapshot_cache_state,
        ..ContextSubstrateProjectionFields::default()
    };

    if matches!(fields.repository_map_state, Some(RepositoryMapState::Missing)) {
        fields.omission_findings.push(ContextOmissionFinding {
            severity: ContextOmissionSeverity::Warning,
            reason_code: CONTEXT_REASON_REPOSITORY_MAP_UNAVAILABLE.to_string(),
            candidate_ref: PROJECT_INDEX_FILE.to_string(),
            message: "repository-map hints are unavailable; narrowing relies on bounded workspace evidence only".to_string(),
            required_fidelity: None,
            observed_mode: None,
        });
    }

    if matches!(fields.snapshot_cache_state, Some(SnapshotCacheState::Stale)) {
        fields.omission_findings.push(ContextOmissionFinding {
            severity: ContextOmissionSeverity::Warning,
            reason_code: CONTEXT_REASON_SNAPSHOT_CACHE_STALE.to_string(),
            candidate_ref: DERIVED_INDEX_IGNORE_PATTERN_MANIFEST.to_string(),
            message: "derived local context cache is stale and will not be trusted as authoritative planning context".to_string(),
            required_fidelity: None,
            observed_mode: None,
        });
    }

    if matches!(fields.snapshot_cache_state, Some(SnapshotCacheState::Tracked)) {
        fields.omission_findings.push(ContextOmissionFinding {
            severity: ContextOmissionSeverity::Blocking,
            reason_code: CONTEXT_REASON_TRACKED_CACHE_DETECTED.to_string(),
            candidate_ref: DERIVED_INDEX_IGNORE_PATTERN_DATABASE.to_string(),
            message: "tracked derived context cache artifacts were detected; repair ignore hygiene before trusting the local substrate".to_string(),
            required_fidelity: None,
            observed_mode: None,
        });
    }

    for candidate in selected_evidence {
        let document = document_map.get(candidate.source_ref.as_str()).copied();
        let fidelity_tier =
            classify_context_fidelity(candidate.source_ref.as_str(), &critical_refs);
        let inclusion_mode = selected_candidate_inclusion_mode(workspace_ref, candidate, document);
        let digest_ref =
            digest_ref_for_candidate(workspace_ref, candidate, document, inclusion_mode);
        let entry = ContextPackEntryProjection {
            source_ref: candidate.source_ref.clone(),
            source_kind: candidate.source_kind,
            authority_rank: candidate.authority_rank,
            fidelity_tier,
            inclusion_mode,
            reason: candidate.selection_reason.clone(),
            required_for_admission: fidelity_tier == ContextFidelityTier::Critical,
            resolved_excerpt_anchor: excerpt_anchor_for_candidate(
                candidate,
                document,
                inclusion_mode,
            ),
            digest_ref,
            lifecycle_relevance: Some(lifecycle_relevance_for_candidate(candidate).to_string()),
            risk_relevance: risk_relevance_for_candidate(candidate, document),
            ranking_rationale: Some(ranking_rationale_for_candidate(candidate)),
        };
        append_selected_candidate_findings(&mut fields.omission_findings, &entry, candidate);
        append_patch_safe_edit_attempt(
            &mut fields.patch_safe_edit_attempts,
            workspace_ref,
            &entry,
            document,
        );
        fields.context_pack_entries.push(entry);
    }

    for candidate in rejected_candidates {
        let document = document_map.get(candidate.source_ref.as_str()).copied();
        let fidelity_tier =
            classify_context_fidelity(candidate.source_ref.as_str(), &critical_refs);
        let entry = ContextPackEntryProjection {
            source_ref: candidate.source_ref.clone(),
            source_kind: candidate.source_kind,
            authority_rank: candidate.authority_rank,
            fidelity_tier,
            inclusion_mode: ContextInclusionMode::Omitted,
            reason: candidate.selection_reason.clone(),
            required_for_admission: fidelity_tier == ContextFidelityTier::Critical,
            resolved_excerpt_anchor: None,
            digest_ref: None,
            lifecycle_relevance: Some(lifecycle_relevance_for_candidate(candidate).to_string()),
            risk_relevance: risk_relevance_for_candidate(candidate, document),
            ranking_rationale: Some(ranking_rationale_for_candidate(candidate)),
        };
        fields.omission_findings.push(rejected_candidate_omission_finding(
            workspace_ref,
            candidate,
            &entry,
            document,
        ));
        fields.context_pack_entries.push(entry);
    }

    fields
}

fn critical_context_refs(inputs: &[ContextInput], selected_targets: &[String]) -> BTreeSet<String> {
    let mut refs = selected_targets.iter().cloned().collect::<BTreeSet<_>>();
    for input in inputs.iter().filter(|input| input.primary) {
        refs.insert(input.reference.clone());
    }
    refs
}

fn classify_context_fidelity(
    source_ref: &str,
    critical_refs: &BTreeSet<String>,
) -> ContextFidelityTier {
    if critical_refs.contains(source_ref) {
        ContextFidelityTier::Critical
    } else if is_archived_context_ref(source_ref) {
        ContextFidelityTier::Archived
    } else if source_ref.ends_with(".md")
        || source_ref.contains("/docs/")
        || source_ref.contains("/specs/")
    {
        ContextFidelityTier::Ambient
    } else {
        ContextFidelityTier::Supporting
    }
}

fn is_archived_context_ref(source_ref: &str) -> bool {
    source_ref.contains("/archive/")
        || source_ref.contains("/archives/")
        || source_ref.starts_with("archive/")
        || source_ref.starts_with("archives/")
        || source_ref.contains(".archive/")
}

fn selected_candidate_inclusion_mode(
    workspace_ref: &Path,
    candidate: &RetrievedEvidenceCandidate,
    document: Option<&RetrievalDocument>,
) -> ContextInclusionMode {
    if is_archived_context_ref(candidate.source_ref.as_str()) {
        return ContextInclusionMode::Omitted;
    }
    let Some(size_bytes) =
        source_ref_size_bytes(workspace_ref, candidate.source_ref.as_str(), document)
    else {
        return ContextInclusionMode::Full;
    };
    if size_bytes >= LARGE_CONTEXT_DIGEST_THRESHOLD_BYTES
        && should_compact_to_digest(candidate.source_ref.as_str())
    {
        ContextInclusionMode::Digest
    } else if size_bytes >= LARGE_CONTEXT_EXCERPT_THRESHOLD_BYTES {
        ContextInclusionMode::Excerpt
    } else {
        ContextInclusionMode::Full
    }
}

fn source_ref_size_bytes(
    workspace_ref: &Path,
    source_ref: &str,
    document: Option<&RetrievalDocument>,
) -> Option<u64> {
    let path = workspace_ref.join(source_ref);
    fs::metadata(&path)
        .ok()
        .map(|metadata| metadata.len())
        .or_else(|| document.map(|document| document.content.len() as u64))
}

fn should_compact_to_digest(source_ref: &str) -> bool {
    source_ref.ends_with(".log")
        || source_ref.ends_with(".diff")
        || source_ref.ends_with(".patch")
        || source_ref.contains("/logs/")
        || source_ref.contains("/generated/")
        || source_ref.contains("/dist/")
        || source_ref.contains("/coverage/")
}

fn digest_ref_for_candidate(
    _workspace_ref: &Path,
    candidate: &RetrievedEvidenceCandidate,
    document: Option<&RetrievalDocument>,
    inclusion_mode: ContextInclusionMode,
) -> Option<DigestBackedArtifactRef> {
    if inclusion_mode != ContextInclusionMode::Digest {
        return None;
    }
    let summary = document
        .map(|document| summarize_digest_content(document.content.as_str()))
        .unwrap_or_else(|| candidate.provenance_summary.clone());
    let digest_source =
        document.map(|document| document.content.as_str()).unwrap_or(candidate.source_ref.as_str());
    Some(DigestBackedArtifactRef {
        digest: semantic_content_hash(digest_source),
        artifact_kind: digest_artifact_kind(candidate.source_ref.as_str()).to_string(),
        summary,
        excerpt_anchor: Some(candidate.source_ref.clone()),
        resolve_path: candidate.source_ref.clone(),
    })
}

fn summarize_digest_content(content: &str) -> String {
    let summary = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(" | ");
    if summary.is_empty() {
        "large artifact compacted to a digest-backed reference".to_string()
    } else {
        summary
    }
}

fn digest_artifact_kind(source_ref: &str) -> &'static str {
    if source_ref.ends_with(".log") || source_ref.contains("/logs/") {
        "log"
    } else if source_ref.ends_with(".diff") || source_ref.ends_with(".patch") {
        "diff"
    } else if source_ref.contains("/generated/") || source_ref.contains("/dist/") {
        "generated_output"
    } else {
        "workspace_artifact"
    }
}

fn excerpt_anchor_for_candidate(
    candidate: &RetrievedEvidenceCandidate,
    _document: Option<&RetrievalDocument>,
    inclusion_mode: ContextInclusionMode,
) -> Option<String> {
    match inclusion_mode {
        ContextInclusionMode::Excerpt => Some(format!("{}#bounded-excerpt", candidate.source_ref)),
        ContextInclusionMode::Digest => Some(format!("{}#digest-summary", candidate.source_ref)),
        _ => None,
    }
}

fn lifecycle_relevance_for_candidate(candidate: &RetrievedEvidenceCandidate) -> &'static str {
    match candidate.source_kind {
        RetrievalSourceKind::WorkspaceFile => "implementation_surface",
        RetrievalSourceKind::ProjectMemory => "project_memory",
        RetrievalSourceKind::Trace => "recent_trace",
        RetrievalSourceKind::ReviewFinding => "review_feedback",
        RetrievalSourceKind::VerificationEvidence => "verification_surface",
        RetrievalSourceKind::CanonArtifact => "governed_artifact",
    }
}

fn risk_relevance_for_candidate(
    candidate: &RetrievedEvidenceCandidate,
    document: Option<&RetrievalDocument>,
) -> Option<String> {
    if candidate.source_kind == RetrievalSourceKind::VerificationEvidence {
        return Some("verification_evidence".to_string());
    }
    document
        .filter(|document| document.content.to_ascii_lowercase().contains("risk"))
        .map(|_| "risk_signal".to_string())
}

fn ranking_rationale_for_candidate(candidate: &RetrievedEvidenceCandidate) -> String {
    let mut parts = vec![format!("origin={}", candidate.match_origin.as_str())];
    if let Some(lexical_score) = candidate.lexical_score {
        parts.push(format!("lexical={:.3}", lexical_score.as_raw()));
    }
    if let Some(semantic_score) = candidate.semantic_score {
        parts.push(format!("semantic={:.3}", semantic_score.as_raw()));
    }
    parts.push(format!("authority={}", candidate.authority_rank.as_str()));
    parts.join(", ")
}

fn append_selected_candidate_findings(
    omission_findings: &mut Vec<ContextOmissionFinding>,
    entry: &ContextPackEntryProjection,
    candidate: &RetrievedEvidenceCandidate,
) {
    if entry.inclusion_mode == ContextInclusionMode::Digest {
        omission_findings.push(ContextOmissionFinding {
            severity: if entry.required_for_admission {
                ContextOmissionSeverity::Blocking
            } else {
                ContextOmissionSeverity::Info
            },
            reason_code: if entry.required_for_admission {
                CONTEXT_REASON_CRITICAL_DOWNGRADED.to_string()
            } else {
                CONTEXT_REASON_ARTIFACT_COMPACTED_TO_DIGEST.to_string()
            },
            candidate_ref: candidate.source_ref.clone(),
            message: if entry.required_for_admission {
                "critical context was compacted to a digest-backed reference and no longer satisfies the required fidelity".to_string()
            } else {
                "large artifact was compacted to a digest-backed reference for bounded context selection".to_string()
            },
            required_fidelity: entry.required_for_admission.then_some(ContextFidelityTier::Critical),
            observed_mode: Some(entry.inclusion_mode),
        });
    }

    if entry.inclusion_mode == ContextInclusionMode::Excerpt && entry.required_for_admission {
        omission_findings.push(ContextOmissionFinding {
            severity: ContextOmissionSeverity::Info,
            reason_code: CONTEXT_REASON_SEARCH_REQUIRED_BEFORE_READ.to_string(),
            candidate_ref: candidate.source_ref.clone(),
            message:
                "critical context was narrowed to a bounded excerpt instead of a full-file read"
                    .to_string(),
            required_fidelity: Some(ContextFidelityTier::Critical),
            observed_mode: Some(entry.inclusion_mode),
        });
    }

    if entry.inclusion_mode == ContextInclusionMode::Omitted {
        omission_findings.push(ContextOmissionFinding {
            severity: if entry.required_for_admission {
                ContextOmissionSeverity::Blocking
            } else {
                ContextOmissionSeverity::Warning
            },
            reason_code: if entry.required_for_admission {
                CONTEXT_REASON_CRITICAL_UNAVAILABLE.to_string()
            } else {
                CONTEXT_REASON_ARCHIVED_CONTEXT_EXCLUDED.to_string()
            },
            candidate_ref: candidate.source_ref.clone(),
            message: if entry.required_for_admission {
                "critical context could not be admitted safely at the required fidelity".to_string()
            } else {
                "archived context was excluded from the active bounded context pack".to_string()
            },
            required_fidelity: entry
                .required_for_admission
                .then_some(ContextFidelityTier::Critical),
            observed_mode: Some(entry.inclusion_mode),
        });
    }
}

fn append_patch_safe_edit_attempt(
    attempts: &mut Vec<PatchSafeEditAttempt>,
    workspace_ref: &Path,
    entry: &ContextPackEntryProjection,
    document: Option<&RetrievalDocument>,
) {
    if entry.source_kind != RetrievalSourceKind::WorkspaceFile {
        return;
    }
    let Some(size_bytes) =
        source_ref_size_bytes(workspace_ref, entry.source_ref.as_str(), document)
    else {
        return;
    };
    if size_bytes < LARGE_CONTEXT_EXCERPT_THRESHOLD_BYTES {
        return;
    }
    let digest_source =
        document.map(|document| document.content.as_str()).unwrap_or(entry.source_ref.as_str());
    attempts.push(PatchSafeEditAttempt {
        target_ref: entry.source_ref.clone(),
        anchor_refs: vec![
            format!("{}#start-anchor", entry.source_ref),
            format!("{}#end-anchor", entry.source_ref),
        ],
        pre_apply_digest: semantic_content_hash(digest_source),
        post_apply_verification: vec![
            "cargo fmt --check".to_string(),
            "targeted validation required after bounded patch application".to_string(),
        ],
        result_state: PatchSafeEditResultState::ManualReviewRequired,
    });
}

fn rejected_candidate_omission_finding(
    workspace_ref: &Path,
    candidate: &RetrievedEvidenceCandidate,
    entry: &ContextPackEntryProjection,
    document: Option<&RetrievalDocument>,
) -> ContextOmissionFinding {
    let size_bytes =
        source_ref_size_bytes(workspace_ref, candidate.source_ref.as_str(), document).unwrap_or(0);
    if entry.required_for_admission && size_bytes >= LARGE_CONTEXT_EXCERPT_THRESHOLD_BYTES {
        return ContextOmissionFinding {
            severity: ContextOmissionSeverity::Blocking,
            reason_code: CONTEXT_REASON_UNSAFE_FULL_READ_REFUSED.to_string(),
            candidate_ref: candidate.source_ref.clone(),
            message: "critical oversized context was refused because no safe bounded full-read path was available".to_string(),
            required_fidelity: Some(ContextFidelityTier::Critical),
            observed_mode: Some(ContextInclusionMode::Omitted),
        };
    }

    ContextOmissionFinding {
        severity: if entry.required_for_admission {
            ContextOmissionSeverity::Blocking
        } else {
            ContextOmissionSeverity::Warning
        },
        reason_code: if entry.required_for_admission {
            CONTEXT_REASON_CRITICAL_UNAVAILABLE.to_string()
        } else {
            CONTEXT_REASON_SEARCH_REQUIRED_BEFORE_READ.to_string()
        },
        candidate_ref: candidate.source_ref.clone(),
        message: if entry.required_for_admission {
            "critical context could not be selected from the bounded repository scan".to_string()
        } else {
            "candidate remained omitted because bounded search signals were stronger elsewhere"
                .to_string()
        },
        required_fidelity: entry.required_for_admission.then_some(ContextFidelityTier::Critical),
        observed_mode: Some(ContextInclusionMode::Omitted),
    }
}

fn repository_map_state_for_workspace(
    workspace_ref: &Path,
    retrieval_index_state: RetrievalIndexState,
    selected_targets: &[String],
) -> RepositoryMapState {
    match retrieval_index_state {
        RetrievalIndexState::Stale => RepositoryMapState::Stale,
        RetrievalIndexState::Degraded
        | RetrievalIndexState::Building
        | RetrievalIndexState::Insufficient
        | RetrievalIndexState::SemanticUnavailable => RepositoryMapState::Degraded,
        RetrievalIndexState::Corrupt | RetrievalIndexState::Incompatible => {
            RepositoryMapState::Corrupt
        }
        RetrievalIndexState::Missing => {
            if ProjectIndex::load(workspace_ref).ok().flatten().is_some()
                || !selected_targets.is_empty()
            {
                RepositoryMapState::Ready
            } else {
                RepositoryMapState::Missing
            }
        }
        RetrievalIndexState::Ready => RepositoryMapState::Ready,
    }
}

fn snapshot_cache_state_for_workspace(
    workspace_ref: &Path,
    retrieval_index_state: RetrievalIndexState,
) -> SnapshotCacheState {
    if !tracked_index_files(workspace_ref).is_empty() {
        return SnapshotCacheState::Tracked;
    }
    match retrieval_index_state {
        RetrievalIndexState::Ready => SnapshotCacheState::Ready,
        RetrievalIndexState::Stale => SnapshotCacheState::Stale,
        RetrievalIndexState::Missing => SnapshotCacheState::Missing,
        RetrievalIndexState::Corrupt | RetrievalIndexState::Incompatible => {
            SnapshotCacheState::Corrupt
        }
        RetrievalIndexState::Building
        | RetrievalIndexState::Insufficient
        | RetrievalIndexState::Degraded
        | RetrievalIndexState::SemanticUnavailable => SnapshotCacheState::Degraded,
    }
}

struct TerminalProjectionInputs<'a> {
    query_id: String,
    workspace_ref: &'a Path,
    inputs: &'a [ContextInput],
    selected_targets: &'a [String],
    policy: &'a AdvancedContextConfig,
    semantic_policy: SemanticAccelerationPolicyState,
    retrieval_state: RetrievalState,
    retrieval_index_state: RetrievalIndexState,
    terminal_reason: String,
}

fn terminal_projection(inputs: TerminalProjectionInputs<'_>) -> AdvancedContextProjection {
    let TerminalProjectionInputs {
        query_id,
        workspace_ref,
        inputs,
        selected_targets,
        policy,
        semantic_policy,
        retrieval_state,
        retrieval_index_state,
        terminal_reason,
    } = inputs;

    let semantic_projection = match semantic_policy {
        SemanticAccelerationPolicyState::Disabled => SemanticProjectionState {
            policy_state: SemanticPolicyState::Disabled,
            capability_state: SemanticCapabilityState::Unsupported,
            hybrid_outcome: HybridOutcome::BaselineOnly,
            terminal_reason: None,
        },
        SemanticAccelerationPolicyState::Local => SemanticProjectionState {
            policy_state: SemanticPolicyState::Local,
            capability_state: SemanticCapabilityState::Unsupported,
            hybrid_outcome: HybridOutcome::Skipped,
            terminal_reason: Some(SEMANTIC_ACCELERATION_SKIPPED_REASON.to_string()),
        },
    };

    let terminal_reason =
        combine_terminal_reasons(Some(terminal_reason), semantic_projection.terminal_reason);
    let semantic_trace_records = build_semantic_trace_records(SemanticTraceRecordInputs {
        query_id: &query_id,
        documents: &[],
        semantic_policy_state: semantic_projection.policy_state,
        semantic_capability_state: semantic_projection.capability_state,
        hybrid_outcome: semantic_projection.hybrid_outcome,
        retrieval_state,
        retrieval_index_state,
        execution_details: SemanticTraceExecutionDetails::default(),
        terminal_reason: terminal_reason.as_deref(),
        selected_evidence: &[],
        rejected_candidates: &[],
    });
    let substrate_fields = build_context_substrate_projection_fields(
        workspace_ref,
        inputs,
        selected_targets,
        &[],
        &[],
        &[],
        retrieval_index_state,
    );

    AdvancedContextProjection {
        query_id,
        retrieval_mode: policy.retrieval_mode,
        retrieval_state,
        retrieval_index_state,
        semantic_policy_state: semantic_projection.policy_state,
        semantic_capability_state: semantic_projection.capability_state,
        hybrid_outcome: semantic_projection.hybrid_outcome,
        budgets: policy.budgets.clone(),
        remote_policy_state: policy.remote_policy,
        used_remote: false,
        terminal_reason,
        selected_evidence: Vec::new(),
        rejected_candidates: Vec::new(),
        semantic_trace_records,
        relationships: Vec::new(),
        impact_findings: Vec::new(),
        context_pack_entries: substrate_fields.context_pack_entries,
        omission_findings: substrate_fields.omission_findings,
        repository_map_state: substrate_fields.repository_map_state,
        snapshot_cache_state: substrate_fields.snapshot_cache_state,
        patch_safe_edit_attempts: substrate_fields.patch_safe_edit_attempts,
    }
}

fn refresh_and_query_index(
    workspace_ref: &Path,
    goal_text: &str,
    selected_targets: &[String],
    documents: &[RetrievalDocument],
    evidence_limit: usize,
    expansion_limit: usize,
    vector_extension_state_override: Option<VectorExtensionState>,
) -> Result<IndexQueryResult, ContextIntelligenceBuildError> {
    let connection = open_connection(workspace_ref)?;
    initialize_schema(&connection, workspace_ref)?;
    refresh_documents(&connection, documents)?;
    let vector_extension_state = vector_extension_state_override
        .unwrap_or_else(|| detect_vector_extension_state(&connection));
    refresh_semantic_chunks(&connection, documents, vector_extension_state)?;
    let refresh_reason = IndexRefreshReason::ManualRefresh;
    persist_derived_index_manifest(
        &connection,
        workspace_ref,
        documents,
        vector_extension_state,
        refresh_reason,
    )?;

    let query = build_fts_query(goal_text, selected_targets);
    let vector_state = vector_extension_state;
    let semantic_query_result = query_semantic_matches(
        &connection,
        goal_text,
        selected_targets,
        expansion_limit,
        vector_state,
    )?;
    if query.is_empty() {
        return Ok(IndexQueryResult {
            lexical_matches: Vec::new(),
            semantic_matches: semantic_query_result.matches,
            vector_extension_state,
            extension_load_attempted: true,
            vector_query_attempted: semantic_query_result.vector_query_attempted,
            vector_chunk_candidates_returned: semantic_query_result
                .vector_chunk_candidates_returned,
        });
    }

    let mut statement = connection
        .prepare(
            "SELECT documents.source_ref,
                    bm25(retrieval_documents_fts) AS lexical_rank
             FROM retrieval_documents_fts
             INNER JOIN retrieval_documents AS documents
                 ON documents.source_ref = retrieval_documents_fts.source_ref
             WHERE retrieval_documents_fts MATCH ?1
             ORDER BY
                 CASE documents.authority_rank
                     WHEN 'structured' THEN 0
                     WHEN 'canon' THEN 1
                     WHEN 'workspace_override' THEN 2
                     ELSE 3
                 END,
                 bm25(retrieval_documents_fts)
             LIMIT ?2",
        )
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    let rows = statement
        .query_map(params![query, evidence_limit as i64], |row| {
            Ok(RankedDocumentRef {
                source_ref: row.get(0)?,
                lexical_score: lexical_score_from_bm25(row.get::<_, f64>(1)?),
            })
        })
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;

    let mut lexical_matches = Vec::new();
    for row in rows {
        lexical_matches.push(
            row.map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?,
        );
    }
    Ok(IndexQueryResult {
        lexical_matches,
        semantic_matches: semantic_query_result.matches,
        vector_extension_state,
        extension_load_attempted: true,
        vector_query_attempted: semantic_query_result.vector_query_attempted,
        vector_chunk_candidates_returned: semantic_query_result.vector_chunk_candidates_returned,
    })
}

#[cfg(feature = "sqlite-vec")]
fn register_sqlite_vec_auto_extension() -> Result<(), ContextIntelligenceBuildError> {
    let registration = SQLITE_VEC_AUTO_EXTENSION_REGISTRATION.get_or_init(|| {
        let init: SqliteVecAutoExtensionInit = unsafe {
            std::mem::transmute::<*const (), SqliteVecAutoExtensionInit>(
                sqlite_vec::sqlite3_vec_init as *const (),
            )
        };
        let rc = unsafe { rusqlite::ffi::sqlite3_auto_extension(Some(init)) };
        sqlite_auto_extension_result(rc)
    });

    registration
        .as_ref()
        .map_err(|error| ContextIntelligenceBuildError::InitializeSemanticIndex(error.clone()))
        .copied()
}

#[cfg(not(feature = "sqlite-vec"))]
fn register_sqlite_vec_auto_extension() -> Result<(), ContextIntelligenceBuildError> {
    Ok(())
}

#[cfg(feature = "sqlite-vec")]
fn sqlite_auto_extension_result(rc: std::ffi::c_int) -> Result<(), String> {
    if rc == rusqlite::ffi::SQLITE_OK {
        Ok(())
    } else {
        Err(format!("sqlite3_auto_extension returned {rc}"))
    }
}

fn open_connection(workspace_ref: &Path) -> Result<Connection, ContextIntelligenceBuildError> {
    register_sqlite_vec_auto_extension()?;
    let state_directory = context_intelligence_state_directory(workspace_ref);
    fs::create_dir_all(&state_directory)
        .map_err(|error| ContextIntelligenceBuildError::CreateStateDirectory(error.to_string()))?;
    let index_path = retrieval_index_path(workspace_ref);
    Connection::open(index_path)
        .map_err(|error| ContextIntelligenceBuildError::OpenIndex(error.to_string()))
}

fn context_intelligence_state_directory(workspace_ref: &Path) -> PathBuf {
    workspace_ref.join(BOUNDLINE_STATE_DIRECTORY).join(CONTEXT_INTELLIGENCE_DIRECTORY)
}

fn retrieval_index_path(workspace_ref: &Path) -> PathBuf {
    context_intelligence_state_directory(workspace_ref).join(RETRIEVAL_INDEX_FILE_NAME)
}

fn retrieval_index_manifest_path(workspace_ref: &Path) -> PathBuf {
    context_intelligence_state_directory(workspace_ref).join(RETRIEVAL_INDEX_MANIFEST_FILE_NAME)
}

fn initialize_schema(
    connection: &Connection,
    workspace_ref: &Path,
) -> Result<(), ContextIntelligenceBuildError> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS retrieval_documents (
                source_ref TEXT PRIMARY KEY,
                source_kind TEXT NOT NULL,
                authority_rank TEXT NOT NULL,
                provenance_summary TEXT NOT NULL,
                compatibility_state TEXT NOT NULL,
                staleness_state TEXT NOT NULL,
                metadata_json TEXT NOT NULL,
                content TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS retrieval_documents_fts
            USING fts5(source_ref UNINDEXED, content, tokenize='unicode61');
            CREATE INDEX IF NOT EXISTS retrieval_documents_source_kind_idx
                ON retrieval_documents(json_extract(metadata_json, '$.source_kind'));
            CREATE INDEX IF NOT EXISTS retrieval_documents_authority_rank_idx
                ON retrieval_documents(json_extract(metadata_json, '$.authority_rank'));",
        )
        .map_err(|error| ContextIntelligenceBuildError::InitializeIndex(error.to_string()))?;

    initialize_semantic_schema(connection, workspace_ref)
}

fn initialize_semantic_schema(
    connection: &Connection,
    workspace_ref: &Path,
) -> Result<(), ContextIntelligenceBuildError> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS semantic_index_manifest (
                manifest_id TEXT PRIMARY KEY,
                workspace_root TEXT NOT NULL,
                vector_extension_state TEXT NOT NULL,
                last_semantic_refresh_reason TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS semantic_chunks (
                chunk_id TEXT PRIMARY KEY,
                source_kind TEXT NOT NULL,
                source_ref TEXT NOT NULL,
                provenance_boundary TEXT NOT NULL,
                provenance_ref TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                embedding_state TEXT NOT NULL,
                embedding_dimensions INTEGER NOT NULL,
                canon_semantic_contract_line TEXT,
                semantic_labels_json TEXT NOT NULL DEFAULT '[]',
                semantic_schema_line TEXT NOT NULL DEFAULT 'boundline.semantic_chunk.v1',
                chunk_text TEXT NOT NULL DEFAULT '',
                embedding_payload_json TEXT NOT NULL DEFAULT '[]'
            );
            CREATE INDEX IF NOT EXISTS semantic_chunks_source_ref_idx
                ON semantic_chunks(source_ref);
            CREATE INDEX IF NOT EXISTS semantic_chunks_embedding_state_idx
                ON semantic_chunks(embedding_state);",
        )
        .map_err(|error| {
            ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string())
        })?;

    ensure_semantic_column(
        connection,
        "semantic_schema_line",
        "TEXT NOT NULL DEFAULT 'boundline.semantic_chunk.v1'",
    )?;
    ensure_semantic_column(connection, "chunk_text", "TEXT NOT NULL DEFAULT ''")?;
    ensure_semantic_column(connection, "embedding_payload_json", "TEXT NOT NULL DEFAULT '[]'")?;
    initialize_semantic_vector_table(connection)?;

    let vector_state = detect_vector_extension_state(connection);
    let manifest = build_derived_index_manifest(
        workspace_ref,
        &[],
        vector_state,
        Some(IndexRefreshReason::ManualRefresh),
        None,
    );
    upsert_semantic_manifest_row(connection, &manifest, SEMANTIC_REFRESH_PENDING_REASON)?;

    Ok(())
}

fn initialize_semantic_vector_table(
    connection: &Connection,
) -> Result<(), ContextIntelligenceBuildError> {
    initialize_semantic_vector_table_for_state(
        connection,
        detect_vector_extension_state(connection),
    )
}

fn initialize_semantic_vector_table_for_state(
    connection: &Connection,
    vector_extension_state: VectorExtensionState,
) -> Result<(), ContextIntelligenceBuildError> {
    if vector_extension_state != VectorExtensionState::Ready {
        return Ok(());
    }

    connection
        .execute_batch(&format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {SEMANTIC_VECTORS_TABLE_NAME} USING vec0({SEMANTIC_VECTOR_TABLE_DEFINITION});"
        ))
        .map_err(|error| ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string()))
}

fn semantic_vector_table_exists(
    connection: &Connection,
) -> Result<bool, ContextIntelligenceBuildError> {
    let mut statement = connection
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    let mut rows = statement
        .query(params![SEMANTIC_VECTORS_TABLE_NAME])
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    rows.next()
        .map(|row| row.is_some())
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))
}

fn ensure_semantic_column(
    connection: &Connection,
    column_name: &str,
    column_definition: &str,
) -> Result<(), ContextIntelligenceBuildError> {
    let mut statement =
        connection.prepare("PRAGMA table_info(semantic_chunks)").map_err(|error| {
            ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string())
        })?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1)).map_err(|error| {
        ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string())
    })?;
    let columns = rows.filter_map(Result::ok).collect::<BTreeSet<_>>();
    if columns.contains(column_name) {
        return Ok(());
    }

    connection
        .execute(
            &format!("ALTER TABLE semantic_chunks ADD COLUMN {column_name} {column_definition}"),
            [],
        )
        .map_err(|error| {
            ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string())
        })?;

    Ok(())
}

fn detect_vector_extension_state(connection: &Connection) -> VectorExtensionState {
    let mut statement = match connection.prepare("PRAGMA module_list") {
        Ok(statement) => statement,
        Err(_) => return VectorExtensionState::Unsupported,
    };
    let rows = match statement.query_map([], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows,
        Err(_) => return VectorExtensionState::Unsupported,
    };

    let modules = rows.filter_map(Result::ok).collect::<Vec<_>>();
    vector_extension_state_from_modules(&modules)
}

fn vector_extension_state_override_from_env() -> Option<VectorExtensionState> {
    #[cfg(debug_assertions)]
    {
        let value = std::env::var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV).ok()?;
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            SEMANTIC_VECTOR_STATE_READY_VALUE => Some(VectorExtensionState::Ready),
            SEMANTIC_VECTOR_STATE_MISSING_VALUE => Some(VectorExtensionState::Missing),
            SEMANTIC_VECTOR_STATE_STALE_VALUE => Some(VectorExtensionState::Stale),
            SEMANTIC_VECTOR_STATE_DEGRADED_VALUE => Some(VectorExtensionState::Degraded),
            SEMANTIC_VECTOR_STATE_CORRUPT_VALUE => Some(VectorExtensionState::Corrupt),
            SEMANTIC_VECTOR_STATE_UNSUPPORTED_VALUE => Some(VectorExtensionState::Unsupported),
            _ => None,
        }
    }

    #[cfg(not(debug_assertions))]
    {
        None
    }
}

fn vector_extension_state_from_modules(modules: &[String]) -> VectorExtensionState {
    let has_vec0 = modules.iter().any(|module| module == SQLITE_VEC_MODULE_NAME);
    let has_vec_each = modules.iter().any(|module| module == SQLITE_VEC_EACH_MODULE_NAME);
    match (has_vec0, has_vec_each) {
        (true, true) => VectorExtensionState::Ready,
        (false, false) => VectorExtensionState::Missing,
        _ => VectorExtensionState::Degraded,
    }
}

fn persist_derived_index_manifest(
    connection: &Connection,
    workspace_ref: &Path,
    documents: &[RetrievalDocument],
    vector_extension_state: VectorExtensionState,
    refresh_reason: IndexRefreshReason,
) -> Result<DerivedIndexManifest, ContextIntelligenceBuildError> {
    let previous_manifest = read_derived_index_manifest(workspace_ref).ok().flatten();
    let manifest = build_derived_index_manifest(
        workspace_ref,
        documents,
        vector_extension_state,
        Some(refresh_reason),
        previous_manifest.as_ref(),
    );
    write_derived_index_manifest(workspace_ref, &manifest)?;
    let manifest_reason = manifest
        .last_refresh_reason
        .map(IndexRefreshReason::as_str)
        .unwrap_or(SEMANTIC_REFRESH_PENDING_REASON);
    upsert_semantic_manifest_row(connection, &manifest, manifest_reason)?;
    Ok(manifest)
}

fn write_derived_index_manifest(
    workspace_ref: &Path,
    manifest: &DerivedIndexManifest,
) -> Result<(), ContextIntelligenceBuildError> {
    manifest
        .validate()
        .map_err(|error| ContextIntelligenceBuildError::WriteManifest(error.to_string()))?;
    let manifest_json = serde_json::to_string_pretty(manifest)
        .map_err(|error| ContextIntelligenceBuildError::WriteManifest(error.to_string()))?;
    let manifest_path = retrieval_index_manifest_path(workspace_ref);
    fs::write(&manifest_path, manifest_json)
        .map_err(|error| ContextIntelligenceBuildError::WriteManifest(error.to_string()))
}

fn read_derived_index_manifest(
    workspace_ref: &Path,
) -> Result<Option<DerivedIndexManifest>, ContextIntelligenceBuildError> {
    let manifest_path = retrieval_index_manifest_path(workspace_ref);
    let manifest_json = match fs::read_to_string(manifest_path) {
        Ok(value) => value,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(ContextIntelligenceBuildError::ReadManifest(error.to_string()));
        }
    };
    let manifest = serde_json::from_str::<DerivedIndexManifest>(&manifest_json)
        .map_err(|error| ContextIntelligenceBuildError::ReadManifest(error.to_string()))?;
    manifest
        .validate()
        .map_err(|error| ContextIntelligenceBuildError::ReadManifest(error.to_string()))?;
    Ok(Some(manifest))
}

fn recommended_action_for_index_state(
    index_state: RetrievalIndexState,
    workspace_ref: &Path,
) -> String {
    let workspace = workspace_ref.to_string_lossy();
    match index_state {
        RetrievalIndexState::Ready => "none".to_string(),
        RetrievalIndexState::Missing | RetrievalIndexState::Stale => {
            format!("boundline index refresh --workspace {workspace}")
        }
        RetrievalIndexState::Incompatible | RetrievalIndexState::Corrupt => {
            format!("boundline index rebuild --workspace {workspace}")
        }
        RetrievalIndexState::Degraded | RetrievalIndexState::SemanticUnavailable => {
            format!("boundline index doctor --workspace {workspace}")
        }
        RetrievalIndexState::Building | RetrievalIndexState::Insufficient => {
            format!("boundline index status --workspace {workspace}")
        }
    }
}

fn index_stale_warning(reason: IndexStaleReason) -> &'static str {
    match reason {
        IndexStaleReason::GitHeadChanged => {
            "git HEAD changed since the last successful derived-index refresh"
        }
        IndexStaleReason::BranchCheckout => "a branch checkout marked the derived index stale",
        IndexStaleReason::Merge => "a merge marked the derived index stale",
        IndexStaleReason::PullWithMerge => "a pull-with-merge marked the derived index stale",
        IndexStaleReason::Rebase => "a rebase marked the derived index stale",
        IndexStaleReason::PostRewrite => "a post-rewrite event marked the derived index stale",
        IndexStaleReason::HookMarkedStale => "a Git freshness hook marked the derived index stale",
    }
}

fn build_derived_index_manifest(
    workspace_ref: &Path,
    documents: &[RetrievalDocument],
    vector_extension_state: VectorExtensionState,
    refresh_reason: Option<IndexRefreshReason>,
    previous_manifest: Option<&DerivedIndexManifest>,
) -> DerivedIndexManifest {
    let git_state = current_git_state(workspace_ref);
    let mut manifest = DerivedIndexManifest {
        schema_version: RETRIEVAL_INDEX_SCHEMA_VERSION.to_string(),
        workspace_root: workspace_ref.to_string_lossy().into_owned(),
        git_branch: git_state
            .as_ref()
            .map(|state| state.branch.clone())
            .or_else(|| previous_manifest.and_then(|manifest| manifest.git_branch.clone())),
        git_head: git_state
            .as_ref()
            .map(|state| state.head.clone())
            .or_else(|| previous_manifest.and_then(|manifest| manifest.git_head.clone())),
        last_seen_head: git_state
            .as_ref()
            .map(|state| state.head.clone())
            .or_else(|| previous_manifest.and_then(|manifest| manifest.last_seen_head.clone())),
        index_status: RetrievalIndexState::Missing,
        last_refresh_at: Some(format_audit_timestamp(current_timestamp_millis())),
        last_refresh_reason: refresh_reason,
        stale_reason: None,
        file_count: documents.len(),
        chunk_count: documents
            .iter()
            .filter(|document| semantic_eligible_document(document))
            .count(),
        fts5_state: ManifestFtsState::Ready,
        sqlite_vec_state: vector_extension_state,
        semantic_engine: semantic_engine_for_vector_state(vector_extension_state),
        workspace_fingerprint: workspace_fingerprint(documents),
        config_fingerprint: derived_index_config_fingerprint(),
        chunker_fingerprint: semantic_chunker_fingerprint(),
        embedding_model_fingerprint: semantic_embedding_fingerprint(),
    };
    manifest.index_status =
        derive_manifest_index_status(previous_manifest, &manifest, workspace_ref);
    manifest
}

fn observe_manifest_git_freshness(
    workspace_ref: &Path,
    manifest: DerivedIndexManifest,
) -> Result<DerivedIndexManifest, ContextIntelligenceBuildError> {
    let Some(git_state) = current_git_state(workspace_ref) else {
        return Ok(manifest);
    };

    let mut updated = manifest.clone();
    updated.git_branch = Some(git_state.branch);
    updated.last_seen_head = Some(git_state.head);
    if let Some(reason) = hook_trigger_stale_reason_from_env() {
        updated.stale_reason = Some(reason);
    }

    if updated != manifest {
        write_derived_index_manifest(workspace_ref, &updated)?;
    }

    Ok(updated)
}

fn hook_trigger_stale_reason_from_env() -> Option<IndexStaleReason> {
    let value = std::env::var(SEMANTIC_INDEX_HOOK_TRIGGER_ENV).ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        SEMANTIC_INDEX_HOOK_TRIGGER_POST_CHECKOUT => Some(IndexStaleReason::BranchCheckout),
        SEMANTIC_INDEX_HOOK_TRIGGER_POST_MERGE => Some(IndexStaleReason::Merge),
        SEMANTIC_INDEX_HOOK_TRIGGER_POST_REWRITE => Some(IndexStaleReason::PostRewrite),
        _ => None,
    }
}

fn current_git_state(workspace_ref: &Path) -> Option<GitWorkspaceState> {
    let branch = git_command_output(workspace_ref, ["rev-parse", "--abbrev-ref", "HEAD"])?;
    let head = git_command_output(workspace_ref, ["rev-parse", "HEAD"])?;
    Some(GitWorkspaceState { branch, head })
}

fn git_command_output<const N: usize>(workspace_ref: &Path, args: [&str; N]) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(workspace_ref).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

fn derive_manifest_index_status(
    previous_manifest: Option<&DerivedIndexManifest>,
    next_manifest: &DerivedIndexManifest,
    workspace_ref: &Path,
) -> RetrievalIndexState {
    if next_manifest.workspace_root != workspace_ref.to_string_lossy() {
        return RetrievalIndexState::Incompatible;
    }
    if next_manifest.fts5_state == ManifestFtsState::Corrupt {
        return RetrievalIndexState::Corrupt;
    }
    if previous_manifest.is_some_and(|manifest| manifest.requires_rebuild_against(next_manifest)) {
        return RetrievalIndexState::Incompatible;
    }
    match next_manifest.sqlite_vec_state {
        VectorExtensionState::Ready => RetrievalIndexState::Ready,
        VectorExtensionState::Missing | VectorExtensionState::Unsupported => {
            RetrievalIndexState::SemanticUnavailable
        }
        VectorExtensionState::Stale | VectorExtensionState::Degraded => {
            RetrievalIndexState::Degraded
        }
        VectorExtensionState::Corrupt => RetrievalIndexState::Corrupt,
    }
}

fn semantic_engine_for_vector_state(
    vector_extension_state: VectorExtensionState,
) -> SemanticEngine {
    match vector_extension_state {
        VectorExtensionState::Ready => SemanticEngine::SqliteVec,
        VectorExtensionState::Missing
        | VectorExtensionState::Unsupported
        | VectorExtensionState::Stale
        | VectorExtensionState::Degraded
        | VectorExtensionState::Corrupt => SemanticEngine::BaselineJson,
    }
}

fn workspace_fingerprint(documents: &[RetrievalDocument]) -> String {
    let fingerprint_input = documents
        .iter()
        .map(|document| {
            format!(
                "{}|{}|{}|{}|{}",
                document.source_ref,
                document.source_kind.as_str(),
                document.compatibility_state.as_str(),
                document.staleness_state.as_str(),
                semantic_content_hash(&document.content)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("fnv64:{:016x}", stable_hash(&fingerprint_input))
}

fn derived_index_config_fingerprint() -> String {
    let config_line = format!(
        "{}|max_indexed_bytes={MAX_INDEXED_BYTES}|max_query_terms={MAX_QUERY_TERMS}",
        DERIVED_INDEX_CONFIG_FINGERPRINT_NAMESPACE,
    );
    format!("fnv64:{:016x}", stable_hash(&config_line))
}

fn semantic_chunker_fingerprint() -> String {
    let chunker_line = format!(
        "{}|schema={SEMANTIC_SCHEMA_LINE_V1}|max_chunk_bytes={MAX_SEMANTIC_CHUNK_BYTES}",
        DERIVED_INDEX_CHUNKER_FINGERPRINT_NAMESPACE,
    );
    format!("fnv64:{:016x}", stable_hash(&chunker_line))
}

fn semantic_embedding_fingerprint() -> String {
    let embedding_line = format!(
        "{}|dimensions={SEMANTIC_EMBEDDING_DIMENSIONS}|ngram_width={SEMANTIC_FEATURE_NGRAM_WIDTH}|min_token_length={MIN_SEMANTIC_TOKEN_LENGTH}",
        DERIVED_INDEX_EMBEDDING_FINGERPRINT_NAMESPACE,
    );
    format!("fnv64:{:016x}", stable_hash(&embedding_line))
}

fn upsert_semantic_manifest_row(
    connection: &Connection,
    manifest: &DerivedIndexManifest,
    refresh_reason: &str,
) -> Result<(), ContextIntelligenceBuildError> {
    connection
        .execute(
            "INSERT OR REPLACE INTO semantic_index_manifest (
                manifest_id,
                workspace_root,
                vector_extension_state,
                last_semantic_refresh_reason
            ) VALUES (?1, ?2, ?3, ?4)",
            params![
                SEMANTIC_INDEX_MANIFEST_ID,
                manifest.workspace_root,
                manifest.sqlite_vec_state.as_str(),
                refresh_reason,
            ],
        )
        .map_err(|error| {
            ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string())
        })?;
    Ok(())
}

fn refresh_documents(
    connection: &Connection,
    documents: &[RetrievalDocument],
) -> Result<(), ContextIntelligenceBuildError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    transaction
        .execute("DELETE FROM retrieval_documents", [])
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    transaction
        .execute("DELETE FROM retrieval_documents_fts", [])
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    for document in documents {
        transaction
            .execute(
                "INSERT INTO retrieval_documents (
                    source_ref,
                    source_kind,
                    authority_rank,
                    provenance_summary,
                    compatibility_state,
                    staleness_state,
                    metadata_json,
                    content
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    document.source_ref,
                    document.source_kind.as_str(),
                    document.authority_rank.as_str(),
                    document.provenance_summary,
                    document.compatibility_state.as_str(),
                    document.staleness_state.as_str(),
                    document.metadata_json,
                    document.content,
                ],
            )
            .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO retrieval_documents_fts (source_ref, content) VALUES (?1, ?2)",
                params![document.source_ref, document.content],
            )
            .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    }

    transaction
        .commit()
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))
}

fn refresh_semantic_chunks(
    connection: &Connection,
    documents: &[RetrievalDocument],
    vector_extension_state: VectorExtensionState,
) -> Result<(), ContextIntelligenceBuildError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    let vector_table_available = semantic_vector_table_exists(&transaction)?;
    let chunk_state = semantic_chunk_state_for_vector_extension(vector_extension_state);
    let existing_chunks = load_existing_semantic_chunk_rows(&transaction)?;
    let desired_chunks = build_desired_semantic_chunk_rows(documents, chunk_state)?;
    let desired_chunk_ids =
        desired_chunks.iter().map(|row| row.chunk_id.clone()).collect::<BTreeSet<_>>();

    let remove_vectors = vector_table_available;
    delete_missing_semantic_chunks(
        &transaction,
        &existing_chunks,
        &desired_chunk_ids,
        remove_vectors,
    )?;

    for desired in &desired_chunks {
        let existing = existing_chunks.get(&desired.chunk_id);
        if existing != Some(desired) {
            upsert_semantic_chunk_row(&transaction, desired)?;
        }
        sync_semantic_vector_row(&transaction, vector_table_available, existing, desired)?;
    }

    transaction
        .commit()
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticChunkRefreshRow {
    chunk_id: String,
    source_kind: String,
    source_ref: String,
    provenance_boundary: String,
    provenance_ref: String,
    content_hash: String,
    embedding_state: String,
    embedding_dimensions: i64,
    canon_semantic_contract_line: Option<String>,
    semantic_labels_json: String,
    semantic_schema_line: String,
    chunk_text: String,
    embedding_payload_json: String,
}

fn build_desired_semantic_chunk_rows(
    documents: &[RetrievalDocument],
    chunk_state: SemanticChunkState,
) -> Result<Vec<SemanticChunkRefreshRow>, ContextIntelligenceBuildError> {
    let mut rows = Vec::new();

    for document in documents.iter().filter(|document| semantic_eligible_document(document)) {
        let chunk_text = truncate_string(document.content.clone(), MAX_SEMANTIC_CHUNK_BYTES);
        let embedding = if chunk_state == SemanticChunkState::Ready {
            semantic_embedding(&chunk_text)
        } else {
            vec![0.0; SEMANTIC_EMBEDDING_DIMENSIONS]
        };
        let semantic_labels_json = serde_json::to_string(&semantic_labels(document))
            .map_err(|error| ContextIntelligenceBuildError::SerializeMetadata(error.to_string()))?;
        let embedding_payload_json = serde_json::to_string(&embedding)
            .map_err(|error| ContextIntelligenceBuildError::SerializeMetadata(error.to_string()))?;

        rows.push(SemanticChunkRefreshRow {
            chunk_id: semantic_chunk_id(&document.source_ref),
            source_kind: document.source_kind.as_str().to_string(),
            source_ref: document.source_ref.clone(),
            provenance_boundary: document
                .canon_semantic_provenance_boundary
                .map(CanonSemanticProvenanceBoundary::as_str)
                .unwrap_or_else(|| document.source_kind.as_str())
                .to_string(),
            provenance_ref: document
                .canon_semantic_provenance_ref
                .clone()
                .unwrap_or_else(|| document.source_ref.clone()),
            content_hash: semantic_content_hash(&chunk_text),
            embedding_state: chunk_state.as_str().to_string(),
            embedding_dimensions: SEMANTIC_EMBEDDING_DIMENSIONS as i64,
            canon_semantic_contract_line: document.canon_semantic_contract_line.clone(),
            semantic_labels_json,
            semantic_schema_line: SEMANTIC_SCHEMA_LINE_V1.to_string(),
            chunk_text,
            embedding_payload_json,
        });
    }

    Ok(rows)
}

fn load_existing_semantic_chunk_rows(
    connection: &Connection,
) -> Result<BTreeMap<String, SemanticChunkRefreshRow>, ContextIntelligenceBuildError> {
    let mut statement = connection
        .prepare(
            "SELECT
                chunk_id,
                source_kind,
                source_ref,
                provenance_boundary,
                provenance_ref,
                content_hash,
                embedding_state,
                embedding_dimensions,
                canon_semantic_contract_line,
                semantic_labels_json,
                semantic_schema_line,
                chunk_text,
                embedding_payload_json
             FROM semantic_chunks",
        )
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok(SemanticChunkRefreshRow {
                chunk_id: row.get(0)?,
                source_kind: row.get(1)?,
                source_ref: row.get(2)?,
                provenance_boundary: row.get(3)?,
                provenance_ref: row.get(4)?,
                content_hash: row.get(5)?,
                embedding_state: row.get(6)?,
                embedding_dimensions: row.get(7)?,
                canon_semantic_contract_line: row.get(8)?,
                semantic_labels_json: row.get(9)?,
                semantic_schema_line: row.get(10)?,
                chunk_text: row.get(11)?,
                embedding_payload_json: row.get(12)?,
            })
        })
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    let mut chunks = BTreeMap::new();
    for row in rows {
        let row =
            row.map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
        chunks.insert(row.chunk_id.clone(), row);
    }

    Ok(chunks)
}

fn delete_missing_semantic_chunks(
    connection: &Connection,
    existing_chunks: &BTreeMap<String, SemanticChunkRefreshRow>,
    desired_chunk_ids: &BTreeSet<String>,
    remove_vectors: bool,
) -> Result<(), ContextIntelligenceBuildError> {
    for chunk_id in existing_chunks.keys().filter(|chunk_id| !desired_chunk_ids.contains(*chunk_id))
    {
        if remove_vectors {
            delete_semantic_vector_row(connection, chunk_id)?;
        }
        connection
            .execute("DELETE FROM semantic_chunks WHERE chunk_id = ?1", params![chunk_id])
            .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    }

    Ok(())
}

fn upsert_semantic_chunk_row(
    connection: &Connection,
    row: &SemanticChunkRefreshRow,
) -> Result<(), ContextIntelligenceBuildError> {
    connection
        .execute(
            "INSERT INTO semantic_chunks (
                chunk_id,
                source_kind,
                source_ref,
                provenance_boundary,
                provenance_ref,
                content_hash,
                embedding_state,
                embedding_dimensions,
                canon_semantic_contract_line,
                semantic_labels_json,
                semantic_schema_line,
                chunk_text,
                embedding_payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(chunk_id) DO UPDATE SET
                source_kind = excluded.source_kind,
                source_ref = excluded.source_ref,
                provenance_boundary = excluded.provenance_boundary,
                provenance_ref = excluded.provenance_ref,
                content_hash = excluded.content_hash,
                embedding_state = excluded.embedding_state,
                embedding_dimensions = excluded.embedding_dimensions,
                canon_semantic_contract_line = excluded.canon_semantic_contract_line,
                semantic_labels_json = excluded.semantic_labels_json,
                semantic_schema_line = excluded.semantic_schema_line,
                chunk_text = excluded.chunk_text,
                embedding_payload_json = excluded.embedding_payload_json",
            params![
                row.chunk_id,
                row.source_kind,
                row.source_ref,
                row.provenance_boundary,
                row.provenance_ref,
                row.content_hash,
                row.embedding_state,
                row.embedding_dimensions,
                row.canon_semantic_contract_line,
                row.semantic_labels_json,
                row.semantic_schema_line,
                row.chunk_text,
                row.embedding_payload_json,
            ],
        )
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    Ok(())
}

fn sync_semantic_vector_row(
    connection: &Connection,
    vector_table_available: bool,
    existing: Option<&SemanticChunkRefreshRow>,
    desired: &SemanticChunkRefreshRow,
) -> Result<(), ContextIntelligenceBuildError> {
    if !vector_table_available {
        return Ok(());
    }

    let vector_exists = semantic_vector_row_exists(connection, &desired.chunk_id)?;
    let vector_required = desired.embedding_state == SemanticChunkState::Ready.as_str();
    if !vector_required {
        if vector_exists {
            delete_semantic_vector_row(connection, &desired.chunk_id)?;
        }
        return Ok(());
    }

    if existing == Some(desired) && vector_exists {
        return Ok(());
    }

    if vector_exists {
        delete_semantic_vector_row(connection, &desired.chunk_id)?;
    }

    connection
        .execute(
            &format!(
                "INSERT INTO {SEMANTIC_VECTORS_TABLE_NAME} (chunk_id, embedding) VALUES (?1, vec_f32(?2))"
            ),
            params![desired.chunk_id, desired.embedding_payload_json],
        )
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    Ok(())
}

fn semantic_vector_row_exists(
    connection: &Connection,
    chunk_id: &str,
) -> Result<bool, ContextIntelligenceBuildError> {
    let count = connection
        .query_row(
            &format!("SELECT COUNT(*) FROM {SEMANTIC_VECTORS_TABLE_NAME} WHERE chunk_id = ?1"),
            params![chunk_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    Ok(count > 0)
}

fn delete_semantic_vector_row(
    connection: &Connection,
    chunk_id: &str,
) -> Result<(), ContextIntelligenceBuildError> {
    connection
        .execute(
            &format!("DELETE FROM {SEMANTIC_VECTORS_TABLE_NAME} WHERE chunk_id = ?1"),
            params![chunk_id],
        )
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    Ok(())
}

fn semantic_chunk_state_for_vector_extension(
    vector_extension_state: VectorExtensionState,
) -> SemanticChunkState {
    match vector_extension_state {
        VectorExtensionState::Ready => SemanticChunkState::Ready,
        VectorExtensionState::Missing | VectorExtensionState::Unsupported => {
            SemanticChunkState::Blocked
        }
        VectorExtensionState::Stale => SemanticChunkState::Stale,
        VectorExtensionState::Degraded => SemanticChunkState::Stale,
        VectorExtensionState::Corrupt => SemanticChunkState::Blocked,
    }
}

fn semantic_eligible_document(document: &RetrievalDocument) -> bool {
    document.compatibility_state == RetrievalCompatibilityState::Compatible
        && !document.content.trim().is_empty()
}

fn semantic_labels(document: &RetrievalDocument) -> Vec<String> {
    let mut labels = vec![
        document.source_kind.as_str().to_string(),
        document.authority_rank.as_str().to_string(),
    ];
    if let Some(extension) =
        Path::new(&document.source_ref).extension().and_then(|value| value.to_str())
    {
        labels.push(extension.to_ascii_lowercase());
    }
    labels.extend(document.canon_semantic_labels.iter().cloned());
    labels.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn semantic_chunk_id(source_ref: &str) -> String {
    SemanticChunkRecord::stable_chunk_id(source_ref, 0)
}

fn semantic_content_hash(value: &str) -> String {
    format!("fnv64:{:016x}", stable_hash(value))
}

fn query_semantic_matches(
    connection: &Connection,
    goal_text: &str,
    selected_targets: &[String],
    expansion_limit: usize,
    vector_extension_state: VectorExtensionState,
) -> Result<SemanticQueryResult, ContextIntelligenceBuildError> {
    if expansion_limit == 0 {
        return Ok(SemanticQueryResult {
            matches: Vec::new(),
            vector_query_attempted: false,
            vector_chunk_candidates_returned: 0,
        });
    }

    let query_text = semantic_query_text(goal_text, selected_targets);
    let query_embedding = semantic_embedding(&query_text);
    if query_embedding.iter().all(|value| *value == 0.0) {
        return Ok(SemanticQueryResult {
            matches: Vec::new(),
            vector_query_attempted: false,
            vector_chunk_candidates_returned: 0,
        });
    }

    if vector_extension_state == VectorExtensionState::Ready
        && semantic_vector_table_exists(connection)?
    {
        return query_semantic_matches_from_vec0(connection, &query_embedding, expansion_limit);
    }

    query_semantic_matches_from_payload(connection, &query_embedding, expansion_limit)
}

fn query_semantic_matches_from_payload(
    connection: &Connection,
    query_embedding: &[f64],
    expansion_limit: usize,
) -> Result<SemanticQueryResult, ContextIntelligenceBuildError> {
    let mut statement = connection
        .prepare(
            "SELECT source_ref, provenance_ref, canon_semantic_contract_line, embedding_payload_json
             FROM semantic_chunks
             WHERE embedding_state = ?1",
        )
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    let rows = statement
        .query_map(params![SemanticChunkState::Ready.as_str()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;

    let mut matches_by_ref = BTreeMap::<String, SemanticMatchResult>::new();
    for row in rows {
        let (source_ref, provenance_ref, contract_line, embedding_payload_json) =
            row.map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
        let embedding = serde_json::from_str::<Vec<f64>>(&embedding_payload_json)
            .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
        let Some(score) = RetrievalScore::from_raw(cosine_similarity(query_embedding, &embedding))
        else {
            continue;
        };
        if score.as_raw() < MIN_SEMANTIC_SIMILARITY_SCORE {
            continue;
        }

        let next_match = SemanticMatchResult {
            source_ref: source_ref.clone(),
            semantic_score: score,
            canon_semantic_contract_line: contract_line.clone(),
            canon_semantic_provenance_ref: contract_line.as_ref().map(|_| provenance_ref.clone()),
        };
        match matches_by_ref.get(&source_ref) {
            Some(existing) if existing.semantic_score.as_raw() >= score.as_raw() => {}
            _ => {
                matches_by_ref.insert(source_ref, next_match);
            }
        }
    }

    let mut matches = matches_by_ref.into_values().collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .semantic_score
            .as_raw()
            .partial_cmp(&left.semantic_score.as_raw())
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.source_ref.cmp(&right.source_ref))
    });
    matches.truncate(expansion_limit);
    Ok(SemanticQueryResult {
        matches,
        vector_query_attempted: false,
        vector_chunk_candidates_returned: 0,
    })
}

fn query_semantic_matches_from_vec0(
    connection: &Connection,
    query_embedding: &[f64],
    expansion_limit: usize,
) -> Result<SemanticQueryResult, ContextIntelligenceBuildError> {
    let query_vector_json = serde_json::to_string(query_embedding)
        .map_err(|error| ContextIntelligenceBuildError::SerializeMetadata(error.to_string()))?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT chunks.source_ref,
                    chunks.provenance_ref,
                    chunks.canon_semantic_contract_line,
                    vectors.distance
             FROM {SEMANTIC_VECTORS_TABLE_NAME} AS vectors
             INNER JOIN semantic_chunks AS chunks
                 ON chunks.chunk_id = vectors.chunk_id
             WHERE vectors.embedding MATCH vec_f32(?1)
               AND vectors.k = ?2
               AND chunks.embedding_state = ?3
             ORDER BY vectors.distance
            "
        ))
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
    let rows = statement
        .query_map(
            params![query_vector_json, expansion_limit as i64, SemanticChunkState::Ready.as_str(),],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            },
        )
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;

    let mut matches_by_ref = BTreeMap::<String, SemanticMatchResult>::new();
    let mut vector_chunk_candidates_returned = 0;
    for row in rows {
        let (source_ref, provenance_ref, contract_line, distance) =
            row.map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;
        let maybe_score = filtered_semantic_score_from_vec_distance(distance);
        let Some(score) = maybe_score else {
            continue;
        };
        vector_chunk_candidates_returned += 1;

        let next_match = SemanticMatchResult {
            source_ref: source_ref.clone(),
            semantic_score: score,
            canon_semantic_contract_line: contract_line.clone(),
            canon_semantic_provenance_ref: contract_line.as_ref().map(|_| provenance_ref.clone()),
        };
        match matches_by_ref.get(&source_ref) {
            Some(existing) if existing.semantic_score.as_raw() >= score.as_raw() => {}
            _ => {
                matches_by_ref.insert(source_ref, next_match);
            }
        }
    }

    let mut matches = matches_by_ref.into_values().collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .semantic_score
            .as_raw()
            .partial_cmp(&left.semantic_score.as_raw())
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.source_ref.cmp(&right.source_ref))
    });
    Ok(SemanticQueryResult {
        matches,
        vector_query_attempted: true,
        vector_chunk_candidates_returned,
    })
}

fn semantic_score_from_vec_distance(distance: f64) -> Option<RetrievalScore> {
    if !distance.is_finite() {
        return None;
    }

    RetrievalScore::from_raw(1.0 / (1.0 + distance.max(0.0)))
}

fn filtered_semantic_score_from_vec_distance(distance: f64) -> Option<RetrievalScore> {
    let score = semantic_score_from_vec_distance(distance)?;
    (score.as_raw() >= MIN_SEMANTIC_SIMILARITY_SCORE).then_some(score)
}

fn semantic_query_text(goal_text: &str, selected_targets: &[String]) -> String {
    std::iter::once(goal_text)
        .chain(selected_targets.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn semantic_embedding(text: &str) -> Vec<f64> {
    let mut vector = vec![0.0; SEMANTIC_EMBEDDING_DIMENSIONS];
    for token in semantic_tokens(text) {
        add_semantic_feature(&mut vector, &token, 1.0);
        for ngram in semantic_ngrams(&token) {
            add_semantic_feature(&mut vector, &ngram, 0.5);
        }
    }
    normalize_embedding(vector)
}

fn semantic_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut previous_was_lowercase = false;

    for character in text.chars() {
        if character.is_alphanumeric() {
            if character.is_uppercase() && previous_was_lowercase && !current.is_empty() {
                push_semantic_token(&mut tokens, &mut current);
            }
            current.extend(character.to_lowercase());
            previous_was_lowercase = character.is_lowercase();
        } else {
            push_semantic_token(&mut tokens, &mut current);
            previous_was_lowercase = false;
        }
    }
    push_semantic_token(&mut tokens, &mut current);

    tokens
}

fn push_semantic_token(tokens: &mut Vec<String>, current: &mut String) {
    if current.len() >= MIN_SEMANTIC_TOKEN_LENGTH {
        tokens.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

fn semantic_ngrams(token: &str) -> Vec<String> {
    let characters = token.chars().collect::<Vec<_>>();
    if characters.len() <= SEMANTIC_FEATURE_NGRAM_WIDTH {
        return Vec::new();
    }

    characters
        .windows(SEMANTIC_FEATURE_NGRAM_WIDTH)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

fn add_semantic_feature(vector: &mut [f64], feature: &str, weight: f64) {
    let index = stable_hash(feature) as usize % vector.len();
    vector[index] += weight;
}

fn normalize_embedding(mut vector: Vec<f64>) -> Vec<f64> {
    let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm == 0.0 {
        return vector;
    }
    for value in &mut vector {
        *value /= norm;
    }
    vector
}

fn cosine_similarity(left: &[f64], right: &[f64]) -> f64 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    left.iter().zip(right).map(|(lhs, rhs)| lhs * rhs).sum::<f64>()
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn lexical_score_from_bm25(raw_rank: f64) -> Option<RetrievalScore> {
    RetrievalScore::from_raw(1.0 / (1.0 + raw_rank.abs()))
}

fn build_fts_query(goal_text: &str, selected_targets: &[String]) -> String {
    let mut tokens = BTreeSet::new();

    for value in std::iter::once(goal_text).chain(selected_targets.iter().map(String::as_str)) {
        for token in value.split(|character: char| !character.is_alphanumeric()) {
            let normalized = token.trim().to_lowercase();
            if normalized.len() >= 3 {
                tokens.insert(normalized);
            }
            if tokens.len() >= MAX_QUERY_TERMS {
                break;
            }
        }
        if tokens.len() >= MAX_QUERY_TERMS {
            break;
        }
    }

    tokens
        .into_iter()
        .take(MAX_QUERY_TERMS)
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn candidate_decisions_from_lexical_matches(
    matches: &[RankedDocumentRef],
) -> Vec<CandidateDecision> {
    matches
        .iter()
        .map(|document| CandidateDecision {
            source_ref: document.source_ref.clone(),
            match_origin: RetrievalMatchOrigin::Fts,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: SelectionStrategy::Fts.selection_reason().to_string(),
            lexical_score: document.lexical_score,
            semantic_score: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        })
        .collect()
}

fn candidate_decisions_from_refs(
    refs: &[String],
    match_origin: RetrievalMatchOrigin,
) -> Vec<CandidateDecision> {
    let selection_reason = match match_origin {
        RetrievalMatchOrigin::Fts => SelectionStrategy::Fts.selection_reason().to_string(),
        RetrievalMatchOrigin::StructuredFallback => {
            SelectionStrategy::StructuredFallback.selection_reason().to_string()
        }
        RetrievalMatchOrigin::SemanticExpand => SEMANTIC_EXPAND_SELECTION_REASON.to_string(),
        RetrievalMatchOrigin::SemanticRerank => SEMANTIC_RERANK_SELECTION_REASON.to_string(),
    };

    refs.iter()
        .map(|source_ref| CandidateDecision {
            source_ref: source_ref.clone(),
            match_origin,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: selection_reason.clone(),
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        })
        .collect()
}

fn merge_rejected_decisions(
    mut base: Vec<CandidateDecision>,
    additional: Vec<CandidateDecision>,
) -> Vec<CandidateDecision> {
    base.extend(additional);
    base
}

fn selectable_documents(
    documents: &[RetrievalDocument],
    semantic_policy: SemanticAccelerationPolicyState,
) -> Vec<RetrievalDocument> {
    documents
        .iter()
        .filter(|document| document_is_queryable(document, semantic_policy))
        .cloned()
        .collect()
}

fn document_is_queryable(
    document: &RetrievalDocument,
    semantic_policy: SemanticAccelerationPolicyState,
) -> bool {
    semantic_policy != SemanticAccelerationPolicyState::Local
        || document.source_kind != RetrievalSourceKind::CanonArtifact
        || document.compatibility_state == RetrievalCompatibilityState::Compatible
}

fn canon_rejected_candidate_decisions(
    documents: &[RetrievalDocument],
    semantic_policy: SemanticAccelerationPolicyState,
) -> Vec<CandidateDecision> {
    if semantic_policy != SemanticAccelerationPolicyState::Local {
        return Vec::new();
    }

    documents
        .iter()
        .filter(|document| !document_is_queryable(document, semantic_policy))
        .map(|document| CandidateDecision {
            source_ref: document.source_ref.clone(),
            match_origin: RetrievalMatchOrigin::StructuredFallback,
            selection_state: CandidateSelectionState::Rejected,
            selection_reason: canon_skip_selection_reason(document),
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: document.canon_semantic_contract_line.clone(),
            canon_semantic_provenance_ref: document.canon_semantic_provenance_ref.clone(),
        })
        .collect()
}

fn canon_skip_selection_reason(document: &RetrievalDocument) -> String {
    format!(
        "{CANON_SEMANTIC_SKIP_SELECTION_PREFIX} {}",
        document
            .compatibility_reason
            .as_deref()
            .unwrap_or(CANON_SEMANTIC_DESCRIPTOR_MISSING_REASON)
    )
}

fn promote_selected_ranked_refs(
    selected_refs: Vec<RankedDocumentRef>,
    selected_targets: &[String],
    documents: &[RetrievalDocument],
    evidence_limit: usize,
) -> Vec<RankedDocumentRef> {
    let available_refs =
        documents.iter().map(|document| document.source_ref.as_str()).collect::<BTreeSet<_>>();
    let lexical_scores = selected_refs
        .iter()
        .map(|document| (document.source_ref.clone(), document.lexical_score))
        .collect::<BTreeMap<_, _>>();
    let mut promoted_refs = selected_targets
        .iter()
        .filter(|target| available_refs.contains(target.as_str()))
        .map(|target| RankedDocumentRef {
            source_ref: target.clone(),
            lexical_score: lexical_scores.get(target).copied().flatten(),
        })
        .collect::<Vec<_>>();

    for document in selected_refs {
        if promoted_refs.iter().any(|existing| existing.source_ref == document.source_ref) {
            continue;
        }
        promoted_refs.push(document);
        if promoted_refs.len() >= evidence_limit {
            break;
        }
    }
    promoted_refs.truncate(evidence_limit);
    promoted_refs
}

fn apply_local_semantic_hybrid(
    inputs: HybridSelectionInputs<'_>,
    base_selected: Vec<CandidateDecision>,
    semantic_matches: Vec<SemanticMatchResult>,
) -> HybridSelectionResult {
    match inputs.semantic_policy {
        SemanticAccelerationPolicyState::Disabled => HybridSelectionResult {
            selected: base_selected,
            rejected: Vec::new(),
            semantic_projection: semantic_projection_for_local_result(
                inputs.semantic_policy,
                VectorExtensionState::Unsupported,
            ),
            retrieval_state: inputs.base_retrieval_state,
        },
        SemanticAccelerationPolicyState::Local
            if inputs.vector_extension_state != VectorExtensionState::Ready =>
        {
            HybridSelectionResult {
                selected: base_selected,
                rejected: Vec::new(),
                semantic_projection: semantic_projection_for_local_result(
                    inputs.semantic_policy,
                    inputs.vector_extension_state,
                ),
                retrieval_state: inputs.base_retrieval_state,
            }
        }
        SemanticAccelerationPolicyState::Local => {
            apply_semantic_hybrid_local_ready(inputs, base_selected, semantic_matches)
        }
    }
}

fn apply_semantic_hybrid_local_ready(
    inputs: HybridSelectionInputs<'_>,
    base_selected: Vec<CandidateDecision>,
    semantic_matches: Vec<SemanticMatchResult>,
) -> HybridSelectionResult {
    let document_map = inputs
        .documents
        .iter()
        .map(|document| (document.source_ref.clone(), document))
        .collect::<BTreeMap<_, _>>();
    let selected_target_refs = inputs.selected_targets.iter().collect::<BTreeSet<_>>();
    let limited_matches =
        semantic_matches.into_iter().take(inputs.expansion_limit).collect::<Vec<_>>();
    if limited_matches.is_empty() {
        return HybridSelectionResult {
            selected: base_selected,
            rejected: Vec::new(),
            semantic_projection: semantic_projection_for_local_hybrid_outcome(
                HybridOutcome::BaselineOnly,
                Some(SEMANTIC_BASELINE_ONLY_REASON.to_string()),
            ),
            retrieval_state: inputs.base_retrieval_state,
        };
    }

    let semantic_scores = limited_matches
        .iter()
        .map(|candidate| (candidate.source_ref.clone(), candidate.clone()))
        .collect::<BTreeMap<_, _>>();
    let locked_prefix_len = base_selected
        .iter()
        .take_while(|candidate| selected_target_refs.contains(&candidate.source_ref))
        .count();
    let mut selected = base_selected;
    let original_order =
        selected.iter().map(|candidate| candidate.source_ref.clone()).collect::<Vec<_>>();
    let mut tail = selected.split_off(locked_prefix_len);
    let original_tail_positions = tail
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.source_ref.clone(), index))
        .collect::<BTreeMap<_, _>>();
    tail.sort_by(|left, right| {
        compare_candidate_by_authority_and_score(
            left,
            right,
            &document_map,
            &semantic_scores,
            &original_tail_positions,
        )
    });

    let mut reranked = false;
    for (index, candidate) in tail.iter_mut().enumerate() {
        if let Some(semantic_match) = semantic_scores.get(&candidate.source_ref) {
            let original_position =
                original_tail_positions.get(&candidate.source_ref).copied().unwrap_or(index);
            if original_position != index {
                reranked = true;
                candidate.match_origin = RetrievalMatchOrigin::SemanticRerank;
                candidate.selection_reason = SEMANTIC_RERANK_SELECTION_REASON.to_string();
                candidate.semantic_score = Some(semantic_match.semantic_score);
            }
        }
    }
    selected.extend(tail);

    let (expanded_count, rejected) = expand_or_reject_semantic_candidates(
        &mut selected,
        &semantic_scores,
        inputs.evidence_limit,
    );
    let selected_order =
        selected.iter().map(|candidate| candidate.source_ref.clone()).collect::<Vec<_>>();
    let (hybrid_outcome, terminal_reason, retrieval_state) = determine_hybrid_outcome(
        expanded_count,
        reranked,
        &selected_order,
        &original_order,
        inputs.base_retrieval_state,
    );

    HybridSelectionResult {
        selected,
        rejected,
        semantic_projection: semantic_projection_for_local_hybrid_outcome(
            hybrid_outcome,
            terminal_reason,
        ),
        retrieval_state,
    }
}

fn compare_candidate_by_authority_and_score(
    left: &CandidateDecision,
    right: &CandidateDecision,
    document_map: &BTreeMap<String, &RetrievalDocument>,
    semantic_scores: &BTreeMap<String, SemanticMatchResult>,
    original_tail_positions: &BTreeMap<String, usize>,
) -> Ordering {
    let left_rank = document_map
        .get(&left.source_ref)
        .map_or(AuthorityRank::Semantic, |document| document.authority_rank);
    let right_rank = document_map
        .get(&right.source_ref)
        .map_or(AuthorityRank::Semantic, |document| document.authority_rank);
    let right_score = semantic_scores
        .get(&right.source_ref)
        .map(|candidate| candidate.semantic_score.as_raw())
        .unwrap_or(0.0);
    let left_score = semantic_scores
        .get(&left.source_ref)
        .map(|candidate| candidate.semantic_score.as_raw())
        .unwrap_or(0.0);
    authority_sort_index(left_rank)
        .cmp(&authority_sort_index(right_rank))
        .then_with(|| right_score.partial_cmp(&left_score).unwrap_or(Ordering::Equal))
        .then_with(|| {
            original_tail_positions
                .get(&left.source_ref)
                .cmp(&original_tail_positions.get(&right.source_ref))
        })
}

fn expand_or_reject_semantic_candidates(
    selected: &mut Vec<CandidateDecision>,
    semantic_scores: &BTreeMap<String, SemanticMatchResult>,
    evidence_limit: usize,
) -> (usize, Vec<CandidateDecision>) {
    let mut expanded_count = 0;
    let mut rejected = Vec::new();
    for semantic_match in semantic_scores.values() {
        if selected.iter().any(|candidate| candidate.source_ref == semantic_match.source_ref) {
            continue;
        }
        if selected.len() < evidence_limit {
            expanded_count += 1;
            selected.push(CandidateDecision {
                source_ref: semantic_match.source_ref.clone(),
                match_origin: RetrievalMatchOrigin::SemanticExpand,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: SEMANTIC_EXPAND_SELECTION_REASON.to_string(),
                lexical_score: None,
                semantic_score: Some(semantic_match.semantic_score),
                canon_semantic_contract_line: semantic_match.canon_semantic_contract_line.clone(),
                canon_semantic_provenance_ref: semantic_match.canon_semantic_provenance_ref.clone(),
            });
            continue;
        }
        rejected.push(CandidateDecision {
            source_ref: semantic_match.source_ref.clone(),
            match_origin: RetrievalMatchOrigin::SemanticExpand,
            selection_state: CandidateSelectionState::Rejected,
            selection_reason: SEMANTIC_REJECTED_LIMIT_REASON.to_string(),
            lexical_score: None,
            semantic_score: Some(semantic_match.semantic_score),
            canon_semantic_contract_line: semantic_match.canon_semantic_contract_line.clone(),
            canon_semantic_provenance_ref: semantic_match.canon_semantic_provenance_ref.clone(),
        });
    }
    (expanded_count, rejected)
}

fn determine_hybrid_outcome(
    expanded_count: usize,
    reranked: bool,
    selected_order: &[String],
    original_order: &[String],
    base_retrieval_state: RetrievalState,
) -> (HybridOutcome, Option<String>, RetrievalState) {
    if expanded_count > 0 {
        (
            HybridOutcome::Expanded,
            Some(format!(
                "{SEMANTIC_EXPANDED_REASON}: {expanded_count} additional bounded match(es)"
            )),
            RetrievalState::Selected,
        )
    } else if reranked && selected_order != original_order {
        (
            HybridOutcome::Reranked,
            Some(SEMANTIC_RERANKED_REASON.to_string()),
            RetrievalState::Selected,
        )
    } else {
        (
            HybridOutcome::BaselineOnly,
            Some(SEMANTIC_BASELINE_ONLY_REASON.to_string()),
            base_retrieval_state,
        )
    }
}

struct HybridSelectionInputs<'a> {
    semantic_policy: SemanticAccelerationPolicyState,
    selected_targets: &'a [String],
    documents: &'a [RetrievalDocument],
    vector_extension_state: VectorExtensionState,
    evidence_limit: usize,
    expansion_limit: usize,
    base_retrieval_state: RetrievalState,
}

fn authority_sort_index(authority_rank: AuthorityRank) -> u8 {
    match authority_rank {
        AuthorityRank::Structured => 0,
        AuthorityRank::Canon => 1,
        AuthorityRank::WorkspaceOverride => 2,
        AuthorityRank::Semantic => 3,
    }
}

fn structured_fallback_refs(
    documents: &[RetrievalDocument],
    selected_targets: &[String],
    evidence_limit: usize,
) -> Vec<String> {
    let mut ordered = documents.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        structured_priority(left, selected_targets)
            .cmp(&structured_priority(right, selected_targets))
            .then_with(|| left.source_ref.cmp(&right.source_ref))
    });
    ordered.into_iter().take(evidence_limit).map(|document| document.source_ref.clone()).collect()
}

struct SemanticTraceRecordInputs<'a> {
    query_id: &'a str,
    documents: &'a [RetrievalDocument],
    semantic_policy_state: SemanticPolicyState,
    semantic_capability_state: SemanticCapabilityState,
    hybrid_outcome: HybridOutcome,
    retrieval_state: RetrievalState,
    retrieval_index_state: RetrievalIndexState,
    execution_details: SemanticTraceExecutionDetails,
    terminal_reason: Option<&'a str>,
    selected_evidence: &'a [RetrievedEvidenceCandidate],
    rejected_candidates: &'a [RetrievedEvidenceCandidate],
}

fn build_semantic_trace_records(inputs: SemanticTraceRecordInputs<'_>) -> Vec<SemanticTraceRecord> {
    let SemanticTraceRecordInputs {
        query_id,
        documents,
        semantic_policy_state,
        semantic_capability_state,
        hybrid_outcome,
        retrieval_state,
        retrieval_index_state,
        execution_details,
        terminal_reason,
        selected_evidence,
        rejected_candidates,
    } = inputs;
    let mut records = Vec::new();
    let document_map = documents
        .iter()
        .map(|document| (document.source_ref.as_str(), document))
        .collect::<BTreeMap<_, _>>();

    records.push(SemanticTraceRecord {
        record_id: format!("{query_id}:semantic:capability"),
        event_kind: SemanticTraceEventKind::CapabilityEvaluated,
        candidate_ref: None,
        match_origin: None,
        compatibility_state: None,
        semantic_score: None,
        canon_artifact_class: None,
        canon_semantic_contract_line: None,
        canon_semantic_provenance_boundary: None,
        canon_semantic_provenance_ref: None,
        reason: format!(
            "{SEMANTIC_TRACE_CAPABILITY_REASON_PREFIX} {}",
            semantic_capability_state.as_str()
        ),
    });

    if semantic_policy_state == SemanticPolicyState::Local
        && execution_details.extension_load_attempted
    {
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:extension-load"),
            event_kind: SemanticTraceEventKind::ExtensionLoadAttempted,
            candidate_ref: None,
            match_origin: None,
            compatibility_state: None,
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: format!(
                "{SEMANTIC_TRACE_EXTENSION_LOAD_ATTEMPTED_PREFIX}: capability={} retrieval_index_state={}",
                semantic_capability_state.as_str(),
                retrieval_index_state.as_str()
            ),
        });
    }

    if semantic_policy_state == SemanticPolicyState::Local
        && retrieval_index_state != RetrievalIndexState::Insufficient
    {
        let eligible_count =
            documents.iter().filter(|document| semantic_eligible_document(document)).count();
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:index"),
            event_kind: SemanticTraceEventKind::IndexRefreshed,
            candidate_ref: None,
            match_origin: None,
            compatibility_state: None,
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: format!(
                "{SEMANTIC_TRACE_INDEX_REFRESHED_PREFIX}: {eligible_count} eligible document(s)"
            ),
        });
    }

    if semantic_policy_state == SemanticPolicyState::Local
        && execution_details.vector_query_attempted
    {
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:vector-query"),
            event_kind: SemanticTraceEventKind::VectorQueryExecuted,
            candidate_ref: None,
            match_origin: None,
            compatibility_state: None,
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: format!(
                "{SEMANTIC_TRACE_VECTOR_QUERY_EXECUTED_PREFIX}: engine={}",
                semantic_engine_for_trace(semantic_policy_state, semantic_capability_state)
                    .as_str()
            ),
        });
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:vector-candidates"),
            event_kind: SemanticTraceEventKind::VectorCandidatesReturned,
            candidate_ref: None,
            match_origin: None,
            compatibility_state: None,
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: format!(
                "{SEMANTIC_TRACE_VECTOR_CANDIDATES_RETURNED_PREFIX}: {}",
                execution_details.vector_chunk_candidates_returned
            ),
        });
    }

    for document in documents.iter().filter(|document| {
        document.source_kind == RetrievalSourceKind::CanonArtifact
            && document.compatibility_state != RetrievalCompatibilityState::Compatible
    }) {
        let reason = document
            .compatibility_reason
            .clone()
            .unwrap_or_else(|| CANON_SEMANTIC_DESCRIPTOR_MISSING_REASON.to_string());
        let record_suffix = stable_hash(&document.source_ref);
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:block:{record_suffix:016x}"),
            event_kind: SemanticTraceEventKind::ChunkBlocked,
            candidate_ref: Some(document.source_ref.clone()),
            match_origin: None,
            compatibility_state: Some(document.compatibility_state),
            semantic_score: None,
            canon_artifact_class: document.canon_artifact_class.clone(),
            canon_semantic_contract_line: document.canon_semantic_contract_line.clone(),
            canon_semantic_provenance_boundary: document.canon_semantic_provenance_boundary,
            canon_semantic_provenance_ref: document.canon_semantic_provenance_ref.clone(),
            reason: format!("{SEMANTIC_TRACE_CHUNK_BLOCKED_PREFIX} {reason}"),
        });
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:canon-skip:{record_suffix:016x}"),
            event_kind: SemanticTraceEventKind::CanonArtifactSkipped,
            candidate_ref: Some(document.source_ref.clone()),
            match_origin: None,
            compatibility_state: Some(document.compatibility_state),
            semantic_score: None,
            canon_artifact_class: document.canon_artifact_class.clone(),
            canon_semantic_contract_line: document.canon_semantic_contract_line.clone(),
            canon_semantic_provenance_boundary: document.canon_semantic_provenance_boundary,
            canon_semantic_provenance_ref: document.canon_semantic_provenance_ref.clone(),
            reason,
        });
    }

    for candidate in selected_evidence {
        let Some(event_kind) = semantic_candidate_event_kind(candidate.match_origin) else {
            continue;
        };
        let document = document_map.get(candidate.source_ref.as_str()).copied();
        records.push(SemanticTraceRecord {
            record_id: format!(
                "{query_id}:semantic:selected:{:016x}",
                stable_hash(&candidate.source_ref)
            ),
            event_kind,
            candidate_ref: Some(candidate.source_ref.clone()),
            match_origin: Some(candidate.match_origin),
            compatibility_state: Some(candidate.compatibility_state),
            semantic_score: candidate.semantic_score,
            canon_artifact_class: document
                .and_then(|document| document.canon_artifact_class.clone()),
            canon_semantic_contract_line: candidate.canon_semantic_contract_line.clone(),
            canon_semantic_provenance_boundary: document
                .and_then(|document| document.canon_semantic_provenance_boundary),
            canon_semantic_provenance_ref: candidate.canon_semantic_provenance_ref.clone(),
            reason: candidate.selection_reason.clone(),
        });
    }

    for candidate in rejected_candidates {
        if !matches!(
            candidate.match_origin,
            RetrievalMatchOrigin::SemanticExpand | RetrievalMatchOrigin::SemanticRerank
        ) {
            continue;
        }
        let document = document_map.get(candidate.source_ref.as_str()).copied();
        records.push(SemanticTraceRecord {
            record_id: format!(
                "{query_id}:semantic:rejected:{:016x}",
                stable_hash(&candidate.source_ref)
            ),
            event_kind: SemanticTraceEventKind::CandidateRejected,
            candidate_ref: Some(candidate.source_ref.clone()),
            match_origin: Some(candidate.match_origin),
            compatibility_state: Some(candidate.compatibility_state),
            semantic_score: candidate.semantic_score,
            canon_artifact_class: document
                .and_then(|document| document.canon_artifact_class.clone()),
            canon_semantic_contract_line: candidate.canon_semantic_contract_line.clone(),
            canon_semantic_provenance_boundary: document
                .and_then(|document| document.canon_semantic_provenance_boundary),
            canon_semantic_provenance_ref: candidate.canon_semantic_provenance_ref.clone(),
            reason: candidate.selection_reason.clone(),
        });
    }

    if matches!(hybrid_outcome, HybridOutcome::Skipped | HybridOutcome::Fallback)
        && terminal_reason.is_some()
    {
        records.push(SemanticTraceRecord {
            record_id: format!("{query_id}:semantic:fallback"),
            event_kind: SemanticTraceEventKind::FallbackApplied,
            candidate_ref: None,
            match_origin: None,
            compatibility_state: None,
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: terminal_reason.unwrap_or_default().to_string(),
        });
    }

    records.push(SemanticTraceRecord {
        record_id: format!("{query_id}:semantic:outcome"),
        event_kind: SemanticTraceEventKind::HybridOutcomeRecorded,
        candidate_ref: None,
        match_origin: None,
        compatibility_state: None,
        semantic_score: None,
        canon_artifact_class: None,
        canon_semantic_contract_line: None,
        canon_semantic_provenance_boundary: None,
        canon_semantic_provenance_ref: None,
        reason: format!(
            "{SEMANTIC_TRACE_HYBRID_OUTCOME_PREFIX} {} with hybrid outcome {}",
            retrieval_state.as_str(),
            hybrid_outcome.as_str()
        ),
    });

    records
}

fn semantic_engine_for_trace(
    semantic_policy_state: SemanticPolicyState,
    semantic_capability_state: SemanticCapabilityState,
) -> SemanticEngine {
    match semantic_policy_state {
        SemanticPolicyState::Disabled => SemanticEngine::Disabled,
        SemanticPolicyState::Local => match semantic_capability_state {
            SemanticCapabilityState::Ready => SemanticEngine::SqliteVec,
            SemanticCapabilityState::Unavailable
            | SemanticCapabilityState::Unsupported
            | SemanticCapabilityState::Degraded
            | SemanticCapabilityState::Corrupt => SemanticEngine::BaselineJson,
        },
    }
}

fn semantic_candidate_event_kind(
    match_origin: RetrievalMatchOrigin,
) -> Option<SemanticTraceEventKind> {
    match match_origin {
        RetrievalMatchOrigin::SemanticExpand => Some(SemanticTraceEventKind::CandidateExpanded),
        RetrievalMatchOrigin::SemanticRerank => Some(SemanticTraceEventKind::CandidateReranked),
        RetrievalMatchOrigin::Fts | RetrievalMatchOrigin::StructuredFallback => None,
    }
}

#[cfg(test)]
fn promote_selected_target_refs(
    selected_refs: Vec<String>,
    selected_targets: &[String],
    documents: &[RetrievalDocument],
    evidence_limit: usize,
) -> Vec<String> {
    let available_refs =
        documents.iter().map(|document| document.source_ref.as_str()).collect::<BTreeSet<_>>();
    let mut promoted_refs = selected_targets
        .iter()
        .filter(|target| available_refs.contains(target.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    for source_ref in selected_refs {
        if promoted_refs.iter().any(|existing| existing == &source_ref) {
            continue;
        }
        promoted_refs.push(source_ref);
        if promoted_refs.len() >= evidence_limit {
            break;
        }
    }
    promoted_refs.truncate(evidence_limit);
    promoted_refs
}

fn structured_priority(document: &RetrievalDocument, selected_targets: &[String]) -> (u8, u8, u8) {
    let selected_target_rank =
        if selected_targets.iter().any(|target| target == &document.source_ref) { 0 } else { 1 };
    let authority_rank = match document.authority_rank {
        AuthorityRank::Structured => 0,
        AuthorityRank::Canon => 1,
        AuthorityRank::WorkspaceOverride => 2,
        AuthorityRank::Semantic => 3,
    };
    let source_kind_rank = match document.source_kind {
        RetrievalSourceKind::WorkspaceFile => 0,
        RetrievalSourceKind::CanonArtifact => 1,
        RetrievalSourceKind::ProjectMemory => 2,
        RetrievalSourceKind::Trace => 3,
        RetrievalSourceKind::ReviewFinding => 4,
        RetrievalSourceKind::VerificationEvidence => 5,
    };
    (selected_target_rank, authority_rank, source_kind_rank)
}

fn derive_relationships_and_findings(
    workspace_ref: &Path,
    selected_evidence: &[RetrievedEvidenceCandidate],
    credibility: ContextPackCredibility,
    staleness_reason: Option<&str>,
) -> (Vec<RelationshipProjection>, Vec<ImpactAnalysisFinding>) {
    let mut relationships = Vec::new();
    let mut findings = Vec::new();

    for candidate in selected_evidence {
        if candidate.source_kind != RetrievalSourceKind::WorkspaceFile
            || !candidate.source_ref.starts_with("src/")
        {
            continue;
        }

        let relationship_id = format!("relationship-{}", relationships.len() + 1);
        if let Some(test_ref) = matching_test_ref(workspace_ref, &candidate.source_ref) {
            relationships.push(RelationshipProjection {
                relationship_id,
                subject_ref: candidate.source_ref.clone(),
                relationship_kind: RelationshipKind::ExercisesTest,
                credibility_state: RelationshipCredibilityState::Credible,
                explanation: format!("located focused test evidence in {test_ref}"),
                supporting_candidate_ids: vec![candidate.candidate_id.clone()],
            });
            continue;
        }

        relationships.push(RelationshipProjection {
            relationship_id: relationship_id.clone(),
            subject_ref: candidate.source_ref.clone(),
            relationship_kind: RelationshipKind::RequiresEvidence,
            credibility_state: RelationshipCredibilityState::Tentative,
            explanation: "no focused regression test is indexed for this bounded source target"
                .to_string(),
            supporting_candidate_ids: vec![candidate.candidate_id.clone()],
        });
        findings.push(ImpactAnalysisFinding {
            finding_id: format!("finding-{}", findings.len() + 1),
            finding_kind: ImpactFindingKind::MissingTest,
            subject_ref: suggested_test_ref(&candidate.source_ref),
            status: ImpactFindingStatus::Open,
            severity: ImpactFindingSeverity::Medium,
            recommended_follow_up: "add or refresh the focused regression test".to_string(),
            supporting_relationship_ids: vec![relationship_id],
        });
    }

    if credibility == ContextPackCredibility::Stale
        && let Some(reason) = staleness_reason
        && let Some(candidate) = selected_evidence.first()
    {
        let supporting_relationship_id = if let Some(relationship) = relationships.first() {
            relationship.relationship_id.clone()
        } else {
            let relationship_id = format!("relationship-{}", relationships.len() + 1);
            relationships.push(RelationshipProjection {
                relationship_id: relationship_id.clone(),
                subject_ref: candidate.source_ref.clone(),
                relationship_kind: RelationshipKind::SupportsRisk,
                credibility_state: RelationshipCredibilityState::Tentative,
                explanation: reason.to_string(),
                supporting_candidate_ids: vec![candidate.candidate_id.clone()],
            });
            relationship_id
        };

        findings.push(ImpactAnalysisFinding {
            finding_id: format!("finding-{}", findings.len() + 1),
            finding_kind: ImpactFindingKind::EvidenceGap,
            subject_ref: candidate.source_ref.clone(),
            status: ImpactFindingStatus::Open,
            severity: ImpactFindingSeverity::Medium,
            recommended_follow_up: format!("refresh bounded evidence: {reason}"),
            supporting_relationship_ids: vec![supporting_relationship_id],
        });
    }

    (relationships, findings)
}

fn matching_test_ref(workspace_ref: &Path, source_ref: &str) -> Option<String> {
    test_candidates(source_ref)
        .into_iter()
        .find(|candidate| workspace_ref.join(candidate).is_file())
}

fn suggested_test_ref(source_ref: &str) -> String {
    test_candidates(source_ref)
        .into_iter()
        .next()
        .unwrap_or_else(|| format!("tests/{}", Path::new(source_ref).display()))
}

fn test_candidates(source_ref: &str) -> Vec<String> {
    let path = Path::new(source_ref);
    let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or("unknown.rs");
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("unknown");
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("rs");
    vec![
        format!("tests/{file_name}"),
        format!("tests/{stem}_test.{extension}"),
        format!("test/{file_name}"),
    ]
}

fn retrieval_source_kind(kind: ContextInputKind) -> RetrievalSourceKind {
    match kind {
        ContextInputKind::WorkspaceFile
        | ContextInputKind::DomainTemplate
        | ContextInputKind::DomainStandard
        | ContextInputKind::ExternalContextInput => RetrievalSourceKind::WorkspaceFile,
        ContextInputKind::SymbolHint => RetrievalSourceKind::VerificationEvidence,
        ContextInputKind::AuthoredBrief
        | ContextInputKind::Negotiation
        | ContextInputKind::CanonMemory => RetrievalSourceKind::ProjectMemory,
        ContextInputKind::RecentTrace => RetrievalSourceKind::Trace,
        ContextInputKind::CanonArtifact | ContextInputKind::CanonCapability => {
            RetrievalSourceKind::CanonArtifact
        }
    }
}

fn authority_rank(kind: ContextInputKind) -> AuthorityRank {
    match kind {
        ContextInputKind::WorkspaceFile
        | ContextInputKind::DomainTemplate
        | ContextInputKind::DomainStandard
        | ContextInputKind::ExternalContextInput => AuthorityRank::Structured,
        ContextInputKind::CanonArtifact
        | ContextInputKind::CanonCapability
        | ContextInputKind::CanonMemory => AuthorityRank::Canon,
        ContextInputKind::AuthoredBrief | ContextInputKind::Negotiation => {
            AuthorityRank::WorkspaceOverride
        }
        ContextInputKind::SymbolHint | ContextInputKind::RecentTrace => AuthorityRank::Semantic,
    }
}

fn default_compatibility_state(
    kind: ContextInputKind,
    has_file_backing: bool,
) -> RetrievalCompatibilityState {
    match kind {
        ContextInputKind::WorkspaceFile
        | ContextInputKind::RecentTrace
        | ContextInputKind::CanonArtifact
        | ContextInputKind::ExternalContextInput
        | ContextInputKind::DomainTemplate
        | ContextInputKind::DomainStandard
            if !has_file_backing =>
        {
            RetrievalCompatibilityState::MissingMetadata
        }
        _ => RetrievalCompatibilityState::Compatible,
    }
}

fn canon_semantic_compatibility(
    workspace_ref: &Path,
    input: &ContextInput,
    relative_path: Option<&str>,
    has_file_backing: bool,
    semantic_policy: SemanticAccelerationPolicyState,
) -> Option<CanonSemanticCompatibility> {
    if !matches!(input.kind, ContextInputKind::CanonArtifact | ContextInputKind::CanonCapability) {
        return None;
    }

    if !has_file_backing {
        return Some(CanonSemanticCompatibility {
            artifact_class: None,
            semantic_contract_line: None,
            semantic_provenance_boundary: None,
            semantic_provenance_ref: None,
            semantic_labels: Vec::new(),
            compatibility_state: RetrievalCompatibilityState::MissingMetadata,
            compatibility_reason: Some(CANON_ARTIFACT_FILE_MISSING_REASON.to_string()),
        });
    }

    let Some(relative_path) = relative_path else {
        return Some(CanonSemanticCompatibility {
            artifact_class: None,
            semantic_contract_line: None,
            semantic_provenance_boundary: None,
            semantic_provenance_ref: None,
            semantic_labels: Vec::new(),
            compatibility_state: RetrievalCompatibilityState::MissingMetadata,
            compatibility_reason: Some(CANON_ARTIFACT_FILE_MISSING_REASON.to_string()),
        });
    };
    let artifact_path = workspace_ref.join(relative_path);
    let Some(surface) = read_canon_semantic_artifact_surface(&artifact_path) else {
        return Some(CanonSemanticCompatibility {
            artifact_class: None,
            semantic_contract_line: None,
            semantic_provenance_boundary: None,
            semantic_provenance_ref: None,
            semantic_labels: Vec::new(),
            compatibility_state: if semantic_policy == SemanticAccelerationPolicyState::Disabled {
                RetrievalCompatibilityState::Compatible
            } else {
                RetrievalCompatibilityState::MissingMetadata
            },
            compatibility_reason: if semantic_policy == SemanticAccelerationPolicyState::Disabled {
                None
            } else {
                Some(CANON_SEMANTIC_SIDECAR_MISSING_REASON.to_string())
            },
        });
    };

    let artifact_class = surface.publication_target_class.clone();
    let semantic_descriptor = surface.semantic_descriptor.as_ref();
    let semantic_contract_line =
        semantic_descriptor.map(|descriptor| descriptor.semantic_contract_line.clone());
    let semantic_provenance_boundary =
        semantic_descriptor.and_then(|descriptor| descriptor.semantic_provenance_boundary);
    let semantic_provenance_ref = semantic_descriptor.and_then(|descriptor| {
        descriptor
            .semantic_provenance_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    });
    let semantic_labels = semantic_descriptor
        .map(|descriptor| descriptor.semantic_labels.clone())
        .unwrap_or_default();

    if semantic_policy == SemanticAccelerationPolicyState::Disabled {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::Compatible,
            compatibility_reason: None,
        });
    }

    let Some(lineage) = surface.lineage.as_ref() else {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::MissingMetadata,
            compatibility_reason: Some(CANON_SEMANTIC_SIDECAR_MISSING_REASON.to_string()),
        });
    };
    if CompatibilityOutcome::check(&lineage.contract_version) == CompatibilityOutcome::Unsupported {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::UnsupportedContract,
            compatibility_reason: Some(format!(
                "Canon artifact indexing contract line `{}` is unsupported",
                lineage.contract_version
            )),
        });
    }

    if surface.promotion_view != PromotionStateView::Stable
        || artifact_class.as_deref() != Some(CANON_PUBLICATION_TARGET_STABLE)
    {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::PolicyBlocked,
            compatibility_reason: Some(CANON_SEMANTIC_SURFACE_BLOCKED_REASON.to_string()),
        });
    }

    let Some(descriptor) = semantic_descriptor else {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::MissingMetadata,
            compatibility_reason: Some(CANON_SEMANTIC_DESCRIPTOR_MISSING_REASON.to_string()),
        });
    };
    if !descriptor.is_supported_contract_line() {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::UnsupportedContract,
            compatibility_reason: Some(format!(
                "Canon artifact semantic contract line `{}` is unsupported",
                descriptor.semantic_contract_line
            )),
        });
    }
    if descriptor.semantic_eligibility == CanonSemanticEligibilityState::Excluded {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::PolicyBlocked,
            compatibility_reason: Some(
                descriptor
                    .semantic_exclusion_reason
                    .clone()
                    .unwrap_or_else(|| CANON_SEMANTIC_EXCLUDED_REASON.to_string()),
            ),
        });
    }
    if semantic_provenance_boundary.is_none() || semantic_provenance_ref.is_none() {
        return Some(CanonSemanticCompatibility {
            artifact_class,
            semantic_contract_line,
            semantic_provenance_boundary,
            semantic_provenance_ref,
            semantic_labels,
            compatibility_state: RetrievalCompatibilityState::MissingMetadata,
            compatibility_reason: Some(CANON_SEMANTIC_PROVENANCE_MISSING_REASON.to_string()),
        });
    }

    Some(CanonSemanticCompatibility {
        artifact_class,
        semantic_contract_line,
        semantic_provenance_boundary,
        semantic_provenance_ref,
        semantic_labels,
        compatibility_state: RetrievalCompatibilityState::Compatible,
        compatibility_reason: None,
    })
}

fn staleness_state(
    kind: ContextInputKind,
    credibility: ContextPackCredibility,
    staleness_reason: Option<&str>,
) -> RetrievalStalenessState {
    if credibility == ContextPackCredibility::Stale
        && staleness_reason.is_some()
        && matches!(
            kind,
            ContextInputKind::RecentTrace
                | ContextInputKind::CanonArtifact
                | ContextInputKind::CanonCapability
                | ContextInputKind::CanonMemory
        )
    {
        return RetrievalStalenessState::Stale;
    }
    RetrievalStalenessState::Fresh
}

fn resolved_relative_path(workspace_ref: &Path, reference: &str) -> Option<String> {
    let reference_path = Path::new(reference);
    if reference_path.is_absolute() {
        return reference_path
            .strip_prefix(workspace_ref)
            .ok()
            .map(|path| path.to_string_lossy().into_owned());
    }

    let absolute_path = workspace_ref.join(reference_path);
    absolute_path.is_file().then(|| reference_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{
        AdvancedContextBuildState, AdvancedContextConfig, CONTEXT_REASON_ARCHIVED_CONTEXT_EXCLUDED,
        CONTEXT_REASON_CRITICAL_DOWNGRADED, CONTEXT_REASON_REPOSITORY_MAP_UNAVAILABLE,
        CONTEXT_REASON_SEARCH_REQUIRED_BEFORE_READ, CONTEXT_REASON_SNAPSHOT_CACHE_STALE,
        CONTEXT_REASON_TRACKED_CACHE_DETECTED, ContextIntelligenceBuildError, RelationshipKind,
        RetrievalDocument, SEMANTIC_CHUNKS_TABLE_NAME, SEMANTIC_INDEX_HOOK_TRIGGER_ENV,
        SEMANTIC_INDEX_HOOK_TRIGGER_POST_MERGE, SEMANTIC_INDEX_HOOK_TRIGGER_POST_REWRITE,
        SEMANTIC_INDEX_MANIFEST_ID, SEMANTIC_REFRESH_PENDING_REASON, SEMANTIC_VECTORS_TABLE_NAME,
        SemanticChunkRefreshRow, aggregate_index_doctor_status, append_patch_safe_edit_attempt,
        append_selected_candidate_findings, build_advanced_context_projection,
        build_advanced_context_projection_with_vector_state,
        build_context_substrate_projection_fields, classify_context_fidelity,
        collect_retrieval_documents, default_compatibility_state, delete_missing_semantic_chunks,
        derive_manifest_index_status, derive_relationships_and_findings,
        detect_vector_extension_state, digest_ref_for_candidate,
        filtered_semantic_score_from_vec_distance, hook_trigger_stale_reason_from_env,
        index_stale_warning, initialize_schema, initialize_semantic_vector_table_for_state,
        inspect_manifest_consistency, inspect_vector_schema_consistency,
        lifecycle_relevance_for_candidate, manifest_consistency_check, open_connection,
        promote_selected_target_refs, query_semantic_matches, query_semantic_matches_from_payload,
        read_derived_index_manifest, recommended_action_for_index_state, refresh_and_query_index,
        rejected_candidate_omission_finding, repository_map_state_for_workspace,
        resolved_relative_path, selected_candidate_inclusion_mode, semantic_engine_for_trace,
        semantic_score_from_vec_distance, snapshot_cache_state_for_workspace, staleness_state,
        structured_fallback_refs, sync_semantic_vector_row, upsert_semantic_manifest_row,
        vector_extension_state_from_modules, vector_schema_check,
        vector_schema_consistency_from_tables,
    };
    #[cfg(feature = "sqlite-vec")]
    use super::{
        query_semantic_matches_from_vec0, register_sqlite_vec_auto_extension,
        sqlite_auto_extension_result,
    };
    use crate::domain::configuration::SemanticAccelerationPolicyState;
    use crate::domain::context_intelligence::{
        AuthorityRank, CandidateSelectionState, ContextFidelityTier, ContextInclusionMode,
        ContextOmissionSeverity, ContextPackEntryProjection, DerivedIndexManifest, HybridOutcome,
        ImpactFindingKind, IndexDoctorConsistencyState, IndexDoctorStatus, IndexRefreshReason,
        IndexStaleReason, ManifestFtsState, RepositoryMapState, RetrievalCompatibilityState,
        RetrievalIndexState, RetrievalMatchOrigin, RetrievalSourceKind, RetrievalStalenessState,
        RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticChunkState,
        SemanticEngine, SemanticPolicyState, SnapshotCacheState, VectorExtensionState,
    };
    use crate::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn context_input(
        kind: ContextInputKind,
        reference: &str,
        source: &str,
        rationale: &str,
        primary: bool,
    ) -> ContextInput {
        ContextInput {
            kind,
            reference: reference.to_string(),
            rationale: rationale.to_string(),
            source: source.to_string(),
            primary,
        }
    }

    fn selected_candidate(
        source_kind: RetrievalSourceKind,
        source_ref: &str,
    ) -> RetrievedEvidenceCandidate {
        RetrievedEvidenceCandidate {
            candidate_id: "candidate-1".to_string(),
            source_kind,
            source_ref: source_ref.to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::Fts,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: "selected through bounded retrieval".to_string(),
            provenance_summary: "bounded evidence projection".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }
    }

    #[test]
    fn substrate_helper_functions_cover_fidelity_inclusion_digest_and_findings() {
        let workspace = temp_workspace("boundline-context-substrate-helpers");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("logs")).unwrap();
        fs::create_dir_all(workspace.join("archive")).unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            format!("{}\n", "critical bounded context".repeat(2_000)),
        )
        .unwrap();
        fs::write(workspace.join("logs/error.log"), "validation failed\nstack trace\n".repeat(800))
            .unwrap();
        fs::write(workspace.join("archive/legacy.md"), "# Legacy\n\nArchived context.\n").unwrap();

        let critical_refs = BTreeSet::from(["src/lib.rs".to_string()]);
        assert_eq!(
            classify_context_fidelity("src/lib.rs", &critical_refs),
            ContextFidelityTier::Critical
        );
        assert_eq!(
            classify_context_fidelity("archive/legacy.md", &critical_refs),
            ContextFidelityTier::Archived
        );
        assert_eq!(
            classify_context_fidelity("docs/brief.md", &critical_refs),
            ContextFidelityTier::Ambient
        );
        assert_eq!(
            classify_context_fidelity("src/helper.rs", &critical_refs),
            ContextFidelityTier::Supporting
        );

        let critical_candidate =
            selected_candidate(RetrievalSourceKind::WorkspaceFile, "src/lib.rs");
        let log_candidate = selected_candidate(RetrievalSourceKind::Trace, "logs/error.log");
        let archived_candidate =
            selected_candidate(RetrievalSourceKind::WorkspaceFile, "archive/legacy.md");

        assert_eq!(
            selected_candidate_inclusion_mode(&workspace, &critical_candidate, None),
            ContextInclusionMode::Excerpt
        );
        assert_eq!(
            selected_candidate_inclusion_mode(&workspace, &log_candidate, None),
            ContextInclusionMode::Digest
        );
        assert_eq!(
            selected_candidate_inclusion_mode(&workspace, &archived_candidate, None),
            ContextInclusionMode::Omitted
        );

        let digest_ref = digest_ref_for_candidate(
            &workspace,
            &log_candidate,
            Some(&RetrievalDocument {
                source_ref: "logs/error.log".to_string(),
                source_kind: RetrievalSourceKind::Trace,
                authority_rank: AuthorityRank::Structured,
                provenance_summary: "bounded evidence projection".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                compatibility_reason: None,
                staleness_state: RetrievalStalenessState::Fresh,
                canon_artifact_class: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_boundary: None,
                canon_semantic_provenance_ref: None,
                canon_semantic_labels: Vec::new(),
                metadata_json: "{}".to_string(),
                content: "validation failed\nstack trace\n".repeat(2),
            }),
            ContextInclusionMode::Digest,
        )
        .unwrap();
        assert_eq!(digest_ref.artifact_kind, "log");
        assert_eq!(digest_ref.resolve_path, "logs/error.log");
        assert!(digest_ref.summary.contains("validation failed"));

        let mut omission_findings = Vec::new();
        append_selected_candidate_findings(
            &mut omission_findings,
            &ContextPackEntryProjection {
                source_ref: "src/lib.rs".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                fidelity_tier: ContextFidelityTier::Critical,
                inclusion_mode: ContextInclusionMode::Excerpt,
                required_for_admission: true,
                reason: "search-before-read bounded the critical file".to_string(),
                authority_rank: AuthorityRank::Structured,
                resolved_excerpt_anchor: Some("src/lib.rs#bounded-excerpt".to_string()),
                lifecycle_relevance: Some("implementation_surface".to_string()),
                risk_relevance: None,
                ranking_rationale: Some("origin=fts".to_string()),
                digest_ref: None,
            },
            &critical_candidate,
        );
        append_selected_candidate_findings(
            &mut omission_findings,
            &ContextPackEntryProjection {
                source_ref: "logs/error.log".to_string(),
                source_kind: RetrievalSourceKind::Trace,
                fidelity_tier: ContextFidelityTier::Supporting,
                inclusion_mode: ContextInclusionMode::Digest,
                required_for_admission: false,
                reason: "large trace compacted to digest".to_string(),
                authority_rank: AuthorityRank::Structured,
                resolved_excerpt_anchor: Some("logs/error.log#digest-summary".to_string()),
                lifecycle_relevance: Some("recent_trace".to_string()),
                risk_relevance: Some("risk_signal".to_string()),
                ranking_rationale: Some("origin=fts".to_string()),
                digest_ref: Some(digest_ref.clone()),
            },
            &log_candidate,
        );
        append_selected_candidate_findings(
            &mut omission_findings,
            &ContextPackEntryProjection {
                source_ref: "archive/legacy.md".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                fidelity_tier: ContextFidelityTier::Archived,
                inclusion_mode: ContextInclusionMode::Omitted,
                required_for_admission: false,
                reason: "archived context excluded".to_string(),
                authority_rank: AuthorityRank::Structured,
                resolved_excerpt_anchor: None,
                lifecycle_relevance: Some("implementation_surface".to_string()),
                risk_relevance: None,
                ranking_rationale: Some("origin=structured_fallback".to_string()),
                digest_ref: None,
            },
            &archived_candidate,
        );

        assert!(omission_findings.iter().any(|finding| {
            finding.reason_code == "search_required_before_read"
                && finding.required_fidelity == Some(ContextFidelityTier::Critical)
                && finding.observed_mode == Some(ContextInclusionMode::Excerpt)
        }));
        assert!(omission_findings.iter().any(|finding| {
            finding.reason_code == "artifact_compacted_to_digest"
                && finding.severity == ContextOmissionSeverity::Info
        }));
        assert!(omission_findings.iter().any(|finding| {
            finding.reason_code == "archived_context_excluded"
                && finding.observed_mode == Some(ContextInclusionMode::Omitted)
        }));

        let rejected = rejected_candidate_omission_finding(
            &workspace,
            &critical_candidate,
            &ContextPackEntryProjection {
                source_ref: "src/lib.rs".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                fidelity_tier: ContextFidelityTier::Critical,
                inclusion_mode: ContextInclusionMode::Omitted,
                required_for_admission: true,
                reason: "unsafe full read refused".to_string(),
                authority_rank: AuthorityRank::Structured,
                resolved_excerpt_anchor: None,
                lifecycle_relevance: Some("implementation_surface".to_string()),
                risk_relevance: None,
                ranking_rationale: Some("origin=fts".to_string()),
                digest_ref: None,
            },
            None,
        );
        assert_eq!(rejected.reason_code, "unsafe_full_read_refused");
        assert_eq!(rejected.observed_mode, Some(ContextInclusionMode::Omitted));

        let mut patch_attempts = Vec::new();
        append_patch_safe_edit_attempt(
            &mut patch_attempts,
            &workspace,
            &ContextPackEntryProjection {
                source_ref: "src/lib.rs".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                fidelity_tier: ContextFidelityTier::Critical,
                inclusion_mode: ContextInclusionMode::Excerpt,
                required_for_admission: true,
                reason: "large file requires patch-safe editing".to_string(),
                authority_rank: AuthorityRank::Structured,
                resolved_excerpt_anchor: Some("src/lib.rs#bounded-excerpt".to_string()),
                lifecycle_relevance: Some("implementation_surface".to_string()),
                risk_relevance: None,
                ranking_rationale: Some("origin=fts".to_string()),
                digest_ref: None,
            },
            None,
        );
        assert_eq!(patch_attempts.len(), 1);
        assert_eq!(patch_attempts[0].target_ref, "src/lib.rs");
    }

    #[test]
    fn substrate_repository_and_snapshot_state_helpers_cover_missing_and_tracked_paths() {
        let workspace = temp_workspace("boundline-context-substrate-state");
        assert_eq!(
            repository_map_state_for_workspace(&workspace, RetrievalIndexState::Missing, &[]),
            RepositoryMapState::Missing
        );
        assert_eq!(
            repository_map_state_for_workspace(
                &workspace,
                RetrievalIndexState::Missing,
                &["src/lib.rs".to_string()]
            ),
            RepositoryMapState::Ready
        );
        assert_eq!(
            repository_map_state_for_workspace(&workspace, RetrievalIndexState::Stale, &[]),
            RepositoryMapState::Stale
        );
        assert_eq!(
            repository_map_state_for_workspace(&workspace, RetrievalIndexState::Corrupt, &[]),
            RepositoryMapState::Corrupt
        );

        assert_eq!(
            snapshot_cache_state_for_workspace(&workspace, RetrievalIndexState::Missing),
            SnapshotCacheState::Missing
        );
        assert_eq!(
            snapshot_cache_state_for_workspace(&workspace, RetrievalIndexState::Stale),
            SnapshotCacheState::Stale
        );

        let status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .status()
            .unwrap();
        assert!(status.success());
        let derived_index =
            workspace.join(".boundline/context-intelligence/retrieval-index.sqlite3");
        fs::create_dir_all(derived_index.parent().unwrap()).unwrap();
        fs::write(&derived_index, "tracked derived index").unwrap();
        let add_status = std::process::Command::new("git")
            .args(["add", ".boundline/context-intelligence/retrieval-index.sqlite3"])
            .current_dir(&workspace)
            .status()
            .unwrap();
        assert!(add_status.success());

        assert_eq!(
            snapshot_cache_state_for_workspace(&workspace, RetrievalIndexState::Ready),
            SnapshotCacheState::Tracked
        );
    }

    #[test]
    fn substrate_projection_fields_emit_warning_and_blocking_cache_findings() {
        let workspace = temp_workspace("boundline-context-substrate-fields");

        let missing_fields = build_context_substrate_projection_fields(
            &workspace,
            &[],
            &[],
            &[],
            &[],
            &[],
            RetrievalIndexState::Missing,
        );
        assert!(missing_fields.omission_findings.iter().any(|finding| {
            finding.reason_code == CONTEXT_REASON_REPOSITORY_MAP_UNAVAILABLE
                && finding.severity == ContextOmissionSeverity::Warning
        }));

        let stale_fields = build_context_substrate_projection_fields(
            &workspace,
            &[],
            &[],
            &[],
            &[],
            &[],
            RetrievalIndexState::Stale,
        );
        assert!(stale_fields.omission_findings.iter().any(|finding| {
            finding.reason_code == CONTEXT_REASON_SNAPSHOT_CACHE_STALE
                && finding.severity == ContextOmissionSeverity::Warning
        }));

        let git_status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .status()
            .unwrap();
        assert!(git_status.success());
        let cache_root = workspace.join(".boundline/context-intelligence");
        fs::create_dir_all(&cache_root).unwrap();
        let cache_file = cache_root.join("retrieval-index.sqlite3");
        fs::write(&cache_file, "tracked").unwrap();
        let add_status = std::process::Command::new("git")
            .args(["add", ".boundline/context-intelligence/retrieval-index.sqlite3"])
            .current_dir(&workspace)
            .status()
            .unwrap();
        assert!(add_status.success());

        let tracked_fields = build_context_substrate_projection_fields(
            &workspace,
            &[],
            &[],
            &[],
            &[],
            &[],
            RetrievalIndexState::Ready,
        );
        assert!(tracked_fields.omission_findings.iter().any(|finding| {
            finding.reason_code == CONTEXT_REASON_TRACKED_CACHE_DETECTED
                && finding.severity == ContextOmissionSeverity::Blocking
        }));
    }

    #[test]
    fn substrate_helper_branches_cover_default_summaries_and_noncritical_omissions() {
        let workspace = temp_workspace("boundline-context-substrate-branches");
        fs::create_dir_all(workspace.join("build/generated")).unwrap();
        fs::write(workspace.join("build/generated/output.txt"), "").unwrap();

        let review_candidate = RetrievedEvidenceCandidate {
            candidate_id: "review-candidate".to_string(),
            source_kind: RetrievalSourceKind::ReviewFinding,
            source_ref: "review/finding.md".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::StructuredFallback,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: "review finding selected".to_string(),
            provenance_summary: "review finding provenance".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        };
        assert_eq!(lifecycle_relevance_for_candidate(&review_candidate), "review_feedback");

        let missing_size_candidate =
            selected_candidate(RetrievalSourceKind::WorkspaceFile, "missing/file.rs");
        assert_eq!(
            selected_candidate_inclusion_mode(&workspace, &missing_size_candidate, None),
            ContextInclusionMode::Full
        );

        let digest_candidate =
            selected_candidate(RetrievalSourceKind::WorkspaceFile, "build/generated/output.txt");
        let digest_document = RetrievalDocument {
            source_ref: "build/generated/output.txt".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            authority_rank: AuthorityRank::Structured,
            provenance_summary: "generated output".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            compatibility_reason: None,
            staleness_state: RetrievalStalenessState::Fresh,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            canon_semantic_labels: Vec::new(),
            metadata_json: "{}".to_string(),
            content: String::new(),
        };
        let digest_ref = digest_ref_for_candidate(
            &workspace,
            &digest_candidate,
            Some(&digest_document),
            ContextInclusionMode::Digest,
        )
        .unwrap();
        assert_eq!(digest_ref.summary, "large artifact compacted to a digest-backed reference");
        assert_eq!(digest_ref.artifact_kind, "generated_output");

        let diff_candidate =
            selected_candidate(RetrievalSourceKind::WorkspaceFile, "patches/fix.diff");
        let diff_ref = digest_ref_for_candidate(
            &workspace,
            &diff_candidate,
            Some(&digest_document),
            ContextInclusionMode::Digest,
        )
        .unwrap();
        assert_eq!(diff_ref.artifact_kind, "diff");

        let workspace_artifact_candidate =
            selected_candidate(RetrievalSourceKind::WorkspaceFile, "notes/output.txt");
        let workspace_artifact_ref = digest_ref_for_candidate(
            &workspace,
            &workspace_artifact_candidate,
            Some(&digest_document),
            ContextInclusionMode::Digest,
        )
        .unwrap();
        assert_eq!(workspace_artifact_ref.artifact_kind, "workspace_artifact");

        let optional_omitted_entry = ContextPackEntryProjection {
            source_ref: "archive/legacy.md".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            authority_rank: AuthorityRank::Structured,
            fidelity_tier: ContextFidelityTier::Archived,
            inclusion_mode: ContextInclusionMode::Omitted,
            required_for_admission: false,
            reason: "archived context excluded".to_string(),
            resolved_excerpt_anchor: None,
            digest_ref: None,
            lifecycle_relevance: Some("implementation_surface".to_string()),
            risk_relevance: None,
            ranking_rationale: Some("origin=structured_fallback".to_string()),
        };
        let required_digest_entry = ContextPackEntryProjection {
            source_ref: "logs/error.log".to_string(),
            source_kind: RetrievalSourceKind::Trace,
            authority_rank: AuthorityRank::Structured,
            fidelity_tier: ContextFidelityTier::Critical,
            inclusion_mode: ContextInclusionMode::Digest,
            required_for_admission: true,
            reason: "critical context compacted".to_string(),
            resolved_excerpt_anchor: Some("logs/error.log#digest-summary".to_string()),
            digest_ref: Some(digest_ref.clone()),
            lifecycle_relevance: Some("recent_trace".to_string()),
            risk_relevance: None,
            ranking_rationale: Some("origin=fts".to_string()),
        };
        let mut findings = Vec::new();
        append_selected_candidate_findings(
            &mut findings,
            &optional_omitted_entry,
            &review_candidate,
        );
        append_selected_candidate_findings(
            &mut findings,
            &required_digest_entry,
            &digest_candidate,
        );
        assert!(findings.iter().any(|finding| {
            finding.reason_code == CONTEXT_REASON_ARCHIVED_CONTEXT_EXCLUDED
                && finding.severity == ContextOmissionSeverity::Warning
        }));
        assert!(findings.iter().any(|finding| {
            finding.reason_code == CONTEXT_REASON_CRITICAL_DOWNGRADED
                && finding.severity == ContextOmissionSeverity::Blocking
        }));

        let mut attempts = Vec::new();
        append_patch_safe_edit_attempt(&mut attempts, &workspace, &optional_omitted_entry, None);
        assert!(attempts.is_empty());

        let missing_size_entry = ContextPackEntryProjection {
            source_ref: "missing/file.rs".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            authority_rank: AuthorityRank::Structured,
            fidelity_tier: ContextFidelityTier::Supporting,
            inclusion_mode: ContextInclusionMode::Full,
            required_for_admission: false,
            reason: "missing file".to_string(),
            resolved_excerpt_anchor: None,
            digest_ref: None,
            lifecycle_relevance: Some("implementation_surface".to_string()),
            risk_relevance: None,
            ranking_rationale: Some("origin=fts".to_string()),
        };
        append_patch_safe_edit_attempt(&mut attempts, &workspace, &missing_size_entry, None);
        assert!(attempts.is_empty());

        assert_eq!(
            rejected_candidate_omission_finding(
                &workspace,
                &review_candidate,
                &optional_omitted_entry,
                None,
            )
            .reason_code,
            CONTEXT_REASON_SEARCH_REQUIRED_BEFORE_READ
        );

        assert_eq!(
            repository_map_state_for_workspace(&workspace, RetrievalIndexState::Degraded, &[]),
            RepositoryMapState::Degraded
        );
        assert_eq!(
            snapshot_cache_state_for_workspace(&workspace, RetrievalIndexState::Corrupt),
            SnapshotCacheState::Corrupt
        );
    }

    fn valid_manifest_fixture(workspace: &std::path::Path) -> DerivedIndexManifest {
        DerivedIndexManifest {
            schema_version: "retrieval-index-v3".to_string(),
            workspace_root: workspace.display().to_string(),
            git_branch: Some("main".to_string()),
            git_head: Some("abc123".to_string()),
            last_seen_head: Some("abc123".to_string()),
            index_status: RetrievalIndexState::Ready,
            last_refresh_at: Some("2026-05-30 12:00:00".to_string()),
            last_refresh_reason: Some(IndexRefreshReason::ManualRefresh),
            stale_reason: None,
            file_count: 1,
            chunk_count: 1,
            fts5_state: ManifestFtsState::Ready,
            sqlite_vec_state: VectorExtensionState::Ready,
            semantic_engine: SemanticEngine::SqliteVec,
            workspace_fingerprint: "fnv64:0000000000000001".to_string(),
            config_fingerprint: "fnv64:0000000000000002".to_string(),
            chunker_fingerprint: "fnv64:0000000000000003".to_string(),
            embedding_model_fingerprint: "fnv64:0000000000000004".to_string(),
        }
    }

    #[test]
    fn build_advanced_context_projection_degrades_to_structured_fallback_when_fts_misses() {
        let workspace = temp_workspace("boundline-advanced-context-fallback");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .unwrap();

        let projection = build_advanced_context_projection(
            "quartz zebra mnemonic",
            &workspace,
            &[context_input(
                ContextInputKind::WorkspaceFile,
                "src/lib.rs",
                "workspace_scan",
                "selected workspace target",
                true,
            )],
            &[],
            AdvancedContextBuildState {
                credibility: ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Disabled,
            },
            &AdvancedContextConfig::default(),
        );

        assert_eq!(projection.retrieval_state, RetrievalState::Degraded);
        assert_eq!(projection.retrieval_index_state, RetrievalIndexState::Ready);
        assert_eq!(projection.selected_evidence.len(), 1);
        assert_eq!(projection.selected_evidence[0].source_ref, "src/lib.rs");
        assert!(projection.terminal_reason.as_deref().is_some_and(|reason| {
            reason.contains("SQLite retrieval returned no stronger local match")
        }));
    }

    #[test]
    fn build_advanced_context_projection_marks_local_semantic_policy_as_skipped() {
        let workspace = temp_workspace("boundline-advanced-context-semantic-policy");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .unwrap();

        let projection = build_advanced_context_projection(
            "fix the add implementation",
            &workspace,
            &[context_input(
                ContextInputKind::WorkspaceFile,
                "src/lib.rs",
                "workspace_scan",
                "selected workspace target",
                true,
            )],
            &[],
            AdvancedContextBuildState {
                credibility: ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Local,
            },
            &AdvancedContextConfig::default(),
        );

        assert_eq!(projection.semantic_policy_state, SemanticPolicyState::Local);
        assert!(matches!(
            projection.hybrid_outcome,
            HybridOutcome::Skipped | HybridOutcome::BaselineOnly
        ));
        assert!(matches!(
            projection.semantic_capability_state,
            SemanticCapabilityState::Ready
                | SemanticCapabilityState::Unavailable
                | SemanticCapabilityState::Unsupported
                | SemanticCapabilityState::Degraded
        ));
        assert!(projection.terminal_reason.as_deref().is_some_and(|reason| {
            reason.contains("semantic acceleration")
                || reason.contains("semantic refresh")
                || reason.contains("sqlite-vec")
        }));
    }

    #[test]
    fn build_advanced_context_projection_expands_v1_candidates_with_local_semantic_matches() {
        let workspace = temp_workspace("boundline-advanced-context-semantic-expand");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "pub fn planner() -> bool { true }\n").unwrap();
        fs::write(
            workspace.join("src/semantic.rs"),
            "pub fn reconcileConfigState() -> bool { true }\n",
        )
        .unwrap();

        let projection = build_advanced_context_projection_with_vector_state(
            "planner reconcile configuration state",
            &workspace,
            &[
                context_input(
                    ContextInputKind::WorkspaceFile,
                    "src/lib.rs",
                    "workspace_scan",
                    "selected bounded implementation surface",
                    true,
                ),
                context_input(
                    ContextInputKind::WorkspaceFile,
                    "src/semantic.rs",
                    "workspace_scan",
                    "related implementation surface",
                    false,
                ),
            ],
            &[],
            AdvancedContextBuildState {
                credibility: ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Local,
            },
            &AdvancedContextConfig::default(),
            Some(VectorExtensionState::Ready),
        );

        assert_eq!(projection.semantic_policy_state, SemanticPolicyState::Local);
        assert_eq!(projection.semantic_capability_state, SemanticCapabilityState::Ready);
        assert_eq!(projection.hybrid_outcome, HybridOutcome::Expanded);
        assert_eq!(projection.retrieval_state, RetrievalState::Selected);
        assert!(projection.selected_evidence.iter().any(|candidate| {
            candidate.source_ref == "src/lib.rs"
                && candidate.match_origin == RetrievalMatchOrigin::Fts
        }));
        assert!(projection.selected_evidence.iter().any(|candidate| {
            candidate.source_ref == "src/semantic.rs"
                && candidate.match_origin == RetrievalMatchOrigin::SemanticExpand
                && candidate.semantic_score.is_some()
        }));
        assert!(
            projection
                .terminal_reason
                .as_deref()
                .is_some_and(|reason| { reason.contains("expanded the V1 candidate set") })
        );
    }

    #[test]
    fn collect_retrieval_documents_classifies_backing_and_staleness() {
        let workspace = temp_workspace("boundline-advanced-context-documents");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "fn demo() {}\n").unwrap();

        let inputs = vec![
            context_input(
                ContextInputKind::WorkspaceFile,
                "src/lib.rs",
                "workspace_scan",
                "selected workspace target",
                true,
            ),
            context_input(
                ContextInputKind::WorkspaceFile,
                "src/lib.rs",
                "workspace_scan",
                "duplicate workspace target",
                false,
            ),
            context_input(
                ContextInputKind::RecentTrace,
                ".boundline/traces/latest.json",
                "latest_trace_ref",
                "persisted trace evidence",
                false,
            ),
            context_input(
                ContextInputKind::AuthoredBrief,
                "operator notes",
                "authored_input_summary",
                "captures the operator-authored task framing",
                false,
            ),
        ];

        let documents = collect_retrieval_documents(
            &workspace,
            &inputs,
            &["src/lib.rs".to_string()],
            ContextPackCredibility::Stale,
            Some("trace snapshot is stale"),
            SemanticAccelerationPolicyState::Disabled,
        );

        assert_eq!(documents.len(), 3);

        let workspace_document =
            documents.iter().find(|document| document.source_ref == "src/lib.rs").unwrap();
        assert_eq!(workspace_document.source_kind, RetrievalSourceKind::WorkspaceFile);
        assert_eq!(workspace_document.authority_rank, AuthorityRank::Structured);
        assert_eq!(workspace_document.compatibility_state, RetrievalCompatibilityState::Compatible);
        assert_eq!(workspace_document.staleness_state, RetrievalStalenessState::Fresh);
        assert!(workspace_document.content.contains("fn demo() {}"));
        assert!(workspace_document.metadata_json.contains("\"selected_target\":true"));

        let trace_document = documents
            .iter()
            .find(|document| document.source_ref == ".boundline/traces/latest.json")
            .unwrap();
        assert_eq!(trace_document.source_kind, RetrievalSourceKind::Trace);
        assert_eq!(
            trace_document.compatibility_state,
            RetrievalCompatibilityState::MissingMetadata
        );
        assert_eq!(trace_document.staleness_state, RetrievalStalenessState::Stale);

        let absolute_ref = workspace.join("src/lib.rs");
        assert_eq!(
            resolved_relative_path(&workspace, &absolute_ref.to_string_lossy()),
            Some("src/lib.rs".to_string())
        );
        assert_eq!(
            default_compatibility_state(ContextInputKind::DomainTemplate, false),
            RetrievalCompatibilityState::MissingMetadata
        );
        assert_eq!(
            staleness_state(
                ContextInputKind::CanonMemory,
                ContextPackCredibility::Stale,
                Some("refresh evidence")
            ),
            RetrievalStalenessState::Stale
        );
    }

    #[test]
    fn initialize_schema_creates_semantic_scaffold_and_records_vector_state() {
        let workspace = temp_workspace("boundline-advanced-context-semantic-schema");
        let connection = open_connection(&workspace).unwrap();

        initialize_schema(&connection, &workspace).unwrap();

        let vector_state: String = connection
            .query_row(
                "SELECT vector_extension_state FROM semantic_index_manifest WHERE manifest_id = ?1",
                rusqlite::params![SEMANTIC_INDEX_MANIFEST_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(vector_state, detect_vector_extension_state(&connection).as_str());

        let chunk_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM semantic_chunks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(chunk_count, 0);
    }

    #[test]
    fn vector_extension_state_helper_reports_ready_only_for_vec_modules() {
        assert_eq!(
            vector_extension_state_from_modules(&["fts5".to_string()]),
            crate::domain::context_intelligence::VectorExtensionState::Missing
        );
        assert_eq!(
            vector_extension_state_from_modules(&["vec0".to_string()]),
            crate::domain::context_intelligence::VectorExtensionState::Degraded
        );
        assert_eq!(
            vector_extension_state_from_modules(&["vec_each".to_string()]),
            crate::domain::context_intelligence::VectorExtensionState::Degraded
        );
    }

    #[test]
    fn structured_helpers_prioritize_selected_targets_and_deduplicate_promotions() {
        let documents = vec![
            RetrievalDocument {
                source_ref: "notes/operator.md".to_string(),
                source_kind: RetrievalSourceKind::ProjectMemory,
                authority_rank: AuthorityRank::WorkspaceOverride,
                provenance_summary: "notes".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                compatibility_reason: None,
                staleness_state: RetrievalStalenessState::Fresh,
                canon_artifact_class: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_boundary: None,
                canon_semantic_provenance_ref: None,
                canon_semantic_labels: Vec::new(),
                metadata_json: "{}".to_string(),
                content: "notes".to_string(),
            },
            RetrievalDocument {
                source_ref: "src/lib.rs".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                authority_rank: AuthorityRank::Structured,
                provenance_summary: "workspace".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                compatibility_reason: None,
                staleness_state: RetrievalStalenessState::Fresh,
                canon_artifact_class: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_boundary: None,
                canon_semantic_provenance_ref: None,
                canon_semantic_labels: Vec::new(),
                metadata_json: "{}".to_string(),
                content: "lib".to_string(),
            },
            RetrievalDocument {
                source_ref: ".canon/run.md".to_string(),
                source_kind: RetrievalSourceKind::CanonArtifact,
                authority_rank: AuthorityRank::Canon,
                provenance_summary: "canon".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                compatibility_reason: None,
                staleness_state: RetrievalStalenessState::Fresh,
                canon_artifact_class: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_boundary: None,
                canon_semantic_provenance_ref: None,
                canon_semantic_labels: Vec::new(),
                metadata_json: "{}".to_string(),
                content: "canon".to_string(),
            },
        ];

        let ordered = structured_fallback_refs(&documents, &["src/lib.rs".to_string()], 3);
        assert_eq!(ordered[0], "src/lib.rs");

        let promoted = promote_selected_target_refs(
            vec!["notes/operator.md".to_string(), "src/lib.rs".to_string()],
            &["src/lib.rs".to_string()],
            &documents,
            2,
        );
        assert_eq!(promoted, vec!["src/lib.rs".to_string(), "notes/operator.md".to_string()]);
    }

    #[test]
    fn derive_relationships_and_findings_cover_missing_test_and_stale_paths() {
        let workspace = temp_workspace("boundline-advanced-context-relationships");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/engine.rs"), "pub fn reconcile() {}\n").unwrap();

        let (relationships, findings) = derive_relationships_and_findings(
            &workspace,
            &[selected_candidate(RetrievalSourceKind::WorkspaceFile, "src/engine.rs")],
            ContextPackCredibility::Stale,
            Some("trace snapshot is stale"),
        );
        assert!(relationships.iter().any(|relationship| {
            relationship.relationship_kind == RelationshipKind::RequiresEvidence
                && relationship.subject_ref == "src/engine.rs"
        }));
        assert!(findings.iter().any(|finding| {
            finding.finding_kind == ImpactFindingKind::MissingTest
                && finding.subject_ref == "tests/engine.rs"
        }));
        assert!(findings.iter().any(|finding| {
            finding.finding_kind == ImpactFindingKind::EvidenceGap
                && finding.supporting_relationship_ids == vec!["relationship-1".to_string()]
        }));

        let (relationships, findings) = derive_relationships_and_findings(
            &workspace,
            &[selected_candidate(RetrievalSourceKind::ProjectMemory, "docs/operator-notes.md")],
            ContextPackCredibility::Stale,
            Some("governance memory is stale"),
        );
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_kind, RelationshipKind::SupportsRisk);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_kind, ImpactFindingKind::EvidenceGap);
        assert_eq!(
            findings[0].recommended_follow_up,
            "refresh bounded evidence: governance memory is stale"
        );
    }

    #[test]
    fn doctor_helper_checks_cover_consistency_states_and_aggregation() {
        let workspace = temp_workspace("boundline-advanced-context-doctor-checks");

        let consistent =
            manifest_consistency_check(&workspace, IndexDoctorConsistencyState::Consistent, true);
        assert_eq!(consistent.result, IndexDoctorStatus::Passed);

        let missing =
            manifest_consistency_check(&workspace, IndexDoctorConsistencyState::Missing, false);
        assert_eq!(missing.result, IndexDoctorStatus::Advisory);
        assert!(missing.detail.contains("manifest is missing"));

        let unobserved =
            manifest_consistency_check(&workspace, IndexDoctorConsistencyState::Missing, true);
        assert_eq!(unobserved.result, IndexDoctorStatus::Advisory);
        assert!(unobserved.detail.contains("could not be observed"));

        let corrupt =
            manifest_consistency_check(&workspace, IndexDoctorConsistencyState::Corrupt, true);
        assert_eq!(corrupt.result, IndexDoctorStatus::Failed);

        let invalid =
            manifest_consistency_check(&workspace, IndexDoctorConsistencyState::Invalid, true);
        assert_eq!(invalid.result, IndexDoctorStatus::Failed);

        let vector_consistent =
            vector_schema_check(&workspace, IndexDoctorConsistencyState::Consistent);
        assert_eq!(vector_consistent.result, IndexDoctorStatus::Passed);

        let vector_missing = vector_schema_check(&workspace, IndexDoctorConsistencyState::Missing);
        assert_eq!(vector_missing.result, IndexDoctorStatus::Advisory);

        let vector_corrupt = vector_schema_check(&workspace, IndexDoctorConsistencyState::Corrupt);
        assert_eq!(vector_corrupt.result, IndexDoctorStatus::Failed);

        let vector_invalid = vector_schema_check(&workspace, IndexDoctorConsistencyState::Invalid);
        assert_eq!(vector_invalid.result, IndexDoctorStatus::Failed);

        assert_eq!(
            aggregate_index_doctor_status(&[consistent, vector_missing]),
            IndexDoctorStatus::Advisory
        );
        assert_eq!(
            aggregate_index_doctor_status(&[vector_consistent, vector_corrupt]),
            IndexDoctorStatus::Failed
        );
        assert_eq!(
            aggregate_index_doctor_status(&[unobserved, invalid, vector_invalid]),
            IndexDoctorStatus::Failed
        );
    }

    #[test]
    fn doctor_helper_checks_cover_fully_consistent_statuses() {
        let workspace = temp_workspace("boundline-advanced-context-doctor-consistent");
        let connection = open_connection(&workspace).unwrap();
        initialize_schema(&connection, &workspace).unwrap();
        drop(connection);

        let manifest = valid_manifest_fixture(&workspace);
        fs::write(
            workspace.join(".boundline/context-intelligence/manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        assert_eq!(
            inspect_manifest_consistency(&workspace),
            (IndexDoctorConsistencyState::Consistent, true)
        );
        assert_eq!(
            inspect_vector_schema_consistency(&workspace),
            IndexDoctorConsistencyState::Consistent
        );
        assert_eq!(
            aggregate_index_doctor_status(&[
                manifest_consistency_check(
                    &workspace,
                    IndexDoctorConsistencyState::Consistent,
                    true
                ),
                vector_schema_check(&workspace, IndexDoctorConsistencyState::Consistent),
            ]),
            IndexDoctorStatus::Passed
        );
    }

    #[test]
    fn inspect_manifest_consistency_marks_missing_index_file_as_invalid() {
        let workspace = temp_workspace("boundline-advanced-context-doctor-missing-index");
        fs::create_dir_all(workspace.join(".boundline/context-intelligence")).unwrap();
        let manifest = valid_manifest_fixture(&workspace);
        fs::write(
            workspace.join(".boundline/context-intelligence/manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        assert_eq!(
            inspect_manifest_consistency(&workspace),
            (IndexDoctorConsistencyState::Invalid, true)
        );
    }

    #[test]
    fn lifecycle_helper_mappings_cover_remaining_recovery_and_env_variants() {
        let workspace = temp_workspace("boundline-advanced-context-helper-mappings");

        assert!(
            recommended_action_for_index_state(RetrievalIndexState::Degraded, &workspace)
                .contains("index doctor")
        );
        assert!(
            recommended_action_for_index_state(RetrievalIndexState::Building, &workspace)
                .contains("index status")
        );

        assert_eq!(
            index_stale_warning(IndexStaleReason::GitHeadChanged),
            "git HEAD changed since the last successful derived-index refresh"
        );
        assert_eq!(
            index_stale_warning(IndexStaleReason::Merge),
            "a merge marked the derived index stale"
        );
        assert_eq!(
            index_stale_warning(IndexStaleReason::PullWithMerge),
            "a pull-with-merge marked the derived index stale"
        );
        assert_eq!(
            index_stale_warning(IndexStaleReason::Rebase),
            "a rebase marked the derived index stale"
        );
        assert_eq!(
            index_stale_warning(IndexStaleReason::PostRewrite),
            "a post-rewrite event marked the derived index stale"
        );
        assert_eq!(
            index_stale_warning(IndexStaleReason::HookMarkedStale),
            "a Git freshness hook marked the derived index stale"
        );

        let original_trigger = std::env::var_os(SEMANTIC_INDEX_HOOK_TRIGGER_ENV);
        unsafe {
            std::env::set_var(
                SEMANTIC_INDEX_HOOK_TRIGGER_ENV,
                SEMANTIC_INDEX_HOOK_TRIGGER_POST_MERGE,
            );
        }
        assert_eq!(hook_trigger_stale_reason_from_env(), Some(IndexStaleReason::Merge));
        unsafe {
            std::env::set_var(
                SEMANTIC_INDEX_HOOK_TRIGGER_ENV,
                SEMANTIC_INDEX_HOOK_TRIGGER_POST_REWRITE,
            );
        }
        assert_eq!(hook_trigger_stale_reason_from_env(), Some(IndexStaleReason::PostRewrite));
        unsafe {
            std::env::set_var(SEMANTIC_INDEX_HOOK_TRIGGER_ENV, "unexpected");
        }
        assert_eq!(hook_trigger_stale_reason_from_env(), None);
        restore_semantic_index_hook_trigger(original_trigger.as_ref());

        let mut incompatible_manifest = valid_manifest_fixture(&workspace);
        incompatible_manifest.workspace_root = format!("{}/other", workspace.display());
        assert_eq!(
            derive_manifest_index_status(None, &incompatible_manifest, &workspace),
            RetrievalIndexState::Incompatible
        );

        let mut degraded_manifest = valid_manifest_fixture(&workspace);
        degraded_manifest.sqlite_vec_state = VectorExtensionState::Degraded;
        assert_eq!(
            derive_manifest_index_status(None, &degraded_manifest, &workspace),
            RetrievalIndexState::Degraded
        );

        let mut corrupt_manifest = valid_manifest_fixture(&workspace);
        corrupt_manifest.sqlite_vec_state = VectorExtensionState::Corrupt;
        assert_eq!(
            derive_manifest_index_status(None, &corrupt_manifest, &workspace),
            RetrievalIndexState::Corrupt
        );
        let mut corrupt_fts_manifest = valid_manifest_fixture(&workspace);
        corrupt_fts_manifest.fts5_state = ManifestFtsState::Corrupt;
        assert_eq!(
            derive_manifest_index_status(None, &corrupt_fts_manifest, &workspace),
            RetrievalIndexState::Corrupt
        );

        assert_eq!(
            semantic_engine_for_trace(
                SemanticPolicyState::Disabled,
                SemanticCapabilityState::Ready,
            ),
            SemanticEngine::Disabled
        );
        assert_eq!(
            semantic_engine_for_trace(SemanticPolicyState::Local, SemanticCapabilityState::Corrupt,),
            SemanticEngine::BaselineJson
        );
    }

    #[test]
    fn semantic_helpers_cover_delete_sync_empty_query_and_distance_guard_paths() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute("CREATE TABLE semantic_chunks (chunk_id TEXT PRIMARY KEY)", []).unwrap();
        connection
            .execute(
                &format!(
                    "CREATE TABLE {SEMANTIC_VECTORS_TABLE_NAME} (chunk_id TEXT PRIMARY KEY, embedding TEXT NOT NULL)"
                ),
                [],
            )
            .unwrap();

        let make_row = |chunk_id: &str, embedding_state: &str| SemanticChunkRefreshRow {
            chunk_id: chunk_id.to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile.as_str().to_string(),
            source_ref: format!("src/{chunk_id}.rs"),
            provenance_boundary: RetrievalSourceKind::WorkspaceFile.as_str().to_string(),
            provenance_ref: format!("src/{chunk_id}.rs"),
            content_hash: "sha256:test".to_string(),
            embedding_state: embedding_state.to_string(),
            embedding_dimensions: 1536,
            canon_semantic_contract_line: None,
            semantic_labels_json: "[]".to_string(),
            semantic_schema_line: "boundline.semantic_chunk.v1".to_string(),
            chunk_text: "chunk text".to_string(),
            embedding_payload_json: "[0.1,0.2]".to_string(),
        };

        let obsolete = make_row("obsolete", SemanticChunkState::Ready.as_str());
        connection
            .execute(
                "INSERT INTO semantic_chunks (chunk_id) VALUES (?1)",
                rusqlite::params![obsolete.chunk_id],
            )
            .unwrap();
        connection
            .execute(
                &format!("INSERT INTO {SEMANTIC_VECTORS_TABLE_NAME} (chunk_id, embedding) VALUES (?1, ?2)"),
                rusqlite::params![obsolete.chunk_id, "[0.1,0.2]"],
            )
            .unwrap();

        let existing_chunks = BTreeMap::from([(obsolete.chunk_id.clone(), obsolete)]);
        delete_missing_semantic_chunks(&connection, &existing_chunks, &BTreeSet::new(), true)
            .unwrap();

        let remaining_chunk_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM semantic_chunks", [], |row| row.get(0))
            .unwrap();
        let remaining_vector_count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {SEMANTIC_VECTORS_TABLE_NAME}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(remaining_chunk_count, 0);
        assert_eq!(remaining_vector_count, 0);

        let blocked = make_row("blocked", SemanticChunkState::Blocked.as_str());
        sync_semantic_vector_row(&connection, false, None, &blocked).unwrap();
        connection
            .execute(
                &format!("INSERT INTO {SEMANTIC_VECTORS_TABLE_NAME} (chunk_id, embedding) VALUES (?1, ?2)"),
                rusqlite::params![blocked.chunk_id, "[0.1,0.2]"],
            )
            .unwrap();
        sync_semantic_vector_row(&connection, true, None, &blocked).unwrap();

        let blocked_vector_count: i64 = connection
            .query_row(
                &format!("SELECT COUNT(*) FROM {SEMANTIC_VECTORS_TABLE_NAME} WHERE chunk_id = ?1"),
                rusqlite::params![blocked.chunk_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(blocked_vector_count, 0);

        let empty_query_result = query_semantic_matches(
            &connection,
            "refresh semantic state",
            &[],
            0,
            VectorExtensionState::Ready,
        )
        .unwrap();
        assert!(empty_query_result.matches.is_empty());
        assert!(!empty_query_result.vector_query_attempted);
        assert_eq!(empty_query_result.vector_chunk_candidates_returned, 0);

        assert!(semantic_score_from_vec_distance(f64::NAN).is_none());
        assert!(filtered_semantic_score_from_vec_distance(f64::NAN).is_none());
        assert!(filtered_semantic_score_from_vec_distance(10.0).is_none());
        assert!(filtered_semantic_score_from_vec_distance(0.0).is_some());
    }

    #[test]
    fn query_semantic_matches_from_payload_skips_nonfinite_scores() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute(
                &format!(
                    "CREATE TABLE {SEMANTIC_CHUNKS_TABLE_NAME} (source_ref TEXT NOT NULL, provenance_ref TEXT NOT NULL, canon_semantic_contract_line TEXT, embedding_payload_json TEXT NOT NULL, embedding_state TEXT NOT NULL)"
                ),
                [],
            )
            .unwrap();
        connection
            .execute(
                &format!(
                    "INSERT INTO {SEMANTIC_CHUNKS_TABLE_NAME} (source_ref, provenance_ref, canon_semantic_contract_line, embedding_payload_json, embedding_state) VALUES (?1, ?2, ?3, ?4, ?5)"
                ),
                rusqlite::params![
                    "src/lib.rs",
                    "src/lib.rs",
                    Option::<String>::None,
                    "[1.0]",
                    SemanticChunkState::Ready.as_str(),
                ],
            )
            .unwrap();

        let result = query_semantic_matches_from_payload(&connection, &[f64::NAN], 4).unwrap();
        assert!(result.matches.is_empty());
    }

    #[test]
    fn query_semantic_matches_from_payload_filters_low_scores_and_keeps_best_duplicate() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute(
                &format!(
                    "CREATE TABLE {SEMANTIC_CHUNKS_TABLE_NAME} (source_ref TEXT NOT NULL, provenance_ref TEXT NOT NULL, canon_semantic_contract_line TEXT, embedding_payload_json TEXT NOT NULL, embedding_state TEXT NOT NULL)"
                ),
                [],
            )
            .unwrap();

        for (source_ref, embedding_payload_json) in [
            ("src/best.rs", "[0.9]"),
            ("src/second.rs", "[0.8]"),
            ("src/best.rs", "[0.4]"),
            ("src/low.rs", "[0.1]"),
        ] {
            connection
                .execute(
                    &format!(
                        "INSERT INTO {SEMANTIC_CHUNKS_TABLE_NAME} (source_ref, provenance_ref, canon_semantic_contract_line, embedding_payload_json, embedding_state) VALUES (?1, ?2, ?3, ?4, ?5)"
                    ),
                    rusqlite::params![
                        source_ref,
                        source_ref,
                        Option::<String>::None,
                        embedding_payload_json,
                        SemanticChunkState::Ready.as_str(),
                    ],
                )
                .unwrap();
        }

        let result = query_semantic_matches_from_payload(&connection, &[1.0], 4).unwrap();
        assert_eq!(result.matches.len(), 2);
        assert_eq!(result.matches[0].source_ref, "src/best.rs");
        assert!((result.matches[0].semantic_score.as_raw() - 0.9).abs() < 0.001);
        assert_eq!(result.matches[1].source_ref, "src/second.rs");
    }

    #[test]
    fn read_derived_index_manifest_reports_directory_read_errors() {
        let workspace = temp_workspace("boundline-advanced-context-manifest-read-error");
        let manifest_path = workspace.join(".boundline/context-intelligence/manifest.json");
        fs::create_dir_all(&manifest_path).unwrap();

        let error = read_derived_index_manifest(&workspace).unwrap_err();
        assert!(matches!(
            error,
            ContextIntelligenceBuildError::ReadManifest(message) if !message.is_empty()
        ));
    }

    #[test]
    fn upsert_semantic_manifest_row_reports_missing_manifest_table() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let workspace = temp_workspace("boundline-advanced-context-manifest-upsert-error");

        let error = upsert_semantic_manifest_row(
            &connection,
            &valid_manifest_fixture(&workspace),
            SEMANTIC_REFRESH_PENDING_REASON,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ContextIntelligenceBuildError::InitializeSemanticIndex(message)
                if message.contains("semantic_index_manifest") || !message.is_empty()
        ));
    }

    #[test]
    fn inspect_manifest_and_vector_consistency_cover_missing_corrupt_and_invalid_paths() {
        let missing_workspace = temp_workspace("boundline-advanced-context-doctor-missing");
        assert_eq!(
            inspect_manifest_consistency(&missing_workspace),
            (IndexDoctorConsistencyState::Missing, false)
        );
        assert_eq!(
            inspect_vector_schema_consistency(&missing_workspace),
            IndexDoctorConsistencyState::Missing
        );

        let corrupt_workspace = temp_workspace("boundline-advanced-context-doctor-corrupt");
        fs::create_dir_all(corrupt_workspace.join(".boundline/context-intelligence")).unwrap();
        fs::write(
            corrupt_workspace.join(".boundline/context-intelligence/manifest.json"),
            "{not-json}",
        )
        .unwrap();
        assert_eq!(
            inspect_manifest_consistency(&corrupt_workspace),
            (IndexDoctorConsistencyState::Corrupt, true)
        );

        let invalid_workspace = temp_workspace("boundline-advanced-context-doctor-invalid");
        let connection = open_connection(&invalid_workspace).unwrap();
        connection.execute("CREATE TABLE placeholder (id INTEGER PRIMARY KEY)", []).unwrap();
        drop(connection);

        let mut manifest = valid_manifest_fixture(&invalid_workspace);
        manifest.workspace_root = format!("{}/other-workspace", invalid_workspace.display());
        fs::write(
            invalid_workspace.join(".boundline/context-intelligence/manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(
            inspect_manifest_consistency(&invalid_workspace),
            (IndexDoctorConsistencyState::Invalid, true)
        );
        assert_eq!(
            inspect_vector_schema_consistency(&invalid_workspace),
            IndexDoctorConsistencyState::Invalid
        );
    }

    #[test]
    fn vector_schema_consistency_from_tables_maps_corrupt_invalid_and_consistent_states() {
        assert_eq!(
            vector_schema_consistency_from_tables(
                Err(ContextIntelligenceBuildError::QueryIndex(
                    "semantic chunks failure".to_string()
                )),
                Ok(false),
                VectorExtensionState::Ready,
            ),
            IndexDoctorConsistencyState::Corrupt
        );
        assert_eq!(
            vector_schema_consistency_from_tables(
                Ok(false),
                Ok(false),
                VectorExtensionState::Ready,
            ),
            IndexDoctorConsistencyState::Invalid
        );
        assert_eq!(
            vector_schema_consistency_from_tables(
                Ok(true),
                Err(ContextIntelligenceBuildError::QueryIndex("vector table failure".to_string())),
                VectorExtensionState::Ready,
            ),
            IndexDoctorConsistencyState::Corrupt
        );
        assert_eq!(
            vector_schema_consistency_from_tables(Ok(true), Ok(false), VectorExtensionState::Ready,),
            IndexDoctorConsistencyState::Invalid
        );
        assert_eq!(
            vector_schema_consistency_from_tables(
                Ok(true),
                Ok(false),
                VectorExtensionState::Missing,
            ),
            IndexDoctorConsistencyState::Consistent
        );
    }

    #[cfg(unix)]
    #[test]
    fn inspect_vector_schema_consistency_marks_open_errors_corrupt()
    -> Result<(), Box<dyn std::error::Error>> {
        let workspace = temp_workspace("boundline-advanced-context-doctor-open-error");
        let state_dir = workspace.join(".boundline/context-intelligence");
        fs::create_dir_all(&state_dir)?;
        let index_path = state_dir.join("retrieval-index.sqlite3");
        fs::write(&index_path, [])?;
        fs::set_permissions(&index_path, fs::Permissions::from_mode(0o000))?;

        let state = inspect_vector_schema_consistency(&workspace);

        fs::set_permissions(&index_path, fs::Permissions::from_mode(0o600))?;
        assert_eq!(state, IndexDoctorConsistencyState::Corrupt);
        Ok(())
    }

    #[test]
    fn initialize_semantic_vector_table_for_state_skips_non_ready_vector_capability()
    -> Result<(), Box<dyn std::error::Error>> {
        let connection = rusqlite::Connection::open_in_memory()?;

        initialize_semantic_vector_table_for_state(&connection, VectorExtensionState::Missing)?;

        assert!(!super::table_exists(&connection, SEMANTIC_VECTORS_TABLE_NAME)?);
        Ok(())
    }

    #[cfg(feature = "sqlite-vec")]
    #[test]
    fn inspect_vector_schema_consistency_marks_ready_without_vector_table_invalid()
    -> Result<(), Box<dyn std::error::Error>> {
        let workspace = temp_workspace("boundline-advanced-context-doctor-vector-table-missing");
        let connection = open_connection(&workspace)?;
        connection.execute("CREATE TABLE semantic_chunks (chunk_id TEXT PRIMARY KEY)", [])?;

        assert_eq!(detect_vector_extension_state(&connection), VectorExtensionState::Ready);
        drop(connection);

        assert_eq!(
            inspect_vector_schema_consistency(&workspace),
            IndexDoctorConsistencyState::Invalid
        );
        Ok(())
    }

    #[rustfmt::skip]
    fn restore_semantic_index_hook_trigger(value: Option<&std::ffi::OsString>) { unsafe { match value { Some(value) => std::env::set_var(SEMANTIC_INDEX_HOOK_TRIGGER_ENV, value), None => std::env::remove_var(SEMANTIC_INDEX_HOOK_TRIGGER_ENV), } } }

    #[test]
    #[rustfmt::skip]
    fn refresh_and_query_index_persists_manifest_through_payload_fallback() -> Result<(), Box<dyn std::error::Error>> { let workspace = temp_workspace("boundline-advanced-context-refresh-and-query");
        let documents = vec![RetrievalDocument {
            source_ref: "src/lib.rs".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            authority_rank: AuthorityRank::Structured,
            provenance_summary: "workspace file".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            compatibility_reason: None,
            staleness_state: RetrievalStalenessState::Fresh,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            canon_semantic_labels: Vec::new(),
            metadata_json: "{}".to_string(),
            content: "fn reconcile_context() {}".to_string(),
        }];

        let result = refresh_and_query_index(&workspace, "reconcile context", &[], &documents, 4, 4, Some(VectorExtensionState::Missing))?;

        assert!(result.lexical_matches.iter().any(|entry| entry.source_ref == "src/lib.rs"));
        assert!(result.semantic_matches.is_empty());
        assert_eq!(result.vector_extension_state, VectorExtensionState::Missing);
        assert!(!result.vector_query_attempted);

        let manifest = read_derived_index_manifest(&workspace)?.ok_or("manifest missing")?;
        assert_eq!(manifest.index_status, RetrievalIndexState::SemanticUnavailable);
        assert_eq!(manifest.last_refresh_reason, Some(IndexRefreshReason::ManualRefresh));
        assert_eq!(manifest.chunk_count, 1);
        Ok(())
    }

    #[cfg(feature = "sqlite-vec")]
    #[test]
    #[rustfmt::skip]
    fn query_semantic_matches_from_vec0_keeps_best_duplicate_source() -> Result<(), Box<dyn std::error::Error>> { register_sqlite_vec_auto_extension()?;
        let connection = rusqlite::Connection::open_in_memory()?;

        initialize_semantic_vector_table_for_state(&connection, VectorExtensionState::Ready)?;
        connection.execute(
            "CREATE TABLE semantic_chunks (
                chunk_id TEXT PRIMARY KEY,
                source_ref TEXT NOT NULL,
                provenance_ref TEXT NOT NULL,
                canon_semantic_contract_line TEXT,
                embedding_state TEXT NOT NULL
            )",
            [],
        )?;

        let query_embedding = {
            let mut values = vec![0.0; 48];
            values[0] = 1.0;
            values
        };
        let best_embedding = serde_json::to_string(&query_embedding)?;
        let other_embedding = {
            let mut values = vec![0.0; 48];
            values[0] = 0.5;
            values[1] = 0.5;
            serde_json::to_string(&values)?
        };
        let duplicate_embedding = {
            let mut values = vec![0.0; 48];
            values[1] = 1.0;
            serde_json::to_string(&values)?
        };

        for (chunk_id, source_ref, embedding_payload_json) in [
            ("dup-best", "src/dup.rs", best_embedding.as_str()),
            ("other", "src/other.rs", other_embedding.as_str()),
            ("dup-low", "src/dup.rs", duplicate_embedding.as_str()),
        ] {
            connection.execute(
                "INSERT INTO semantic_chunks (
                    chunk_id,
                    source_ref,
                    provenance_ref,
                    canon_semantic_contract_line,
                    embedding_state
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    chunk_id,
                    source_ref,
                    source_ref,
                    Option::<String>::None,
                    SemanticChunkState::Ready.as_str(),
                ],
            )?;
            connection.execute(
                &format!(
                    "INSERT INTO {SEMANTIC_VECTORS_TABLE_NAME} (chunk_id, embedding) VALUES (?1, vec_f32(?2))"
                ),
                rusqlite::params![chunk_id, embedding_payload_json],
            )?;
        }

        let result = query_semantic_matches_from_vec0(&connection, &query_embedding, 4)?;
        assert!(result.vector_query_attempted);
        assert_eq!(result.vector_chunk_candidates_returned, 3);
        assert_eq!(result.matches.len(), 2);
        assert_eq!(result.matches[0].source_ref, "src/dup.rs");
        assert_eq!(result.matches[1].source_ref, "src/other.rs");
        assert!(
            result.matches[0].semantic_score.as_raw() > result.matches[1].semantic_score.as_raw()
        );
        Ok(())
    }

    #[test]
    fn build_advanced_context_projection_returns_insufficient_for_incompatible_canon_only_input() {
        let workspace = temp_workspace("boundline-advanced-context-canon-only");
        fs::create_dir_all(workspace.join(".canon")).unwrap();
        fs::write(workspace.join(".canon/contract.md"), "# Canon\n").unwrap();

        let projection = build_advanced_context_projection(
            "canon contract",
            &workspace,
            &[context_input(
                ContextInputKind::CanonArtifact,
                ".canon/contract.md",
                "canon_artifact",
                "canon artifact without semantic metadata",
                true,
            )],
            &[],
            AdvancedContextBuildState {
                credibility: ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Local,
            },
            &AdvancedContextConfig::default(),
        );

        assert_eq!(projection.retrieval_state, RetrievalState::Insufficient);
        assert!(projection.selected_evidence.is_empty());
        assert!(
            projection
                .rejected_candidates
                .iter()
                .any(|candidate| { candidate.source_ref == ".canon/contract.md" })
        );
    }

    #[cfg(feature = "sqlite-vec")]
    #[test]
    fn sqlite_vec_auto_extension_registration_is_idempotent() {
        register_sqlite_vec_auto_extension().unwrap();
        register_sqlite_vec_auto_extension().unwrap();
    }

    #[cfg(feature = "sqlite-vec")]
    #[test]
    fn sqlite_auto_extension_result_maps_success_and_failure() {
        assert!(sqlite_auto_extension_result(rusqlite::ffi::SQLITE_OK).is_ok());
        assert!(sqlite_auto_extension_result(1).is_err());
    }
}
