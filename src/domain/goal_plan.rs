//! Goal-plan domain models persisted across planning, status, and inspect.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::cluster::{ClusterDeliveryStory, ClusterSessionProjection};
use crate::domain::context_intelligence::AdvancedContextProjection;
use crate::domain::decision::{DecisionType, EvidenceRef};
use crate::domain::governance::{
    BacklogQualityAssessment, CompactedCanonMemory, PlanningAnalysisBacklogEvidence,
    planning_analysis_backlog_evidence,
};
use crate::domain::guidance::GuidanceGuardianProjection;
use crate::domain::session::{
    ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState, DelegationPacket,
    DelegationPacketState,
};
use crate::domain::trace::current_timestamp_millis;
use crate::domain::workflow::WorkflowProgressState;

/// Lifecycle state of a persisted goal-derived plan proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPlanStatus {
    Draft,
    Confirmed,
    Superseded,
}

/// Lightweight workspace signals collected during plan derivation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSignals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub file_count: usize,
    pub has_config: bool,
    pub has_canon: bool,
    pub has_tests: bool,
}

/// Credibility assigned to the bounded context pack that justified planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackCredibility {
    Credible,
    Insufficient,
    Stale,
}

impl ContextPackCredibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credible => "credible",
            Self::Insufficient => "insufficient",
            Self::Stale => "stale",
        }
    }
}

/// Provenance category for one context input carried into goal planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInputKind {
    WorkspaceFile,
    SymbolHint,
    AuthoredBrief,
    Negotiation,
    RecentTrace,
    DomainTemplate,
    DomainStandard,
    ExternalContextInput,
    CanonArtifact,
    CanonCapability,
    CanonMemory,
}

impl ContextInputKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceFile => "workspace_file",
            Self::SymbolHint => "symbol_hint",
            Self::AuthoredBrief => "authored_brief",
            Self::Negotiation => "negotiation",
            Self::RecentTrace => "recent_trace",
            Self::DomainTemplate => "domain_template",
            Self::DomainStandard => "domain_standard",
            Self::ExternalContextInput => "external_context_input",
            Self::CanonArtifact => "canon_artifact",
            Self::CanonCapability => "canon_capability",
            Self::CanonMemory => "canon_memory",
        }
    }
}

/// One bounded context input persisted with the plan so later surfaces can
/// explain why a target was selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextInput {
    pub kind: ContextInputKind,
    pub reference: String,
    pub rationale: String,
    pub source: String,
    #[serde(default)]
    pub primary: bool,
}

impl ContextInput {
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.reference.trim().is_empty() {
            return Err(GoalPlanError::MissingContextInputReference);
        }
        if self.rationale.trim().is_empty() {
            return Err(GoalPlanError::MissingContextInputRationale {
                reference: self.reference.clone(),
            });
        }
        if self.source.trim().is_empty() {
            return Err(GoalPlanError::MissingContextInputSource {
                reference: self.reference.clone(),
            });
        }
        Ok(())
    }

    pub fn provenance_line(&self) -> String {
        // Keep the source label operator-visible so local scans and Canon
        // enrichment remain distinguishable in runtime projections.
        format!(
            "{}: {} ({}) [source={}]",
            self.kind.as_str(),
            self.reference,
            self.rationale,
            self.source
        )
    }
}

/// Bounded planning context assembled from workspace evidence plus optional
/// authored, domain, and governance sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPack {
    pub pack_id: String,
    pub summary: String,
    pub credibility: ContextPackCredibility,
    #[serde(default)]
    pub inputs: Vec<ContextInput>,
    #[serde(default)]
    pub selected_targets: Vec<String>,
    /// Optional advanced-context retrieval projection carried with this plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advanced_context: Option<AdvancedContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness_reason: Option<String>,
}

impl ContextPack {
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.pack_id.trim().is_empty() {
            return Err(GoalPlanError::MissingContextPackId);
        }
        if self.summary.trim().is_empty() {
            return Err(GoalPlanError::MissingContextPackSummary);
        }
        for input in &self.inputs {
            input.validate()?;
        }
        if let Some(advanced_context) = &self.advanced_context {
            advanced_context
                .validate()
                .map_err(|error| GoalPlanError::InvalidAdvancedContext(error.to_string()))?;
        }
        if self.credibility == ContextPackCredibility::Credible
            && self.primary_inputs().is_empty()
            && self.selected_targets.is_empty()
        {
            return Err(GoalPlanError::MissingCredibleContextPrimaryInput);
        }
        if self.credibility == ContextPackCredibility::Stale
            && self.staleness_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(GoalPlanError::MissingContextStalenessReason);
        }
        Ok(())
    }

    pub fn primary_inputs(&self) -> Vec<&ContextInput> {
        self.inputs.iter().filter(|input| input.primary).collect()
    }

    pub fn primary_references(&self) -> Vec<String> {
        let primary = self
            .primary_inputs()
            .into_iter()
            .map(|input| input.reference.clone())
            .collect::<Vec<_>>();
        if primary.is_empty() { self.selected_targets.clone() } else { primary }
    }

    pub fn provenance_lines(&self) -> Vec<String> {
        self.inputs.iter().map(ContextInput::provenance_line).collect()
    }
}

/// Whether expert-pack routing selected a domain pack for the current plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExpertPackSelectionState {
    Selected,
    NoneSelected,
}

impl ExpertPackSelectionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::NoneSelected => "none-selected",
        }
    }
}

/// Outcome assigned to one expert-pack signal during pack selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExpertPackSignalStatus {
    Supporting,
    Ignored,
    Blocked,
}

impl ExpertPackSignalStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supporting => "supporting",
            Self::Ignored => "ignored",
            Self::Blocked => "blocked",
        }
    }
}

/// Whether a governed Canon expertise input was used or ignored during pack
/// selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonExpertiseInputDisposition {
    Used,
    Ignored,
}

impl CanonExpertiseInputDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Used => "used",
            Self::Ignored => "ignored",
        }
    }
}

/// One signal considered while selecting expert packs and runtime-role hints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpertPackSignal {
    pub kind: String,
    pub reference: String,
    pub source: String,
    pub status: ExpertPackSignalStatus,
    pub rationale: String,
}

impl ExpertPackSignal {
    pub fn provenance_line(&self) -> String {
        format!(
            "expert_pack_signal: {}={} [{}] ({}) [source={}]",
            self.kind,
            self.reference,
            self.status.as_str(),
            self.rationale,
            self.source
        )
    }
}

/// Candidate expert pack that was considered but rejected with an explicit
/// reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedExpertCandidate {
    pub pack_id: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_signals: Vec<ExpertPackSignal>,
}

impl RejectedExpertCandidate {
    pub fn summary_line(&self) -> String {
        format!("expert_pack_rejected: {} ({})", self.pack_id, self.reason)
    }
}

/// Persisted explanation of how a Canon expertise publication influenced expert
/// pack selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonExpertiseInputConsideration {
    pub contract_version: String,
    pub mode: String,
    pub expertise_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domain_families: Vec<String>,
    pub source_ref: String,
    pub promotion_state: String,
    pub publication_target_class: String,
    pub disposition: CanonExpertiseInputDisposition,
    pub disposition_reason: String,
}

impl CanonExpertiseInputConsideration {
    pub fn provenance_line(&self) -> String {
        let domain_families = if self.domain_families.is_empty() {
            "none".to_string()
        } else {
            self.domain_families.join(", ")
        };
        format!(
            "canon_expertise_input: {} [{}] families={} disposition={} ({}) source_ref={} target_class={} promotion_state={}",
            self.expertise_kind,
            self.mode,
            domain_families,
            self.disposition.as_str(),
            self.disposition_reason,
            self.source_ref,
            self.publication_target_class,
            self.promotion_state
        )
    }
}

/// Persisted result of expert-pack selection, including supporting and rejected
/// signals so operator surfaces can explain why a pack did or did not win.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpertPackSelectionOutcome {
    pub target_ref: String,
    pub selection_state: ExpertPackSelectionState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_expert_packs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_runtime_roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_signals: Vec<ExpertPackSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_expert_candidates: Vec<RejectedExpertCandidate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canon_inputs_considered: Vec<CanonExpertiseInputConsideration>,
    pub summary: String,
}

impl ExpertPackSelectionOutcome {
    pub fn provenance_lines(&self) -> Vec<String> {
        let mut lines = self
            .supporting_signals
            .iter()
            .map(ExpertPackSignal::provenance_line)
            .collect::<Vec<_>>();
        lines.extend(
            self.rejected_expert_candidates.iter().map(RejectedExpertCandidate::summary_line),
        );
        lines.extend(
            self.canon_inputs_considered
                .iter()
                .map(CanonExpertiseInputConsideration::provenance_line),
        );
        lines
    }
}

/// An inferred flow proposal attached to a goal plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferredFlow {
    pub flow_name: String,
    pub confidence_reason: String,
    #[serde(default)]
    pub confirmed: bool,
}

/// Persisted flow state projected from the goal plan into status and inspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPlanFlowMode {
    Proposed,
    Confirmed,
    Skipped,
    Absent,
}

impl GoalPlanFlowMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Confirmed => "confirmed",
            Self::Skipped => "skipped",
            Self::Absent => "absent",
        }
    }
}

/// Flattened flow projection for operator-facing views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalPlanFlowState {
    pub mode: GoalPlanFlowMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_reason: Option<String>,
}

impl GoalPlanFlowState {
    pub fn summary_text(&self) -> String {
        match (self.flow_name.as_deref(), self.confidence_reason.as_deref()) {
            (Some(flow_name), Some(confidence_reason)) => {
                format!("{} ({flow_name}) - {confidence_reason}", self.mode.as_str())
            }
            (Some(flow_name), None) => format!("{} ({flow_name})", self.mode.as_str()),
            _ => self.mode.as_str().to_string(),
        }
    }
}

/// Planning quality state projected for host outputs and status surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanQualityState {
    Ready,
    ClarificationRequired,
    Blocked,
}

const PLAN_QUALITY_FINDING_PLANNING_RATIONALE: &str = "planning_rationale";
const PLAN_QUALITY_FINDING_VERIFICATION_STRATEGY: &str = "verification_strategy";
const PLAN_QUALITY_FINDING_CONTEXT_PACK_INSUFFICIENT: &str = "context_pack_insufficient";
const PLAN_QUALITY_FINDING_CONTEXT_PACK_STALE: &str = "context_pack_stale";
const PLAN_QUALITY_ASSUMPTION_DEFAULT_ROUTE_OVERRIDE: &str =
    "no explicit route override is required for this plan";
const PLANNING_ANALYSIS_CODE_SUCCESS_CRITERION_UNCOVERED: &str = "success_criterion_uncovered";
const PLANNING_ANALYSIS_CODE_VALIDATION_COVERAGE_MISSING: &str = "validation_coverage_missing";
const PLANNING_ANALYSIS_CODE_ARTIFACT_CONTRADICTION: &str = "artifact_contradiction";
const PLANNING_ANALYSIS_CODE_EXECUTION_INPUT_MISSING: &str = "execution_input_missing";
const PLANNING_ANALYSIS_CODE_PRODUCER_CONTRACT_GAP: &str = "producer_contract_gap";
const PLANNING_ANALYSIS_CODE_EXPECTED_OUTCOME_MISSING: &str = "expected_outcome_missing";
const PLANNING_ANALYSIS_CODE_COVERAGE_SIGNAL_PARTIAL: &str = "coverage_signal_partial";
const PLANNING_ANALYSIS_MESSAGE_UNMAPPED_ITEMS: &str =
    "required success criteria are not covered by the active planning packet";
const PLANNING_ANALYSIS_MESSAGE_MISSING_EXPECTED_OUTCOMES: &str =
    "planned tasks are missing measurable expected outcomes";
const PLANNING_ANALYSIS_MESSAGE_VALIDATION_COVERAGE_MISSING: &str =
    "selected slice is missing a matching acceptance anchor";
const PLANNING_ANALYSIS_MESSAGE_ARTIFACT_CONTRADICTION: &str =
    "execution handoff conflicts with the sequenced backlog slice order";
const PLANNING_ANALYSIS_MESSAGE_EXECUTION_INPUT_MISSING: &str =
    "execution handoff is missing implementation artifact references";
const PLANNING_ANALYSIS_MESSAGE_PRODUCER_CONTRACT_GAP: &str =
    "execution handoff requires Canon-authored dependency prerequisites";
const PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT: &str = "backlog_document";
const PLANNING_ANALYSIS_ARTIFACT_KIND_GOAL_PLAN: &str = "goal_plan";
const PLANNING_ANALYSIS_ARTIFACT_KIND_VERIFICATION_STRATEGY: &str = "verification_strategy";
const PLANNING_ANALYSIS_ARTIFACT_REF_SUCCESS_CRITERIA: &str = "success_criteria";
const PLANNING_ANALYSIS_ARTIFACT_REF_VERIFICATION_STRATEGY: &str = "verification_strategy";
const BACKLOG_DOCUMENT_ACCEPTANCE_ANCHORS: &str = "acceptance-anchors.md";
const BACKLOG_DOCUMENT_EXECUTION_HANDOFF: &str = "execution-handoff.md";
const BACKLOG_DOCUMENT_SEQUENCING_PLAN: &str = "sequencing-plan.md";
const BACKLOG_DOCUMENT_PLANNING_RISKS: &str = "planning-risks.md";
const PLANNING_ANALYSIS_ANCHOR_DEPENDENCY_PREREQUISITES: &str = "Dependency Prerequisites";

impl PlanQualityState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ClarificationRequired => "clarification_required",
            Self::Blocked => "blocked",
        }
    }
}

/// Additive quality projection for the generated plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanQualityAssessment {
    pub state: PlanQualityState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,
}

impl Default for PlanQualityAssessment {
    fn default() -> Self {
        Self { state: PlanQualityState::Ready, findings: Vec::new(), assumptions: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningAnalysisState {
    Clean,
    Findings,
    Blocked,
}

impl PlanningAnalysisState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningAnalysisSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl PlanningAnalysisSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningAnalysisSource {
    Goal,
    Plan,
    Backlog,
    Validation,
    Risk,
    Constraint,
    ExecutionReadiness,
    GovernedEvidence,
}

impl PlanningAnalysisSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Goal => "goal",
            Self::Plan => "plan",
            Self::Backlog => "backlog",
            Self::Validation => "validation",
            Self::Risk => "risk",
            Self::Constraint => "constraint",
            Self::ExecutionReadiness => "execution_readiness",
            Self::GovernedEvidence => "governed_evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningAnalysisSourceRef {
    pub artifact_kind: String,
    pub artifact_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningAnalysisFinding {
    pub severity: PlanningAnalysisSeverity,
    pub source: PlanningAnalysisSource,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<PlanningAnalysisSourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlanningAnalysisCoverage {
    pub success_criteria_total: usize,
    pub success_criteria_covered: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backlog_slice_total: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backlog_slice_covered: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_anchor_total: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_anchor_covered: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_total: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_covered: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraint_total: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraint_covered: Option<usize>,
    #[serde(default)]
    pub governed_evidence_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningAnalysisProjection {
    pub state: PlanningAnalysisState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<PlanningAnalysisFinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<PlanningAnalysisCoverage>,
}

impl Default for PlanningAnalysisProjection {
    fn default() -> Self {
        Self { state: PlanningAnalysisState::Clean, findings: Vec::new(), coverage: None }
    }
}

/// A single planned task in a goal-derived plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedTask {
    pub task_id: String,
    pub description: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_type_hint: Option<DecisionType>,
}

impl PlannedTask {
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.task_id.trim().is_empty() {
            return Err(GoalPlanError::MissingTaskId);
        }
        if self.description.trim().is_empty() {
            return Err(GoalPlanError::MissingTaskDescription { task_id: self.task_id.clone() });
        }
        if self.target.trim().is_empty() {
            return Err(GoalPlanError::MissingTaskTarget { task_id: self.task_id.clone() });
        }
        Ok(())
    }
}

/// Persisted bounded task draft derived from goal text, workspace evidence,
/// authored documents, governance context, and planning-time guidance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalPlan {
    pub plan_id: String,
    pub goal_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_goal_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_resolution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_acceptance_boundary: Option<String>,
    pub tasks: Vec<PlannedTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_pack: Option<ContextPack>,
    #[serde(default)]
    pub source_evidence: Vec<EvidenceRef>,
    #[serde(default)]
    pub workspace_signals: WorkspaceSignals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_policy_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_strategy: Option<String>,
    #[serde(default)]
    pub plan_quality: PlanQualityAssessment,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_analysis: Option<PlanningAnalysisProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<InferredFlow>,
    #[serde(default)]
    pub flow_skipped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_progress: Option<WorkflowProgressState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compacted_canon_memory: Option<CompactedCanonMemory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expert_selection: Option<ExpertPackSelectionOutcome>,
    /// Flattened guidance and guardian projection reused by status and inspect.
    #[serde(flatten)]
    pub guidance_guardian: GuidanceGuardianProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_session_projection: Option<ClusterSessionProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_delivery_story: Option<ClusterDeliveryStory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegation_packet_history: Vec<DelegationPacket>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_continuity: Option<DelegationContinuityState>,
    #[serde(default = "default_goal_plan_revision")]
    pub proposal_revision: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by_revision: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_at: Option<u64>,
    pub created_at: u64,
    pub status: GoalPlanStatus,
}

const fn default_goal_plan_revision() -> usize {
    1
}

impl GoalPlan {
    /// Creates a new draft plan and validates the initial task set immediately.
    pub fn new(
        goal_text: impl Into<String>,
        tasks: Vec<PlannedTask>,
    ) -> Result<Self, GoalPlanError> {
        let mut plan = Self {
            plan_id: Uuid::new_v4().to_string(),
            goal_text: goal_text.into(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            tasks,
            context_pack: None,
            source_evidence: Vec::new(),
            workspace_signals: WorkspaceSignals::default(),
            routing_policy_summary: None,
            planning_rationale: None,
            verification_strategy: None,
            plan_quality: PlanQualityAssessment::default(),
            planning_analysis: None,
            flow: None,
            flow_skipped: false,
            workflow_progress: None,
            compacted_canon_memory: None,
            expert_selection: None,
            guidance_guardian: GuidanceGuardianProjection::default(),
            cluster_session_projection: None,
            cluster_delivery_story: None,
            delegation_packet_history: Vec::new(),
            delegation_continuity: None,
            proposal_revision: default_goal_plan_revision(),
            superseded_by_revision: None,
            superseded_reason: None,
            confirmed_at: None,
            created_at: current_timestamp_millis(),
            status: GoalPlanStatus::Draft,
        };
        plan.refresh_plan_quality();
        plan.validate()?;
        Ok(plan)
    }

    /// Validates the persisted plan shape and nested projections.
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.goal_text.trim().is_empty() {
            return Err(GoalPlanError::MissingGoalText);
        }
        if self.tasks.is_empty() {
            return Err(GoalPlanError::NoTasks);
        }
        for task in &self.tasks {
            task.validate()?;
        }
        if let Some(context_pack) = &self.context_pack {
            context_pack.validate()?;
        }
        if let Some(workflow_progress) = &self.workflow_progress {
            workflow_progress
                .validate()
                .map_err(|error| GoalPlanError::InvalidWorkflowProgress(error.to_string()))?;
        }
        for packet in &self.delegation_packet_history {
            packet.validate().map_err(GoalPlanError::InvalidDelegationPacket)?;
        }
        if let Some(continuity) = &self.delegation_continuity {
            continuity
                .validate(&self.delegation_packet_history)
                .map_err(GoalPlanError::InvalidDelegationContinuity)?;
        }
        if let Some(projection) = &self.cluster_session_projection {
            projection
                .validate()
                .map_err(|error| GoalPlanError::InvalidClusterProjection(error.to_string()))?;
        }
        if let Some(story) = &self.cluster_delivery_story {
            story
                .validate()
                .map_err(|error| GoalPlanError::InvalidClusterDeliveryStory(error.to_string()))?;
        }
        if self.proposal_revision == 0 {
            return Err(GoalPlanError::MissingProposalRevision);
        }
        Ok(())
    }

    /// Returns true when the plan is still a proposal and requires explicit confirmation.
    pub fn requires_confirmation(&self) -> bool {
        self.status == GoalPlanStatus::Draft
    }

    /// Human-readable proposal state used by operator-facing surfaces.
    pub fn proposal_state_text(&self) -> &'static str {
        match self.status {
            GoalPlanStatus::Draft => "proposed",
            GoalPlanStatus::Confirmed => "confirmed",
            GoalPlanStatus::Superseded => "superseded",
        }
    }

    /// Confirms a draft plan and latches any inferred flow as confirmed.
    pub fn confirm(&mut self) -> Result<(), GoalPlanError> {
        if self.status != GoalPlanStatus::Draft {
            return Err(GoalPlanError::InvalidTransition {
                from: self.status,
                to: GoalPlanStatus::Confirmed,
            });
        }
        self.status = GoalPlanStatus::Confirmed;
        if let Some(flow) = self.flow.as_mut() {
            flow.confirmed = true;
        }
        self.confirmed_at = Some(current_timestamp_millis());
        Ok(())
    }

    /// Marks a confirmed plan as superseded by a later revision.
    pub fn supersede(&mut self) -> Result<(), GoalPlanError> {
        if self.status != GoalPlanStatus::Confirmed {
            return Err(GoalPlanError::InvalidTransition {
                from: self.status,
                to: GoalPlanStatus::Superseded,
            });
        }
        self.status = GoalPlanStatus::Superseded;
        Ok(())
    }

    pub fn supersede_with(
        &mut self,
        superseded_by_revision: usize,
        reason: impl Into<String>,
    ) -> Result<(), GoalPlanError> {
        self.supersede()?;
        self.superseded_by_revision = Some(superseded_by_revision);
        self.superseded_reason = Some(reason.into());
        Ok(())
    }

    pub fn with_signals(mut self, signals: WorkspaceSignals) -> Self {
        self.workspace_signals = signals;
        self
    }

    pub fn with_planning_rationale(mut self, planning_rationale: impl Into<String>) -> Self {
        self.planning_rationale = Some(planning_rationale.into());
        self.refresh_plan_quality();
        self
    }

    pub fn with_routing_policy_summary(
        mut self,
        routing_policy_summary: impl Into<String>,
    ) -> Self {
        self.routing_policy_summary = Some(routing_policy_summary.into());
        self
    }

    pub fn with_verification_strategy(mut self, verification_strategy: impl Into<String>) -> Self {
        self.verification_strategy = Some(verification_strategy.into());
        self.refresh_plan_quality();
        self
    }

    pub fn plan_quality_state(&self) -> Option<String> {
        Some(self.plan_quality_assessment().state.as_str().to_string())
    }

    pub fn plan_quality_findings(&self) -> Option<Vec<String>> {
        let assessment = self.plan_quality_assessment();
        (!assessment.findings.is_empty()).then_some(assessment.findings)
    }

    pub fn plan_quality_assumptions(&self) -> Option<Vec<String>> {
        let assessment = self.plan_quality_assessment();
        (!assessment.assumptions.is_empty()).then_some(assessment.assumptions)
    }

    pub fn plan_quality_assessment(&self) -> PlanQualityAssessment {
        self.assess_plan_quality()
    }

    pub fn planning_analysis_projection(
        &self,
        backlog_quality: &BacklogQualityAssessment,
        backlog_document_refs: &[String],
        backlog_documents: &[String],
    ) -> PlanningAnalysisProjection {
        let backlog_evidence =
            planning_analysis_backlog_evidence(backlog_document_refs, backlog_documents);
        let success_criteria_total = self.tasks.len();
        let uncovered_success_criteria = deduplicated_items(&backlog_quality.unmapped_items);
        let success_criteria_covered =
            success_criteria_total.saturating_sub(uncovered_success_criteria.len());
        let explicit_constraint_total = self.explicit_constraint_count();
        let validation_anchor_total =
            planning_analysis_validation_anchor_total(&backlog_evidence, backlog_document_refs);
        let validation_anchor_covered = planning_analysis_validation_anchor_covered(
            self,
            &backlog_evidence,
            validation_anchor_total,
        );
        let risk_covered = planning_analysis_risk_covered(self, &backlog_evidence);
        let constraint_covered =
            (explicit_constraint_total > 0).then_some(explicit_constraint_total);
        let governed_evidence_ready =
            planning_analysis_governed_evidence_ready(backlog_document_refs, &backlog_evidence);
        let coverage = PlanningAnalysisCoverage {
            success_criteria_total,
            success_criteria_covered,
            backlog_slice_total: (!backlog_evidence.slice_ids.is_empty())
                .then_some(backlog_evidence.slice_ids.len()),
            backlog_slice_covered: planning_analysis_backlog_slice_covered(&backlog_evidence),
            validation_anchor_total,
            validation_anchor_covered,
            risk_total: (backlog_evidence.planning_risk_count > 0)
                .then_some(backlog_evidence.planning_risk_count),
            risk_covered,
            constraint_total: (explicit_constraint_total > 0).then_some(explicit_constraint_total),
            constraint_covered,
            governed_evidence_ready,
        };
        let mut findings = Vec::new();

        if !uncovered_success_criteria.is_empty() {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Critical,
                source: PlanningAnalysisSource::Goal,
                code: PLANNING_ANALYSIS_CODE_SUCCESS_CRITERION_UNCOVERED.to_string(),
                message: format!(
                    "{PLANNING_ANALYSIS_MESSAGE_UNMAPPED_ITEMS}: {}",
                    uncovered_success_criteria.join(", ")
                ),
                source_refs: vec![PlanningAnalysisSourceRef {
                    artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_GOAL_PLAN.to_string(),
                    artifact_ref: PLANNING_ANALYSIS_ARTIFACT_REF_SUCCESS_CRITERIA.to_string(),
                    anchor: uncovered_success_criteria.first().cloned(),
                }],
            });
        }

        if let Some(selected_slice_id) = backlog_evidence.selected_slice_id.as_ref()
            && validation_anchor_total == Some(1)
            && validation_anchor_covered == Some(0)
        {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Critical,
                source: PlanningAnalysisSource::Validation,
                code: PLANNING_ANALYSIS_CODE_VALIDATION_COVERAGE_MISSING.to_string(),
                message: PLANNING_ANALYSIS_MESSAGE_VALIDATION_COVERAGE_MISSING.to_string(),
                source_refs: vec![
                    PlanningAnalysisSourceRef {
                        artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_VERIFICATION_STRATEGY
                            .to_string(),
                        artifact_ref: PLANNING_ANALYSIS_ARTIFACT_REF_VERIFICATION_STRATEGY
                            .to_string(),
                        anchor: None,
                    },
                    PlanningAnalysisSourceRef {
                        artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                        artifact_ref: BACKLOG_DOCUMENT_ACCEPTANCE_ANCHORS.to_string(),
                        anchor: Some(format!("slice_id={selected_slice_id}")),
                    },
                ],
            });
        }

        if let (Some(first_sequenced_slice_id), Some(selected_slice_id)) = (
            backlog_evidence.first_sequenced_slice_id.as_ref(),
            backlog_evidence.selected_slice_id.as_ref(),
        ) && first_sequenced_slice_id != selected_slice_id
        {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Critical,
                source: PlanningAnalysisSource::Backlog,
                code: PLANNING_ANALYSIS_CODE_ARTIFACT_CONTRADICTION.to_string(),
                message: PLANNING_ANALYSIS_MESSAGE_ARTIFACT_CONTRADICTION.to_string(),
                source_refs: vec![
                    PlanningAnalysisSourceRef {
                        artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                        artifact_ref: BACKLOG_DOCUMENT_SEQUENCING_PLAN.to_string(),
                        anchor: Some(format!("slice_id={first_sequenced_slice_id}")),
                    },
                    PlanningAnalysisSourceRef {
                        artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                        artifact_ref: BACKLOG_DOCUMENT_EXECUTION_HANDOFF.to_string(),
                        anchor: Some(format!("slice_id={selected_slice_id}")),
                    },
                ],
            });
        }

        if !backlog_document_refs.is_empty()
            && backlog_evidence.selected_slice_id.is_some()
            && backlog_evidence.dependency_prerequisite_count == 0
        {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Critical,
                source: PlanningAnalysisSource::GovernedEvidence,
                code: PLANNING_ANALYSIS_CODE_PRODUCER_CONTRACT_GAP.to_string(),
                message: PLANNING_ANALYSIS_MESSAGE_PRODUCER_CONTRACT_GAP.to_string(),
                source_refs: vec![PlanningAnalysisSourceRef {
                    artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                    artifact_ref: BACKLOG_DOCUMENT_EXECUTION_HANDOFF.to_string(),
                    anchor: Some(PLANNING_ANALYSIS_ANCHOR_DEPENDENCY_PREREQUISITES.to_string()),
                }],
            });
        }

        if !backlog_document_refs.is_empty()
            && backlog_evidence.selected_slice_id.is_some()
            && backlog_evidence.implementation_artifact_ref_count == 0
        {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Critical,
                source: PlanningAnalysisSource::ExecutionReadiness,
                code: PLANNING_ANALYSIS_CODE_EXECUTION_INPUT_MISSING.to_string(),
                message: PLANNING_ANALYSIS_MESSAGE_EXECUTION_INPUT_MISSING.to_string(),
                source_refs: vec![PlanningAnalysisSourceRef {
                    artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                    artifact_ref: BACKLOG_DOCUMENT_EXECUTION_HANDOFF.to_string(),
                    anchor: Some("Implementation Artifact References".to_string()),
                }],
            });
        }

        let tasks_missing_expected_outcomes = self
            .tasks
            .iter()
            .filter(|task| {
                task.expected_outcome.as_deref().map(str::trim).unwrap_or_default().is_empty()
            })
            .map(|task| task.task_id.clone())
            .collect::<Vec<_>>();
        if !tasks_missing_expected_outcomes.is_empty() {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Medium,
                source: PlanningAnalysisSource::Plan,
                code: PLANNING_ANALYSIS_CODE_EXPECTED_OUTCOME_MISSING.to_string(),
                message: format!(
                    "{PLANNING_ANALYSIS_MESSAGE_MISSING_EXPECTED_OUTCOMES}: {}",
                    tasks_missing_expected_outcomes.join(", ")
                ),
                source_refs: tasks_missing_expected_outcomes
                    .iter()
                    .map(|task_id| PlanningAnalysisSourceRef {
                        artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_GOAL_PLAN.to_string(),
                        artifact_ref: task_id.clone(),
                        anchor: None,
                    })
                    .collect(),
            });
        }

        if let (Some(risk_total), Some(risk_covered)) = (coverage.risk_total, coverage.risk_covered)
            && risk_covered < risk_total
        {
            findings.push(PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Medium,
                source: PlanningAnalysisSource::Risk,
                code: PLANNING_ANALYSIS_CODE_COVERAGE_SIGNAL_PARTIAL.to_string(),
                message: "planning risk coverage remains partial for execution readiness"
                    .to_string(),
                source_refs: vec![PlanningAnalysisSourceRef {
                    artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                    artifact_ref: BACKLOG_DOCUMENT_PLANNING_RISKS.to_string(),
                    anchor: None,
                }],
            });
        }

        let findings = deduplicate_planning_analysis_findings(findings);
        PlanningAnalysisProjection {
            state: planning_analysis_state_for_findings(&findings),
            findings,
            coverage: Some(coverage),
        }
    }

    pub fn planning_analysis_state(&self) -> Option<String> {
        self.planning_analysis.as_ref().map(|projection| projection.state.as_str().to_string())
    }

    pub fn planning_analysis_findings(&self) -> Option<Vec<PlanningAnalysisFinding>> {
        self.planning_analysis.as_ref().and_then(|projection| {
            (!projection.findings.is_empty()).then_some(projection.findings.clone())
        })
    }

    pub fn planning_analysis_coverage(&self) -> Option<PlanningAnalysisCoverage> {
        self.planning_analysis.as_ref().and_then(|projection| projection.coverage.clone())
    }

    fn refresh_plan_quality(&mut self) {
        self.plan_quality = self.assess_plan_quality();
    }

    fn assess_plan_quality(&self) -> PlanQualityAssessment {
        let mut findings = Vec::new();
        let mut assumptions = Vec::new();

        if self.routing_policy_summary.is_none() {
            assumptions.push(PLAN_QUALITY_ASSUMPTION_DEFAULT_ROUTE_OVERRIDE.to_string());
        }

        if let Some(context_pack) = &self.context_pack {
            match context_pack.credibility {
                ContextPackCredibility::Insufficient => {
                    findings.push(PLAN_QUALITY_FINDING_CONTEXT_PACK_INSUFFICIENT.to_string());
                }
                ContextPackCredibility::Stale => {
                    findings.push(PLAN_QUALITY_FINDING_CONTEXT_PACK_STALE.to_string());
                }
                ContextPackCredibility::Credible => {}
            }
        }

        if self.planning_rationale.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            findings.push(PLAN_QUALITY_FINDING_PLANNING_RATIONALE.to_string());
        }
        if self.verification_strategy.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            findings.push(PLAN_QUALITY_FINDING_VERIFICATION_STRATEGY.to_string());
        }
        PlanQualityAssessment {
            state: if findings.iter().any(|finding| {
                matches!(
                    finding.as_str(),
                    PLAN_QUALITY_FINDING_CONTEXT_PACK_INSUFFICIENT
                        | PLAN_QUALITY_FINDING_CONTEXT_PACK_STALE
                )
            }) {
                PlanQualityState::Blocked
            } else if findings.is_empty() {
                PlanQualityState::Ready
            } else {
                PlanQualityState::ClarificationRequired
            },
            findings,
            assumptions,
        }
    }

    fn explicit_constraint_count(&self) -> usize {
        [self.negotiation_acceptance_boundary.as_deref(), self.routing_policy_summary.as_deref()]
            .into_iter()
            .filter_map(|value| value.map(str::trim))
            .filter(|value| !value.is_empty())
            .count()
    }

    /// Returns the next monotonically increasing proposal revision number.
    pub fn next_revision(&self) -> usize {
        self.proposal_revision + 1
    }

    pub fn with_flow(mut self, flow: InferredFlow) -> Self {
        self.flow = Some(flow);
        self.flow_skipped = false;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<EvidenceRef>) -> Self {
        self.source_evidence = evidence;
        self
    }

    pub fn with_context_pack(mut self, context_pack: ContextPack) -> Self {
        self.context_pack = Some(context_pack);
        self
    }

    pub fn with_workflow_progress(mut self, workflow_progress: WorkflowProgressState) -> Self {
        self.workflow_progress = Some(workflow_progress);
        self
    }

    pub fn with_compacted_canon_memory(
        mut self,
        compacted_canon_memory: CompactedCanonMemory,
    ) -> Self {
        self.compacted_canon_memory = Some(compacted_canon_memory);
        self
    }

    pub fn with_expert_selection(mut self, expert_selection: ExpertPackSelectionOutcome) -> Self {
        self.expert_selection = Some(expert_selection);
        self
    }

    pub fn with_guidance_guardian(mut self, guidance_guardian: GuidanceGuardianProjection) -> Self {
        self.guidance_guardian = guidance_guardian;
        self
    }

    pub fn with_delegation_state(
        mut self,
        packet_history: Vec<DelegationPacket>,
        continuity: DelegationContinuityState,
    ) -> Result<Self, GoalPlanError> {
        self.delegation_packet_history = packet_history;
        self.delegation_continuity = Some(continuity);
        self.validate()?;
        Ok(self)
    }

    pub fn with_negotiation_projection(
        mut self,
        goal_summary: impl Into<String>,
        resolution: impl Into<String>,
        acceptance_boundary: impl Into<String>,
    ) -> Self {
        self.negotiation_goal_summary = Some(goal_summary.into());
        self.negotiation_resolution = Some(resolution.into());
        self.negotiation_acceptance_boundary = Some(acceptance_boundary.into());
        self
    }

    pub fn mark_flow_skipped(&mut self) {
        self.flow = None;
        self.flow_skipped = true;
    }

    /// Returns the flattened flow state projected into status and inspect views.
    pub fn flow_state(&self) -> GoalPlanFlowState {
        match self.flow.as_ref() {
            Some(flow) => GoalPlanFlowState {
                mode: if flow.confirmed {
                    GoalPlanFlowMode::Confirmed
                } else {
                    GoalPlanFlowMode::Proposed
                },
                flow_name: Some(flow.flow_name.clone()),
                confidence_reason: Some(flow.confidence_reason.clone()),
            },
            None => GoalPlanFlowState {
                mode: if self.flow_skipped {
                    GoalPlanFlowMode::Skipped
                } else {
                    GoalPlanFlowMode::Absent
                },
                flow_name: None,
                confidence_reason: None,
            },
        }
    }

    pub fn workflow_name(&self) -> Option<String> {
        self.workflow_progress.as_ref().map(|workflow| workflow.workflow_name.clone())
    }

    pub fn workflow_phase_text(&self) -> Option<String> {
        self.workflow_progress.as_ref().and_then(WorkflowProgressState::current_phase_text)
    }

    pub fn workflow_next_action(&self) -> Option<String> {
        self.workflow_progress.as_ref().and_then(WorkflowProgressState::next_action_text)
    }

    pub fn delegation_continuity(&self) -> Option<&DelegationContinuityState> {
        self.delegation_continuity.as_ref()
    }

    pub fn delegation_packet_history(&self) -> &[DelegationPacket] {
        &self.delegation_packet_history
    }

    pub fn active_delegation_packet(&self) -> Option<&DelegationPacket> {
        let active_packet_id = self.delegation_continuity.as_ref()?.active_packet_id.as_deref()?;
        self.delegation_packet_history.iter().find(|packet| packet.packet_id == active_packet_id)
    }

    pub fn record_delegation_packet(
        &mut self,
        packet: DelegationPacket,
        continuity: DelegationContinuityState,
    ) -> Result<(), GoalPlanError> {
        packet.validate().map_err(GoalPlanError::InvalidDelegationPacket)?;

        let mut history = self.delegation_packet_history.clone();
        let next_packet_id = packet.packet_id.clone();
        if let Some(active_packet_id) = self
            .delegation_continuity
            .as_ref()
            .and_then(|state| state.active_packet_id.as_ref())
            .filter(|active_packet_id| *active_packet_id != &next_packet_id)
            && let Some(existing_packet) = history
                .iter_mut()
                .find(|existing_packet| existing_packet.packet_id == *active_packet_id)
            && matches!(
                existing_packet.state,
                DelegationPacketState::Active | DelegationPacketState::Stuck
            )
        {
            existing_packet.mark_superseded(next_packet_id.clone());
        }

        if let Some(existing_packet) =
            history.iter_mut().find(|existing_packet| existing_packet.packet_id == next_packet_id)
        {
            *existing_packet = packet;
        } else {
            history.push(packet);
        }

        continuity.validate(&history).map_err(GoalPlanError::InvalidDelegationContinuity)?;
        self.delegation_packet_history = history;
        self.delegation_continuity = Some(continuity);
        Ok(())
    }

    pub fn resolve_active_delegation(
        &mut self,
        headline: impl Into<String>,
        evidence_summary: impl Into<String>,
        next_command: impl Into<String>,
    ) -> Result<(), GoalPlanError> {
        let Some(active_packet_id) =
            self.delegation_continuity.as_ref().and_then(|state| state.active_packet_id.clone())
        else {
            return Ok(());
        };

        let packet = self
            .delegation_packet_history
            .iter_mut()
            .find(|packet| packet.packet_id == active_packet_id)
            .ok_or_else(|| {
                GoalPlanError::InvalidDelegationContinuity(format!(
                    "delegation active_packet_id `{active_packet_id}` is missing from history"
                ))
            })?;
        packet.mark_resolved();

        let continuity = DelegationContinuityState {
            active_packet_id: None,
            mode: DelegationContinuityMode::Resolved,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: next_command.into(),
            headline: headline.into(),
            evidence_summary: evidence_summary.into(),
        };
        continuity
            .validate(&self.delegation_packet_history)
            .map_err(GoalPlanError::InvalidDelegationContinuity)?;
        self.delegation_continuity = Some(continuity);
        Ok(())
    }

    /// Returns a compact context summary combining workspace context, Canon
    /// memory, and expert selection when present.
    pub fn context_summary(&self) -> Option<String> {
        let base_summary = match (
            self.context_pack.as_ref().map(|pack| pack.summary.clone()),
            self.compacted_canon_memory.as_ref().map(CompactedCanonMemory::summary_text),
        ) {
            (Some(context_summary), Some(canon_summary)) => {
                Some(format!("{context_summary}; canon memory: {canon_summary}"))
            }
            (Some(context_summary), None) => Some(context_summary),
            (None, Some(canon_summary)) => Some(format!("canon memory: {canon_summary}")),
            (None, None) => None,
        };

        match (
            base_summary,
            self.expert_selection.as_ref().map(|selection| selection.summary.clone()),
        ) {
            (Some(context_summary), Some(selection_summary)) => {
                Some(format!("{context_summary}; expert selection: {selection_summary}"))
            }
            (Some(context_summary), None) => Some(context_summary),
            (None, Some(selection_summary)) => {
                Some(format!("expert selection: {selection_summary}"))
            }
            (None, None) => None,
        }
    }

    /// Returns the credibility label for the persisted bounded context pack.
    pub fn context_credibility(&self) -> Option<String> {
        self.context_pack.as_ref().map(|pack| pack.credibility.as_str().to_string())
    }

    /// Returns the most important context references that justified planning.
    pub fn context_primary_inputs(&self) -> Vec<String> {
        let mut inputs =
            self.context_pack.as_ref().map(ContextPack::primary_references).unwrap_or_default();
        if inputs.is_empty()
            && let Some(memory) = self.compacted_canon_memory.as_ref()
        {
            inputs.extend(memory.artifact_refs.iter().take(2).cloned());
        }
        inputs
    }

    /// Returns operator-visible provenance lines for persisted planning context.
    pub fn context_provenance_lines(&self) -> Vec<String> {
        let mut lines =
            self.context_pack.as_ref().map(ContextPack::provenance_lines).unwrap_or_default();
        if let Some(memory) = self.compacted_canon_memory.as_ref() {
            lines.extend(memory.provenance_lines());
        }
        if let Some(selection) = self.expert_selection.as_ref() {
            lines.extend(selection.provenance_lines());
        }
        lines
    }

    pub fn expert_selection_summary(&self) -> Option<String> {
        self.expert_selection.as_ref().map(|selection| selection.summary.clone())
    }

    pub fn selected_expert_packs(&self) -> Vec<String> {
        self.expert_selection
            .as_ref()
            .map(|selection| selection.selected_expert_packs.clone())
            .unwrap_or_default()
    }

    pub fn suggested_runtime_roles(&self) -> Vec<String> {
        self.expert_selection
            .as_ref()
            .map(|selection| selection.suggested_runtime_roles.clone())
            .unwrap_or_default()
    }

    pub fn canon_memory_staleness_reason(&self) -> Option<String> {
        self.compacted_canon_memory.as_ref().and_then(|memory| {
            (memory.credibility != crate::domain::governance::MemoryCredibilityState::Credible)
                .then(|| memory.reason_code.clone().unwrap_or_else(|| memory.headline.clone()))
        })
    }
}

fn planning_analysis_validation_anchor_total(
    backlog_evidence: &PlanningAnalysisBacklogEvidence,
    backlog_document_refs: &[String],
) -> Option<usize> {
    if backlog_document_refs.is_empty() || backlog_evidence.selected_slice_id.is_none() {
        None
    } else {
        Some(1)
    }
}

fn planning_analysis_validation_anchor_covered(
    goal_plan: &GoalPlan,
    backlog_evidence: &PlanningAnalysisBacklogEvidence,
    validation_anchor_total: Option<usize>,
) -> Option<usize> {
    let verification_strategy_present = goal_plan
        .verification_strategy
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let selected_slice_is_anchored =
        backlog_evidence.selected_slice_id.as_ref().is_some_and(|selected_slice_id| {
            backlog_evidence
                .acceptance_anchor_slice_ids
                .iter()
                .any(|slice_id| slice_id == selected_slice_id)
        });
    match validation_anchor_total {
        Some(1) if verification_strategy_present && selected_slice_is_anchored => Some(1),
        Some(1) => Some(0),
        _ => None,
    }
}

fn planning_analysis_risk_covered(
    goal_plan: &GoalPlan,
    backlog_evidence: &PlanningAnalysisBacklogEvidence,
) -> Option<usize> {
    (backlog_evidence.planning_risk_count > 0).then(|| {
        if goal_plan
            .verification_strategy
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
        {
            backlog_evidence.planning_risk_count
        } else {
            0
        }
    })
}

fn planning_analysis_governed_evidence_ready(
    backlog_document_refs: &[String],
    backlog_evidence: &PlanningAnalysisBacklogEvidence,
) -> bool {
    if backlog_document_refs.is_empty() || backlog_evidence.closure_limited {
        return true;
    }
    backlog_evidence.selected_slice_id.is_some()
        && backlog_evidence.implementation_artifact_ref_count > 0
        && backlog_evidence.dependency_prerequisite_count > 0
        && backlog_evidence.independent_verification_anchor_count > 0
}

fn planning_analysis_backlog_slice_covered(
    backlog_evidence: &PlanningAnalysisBacklogEvidence,
) -> Option<usize> {
    (!backlog_evidence.slice_ids.is_empty()).then_some(usize::from(
        backlog_evidence.selected_slice_id.is_some()
            && backlog_evidence.implementation_artifact_ref_count > 0
            && backlog_evidence.dependency_prerequisite_count > 0
            && backlog_evidence.independent_verification_anchor_count > 0,
    ))
}

fn planning_analysis_state_for_findings(
    findings: &[PlanningAnalysisFinding],
) -> PlanningAnalysisState {
    if findings.iter().any(|finding| matches!(finding.severity, PlanningAnalysisSeverity::Critical))
    {
        PlanningAnalysisState::Blocked
    } else if findings.is_empty() {
        PlanningAnalysisState::Clean
    } else {
        PlanningAnalysisState::Findings
    }
}

fn deduplicated_items(items: &[String]) -> Vec<String> {
    let mut deduplicated = Vec::new();
    for item in items {
        let normalized = item.trim();
        if normalized.is_empty() || deduplicated.iter().any(|existing| existing == normalized) {
            continue;
        }
        deduplicated.push(normalized.to_string());
    }
    deduplicated
}

fn deduplicate_planning_analysis_findings(
    findings: Vec<PlanningAnalysisFinding>,
) -> Vec<PlanningAnalysisFinding> {
    let mut deduplicated = Vec::<PlanningAnalysisFinding>::new();

    for mut finding in findings {
        if let Some(existing) = deduplicated.iter_mut().find(|existing| {
            existing.severity == finding.severity
                && existing.source == finding.source
                && existing.code == finding.code
                && existing.message == finding.message
        }) {
            for source_ref in finding.source_refs.drain(..) {
                if !existing.source_refs.iter().any(|existing_ref| existing_ref == &source_ref) {
                    existing.source_refs.push(source_ref);
                }
            }
            continue;
        }
        deduplicated.push(finding);
    }

    deduplicated
}

/// Validation failures for persisted goal-plan state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GoalPlanError {
    #[error("goal text must not be empty")]
    MissingGoalText,
    #[error("goal plan must have at least one task")]
    NoTasks,
    #[error("task id must not be empty")]
    MissingTaskId,
    #[error("task `{task_id}` description must not be empty")]
    MissingTaskDescription { task_id: String },
    #[error("task `{task_id}` target must not be empty")]
    MissingTaskTarget { task_id: String },
    #[error("context pack id must not be empty")]
    MissingContextPackId,
    #[error("context pack summary must not be empty")]
    MissingContextPackSummary,
    #[error("credible context pack must have at least one primary input or selected target")]
    MissingCredibleContextPrimaryInput,
    #[error("stale context pack must explain why it is stale")]
    MissingContextStalenessReason,
    #[error("goal plan proposal revision must be at least 1")]
    MissingProposalRevision,
    #[error("context input reference must not be empty")]
    MissingContextInputReference,
    #[error("context input `{reference}` rationale must not be empty")]
    MissingContextInputRationale { reference: String },
    #[error("context input `{reference}` source must not be empty")]
    MissingContextInputSource { reference: String },
    #[error("goal plan workflow progress is invalid: {0}")]
    InvalidWorkflowProgress(String),
    #[error("invalid delegation packet: {0}")]
    InvalidDelegationPacket(String),
    #[error("invalid delegation continuity: {0}")]
    InvalidDelegationContinuity(String),
    #[error("invalid cluster projection: {0}")]
    InvalidClusterProjection(String),
    #[error("invalid cluster delivery story: {0}")]
    InvalidClusterDeliveryStory(String),
    #[error("invalid advanced context projection: {0}")]
    InvalidAdvancedContext(String),
    #[error("invalid goal plan status transition from {from:?} to {to:?}")]
    InvalidTransition { from: GoalPlanStatus, to: GoalPlanStatus },
}

#[cfg(test)]
mod tests {
    use super::{
        BACKLOG_DOCUMENT_ACCEPTANCE_ANCHORS, CanonExpertiseInputConsideration,
        CanonExpertiseInputDisposition, ContextInput, ContextInputKind, ContextPack,
        ContextPackCredibility, ExpertPackSelectionOutcome, ExpertPackSelectionState,
        ExpertPackSignal, ExpertPackSignalStatus, GoalPlan, GoalPlanError,
        PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT,
        PLANNING_ANALYSIS_ARTIFACT_KIND_GOAL_PLAN, PLANNING_ANALYSIS_ARTIFACT_REF_SUCCESS_CRITERIA,
        PLANNING_ANALYSIS_CODE_ARTIFACT_CONTRADICTION,
        PLANNING_ANALYSIS_CODE_COVERAGE_SIGNAL_PARTIAL,
        PLANNING_ANALYSIS_CODE_EXECUTION_INPUT_MISSING,
        PLANNING_ANALYSIS_CODE_EXPECTED_OUTCOME_MISSING,
        PLANNING_ANALYSIS_CODE_PRODUCER_CONTRACT_GAP,
        PLANNING_ANALYSIS_CODE_SUCCESS_CRITERION_UNCOVERED,
        PLANNING_ANALYSIS_CODE_VALIDATION_COVERAGE_MISSING,
        PLANNING_ANALYSIS_MESSAGE_VALIDATION_COVERAGE_MISSING, PlannedTask,
        PlanningAnalysisFinding, PlanningAnalysisSeverity, PlanningAnalysisSource,
        PlanningAnalysisSourceRef, PlanningAnalysisState, RejectedExpertCandidate,
        deduplicate_planning_analysis_findings, deduplicated_items,
        planning_analysis_state_for_findings,
    };
    use crate::domain::governance::{
        BacklogQualityAssessment, BacklogQualityState, CanonEvidenceInspectSummary,
        CanonModeSummary, CanonResultActionSummary, CompactedCanonMemory, MemoryCredibilityState,
    };
    use crate::domain::guidance::GuidanceGuardianProjection;
    use crate::domain::session::{
        ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState, DelegationPacket,
        DelegationPacketKind, DelegationPacketState, StuckEvidenceMarker, StuckRecoveryAction,
    };

    fn build_plan() -> GoalPlan {
        GoalPlan::new(
            "Fix delegated continuity",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Repair the blocked bounded flow".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("status explains the blocked continuation".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
    }

    fn build_packet(packet_id: &str, kind: DelegationPacketKind) -> DelegationPacket {
        DelegationPacket {
            packet_id: packet_id.to_string(),
            kind,
            state: DelegationPacketState::Active,
            created_at: 100,
            resolved_at: None,
            source_route_owner: "native".to_string(),
            target_owner: match kind {
                DelegationPacketKind::Handoff => "codex".to_string(),
                DelegationPacketKind::Escalation => "operator".to_string(),
            },
            continuity_reason: "declared runtime cannot continue the bounded step".to_string(),
            recommended_next_action: "boundline status".to_string(),
            evidence_refs: vec!["routing:implementation=claude/sonnet-4".to_string()],
            capability_summary: Some(
                "claude lacks continuation support for implementation".to_string(),
            ),
            stuck_marker: None,
            superseded_by_packet_id: None,
        }
    }

    fn backlog_document_ref(file_name: &str) -> String {
        format!(".canon/backlog/{file_name}")
    }

    fn backlog_document_refs(file_names: &[&str]) -> Vec<String> {
        file_names.iter().map(|file_name| backlog_document_ref(file_name)).collect()
    }

    fn ready_backlog_quality() -> BacklogQualityAssessment {
        BacklogQualityAssessment {
            state: BacklogQualityState::Ready,
            findings: Vec::new(),
            task_count: Some(1),
            mvp_scope: Some("SLICE-SESSION-001".to_string()),
            unmapped_items: Vec::new(),
        }
    }

    #[test]
    fn with_guidance_guardian_preserves_projection_fields() {
        let projection = GuidanceGuardianProjection {
            capability_resolution_summary: Some("resolved guidance for verification".to_string()),
            loaded_guidance_sources: vec![
                "assistant/packs/engineering-foundations.toml".to_string(),
            ],
            guardian_timeline: vec!["testing-evidence: completed".to_string()],
            ..GuidanceGuardianProjection::default()
        };

        let plan = build_plan().with_guidance_guardian(projection.clone());

        assert_eq!(plan.guidance_guardian, projection);
        assert!(!plan.guidance_guardian.is_empty());
    }

    #[test]
    fn recording_delegation_packet_supersedes_previous_active_packet() {
        let mut plan = build_plan();
        let first_packet = build_packet("packet-1", DelegationPacketKind::Handoff);
        let second_packet = build_packet("packet-2", DelegationPacketKind::Escalation);

        plan.record_delegation_packet(
            first_packet,
            DelegationContinuityState {
                active_packet_id: Some("packet-1".to_string()),
                mode: DelegationContinuityMode::HandoffRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline status".to_string(),
                headline: "handoff required: implementation route cannot continue".to_string(),
                evidence_summary: "routing policy requires a handoff".to_string(),
            },
        )
        .unwrap();

        plan.record_delegation_packet(
            second_packet,
            DelegationContinuityState {
                active_packet_id: Some("packet-2".to_string()),
                mode: DelegationContinuityMode::EscalationRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: "escalation required: no declared continuation path remains".to_string(),
                evidence_summary: "all declared routes are blocked by capability policy"
                    .to_string(),
            },
        )
        .unwrap();

        let first_packet = plan
            .delegation_packet_history()
            .iter()
            .find(|packet| packet.packet_id == "packet-1")
            .unwrap();
        assert_eq!(first_packet.state, DelegationPacketState::Superseded);
        assert_eq!(first_packet.superseded_by_packet_id.as_deref(), Some("packet-2"));
        assert_eq!(plan.active_delegation_packet().unwrap().packet_id, "packet-2");
        assert_eq!(
            plan.delegation_continuity().unwrap().mode,
            DelegationContinuityMode::EscalationRequired
        );
    }

    #[test]
    fn resolving_delegation_packet_preserves_history_and_clears_active_pointer() {
        let mut plan = build_plan();
        let mut packet = build_packet("packet-stuck", DelegationPacketKind::Handoff);
        packet.state = DelegationPacketState::Stuck;
        packet.stuck_marker = Some(StuckEvidenceMarker {
            repeated_attempts: 3,
            same_reason_count: 3,
            unchanged_workspace_signal: true,
            stale_route_policy: false,
            recommended_recovery: StuckRecoveryAction::Replan,
        });

        plan.record_delegation_packet(
            packet,
            DelegationContinuityState {
                active_packet_id: Some("packet-stuck".to_string()),
                mode: DelegationContinuityMode::Stuck,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: "stuck delegated continuity requires recovery".to_string(),
                evidence_summary: "the same blocked continuity reason repeated three times"
                    .to_string(),
            },
        )
        .unwrap();

        plan.resolve_active_delegation(
            "delegated continuity resolved after config update",
            "operator updated the declared runtime policy",
            "boundline run",
        )
        .unwrap();

        let resolved_packet = plan
            .delegation_packet_history()
            .iter()
            .find(|packet| packet.packet_id == "packet-stuck")
            .unwrap();
        assert_eq!(resolved_packet.state, DelegationPacketState::Resolved);
        assert!(resolved_packet.resolved_at.is_some());

        let continuity = plan.delegation_continuity().unwrap();
        assert_eq!(continuity.mode, DelegationContinuityMode::Resolved);
        assert!(continuity.active_packet_id.is_none());
        assert_eq!(continuity.next_command, "boundline run");
    }

    #[test]
    fn context_and_flow_helpers_surface_negotiation_and_canon_memory_details() {
        let mut plan = GoalPlan::new(
            "Tighten bounded context",
            vec![PlannedTask {
                task_id: "planned-task-context".to_string(),
                description: "Confirm the governed packet context".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("status reflects bounded context".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_context_pack(ContextPack {
            pack_id: "cp-context".to_string(),
            summary: "bounded context from src/lib.rs".to_string(),
            credibility: ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "primary workspace slice".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        })
        .with_negotiation_projection(
            "deliver the smallest safe fix",
            "confirmed",
            "tests stay green",
        )
        .with_compacted_canon_memory(CompactedCanonMemory {
            headline: "Governed packet needs refresh".to_string(),
            credibility: MemoryCredibilityState::Stale,
            stage_key: Some("bug-fix:verify".to_string()),
            run_ref: Some("canon-run-1".to_string()),
            packet_ref: Some(".canon/runs/canon-run-1".to_string()),
            reason_code: Some("refresh_required".to_string()),
            artifact_refs: vec![".canon/runs/canon-run-1/verification.md".to_string()],
            mode_summary: Some(CanonModeSummary {
                headline: "Discovery mode packet ready".to_string(),
                artifact_packet_summary: "packet can be resumed".to_string(),
                execution_posture: Some("awaiting operator review".to_string()),
                primary_artifact_title: "verification packet".to_string(),
                primary_artifact_path: ".canon/runs/canon-run-1/verification.md".to_string(),
                primary_artifact_action: CanonResultActionSummary {
                    label: "inspect".to_string(),
                    target: ".canon/runs/canon-run-1/verification.md".to_string(),
                },
                result_excerpt: "governed packet is reusable once refreshed".to_string(),
                action_chip_labels: vec!["inspect".to_string()],
            }),
            possible_actions: Vec::new(),
            recommended_next_action: None,
            evidence_summary: Some(CanonEvidenceInspectSummary {
                execution_posture: Some("paused".to_string()),
                carried_forward_items: Vec::new(),
                artifact_provenance_links: vec![
                    "canon:packet=.canon/runs/canon-run-1".to_string(),
                    "canon:artifact=.canon/runs/canon-run-1/verification.md".to_string(),
                ],
                closure_status: None,
                closure_findings: Vec::new(),
            }),
            authority_provenance_lines: Vec::new(),
            adaptive_provenance_lines: Vec::new(),
            semantic_provenance_lines: Vec::new(),
        });

        assert_eq!(
            plan.context_summary().as_deref(),
            Some(
                "bounded context from src/lib.rs; canon memory: Governed packet needs refresh [stale]"
            )
        );
        assert_eq!(plan.context_credibility().as_deref(), Some("credible"));
        assert_eq!(plan.context_primary_inputs(), vec!["src/lib.rs".to_string()]);
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("canon_memory_packet: .canon/runs/canon-run-1"))
        );
        assert!(plan
            .context_provenance_lines()
            .iter()
            .any(|line| line.contains("canon_memory_mode: Discovery mode packet ready; packet can be resumed; execution posture: awaiting operator review")));
        assert_eq!(plan.canon_memory_staleness_reason().as_deref(), Some("refresh_required"));
        assert_eq!(plan.negotiation_goal_summary.as_deref(), Some("deliver the smallest safe fix"));
        assert_eq!(plan.negotiation_resolution.as_deref(), Some("confirmed"));
        assert_eq!(plan.negotiation_acceptance_boundary.as_deref(), Some("tests stay green"));

        plan.mark_flow_skipped();
        let flow_state = plan.flow_state();
        assert_eq!(flow_state.mode, super::GoalPlanFlowMode::Skipped);
        assert!(flow_state.flow_name.is_none());
    }

    #[test]
    fn context_input_provenance_line_includes_source_label() {
        let input = ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "failing test target".to_string(),
            source: "workspace_signal, symbol_scan".to_string(),
            primary: true,
        };

        assert_eq!(
            input.provenance_line(),
            "workspace_file: src/lib.rs (failing test target) [source=workspace_signal, symbol_scan]"
        );
    }

    #[test]
    fn context_pack_validation_and_selection_strings_cover_remaining_branches() {
        assert_eq!(ContextInputKind::RecentTrace.as_str(), "recent_trace");
        assert_eq!(ContextInputKind::CanonCapability.as_str(), "canon_capability");
        assert_eq!(ContextInputKind::CanonMemory.as_str(), "canon_memory");
        assert_eq!(ExpertPackSelectionState::Selected.as_str(), "selected");
        assert_eq!(ExpertPackSelectionState::NoneSelected.as_str(), "none-selected");
        assert_eq!(ExpertPackSignalStatus::Ignored.as_str(), "ignored");
        assert_eq!(ExpertPackSignalStatus::Blocked.as_str(), "blocked");
        assert_eq!(CanonExpertiseInputDisposition::Used.as_str(), "used");
        assert_eq!(CanonExpertiseInputDisposition::Ignored.as_str(), "ignored");

        let missing_id = ContextPack {
            pack_id: " ".to_string(),
            summary: "bounded context".to_string(),
            credibility: ContextPackCredibility::Insufficient,
            inputs: Vec::new(),
            selected_targets: Vec::new(),
            advanced_context: None,
            staleness_reason: None,
        };
        assert_eq!(missing_id.validate().unwrap_err(), GoalPlanError::MissingContextPackId);

        let missing_summary = ContextPack {
            pack_id: "cp-1".to_string(),
            summary: " ".to_string(),
            credibility: ContextPackCredibility::Insufficient,
            inputs: Vec::new(),
            selected_targets: Vec::new(),
            advanced_context: None,
            staleness_reason: None,
        };
        assert_eq!(
            missing_summary.validate().unwrap_err(),
            GoalPlanError::MissingContextPackSummary
        );

        let missing_primary = ContextPack {
            pack_id: "cp-2".to_string(),
            summary: "bounded context".to_string(),
            credibility: ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "secondary evidence".to_string(),
                source: "workspace_scan".to_string(),
                primary: false,
            }],
            selected_targets: Vec::new(),
            advanced_context: None,
            staleness_reason: None,
        };
        assert_eq!(
            missing_primary.validate().unwrap_err(),
            GoalPlanError::MissingCredibleContextPrimaryInput
        );

        let stale_without_reason = ContextPack {
            pack_id: "cp-3".to_string(),
            summary: "bounded context".to_string(),
            credibility: ContextPackCredibility::Stale,
            inputs: Vec::new(),
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        };
        assert_eq!(
            stale_without_reason.validate().unwrap_err(),
            GoalPlanError::MissingContextStalenessReason
        );

        let fallback_targets = ContextPack {
            pack_id: "cp-4".to_string(),
            summary: "bounded context".to_string(),
            credibility: ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "secondary evidence".to_string(),
                source: "workspace_scan".to_string(),
                primary: false,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        };
        assert_eq!(fallback_targets.primary_references(), vec!["src/lib.rs".to_string()]);
    }

    #[test]
    fn goal_plan_context_helpers_surface_expert_selection_and_memory_fallbacks() {
        let expert_selection = ExpertPackSelectionOutcome {
            target_ref: "src/lib.rs".to_string(),
            selection_state: ExpertPackSelectionState::Selected,
            selected_expert_packs: vec!["domain-react-expert-pack".to_string()],
            suggested_runtime_roles: vec!["frontend".to_string()],
            supporting_signals: vec![ExpertPackSignal {
                kind: "reviewer_role".to_string(),
                reference: "frontend".to_string(),
                source: "routing_resolution".to_string(),
                status: ExpertPackSignalStatus::Blocked,
                rationale: "declared reviewer role requires explicit verification".to_string(),
            }],
            rejected_expert_candidates: vec![RejectedExpertCandidate {
                pack_id: "domain-rust-expert-pack".to_string(),
                reason: "bounded target resolved to react".to_string(),
                blocking_signals: Vec::new(),
            }],
            canon_inputs_considered: vec![CanonExpertiseInputConsideration {
                contract_version: "v1".to_string(),
                mode: "domain-language".to_string(),
                expertise_kind: "domain-language".to_string(),
                domain_families: Vec::new(),
                source_ref: "canon-run:domain-language".to_string(),
                promotion_state: "auto".to_string(),
                publication_target_class: "stable".to_string(),
                disposition: CanonExpertiseInputDisposition::Ignored,
                disposition_reason: "no selected domain families are available for Canon matching"
                    .to_string(),
            }],
            summary: "selected domain-react-expert-pack with frontend verification".to_string(),
        };

        let selection_only_plan = build_plan().with_expert_selection(expert_selection.clone());
        assert_eq!(
            selection_only_plan.context_summary().as_deref(),
            Some("expert selection: selected domain-react-expert-pack with frontend verification")
        );

        let plan = build_plan()
            .with_expert_selection(expert_selection)
            .with_routing_policy_summary("planning route=copilot/gpt-4.1 [workspace]")
            .with_compacted_canon_memory(CompactedCanonMemory {
                headline: "Canon packet remains usable".to_string(),
                credibility: MemoryCredibilityState::Credible,
                stage_key: Some("change:verify".to_string()),
                run_ref: Some("canon-run-2".to_string()),
                packet_ref: Some(".canon/runs/canon-run-2".to_string()),
                reason_code: None,
                artifact_refs: vec![
                    ".canon/runs/canon-run-2/verification.md".to_string(),
                    ".canon/runs/canon-run-2/evidence.md".to_string(),
                    ".canon/runs/canon-run-2/extra.md".to_string(),
                ],
                mode_summary: None,
                possible_actions: Vec::new(),
                recommended_next_action: None,
                evidence_summary: None,
                authority_provenance_lines: Vec::new(),
                adaptive_provenance_lines: Vec::new(),
                semantic_provenance_lines: Vec::new(),
            });

        assert_eq!(
            plan.routing_policy_summary.as_deref(),
            Some("planning route=copilot/gpt-4.1 [workspace]")
        );
        assert_eq!(
            plan.context_primary_inputs(),
            vec![
                ".canon/runs/canon-run-2/verification.md".to_string(),
                ".canon/runs/canon-run-2/evidence.md".to_string(),
            ]
        );
        assert_eq!(
            plan.expert_selection_summary().as_deref(),
            Some("selected domain-react-expert-pack with frontend verification")
        );
        assert_eq!(plan.selected_expert_packs(), vec!["domain-react-expert-pack".to_string()]);
        assert_eq!(plan.suggested_runtime_roles(), vec!["frontend".to_string()]);
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("expert_pack_signal: reviewer_role=frontend [blocked]"))
        );
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("expert_pack_rejected: domain-rust-expert-pack"))
        );
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("families=none") && line.contains("disposition=ignored"))
        );
    }

    #[test]
    fn recording_existing_delegation_packet_replaces_it_in_place() {
        let mut plan = build_plan();
        let packet = build_packet("packet-1", DelegationPacketKind::Handoff);
        let continuity = DelegationContinuityState {
            active_packet_id: Some("packet-1".to_string()),
            mode: DelegationContinuityMode::HandoffRequired,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: "boundline status".to_string(),
            headline: "handoff required".to_string(),
            evidence_summary: "initial bounded packet".to_string(),
        };

        plan.record_delegation_packet(packet, continuity.clone()).unwrap();

        let mut replacement = build_packet("packet-1", DelegationPacketKind::Handoff);
        replacement.capability_summary = Some("replacement packet state".to_string());

        plan.record_delegation_packet(replacement, continuity).unwrap();

        assert_eq!(plan.delegation_packet_history().len(), 1);
        assert_eq!(plan.delegation_packet_history()[0].state, DelegationPacketState::Active);
        assert_eq!(
            plan.delegation_packet_history()[0].capability_summary.as_deref(),
            Some("replacement packet state")
        );
    }

    #[test]
    fn planning_analysis_projection_reports_blocking_findings_and_coverage()
    -> Result<(), GoalPlanError> {
        let plan = GoalPlan::new(
            "Goal",
            vec![
                PlannedTask {
                    task_id: "T001".to_string(),
                    description: "Implement T001".to_string(),
                    target: "src/T001.rs".to_string(),
                    expected_outcome: Some("first slice verified".to_string()),
                    decision_type_hint: None,
                },
                PlannedTask {
                    task_id: "T002".to_string(),
                    description: "Implement T002".to_string(),
                    target: "src/T002.rs".to_string(),
                    expected_outcome: None,
                    decision_type_hint: None,
                },
            ],
        )?
        .with_planning_rationale("execution must honor the selected delivery slice order")
        .with_verification_strategy(
            "run acceptance and sequencing verification after implementation",
        );
        let document_refs = backlog_document_refs(&[
            "delivery-slices.md",
            "sequencing-plan.md",
            "acceptance-anchors.md",
            "planning-risks.md",
            "execution-handoff.md",
        ]);
        let projection = plan.planning_analysis_projection(
            &BacklogQualityAssessment {
                unmapped_items: vec![
                    "acceptance target".to_string(),
                    "acceptance target".to_string(),
                ],
                ..ready_backlog_quality()
            },
            &document_refs,
            &[
                "- [SLICE-SESSION-001] First bounded execution slice.\n- [SLICE-SESSION-002] Follow-up slice.\n".to_string(),
                "1. [SLICE-SESSION-001] first\n2. [SLICE-SESSION-002] second\n".to_string(),
                "- [SLICE-SESSION-001] Different slice owns the acceptance proof.\n".to_string(),
                "- mitigate flaky dependency\n- confirm migration constraint\n".to_string(),
                concat!(
                    "## Selected Slice\n\nSLICE-SESSION-002\n\n",
                    "## Implementation Artifact References\n\n\n",
                    "## Independent Verification Anchors\n\n",
                    "- integration coverage exists\n"
                )
                .to_string(),
            ],
        );

        assert_eq!(projection.state, PlanningAnalysisState::Blocked);
        assert!(projection.findings.len() >= 4);
        let coverage = projection.coverage.ok_or(GoalPlanError::MissingGoalText)?;
        assert_eq!(coverage.success_criteria_total, 2);
        assert_eq!(coverage.success_criteria_covered, 1);
        assert_eq!(coverage.backlog_slice_total, Some(2));
        assert_eq!(coverage.backlog_slice_covered, Some(0));
        assert_eq!(coverage.validation_anchor_total, Some(1));
        assert_eq!(coverage.validation_anchor_covered, Some(0));
        assert_eq!(coverage.risk_total, Some(2));
        assert_eq!(coverage.risk_covered, Some(2));
        assert_eq!(coverage.constraint_total, None);
        assert_eq!(coverage.constraint_covered, None);
        assert!(!coverage.governed_evidence_ready);
        assert!(projection.findings.iter().any(|finding| {
            finding.severity == PlanningAnalysisSeverity::Critical
                && finding.source == PlanningAnalysisSource::Goal
                && finding.code == PLANNING_ANALYSIS_CODE_SUCCESS_CRITERION_UNCOVERED
        }));
        assert!(projection.findings.iter().any(|finding| {
            finding.severity == PlanningAnalysisSeverity::Critical
                && finding.source == PlanningAnalysisSource::Validation
                && finding.code == PLANNING_ANALYSIS_CODE_VALIDATION_COVERAGE_MISSING
        }));
        assert!(projection.findings.iter().any(|finding| {
            finding.severity == PlanningAnalysisSeverity::Critical
                && finding.source == PlanningAnalysisSource::Backlog
                && finding.code == PLANNING_ANALYSIS_CODE_ARTIFACT_CONTRADICTION
        }));
        assert!(projection.findings.iter().any(|finding| {
            finding.severity == PlanningAnalysisSeverity::Critical
                && finding.source == PlanningAnalysisSource::GovernedEvidence
                && finding.code == PLANNING_ANALYSIS_CODE_PRODUCER_CONTRACT_GAP
        }));
        assert!(projection.findings.iter().any(|finding| {
            finding.severity == PlanningAnalysisSeverity::Critical
                && finding.source == PlanningAnalysisSource::ExecutionReadiness
                && finding.code == PLANNING_ANALYSIS_CODE_EXECUTION_INPUT_MISSING
        }));
        assert!(projection.findings.iter().any(|finding| {
            finding.severity == PlanningAnalysisSeverity::Medium
                && finding.source == PlanningAnalysisSource::Plan
                && finding.code == PLANNING_ANALYSIS_CODE_EXPECTED_OUTCOME_MISSING
        }));

        Ok(())
    }

    #[test]
    fn planning_analysis_projection_reports_clean_governed_evidence_when_inputs_align()
    -> Result<(), GoalPlanError> {
        let plan = GoalPlan::new(
            "Goal",
            vec![PlannedTask {
                task_id: "T001".to_string(),
                description: "Implement the first slice".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("first slice shipped".to_string()),
                decision_type_hint: None,
            }],
        )?
        .with_planning_rationale("execute the first slice with governed evidence")
        .with_verification_strategy("run the independent verification anchors")
        .with_negotiation_projection(
            "ship the selected slice",
            "confirmed",
            "must preserve the bounded acceptance scope",
        )
        .with_routing_policy_summary("runtime=canon/backlog");
        let document_refs = backlog_document_refs(&[
            "delivery-slices.md",
            "sequencing-plan.md",
            "acceptance-anchors.md",
            "planning-risks.md",
            "execution-handoff.md",
        ]);
        let projection = plan.planning_analysis_projection(
            &ready_backlog_quality(),
            &document_refs,
            &[
                "- [SLICE-SESSION-001] First bounded execution slice.\n".to_string(),
                "1. [SLICE-SESSION-001] first\n".to_string(),
                "- [SLICE-SESSION-001] Acceptance proof captured.\n".to_string(),
                "- mitigate flaky dependency\n".to_string(),
                concat!(
                    "## Selected Slice\n\nSLICE-SESSION-001\n\n",
                    "## Implementation Artifact References\n\n",
                    "- src/lib.rs\n\n",
                    "## Dependency Prerequisites\n\n",
                    "- upstream review completed\n\n",
                    "## Independent Verification Anchors\n\n",
                    "- cargo test --lib\n"
                )
                .to_string(),
            ],
        );

        assert_eq!(projection.state, PlanningAnalysisState::Clean);
        assert!(projection.findings.is_empty());
        let coverage = projection.coverage.ok_or(GoalPlanError::MissingGoalText)?;
        assert_eq!(coverage.success_criteria_total, 1);
        assert_eq!(coverage.success_criteria_covered, 1);
        assert_eq!(coverage.backlog_slice_total, Some(1));
        assert_eq!(coverage.backlog_slice_covered, Some(1));
        assert_eq!(coverage.validation_anchor_total, Some(1));
        assert_eq!(coverage.validation_anchor_covered, Some(1));
        assert_eq!(coverage.risk_total, Some(1));
        assert_eq!(coverage.risk_covered, Some(1));
        assert_eq!(coverage.constraint_total, Some(2));
        assert_eq!(coverage.constraint_covered, Some(2));
        assert!(coverage.governed_evidence_ready);

        Ok(())
    }

    #[test]
    fn planning_analysis_helpers_cover_new_sources_and_deduplication_paths() {
        let finding = PlanningAnalysisFinding {
            severity: PlanningAnalysisSeverity::Critical,
            source: PlanningAnalysisSource::Validation,
            code: PLANNING_ANALYSIS_CODE_VALIDATION_COVERAGE_MISSING.to_string(),
            message: PLANNING_ANALYSIS_MESSAGE_VALIDATION_COVERAGE_MISSING.to_string(),
            source_refs: vec![PlanningAnalysisSourceRef {
                artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_BACKLOG_DOCUMENT.to_string(),
                artifact_ref: BACKLOG_DOCUMENT_ACCEPTANCE_ANCHORS.to_string(),
                anchor: Some("slice_id=SLICE-SESSION-001".to_string()),
            }],
        };
        let deduplicated = deduplicate_planning_analysis_findings(vec![
            finding.clone(),
            finding.clone(),
            PlanningAnalysisFinding {
                source_refs: vec![PlanningAnalysisSourceRef {
                    artifact_kind: PLANNING_ANALYSIS_ARTIFACT_KIND_GOAL_PLAN.to_string(),
                    artifact_ref: PLANNING_ANALYSIS_ARTIFACT_REF_SUCCESS_CRITERIA.to_string(),
                    anchor: Some("criterion-1".to_string()),
                }],
                ..finding.clone()
            },
        ]);

        assert_eq!(PlanningAnalysisSource::Goal.as_str(), "goal");
        assert_eq!(PlanningAnalysisSource::Validation.as_str(), "validation");
        assert_eq!(PlanningAnalysisSource::Risk.as_str(), "risk");
        assert_eq!(PlanningAnalysisSource::Constraint.as_str(), "constraint");
        assert_eq!(PlanningAnalysisSource::ExecutionReadiness.as_str(), "execution_readiness");
        assert_eq!(PlanningAnalysisSource::GovernedEvidence.as_str(), "governed_evidence");
        assert_eq!(planning_analysis_state_for_findings(&[]), PlanningAnalysisState::Clean);
        assert_eq!(
            planning_analysis_state_for_findings(&[PlanningAnalysisFinding {
                severity: PlanningAnalysisSeverity::Medium,
                source: PlanningAnalysisSource::Risk,
                code: PLANNING_ANALYSIS_CODE_COVERAGE_SIGNAL_PARTIAL.to_string(),
                message: "warning".to_string(),
                source_refs: Vec::new(),
            }]),
            PlanningAnalysisState::Findings
        );
        assert_eq!(
            planning_analysis_state_for_findings(std::slice::from_ref(&finding)),
            PlanningAnalysisState::Blocked
        );
        assert_eq!(
            deduplicated_items(&[
                "criterion-a".to_string(),
                "criterion-a".to_string(),
                " ".to_string(),
                "criterion-b".to_string(),
            ]),
            vec!["criterion-a".to_string(), "criterion-b".to_string()]
        );
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(deduplicated[0].source_refs.len(), 2);
    }
}
