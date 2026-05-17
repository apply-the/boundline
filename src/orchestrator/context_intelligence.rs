//! Builds bounded advanced-context retrieval projections using a local
//! SQLite + FTS5 index with structured fallback ordering.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::configuration::AdvancedContextConfig;
use crate::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ImpactAnalysisFinding,
    ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus, RelationshipCredibilityState,
    RelationshipKind, RelationshipProjection, RetrievalCompatibilityState, RetrievalIndexState,
    RetrievalMode, RetrievalSourceKind, RetrievalStalenessState, RetrievalState,
    RetrievedEvidenceCandidate,
};
use crate::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};

const BOUNDLINE_STATE_DIRECTORY: &str = ".boundline";
const CONTEXT_INTELLIGENCE_DIRECTORY: &str = "context-intelligence";
const RETRIEVAL_INDEX_FILE_NAME: &str = "retrieval-index.sqlite3";
const MAX_INDEXED_BYTES: usize = 32 * 1024;
const MAX_QUERY_TERMS: usize = 8;

/// Builds the persisted advanced-context projection that planning, status,
/// and inspect surfaces share.
pub fn build_advanced_context_projection(
    goal_text: &str,
    workspace_ref: &Path,
    inputs: &[ContextInput],
    selected_targets: &[String],
    credibility: ContextPackCredibility,
    staleness_reason: Option<&str>,
    policy: &AdvancedContextConfig,
) -> AdvancedContextProjection {
    let query_id = Uuid::new_v4().to_string();

    if policy.retrieval_mode == RetrievalMode::Disabled {
        return terminal_projection(
            query_id,
            policy,
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
    );

    if documents.is_empty() {
        return terminal_projection(
            query_id,
            policy,
            RetrievalState::Insufficient,
            RetrievalIndexState::Insufficient,
            "no local documents were available for bounded advanced retrieval".to_string(),
        );
    }

    let default_index_state = default_index_state(credibility, &documents);
    let default_degraded_reason = credibility_degradation_reason(credibility, staleness_reason);

    let (selection_strategy, selected_refs, retrieval_index_state) = match refresh_and_query_index(
        workspace_ref,
        goal_text,
        selected_targets,
        &documents,
        policy.budgets.evidence_limit,
    ) {
        Ok(retrieved_refs) if !retrieved_refs.is_empty() => {
            (SelectionStrategy::Fts, retrieved_refs, default_index_state)
        }
        Ok(_) => {
            let fallback_refs = structured_fallback_refs(
                &documents,
                selected_targets,
                policy.budgets.evidence_limit,
            );
            if fallback_refs.is_empty() {
                return terminal_projection(
                    query_id,
                    policy,
                    RetrievalState::Insufficient,
                    default_index_state,
                    "no indexed evidence matched the bounded goal".to_string(),
                );
            }
            (
                    SelectionStrategy::StructuredFallback(
                        "SQLite retrieval returned no stronger local match; promoted structured bounded context evidence"
                            .to_string(),
                    ),
                    fallback_refs,
                    default_index_state,
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
                    RetrievalState::Unavailable,
                    RetrievalIndexState::Stale,
                    error.to_string(),
                );
            }
            (
                SelectionStrategy::StructuredFallback(format!(
                    "SQLite retrieval degraded to structured fallback: {error}"
                )),
                fallback_refs,
                RetrievalIndexState::Stale,
            )
        }
    };
    let selected_refs = promote_selected_target_refs(
        selected_refs,
        selected_targets,
        &documents,
        policy.budgets.evidence_limit,
    );

    let document_map = documents
        .iter()
        .map(|document| (document.source_ref.clone(), document))
        .collect::<BTreeMap<_, _>>();
    let selected_evidence = selected_refs
        .iter()
        .enumerate()
        .filter_map(|(index, source_ref)| {
            document_map.get(source_ref).map(|document| RetrievedEvidenceCandidate {
                candidate_id: format!("candidate-{}", index + 1),
                source_kind: document.source_kind,
                source_ref: document.source_ref.clone(),
                authority_rank: document.authority_rank,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: selection_strategy.selection_reason().to_string(),
                provenance_summary: document.provenance_summary.clone(),
                compatibility_state: document.compatibility_state,
                staleness_state: document.staleness_state,
            })
        })
        .collect::<Vec<_>>();

    let (relationships, impact_findings) = derive_relationships_and_findings(
        workspace_ref,
        &selected_evidence,
        credibility,
        staleness_reason,
    );

    let mut projection = AdvancedContextProjection {
        query_id: query_id.clone(),
        retrieval_mode: policy.retrieval_mode,
        retrieval_state: selection_strategy.retrieval_state(credibility),
        retrieval_index_state,
        budgets: policy.budgets.clone(),
        remote_policy_state: policy.remote_policy,
        used_remote: false,
        terminal_reason: selection_strategy
            .terminal_reason()
            .map(str::to_string)
            .or(default_degraded_reason),
        selected_evidence,
        rejected_candidates: Vec::new(),
        relationships,
        impact_findings,
    };

    if projection.validate().is_err() {
        projection = terminal_projection(
            query_id,
            policy,
            RetrievalState::Unavailable,
            RetrievalIndexState::Stale,
            "advanced retrieval projection validation failed after local indexing".to_string(),
        );
    }

    projection
}

#[derive(Debug, Clone)]
enum SelectionStrategy {
    Fts,
    StructuredFallback(String),
}

impl SelectionStrategy {
    fn selection_reason(&self) -> &'static str {
        match self {
            Self::Fts => "matched SQLite FTS evidence for the bounded goal",
            Self::StructuredFallback(_) => {
                "promoted bounded context evidence through structured fallback ordering"
            }
        }
    }

    fn terminal_reason(&self) -> Option<&str> {
        match self {
            Self::Fts => None,
            Self::StructuredFallback(reason) => Some(reason.as_str()),
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

#[derive(Debug, Clone)]
struct RetrievalDocument {
    source_ref: String,
    source_kind: RetrievalSourceKind,
    authority_rank: AuthorityRank,
    provenance_summary: String,
    compatibility_state: RetrievalCompatibilityState,
    staleness_state: RetrievalStalenessState,
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
}

#[derive(Debug, Error)]
enum ContextIntelligenceBuildError {
    #[error("failed to create advanced retrieval state directory: {0}")]
    CreateStateDirectory(String),
    #[error("failed to open advanced retrieval index: {0}")]
    OpenIndex(String),
    #[error("failed to initialize advanced retrieval index: {0}")]
    InitializeIndex(String),
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
        let compatibility_state = compatibility_state(input.kind, has_file_backing);
        let staleness_state = staleness_state(input.kind, credibility, staleness_reason);
        let metadata = RetrievalDocumentMetadata {
            source_kind,
            authority_rank,
            source: input.source.clone(),
            primary: input.primary,
            selected_target: selected_targets.iter().any(|target| target == &input.reference),
            relative_path,
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
            staleness_state,
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
    retrieval_state: RetrievalState,
    retrieval_index_state: RetrievalIndexState,
    terminal_reason: String,
) -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id,
        retrieval_mode: policy.retrieval_mode,
        retrieval_state,
        retrieval_index_state,
        budgets: policy.budgets.clone(),
        remote_policy_state: policy.remote_policy,
        used_remote: false,
        terminal_reason: Some(terminal_reason),
        selected_evidence: Vec::new(),
        rejected_candidates: Vec::new(),
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
) -> Result<Vec<String>, ContextIntelligenceBuildError> {
    let connection = open_connection(workspace_ref)?;
    initialize_schema(&connection)?;
    refresh_documents(&connection, documents)?;

    let query = build_fts_query(goal_text, selected_targets);
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let mut statement = connection
        .prepare(
            "SELECT documents.source_ref
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
        .query_map(params![query, evidence_limit as i64], |row| row.get::<_, String>(0))
        .map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?;

    let mut refs = Vec::new();
    for row in rows {
        refs.push(
            row.map_err(|error| ContextIntelligenceBuildError::QueryIndex(error.to_string()))?,
        );
    }
    Ok(refs)
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

fn initialize_schema(connection: &Connection) -> Result<(), ContextIntelligenceBuildError> {
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
        .map_err(|error| ContextIntelligenceBuildError::InitializeIndex(error.to_string()))
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

fn compatibility_state(
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
