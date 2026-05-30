//! Persisted execution traces and flattened trace-summary projections.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::audit::SessionAuditProjection;
use crate::domain::cluster::ClusterDeliveryStory;
use crate::domain::context_intelligence::AdvancedContextProjection;
use crate::domain::guidance::GuidanceGuardianProjection;
use crate::domain::limits::TerminalCondition;
use crate::domain::reasoning::ProfileActivationRecord;
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::{DelegationStatusView, DelightFeedbackSignal};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskStatus, TerminalReason};

/// Returns the current UNIX timestamp in milliseconds.
pub fn current_timestamp_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

/// Event kinds recorded in persisted execution traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventType {
    TaskStarted,
    FlowSelected,
    CheckpointCreated,
    StageTransitioned,
    StepStarted,
    StepCompleted,
    GovernanceSelected,
    GovernanceStarted,
    GovernanceDecisionRecorded,
    GovernanceAwaitingApproval,
    GovernanceCompleted,
    GovernanceBlocked,
    GovernancePacketRejected,
    ReasoningProfileActivated,
    ReasoningParticipantStarted,
    ReasoningParticipantCompleted,
    ReasoningDisagreementRecorded,
    ReasoningDebateRoundCompleted,
    ReasoningReflexionRevisionCompleted,
    ReasoningAdjudicationRecorded,
    ReasoningConfidenceRecorded,
    ReasoningProfileBlocked,
    ReasoningProfileInterrupted,
    ReasoningProfileEscalated,
    ProjectScalePathProposed,
    ProjectScaleStageTransitioned,
    ReviewStarted,
    ReviewTriggerIgnored,
    ReviewerStarted,
    ReviewerCompleted,
    ReviewCouncilAssembled,
    ReviewStopSemanticsRecorded,
    ReviewVoteResolved,
    ReviewAdjudicated,
    ReviewTerminalRecorded,
    VotingDecisionRecorded,
    RetryScheduled,
    StageRetryScheduled,
    Replanned,
    StageReplanned,
    StageFailed,
    TerminalRecorded,
    DecisionCreated,
    DecisionDispatched,
    DecisionVerified,
    DecisionFailed,
    DecisionRecovered,
    GoalPlanCreated,
    FlowInferred,
}

impl TraceEventType {
    /// Returns true when the event belongs to the additive reasoning-profile family.
    pub const fn is_reasoning_event(self) -> bool {
        matches!(
            self,
            Self::ReasoningProfileActivated
                | Self::ReasoningParticipantStarted
                | Self::ReasoningParticipantCompleted
                | Self::ReasoningDisagreementRecorded
                | Self::ReasoningDebateRoundCompleted
                | Self::ReasoningReflexionRevisionCompleted
                | Self::ReasoningAdjudicationRecorded
                | Self::ReasoningConfidenceRecorded
                | Self::ReasoningProfileBlocked
                | Self::ReasoningProfileInterrupted
                | Self::ReasoningProfileEscalated
        )
    }

    /// Returns true when the event belongs to the native decision-loop family.
    pub const fn is_decision_loop_event(self) -> bool {
        matches!(
            self,
            Self::DecisionCreated
                | Self::DecisionDispatched
                | Self::DecisionVerified
                | Self::DecisionFailed
                | Self::DecisionRecovered
                | Self::GoalPlanCreated
                | Self::FlowInferred
        )
    }

    /// Returns the routing-projection slot associated with this event type, when any.
    pub const fn routing_projection_key(self) -> Option<&'static str> {
        match self {
            Self::GoalPlanCreated => Some("goal_plan_created"),
            Self::FlowInferred => Some("flow_inferred"),
            Self::DecisionCreated => Some("decision_created"),
            Self::DecisionDispatched => Some("decision_dispatched"),
            Self::DecisionVerified => Some("decision_verified"),
            Self::DecisionFailed => Some("decision_failed"),
            Self::DecisionRecovered => Some("decision_recovered"),
            _ => None,
        }
    }
}

/// One event recorded inside a persisted execution trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub event_id: String,
    pub event_type: TraceEventType,
    pub step_id: Option<String>,
    pub plan_revision: usize,
    pub payload: Value,
    pub recorded_at: u64,
}

/// Persisted execution trace for one task run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub task_id: String,
    pub session_id: String,
    pub goal: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub terminal_status: Option<TaskStatus>,
    pub terminal_reason: Option<TerminalReason>,
    pub events: Vec<TraceEvent>,
    pub trace_location: Option<String>,
}

/// Inspect closure kinds synthesized from the flattened trace summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectClosureKind {
    Context,
    Council,
    Timeline,
}

impl InspectClosureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Council => "council",
            Self::Timeline => "timeline",
        }
    }
}

/// Human-facing inspect closure synthesized from the flattened trace summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectClosureView {
    pub view_kind: InspectClosureKind,
    pub headline: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub narrative_lines: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_attribution: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_inputs: Vec<String>,
    pub terminal_status: TaskStatus,
    pub terminal_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
}

impl InspectClosureView {
    pub fn validate(&self) -> Result<(), String> {
        if self.headline.trim().is_empty() {
            return Err(format!("inspect {} headline must not be empty", self.view_kind.as_str()));
        }

        if self.narrative_lines.is_empty() && self.missing_inputs.is_empty() {
            return Err(format!(
                "inspect {} view requires narrative_lines or missing_inputs",
                self.view_kind.as_str()
            ));
        }

        if self.terminal_reason.trim().is_empty() {
            return Err(format!(
                "inspect {} terminal_reason must not be empty",
                self.view_kind.as_str()
            ));
        }

        if self.source_attribution.iter().any(|line| line.trim().is_empty()) {
            return Err(format!(
                "inspect {} source_attribution entries must not be empty",
                self.view_kind.as_str()
            ));
        }

        if self.missing_inputs.iter().any(|line| line.trim().is_empty()) {
            return Err(format!(
                "inspect {} missing_inputs entries must not be empty",
                self.view_kind.as_str()
            ));
        }

        Ok(())
    }
}

/// Flattened read-side trace summary reused by inspect and other CLI surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceSummaryView {
    pub trace_ref: String,
    pub goal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_started_at: Option<u64>,
    /// Optional advanced-context retrieval projection surfaced by `inspect`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advanced_context: Option<AdvancedContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_goal_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_resolution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_acceptance_boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_delivery_story: Option<ClusterDeliveryStory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_summary: Option<String>,
    #[serde(default, skip_serializing_if = "RoutingDecisionProjection::is_empty")]
    pub routing_projection: RoutingDecisionProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_plan_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authored_input_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authored_input_deduplicated_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_brief_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_plan_brief_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_brief_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_credibility: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_primary_inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_provenance: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_staleness_reason: Option<String>,
    #[serde(flatten)]
    pub guidance_guardian: GuidanceGuardianProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_headline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarification_missing_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_timeline: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adaptive_evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_checkpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_checkpoint_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_checkpoint_restore_command: Option<String>,
    pub executed_steps: Vec<TraceStepSummary>,
    pub recovery_events: Vec<TraceRecoveryEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub governance_timeline: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_runtime_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_rollout_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_approval_provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_profile: Option<ProfileActivationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation: Option<DelegationStatusView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspect_context: Option<InspectClosureView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspect_council: Option<InspectClosureView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspect_timeline: Option<InspectClosureView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub review_timeline: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_audit: Option<SessionAuditProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delight_feedback: Option<DelightFeedbackSignal>,
    pub terminal_status: TaskStatus,
    pub terminal_reason: TerminalReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
}

impl Default for TraceSummaryView {
    fn default() -> Self {
        Self {
            trace_ref: String::new(),
            goal: String::new(),
            trace_started_at: None,
            advanced_context: None,
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            routing_summary: None,
            routing_projection: RoutingDecisionProjection::default(),
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
            goal_brief_ref: None,
            session_plan_brief_ref: None,
            run_brief_ref: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: Vec::new(),
            context_provenance: Vec::new(),
            context_staleness_reason: None,
            guidance_guardian: GuidanceGuardianProjection::default(),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: Vec::new(),
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            decision_timeline: Vec::new(),
            failure_evidence: Vec::new(),
            adaptive_evidence: Vec::new(),
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            executed_steps: Vec::new(),
            recovery_events: Vec::new(),
            governance_timeline: Vec::new(),
            governance_runtime_state: None,
            governance_rollout_profile: None,
            governance_reason: None,
            governance_approval_provenance: None,
            governance_next_action: None,
            reasoning_profile: None,
            delegation: None,
            inspect_context: None,
            inspect_council: None,
            inspect_timeline: None,
            review_timeline: Vec::new(),
            session_audit: None,
            delight_feedback: None,
            terminal_status: TaskStatus::Planned,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "", None),
            duration: None,
        }
    }
}

/// Summary of one executed step inside the flattened trace view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStepSummary {
    pub step_id: String,
    pub step_kind: StepKind,
    pub attempts: usize,
    pub final_status: StepStatus,
    pub headline: String,
}

/// Summary of one recovery event inside the flattened trace view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceRecoveryEvent {
    pub event_type: TraceEventType,
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_step_id: Option<String>,
}

impl ExecutionTrace {
    /// Creates a new persisted execution trace for the given task, session, and goal.
    pub fn new(
        task_id: impl Into<String>,
        session_id: impl Into<String>,
        goal: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            session_id: session_id.into(),
            goal: goal.into(),
            started_at: current_timestamp_millis(),
            ended_at: None,
            terminal_status: None,
            terminal_reason: None,
            events: Vec::new(),
            trace_location: None,
        }
    }

    /// Records one trace event at the current timestamp.
    pub fn record_event(
        &mut self,
        event_type: TraceEventType,
        step_id: Option<String>,
        plan_revision: usize,
        payload: Value,
    ) {
        self.events.push(TraceEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            step_id,
            plan_revision,
            payload,
            recorded_at: current_timestamp_millis(),
        });
    }

    /// Finalizes the trace with terminal state and timestamp.
    pub fn finalize(&mut self, terminal_status: TaskStatus, terminal_reason: TerminalReason) {
        self.ended_at = Some(current_timestamp_millis());
        self.terminal_status = Some(terminal_status);
        self.terminal_reason = Some(terminal_reason);
    }

    /// Stores the persisted trace location after the trace has been written.
    pub fn set_trace_location(&mut self, trace_location: impl Into<String>) {
        self.trace_location = Some(trace_location.into());
    }

    /// Returns the trace duration in milliseconds, if the trace is finalized.
    pub fn duration_millis(&self) -> Option<u64> {
        self.ended_at.map(|ended_at| ended_at.saturating_sub(self.started_at))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionTrace, InspectClosureKind, InspectClosureView, TraceEventType, TraceSummaryView,
    };
    use crate::domain::context_intelligence::{
        AdvancedContextProjection, HybridOutcome, RemoteTransmissionPolicyState,
        RetrievalIndexState, RetrievalMode, RetrievalState, SemanticCapabilityState,
        SemanticPolicyState, SemanticTraceEventKind, SemanticTraceRecord,
    };
    use crate::domain::limits::TerminalCondition;
    use crate::domain::task::{TaskStatus, TerminalReason};

    #[test]
    fn trace_event_type_helpers_cover_reasoning_family() {
        let reasoning_events = [
            TraceEventType::ReasoningProfileActivated,
            TraceEventType::ReasoningParticipantStarted,
            TraceEventType::ReasoningParticipantCompleted,
            TraceEventType::ReasoningDisagreementRecorded,
            TraceEventType::ReasoningDebateRoundCompleted,
            TraceEventType::ReasoningReflexionRevisionCompleted,
            TraceEventType::ReasoningAdjudicationRecorded,
            TraceEventType::ReasoningConfidenceRecorded,
            TraceEventType::ReasoningProfileBlocked,
            TraceEventType::ReasoningProfileInterrupted,
            TraceEventType::ReasoningProfileEscalated,
        ];

        for event_type in reasoning_events {
            assert!(event_type.is_reasoning_event());
            assert_eq!(event_type.routing_projection_key(), None);
        }

        assert!(!TraceEventType::TaskStarted.is_reasoning_event());
        assert!(!TraceEventType::GovernanceCompleted.is_reasoning_event());
    }

    #[test]
    fn trace_event_type_helpers_cover_decision_loop_and_routing_keys() {
        let routed_events = [
            (TraceEventType::GoalPlanCreated, "goal_plan_created"),
            (TraceEventType::FlowInferred, "flow_inferred"),
            (TraceEventType::DecisionCreated, "decision_created"),
            (TraceEventType::DecisionDispatched, "decision_dispatched"),
            (TraceEventType::DecisionVerified, "decision_verified"),
            (TraceEventType::DecisionFailed, "decision_failed"),
            (TraceEventType::DecisionRecovered, "decision_recovered"),
        ];

        for (event_type, key) in routed_events {
            assert!(event_type.is_decision_loop_event());
            assert_eq!(event_type.routing_projection_key(), Some(key));
        }

        assert!(!TraceEventType::TaskStarted.is_decision_loop_event());
        assert_eq!(TraceEventType::TaskStarted.routing_projection_key(), None);
        assert!(!TraceEventType::GovernanceBlocked.is_decision_loop_event());
        assert_eq!(TraceEventType::GovernanceBlocked.routing_projection_key(), None);
    }

    #[test]
    fn trace_summary_view_default_uses_goal_satisfied_terminal_reason() {
        let summary = TraceSummaryView::default();

        assert_eq!(summary.terminal_status, TaskStatus::Planned);
        assert_eq!(summary.terminal_reason.condition, TerminalCondition::GoalSatisfied);
        assert!(summary.terminal_reason.message.is_empty());
        assert!(summary.context_primary_inputs.is_empty());
        assert!(summary.context_provenance.is_empty());
    }

    #[test]
    fn trace_summary_view_serializes_semantic_trace_lifecycle_records() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/boundline-semantic-trace.json".to_string(),
            goal: "inspect semantic trace lifecycle".to_string(),
            advanced_context: Some(AdvancedContextProjection {
                query_id: "trace-summary-semantic".to_string(),
                retrieval_mode: RetrievalMode::Local,
                retrieval_state: RetrievalState::Selected,
                retrieval_index_state: RetrievalIndexState::Ready,
                semantic_policy_state: SemanticPolicyState::Local,
                semantic_capability_state: SemanticCapabilityState::Ready,
                hybrid_outcome: HybridOutcome::Expanded,
                budgets: Default::default(),
                remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
                used_remote: false,
                terminal_reason: Some("semantic retrieval succeeded".to_string()),
                selected_evidence: Vec::new(),
                rejected_candidates: Vec::new(),
                semantic_trace_records: vec![
                    SemanticTraceRecord {
                        record_id: "trace-extension-load".to_string(),
                        event_kind: SemanticTraceEventKind::ExtensionLoadAttempted,
                        candidate_ref: None,
                        match_origin: None,
                        compatibility_state: None,
                        semantic_score: None,
                        canon_artifact_class: None,
                        canon_semantic_contract_line: None,
                        canon_semantic_provenance_boundary: None,
                        canon_semantic_provenance_ref: None,
                        reason:
                            "trusted sqlite-vec extension load attempted: capability=ready retrieval_index_state=ready"
                                .to_string(),
                    },
                    SemanticTraceRecord {
                        record_id: "trace-vector-query".to_string(),
                        event_kind: SemanticTraceEventKind::VectorQueryExecuted,
                        candidate_ref: None,
                        match_origin: None,
                        compatibility_state: None,
                        semantic_score: None,
                        canon_artifact_class: None,
                        canon_semantic_contract_line: None,
                        canon_semantic_provenance_boundary: None,
                        canon_semantic_provenance_ref: None,
                        reason:
                            "vector query executed through semantic engine: engine=sqlite_vec"
                                .to_string(),
                    },
                ],
                relationships: Vec::new(),
                impact_findings: Vec::new(),
            }),
            ..TraceSummaryView::default()
        };

        let serialized = serde_json::to_value(&summary).expect("trace summary serializes");
        let trace_records = serialized["advanced_context"]["semantic_trace_records"]
            .as_array()
            .expect("semantic trace records array");

        assert_eq!(trace_records.len(), 2);
        assert_eq!(trace_records[0]["event_kind"].as_str(), Some("extension_load_attempted"));
        assert_eq!(trace_records[1]["event_kind"].as_str(), Some("vector_query_executed"));
    }

    #[test]
    fn inspect_closure_kind_as_str_covers_all_variants() {
        assert_eq!(InspectClosureKind::Context.as_str(), "context");
        assert_eq!(InspectClosureKind::Council.as_str(), "council");
        assert_eq!(InspectClosureKind::Timeline.as_str(), "timeline");
    }

    #[test]
    fn inspect_closure_view_validate_covers_all_error_branches() {
        let valid = InspectClosureView {
            view_kind: InspectClosureKind::Context,
            headline: "context looks good".to_string(),
            narrative_lines: vec!["workspace is consistent".to_string()],
            source_attribution: vec!["src/lib.rs".to_string()],
            missing_inputs: Vec::new(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: "goal satisfied".to_string(),
            next_action: None,
        };
        assert!(valid.validate().is_ok());

        // empty headline
        let mut bad = valid.clone();
        bad.headline = "  ".to_string();
        assert!(bad.validate().unwrap_err().contains("headline must not be empty"));

        // both narrative_lines and missing_inputs empty
        let mut bad = valid.clone();
        bad.narrative_lines = Vec::new();
        assert!(bad.validate().unwrap_err().contains("requires narrative_lines or missing_inputs"));

        // missing_inputs alone satisfies the check
        let mut ok = valid.clone();
        ok.narrative_lines = Vec::new();
        ok.missing_inputs = vec!["src/missing.rs".to_string()];
        assert!(ok.validate().is_ok());

        // empty terminal_reason
        let mut bad = valid.clone();
        bad.terminal_reason = String::new();
        assert!(bad.validate().unwrap_err().contains("terminal_reason must not be empty"));

        // blank source_attribution entry
        let mut bad = valid.clone();
        bad.source_attribution = vec!["  ".to_string()];
        assert!(
            bad.validate().unwrap_err().contains("source_attribution entries must not be empty")
        );

        // blank missing_inputs entry
        let mut bad = valid.clone();
        bad.missing_inputs = vec!["".to_string()];
        assert!(bad.validate().unwrap_err().contains("missing_inputs entries must not be empty"));
    }

    #[test]
    fn execution_trace_methods_cover_record_finalize_location_and_duration() {
        use serde_json::json;

        let mut trace = ExecutionTrace::new("task-1", "session-1", "Build the CLI");
        assert!(trace.events.is_empty());
        assert!(trace.terminal_status.is_none());
        assert!(trace.ended_at.is_none());
        assert!(trace.trace_location.is_none());
        assert_eq!(trace.duration_millis(), None);

        trace.record_event(TraceEventType::TaskStarted, None, 0, json!({"goal": "Build the CLI"}));
        assert_eq!(trace.events.len(), 1);
        assert_eq!(trace.events[0].event_type, TraceEventType::TaskStarted);
        assert!(!trace.events[0].event_id.is_empty());

        trace.record_event(
            TraceEventType::StepCompleted,
            Some("step-1".to_string()),
            1,
            json!({"outcome": "success"}),
        );
        assert_eq!(trace.events.len(), 2);
        assert_eq!(trace.events[1].step_id.as_deref(), Some("step-1"));
        assert_eq!(trace.events[1].plan_revision, 1);

        trace.set_trace_location("/tmp/traces/task-1.json");
        assert_eq!(trace.trace_location.as_deref(), Some("/tmp/traces/task-1.json"));

        let reason = TerminalReason::new(TerminalCondition::GoalSatisfied, "all done", None);
        trace.finalize(TaskStatus::Succeeded, reason);
        assert_eq!(trace.terminal_status, Some(TaskStatus::Succeeded));
        assert!(trace.terminal_reason.is_some());
        assert!(trace.ended_at.is_some());
        assert!(trace.duration_millis().is_some());
    }
}
