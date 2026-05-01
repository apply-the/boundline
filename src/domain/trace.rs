use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::cluster::ClusterDeliveryStory;
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskStatus, TerminalReason};

pub fn current_timestamp_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventType {
    TaskStarted,
    FlowSelected,
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
    ReviewStarted,
    ReviewTriggerIgnored,
    ReviewerStarted,
    ReviewerCompleted,
    ReviewVoteResolved,
    ReviewAdjudicated,
    ReviewTerminalRecorded,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub event_id: String,
    pub event_type: TraceEventType,
    pub step_id: Option<String>,
    pub plan_revision: usize,
    pub payload: Value,
    pub recorded_at: u64,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceSummaryView {
    pub trace_ref: String,
    pub goal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_delivery_story: Option<ClusterDeliveryStory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_plan_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authored_input_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authored_input_deduplicated_sources: Vec<String>,
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
    pub executed_steps: Vec<TraceStepSummary>,
    pub recovery_events: Vec<TraceRecoveryEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub governance_timeline: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub review_timeline: Vec<String>,
    pub terminal_status: TaskStatus,
    pub terminal_reason: TerminalReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStepSummary {
    pub step_id: String,
    pub step_kind: StepKind,
    pub attempts: usize,
    pub final_status: StepStatus,
    pub headline: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceRecoveryEvent {
    pub event_type: TraceEventType,
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_step_id: Option<String>,
}

impl ExecutionTrace {
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

    pub fn finalize(&mut self, terminal_status: TaskStatus, terminal_reason: TerminalReason) {
        self.ended_at = Some(current_timestamp_millis());
        self.terminal_status = Some(terminal_status);
        self.terminal_reason = Some(terminal_reason);
    }

    pub fn set_trace_location(&mut self, trace_location: impl Into<String>) {
        self.trace_location = Some(trace_location.into());
    }

    pub fn duration_millis(&self) -> Option<u64> {
        self.ended_at.map(|ended_at| ended_at.saturating_sub(self.started_at))
    }
}
