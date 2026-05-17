//! Typed advanced-context intelligence models shared by planning, runtime,
//! session projections, and trace inspection.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    pub selection_state: CandidateSelectionState,
    pub selection_reason: String,
    pub provenance_summary: String,
    pub compatibility_state: RetrievalCompatibilityState,
    pub staleness_state: RetrievalStalenessState,
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

    /// Returns the impact-finding count recorded in this projection.
    pub fn impact_finding_count(&self) -> usize {
        self.impact_findings.len()
    }

    /// Validates that the projection remains internally consistent.
    pub fn validate(&self) -> Result<(), ContextIntelligenceError> {
        if self.query_id.trim().is_empty() {
            return Err(ContextIntelligenceError::MissingQueryId);
        }
        self.budgets.validate()?;

        if self.retrieval_state.is_selected() && self.selected_evidence.is_empty() {
            return Err(ContextIntelligenceError::MissingSelectedEvidence);
        }
        if !self.retrieval_state.is_selected()
            && self.terminal_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(ContextIntelligenceError::MissingTerminalReason);
        }
        if self.retrieval_mode != RetrievalMode::Remote && self.used_remote {
            return Err(ContextIntelligenceError::UnexpectedRemoteUsage);
        }
        if self.remote_policy_state != RemoteTransmissionPolicyState::RemoteAllowed
            && self.used_remote
        {
            return Err(ContextIntelligenceError::BlockedRemoteUsage);
        }

        for candidate in &self.selected_evidence {
            candidate.validate()?;
        }
        for candidate in &self.rejected_candidates {
            candidate.validate()?;
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

/// Errors returned while validating advanced-context projections.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContextIntelligenceError {
    #[error("advanced context projection requires a query id")]
    MissingQueryId,
    #[error("advanced context projection requires selected evidence for a selected state")]
    MissingSelectedEvidence,
    #[error("advanced context projection requires a terminal reason for non-selected states")]
    MissingTerminalReason,
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
