//! Typed advanced-context intelligence models shared by planning, runtime,
//! session projections, and trace inspection.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::governance::CanonSemanticProvenanceBoundary;

/// Default refinement passes allowed for one retrieval query.
pub const DEFAULT_REFINEMENT_BUDGET: usize = 2;
/// Default stale-refresh retries allowed for one retrieval query.
pub const DEFAULT_REFRESH_BUDGET: usize = 1;
/// Default candidate discovery depth for one retrieval query.
pub const DEFAULT_DEPTH_LIMIT: usize = 12;
/// Default semantic-expansion limit for one retrieval query.
pub const DEFAULT_EXPANSION_LIMIT: usize = 8;
/// Default relationship traversal limit for one retrieval query.
pub const DEFAULT_TRAVERSAL_LIMIT: usize = 8;
/// Default selected-evidence limit for one retrieval query.
pub const DEFAULT_EVIDENCE_LIMIT: usize = 6;

/// Retrieval operating modes exposed by Boundline configuration and runtime output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    Disabled,
    Local,
    Remote,
}

impl RetrievalMode {
    /// Returns the stable serialization label for this retrieval mode.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }
}

/// Terminal or projected state of one advanced-context retrieval query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalState {
    Selected,
    Degraded,
    Insufficient,
    Exhausted,
    Unavailable,
}

impl RetrievalState {
    /// Returns the stable serialization label for this retrieval state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::Degraded => "degraded",
            Self::Insufficient => "insufficient",
            Self::Exhausted => "exhausted",
            Self::Unavailable => "unavailable",
        }
    }

    /// Returns true when the retrieval ended with selected evidence.
    pub const fn is_selected(self) -> bool {
        matches!(self, Self::Selected)
    }
}

/// State of the workspace-local retrieval index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalIndexState {
    Ready,
    Stale,
    Building,
    Insufficient,
}

impl RetrievalIndexState {
    /// Returns the stable serialization label for this index state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Building => "building",
            Self::Insufficient => "insufficient",
        }
    }
}

/// Effective semantic policy state for one advanced-context query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPolicyState {
    Disabled,
    Local,
}

impl SemanticPolicyState {
    /// Returns the stable serialization label for this semantic policy state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Local => "local",
        }
    }
}

/// Runtime semantic capability observed for one advanced-context query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCapabilityState {
    Ready,
    Unavailable,
    Unsupported,
    Degraded,
}

impl SemanticCapabilityState {
    /// Returns the stable serialization label for this semantic capability state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Degraded => "degraded",
        }
    }
}

/// Hybrid semantic outcome recorded for one advanced-context query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridOutcome {
    BaselineOnly,
    Expanded,
    Reranked,
    Skipped,
    Fallback,
}

impl HybridOutcome {
    /// Returns the stable serialization label for this hybrid outcome.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaselineOnly => "baseline_only",
            Self::Expanded => "expanded",
            Self::Reranked => "reranked",
            Self::Skipped => "skipped",
            Self::Fallback => "fallback",
        }
    }
}

/// Embedding lifecycle state for one semantic chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticChunkState {
    Pending,
    Ready,
    Stale,
    Blocked,
}

impl SemanticChunkState {
    /// Returns the stable serialization label for this semantic chunk state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Blocked => "blocked",
        }
    }
}

/// Availability state for the optional vector extension backing semantic retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorExtensionState {
    Ready,
    Missing,
    Unsupported,
    Stale,
}

impl VectorExtensionState {
    /// Returns the stable serialization label for this vector extension state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
            Self::Stale => "stale",
        }
    }
}

/// Authority rank assigned to one retrieved evidence candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRank {
    Structured,
    Canon,
    WorkspaceOverride,
    Semantic,
}

impl AuthorityRank {
    /// Returns the stable serialization label for this authority rank.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Structured => "structured",
            Self::Canon => "canon",
            Self::WorkspaceOverride => "workspace_override",
            Self::Semantic => "semantic",
        }
    }
}

/// Source families that may participate in advanced retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalSourceKind {
    WorkspaceFile,
    ProjectMemory,
    Trace,
    ReviewFinding,
    VerificationEvidence,
    CanonArtifact,
}

impl RetrievalSourceKind {
    /// Returns the stable serialization label for this source kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceFile => "workspace_file",
            Self::ProjectMemory => "project_memory",
            Self::Trace => "trace",
            Self::ReviewFinding => "review_finding",
            Self::VerificationEvidence => "verification_evidence",
            Self::CanonArtifact => "canon_artifact",
        }
    }
}

/// Selection state recorded for one evidence candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateSelectionState {
    Discovered,
    Selected,
    Downgraded,
    Rejected,
    Expired,
}

impl CandidateSelectionState {
    /// Returns true when a candidate outcome requires a visible reason string.
    pub const fn requires_reason(self) -> bool {
        matches!(self, Self::Selected | Self::Downgraded | Self::Rejected)
    }
}

/// Retrieval origin recorded for one evidence candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMatchOrigin {
    Fts,
    SemanticExpand,
    SemanticRerank,
    StructuredFallback,
}

impl RetrievalMatchOrigin {
    /// Returns the stable serialization label for this match origin.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fts => "fts",
            Self::SemanticExpand => "semantic_expand",
            Self::SemanticRerank => "semantic_rerank",
            Self::StructuredFallback => "structured_fallback",
        }
    }

    /// Returns true when semantic scoring metadata is required for this origin.
    pub const fn requires_semantic_score(self) -> bool {
        matches!(self, Self::SemanticExpand | Self::SemanticRerank)
    }
}

/// Typed semantic-trace events preserved inside one advanced-context projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTraceEventKind {
    CapabilityEvaluated,
    IndexRefreshed,
    ChunkBlocked,
    CandidateExpanded,
    CandidateReranked,
    CandidateRejected,
    CanonArtifactSkipped,
    FallbackApplied,
    HybridOutcomeRecorded,
}

impl SemanticTraceEventKind {
    /// Returns the stable serialization label for this semantic trace event kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CapabilityEvaluated => "capability_evaluated",
            Self::IndexRefreshed => "index_refreshed",
            Self::ChunkBlocked => "chunk_blocked",
            Self::CandidateExpanded => "candidate_expanded",
            Self::CandidateReranked => "candidate_reranked",
            Self::CandidateRejected => "candidate_rejected",
            Self::CanonArtifactSkipped => "canon_artifact_skipped",
            Self::FallbackApplied => "fallback_applied",
            Self::HybridOutcomeRecorded => "hybrid_outcome_recorded",
        }
    }

    /// Returns true when this event should point back to a candidate or document ref.
    pub const fn requires_candidate_ref(self) -> bool {
        matches!(
            self,
            Self::ChunkBlocked
                | Self::CandidateExpanded
                | Self::CandidateReranked
                | Self::CandidateRejected
                | Self::CanonArtifactSkipped
        )
    }
}

/// Stable score payload surfaced on advanced-context candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RetrievalScore {
    milli_value: i64,
}

impl RetrievalScore {
    /// Builds a stable score from a raw floating-point value.
    pub fn from_raw(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        Some(Self { milli_value: (value * 1000.0).round() as i64 })
    }

    /// Returns the raw floating-point representation for rendering.
    pub fn as_raw(self) -> f64 {
        self.milli_value as f64 / 1000.0
    }
}

/// Typed semantic trace record embedded in one advanced-context projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticTraceRecord {
    pub record_id: String,
    pub event_kind: SemanticTraceEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_origin: Option<RetrievalMatchOrigin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility_state: Option<RetrievalCompatibilityState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_score: Option<RetrievalScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_artifact_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_semantic_contract_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_semantic_provenance_boundary: Option<CanonSemanticProvenanceBoundary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_semantic_provenance_ref: Option<String>,
    pub reason: String,
}

impl SemanticTraceRecord {
    /// Validates one typed semantic trace record before it is persisted.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.record_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticTraceRecordId);
        }
        if self.event_kind.requires_candidate_ref()
            && self.candidate_ref.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(ContextIntelligenceError::MissingSemanticTraceCandidateRef {
                record_id: self.record_id.clone(),
                event_kind: self.event_kind,
            });
        }
        if self.reason.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticTraceReason {
                record_id: self.record_id.clone(),
            });
        }
        if self.match_origin.is_some_and(RetrievalMatchOrigin::requires_semantic_score)
            && self.semantic_score.is_none()
        {
            return Err(ContextIntelligenceError::MissingSemanticTraceScore {
                record_id: self.record_id.clone(),
                match_origin: self.match_origin.unwrap_or(RetrievalMatchOrigin::Fts),
            });
        }
        if self.canon_semantic_contract_line.is_some()
            ^ self.canon_semantic_provenance_boundary.is_some()
            || self.canon_semantic_contract_line.is_some()
                ^ self.canon_semantic_provenance_ref.is_some()
        {
            return Err(ContextIntelligenceError::IncompleteSemanticTraceCanonMetadata {
                record_id: self.record_id.clone(),
            });
        }
        Ok(())
    }
}

/// Compatibility outcome for Canon-backed or provider-backed evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalCompatibilityState {
    Compatible,
    UnsupportedContract,
    MissingMetadata,
    PolicyBlocked,
}

impl RetrievalCompatibilityState {
    /// Returns the stable serialization label for this compatibility state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::UnsupportedContract => "unsupported_contract",
            Self::MissingMetadata => "missing_metadata",
            Self::PolicyBlocked => "policy_blocked",
        }
    }
}

/// Freshness state recorded for one evidence candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStalenessState {
    Fresh,
    Stale,
}

impl RetrievalStalenessState {
    /// Returns the stable serialization label for this staleness state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
        }
    }
}

/// State of remote transmission policy for advanced retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteTransmissionPolicyState {
    Blocked,
    LocalOnly,
    RemoteAllowed,
}

impl RemoteTransmissionPolicyState {
    /// Returns the stable serialization label for this policy state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::LocalOnly => "local_only",
            Self::RemoteAllowed => "remote_allowed",
        }
    }
}

/// Relationship kinds projected from retrieved evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    AffectsSystem,
    AffectsDomain,
    ExercisesTest,
    ExposesContract,
    SuggestsReviewer,
    SupportsRisk,
    RequiresEvidence,
}

impl RelationshipKind {
    /// Returns the stable serialization label for this relationship kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AffectsSystem => "affects_system",
            Self::AffectsDomain => "affects_domain",
            Self::ExercisesTest => "exercises_test",
            Self::ExposesContract => "exposes_contract",
            Self::SuggestsReviewer => "suggests_reviewer",
            Self::SupportsRisk => "supports_risk",
            Self::RequiresEvidence => "requires_evidence",
        }
    }
}

/// Credibility assigned to one projected relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipCredibilityState {
    Credible,
    Tentative,
    Insufficient,
}

impl RelationshipCredibilityState {
    /// Returns the stable serialization label for this credibility state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credible => "credible",
            Self::Tentative => "tentative",
            Self::Insufficient => "insufficient",
        }
    }
}

/// Impact-finding kinds surfaced from retrieved evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactFindingKind {
    AffectedSystem,
    AffectedDomain,
    MissingTest,
    ContractExposure,
    ReviewerGap,
    EvidenceGap,
}

impl ImpactFindingKind {
    /// Returns the stable serialization label for this impact-finding kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AffectedSystem => "affected_system",
            Self::AffectedDomain => "affected_domain",
            Self::MissingTest => "missing_test",
            Self::ContractExposure => "contract_exposure",
            Self::ReviewerGap => "reviewer_gap",
            Self::EvidenceGap => "evidence_gap",
        }
    }
}

/// Lifecycle state recorded for one impact finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactFindingStatus {
    Open,
    Acknowledged,
    Resolved,
    Invalidated,
}

impl ImpactFindingStatus {
    /// Returns the stable serialization label for this finding status.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Acknowledged => "acknowledged",
            Self::Resolved => "resolved",
            Self::Invalidated => "invalidated",
        }
    }
}

/// Delivery-facing severity assigned to one impact finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactFindingSeverity {
    Low,
    Medium,
    High,
}

impl ImpactFindingSeverity {
    /// Returns the stable serialization label for this severity state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Typed retrieval budgets used to keep one advanced-context query bounded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalBudgets {
    pub refinement_budget: usize,
    pub refresh_budget: usize,
    pub depth_limit: usize,
    pub expansion_limit: usize,
    pub traversal_limit: usize,
    pub evidence_limit: usize,
}

impl Default for RetrievalBudgets {
    fn default() -> Self {
        Self {
            refinement_budget: DEFAULT_REFINEMENT_BUDGET,
            refresh_budget: DEFAULT_REFRESH_BUDGET,
            depth_limit: DEFAULT_DEPTH_LIMIT,
            expansion_limit: DEFAULT_EXPANSION_LIMIT,
            traversal_limit: DEFAULT_TRAVERSAL_LIMIT,
            evidence_limit: DEFAULT_EVIDENCE_LIMIT,
        }
    }
}

impl RetrievalBudgets {
    /// Validates that all retrieval budgets stay strictly positive.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        for (label, value) in [
            ("refinement_budget", self.refinement_budget),
            ("refresh_budget", self.refresh_budget),
            ("depth_limit", self.depth_limit),
            ("expansion_limit", self.expansion_limit),
            ("traversal_limit", self.traversal_limit),
            ("evidence_limit", self.evidence_limit),
        ] {
            if value == 0 {
                return Err(ContextIntelligenceError::InvalidBudget(label.to_string()));
            }
        }
        Ok(())
    }
}

/// One candidate surfaced or rejected during advanced-context retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievedEvidenceCandidate {
    pub candidate_id: String,
    pub source_kind: RetrievalSourceKind,
    pub source_ref: String,
    pub authority_rank: AuthorityRank,
    #[serde(default = "default_retrieval_match_origin")]
    pub match_origin: RetrievalMatchOrigin,
    pub selection_state: CandidateSelectionState,
    pub selection_reason: String,
    pub provenance_summary: String,
    pub compatibility_state: RetrievalCompatibilityState,
    pub staleness_state: RetrievalStalenessState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical_score: Option<RetrievalScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_score: Option<RetrievalScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_semantic_contract_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_semantic_provenance_ref: Option<String>,
}

impl RetrievedEvidenceCandidate {
    /// Validates one evidence candidate before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.candidate_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingCandidateId);
        }
        if self.source_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSourceRef {
                candidate_id: self.candidate_id.clone(),
            });
        }
        if self.selection_state.requires_reason() && self.selection_reason.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSelectionReason {
                candidate_id: self.candidate_id.clone(),
            });
        }
        if self.provenance_summary.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingProvenanceSummary {
                candidate_id: self.candidate_id.clone(),
            });
        }
        if self.match_origin.requires_semantic_score() && self.semantic_score.is_none() {
            return Err(ContextIntelligenceError::MissingSemanticScore {
                candidate_id: self.candidate_id.clone(),
                match_origin: self.match_origin,
            });
        }
        if self.canon_semantic_contract_line.is_some()
            ^ self.canon_semantic_provenance_ref.is_some()
        {
            return Err(ContextIntelligenceError::IncompleteCanonSemanticMetadata {
                candidate_id: self.candidate_id.clone(),
            });
        }
        Ok(())
    }
}

/// One relationship projected from selected evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipProjection {
    pub relationship_id: String,
    pub subject_ref: String,
    pub relationship_kind: RelationshipKind,
    pub credibility_state: RelationshipCredibilityState,
    pub explanation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_candidate_ids: Vec<String>,
}

impl RelationshipProjection {
    /// Validates one relationship projection before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.relationship_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingRelationshipId);
        }
        if self.subject_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingRelationshipSubject {
                relationship_id: self.relationship_id.clone(),
            });
        }
        if self.explanation.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingRelationshipExplanation {
                relationship_id: self.relationship_id.clone(),
            });
        }
        if self.supporting_candidate_ids.is_empty() {
            return Err(ContextIntelligenceError::MissingRelationshipSupport {
                relationship_id: self.relationship_id.clone(),
            });
        }
        Ok(())
    }
}

/// One impact finding inferred from projected relationships.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactAnalysisFinding {
    pub finding_id: String,
    pub finding_kind: ImpactFindingKind,
    pub subject_ref: String,
    pub status: ImpactFindingStatus,
    pub severity: ImpactFindingSeverity,
    pub recommended_follow_up: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_relationship_ids: Vec<String>,
}

impl ImpactAnalysisFinding {
    /// Validates one impact finding before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.finding_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingFindingId);
        }
        if self.subject_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingFindingSubject {
                finding_id: self.finding_id.clone(),
            });
        }
        if self.recommended_follow_up.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingFindingFollowUp {
                finding_id: self.finding_id.clone(),
            });
        }
        if self.supporting_relationship_ids.is_empty() {
            return Err(ContextIntelligenceError::MissingFindingSupport {
                finding_id: self.finding_id.clone(),
            });
        }
        Ok(())
    }
}

/// Persisted projection of one advanced-context retrieval decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedContextProjection {
    pub query_id: String,
    pub retrieval_mode: RetrievalMode,
    pub retrieval_state: RetrievalState,
    pub retrieval_index_state: RetrievalIndexState,
    #[serde(default = "default_semantic_policy_state")]
    pub semantic_policy_state: SemanticPolicyState,
    #[serde(default = "default_semantic_capability_state")]
    pub semantic_capability_state: SemanticCapabilityState,
    #[serde(default = "default_hybrid_outcome")]
    pub hybrid_outcome: HybridOutcome,
    #[serde(default)]
    pub budgets: RetrievalBudgets,
    pub remote_policy_state: RemoteTransmissionPolicyState,
    #[serde(default)]
    pub used_remote: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_evidence: Vec<RetrievedEvidenceCandidate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_candidates: Vec<RetrievedEvidenceCandidate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_trace_records: Vec<SemanticTraceRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipProjection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impact_findings: Vec<ImpactAnalysisFinding>,
}

impl AdvancedContextProjection {
    /// Returns the stable authority-order summary required by status and inspect.
    pub fn authority_order_text(&self) -> &'static str {
        "structured>canon>workspace_override>semantic"
    }

    /// Returns the selected evidence count recorded in this projection.
    pub fn selected_evidence_count(&self) -> usize {
        self.selected_evidence.len()
    }

    /// Returns the number of selected candidates that entered through a semantic path.
    pub fn semantic_selected_count(&self) -> usize {
        self.selected_evidence
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.match_origin,
                    RetrievalMatchOrigin::SemanticExpand | RetrievalMatchOrigin::SemanticRerank
                )
            })
            .count()
    }

    /// Returns the number of rejected or skipped candidates recorded for semantic evaluation.
    pub fn semantic_rejected_count(&self) -> usize {
        self.rejected_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.match_origin,
                    RetrievalMatchOrigin::SemanticExpand | RetrievalMatchOrigin::SemanticRerank
                )
            })
            .count()
    }

    /// Returns the impact-finding count recorded in this projection.
    pub fn impact_finding_count(&self) -> usize {
        self.impact_findings.len()
    }

    fn validate_semantic_consistency(&self) -> Result<(), ContextIntelligenceError> {
        if self.semantic_policy_state == SemanticPolicyState::Disabled
            && matches!(self.hybrid_outcome, HybridOutcome::Expanded | HybridOutcome::Reranked)
        {
            return Err(ContextIntelligenceError::InvalidSemanticHybridOutcome {
                policy_state: self.semantic_policy_state,
                hybrid_outcome: self.hybrid_outcome,
            });
        }
        if matches!(self.hybrid_outcome, HybridOutcome::Skipped | HybridOutcome::Fallback)
            && self.terminal_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(ContextIntelligenceError::MissingSemanticTerminalReason {
                hybrid_outcome: self.hybrid_outcome,
            });
        }
        Ok(())
    }

    fn validate_retrieval_consistency(&self) -> Result<(), ContextIntelligenceError> {
        if self.retrieval_state.is_selected() && self.selected_evidence.is_empty() {
            return Err(ContextIntelligenceError::MissingSelectedEvidence);
        }
        if !self.retrieval_state.is_selected()
            && self.terminal_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(ContextIntelligenceError::MissingTerminalReason);
        }
        Ok(())
    }

    fn validate_remote_consistency(&self) -> Result<(), ContextIntelligenceError> {
        if self.retrieval_mode != RetrievalMode::Remote && self.used_remote {
            return Err(ContextIntelligenceError::UnexpectedRemoteUsage);
        }
        if self.remote_policy_state != RemoteTransmissionPolicyState::RemoteAllowed
            && self.used_remote
        {
            return Err(ContextIntelligenceError::BlockedRemoteUsage);
        }
        Ok(())
    }

    /// Validates that the projection remains internally consistent.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.query_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingQueryId);
        }
        self.budgets.validate()?;
        self.validate_semantic_consistency()?;
        self.validate_retrieval_consistency()?;
        self.validate_remote_consistency()?;
        for candidate in &self.selected_evidence {
            candidate.validate()?;
        }
        for candidate in &self.rejected_candidates {
            candidate.validate()?;
        }
        for record in &self.semantic_trace_records {
            record.validate()?;
        }
        for relationship in &self.relationships {
            relationship.validate()?;
        }
        for finding in &self.impact_findings {
            finding.validate()?;
        }
        Ok(())
    }
}

const fn default_semantic_policy_state() -> SemanticPolicyState {
    SemanticPolicyState::Disabled
}

const fn default_semantic_capability_state() -> SemanticCapabilityState {
    SemanticCapabilityState::Unsupported
}

const fn default_hybrid_outcome() -> HybridOutcome {
    HybridOutcome::BaselineOnly
}

const fn default_retrieval_match_origin() -> RetrievalMatchOrigin {
    RetrievalMatchOrigin::Fts
}

/// Errors returned while validating advanced-context projections.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContextIntelligenceError {
    #[error("advanced context projection requires a query id")]
    MissingQueryId,
    #[error("advanced context projection requires selected evidence for a selected state")]
    MissingSelectedEvidence,
    #[error("advanced context projection requires a terminal reason for non-selected states")]
    MissingTerminalReason,
    #[error(
        "advanced context projection cannot report semantic hybrid outcome `{}` while semantic policy is `{}`",
        hybrid_outcome.as_str(),
        policy_state.as_str()
    )]
    InvalidSemanticHybridOutcome {
        policy_state: SemanticPolicyState,
        hybrid_outcome: HybridOutcome,
    },
    #[error(
        "advanced context projection requires a terminal reason when semantic hybrid outcome is `{}`",
        hybrid_outcome.as_str()
    )]
    MissingSemanticTerminalReason { hybrid_outcome: HybridOutcome },
    #[error("advanced context projection reported remote usage outside remote mode")]
    UnexpectedRemoteUsage,
    #[error("advanced context projection reported remote usage while policy blocked transmission")]
    BlockedRemoteUsage,
    #[error("retrieval budget `{0}` must be greater than zero")]
    InvalidBudget(String),
    #[error("retrieved evidence candidate requires a candidate id")]
    MissingCandidateId,
    #[error("retrieved evidence candidate `{candidate_id}` requires a source ref")]
    MissingSourceRef { candidate_id: String },
    #[error("retrieved evidence candidate `{candidate_id}` requires a selection reason")]
    MissingSelectionReason { candidate_id: String },
    #[error("retrieved evidence candidate `{candidate_id}` requires a provenance summary")]
    MissingProvenanceSummary { candidate_id: String },
    #[error(
        "retrieved evidence candidate `{candidate_id}` requires a semantic score for match origin `{}`",
        match_origin.as_str()
    )]
    MissingSemanticScore { candidate_id: String, match_origin: RetrievalMatchOrigin },
    #[error(
        "retrieved evidence candidate `{candidate_id}` reported incomplete Canon semantic metadata"
    )]
    IncompleteCanonSemanticMetadata { candidate_id: String },
    #[error("semantic trace record requires an id")]
    MissingSemanticTraceRecordId,
    #[error(
        "semantic trace record `{record_id}` requires a candidate ref for event kind `{}`",
        event_kind.as_str()
    )]
    MissingSemanticTraceCandidateRef { record_id: String, event_kind: SemanticTraceEventKind },
    #[error("semantic trace record `{record_id}` requires a reason")]
    MissingSemanticTraceReason { record_id: String },
    #[error(
        "semantic trace record `{record_id}` requires a semantic score for match origin `{}`",
        match_origin.as_str()
    )]
    MissingSemanticTraceScore { record_id: String, match_origin: RetrievalMatchOrigin },
    #[error("semantic trace record `{record_id}` reported incomplete Canon semantic metadata")]
    IncompleteSemanticTraceCanonMetadata { record_id: String },
    #[error("relationship projection requires a relationship id")]
    MissingRelationshipId,
    #[error("relationship `{relationship_id}` requires a subject ref")]
    MissingRelationshipSubject { relationship_id: String },
    #[error("relationship `{relationship_id}` requires an explanation")]
    MissingRelationshipExplanation { relationship_id: String },
    #[error("relationship `{relationship_id}` requires supporting evidence")]
    MissingRelationshipSupport { relationship_id: String },
    #[error("impact finding requires a finding id")]
    MissingFindingId,
    #[error("impact finding `{finding_id}` requires a subject ref")]
    MissingFindingSubject { finding_id: String },
    #[error("impact finding `{finding_id}` requires a follow-up action")]
    MissingFindingFollowUp { finding_id: String },
    #[error("impact finding `{finding_id}` requires supporting relationships")]
    MissingFindingSupport { finding_id: String },
}

#[cfg(test)]
mod tests {
    use super::{
        AdvancedContextProjection, AuthorityRank, CandidateSelectionState,
        ContextIntelligenceError, HybridOutcome, ImpactAnalysisFinding, ImpactFindingKind,
        ImpactFindingSeverity, ImpactFindingStatus, RemoteTransmissionPolicyState,
        RetrievalBudgets, RetrievalCompatibilityState, RetrievalIndexState, RetrievalMatchOrigin,
        RetrievalMode, RetrievalScore, RetrievalSourceKind, RetrievalStalenessState,
        RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState,
        SemanticTraceEventKind, SemanticTraceRecord,
    };
    use crate::domain::governance::CanonSemanticProvenanceBoundary;

    fn semantic_candidate() -> RetrievedEvidenceCandidate {
        RetrievedEvidenceCandidate {
            candidate_id: "candidate-domain-semantic".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            source_ref: "src/lib.rs".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::SemanticExpand,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: "semantic match broadened the bounded evidence set".to_string(),
            provenance_summary: "workspace file selected through semantic expansion".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: RetrievalScore::from_raw(0.812),
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }
    }

    #[test]
    fn semantic_candidates_require_semantic_scores() {
        let mut candidate = semantic_candidate();
        candidate.semantic_score = None;

        assert_eq!(
            candidate.validate().unwrap_err(),
            ContextIntelligenceError::MissingSemanticScore {
                candidate_id: "candidate-domain-semantic".to_string(),
                match_origin: RetrievalMatchOrigin::SemanticExpand,
            }
        );
    }

    #[test]
    fn serde_defaults_cover_optional_semantic_fields_on_projection() {
        // Deserializing a minimal JSON projection exercises the const fn default_*
        // functions that back the #[serde(default)] attributes (lines 925-939).
        let json = r#"{
            "query_id": "query-defaults",
            "retrieval_mode": "disabled",
            "retrieval_state": "unavailable",
            "retrieval_index_state": "insufficient",
            "remote_policy_state": "local_only",
            "used_remote": false
        }"#;
        let projection: AdvancedContextProjection = serde_json::from_str(json).unwrap();
        assert_eq!(projection.semantic_policy_state, SemanticPolicyState::Disabled);
        assert_eq!(projection.semantic_capability_state, SemanticCapabilityState::Unsupported);
        assert_eq!(projection.hybrid_outcome, HybridOutcome::BaselineOnly);
        assert!(projection.selected_evidence.is_empty());
        assert!(projection.rejected_candidates.is_empty());
        assert!(projection.semantic_trace_records.is_empty());
        assert!(projection.relationships.is_empty());
        assert!(projection.impact_findings.is_empty());
    }

    #[test]
    fn advanced_context_projection_counts_semantic_candidates() {
        let rejected_candidate = RetrievedEvidenceCandidate {
            candidate_id: "candidate-domain-rejected".to_string(),
            selection_state: CandidateSelectionState::Rejected,
            selection_reason: "semantic expansion stayed below the bounded confidence bar"
                .to_string(),
            ..semantic_candidate()
        };
        let projection = AdvancedContextProjection {
            query_id: "query-domain-semantic".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Selected,
            retrieval_index_state: RetrievalIndexState::Ready,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Ready,
            hybrid_outcome: HybridOutcome::Expanded,
            budgets: RetrievalBudgets::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some(
                "semantic expansion selected one additional bounded evidence candidate".to_string(),
            ),
            selected_evidence: vec![semantic_candidate()],
            rejected_candidates: vec![rejected_candidate],
            semantic_trace_records: Vec::new(),
            relationships: Vec::new(),
            impact_findings: Vec::new(),
        };

        assert_eq!(projection.semantic_selected_count(), 1);
        assert_eq!(projection.semantic_rejected_count(), 1);
        assert_eq!(semantic_candidate().semantic_score.unwrap().as_raw(), 0.812);
    }

    #[test]
    fn semantic_trace_helpers_cover_event_labels_and_validation_edges() {
        for (event_kind, label, requires_candidate_ref) in [
            (SemanticTraceEventKind::CapabilityEvaluated, "capability_evaluated", false),
            (SemanticTraceEventKind::IndexRefreshed, "index_refreshed", false),
            (SemanticTraceEventKind::ChunkBlocked, "chunk_blocked", true),
            (SemanticTraceEventKind::CandidateExpanded, "candidate_expanded", true),
            (SemanticTraceEventKind::CandidateReranked, "candidate_reranked", true),
            (SemanticTraceEventKind::CandidateRejected, "candidate_rejected", true),
            (SemanticTraceEventKind::CanonArtifactSkipped, "canon_artifact_skipped", true),
            (SemanticTraceEventKind::FallbackApplied, "fallback_applied", false),
            (SemanticTraceEventKind::HybridOutcomeRecorded, "hybrid_outcome_recorded", false),
        ] {
            assert_eq!(event_kind.as_str(), label);
            assert_eq!(event_kind.requires_candidate_ref(), requires_candidate_ref);
        }

        assert!(RetrievalScore::from_raw(f64::NAN).is_none());
        assert!(RetrievalScore::from_raw(f64::INFINITY).is_none());

        let missing_candidate_ref = SemanticTraceRecord {
            record_id: "trace-missing-ref".to_string(),
            event_kind: SemanticTraceEventKind::CandidateExpanded,
            candidate_ref: None,
            match_origin: Some(RetrievalMatchOrigin::SemanticExpand),
            compatibility_state: Some(RetrievalCompatibilityState::Compatible),
            semantic_score: RetrievalScore::from_raw(0.9),
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: "expanded candidate".to_string(),
        };
        assert!(matches!(
            missing_candidate_ref.validate(),
            Err(ContextIntelligenceError::MissingSemanticTraceCandidateRef { .. })
        ));

        let missing_reason = SemanticTraceRecord {
            record_id: "trace-missing-reason".to_string(),
            event_kind: SemanticTraceEventKind::CapabilityEvaluated,
            candidate_ref: None,
            match_origin: None,
            compatibility_state: None,
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: " ".to_string(),
        };
        assert!(matches!(
            missing_reason.validate(),
            Err(ContextIntelligenceError::MissingSemanticTraceReason { .. })
        ));

        let missing_score = SemanticTraceRecord {
            record_id: "trace-missing-score".to_string(),
            event_kind: SemanticTraceEventKind::CandidateReranked,
            candidate_ref: Some("src/lib.rs".to_string()),
            match_origin: Some(RetrievalMatchOrigin::SemanticRerank),
            compatibility_state: Some(RetrievalCompatibilityState::Compatible),
            semantic_score: None,
            canon_artifact_class: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_boundary: None,
            canon_semantic_provenance_ref: None,
            reason: "reranked candidate".to_string(),
        };
        assert!(matches!(
            missing_score.validate(),
            Err(ContextIntelligenceError::MissingSemanticTraceScore { .. })
        ));

        let incomplete_canon_metadata = SemanticTraceRecord {
            record_id: "trace-canon-metadata".to_string(),
            event_kind: SemanticTraceEventKind::CanonArtifactSkipped,
            candidate_ref: Some(".canon/run.md".to_string()),
            match_origin: None,
            compatibility_state: Some(RetrievalCompatibilityState::PolicyBlocked),
            semantic_score: None,
            canon_artifact_class: Some("stable".to_string()),
            canon_semantic_contract_line: Some("v1".to_string()),
            canon_semantic_provenance_boundary: Some(CanonSemanticProvenanceBoundary::Section),
            canon_semantic_provenance_ref: None,
            reason: "blocked by policy".to_string(),
        };
        assert!(matches!(
            incomplete_canon_metadata.validate(),
            Err(ContextIntelligenceError::IncompleteSemanticTraceCanonMetadata { .. })
        ));
    }

    #[test]
    fn advanced_context_projection_validation_covers_semantic_and_remote_consistency() {
        let projection = AdvancedContextProjection {
            query_id: "query-domain-valid".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Selected,
            retrieval_index_state: RetrievalIndexState::Ready,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Ready,
            hybrid_outcome: HybridOutcome::Expanded,
            budgets: RetrievalBudgets::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some("semantic expansion succeeded".to_string()),
            selected_evidence: vec![semantic_candidate()],
            rejected_candidates: Vec::new(),
            semantic_trace_records: Vec::new(),
            relationships: Vec::new(),
            impact_findings: vec![ImpactAnalysisFinding {
                finding_id: "finding-1".to_string(),
                finding_kind: ImpactFindingKind::EvidenceGap,
                subject_ref: "src/lib.rs".to_string(),
                status: ImpactFindingStatus::Open,
                severity: ImpactFindingSeverity::Medium,
                recommended_follow_up: "run targeted tests".to_string(),
                supporting_relationship_ids: vec!["relationship-1".to_string()],
            }],
        };
        assert_eq!(projection.impact_finding_count(), 1);
        projection.validate().unwrap();

        let invalid_hybrid = AdvancedContextProjection {
            semantic_policy_state: SemanticPolicyState::Disabled,
            hybrid_outcome: HybridOutcome::Expanded,
            ..projection.clone()
        };
        assert!(matches!(
            invalid_hybrid.validate(),
            Err(ContextIntelligenceError::InvalidSemanticHybridOutcome { .. })
        ));

        let missing_semantic_reason = AdvancedContextProjection {
            retrieval_state: RetrievalState::Degraded,
            hybrid_outcome: HybridOutcome::Fallback,
            selected_evidence: Vec::new(),
            terminal_reason: None,
            ..projection.clone()
        };
        assert!(matches!(
            missing_semantic_reason.validate(),
            Err(ContextIntelligenceError::MissingSemanticTerminalReason { .. })
        ));

        let missing_selected =
            AdvancedContextProjection { selected_evidence: Vec::new(), ..projection.clone() };
        assert!(matches!(
            missing_selected.validate(),
            Err(ContextIntelligenceError::MissingSelectedEvidence)
        ));

        let missing_terminal = AdvancedContextProjection {
            retrieval_state: RetrievalState::Insufficient,
            hybrid_outcome: HybridOutcome::BaselineOnly,
            selected_evidence: Vec::new(),
            terminal_reason: None,
            ..projection.clone()
        };
        assert!(matches!(
            missing_terminal.validate(),
            Err(ContextIntelligenceError::MissingTerminalReason)
        ));

        let unexpected_remote =
            AdvancedContextProjection { used_remote: true, ..projection.clone() };
        assert!(matches!(
            unexpected_remote.validate(),
            Err(ContextIntelligenceError::UnexpectedRemoteUsage)
        ));

        let blocked_remote = AdvancedContextProjection {
            retrieval_mode: RetrievalMode::Remote,
            used_remote: true,
            remote_policy_state: RemoteTransmissionPolicyState::Blocked,
            ..projection
        };
        assert!(matches!(
            blocked_remote.validate(),
            Err(ContextIntelligenceError::BlockedRemoteUsage)
        ));
    }
}
