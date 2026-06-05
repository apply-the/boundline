//! Typed advanced-context intelligence models shared by planning, runtime,
//! session projections, and trace inspection.

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
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
    Missing,
    Incompatible,
    Degraded,
    Corrupt,
    SemanticUnavailable,
}

impl RetrievalIndexState {
    /// Returns the stable serialization label for this index state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Building => "building",
            Self::Insufficient => "insufficient",
            Self::Missing => "missing",
            Self::Incompatible => "incompatible",
            Self::Degraded => "degraded",
            Self::Corrupt => "corrupt",
            Self::SemanticUnavailable => "semantic_unavailable",
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
    #[serde(alias = "missing")]
    Unavailable,
    Unsupported,
    Degraded,
    Corrupt,
}

impl SemanticCapabilityState {
    /// Returns the stable serialization label for this semantic capability state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Degraded => "degraded",
            Self::Corrupt => "corrupt",
        }
    }
}

/// Effective semantic engine recorded for one derived index or retrieval surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEngine {
    Disabled,
    BaselineJson,
    SqliteVec,
}

impl SemanticEngine {
    /// Returns the stable serialization label for this semantic engine state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::BaselineJson => "baseline_json",
            Self::SqliteVec => "sqlite_vec",
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
    Deleted,
    MissingVector,
}

impl SemanticChunkState {
    /// Returns the stable serialization label for this semantic chunk state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Blocked => "blocked",
            Self::Deleted => "deleted",
            Self::MissingVector => "missing_vector",
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
    Degraded,
    Corrupt,
}

impl VectorExtensionState {
    /// Returns the stable serialization label for this vector extension state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
            Self::Stale => "stale",
            Self::Degraded => "degraded",
            Self::Corrupt => "corrupt",
        }
    }
}

/// Lightweight FTS5 health recorded in the derived-index manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestFtsState {
    Ready,
    Missing,
    Corrupt,
}

impl ManifestFtsState {
    /// Returns the stable serialization label for this FTS5 manifest state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Corrupt => "corrupt",
        }
    }
}

/// Refresh reason recorded in the manifest after one bounded index operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexRefreshReason {
    ManualRefresh,
    Rebuild,
    SchemaChange,
    BranchChange,
    ConfigChange,
    ChunkerChange,
    CapabilityChange,
    DoctorRepair,
}

impl IndexRefreshReason {
    /// Returns the stable serialization label for this refresh reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManualRefresh => "manual_refresh",
            Self::Rebuild => "rebuild",
            Self::SchemaChange => "schema_change",
            Self::BranchChange => "branch_change",
            Self::ConfigChange => "config_change",
            Self::ChunkerChange => "chunker_change",
            Self::CapabilityChange => "capability_change",
            Self::DoctorRepair => "doctor_repair",
        }
    }
}

/// Freshness reason recorded when the derived index becomes stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexStaleReason {
    GitHeadChanged,
    BranchCheckout,
    Merge,
    PullWithMerge,
    Rebase,
    PostRewrite,
    HookMarkedStale,
}

impl IndexStaleReason {
    /// Returns the stable serialization label for this stale reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GitHeadChanged => "git_head_changed",
            Self::BranchCheckout => "branch_checkout",
            Self::Merge => "merge",
            Self::PullWithMerge => "pull_with_merge",
            Self::Rebase => "rebase",
            Self::PostRewrite => "post_rewrite",
            Self::HookMarkedStale => "hook_marked_stale",
        }
    }
}

/// Presence state for one indexed source snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePresenceState {
    Present,
    Deleted,
    Skipped,
}

impl SourcePresenceState {
    /// Returns the stable serialization label for this source presence state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Deleted => "deleted",
            Self::Skipped => "skipped",
        }
    }
}

/// Compatibility outcome for one indexed source snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDigestCompatibilityState {
    Compatible,
    Excluded,
    Unsupported,
    Blocked,
}

impl SourceDigestCompatibilityState {
    /// Returns the stable serialization label for this source-digest compatibility state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Excluded => "excluded",
            Self::Unsupported => "unsupported",
            Self::Blocked => "blocked",
        }
    }
}

/// Health state for one persisted vector row inside the derived index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticVectorState {
    Ready,
    Missing,
    Stale,
    DimensionMismatch,
    Corrupt,
}

impl SemanticVectorState {
    /// Returns the stable serialization label for this semantic vector state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::DimensionMismatch => "dimension_mismatch",
            Self::Corrupt => "corrupt",
        }
    }
}

/// Command name recorded for one derived-index lifecycle operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexMaintenanceCommand {
    Status,
    Refresh,
    Rebuild,
    Clean,
    Doctor,
}

impl IndexMaintenanceCommand {
    /// Returns the stable serialization label for this maintenance command.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Refresh => "refresh",
            Self::Rebuild => "rebuild",
            Self::Clean => "clean",
            Self::Doctor => "doctor",
        }
    }
}

/// Trigger source recorded for one derived-index lifecycle operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexMaintenanceTrigger {
    Manual,
    PostCheckout,
    PostMerge,
    PostRewrite,
    SchemaChange,
    ConfigChange,
    CapabilityChange,
}

impl IndexMaintenanceTrigger {
    /// Returns the stable serialization label for this maintenance trigger.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::PostCheckout => "post_checkout",
            Self::PostMerge => "post_merge",
            Self::PostRewrite => "post_rewrite",
            Self::SchemaChange => "schema_change",
            Self::ConfigChange => "config_change",
            Self::CapabilityChange => "capability_change",
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
    ExtensionLoadAttempted,
    IndexRefreshed,
    ChunkBlocked,
    VectorQueryExecuted,
    VectorCandidatesReturned,
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
            Self::ExtensionLoadAttempted => "extension_load_attempted",
            Self::IndexRefreshed => "index_refreshed",
            Self::ChunkBlocked => "chunk_blocked",
            Self::VectorQueryExecuted => "vector_query_executed",
            Self::VectorCandidatesReturned => "vector_candidates_returned",
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

/// Fidelity tier assigned to one candidate before inclusion or omission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextFidelityTier {
    Critical,
    Supporting,
    Ambient,
    Archived,
}

impl ContextFidelityTier {
    /// Returns the stable serialization label for this fidelity tier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Supporting => "supporting",
            Self::Ambient => "ambient",
            Self::Archived => "archived",
        }
    }
}

/// Inclusion mode used for one persisted context-pack entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInclusionMode {
    Full,
    Excerpt,
    Summary,
    Signature,
    Digest,
    Omitted,
}

impl ContextInclusionMode {
    /// Returns the stable serialization label for this inclusion mode.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Excerpt => "excerpt",
            Self::Summary => "summary",
            Self::Signature => "signature",
            Self::Digest => "digest",
            Self::Omitted => "omitted",
        }
    }

    /// Returns true when the mode is lossy for critical execution decisions.
    pub const fn is_lossy_for_critical(self) -> bool {
        matches!(self, Self::Summary | Self::Signature | Self::Digest | Self::Omitted)
    }
}

/// Severity attached to one inspectable omission or downgrade finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextOmissionSeverity {
    Info,
    Warning,
    Blocking,
}

impl ContextOmissionSeverity {
    /// Returns the stable serialization label for this omission severity.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Blocking => "blocking",
        }
    }

    /// Returns true when the finding must stop planning or execution.
    pub const fn is_blocking(self) -> bool {
        matches!(self, Self::Blocking)
    }
}

/// Derived repository-map freshness surfaced through runtime projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryMapState {
    Ready,
    Stale,
    Missing,
    Degraded,
    Corrupt,
}

impl RepositoryMapState {
    /// Returns the stable serialization label for this repository-map state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Missing => "missing",
            Self::Degraded => "degraded",
            Self::Corrupt => "corrupt",
        }
    }
}

/// Freshness state for the derived local snapshot cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotCacheState {
    Ready,
    Stale,
    Missing,
    Degraded,
    Tracked,
    Corrupt,
}

impl SnapshotCacheState {
    /// Returns the stable serialization label for this snapshot-cache state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Missing => "missing",
            Self::Degraded => "degraded",
            Self::Tracked => "tracked",
            Self::Corrupt => "corrupt",
        }
    }
}

/// Outcome recorded for one patch-safe large-file edit attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchSafeEditResultState {
    Applied,
    Drifted,
    Rejected,
    ManualReviewRequired,
}

impl PatchSafeEditResultState {
    /// Returns the stable serialization label for this patch-safe result.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Drifted => "drifted",
            Self::Rejected => "rejected",
            Self::ManualReviewRequired => "manual_review_required",
        }
    }
}

/// Recoverable reference for a compacted large artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigestBackedArtifactRef {
    pub digest: String,
    pub artifact_kind: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt_anchor: Option<String>,
    pub resolve_path: String,
}

impl DigestBackedArtifactRef {
    /// Validates the compacted artifact reference before it is projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.digest.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingDigestBackedArtifactDigest);
        }
        if self.artifact_kind.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingDigestBackedArtifactKind {
                digest: self.digest.clone(),
            });
        }
        if self.summary.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingDigestBackedArtifactSummary {
                digest: self.digest.clone(),
            });
        }
        if self.resolve_path.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingDigestBackedArtifactResolvePath {
                digest: self.digest.clone(),
            });
        }
        Ok(())
    }
}

/// Persisted handling decision for one context candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPackEntryProjection {
    pub source_ref: String,
    pub source_kind: RetrievalSourceKind,
    pub authority_rank: AuthorityRank,
    pub fidelity_tier: ContextFidelityTier,
    pub inclusion_mode: ContextInclusionMode,
    pub reason: String,
    pub required_for_admission: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_excerpt_anchor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest_ref: Option<DigestBackedArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_relevance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_relevance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_rationale: Option<String>,
}

impl ContextPackEntryProjection {
    /// Validates one context-pack entry before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.source_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingContextPackEntrySourceRef);
        }
        if self.reason.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingContextPackEntryReason {
                source_ref: self.source_ref.clone(),
            });
        }
        if let Some(digest_ref) = &self.digest_ref {
            digest_ref.validate()?;
        }
        Ok(())
    }
}

/// Inspectable explanation for a context omission, downgrade, or refusal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextOmissionFinding {
    pub severity: ContextOmissionSeverity,
    pub reason_code: String,
    pub candidate_ref: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_fidelity: Option<ContextFidelityTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_mode: Option<ContextInclusionMode>,
}

impl ContextOmissionFinding {
    /// Returns true when the omission finding should block continuation.
    pub const fn blocks_continuation(&self) -> bool {
        self.severity.is_blocking()
    }

    /// Validates one omission finding before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.reason_code.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingOmissionReasonCode);
        }
        if self.candidate_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingOmissionCandidateRef {
                reason_code: self.reason_code.clone(),
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingOmissionMessage {
                reason_code: self.reason_code.clone(),
            });
        }
        Ok(())
    }
}

/// Patch-safe large-file edit attempt recorded for inspectable runtime state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchSafeEditAttempt {
    pub target_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchor_refs: Vec<String>,
    pub pre_apply_digest: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_apply_verification: Vec<String>,
    pub result_state: PatchSafeEditResultState,
}

impl PatchSafeEditAttempt {
    /// Validates one patch-safe edit attempt before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.target_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingPatchSafeTargetRef);
        }
        if self.anchor_refs.is_empty() {
            return Err(ContextIntelligenceError::MissingPatchSafeAnchorRefs {
                target_ref: self.target_ref.clone(),
            });
        }
        if self.pre_apply_digest.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingPatchSafePreApplyDigest {
                target_ref: self.target_ref.clone(),
            });
        }
        if self.post_apply_verification.is_empty() {
            return Err(ContextIntelligenceError::MissingPatchSafePostApplyVerification {
                target_ref: self.target_ref.clone(),
            });
        }
        Ok(())
    }
}

/// One candidate surfaced or rejected during advanced-context retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

    /// Returns the collapsed chunk count when the semantic path selected or rejected this source.
    pub const fn collapsed_from_chunk_count(&self) -> Option<usize> {
        match self.match_origin {
            RetrievalMatchOrigin::SemanticExpand | RetrievalMatchOrigin::SemanticRerank => Some(1),
            RetrievalMatchOrigin::Fts | RetrievalMatchOrigin::StructuredFallback => None,
        }
    }
}

impl Serialize for RetrievedEvidenceCandidate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RetrievedEvidenceCandidate", 14)?;
        state.serialize_field("candidate_id", &self.candidate_id)?;
        state.serialize_field("source_kind", &self.source_kind)?;
        state.serialize_field("source_ref", &self.source_ref)?;
        state.serialize_field("authority_rank", &self.authority_rank)?;
        state.serialize_field("match_origin", &self.match_origin)?;
        state.serialize_field("selection_state", &self.selection_state)?;
        state.serialize_field("selection_reason", &self.selection_reason)?;
        state.serialize_field("provenance_summary", &self.provenance_summary)?;
        state.serialize_field("compatibility_state", &self.compatibility_state)?;
        state.serialize_field("staleness_state", &self.staleness_state)?;
        if let Some(lexical_score) = self.lexical_score {
            state.serialize_field("lexical_score", &lexical_score)?;
        }
        if let Some(semantic_score) = self.semantic_score {
            state.serialize_field("semantic_score", &semantic_score)?;
        }
        if let Some(contract_line) = self.canon_semantic_contract_line.as_ref() {
            state.serialize_field("canon_semantic_contract_line", contract_line)?;
        }
        if let Some(provenance_ref) = self.canon_semantic_provenance_ref.as_ref() {
            state.serialize_field("canon_semantic_provenance_ref", provenance_ref)?;
        }
        if let Some(collapsed_count) = self.collapsed_from_chunk_count() {
            state.serialize_field("collapsed_from_chunk_count", &collapsed_count)?;
        }
        state.end()
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

/// Companion manifest persisted next to the derived retrieval index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexManifest {
    pub schema_version: String,
    pub workspace_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_head: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_head: Option<String>,
    pub index_status: RetrievalIndexState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh_reason: Option<IndexRefreshReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_reason: Option<IndexStaleReason>,
    pub file_count: usize,
    pub chunk_count: usize,
    pub fts5_state: ManifestFtsState,
    pub sqlite_vec_state: VectorExtensionState,
    pub semantic_engine: SemanticEngine,
    pub workspace_fingerprint: String,
    pub config_fingerprint: String,
    pub chunker_fingerprint: String,
    pub embedding_model_fingerprint: String,
}

impl DerivedIndexManifest {
    /// Validates one derived-index manifest before it is persisted or projected.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.schema_version.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingManifestSchemaVersion);
        }
        if self.workspace_root.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingManifestWorkspaceRoot);
        }
        if self.workspace_fingerprint.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingManifestWorkspaceFingerprint);
        }
        if self.config_fingerprint.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingManifestConfigFingerprint);
        }
        if self.chunker_fingerprint.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingManifestChunkerFingerprint);
        }
        if self.embedding_model_fingerprint.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingManifestEmbeddingFingerprint);
        }
        if self.index_status == RetrievalIndexState::Ready
            && self.fts5_state != ManifestFtsState::Ready
        {
            return Err(ContextIntelligenceError::InvalidReadyManifestFtsState {
                fts5_state: self.fts5_state,
            });
        }
        if self.semantic_engine == SemanticEngine::SqliteVec
            && self.sqlite_vec_state != VectorExtensionState::Ready
        {
            return Err(ContextIntelligenceError::InvalidSqliteVecManifestState {
                sqlite_vec_state: self.sqlite_vec_state,
            });
        }
        Ok(())
    }

    /// Returns true when a cheap HEAD probe shows the manifest is stale.
    pub fn head_is_stale(&self) -> bool {
        matches!(
            (&self.git_head, &self.last_seen_head),
            (Some(git_head), Some(last_seen_head)) if git_head != last_seen_head
        )
    }

    /// Returns the most specific stale reason that can be derived cheaply.
    pub fn effective_stale_reason(&self) -> Option<IndexStaleReason> {
        self.stale_reason.or_else(|| {
            if self.head_is_stale() {
                Some(IndexStaleReason::GitHeadChanged)
            } else if self.index_status == RetrievalIndexState::Stale {
                Some(IndexStaleReason::HookMarkedStale)
            } else {
                None
            }
        })
    }

    /// Returns true when the next manifest shape requires a rebuild instead of reuse.
    pub fn requires_rebuild_against(&self, next_manifest: &Self) -> bool {
        self.schema_version != next_manifest.schema_version
            || self.workspace_root != next_manifest.workspace_root
            || self.config_fingerprint != next_manifest.config_fingerprint
            || self.chunker_fingerprint != next_manifest.chunker_fingerprint
            || self.embedding_model_fingerprint != next_manifest.embedding_model_fingerprint
    }
}

/// Indexed digest snapshot for one eligible source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDigestRecord {
    pub source_ref: String,
    pub source_kind: RetrievalSourceKind,
    pub content_hash: String,
    pub compatibility_state: SourceDigestCompatibilityState,
    pub authority_rank: AuthorityRank,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_indexed_at: Option<String>,
    pub chunk_count: usize,
    pub source_presence_state: SourcePresenceState,
}

impl SourceDigestRecord {
    /// Validates one indexed source digest record.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.source_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSourceDigestRef);
        }
        if self.content_hash.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSourceDigestHash {
                source_ref: self.source_ref.clone(),
            });
        }
        if self.source_presence_state == SourcePresenceState::Deleted && self.chunk_count != 0 {
            return Err(ContextIntelligenceError::InvalidDeletedSourceChunkCount {
                source_ref: self.source_ref.clone(),
                chunk_count: self.chunk_count,
            });
        }
        Ok(())
    }
}

/// Stable chunk metadata persisted for one source-level evidence fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChunkRecord {
    pub chunk_id: String,
    pub source_ref: String,
    pub chunk_ordinal: usize,
    pub chunk_range: String,
    pub provenance_boundary: String,
    pub provenance_ref: String,
    pub content_hash: String,
    pub chunk_state: SemanticChunkState,
    pub embedding_dimensions: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_semantic_contract_line: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_labels: Vec<String>,
}

impl SemanticChunkRecord {
    /// Returns the stable chunk identifier for one source ref and chunk ordinal.
    pub fn stable_chunk_id(source_ref: &str, chunk_ordinal: usize) -> String {
        format!("semantic:{source_ref}:{chunk_ordinal}")
    }

    /// Validates one persisted semantic chunk record.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.chunk_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticChunkId);
        }
        if self.source_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticChunkSourceRef {
                chunk_id: self.chunk_id.clone(),
            });
        }
        if self.chunk_range.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticChunkRange {
                chunk_id: self.chunk_id.clone(),
            });
        }
        if self.provenance_boundary.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticChunkBoundary {
                chunk_id: self.chunk_id.clone(),
            });
        }
        if self.provenance_ref.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticChunkProvenanceRef {
                chunk_id: self.chunk_id.clone(),
            });
        }
        if self.content_hash.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticChunkHash {
                chunk_id: self.chunk_id.clone(),
            });
        }
        let expected_chunk_id = Self::stable_chunk_id(&self.source_ref, self.chunk_ordinal);
        if self.chunk_id != expected_chunk_id {
            return Err(ContextIntelligenceError::InvalidSemanticChunkId {
                chunk_id: self.chunk_id.clone(),
                expected_chunk_id,
            });
        }
        if self.embedding_dimensions == 0 {
            return Err(ContextIntelligenceError::InvalidSemanticChunkDimensions {
                chunk_id: self.chunk_id.clone(),
            });
        }
        Ok(())
    }
}

/// Vector row metadata persisted for one semantic chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticVectorRecord {
    pub chunk_id: String,
    pub vector_schema_line: String,
    pub embedding_dimensions: usize,
    pub write_generation: u64,
    pub vector_state: SemanticVectorState,
}

impl SemanticVectorRecord {
    /// Validates one persisted semantic vector record.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.chunk_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticVectorChunkId);
        }
        if self.vector_schema_line.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingSemanticVectorSchemaLine {
                chunk_id: self.chunk_id.clone(),
            });
        }
        if self.embedding_dimensions == 0 {
            return Err(ContextIntelligenceError::InvalidSemanticVectorDimensions {
                chunk_id: self.chunk_id.clone(),
            });
        }
        Ok(())
    }
}

/// Operator-visible lifecycle operation over the derived index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexMaintenanceOperation {
    pub operation_id: String,
    pub command_name: IndexMaintenanceCommand,
    pub trigger: IndexMaintenanceTrigger,
    pub pre_state: RetrievalIndexState,
    pub post_state: RetrievalIndexState,
    pub sources_scanned: usize,
    pub chunks_upserted: usize,
    pub chunks_deleted: usize,
    pub vector_rows_written: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<String>,
}

impl IndexMaintenanceOperation {
    /// Validates one derived-index lifecycle operation snapshot.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.operation_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexOperationId);
        }
        if self.command_name == IndexMaintenanceCommand::Status
            && self.pre_state == RetrievalIndexState::Missing
            && self.post_state == RetrievalIndexState::Ready
        {
            return Err(ContextIntelligenceError::InvalidStatusOperationStateTransition);
        }
        if self.post_state != RetrievalIndexState::Ready
            && self.recommended_action.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(ContextIntelligenceError::MissingIndexOperationRecommendedAction {
                operation_id: self.operation_id.clone(),
            });
        }
        Ok(())
    }
}

/// Serialized operator-facing report for one derived-index lifecycle command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexLifecycleReport {
    pub command: IndexMaintenanceCommand,
    pub workspace_root: String,
    pub operation_id: String,
    pub pre_state: RetrievalIndexState,
    pub post_state: RetrievalIndexState,
    pub recommended_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_reason: Option<IndexStaleReason>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<DerivedIndexManifest>,
}

impl IndexLifecycleReport {
    /// Validates one serialized lifecycle report before it is emitted.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.workspace_root.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexLifecycleWorkspaceRoot);
        }
        if self.operation_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexLifecycleOperationId);
        }
        if self.recommended_action.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexLifecycleRecommendedAction);
        }
        if self.post_state == RetrievalIndexState::Stale && self.stale_reason.is_none() {
            return Err(ContextIntelligenceError::MissingIndexLifecycleStaleReason);
        }
        Ok(())
    }
}

/// Status for one derived-index doctor report or check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexDoctorStatus {
    Passed,
    Advisory,
    Failed,
}

impl IndexDoctorStatus {
    /// Returns the stable serialization label for this doctor status.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Advisory => "advisory",
            Self::Failed => "failed",
        }
    }
}

/// Consistency state for one derived-index artifact inspected by `index doctor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexDoctorConsistencyState {
    Consistent,
    Missing,
    Corrupt,
    Invalid,
}

impl IndexDoctorConsistencyState {
    /// Returns the stable serialization label for this doctor consistency state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Consistent => "consistent",
            Self::Missing => "missing",
            Self::Corrupt => "corrupt",
            Self::Invalid => "invalid",
        }
    }
}

/// One operator-facing doctor check result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDoctorCheck {
    pub check_name: String,
    pub result: IndexDoctorStatus,
    pub detail: String,
    pub suggested_fix: String,
}

impl IndexDoctorCheck {
    /// Validates one serialized doctor check before it is emitted.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.check_name.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexDoctorCheckName);
        }
        if self.detail.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexDoctorCheckDetail {
                check_name: self.check_name.clone(),
            });
        }
        if self.suggested_fix.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingIndexDoctorCheckSuggestedFix {
                check_name: self.check_name.clone(),
            });
        }
        Ok(())
    }
}

/// Serialized operator-facing report for one derived-index doctor command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDoctorReport {
    pub status: IndexDoctorStatus,
    pub checks: Vec<IndexDoctorCheck>,
    pub tracked_index_files: Vec<String>,
    pub missing_ignore_rules: Vec<String>,
    pub wal_sidecars_present: bool,
    pub manifest_consistency: IndexDoctorConsistencyState,
    pub vector_schema_consistency: IndexDoctorConsistencyState,
}

impl IndexDoctorReport {
    /// Validates one serialized doctor report before it is emitted.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.checks.is_empty() {
            return Err(ContextIntelligenceError::MissingIndexDoctorChecks);
        }
        for check in &self.checks {
            check.validate()?;
        }
        Ok(())
    }
}

/// Persisted projection of one advanced-context retrieval decision.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_pack_entries: Vec<ContextPackEntryProjection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omission_findings: Vec<ContextOmissionFinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_map_state: Option<RepositoryMapState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_cache_state: Option<SnapshotCacheState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patch_safe_edit_attempts: Vec<PatchSafeEditAttempt>,
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

    /// Returns true when the context substrate recorded a blocking omission.
    pub fn has_blocking_context_gap(&self) -> bool {
        self.omission_findings.iter().any(ContextOmissionFinding::blocks_continuation)
    }

    /// Returns the contract-facing semantic capability label for compact status surfaces.
    pub const fn semantic_capability_contract_label(&self) -> &'static str {
        match self.semantic_capability_state {
            SemanticCapabilityState::Unavailable => "missing",
            SemanticCapabilityState::Ready => "ready",
            SemanticCapabilityState::Unsupported => "unsupported",
            SemanticCapabilityState::Degraded => "degraded",
            SemanticCapabilityState::Corrupt => "corrupt",
        }
    }

    /// Returns the effective semantic engine surfaced to operators.
    pub const fn semantic_engine(&self) -> SemanticEngine {
        match self.semantic_policy_state {
            SemanticPolicyState::Disabled => SemanticEngine::Disabled,
            SemanticPolicyState::Local => match self.semantic_capability_state {
                SemanticCapabilityState::Ready => SemanticEngine::SqliteVec,
                SemanticCapabilityState::Unavailable
                | SemanticCapabilityState::Unsupported
                | SemanticCapabilityState::Degraded
                | SemanticCapabilityState::Corrupt => SemanticEngine::BaselineJson,
            },
        }
    }

    /// Returns the number of vector queries attributed to this projection.
    pub fn vector_query_count(&self) -> usize {
        if self.semantic_policy_state == SemanticPolicyState::Disabled {
            return 0;
        }
        if self.semantic_capability_state == SemanticCapabilityState::Ready
            || self.semantic_selected_count() > 0
            || self.semantic_rejected_count() > 0
        {
            1
        } else {
            0
        }
    }

    /// Returns the number of chunk candidates surfaced before source-level collapse.
    pub fn vector_candidates_returned(&self) -> usize {
        self.semantic_selected_count() + self.semantic_rejected_count()
    }

    /// Returns the explicit semantic fallback reason when the preferred vector path did not win.
    pub fn semantic_fallback_reason(&self) -> Option<&str> {
        if self.semantic_policy_state == SemanticPolicyState::Disabled {
            return None;
        }
        if self.semantic_capability_state != SemanticCapabilityState::Ready
            || matches!(self.hybrid_outcome, HybridOutcome::Fallback | HybridOutcome::Skipped)
        {
            return self.terminal_reason.as_deref();
        }
        None
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
        for entry in &self.context_pack_entries {
            entry.validate()?;
        }
        for finding in &self.omission_findings {
            finding.validate()?;
        }
        for attempt in &self.patch_safe_edit_attempts {
            attempt.validate()?;
        }
        Ok(())
    }
}

impl Serialize for AdvancedContextProjection {
    #[rustfmt::skip]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer { let mut state = serializer.serialize_struct("AdvancedContextProjection", 24)?;
        state.serialize_field("query_id", &self.query_id)?;
        state.serialize_field("retrieval_mode", &self.retrieval_mode)?;
        state.serialize_field("retrieval_state", &self.retrieval_state)?;
        state.serialize_field("retrieval_index_state", &self.retrieval_index_state)?;
        state.serialize_field("semantic_policy_state", &self.semantic_policy_state)?;
        state.serialize_field(
            "semantic_capability_state",
            self.semantic_capability_contract_label(),
        )?;
        state.serialize_field("semantic_engine", &self.semantic_engine())?;
        state.serialize_field("hybrid_outcome", &self.hybrid_outcome)?;
        state.serialize_field("vector_query_count", &self.vector_query_count())?;
        state.serialize_field("vector_candidates_returned", &self.vector_candidates_returned())?;
        if let Some(fallback_reason) = self.semantic_fallback_reason() {
            state.serialize_field("semantic_fallback_reason", fallback_reason)?;
        }
        state.serialize_field("budgets", &self.budgets)?;
        state.serialize_field("remote_policy_state", &self.remote_policy_state)?;
        state.serialize_field("used_remote", &self.used_remote)?;
        if let Some(terminal_reason) = self.terminal_reason.as_ref() {
            state.serialize_field("terminal_reason", terminal_reason)?;
        }
        if !self.selected_evidence.is_empty() {
            state.serialize_field("selected_evidence", &self.selected_evidence)?;
        }
        if !self.rejected_candidates.is_empty() {
            state.serialize_field("rejected_candidates", &self.rejected_candidates)?;
        }
        if !self.semantic_trace_records.is_empty() {
            state.serialize_field("semantic_trace_records", &self.semantic_trace_records)?;
        }
        if !self.relationships.is_empty() {
            state.serialize_field("relationships", &self.relationships)?;
        }
        if !self.impact_findings.is_empty() {
            state.serialize_field("impact_findings", &self.impact_findings)?;
        }
        if !self.context_pack_entries.is_empty() {
            state.serialize_field("context_pack_entries", &self.context_pack_entries)?;
        }
        if !self.omission_findings.is_empty() {
            state.serialize_field("omission_findings", &self.omission_findings)?;
        }
        if let Some(repository_map_state) = self.repository_map_state {
            state.serialize_field("repository_map_state", &repository_map_state)?;
        }
        if let Some(snapshot_cache_state) = self.snapshot_cache_state {
            state.serialize_field("snapshot_cache_state", &snapshot_cache_state)?;
        }
        if !self.patch_safe_edit_attempts.is_empty() {
            state.serialize_field("patch_safe_edit_attempts", &self.patch_safe_edit_attempts)?;
        }
        state.end()
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
    #[error("derived index manifest requires a schema version")]
    MissingManifestSchemaVersion,
    #[error("derived index manifest requires a workspace root")]
    MissingManifestWorkspaceRoot,
    #[error("derived index manifest requires a workspace fingerprint")]
    MissingManifestWorkspaceFingerprint,
    #[error("derived index manifest requires a config fingerprint")]
    MissingManifestConfigFingerprint,
    #[error("derived index manifest requires a chunker fingerprint")]
    MissingManifestChunkerFingerprint,
    #[error("derived index manifest requires an embedding-model fingerprint")]
    MissingManifestEmbeddingFingerprint,
    #[error(
        "derived index manifest cannot report ready while FTS5 state is `{}`",
        fts5_state.as_str()
    )]
    InvalidReadyManifestFtsState { fts5_state: ManifestFtsState },
    #[error(
        "derived index manifest cannot report sqlite_vec engine while vector state is `{}`",
        sqlite_vec_state.as_str()
    )]
    InvalidSqliteVecManifestState { sqlite_vec_state: VectorExtensionState },
    #[error("source digest record requires a source ref")]
    MissingSourceDigestRef,
    #[error("source digest `{source_ref}` requires a content hash")]
    MissingSourceDigestHash { source_ref: String },
    #[error("source digest `{source_ref}` cannot stay deleted while chunk_count is {chunk_count}")]
    InvalidDeletedSourceChunkCount { source_ref: String, chunk_count: usize },
    #[error("semantic chunk record requires a chunk id")]
    MissingSemanticChunkId,
    #[error("semantic chunk `{chunk_id}` requires a source ref")]
    MissingSemanticChunkSourceRef { chunk_id: String },
    #[error("semantic chunk `{chunk_id}` requires a chunk range")]
    MissingSemanticChunkRange { chunk_id: String },
    #[error("semantic chunk `{chunk_id}` requires a provenance boundary")]
    MissingSemanticChunkBoundary { chunk_id: String },
    #[error("semantic chunk `{chunk_id}` requires a provenance ref")]
    MissingSemanticChunkProvenanceRef { chunk_id: String },
    #[error("semantic chunk `{chunk_id}` requires a content hash")]
    MissingSemanticChunkHash { chunk_id: String },
    #[error("semantic chunk `{chunk_id}` must match the stable chunk id `{expected_chunk_id}`")]
    InvalidSemanticChunkId { chunk_id: String, expected_chunk_id: String },
    #[error("semantic chunk `{chunk_id}` requires embedding dimensions greater than zero")]
    InvalidSemanticChunkDimensions { chunk_id: String },
    #[error("semantic vector record requires a chunk id")]
    MissingSemanticVectorChunkId,
    #[error("semantic vector record `{chunk_id}` requires a schema line")]
    MissingSemanticVectorSchemaLine { chunk_id: String },
    #[error("semantic vector record `{chunk_id}` requires embedding dimensions greater than zero")]
    InvalidSemanticVectorDimensions { chunk_id: String },
    #[error("index maintenance operation requires an id")]
    MissingIndexOperationId,
    #[error("index status cannot report a missing->ready mutation without a refresh or rebuild")]
    InvalidStatusOperationStateTransition,
    #[error(
        "index maintenance operation `{operation_id}` requires a recommended action for non-ready post states"
    )]
    MissingIndexOperationRecommendedAction { operation_id: String },
    #[error("index lifecycle report requires a workspace root")]
    MissingIndexLifecycleWorkspaceRoot,
    #[error("index lifecycle report requires an operation id")]
    MissingIndexLifecycleOperationId,
    #[error("index lifecycle report requires a recommended action")]
    MissingIndexLifecycleRecommendedAction,
    #[error("index lifecycle report requires a stale reason when post_state is stale")]
    MissingIndexLifecycleStaleReason,
    #[error("index doctor report requires at least one check")]
    MissingIndexDoctorChecks,
    #[error("index doctor check requires a check name")]
    MissingIndexDoctorCheckName,
    #[error("index doctor check `{check_name}` requires a detail")]
    MissingIndexDoctorCheckDetail { check_name: String },
    #[error("index doctor check `{check_name}` requires a suggested fix")]
    MissingIndexDoctorCheckSuggestedFix { check_name: String },
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
    #[error("digest-backed artifact reference requires a digest")]
    MissingDigestBackedArtifactDigest,
    #[error("digest-backed artifact `{digest}` requires an artifact kind")]
    MissingDigestBackedArtifactKind { digest: String },
    #[error("digest-backed artifact `{digest}` requires a summary")]
    MissingDigestBackedArtifactSummary { digest: String },
    #[error("digest-backed artifact `{digest}` requires a resolve path")]
    MissingDigestBackedArtifactResolvePath { digest: String },
    #[error("context-pack entry requires a source ref")]
    MissingContextPackEntrySourceRef,
    #[error("context-pack entry `{source_ref}` requires a reason")]
    MissingContextPackEntryReason { source_ref: String },
    #[error("context omission finding requires a reason code")]
    MissingOmissionReasonCode,
    #[error("context omission finding `{reason_code}` requires a candidate ref")]
    MissingOmissionCandidateRef { reason_code: String },
    #[error("context omission finding `{reason_code}` requires a message")]
    MissingOmissionMessage { reason_code: String },
    #[error("patch-safe edit attempt requires a target ref")]
    MissingPatchSafeTargetRef,
    #[error("patch-safe edit attempt `{target_ref}` requires anchor refs")]
    MissingPatchSafeAnchorRefs { target_ref: String },
    #[error("patch-safe edit attempt `{target_ref}` requires a pre-apply digest")]
    MissingPatchSafePreApplyDigest { target_ref: String },
    #[error("patch-safe edit attempt `{target_ref}` requires post-apply verification evidence")]
    MissingPatchSafePostApplyVerification { target_ref: String },
}

#[cfg(test)]
mod tests {
    use super::{
        AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ContextFidelityTier,
        ContextInclusionMode, ContextIntelligenceError, ContextOmissionFinding,
        ContextOmissionSeverity, ContextPackEntryProjection, DigestBackedArtifactRef,
        HybridOutcome, ImpactAnalysisFinding, ImpactFindingKind, ImpactFindingSeverity,
        ImpactFindingStatus, IndexMaintenanceCommand, IndexMaintenanceOperation,
        IndexMaintenanceTrigger, PatchSafeEditAttempt, PatchSafeEditResultState,
        RemoteTransmissionPolicyState, RepositoryMapState, RetrievalBudgets,
        RetrievalCompatibilityState, RetrievalIndexState, RetrievalMatchOrigin, RetrievalMode,
        RetrievalScore, RetrievalSourceKind, RetrievalStalenessState, RetrievalState,
        RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticChunkRecord,
        SemanticChunkState, SemanticPolicyState, SemanticTraceEventKind, SemanticTraceRecord,
        SemanticVectorRecord, SemanticVectorState, SnapshotCacheState,
        SourceDigestCompatibilityState, SourceDigestRecord, SourcePresenceState,
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
    fn context_substrate_types_expose_stable_labels_and_validate_required_fields() {
        for (tier, label) in [
            (ContextFidelityTier::Critical, "critical"),
            (ContextFidelityTier::Supporting, "supporting"),
            (ContextFidelityTier::Ambient, "ambient"),
            (ContextFidelityTier::Archived, "archived"),
        ] {
            assert_eq!(tier.as_str(), label);
        }

        for (mode, label, lossy) in [
            (ContextInclusionMode::Full, "full", false),
            (ContextInclusionMode::Excerpt, "excerpt", false),
            (ContextInclusionMode::Summary, "summary", true),
            (ContextInclusionMode::Signature, "signature", true),
            (ContextInclusionMode::Digest, "digest", true),
            (ContextInclusionMode::Omitted, "omitted", true),
        ] {
            assert_eq!(mode.as_str(), label);
            assert_eq!(mode.is_lossy_for_critical(), lossy);
        }

        for (severity, label) in [
            (ContextOmissionSeverity::Info, "info"),
            (ContextOmissionSeverity::Warning, "warning"),
            (ContextOmissionSeverity::Blocking, "blocking"),
        ] {
            assert_eq!(severity.as_str(), label);
        }

        for (state, label) in [
            (RepositoryMapState::Ready, "ready"),
            (RepositoryMapState::Missing, "missing"),
            (RepositoryMapState::Stale, "stale"),
            (RepositoryMapState::Degraded, "degraded"),
            (RepositoryMapState::Corrupt, "corrupt"),
        ] {
            assert_eq!(state.as_str(), label);
        }

        for (state, label) in [
            (SnapshotCacheState::Ready, "ready"),
            (SnapshotCacheState::Missing, "missing"),
            (SnapshotCacheState::Stale, "stale"),
            (SnapshotCacheState::Tracked, "tracked"),
            (SnapshotCacheState::Degraded, "degraded"),
            (SnapshotCacheState::Corrupt, "corrupt"),
        ] {
            assert_eq!(state.as_str(), label);
        }

        for (state, label) in [
            (PatchSafeEditResultState::Applied, "applied"),
            (PatchSafeEditResultState::Drifted, "drifted"),
            (PatchSafeEditResultState::Rejected, "rejected"),
            (PatchSafeEditResultState::ManualReviewRequired, "manual_review_required"),
        ] {
            assert_eq!(state.as_str(), label);
        }

        let digest_error = DigestBackedArtifactRef {
            digest: String::new(),
            artifact_kind: "log".to_string(),
            summary: "summary".to_string(),
            excerpt_anchor: None,
            resolve_path: "logs/error.log".to_string(),
        }
        .validate()
        .unwrap_err();
        assert_eq!(digest_error, ContextIntelligenceError::MissingDigestBackedArtifactDigest);

        let digest_kind_error = DigestBackedArtifactRef {
            digest: "fnv64:testdigest".to_string(),
            artifact_kind: String::new(),
            summary: "summary".to_string(),
            excerpt_anchor: None,
            resolve_path: "logs/error.log".to_string(),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            digest_kind_error,
            ContextIntelligenceError::MissingDigestBackedArtifactKind {
                digest: "fnv64:testdigest".to_string(),
            }
        );

        let digest_summary_error = DigestBackedArtifactRef {
            digest: "fnv64:testdigest".to_string(),
            artifact_kind: "log".to_string(),
            summary: String::new(),
            excerpt_anchor: None,
            resolve_path: "logs/error.log".to_string(),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            digest_summary_error,
            ContextIntelligenceError::MissingDigestBackedArtifactSummary {
                digest: "fnv64:testdigest".to_string(),
            }
        );

        let digest_path_error = DigestBackedArtifactRef {
            digest: "fnv64:testdigest".to_string(),
            artifact_kind: "log".to_string(),
            summary: "summary".to_string(),
            excerpt_anchor: None,
            resolve_path: String::new(),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            digest_path_error,
            ContextIntelligenceError::MissingDigestBackedArtifactResolvePath {
                digest: "fnv64:testdigest".to_string(),
            }
        );

        let entry_error = ContextPackEntryProjection {
            source_ref: "src/lib.rs".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            authority_rank: AuthorityRank::Structured,
            fidelity_tier: ContextFidelityTier::Critical,
            inclusion_mode: ContextInclusionMode::Excerpt,
            required_for_admission: true,
            reason: String::new(),
            resolved_excerpt_anchor: None,
            digest_ref: None,
            lifecycle_relevance: Some("implementation_surface".to_string()),
            risk_relevance: None,
            ranking_rationale: Some("origin=fts".to_string()),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            entry_error,
            ContextIntelligenceError::MissingContextPackEntryReason {
                source_ref: "src/lib.rs".to_string(),
            }
        );

        let entry_source_error = ContextPackEntryProjection {
            source_ref: String::new(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            authority_rank: AuthorityRank::Structured,
            fidelity_tier: ContextFidelityTier::Critical,
            inclusion_mode: ContextInclusionMode::Excerpt,
            required_for_admission: true,
            reason: "bounded excerpt".to_string(),
            resolved_excerpt_anchor: None,
            digest_ref: None,
            lifecycle_relevance: Some("implementation_surface".to_string()),
            risk_relevance: None,
            ranking_rationale: Some("origin=fts".to_string()),
        }
        .validate()
        .unwrap_err();
        assert_eq!(entry_source_error, ContextIntelligenceError::MissingContextPackEntrySourceRef);

        let omission_error = ContextOmissionFinding {
            severity: ContextOmissionSeverity::Blocking,
            reason_code: String::new(),
            candidate_ref: "src/lib.rs".to_string(),
            message: "critical context missing".to_string(),
            required_fidelity: Some(ContextFidelityTier::Critical),
            observed_mode: Some(ContextInclusionMode::Omitted),
        }
        .validate()
        .unwrap_err();
        assert_eq!(omission_error, ContextIntelligenceError::MissingOmissionReasonCode);

        let omission_candidate_error = ContextOmissionFinding {
            severity: ContextOmissionSeverity::Blocking,
            reason_code: "critical_unavailable".to_string(),
            candidate_ref: String::new(),
            message: "critical context missing".to_string(),
            required_fidelity: Some(ContextFidelityTier::Critical),
            observed_mode: Some(ContextInclusionMode::Omitted),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            omission_candidate_error,
            ContextIntelligenceError::MissingOmissionCandidateRef {
                reason_code: "critical_unavailable".to_string(),
            }
        );

        let omission_message_error = ContextOmissionFinding {
            severity: ContextOmissionSeverity::Blocking,
            reason_code: "critical_unavailable".to_string(),
            candidate_ref: "src/lib.rs".to_string(),
            message: String::new(),
            required_fidelity: Some(ContextFidelityTier::Critical),
            observed_mode: Some(ContextInclusionMode::Omitted),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            omission_message_error,
            ContextIntelligenceError::MissingOmissionMessage {
                reason_code: "critical_unavailable".to_string(),
            }
        );

        let patch_error = PatchSafeEditAttempt {
            target_ref: "src/lib.rs".to_string(),
            anchor_refs: Vec::new(),
            pre_apply_digest: "fnv64:test".to_string(),
            post_apply_verification: vec!["cargo test".to_string()],
            result_state: PatchSafeEditResultState::ManualReviewRequired,
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            patch_error,
            ContextIntelligenceError::MissingPatchSafeAnchorRefs {
                target_ref: "src/lib.rs".to_string(),
            }
        );

        let patch_target_error = PatchSafeEditAttempt {
            target_ref: String::new(),
            anchor_refs: vec!["src/lib.rs#start-anchor".to_string()],
            pre_apply_digest: "fnv64:test".to_string(),
            post_apply_verification: vec!["cargo test".to_string()],
            result_state: PatchSafeEditResultState::ManualReviewRequired,
        }
        .validate()
        .unwrap_err();
        assert_eq!(patch_target_error, ContextIntelligenceError::MissingPatchSafeTargetRef);

        let patch_digest_error = PatchSafeEditAttempt {
            target_ref: "src/lib.rs".to_string(),
            anchor_refs: vec!["src/lib.rs#start-anchor".to_string()],
            pre_apply_digest: String::new(),
            post_apply_verification: vec!["cargo test".to_string()],
            result_state: PatchSafeEditResultState::ManualReviewRequired,
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            patch_digest_error,
            ContextIntelligenceError::MissingPatchSafePreApplyDigest {
                target_ref: "src/lib.rs".to_string(),
            }
        );

        let patch_verification_error = PatchSafeEditAttempt {
            target_ref: "src/lib.rs".to_string(),
            anchor_refs: vec!["src/lib.rs#start-anchor".to_string()],
            pre_apply_digest: "fnv64:test".to_string(),
            post_apply_verification: Vec::new(),
            result_state: PatchSafeEditResultState::ManualReviewRequired,
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            patch_verification_error,
            ContextIntelligenceError::MissingPatchSafePostApplyVerification {
                target_ref: "src/lib.rs".to_string(),
            }
        );

        let serialized_projection = serde_json::to_value(AdvancedContextProjection {
            query_id: "query-domain-context-substrate".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Selected,
            retrieval_index_state: RetrievalIndexState::Ready,
            semantic_policy_state: SemanticPolicyState::Disabled,
            semantic_capability_state: SemanticCapabilityState::Unsupported,
            hybrid_outcome: HybridOutcome::BaselineOnly,
            budgets: RetrievalBudgets::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some("bounded context selected local evidence".to_string()),
            selected_evidence: vec![semantic_candidate()],
            rejected_candidates: Vec::new(),
            semantic_trace_records: Vec::new(),
            relationships: Vec::new(),
            impact_findings: Vec::new(),
            context_pack_entries: Vec::new(),
            omission_findings: Vec::new(),
            repository_map_state: None,
            snapshot_cache_state: None,
            patch_safe_edit_attempts: vec![PatchSafeEditAttempt {
                target_ref: "src/lib.rs".to_string(),
                anchor_refs: vec!["src/lib.rs#start-anchor".to_string()],
                pre_apply_digest: "fnv64:test".to_string(),
                post_apply_verification: vec!["cargo test".to_string()],
                result_state: PatchSafeEditResultState::ManualReviewRequired,
            }],
        })
        .unwrap();
        assert_eq!(
            serialized_projection
                .get("patch_safe_edit_attempts")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(1)
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
            context_pack_entries: Vec::new(),
            omission_findings: Vec::new(),
            repository_map_state: None,
            snapshot_cache_state: None,
            patch_safe_edit_attempts: Vec::new(),
        };

        assert_eq!(projection.semantic_selected_count(), 1);
        assert_eq!(projection.semantic_rejected_count(), 1);
        assert_eq!(semantic_candidate().semantic_score.unwrap().as_raw(), 0.812);
    }

    #[test]
    fn semantic_trace_helpers_cover_event_labels_and_validation_edges() {
        for (event_kind, label, requires_candidate_ref) in [
            (SemanticTraceEventKind::CapabilityEvaluated, "capability_evaluated", false),
            (SemanticTraceEventKind::ExtensionLoadAttempted, "extension_load_attempted", false),
            (SemanticTraceEventKind::IndexRefreshed, "index_refreshed", false),
            (SemanticTraceEventKind::ChunkBlocked, "chunk_blocked", true),
            (SemanticTraceEventKind::VectorQueryExecuted, "vector_query_executed", false),
            (SemanticTraceEventKind::VectorCandidatesReturned, "vector_candidates_returned", false),
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
            context_pack_entries: Vec::new(),
            omission_findings: Vec::new(),
            repository_map_state: None,
            snapshot_cache_state: None,
            patch_safe_edit_attempts: Vec::new(),
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

    #[test]
    fn digest_chunk_vector_and_operation_validators_cover_remaining_error_paths() {
        assert_eq!(
            SourceDigestRecord {
                source_ref: "src/lib.rs".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                content_hash: "sha256:abc".to_string(),
                compatibility_state: SourceDigestCompatibilityState::Compatible,
                authority_rank: AuthorityRank::Structured,
                last_indexed_at: None,
                chunk_count: 1,
                source_presence_state: SourcePresenceState::Deleted,
            }
            .validate(),
            Err(ContextIntelligenceError::InvalidDeletedSourceChunkCount {
                source_ref: "src/lib.rs".to_string(),
                chunk_count: 1,
            })
        );

        assert_eq!(
            SemanticChunkRecord {
                chunk_id: SemanticChunkRecord::stable_chunk_id("src/lib.rs", 0),
                source_ref: "src/lib.rs".to_string(),
                chunk_ordinal: 0,
                chunk_range: "1-3".to_string(),
                provenance_boundary: CanonSemanticProvenanceBoundary::Surface.as_str().to_string(),
                provenance_ref: "src/lib.rs".to_string(),
                content_hash: "sha256:def".to_string(),
                chunk_state: SemanticChunkState::Ready,
                embedding_dimensions: 0,
                canon_semantic_contract_line: None,
                semantic_labels: Vec::new(),
            }
            .validate(),
            Err(ContextIntelligenceError::InvalidSemanticChunkDimensions {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            })
        );

        assert_eq!(
            SemanticVectorRecord {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
                vector_schema_line: "sqlite_vec_v1".to_string(),
                embedding_dimensions: 0,
                write_generation: 1,
                vector_state: SemanticVectorState::Ready,
            }
            .validate(),
            Err(ContextIntelligenceError::InvalidSemanticVectorDimensions {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            })
        );

        assert_eq!(
            IndexMaintenanceOperation {
                operation_id: "operation-1".to_string(),
                command_name: IndexMaintenanceCommand::Refresh,
                trigger: IndexMaintenanceTrigger::Manual,
                pre_state: RetrievalIndexState::Missing,
                post_state: RetrievalIndexState::Stale,
                sources_scanned: 1,
                chunks_upserted: 0,
                chunks_deleted: 0,
                vector_rows_written: 0,
                fallback_reason: None,
                recommended_action: Some("  ".to_string()),
            }
            .validate(),
            Err(ContextIntelligenceError::MissingIndexOperationRecommendedAction {
                operation_id: "operation-1".to_string(),
            })
        );
    }

    #[test]
    fn digest_chunk_vector_and_operation_validators_accept_valid_records() {
        SourceDigestRecord {
            source_ref: "src/lib.rs".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            content_hash: "sha256:abc".to_string(),
            compatibility_state: SourceDigestCompatibilityState::Compatible,
            authority_rank: AuthorityRank::Structured,
            last_indexed_at: None,
            chunk_count: 1,
            source_presence_state: SourcePresenceState::Present,
        }
        .validate()
        .unwrap();

        SemanticChunkRecord {
            chunk_id: SemanticChunkRecord::stable_chunk_id("src/lib.rs", 0),
            source_ref: "src/lib.rs".to_string(),
            chunk_ordinal: 0,
            chunk_range: "1-3".to_string(),
            provenance_boundary: CanonSemanticProvenanceBoundary::Surface.as_str().to_string(),
            provenance_ref: "src/lib.rs".to_string(),
            content_hash: "sha256:def".to_string(),
            chunk_state: SemanticChunkState::Ready,
            embedding_dimensions: 1536,
            canon_semantic_contract_line: None,
            semantic_labels: Vec::new(),
        }
        .validate()
        .unwrap();

        SemanticVectorRecord {
            chunk_id: "semantic:src/lib.rs:0".to_string(),
            vector_schema_line: "sqlite_vec_v1".to_string(),
            embedding_dimensions: 1536,
            write_generation: 1,
            vector_state: SemanticVectorState::Ready,
        }
        .validate()
        .unwrap();

        IndexMaintenanceOperation {
            operation_id: "operation-2".to_string(),
            command_name: IndexMaintenanceCommand::Refresh,
            trigger: IndexMaintenanceTrigger::Manual,
            pre_state: RetrievalIndexState::Missing,
            post_state: RetrievalIndexState::Ready,
            sources_scanned: 1,
            chunks_upserted: 1,
            chunks_deleted: 0,
            vector_rows_written: 1,
            fallback_reason: None,
            recommended_action: None,
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn advanced_context_projection_custom_serializer_omits_empty_collections() {
        let projection = AdvancedContextProjection {
            query_id: "projection-serialize".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Degraded,
            retrieval_index_state: RetrievalIndexState::Degraded,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Degraded,
            hybrid_outcome: HybridOutcome::Skipped,
            budgets: RetrievalBudgets::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: None,
            selected_evidence: Vec::new(),
            rejected_candidates: Vec::new(),
            semantic_trace_records: Vec::new(),
            relationships: Vec::new(),
            impact_findings: Vec::new(),
            context_pack_entries: Vec::new(),
            omission_findings: Vec::new(),
            repository_map_state: None,
            snapshot_cache_state: None,
            patch_safe_edit_attempts: Vec::new(),
        };

        let serialized = serde_json::to_value(&projection).unwrap();
        let object = serialized.as_object().unwrap();
        assert_eq!(object.get("semantic_capability_state"), Some(&serde_json::json!("degraded")));
        assert_eq!(object.get("vector_query_count"), Some(&serde_json::json!(0)));
        assert!(!object.contains_key("selected_evidence"));
        assert!(!object.contains_key("impact_findings"));
    }
}
