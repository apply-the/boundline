//! Builds bounded advanced-context retrieval projections using a local
//! SQLite + FTS5 index with structured fallback ordering.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use crate::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
    ImpactAnalysisFinding, ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus,
    RelationshipCredibilityState, RelationshipKind, RelationshipProjection,
    RetrievalCompatibilityState, RetrievalIndexState, RetrievalMatchOrigin, RetrievalMode,
    RetrievalScore, RetrievalSourceKind, RetrievalStalenessState, RetrievalState,
    RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticChunkState, SemanticPolicyState,
    SemanticTraceEventKind, SemanticTraceRecord, VectorExtensionState,
};
use crate::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use crate::domain::governance::{CanonSemanticEligibilityState, CanonSemanticProvenanceBoundary};
use crate::domain::project_memory::{
    CompatibilityOutcome, PromotionStateView, read_canon_semantic_artifact_surface,
};

const BOUNDLINE_STATE_DIRECTORY: &str = ".boundline";
const CONTEXT_INTELLIGENCE_DIRECTORY: &str = "context-intelligence";
const RETRIEVAL_INDEX_FILE_NAME: &str = "retrieval-index.sqlite3";
const SEMANTIC_INDEX_MANIFEST_ID: &str = "semantic-index-manifest";
const SEMANTIC_VECTOR_STATE_OVERRIDE_ENV: &str = "BOUNDLINE_SEMANTIC_VECTOR_STATE_OVERRIDE";
const SEMANTIC_VECTOR_STATE_READY_VALUE: &str = "ready";
const SEMANTIC_VECTOR_STATE_MISSING_VALUE: &str = "missing";
const SEMANTIC_VECTOR_STATE_STALE_VALUE: &str = "stale";
const SEMANTIC_VECTOR_STATE_UNSUPPORTED_VALUE: &str = "unsupported";
const SEMANTIC_REFRESH_PENDING_REASON: &str =
    "semantic acceleration scaffold initialized; no semantic refresh has completed yet";
const SQLITE_VEC_MODULE_NAME: &str = "vec0";
const SQLITE_VEC_EACH_MODULE_NAME: &str = "vec_each";
const SEMANTIC_ACCELERATION_MISSING_REASON: &str = "semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval";
const SEMANTIC_ACCELERATION_SKIPPED_REASON: &str = "semantic acceleration is configured locally but bounded retrieval did not reach semantic expansion";
const SEMANTIC_ACCELERATION_STALE_REASON: &str = "semantic acceleration is enabled but semantic vector state is stale; using baseline structured retrieval";
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
const MAX_SEMANTIC_CHUNK_BYTES: usize = 8 * 1024;
const SEMANTIC_EMBEDDING_DIMENSIONS: usize = 48;
const SEMANTIC_FEATURE_NGRAM_WIDTH: usize = 3;
const MIN_SEMANTIC_TOKEN_LENGTH: usize = 3;
const MIN_SEMANTIC_SIMILARITY_SCORE: f64 = 0.220;
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const MAX_INDEXED_BYTES: usize = 32 * 1024;
const MAX_QUERY_TERMS: usize = 8;
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
const SEMANTIC_TRACE_INDEX_REFRESHED_PREFIX: &str =
    "semantic index refreshed eligible local evidence set";
const SEMANTIC_TRACE_CHUNK_BLOCKED_PREFIX: &str = "semantic chunk was blocked:";
const SEMANTIC_TRACE_HYBRID_OUTCOME_PREFIX: &str = "semantic retrieval ended with retrieval state";

#[derive(Debug, Clone, Copy)]
pub struct AdvancedContextBuildState<'a> {
    pub credibility: ContextPackCredibility,
    pub staleness_reason: Option<&'a str>,
    pub semantic_policy: SemanticAccelerationPolicyState,
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
        return terminal_projection(
            query_id,
            policy,
            semantic_policy,
            RetrievalState::Insufficient,
            RetrievalIndexState::Insufficient,
            "advanced retrieval is disabled by configuration".to_string(),
        );
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
        return terminal_projection(
            query_id,
            policy,
            semantic_policy,
            RetrievalState::Insufficient,
            RetrievalIndexState::Insufficient,
            "no local documents were available for bounded advanced retrieval".to_string(),
        );
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
            terminal_reason: terminal_reason.as_deref(),
            selected_evidence: &[],
            rejected_candidates: &rejected_candidates,
        });

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
        };
    }

    let (
        base_selected_decisions,
        rejected_decisions,
        retrieval_state,
        retrieval_index_state,
        semantic_projection,
        base_terminal_reason,
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
            )
        }
        Ok(result) => {
            let fallback_refs = structured_fallback_refs(
                &queryable_documents,
                selected_targets,
                policy.budgets.evidence_limit,
            );
            if fallback_refs.is_empty() {
                return terminal_projection(
                    query_id,
                    policy,
                    semantic_policy,
                    RetrievalState::Insufficient,
                    default_index_state,
                    "no indexed evidence matched the bounded goal".to_string(),
                );
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
            )
        }
        Err(error) => {
            let fallback_refs = structured_fallback_refs(
                &documents,
                selected_targets,
                policy.budgets.evidence_limit,
            );
            if fallback_refs.is_empty() {
                return terminal_projection(
                    query_id,
                    policy,
                    semantic_policy,
                    RetrievalState::Unavailable,
                    RetrievalIndexState::Stale,
                    error.to_string(),
                );
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
        terminal_reason: terminal_reason.as_deref(),
        selected_evidence: &selected_evidence,
        rejected_candidates: &rejected_candidates,
    });

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
    };

    if projection.validate().is_err() {
        projection = terminal_projection(
            query_id,
            policy,
            semantic_policy,
            RetrievalState::Unavailable,
            RetrievalIndexState::Stale,
            "advanced retrieval projection validation failed after local indexing".to_string(),
        );
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

fn terminal_projection(
    query_id: String,
    policy: &AdvancedContextConfig,
    semantic_policy: SemanticAccelerationPolicyState,
    retrieval_state: RetrievalState,
    retrieval_index_state: RetrievalIndexState,
    terminal_reason: String,
) -> AdvancedContextProjection {
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
        terminal_reason: terminal_reason.as_deref(),
        selected_evidence: &[],
        rejected_candidates: &[],
    });

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

    let query = build_fts_query(goal_text, selected_targets);
    if query.is_empty() {
        return Ok(IndexQueryResult {
            lexical_matches: Vec::new(),
            semantic_matches: query_semantic_matches(
                &connection,
                goal_text,
                selected_targets,
                expansion_limit,
            )?,
            vector_extension_state,
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
        semantic_matches: query_semantic_matches(
            &connection,
            goal_text,
            selected_targets,
            expansion_limit,
        )?,
        vector_extension_state,
    })
}

fn open_connection(workspace_ref: &Path) -> Result<Connection, ContextIntelligenceBuildError> {
    let state_directory =
        workspace_ref.join(BOUNDLINE_STATE_DIRECTORY).join(CONTEXT_INTELLIGENCE_DIRECTORY);
    fs::create_dir_all(&state_directory)
        .map_err(|error| ContextIntelligenceBuildError::CreateStateDirectory(error.to_string()))?;
    let index_path = state_directory.join(RETRIEVAL_INDEX_FILE_NAME);
    Connection::open(index_path)
        .map_err(|error| ContextIntelligenceBuildError::OpenIndex(error.to_string()))
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

    let workspace_root = workspace_ref.to_string_lossy().into_owned();
    let vector_extension_state = detect_vector_extension_state(connection);
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
                workspace_root,
                vector_extension_state.as_str(),
                SEMANTIC_REFRESH_PENDING_REASON,
            ],
        )
        .map_err(|error| {
            ContextIntelligenceBuildError::InitializeSemanticIndex(error.to_string())
        })?;

    Ok(())
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
    if modules
        .iter()
        .any(|module| module == SQLITE_VEC_MODULE_NAME || module == SQLITE_VEC_EACH_MODULE_NAME)
    {
        VectorExtensionState::Ready
    } else {
        VectorExtensionState::Missing
    }
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

    transaction
        .execute("DELETE FROM semantic_chunks", [])
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;

    let chunk_state = semantic_chunk_state_for_vector_extension(vector_extension_state);
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

        transaction
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
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    semantic_chunk_id(&document.source_ref),
                    document.source_kind.as_str(),
                    document.source_ref,
                    document
                        .canon_semantic_provenance_boundary
                        .map(CanonSemanticProvenanceBoundary::as_str)
                        .unwrap_or_else(|| document.source_kind.as_str()),
                    document
                        .canon_semantic_provenance_ref
                        .clone()
                        .unwrap_or_else(|| document.source_ref.clone()),
                    semantic_content_hash(&chunk_text),
                    chunk_state.as_str(),
                    SEMANTIC_EMBEDDING_DIMENSIONS as i64,
                    document.canon_semantic_contract_line.clone(),
                    semantic_labels_json,
                    SEMANTIC_SCHEMA_LINE_V1,
                    chunk_text,
                    embedding_payload_json,
                ],
            )
            .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))?;
    }

    transaction
        .commit()
        .map_err(|error| ContextIntelligenceBuildError::RefreshIndex(error.to_string()))
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
    format!("semantic:{}", source_ref)
}

fn semantic_content_hash(value: &str) -> String {
    format!("fnv64:{:016x}", stable_hash(value))
}

fn query_semantic_matches(
    connection: &Connection,
    goal_text: &str,
    selected_targets: &[String],
    expansion_limit: usize,
) -> Result<Vec<SemanticMatchResult>, ContextIntelligenceBuildError> {
    if expansion_limit == 0 {
        return Ok(Vec::new());
    }

    let query_text = semantic_query_text(goal_text, selected_targets);
    let query_embedding = semantic_embedding(&query_text);
    if query_embedding.iter().all(|value| *value == 0.0) {
        return Ok(Vec::new());
    }

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
        let Some(score) = RetrievalScore::from_raw(cosine_similarity(&query_embedding, &embedding))
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
    Ok(matches)
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
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{
        AdvancedContextBuildState, AdvancedContextConfig, RelationshipKind, RetrievalDocument,
        SEMANTIC_INDEX_MANIFEST_ID, build_advanced_context_projection,
        build_advanced_context_projection_with_vector_state, collect_retrieval_documents,
        default_compatibility_state, derive_relationships_and_findings,
        detect_vector_extension_state, initialize_schema, open_connection,
        promote_selected_target_refs, resolved_relative_path, staleness_state,
        structured_fallback_refs, vector_extension_state_from_modules,
    };
    use crate::domain::configuration::SemanticAccelerationPolicyState;
    use crate::domain::context_intelligence::{
        AuthorityRank, CandidateSelectionState, HybridOutcome, ImpactFindingKind,
        RetrievalCompatibilityState, RetrievalIndexState, RetrievalMatchOrigin,
        RetrievalSourceKind, RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate,
        SemanticCapabilityState, SemanticPolicyState, VectorExtensionState,
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
        assert_eq!(projection.hybrid_outcome, HybridOutcome::Skipped);
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
            crate::domain::context_intelligence::VectorExtensionState::Ready
        );
        assert_eq!(
            vector_extension_state_from_modules(&["vec_each".to_string()]),
            crate::domain::context_intelligence::VectorExtensionState::Ready
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
}
