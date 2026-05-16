//! Session-native persistence models, routing projections, and operator-facing
//! view helpers.

use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::domain::brief::AuthoredBriefBundle;
use crate::domain::cluster::ClusterDeliveryStory;
use crate::domain::decision::{Decision, DecisionStatus};
use crate::domain::flow::SessionFlowState;
use crate::domain::flow_policy::FlowPolicy;
use crate::domain::goal_plan::GoalPlan;
use crate::domain::governance::{
    AutopilotDecisionRecord, CompactedCanonMemory, GovernedSessionLifecycle, GovernedStagePacket,
    GovernedStageRecord, PacketReuseBinding,
};
use crate::domain::negotiation::NegotiatedDeliveryPacket;
use crate::domain::task::{Task, TaskPersistenceError, TaskStatus, TerminalReason};
use crate::domain::task_context::{
    LATEST_GOVERNANCE_APPROVAL_PROVENANCE_KEY, LATEST_GOVERNANCE_CONTRACT_LINES_KEY,
    LATEST_GOVERNANCE_DECISION_KEY, LATEST_GOVERNANCE_PACKET_KEY,
    LATEST_GOVERNANCE_PACKET_REUSE_KEY, LATEST_GOVERNANCE_REASON_KEY,
    LATEST_GOVERNANCE_ROLLOUT_PROFILE_KEY, LATEST_GOVERNANCE_RUNTIME_STATE_KEY,
    LATEST_GOVERNANCE_STAGE_KEY,
};
use crate::domain::trace::current_timestamp_millis;
use crate::domain::workflow::{ProjectScalePath, WorkflowProgressState};

/// Session-side projection of an in-progress project-scale path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScaleSessionState {
    pub path: ProjectScalePath,
    #[serde(default)]
    pub active_stage_index: usize,
    #[serde(default)]
    pub active_work_unit_id: Option<String>,
    #[serde(default)]
    pub checkpoint_refs: Vec<String>,
    #[serde(default)]
    pub trace_refs: Vec<String>,
    pub next_action: String,
}

impl ProjectScaleSessionState {
    /// Returns the active project-scale stage label, if the current index is valid.
    pub fn active_stage_text(&self) -> Option<String> {
        self.path.stages.get(self.active_stage_index).map(|stage| stage.kind.as_str().to_string())
    }
}

/// Persisted voting state attached to the active session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingSessionState {
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_evidence_ref: Option<String>,
    pub result: String,
    #[serde(default)]
    pub reviewer_findings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjudication_result: Option<String>,
    #[serde(default)]
    pub blocking: bool,
    pub next_action: String,
}

/// Lifecycle status of the active workspace session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Initialized,
    GoalCaptured,
    Planned,
    Running,
    Succeeded,
    Failed,
    Exhausted,
    Aborted,
    Invalid,
}

impl SessionStatus {
    /// Returns true when the session is in a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Exhausted | Self::Aborted)
    }
}

/// Session-native commands that can trigger persisted state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionCommand {
    Start,
    Capture,
    Flow,
    Plan,
    Step,
    Run,
    Status,
    Next,
    Inspect,
}

/// Authoritative persisted session snapshot for one workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveSessionRecord {
    pub session_id: String,
    pub workspace_ref: String,
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_brief: Option<AuthoredBriefBundle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_packet: Option<NegotiatedDeliveryPacket>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_flow: Option<SessionFlowState>,
    pub active_task: Option<Task>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_plan: Option<GoalPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_progress: Option<WorkflowProgressState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<Decision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_flow_policy: Option<FlowPolicy>,
    pub latest_status: SessionStatus,
    pub latest_terminal_reason: Option<TerminalReason>,
    pub latest_trace_ref: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_lifecycle: Option<GovernedSessionLifecycle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_scale: Option<ProjectScaleSessionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting: Option<VotingSessionState>,
}

impl ActiveSessionRecord {
    /// Validates the persisted session snapshot and all nested state used by
    /// status, inspect, and runtime orchestration.
    pub fn validate(&self) -> Result<(), SessionValidationError> {
        if self.session_id.trim().is_empty() {
            return Err(SessionValidationError::MissingSessionId);
        }

        if self.workspace_ref.trim().is_empty() {
            return Err(SessionValidationError::MissingWorkspaceRef);
        }

        if self.updated_at < self.created_at {
            return Err(SessionValidationError::UpdatedBeforeCreated {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }

        if status_requires_goal(self.latest_status)
            && self.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(SessionValidationError::MissingGoal(self.latest_status));
        }

        if status_requires_task(self.latest_status)
            && self.active_task.is_none()
            && !status_allows_goal_plan_without_task(self.latest_status, self.goal_plan.as_ref())
        {
            return Err(SessionValidationError::MissingActiveTask(self.latest_status));
        }

        if let Some(active_flow) = &self.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionValidationError::InvalidFlowState(error.to_string()))?;
        }

        if let Some(workflow_progress) = &self.workflow_progress {
            workflow_progress.validate().map_err(|error| {
                SessionValidationError::InvalidWorkflowProgress(error.to_string())
            })?;
        }

        if self.latest_status.is_terminal() && self.latest_terminal_reason.is_none() {
            return Err(SessionValidationError::MissingTerminalReason(self.latest_status));
        }

        if let Some(task) = &self.active_task {
            task.validate_persisted_state()
                .map_err(|error| SessionValidationError::InvalidTask(error.to_string()))?;

            if !task_belongs_to_session_workspace(self, task)? {
                return Err(SessionValidationError::TaskWorkspaceMismatch {
                    expected: self.workspace_ref.clone(),
                    actual: task.context.workspace_ref.clone(),
                });
            }

            if let Some(goal) = &self.goal
                && task.goal.trim() != goal.trim()
            {
                return Err(SessionValidationError::TaskGoalMismatch {
                    expected: goal.clone(),
                    actual: task.goal.clone(),
                });
            }

            if let Some(expected_status) = expected_task_status(self.latest_status)
                && task.status != expected_status
            {
                return Err(SessionValidationError::TaskStatusMismatch {
                    expected: expected_status,
                    actual: task.status,
                });
            }
        }

        if let Some(trace_ref) = &self.latest_trace_ref
            && !trace_within_session_scope(self, trace_ref)
        {
            return Err(SessionValidationError::TraceOutsideWorkspace {
                workspace_ref: self.workspace_ref.clone(),
                trace_ref: trace_ref.clone(),
            });
        }

        Ok(())
    }

    /// Returns the active workflow progress, preferring session-local state and
    /// falling back to the goal plan projection when necessary.
    pub fn active_workflow_progress(&self) -> Option<&WorkflowProgressState> {
        self.workflow_progress.as_ref().or_else(|| {
            self.goal_plan.as_ref().and_then(|goal_plan| goal_plan.workflow_progress.as_ref())
        })
    }

    /// Returns the active workflow name, if one is present.
    pub fn active_workflow_name(&self) -> Option<String> {
        self.active_workflow_progress().map(|workflow| workflow.workflow_name.clone())
    }

    /// Returns the current workflow phase label, if one is active.
    pub fn active_workflow_phase_text(&self) -> Option<String> {
        self.active_workflow_progress().and_then(WorkflowProgressState::current_phase_text)
    }

    /// Returns the next suggested workflow action, if one is active.
    pub fn active_workflow_next_action(&self) -> Option<String> {
        self.active_workflow_progress().and_then(WorkflowProgressState::next_action_text)
    }
}

fn task_belongs_to_session_workspace(
    record: &ActiveSessionRecord,
    task: &Task,
) -> Result<bool, SessionValidationError> {
    if task.context.belongs_to_workspace(&record.workspace_ref) {
        return Ok(true);
    }

    let Some(projection) = task
        .context
        .cluster_session_projection()
        .map_err(|error| SessionValidationError::InvalidTask(error.to_string()))?
    else {
        return Ok(false);
    };

    projection
        .validate()
        .map_err(|error| SessionValidationError::InvalidTask(error.to_string()))?;

    Ok(projection.primary_workspace_ref == record.workspace_ref
        && projection
            .member_workspace_refs
            .iter()
            .any(|workspace_ref| workspace_ref == &task.context.workspace_ref))
}

/// Persisted explanation of one session transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTransition {
    pub trigger_command: SessionCommand,
    pub from_status: Option<SessionStatus>,
    pub to_status: SessionStatus,
    pub trace_ref: Option<String>,
    pub reason: String,
}

impl SessionTransition {
    /// Validates that the transition matches the authoritative session record
    /// it claims to describe.
    pub fn validate(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        if self.reason.trim().is_empty() {
            return Err(SessionValidationError::MissingTransitionReason);
        }

        if self.to_status != record.latest_status {
            return Err(SessionValidationError::TransitionStatusMismatch {
                expected: record.latest_status,
                actual: self.to_status,
            });
        }

        if self.trace_ref != record.latest_trace_ref {
            return Err(SessionValidationError::TransitionTraceMismatch {
                expected: record.latest_trace_ref.clone(),
                actual: self.trace_ref.clone(),
            });
        }

        Ok(())
    }
}

/// Source that currently owns the authoritative follow-up state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuityAuthority {
    NativeSession,
    CompatibilityTrace,
    NoFollowUpState,
}

impl ContinuityAuthority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NativeSession => "native_session",
            Self::CompatibilityTrace => "compatibility_trace",
            Self::NoFollowUpState => "no_follow_up_state",
        }
    }
}

/// Delegation packet kind recorded in continuity history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationPacketKind {
    Handoff,
    Escalation,
}

impl DelegationPacketKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Handoff => "handoff",
            Self::Escalation => "escalation",
        }
    }
}

/// Persisted state of one delegation packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationPacketState {
    Active,
    Resolved,
    Superseded,
    Stuck,
    Exhausted,
}

impl DelegationPacketState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Resolved => "resolved",
            Self::Superseded => "superseded",
            Self::Stuck => "stuck",
            Self::Exhausted => "exhausted",
        }
    }
}

/// Session-level follow-through mode for delegation continuity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationContinuityMode {
    None,
    HandoffRequired,
    EscalationRequired,
    Resolved,
    Stuck,
    Exhausted,
    InspectOnly,
}

impl DelegationContinuityMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::HandoffRequired => "handoff_required",
            Self::EscalationRequired => "escalation_required",
            Self::Resolved => "resolved",
            Self::Stuck => "stuck",
            Self::Exhausted => "exhausted",
            Self::InspectOnly => "inspect_only",
        }
    }

    pub const fn requires_active_packet(self) -> bool {
        matches!(self, Self::HandoffRequired | Self::EscalationRequired | Self::Stuck)
    }
}

/// Recommended operator recovery action when continuity becomes stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StuckRecoveryAction {
    Replan,
    ResolvePacket,
    UpdateConfig,
    RerunValidation,
    Escalate,
}

impl StuckRecoveryAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Replan => "replan",
            Self::ResolvePacket => "resolve_packet",
            Self::UpdateConfig => "update_config",
            Self::RerunValidation => "rerun_validation",
            Self::Escalate => "escalate",
        }
    }
}

/// Evidence that explains why a delegation packet is classified as stuck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StuckEvidenceMarker {
    #[serde(default)]
    pub repeated_attempts: usize,
    #[serde(default)]
    pub same_reason_count: usize,
    #[serde(default)]
    pub unchanged_workspace_signal: bool,
    #[serde(default)]
    pub stale_route_policy: bool,
    pub recommended_recovery: StuckRecoveryAction,
}

impl StuckEvidenceMarker {
    /// Validates that the marker preserves at least one repeated or unchanged signal.
    pub fn validate(&self) -> Result<(), String> {
        if self.repeated_attempts == 0
            && self.same_reason_count == 0
            && !self.unchanged_workspace_signal
            && !self.stale_route_policy
        {
            return Err(
                "stuck evidence marker must preserve at least one repeated or unchanged signal"
                    .to_string(),
            );
        }

        Ok(())
    }
}

/// Persisted delegation packet recorded when continuity leaves the current
/// route owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationPacket {
    pub packet_id: String,
    pub kind: DelegationPacketKind,
    pub state: DelegationPacketState,
    pub created_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<u64>,
    pub source_route_owner: String,
    pub target_owner: String,
    #[serde(default)]
    pub continuity_reason: String,
    pub recommended_next_action: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stuck_marker: Option<StuckEvidenceMarker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by_packet_id: Option<String>,
}

impl DelegationPacket {
    /// Validates the persisted delegation packet shape.
    pub fn validate(&self) -> Result<(), String> {
        if self.packet_id.trim().is_empty() {
            return Err("delegation packet_id must not be empty".to_string());
        }
        if self.source_route_owner.trim().is_empty() {
            return Err("delegation source_route_owner must not be empty".to_string());
        }
        if self.target_owner.trim().is_empty() {
            return Err("delegation target_owner must not be empty".to_string());
        }
        if self.recommended_next_action.trim().is_empty() {
            return Err("delegation recommended_next_action must not be empty".to_string());
        }
        if self.continuity_reason.trim().is_empty() && self.evidence_refs.is_empty() {
            return Err(
                "delegation packet must include decisive evidence or an explicit continuity reason"
                    .to_string(),
            );
        }
        if self.state == DelegationPacketState::Active && self.resolved_at.is_some() {
            return Err("active delegation packets cannot carry resolved_at".to_string());
        }
        if self.state == DelegationPacketState::Stuck && self.stuck_marker.is_none() {
            return Err("stuck delegation packets must carry a stuck marker".to_string());
        }
        if let Some(marker) = &self.stuck_marker {
            marker.validate()?;
        }
        if self.state == DelegationPacketState::Superseded {
            if self.superseded_by_packet_id.as_deref().map(str::trim).unwrap_or_default().is_empty()
            {
                return Err(
                    "superseded delegation packets must name the successor packet".to_string()
                );
            }
        } else if self.superseded_by_packet_id.is_some() {
            return Err(
                "only superseded delegation packets may carry superseded_by_packet_id".to_string()
            );
        }

        Ok(())
    }

    /// Returns a compact operator-facing headline for the packet.
    pub fn headline(&self) -> String {
        let subject = if self.continuity_reason.trim().is_empty() {
            self.evidence_refs
                .first()
                .cloned()
                .unwrap_or_else(|| "bounded continuity requires inspection".to_string())
        } else {
            self.continuity_reason.clone()
        };

        format!("{} required: {subject}", self.kind.as_str())
    }

    /// Returns the most useful evidence summary recorded for the packet.
    pub fn evidence_summary(&self) -> String {
        if !self.evidence_refs.is_empty() {
            return self.evidence_refs.join(", ");
        }
        if let Some(summary) = self.capability_summary.as_ref() {
            return summary.clone();
        }
        self.continuity_reason.clone()
    }

    /// Marks the packet as resolved and records the resolution timestamp.
    pub fn mark_resolved(&mut self) {
        self.state = DelegationPacketState::Resolved;
        self.resolved_at = Some(current_timestamp_millis());
        self.superseded_by_packet_id = None;
    }

    /// Marks the packet as superseded by a newer successor packet.
    pub fn mark_superseded(&mut self, successor_packet_id: impl Into<String>) {
        self.state = DelegationPacketState::Superseded;
        self.superseded_by_packet_id = Some(successor_packet_id.into());
    }
}

/// Persisted continuity state projected from delegation history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationContinuityState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_packet_id: Option<String>,
    pub mode: DelegationContinuityMode,
    pub authority_source: ContinuityAuthority,
    pub next_command: String,
    pub headline: String,
    pub evidence_summary: String,
}

impl DelegationContinuityState {
    /// Validates the continuity view against the packet history it references.
    pub fn validate(&self, packet_history: &[DelegationPacket]) -> Result<(), String> {
        if self.next_command.trim().is_empty() {
            return Err("delegation next_command must not be empty".to_string());
        }
        if self.headline.trim().is_empty() {
            return Err("delegation headline must not be empty".to_string());
        }
        if self.evidence_summary.trim().is_empty() {
            return Err("delegation evidence_summary must not be empty".to_string());
        }

        if self.mode.requires_active_packet() {
            let active_packet_id = self
                .active_packet_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!("delegation mode {} requires an active_packet_id", self.mode.as_str())
                })?;
            let active_packet = packet_history
                .iter()
                .find(|packet| packet.packet_id == active_packet_id)
                .ok_or_else(|| {
                    format!(
                        "delegation active_packet_id `{active_packet_id}` is missing from history"
                    )
                })?;

            match self.mode {
                DelegationContinuityMode::HandoffRequired
                    if active_packet.kind != DelegationPacketKind::Handoff =>
                {
                    return Err(
                        "handoff-required continuity must point to a handoff packet".to_string()
                    );
                }
                DelegationContinuityMode::EscalationRequired
                    if active_packet.kind != DelegationPacketKind::Escalation =>
                {
                    return Err(
                        "escalation-required continuity must point to an escalation packet"
                            .to_string(),
                    );
                }
                DelegationContinuityMode::Stuck
                    if active_packet.state != DelegationPacketState::Stuck =>
                {
                    return Err(
                        "stuck continuity must point to a stuck delegation packet".to_string()
                    );
                }
                _ => {}
            }
        } else if self.active_packet_id.is_some() {
            return Err(format!(
                "delegation mode {} must not keep an active_packet_id",
                self.mode.as_str()
            ));
        }

        if self.authority_source == ContinuityAuthority::NoFollowUpState
            && self.mode != DelegationContinuityMode::None
        {
            return Err("no_follow_up_state authority may only be used with delegation mode none"
                .to_string());
        }

        Ok(())
    }
}

/// Follow-up mode exposed when compatibility traces remain authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityFollowUpMode {
    InspectOnly,
    Resumable,
    Superseded,
}

impl CompatibilityFollowUpMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InspectOnly => "inspect_only",
            Self::Resumable => "resumable",
            Self::Superseded => "superseded",
        }
    }
}

/// Flattened follow-up projection derived from a compatibility trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityFollowUpView {
    pub follow_up_mode: CompatibilityFollowUpMode,
    pub trace_ref: String,
    pub routing_summary: String,
    pub execution_condition: String,
    pub terminal_status: TaskStatus,
    pub terminal_reason: String,
    pub next_command: String,
}

/// Flattened delegation projection reused by status and inspect surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationStatusView {
    pub mode: DelegationContinuityMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_kind: Option<DelegationPacketKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_state: Option<DelegationPacketState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_owner: Option<String>,
    pub headline: String,
    pub evidence_summary: String,
}

impl DelegationStatusView {
    /// Builds the flattened delegation view from continuity state and packet history.
    pub fn from_continuity(
        continuity: &DelegationContinuityState,
        packet_history: &[DelegationPacket],
    ) -> Result<Self, String> {
        continuity.validate(packet_history)?;
        let packet = continuity
            .active_packet_id
            .as_ref()
            .and_then(|packet_id| {
                packet_history.iter().find(|packet| packet.packet_id == *packet_id)
            })
            .or_else(|| packet_history.last());

        Ok(Self {
            mode: continuity.mode,
            packet_id: packet.map(|packet| packet.packet_id.clone()),
            packet_kind: packet.map(|packet| packet.kind),
            packet_state: packet.map(|packet| packet.state),
            target_owner: packet.map(|packet| packet.target_owner.clone()),
            headline: continuity.headline.clone(),
            evidence_summary: continuity.evidence_summary.clone(),
        })
    }
}

/// Returns the authoritative delegation view for the current session, whether
/// it lives on the goal plan or active task context.
pub fn delegation_status_view(record: &ActiveSessionRecord) -> Option<DelegationStatusView> {
    if let Some(goal_plan) = record.goal_plan.as_ref()
        && let Some(continuity) = goal_plan.delegation_continuity().cloned()
    {
        return DelegationStatusView::from_continuity(
            &continuity,
            goal_plan.delegation_packet_history(),
        )
        .ok();
    }

    let task = record.active_task.as_ref()?;
    let continuity = task.context.delegation_continuity_state().ok().flatten()?;
    let packet_history = task.context.delegation_packet_history().ok()?;
    DelegationStatusView::from_continuity(&continuity, &packet_history).ok()
}

/// Returns the next delegation command recorded in the authoritative session state.
pub fn delegation_next_command(record: &ActiveSessionRecord) -> Option<String> {
    if let Some(goal_plan) = record.goal_plan.as_ref()
        && let Some(continuity) = goal_plan.delegation_continuity()
    {
        return Some(continuity.next_command.clone());
    }

    record
        .active_task
        .as_ref()
        .and_then(|task| task.context.delegation_continuity_state().ok().flatten())
        .map(|continuity| continuity.next_command)
}

/// Flattened read-side projection used by `status`, `next`, and related CLI
/// surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStatusView {
    pub session_id: String,
    pub workspace_ref: String,
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_goal_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_resolution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_acceptance_boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_delivery_story: Option<ClusterDeliveryStory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_sources: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_deduplicated_sources: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_credibility: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_primary_inputs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_provenance: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_staleness_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_headline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_missing_fields: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_governance_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_flow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_plan_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_plan_revision: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_authority: Option<ContinuityAuthority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation: Option<DelegationStatusView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility_follow_up: Option<CompatibilityFollowUpView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_stage_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_stages: Option<usize>,
    pub plan_revision: Option<usize>,
    pub current_step_id: Option<String>,
    pub current_step_index: Option<usize>,
    pub latest_status: SessionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_path: Option<String>,
    pub latest_trace_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_decision_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_decision_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_changed_files: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_checkpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_checkpoint_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_checkpoint_restore_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_workspace_slice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_selection_headline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_candidate_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_selection_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_rejected_candidates: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_attempt_lineage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_validation_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_exhaustion_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_vote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_council_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_independence_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_stop_semantics: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_selection_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_headline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_run_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_runtime_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_rollout_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_contract_lines: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_approval_provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_blocked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_packet_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_packet_source_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_packet_binding_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_approval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_governance_candidates: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_lifecycle_runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_lifecycle_opt_out: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_lifecycle_mode_selection: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_lifecycle_selected_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_scale_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_scale_current_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_scale_next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_scale_checkpoint_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting_trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting_result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting_adjudication: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting_reviewed_evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting_blocking: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_voting_next_action: Option<String>,
    pub next_command: Option<String>,
    pub explanation: String,
}

impl Default for SessionStatusView {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            workspace_ref: String::new(),
            goal: None,
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            authored_input_summary: None,
            authored_input_sources: None,
            authored_input_deduplicated_sources: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: None,
            context_provenance: None,
            context_staleness_reason: None,
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: None,
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            active_flow: None,
            flow_state: None,
            goal_plan_state: None,
            goal_plan_revision: None,
            planning_rationale: None,
            verification_strategy: None,
            active_workflow: None,
            workflow_phase: None,
            workflow_next_action: None,
            continuity_authority: None,
            delegation: None,
            compatibility_follow_up: None,
            current_stage_id: None,
            current_stage_index: None,
            total_stages: None,
            plan_revision: None,
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Initialized,
            execution_path: None,
            latest_trace_ref: None,
            latest_decision_status: None,
            latest_decision_target: None,
            latest_changed_files: None,
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            latest_workspace_slice: None,
            latest_selection_headline: None,
            latest_candidate_family: None,
            latest_selection_reason: None,
            latest_rejected_candidates: None,
            latest_attempt_lineage: None,
            latest_validation_status: None,
            latest_exhaustion_reason: None,
            latest_review_trigger: None,
            latest_review_vote: None,
            latest_review_outcome: None,
            latest_review_council_profile: None,
            latest_review_independence_state: None,
            latest_review_stop_semantics: None,
            latest_review_selection_summary: None,
            latest_review_headline: None,
            latest_governance_stage: None,
            latest_governance_runtime: None,
            latest_governance_mode: None,
            latest_governance_run_ref: None,
            latest_governance_state: None,
            latest_governance_runtime_state: None,
            latest_governance_rollout_profile: None,
            latest_governance_reason: None,
            latest_governance_contract_lines: None,
            latest_governance_approval_provenance: None,
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: None,
            latest_governance_packet_source_stage: None,
            latest_governance_packet_binding_reason: None,
            latest_governance_approval: None,
            latest_governance_decision: None,
            latest_governance_candidates: None,
            governance_next_action: None,
            governance_lifecycle_runtime: None,
            governance_lifecycle_opt_out: None,
            governance_lifecycle_mode_selection: None,
            governance_lifecycle_selected_mode: None,
            project_scale_path: None,
            project_scale_current_stage: None,
            project_scale_next_action: None,
            project_scale_checkpoint_refs: None,
            latest_voting_trigger: None,
            latest_voting_result: None,
            latest_voting_adjudication: None,
            latest_voting_reviewed_evidence: None,
            latest_voting_blocking: None,
            latest_voting_next_action: None,
            next_command: None,
            explanation: String::new(),
        }
    }
}

impl SessionStatusView {
    /// Validates that the flattened status view matches the authoritative
    /// session record it was projected from.
    pub fn validate(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        self.validate_identity(record)?;
        self.validate_negotiation(record)?;
        self.validate_context(record)?;
        self.validate_flow(record)?;
        self.validate_trace_and_decisions(record)?;
        self.validate_authored_brief(record)?;
        self.validate_task_state(record)?;
        self.validate_governance(record)?;
        self.validate_project_scale(record)?;
        self.validate_voting(record)?;
        self.validate_invariants()?;
        self.validate_active_task_plan(record)?;
        Ok(())
    }

    fn validate_identity(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        if self.session_id != record.session_id {
            return Err(SessionValidationError::StatusViewSessionMismatch {
                expected: record.session_id.clone(),
                actual: self.session_id.clone(),
            });
        }
        if self.workspace_ref != record.workspace_ref {
            return Err(SessionValidationError::StatusViewWorkspaceMismatch {
                expected: record.workspace_ref.clone(),
                actual: self.workspace_ref.clone(),
            });
        }
        if self.latest_status != record.latest_status {
            return Err(SessionValidationError::StatusViewStatusMismatch {
                expected: record.latest_status,
                actual: self.latest_status,
            });
        }
        if self.goal != record.goal {
            return Err(SessionValidationError::StatusViewGoalMismatch {
                expected: record.goal.clone(),
                actual: self.goal.clone(),
            });
        }
        Ok(())
    }

    fn validate_negotiation(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        let expected_negotiation_goal_summary =
            record.negotiation_packet.as_ref().map(|packet| packet.goal_summary.clone());
        if self.negotiation_goal_summary != expected_negotiation_goal_summary {
            return Err(SessionValidationError::StatusViewNegotiationGoalSummaryMismatch {
                expected: expected_negotiation_goal_summary,
                actual: self.negotiation_goal_summary.clone(),
            });
        }
        let expected_negotiation_resolution = record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.resolution_state.as_str().to_string());
        if self.negotiation_resolution != expected_negotiation_resolution {
            return Err(SessionValidationError::StatusViewNegotiationResolutionMismatch {
                expected: expected_negotiation_resolution,
                actual: self.negotiation_resolution.clone(),
            });
        }
        let expected_negotiation_acceptance_boundary = record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.acceptance_boundary.success_headline.clone());
        if self.negotiation_acceptance_boundary != expected_negotiation_acceptance_boundary {
            return Err(SessionValidationError::StatusViewNegotiationAcceptanceBoundaryMismatch {
                expected: expected_negotiation_acceptance_boundary,
                actual: self.negotiation_acceptance_boundary.clone(),
            });
        }
        Ok(())
    }

    fn validate_context(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        let expected_context_summary =
            record.goal_plan.as_ref().and_then(|goal_plan| goal_plan.context_summary()).or_else(
                || record.active_task.as_ref().and_then(task_state_canon_memory_context_summary),
            );
        if self.context_summary != expected_context_summary {
            return Err(SessionValidationError::StatusViewContextSummaryMismatch {
                expected: expected_context_summary,
                actual: self.context_summary.clone(),
            });
        }
        let expected_context_credibility = record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_credibility())
            .or_else(|| {
                record.active_task.as_ref().and_then(task_state_canon_memory_context_credibility)
            });
        if self.context_credibility != expected_context_credibility {
            return Err(SessionValidationError::StatusViewContextCredibilityMismatch {
                expected: expected_context_credibility,
                actual: self.context_credibility.clone(),
            });
        }
        let expected_context_primary_inputs = record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| {
                let inputs = goal_plan.context_primary_inputs();
                (!inputs.is_empty()).then_some(inputs)
            })
            .or_else(|| {
                record.active_task.as_ref().and_then(task_state_canon_memory_primary_inputs)
            });
        if self.context_primary_inputs != expected_context_primary_inputs {
            return Err(SessionValidationError::StatusViewContextPrimaryInputsMismatch {
                expected: expected_context_primary_inputs,
                actual: self.context_primary_inputs.clone(),
            });
        }
        let expected_context_provenance = record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| {
                let lines = goal_plan.context_provenance_lines();
                (!lines.is_empty()).then_some(lines)
            })
            .or_else(|| record.active_task.as_ref().and_then(task_state_canon_memory_provenance));
        if self.context_provenance != expected_context_provenance {
            return Err(SessionValidationError::StatusViewContextProvenanceMismatch {
                expected: expected_context_provenance,
                actual: self.context_provenance.clone(),
            });
        }
        let expected_context_staleness_reason = record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_pack.as_ref())
            .and_then(|pack| pack.staleness_reason.clone())
            .or_else(|| record.goal_plan.as_ref().and_then(GoalPlan::canon_memory_staleness_reason))
            .or_else(|| {
                record.active_task.as_ref().and_then(task_state_canon_memory_staleness_reason)
            });
        if self.context_staleness_reason != expected_context_staleness_reason {
            return Err(SessionValidationError::StatusViewContextStalenessReasonMismatch {
                expected: expected_context_staleness_reason,
                actual: self.context_staleness_reason.clone(),
            });
        }
        Ok(())
    }

    fn validate_flow(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        let expected_flow = record.active_flow.as_ref().map(|flow| flow.flow_name.clone());
        if self.active_flow != expected_flow {
            return Err(SessionValidationError::StatusViewFlowMismatch {
                expected: expected_flow,
                actual: self.active_flow.clone(),
            });
        }
        let expected_flow_state =
            record.goal_plan.as_ref().map(|goal_plan| goal_plan.flow_state().summary_text());
        if self.flow_state != expected_flow_state {
            return Err(SessionValidationError::StatusViewFlowStateMismatch {
                expected: expected_flow_state,
                actual: self.flow_state.clone(),
            });
        }
        let expected_active_workflow = record.active_workflow_name();
        if self.active_workflow != expected_active_workflow {
            return Err(SessionValidationError::StatusViewWorkflowMismatch {
                expected: expected_active_workflow,
                actual: self.active_workflow.clone(),
            });
        }
        let expected_workflow_phase = record.active_workflow_phase_text();
        if self.workflow_phase != expected_workflow_phase {
            return Err(SessionValidationError::StatusViewWorkflowPhaseMismatch {
                expected: expected_workflow_phase,
                actual: self.workflow_phase.clone(),
            });
        }
        let expected_workflow_next_action = record.active_workflow_next_action();
        if self.workflow_next_action != expected_workflow_next_action {
            return Err(SessionValidationError::StatusViewWorkflowNextActionMismatch {
                expected: expected_workflow_next_action,
                actual: self.workflow_next_action.clone(),
            });
        }
        let expected_stage_id =
            record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone());
        if self.current_stage_id != expected_stage_id {
            return Err(SessionValidationError::StatusViewStageMismatch {
                expected: expected_stage_id,
                actual: self.current_stage_id.clone(),
            });
        }
        let expected_delegation = delegation_status_view(record);
        if self.delegation != expected_delegation {
            return Err(SessionValidationError::StatusViewDelegationMismatch {
                expected: expected_delegation.map(Box::new),
                actual: self.delegation.clone().map(Box::new),
            });
        }
        let expected_stage_index = record.active_flow.as_ref().map(|flow| flow.current_stage_index);
        if self.current_stage_index != expected_stage_index {
            return Err(SessionValidationError::StatusViewStageIndexMismatch {
                expected: expected_stage_index,
                actual: self.current_stage_index,
            });
        }
        let expected_total_stages = record.active_flow.as_ref().map(|flow| flow.total_stages);
        if self.total_stages != expected_total_stages {
            return Err(SessionValidationError::StatusViewStageCountMismatch {
                expected: expected_total_stages,
                actual: self.total_stages,
            });
        }
        Ok(())
    }

    fn validate_trace_and_decisions(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        if self.latest_trace_ref != record.latest_trace_ref {
            return Err(SessionValidationError::StatusViewTraceMismatch {
                expected: record.latest_trace_ref.clone(),
                actual: self.latest_trace_ref.clone(),
            });
        }
        let expected_latest_decision_status = record
            .decisions
            .last()
            .map(|decision| decision_status_text(decision.status).to_string());
        if self.latest_decision_status != expected_latest_decision_status {
            return Err(SessionValidationError::StatusViewDecisionStatusMismatch {
                expected: expected_latest_decision_status,
                actual: self.latest_decision_status.clone(),
            });
        }
        let expected_latest_decision_target =
            record.decisions.last().map(|decision| decision.target.clone());
        if self.latest_decision_target != expected_latest_decision_target {
            return Err(SessionValidationError::StatusViewDecisionTargetMismatch {
                expected: expected_latest_decision_target,
                actual: self.latest_decision_target.clone(),
            });
        }
        Ok(())
    }

    fn validate_authored_brief(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        let expected_authored_input_deduplicated_sources =
            record.authored_brief.as_ref().and_then(|bundle| {
                let labels = bundle.deduplicated_source_labels();
                (!labels.is_empty()).then_some(labels)
            });
        if self.authored_input_deduplicated_sources != expected_authored_input_deduplicated_sources
        {
            return Err(
                SessionValidationError::StatusViewAuthoredInputDeduplicatedSourcesMismatch {
                    expected: expected_authored_input_deduplicated_sources,
                    actual: self.authored_input_deduplicated_sources.clone(),
                },
            );
        }
        let expected_clarification_headline =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_headline());
        if self.clarification_headline != expected_clarification_headline {
            return Err(SessionValidationError::StatusViewClarificationHeadlineMismatch {
                expected: expected_clarification_headline,
                actual: self.clarification_headline.clone(),
            });
        }
        let expected_clarification_prompt =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_prompt());
        if self.clarification_prompt != expected_clarification_prompt {
            return Err(SessionValidationError::StatusViewClarificationPromptMismatch {
                expected: expected_clarification_prompt,
                actual: self.clarification_prompt.clone(),
            });
        }
        let expected_clarification_missing_fields =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_missing_fields());
        if self.clarification_missing_fields != expected_clarification_missing_fields {
            return Err(SessionValidationError::StatusViewClarificationMissingFieldsMismatch {
                expected: expected_clarification_missing_fields,
                actual: self.clarification_missing_fields.clone(),
            });
        }
        Ok(())
    }

    fn validate_task_state(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        let expected_changed_files = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_strings(task, "latest_changed_files"));
        if self.latest_changed_files != expected_changed_files {
            return Err(SessionValidationError::StatusViewChangedFilesMismatch {
                expected: expected_changed_files,
                actual: self.latest_changed_files.clone(),
            });
        }
        let expected_workspace_slice =
            record.active_task.as_ref().and_then(task_state_workspace_slice_summary);
        if self.latest_workspace_slice != expected_workspace_slice {
            return Err(SessionValidationError::StatusViewWorkspaceSliceMismatch {
                expected: expected_workspace_slice,
                actual: self.latest_workspace_slice.clone(),
            });
        }
        let expected_selection_headline = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_selection_headline"));
        if self.latest_selection_headline != expected_selection_headline {
            return Err(SessionValidationError::StatusViewSelectionHeadlineMismatch {
                expected: expected_selection_headline,
                actual: self.latest_selection_headline.clone(),
            });
        }
        let expected_attempt_lineage =
            record.active_task.as_ref().and_then(task_state_attempt_lineage_summary);
        if self.latest_attempt_lineage != expected_attempt_lineage {
            return Err(SessionValidationError::StatusViewAttemptLineageMismatch {
                expected: expected_attempt_lineage,
                actual: self.latest_attempt_lineage.clone(),
            });
        }
        let expected_validation_status = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_validation_status"));
        if self.latest_validation_status != expected_validation_status {
            return Err(SessionValidationError::StatusViewValidationStatusMismatch {
                expected: expected_validation_status,
                actual: self.latest_validation_status.clone(),
            });
        }
        let expected_review_trigger = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_review_trigger"));
        if self.latest_review_trigger != expected_review_trigger {
            return Err(SessionValidationError::StatusViewReviewTriggerMismatch {
                expected: expected_review_trigger,
                actual: self.latest_review_trigger.clone(),
            });
        }
        let expected_review_vote = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_review_vote"));
        if self.latest_review_vote != expected_review_vote {
            return Err(SessionValidationError::StatusViewReviewVoteMismatch {
                expected: expected_review_vote,
                actual: self.latest_review_vote.clone(),
            });
        }
        let expected_review_outcome = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_review_outcome"));
        if self.latest_review_outcome != expected_review_outcome {
            return Err(SessionValidationError::StatusViewReviewOutcomeMismatch {
                expected: expected_review_outcome,
                actual: self.latest_review_outcome.clone(),
            });
        }
        let expected_review_headline =
            record.active_task.as_ref().and_then(task_state_review_headline);
        if self.latest_review_headline != expected_review_headline {
            return Err(SessionValidationError::StatusViewReviewHeadlineMismatch {
                expected: expected_review_headline,
                actual: self.latest_review_headline.clone(),
            });
        }
        Ok(())
    }

    fn validate_governance(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        let expected_governance_stage =
            record.active_task.as_ref().and_then(task_state_governance_stage_key);
        if self.latest_governance_stage != expected_governance_stage {
            return Err(SessionValidationError::StatusViewGovernanceStageMismatch {
                expected: expected_governance_stage,
                actual: self.latest_governance_stage.clone(),
            });
        }
        let expected_governance_runtime =
            record.active_task.as_ref().and_then(task_state_governance_runtime_text);
        if self.latest_governance_runtime != expected_governance_runtime {
            return Err(SessionValidationError::StatusViewGovernanceRuntimeMismatch {
                expected: expected_governance_runtime,
                actual: self.latest_governance_runtime.clone(),
            });
        }
        let expected_governance_mode =
            record.active_task.as_ref().and_then(task_state_governance_mode_text);
        if self.latest_governance_mode != expected_governance_mode {
            return Err(SessionValidationError::StatusViewGovernanceModeMismatch {
                expected: expected_governance_mode,
                actual: self.latest_governance_mode.clone(),
            });
        }
        let expected_governance_run_ref =
            record.active_task.as_ref().and_then(task_state_governance_canon_run_ref);
        if self.latest_governance_run_ref != expected_governance_run_ref {
            return Err(SessionValidationError::StatusViewGovernanceRunRefMismatch {
                expected: expected_governance_run_ref,
                actual: self.latest_governance_run_ref.clone(),
            });
        }
        let expected_governance_state =
            record.active_task.as_ref().and_then(task_state_governance_state_text);
        if self.latest_governance_state != expected_governance_state {
            return Err(SessionValidationError::StatusViewGovernanceStateMismatch {
                expected: expected_governance_state,
                actual: self.latest_governance_state.clone(),
            });
        }
        let expected_governance_blocked_reason =
            record.active_task.as_ref().and_then(task_state_governance_blocked_reason);
        if self.latest_governance_blocked_reason != expected_governance_blocked_reason {
            return Err(SessionValidationError::StatusViewGovernanceBlockedReasonMismatch {
                expected: expected_governance_blocked_reason,
                actual: self.latest_governance_blocked_reason.clone(),
            });
        }
        let expected_governance_packet_ref =
            record.active_task.as_ref().and_then(task_state_governance_packet_ref);
        if self.latest_governance_packet_ref != expected_governance_packet_ref {
            return Err(SessionValidationError::StatusViewGovernancePacketRefMismatch {
                expected: expected_governance_packet_ref,
                actual: self.latest_governance_packet_ref.clone(),
            });
        }
        let expected_governance_packet_source_stage =
            record.active_task.as_ref().and_then(task_state_governance_packet_source_stage);
        if self.latest_governance_packet_source_stage != expected_governance_packet_source_stage {
            return Err(SessionValidationError::StatusViewGovernancePacketSourceMismatch {
                expected: expected_governance_packet_source_stage,
                actual: self.latest_governance_packet_source_stage.clone(),
            });
        }
        let expected_governance_packet_binding_reason =
            record.active_task.as_ref().and_then(task_state_governance_packet_binding_reason);
        if self.latest_governance_packet_binding_reason != expected_governance_packet_binding_reason
        {
            return Err(SessionValidationError::StatusViewGovernancePacketBindingMismatch {
                expected: expected_governance_packet_binding_reason,
                actual: self.latest_governance_packet_binding_reason.clone(),
            });
        }
        let expected_governance_approval =
            record.active_task.as_ref().and_then(task_state_governance_approval_text);
        if self.latest_governance_approval != expected_governance_approval {
            return Err(SessionValidationError::StatusViewGovernanceApprovalMismatch {
                expected: expected_governance_approval,
                actual: self.latest_governance_approval.clone(),
            });
        }
        let expected_governance_decision =
            record.active_task.as_ref().and_then(task_state_governance_decision_headline);
        if self.latest_governance_decision != expected_governance_decision {
            return Err(SessionValidationError::StatusViewGovernanceDecisionMismatch {
                expected: expected_governance_decision,
                actual: self.latest_governance_decision.clone(),
            });
        }
        let expected_governance_candidates =
            record.active_task.as_ref().and_then(task_state_governance_candidate_actions);
        if self.latest_governance_candidates != expected_governance_candidates {
            return Err(SessionValidationError::StatusViewGovernanceCandidatesMismatch {
                expected: expected_governance_candidates,
                actual: self.latest_governance_candidates.clone(),
            });
        }
        let expected_governance_next_action =
            record.active_task.as_ref().and_then(task_state_governance_next_action);
        if self.governance_next_action != expected_governance_next_action {
            return Err(SessionValidationError::StatusViewGovernanceNextActionMismatch {
                expected: expected_governance_next_action,
                actual: self.governance_next_action.clone(),
            });
        }
        Ok(())
    }

    fn validate_project_scale(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        let expected_project_scale_path =
            record.project_scale.as_ref().map(|state| state.path.stage_names());
        if self.project_scale_path != expected_project_scale_path {
            return Err(SessionValidationError::StatusViewProjectScalePathMismatch {
                expected: expected_project_scale_path,
                actual: self.project_scale_path.clone(),
            });
        }
        let expected_project_scale_stage =
            record.project_scale.as_ref().and_then(ProjectScaleSessionState::active_stage_text);
        if self.project_scale_current_stage != expected_project_scale_stage {
            return Err(SessionValidationError::StatusViewProjectScaleStageMismatch {
                expected: expected_project_scale_stage,
                actual: self.project_scale_current_stage.clone(),
            });
        }
        let expected_project_scale_next =
            record.project_scale.as_ref().map(|state| state.next_action.clone());
        if self.project_scale_next_action != expected_project_scale_next {
            return Err(SessionValidationError::StatusViewProjectScaleNextActionMismatch {
                expected: expected_project_scale_next,
                actual: self.project_scale_next_action.clone(),
            });
        }
        let expected_project_scale_checkpoints = record.project_scale.as_ref().and_then(|state| {
            (!state.checkpoint_refs.is_empty()).then_some(state.checkpoint_refs.clone())
        });
        if self.project_scale_checkpoint_refs != expected_project_scale_checkpoints {
            return Err(SessionValidationError::StatusViewProjectScaleCheckpointRefsMismatch {
                expected: expected_project_scale_checkpoints,
                actual: self.project_scale_checkpoint_refs.clone(),
            });
        }
        Ok(())
    }

    fn validate_voting(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        let expected_vote = record.latest_voting.as_ref();
        if self.latest_voting_trigger != expected_vote.map(|vote| vote.trigger.clone()) {
            return Err(SessionValidationError::StatusViewVotingTriggerMismatch {
                expected: expected_vote.map(|vote| vote.trigger.clone()),
                actual: self.latest_voting_trigger.clone(),
            });
        }
        if self.latest_voting_result != expected_vote.map(|vote| vote.result.clone()) {
            return Err(SessionValidationError::StatusViewVotingResultMismatch {
                expected: expected_vote.map(|vote| vote.result.clone()),
                actual: self.latest_voting_result.clone(),
            });
        }
        if self.latest_voting_adjudication
            != expected_vote.and_then(|vote| vote.adjudication_result.clone())
        {
            return Err(SessionValidationError::StatusViewVotingAdjudicationMismatch {
                expected: expected_vote.and_then(|vote| vote.adjudication_result.clone()),
                actual: self.latest_voting_adjudication.clone(),
            });
        }
        if self.latest_voting_reviewed_evidence
            != expected_vote.and_then(|vote| vote.reviewed_evidence_ref.clone())
        {
            return Err(SessionValidationError::StatusViewVotingEvidenceMismatch {
                expected: expected_vote.and_then(|vote| vote.reviewed_evidence_ref.clone()),
                actual: self.latest_voting_reviewed_evidence.clone(),
            });
        }
        if self.latest_voting_blocking != expected_vote.map(|vote| vote.blocking) {
            return Err(SessionValidationError::StatusViewVotingBlockingMismatch {
                expected: expected_vote.map(|vote| vote.blocking),
                actual: self.latest_voting_blocking,
            });
        }
        if self.latest_voting_next_action != expected_vote.map(|vote| vote.next_action.clone()) {
            return Err(SessionValidationError::StatusViewVotingNextActionMismatch {
                expected: expected_vote.map(|vote| vote.next_action.clone()),
                actual: self.latest_voting_next_action.clone(),
            });
        }
        Ok(())
    }

    fn validate_invariants(&self) -> Result<(), SessionValidationError> {
        if self.explanation.trim().is_empty() {
            return Err(SessionValidationError::MissingStatusExplanation);
        }
        if let Some(governance_next_action) = &self.governance_next_action
            && governance_next_action.trim().is_empty()
        {
            return Err(SessionValidationError::MissingGovernanceNextAction);
        }
        if let Some(next_command) = &self.next_command
            && next_command.trim().is_empty()
        {
            return Err(SessionValidationError::MissingNextCommand);
        }
        Ok(())
    }

    fn validate_active_task_plan(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<(), SessionValidationError> {
        if let Some(task) = &record.active_task {
            let expected_index = task.plan.current_step_index;
            if self.current_step_index != Some(expected_index) {
                return Err(SessionValidationError::StatusViewStepIndexMismatch {
                    expected: Some(expected_index),
                    actual: self.current_step_index,
                });
            }
            let expected_step_id = task.plan.current_step().map(|step| step.id.clone());
            if self.current_step_id != expected_step_id {
                return Err(SessionValidationError::StatusViewStepIdMismatch {
                    expected: expected_step_id,
                    actual: self.current_step_id.clone(),
                });
            }
            if self.plan_revision != Some(task.plan.revision) {
                return Err(SessionValidationError::StatusViewPlanRevisionMismatch {
                    expected: Some(task.plan.revision),
                    actual: self.plan_revision,
                });
            }
        }
        Ok(())
    }
}

/// High-level execution mode selected for the current session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    Native,
    Compatibility,
    Blocked,
}

impl RoutingMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Compatibility => "compatibility",
            Self::Blocked => "blocked",
        }
    }
}

/// Persisted source that explains why the current routing mode was selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingSource {
    GoalPlan,
    ExecutionProfile,
    GoalCapture,
    SessionState,
}

impl RoutingSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GoalPlan => "goal_plan",
            Self::ExecutionProfile => "execution_profile",
            Self::GoalCapture => "goal_capture",
            Self::SessionState => "session_state",
        }
    }
}

/// Flattened routing projection derived from the active session snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingOutcome {
    pub mode: RoutingMode,
    pub source: RoutingSource,
    pub reason: String,
}

impl RoutingOutcome {
    /// Returns the stable execution-path key used by operator-facing surfaces,
    /// when the current routing combination maps to one.
    pub fn execution_path_key(&self) -> Option<&'static str> {
        match (self.mode, self.source) {
            (RoutingMode::Native, RoutingSource::GoalPlan) => Some("native_goal_plan"),
            (RoutingMode::Compatibility, RoutingSource::ExecutionProfile) => {
                Some("fixture_compatibility")
            }
            (RoutingMode::Blocked, RoutingSource::GoalPlan) => {
                Some("native_goal_plan_pending_plan_confirmation")
            }
            (RoutingMode::Blocked, RoutingSource::GoalCapture) => {
                Some("native_session_pending_plan")
            }
            _ => None,
        }
    }
}

/// Resolves the current routing outcome from the authoritative session state.
pub fn routing_outcome(record: &ActiveSessionRecord) -> RoutingOutcome {
    if let Some(delegation) = delegation_status_view(record) {
        match delegation.mode {
            DelegationContinuityMode::HandoffRequired
            | DelegationContinuityMode::EscalationRequired
            | DelegationContinuityMode::Stuck
            | DelegationContinuityMode::InspectOnly
            | DelegationContinuityMode::Exhausted => {
                return RoutingOutcome {
                    mode: RoutingMode::Blocked,
                    source: RoutingSource::SessionState,
                    reason: delegation.headline,
                };
            }
            DelegationContinuityMode::Resolved | DelegationContinuityMode::None => {}
        }
    }

    if let Some(goal_plan) = record.goal_plan.as_ref() {
        if goal_plan.requires_confirmation() {
            return RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::GoalPlan,
                reason: "plan confirmation is still pending before native execution".to_string(),
            };
        }

        return RoutingOutcome {
            mode: RoutingMode::Native,
            source: RoutingSource::GoalPlan,
            reason: "goal plan is ready for native execution".to_string(),
        };
    }

    if record.active_task.is_some() {
        return RoutingOutcome {
            mode: RoutingMode::Compatibility,
            source: RoutingSource::ExecutionProfile,
            reason: "compatibility execution remains active from the persisted task".to_string(),
        };
    }

    if record.goal.is_some() {
        return RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalCapture,
            reason: "goal captured but a goal plan is not ready yet".to_string(),
        };
    }

    RoutingOutcome {
        mode: RoutingMode::Blocked,
        source: RoutingSource::SessionState,
        reason: "session has no goal plan or compatibility task to route".to_string(),
    }
}

/// Returns the stable execution-path label for the current session, if one applies.
pub fn execution_path_text(record: &ActiveSessionRecord) -> Option<String> {
    routing_outcome(record).execution_path_key().map(str::to_string)
}

/// Returns the operator-facing label for a persisted decision status.
pub fn decision_status_text(status: DecisionStatus) -> &'static str {
    match status {
        DecisionStatus::Pending => "pending",
        DecisionStatus::Dispatched => "dispatched",
        DecisionStatus::Verified => "verified",
        DecisionStatus::Failed => "failed",
        DecisionStatus::Recovered => "recovered",
    }
}

/// Validation failures for persisted session and session-view state.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SessionValidationError {
    #[error("session_id must not be empty")]
    MissingSessionId,
    #[error("workspace_ref must not be empty")]
    MissingWorkspaceRef,
    #[error("updated_at {updated_at} must be greater than or equal to created_at {created_at}")]
    UpdatedBeforeCreated { created_at: u64, updated_at: u64 },
    #[error("status {0:?} requires a goal")]
    MissingGoal(SessionStatus),
    #[error("status {0:?} requires an active task")]
    MissingActiveTask(SessionStatus),
    #[error("session flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("status {0:?} requires a terminal reason")]
    MissingTerminalReason(SessionStatus),
    #[error("session task workspace_ref mismatch: expected {expected}, got {actual}")]
    TaskWorkspaceMismatch { expected: String, actual: String },
    #[error("session task goal mismatch: expected {expected}, got {actual}")]
    TaskGoalMismatch { expected: String, actual: String },
    #[error("session task status mismatch: expected {expected:?}, got {actual:?}")]
    TaskStatusMismatch { expected: TaskStatus, actual: TaskStatus },
    #[error("latest_trace_ref {trace_ref} must point inside workspace {workspace_ref}")]
    TraceOutsideWorkspace { workspace_ref: String, trace_ref: String },
    #[error("active task is invalid: {0}")]
    InvalidTask(String),
    #[error("workflow progress is invalid: {0}")]
    InvalidWorkflowProgress(String),
    #[error("session transition reason must not be empty")]
    MissingTransitionReason,
    #[error("session transition status mismatch: expected {expected:?}, got {actual:?}")]
    TransitionStatusMismatch { expected: SessionStatus, actual: SessionStatus },
    #[error("session transition trace mismatch: expected {expected:?}, got {actual:?}")]
    TransitionTraceMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view session mismatch: expected {expected}, got {actual}")]
    StatusViewSessionMismatch { expected: String, actual: String },
    #[error("status view workspace mismatch: expected {expected}, got {actual}")]
    StatusViewWorkspaceMismatch { expected: String, actual: String },
    #[error("status view status mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStatusMismatch { expected: SessionStatus, actual: SessionStatus },
    #[error("status view goal mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGoalMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view negotiation goal summary mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewNegotiationGoalSummaryMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view negotiation resolution mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewNegotiationResolutionMismatch { expected: Option<String>, actual: Option<String> },
    #[error(
        "status view negotiation acceptance boundary mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewNegotiationAcceptanceBoundaryMismatch {
        expected: Option<String>,
        actual: Option<String>,
    },
    #[error("status view context summary mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewContextSummaryMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view context credibility mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewContextCredibilityMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view context primary inputs mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewContextPrimaryInputsMismatch {
        expected: Option<Vec<String>>,
        actual: Option<Vec<String>>,
    },
    #[error("status view context provenance mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewContextProvenanceMismatch {
        expected: Option<Vec<String>>,
        actual: Option<Vec<String>>,
    },
    #[error("status view context staleness reason mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewContextStalenessReasonMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view flow mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewFlowMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view flow state mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewFlowStateMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view workflow mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewWorkflowMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view workflow phase mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewWorkflowPhaseMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view workflow next action mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewWorkflowNextActionMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view delegation mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewDelegationMismatch {
        expected: Option<Box<DelegationStatusView>>,
        actual: Option<Box<DelegationStatusView>>,
    },
    #[error("status view stage mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStageMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view stage index mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStageIndexMismatch { expected: Option<usize>, actual: Option<usize> },
    #[error("status view total stages mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStageCountMismatch { expected: Option<usize>, actual: Option<usize> },
    #[error("status view trace mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewTraceMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view latest decision status mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewDecisionStatusMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view latest decision target mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewDecisionTargetMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view changed files mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewChangedFilesMismatch { expected: Option<Vec<String>>, actual: Option<Vec<String>> },
    #[error(
        "status view authored input deduplicated sources mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewAuthoredInputDeduplicatedSourcesMismatch {
        expected: Option<Vec<String>>,
        actual: Option<Vec<String>>,
    },
    #[error("status view clarification headline mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewClarificationHeadlineMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view clarification prompt mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewClarificationPromptMismatch { expected: Option<String>, actual: Option<String> },
    #[error(
        "status view clarification missing fields mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewClarificationMissingFieldsMismatch {
        expected: Option<Vec<String>>,
        actual: Option<Vec<String>>,
    },
    #[error("status view workspace slice mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewWorkspaceSliceMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view selection headline mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewSelectionHeadlineMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view attempt lineage mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewAttemptLineageMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view validation status mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewValidationStatusMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review trigger mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewTriggerMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review vote mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewVoteMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review outcome mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewOutcomeMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review headline mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewHeadlineMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance stage mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceStageMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance runtime mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceRuntimeMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance mode mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceModeMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance run ref mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceRunRefMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance state mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceStateMismatch { expected: Option<String>, actual: Option<String> },
    #[error(
        "status view governance blocked reason mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewGovernanceBlockedReasonMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance packet ref mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernancePacketRefMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance packet source mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernancePacketSourceMismatch { expected: Option<String>, actual: Option<String> },
    #[error(
        "status view governance packet binding mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewGovernancePacketBindingMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance approval mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceApprovalMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance decision mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceDecisionMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view governance candidates mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceCandidatesMismatch {
        expected: Option<Vec<String>>,
        actual: Option<Vec<String>>,
    },
    #[error("status view governance next action mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGovernanceNextActionMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view project-scale path mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewProjectScalePathMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view project-scale stage mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewProjectScaleStageMismatch { expected: Option<String>, actual: Option<String> },
    #[error(
        "status view project-scale next action mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewProjectScaleNextActionMismatch { expected: Option<String>, actual: Option<String> },
    #[error(
        "status view project-scale checkpoint refs mismatch: expected {expected:?}, got {actual:?}"
    )]
    StatusViewProjectScaleCheckpointRefsMismatch {
        expected: Option<Vec<String>>,
        actual: Option<Vec<String>>,
    },
    #[error("status view voting trigger mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewVotingTriggerMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view voting result mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewVotingResultMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view voting adjudication mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewVotingAdjudicationMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view voting evidence mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewVotingEvidenceMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view voting blocking mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewVotingBlockingMismatch { expected: Option<bool>, actual: Option<bool> },
    #[error("status view voting next action mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewVotingNextActionMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view explanation must not be empty")]
    MissingStatusExplanation,
    #[error("status view governance_next_action must not be empty when present")]
    MissingGovernanceNextAction,
    #[error("status view next_command must not be empty when present")]
    MissingNextCommand,
    #[error("status view step index mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStepIndexMismatch { expected: Option<usize>, actual: Option<usize> },
    #[error("status view step id mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStepIdMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view plan revision mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewPlanRevisionMismatch { expected: Option<usize>, actual: Option<usize> },
}

fn status_requires_goal(status: SessionStatus) -> bool {
    !matches!(status, SessionStatus::Initialized | SessionStatus::Invalid)
}

fn status_requires_task(status: SessionStatus) -> bool {
    matches!(
        status,
        SessionStatus::Planned
            | SessionStatus::Running
            | SessionStatus::Succeeded
            | SessionStatus::Failed
            | SessionStatus::Exhausted
            | SessionStatus::Aborted
    )
}

fn status_allows_goal_plan_without_task(
    status: SessionStatus,
    goal_plan: Option<&GoalPlan>,
) -> bool {
    matches!(
        status,
        SessionStatus::Planned
            | SessionStatus::Running
            | SessionStatus::Succeeded
            | SessionStatus::Failed
            | SessionStatus::Exhausted
            | SessionStatus::Aborted
    ) && goal_plan.is_some()
}

fn expected_task_status(status: SessionStatus) -> Option<TaskStatus> {
    match status {
        SessionStatus::Planned => Some(TaskStatus::Planned),
        SessionStatus::Running => Some(TaskStatus::Running),
        SessionStatus::Succeeded => Some(TaskStatus::Succeeded),
        SessionStatus::Failed => Some(TaskStatus::Failed),
        SessionStatus::Exhausted => Some(TaskStatus::Exhausted),
        SessionStatus::Aborted => Some(TaskStatus::Aborted),
        SessionStatus::Initialized | SessionStatus::GoalCaptured | SessionStatus::Invalid => None,
    }
}

fn trace_within_workspace(workspace_ref: &str, trace_ref: &str) -> bool {
    let trace_path = Path::new(trace_ref);
    if trace_path.is_absolute() {
        trace_path.starts_with(Path::new(workspace_ref))
    } else {
        !trace_path.starts_with("..")
    }
}

fn trace_within_session_scope(record: &ActiveSessionRecord, trace_ref: &str) -> bool {
    if trace_within_workspace(&record.workspace_ref, trace_ref) {
        return true;
    }

    record.active_task.as_ref().is_some_and(|task| {
        task.context.cluster_session_projection().ok().flatten().is_some_and(|projection| {
            projection
                .member_workspace_refs
                .iter()
                .any(|workspace_ref| trace_within_workspace(workspace_ref, trace_ref))
        })
    })
}

/// Reads a string field from persisted task context state.
pub fn task_state_string(task: &Task, key: &str) -> Option<String> {
    task.context.state.get(key).and_then(|value| value.as_str().map(str::to_string))
}

fn task_state_json<T: DeserializeOwned>(task: &Task, key: &str) -> Option<T> {
    task.context.state.get(key).cloned().and_then(|value| serde_json::from_value(value).ok())
}

/// Reads a string array field from persisted task context state.
pub fn task_state_strings(task: &Task, key: &str) -> Option<Vec<String>> {
    task.context.state.get(key).and_then(|value| {
        value.as_array().map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
    })
}

/// Returns the latest governed stage record from task context state.
pub fn task_state_governed_stage(task: &Task) -> Option<GovernedStageRecord> {
    task_state_json(task, LATEST_GOVERNANCE_STAGE_KEY)
}

/// Returns the latest governed packet from task context state.
pub fn task_state_governed_packet(task: &Task) -> Option<GovernedStagePacket> {
    task_state_json(task, LATEST_GOVERNANCE_PACKET_KEY)
}

/// Returns the latest governance packet reuse binding from task context state.
pub fn task_state_governance_packet_reuse(task: &Task) -> Option<PacketReuseBinding> {
    task_state_json(task, LATEST_GOVERNANCE_PACKET_REUSE_KEY)
}

/// Returns the latest governance autopilot decision from task context state.
pub fn task_state_governance_decision(task: &Task) -> Option<AutopilotDecisionRecord> {
    task_state_json(task, LATEST_GOVERNANCE_DECISION_KEY)
}

fn encoded_text<T: Serialize>(value: &T) -> Option<String> {
    serde_json::to_value(value).ok().and_then(|value| value.as_str().map(str::to_string))
}

fn autopilot_action_text(action: crate::domain::governance::AutopilotAction) -> &'static str {
    match action {
        crate::domain::governance::AutopilotAction::SelectMode => "select_mode",
        crate::domain::governance::AutopilotAction::RetryStageWithNarrowedContext => {
            "retry_stage_with_narrowed_context"
        }
        crate::domain::governance::AutopilotAction::EscalateVerification => "escalate_verification",
        crate::domain::governance::AutopilotAction::EscalatePrReview => "escalate_pr_review",
        crate::domain::governance::AutopilotAction::AwaitApproval => "await_approval",
        crate::domain::governance::AutopilotAction::BlockStage => "block_stage",
    }
}

/// Returns the latest governance stage key from task context state.
pub fn task_state_governance_stage_key(task: &Task) -> Option<String> {
    task_state_governed_stage(task).map(|record| record.stage_key)
}

/// Returns the latest governance runtime label from task context state.
pub fn task_state_governance_runtime_text(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| encoded_text(&record.runtime))
}

/// Returns the latest governance mode label from task context state.
pub fn task_state_governance_mode_text(task: &Task) -> Option<String> {
    task_state_governed_packet(task)
        .and_then(|packet| packet.canon_mode)
        .and_then(|mode| encoded_text(&mode))
}

/// Returns the latest Canon governance run ref from task context state.
pub fn task_state_governance_canon_run_ref(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| record.canon_run_ref)
}

/// Returns the latest governance lifecycle-state label from task context state.
pub fn task_state_governance_state_text(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| encoded_text(&record.lifecycle_state))
}

/// Returns the latest runtime-owned governance posture label from task context state.
pub fn task_state_governance_runtime_state_text(task: &Task) -> Option<String> {
    task_state_string(task, LATEST_GOVERNANCE_RUNTIME_STATE_KEY)
}

/// Returns the latest operator-visible rollout profile from task context state.
pub fn task_state_governance_rollout_profile_text(task: &Task) -> Option<String> {
    task_state_string(task, LATEST_GOVERNANCE_ROLLOUT_PROFILE_KEY)
}

/// Returns the latest governance posture rationale from task context state.
pub fn task_state_governance_reason(task: &Task) -> Option<String> {
    task_state_string(task, LATEST_GOVERNANCE_REASON_KEY)
}

/// Returns the latest contract-line projection for governance posture.
pub fn task_state_governance_contract_lines(task: &Task) -> Option<Vec<String>> {
    task_state_strings(task, LATEST_GOVERNANCE_CONTRACT_LINES_KEY)
}

/// Returns the latest approval provenance for the current governance posture.
pub fn task_state_governance_approval_provenance(task: &Task) -> Option<String> {
    task_state_string(task, LATEST_GOVERNANCE_APPROVAL_PROVENANCE_KEY)
}

/// Returns the latest governance blocked reason, falling back to Canon-memory
/// staleness when that is the authoritative blocker.
pub fn task_state_governance_blocked_reason(task: &Task) -> Option<String> {
    task_state_governed_stage(task)
        .and_then(|record| record.blocked_reason)
        .or_else(|| task_state_canon_memory_staleness_reason(task))
}

/// Returns the latest governance packet ref from task context state.
pub fn task_state_governance_packet_ref(task: &Task) -> Option<String> {
    task_state_governed_packet(task)
        .map(|packet| packet.packet_ref)
        .or_else(|| task_state_governed_stage(task).and_then(|record| record.packet_ref))
}

/// Returns the source stage recorded for governance packet reuse.
pub fn task_state_governance_packet_source_stage(task: &Task) -> Option<String> {
    task_state_governance_packet_reuse(task).map(|binding| binding.upstream_stage_key)
}

/// Returns the packet-binding reason recorded for governance packet reuse.
pub fn task_state_governance_packet_binding_reason(task: &Task) -> Option<String> {
    task_state_governance_packet_reuse(task)
        .map(|binding| binding.binding_reason.as_str().to_string())
}

/// Formats governance packet provenance from optional source-stage and binding-reason fields.
pub fn governance_packet_provenance_text(
    packet_source_stage: Option<&str>,
    packet_binding_reason: Option<&str>,
) -> Option<String> {
    let packet_source_stage = packet_source_stage.map(str::trim).filter(|value| !value.is_empty());
    let packet_binding_reason =
        packet_binding_reason.map(str::trim).filter(|value| !value.is_empty());

    match (packet_source_stage, packet_binding_reason) {
        (Some(packet_source_stage), Some(packet_binding_reason)) => {
            Some(format!("{packet_source_stage} ({packet_binding_reason})"))
        }
        (Some(packet_source_stage), None) => Some(packet_source_stage.to_string()),
        (None, Some(packet_binding_reason)) => Some(packet_binding_reason.to_string()),
        (None, None) => None,
    }
}

/// Returns the latest governance approval label from task context state.
pub fn task_state_governance_approval_text(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| encoded_text(&record.approval_state))
}

/// Returns the latest governance decision headline from task context state.
pub fn task_state_governance_decision_headline(task: &Task) -> Option<String> {
    task_state_governance_decision(task).map(|decision| decision.rationale)
}

/// Returns the candidate governance actions recorded in task context state.
pub fn task_state_governance_candidate_actions(task: &Task) -> Option<Vec<String>> {
    task_state_governance_decision(task)
        .map(|decision| {
            decision
                .candidate_actions
                .into_iter()
                .map(|action| autopilot_action_text(action).to_string())
                .collect::<Vec<_>>()
        })
        .or_else(|| {
            task_state_compacted_canon_memory(task).and_then(|memory| {
                let actions = memory
                    .possible_actions
                    .into_iter()
                    .map(|action| action.action)
                    .collect::<Vec<_>>();
                (!actions.is_empty()).then_some(actions)
            })
        })
}

/// Returns the generic next governance action for a known governance lifecycle state.
pub fn governance_next_action_for_state(governance_state: Option<&str>) -> Option<String> {
    match governance_state {
        Some("awaiting_approval") => {
            Some("wait for approval and rerun boundline status".to_string())
        }
        Some("blocked") => {
            Some("resolve the governance blocker, then rerun boundline step".to_string())
        }
        _ => None,
    }
}

/// Returns the most authoritative next governance action recorded for the task.
pub fn task_state_governance_next_action(task: &Task) -> Option<String> {
    if let Some(memory) = task_state_compacted_canon_memory(task)
        && let Some(next_action) = memory.next_action_text()
    {
        return Some(next_action);
    }

    let governance_state = task_state_governance_state_text(task);
    governance_next_action_for_state(governance_state.as_deref())
}

/// Returns the latest compacted Canon memory snapshot from task context state.
pub fn task_state_compacted_canon_memory(task: &Task) -> Option<CompactedCanonMemory> {
    task.context.latest_compacted_canon_memory().ok().flatten()
}

/// Returns the Canon-memory-backed context summary from task context state.
pub fn task_state_canon_memory_context_summary(task: &Task) -> Option<String> {
    task_state_compacted_canon_memory(task)
        .map(|memory| format!("canon memory: {}", memory.summary_text()))
}

/// Returns the Canon-memory-backed context credibility label from task context state.
pub fn task_state_canon_memory_context_credibility(task: &Task) -> Option<String> {
    task_state_compacted_canon_memory(task).map(|memory| memory.credibility.as_str().to_string())
}

/// Returns the Canon-memory-backed primary inputs from task context state.
pub fn task_state_canon_memory_primary_inputs(task: &Task) -> Option<Vec<String>> {
    task_state_compacted_canon_memory(task)
        .and_then(|memory| (!memory.artifact_refs.is_empty()).then_some(memory.artifact_refs))
}

/// Returns the Canon-memory-backed provenance lines from task context state.
pub fn task_state_canon_memory_provenance(task: &Task) -> Option<Vec<String>> {
    task_state_compacted_canon_memory(task).map(|memory| memory.provenance_lines())
}

/// Returns the Canon-memory-backed staleness reason from task context state.
pub fn task_state_canon_memory_staleness_reason(task: &Task) -> Option<String> {
    task_state_compacted_canon_memory(task).and_then(|memory| {
        (memory.credibility != crate::domain::governance::MemoryCredibilityState::Credible)
            .then(|| memory.reason_code.unwrap_or(memory.headline))
    })
}

/// Returns a compact summary of the latest adaptive workspace slice.
pub fn task_state_workspace_slice_summary(task: &Task) -> Option<String> {
    let slice = task.context.state.get("latest_workspace_slice")?;
    let selected_targets = slice.get("selected_targets")?.as_array()?;
    let targets = selected_targets.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();

    if targets.is_empty() { None } else { Some(targets.join(", ")) }
}

/// Returns a compact summary of the latest adaptive attempt lineage.
pub fn task_state_attempt_lineage_summary(task: &Task) -> Option<String> {
    let lineage = task.context.state.get("latest_attempt_lineage")?;
    let current = lineage.get("current_attempt_id")?.as_str()?;
    let transition = lineage.get("transition_kind")?.as_str()?;
    let previous = lineage.get("previous_attempt_id").and_then(Value::as_str);

    previous.map_or_else(
        || Some(format!("{current} ({transition})")),
        |previous| Some(format!("{current} {transition} {previous}")),
    )
}

fn task_state_review_headline(task: &Task) -> Option<String> {
    let latest_finding = task
        .context
        .state
        .get("latest_review_findings")
        .and_then(Value::as_array)
        .and_then(|findings| findings.last());
    if let Some(finding) = latest_finding {
        let reviewer_id = finding.get("reviewer_id").and_then(Value::as_str).unwrap_or("reviewer");
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        return Some(format!("{reviewer_id} {disposition}: {summary}"));
    }

    let participants = task
        .context
        .state
        .get("latest_review_participants")
        .and_then(Value::as_array)
        .map(|participants| {
            participants
                .iter()
                .filter_map(|participant| {
                    let reviewer_id = participant.get("reviewer_id").and_then(Value::as_str)?;
                    let status =
                        participant.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    Some(format!("{reviewer_id} {status}"))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if participants.is_empty() {
        None
    } else {
        Some(format!("participants: {}", participants.join(", ")))
    }
}

impl From<TaskPersistenceError> for SessionValidationError {
    fn from(value: TaskPersistenceError) -> Self {
        Self::InvalidTask(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ActiveSessionRecord, CompatibilityFollowUpMode, DelegationStatusView,
        ProjectScaleSessionState, RoutingMode, RoutingSource, SessionStatus, SessionStatusView,
        SessionValidationError, VotingSessionState, delegation_next_command, execution_path_text,
        routing_outcome, task_state_attempt_lineage_summary, task_state_review_headline,
        task_state_string, task_state_strings, task_state_workspace_slice_summary,
        trace_within_workspace,
    };
    use crate::domain::goal_plan::{
        ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan,
        InferredFlow, PlannedTask,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::negotiation::NegotiatedDeliveryPacket;
    use crate::domain::plan::Plan;
    use crate::domain::session::{
        ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState, DelegationPacket,
        DelegationPacketKind, DelegationPacketState, StuckEvidenceMarker, StuckRecoveryAction,
        delegation_status_view,
    };
    use crate::domain::step::Step;
    use crate::domain::task::{Task, TaskPersistenceError, TaskRunRequest};
    use crate::domain::workflow::{
        ProjectScalePath, ProjectScalePathKind, ProjectScaleStage, ProjectScaleStageKind,
        WorkflowLifecycleState, WorkflowPhase, WorkflowProgressState,
    };

    fn build_task(workspace_ref: &str) -> Task {
        let request = TaskRunRequest {
            goal: "Deliver a session-backed CLI".to_string(),
            input: json!({"ticket": "SESSION-TEST"}),
            session_id: "session-1".to_string(),
            workspace_ref: workspace_ref.to_string(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let plan = Plan::new(vec![Step::decision("analyze", json!({})).unwrap()]).unwrap();
        Task::new("task-1", &request, plan).unwrap()
    }

    fn build_record(workspace_ref: &str) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: "session-1".to_string(),
            workspace_ref: workspace_ref.to_string(),
            goal: Some("Deliver a session-backed CLI".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: Some(
                crate::domain::flow::built_in_flow("bug-fix").unwrap().initial_state(),
            ),
            active_task: Some(build_task(workspace_ref)),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: Some(format!("{workspace_ref}/.boundline/traces/task-1.json")),
            created_at: 10,
            updated_at: 20,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        }
    }

    fn build_view(record: &ActiveSessionRecord) -> SessionStatusView {
        SessionStatusView {
            session_id: record.session_id.clone(),
            workspace_ref: record.workspace_ref.clone(),
            goal: record.goal.clone(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            authored_input_summary: None,
            authored_input_sources: None,
            authored_input_deduplicated_sources: None,
            context_summary: record
                .goal_plan
                .as_ref()
                .and_then(|goal_plan| goal_plan.context_summary()),
            context_credibility: record
                .goal_plan
                .as_ref()
                .and_then(|goal_plan| goal_plan.context_credibility()),
            context_primary_inputs: record.goal_plan.as_ref().and_then(|goal_plan| {
                let inputs = goal_plan.context_primary_inputs();
                (!inputs.is_empty()).then_some(inputs)
            }),
            context_provenance: record.goal_plan.as_ref().and_then(|goal_plan| {
                let lines = goal_plan.context_provenance_lines();
                (!lines.is_empty()).then_some(lines)
            }),
            context_staleness_reason: record
                .goal_plan
                .as_ref()
                .and_then(|goal_plan| goal_plan.context_pack.as_ref())
                .and_then(|pack| pack.staleness_reason.clone()),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: None,
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
            flow_state: record
                .goal_plan
                .as_ref()
                .map(|goal_plan| goal_plan.flow_state().summary_text()),
            goal_plan_state: record
                .goal_plan
                .as_ref()
                .map(|goal_plan| goal_plan.proposal_state_text().to_string()),
            goal_plan_revision: record
                .goal_plan
                .as_ref()
                .map(|goal_plan| goal_plan.proposal_revision),
            planning_rationale: record
                .goal_plan
                .as_ref()
                .and_then(|goal_plan| goal_plan.planning_rationale.clone()),
            verification_strategy: record
                .goal_plan
                .as_ref()
                .and_then(|goal_plan| goal_plan.verification_strategy.clone()),
            active_workflow: record.active_workflow_name(),
            workflow_phase: record.active_workflow_phase_text(),
            workflow_next_action: record.active_workflow_next_action(),
            continuity_authority: None,
            delegation: delegation_status_view(record),
            compatibility_follow_up: None,
            current_stage_id: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
            current_stage_index: record.active_flow.as_ref().map(|flow| flow.current_stage_index),
            total_stages: record.active_flow.as_ref().map(|flow| flow.total_stages),
            plan_revision: record.active_task.as_ref().map(|task| task.plan.revision),
            current_step_id: record
                .active_task
                .as_ref()
                .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
            current_step_index: record
                .active_task
                .as_ref()
                .map(|task| task.plan.current_step_index),
            latest_status: record.latest_status,
            execution_path: execution_path_text(record),
            latest_trace_ref: record.latest_trace_ref.clone(),
            latest_decision_status: record
                .decisions
                .last()
                .map(|decision| super::decision_status_text(decision.status).to_string()),
            latest_decision_target: record.decisions.last().map(|decision| decision.target.clone()),
            latest_changed_files: None,
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            latest_workspace_slice: None,
            latest_selection_headline: None,
            latest_candidate_family: None,
            latest_selection_reason: None,
            latest_rejected_candidates: None,
            latest_attempt_lineage: None,
            latest_validation_status: None,
            latest_exhaustion_reason: None,
            latest_review_trigger: None,
            latest_review_vote: None,
            latest_review_outcome: None,
            latest_review_council_profile: None,
            latest_review_independence_state: None,
            latest_review_stop_semantics: None,
            latest_review_selection_summary: None,
            latest_review_headline: None,
            latest_governance_stage: None,
            latest_governance_runtime: None,
            latest_governance_mode: None,
            latest_governance_run_ref: None,
            latest_governance_state: None,
            latest_governance_runtime_state: None,
            latest_governance_rollout_profile: None,
            latest_governance_reason: None,
            latest_governance_contract_lines: None,
            latest_governance_approval_provenance: None,
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: None,
            latest_governance_packet_source_stage: None,
            latest_governance_packet_binding_reason: None,
            latest_governance_approval: None,
            latest_governance_decision: None,
            latest_governance_candidates: None,
            governance_next_action: None,
            governance_lifecycle_runtime: None,
            governance_lifecycle_opt_out: None,
            governance_lifecycle_mode_selection: None,
            governance_lifecycle_selected_mode: None,
            project_scale_path: None,
            project_scale_current_stage: None,
            project_scale_next_action: None,
            project_scale_checkpoint_refs: None,
            latest_voting_trigger: None,
            latest_voting_result: None,
            latest_voting_adjudication: None,
            latest_voting_reviewed_evidence: None,
            latest_voting_blocking: None,
            latest_voting_next_action: None,
            next_command: Some("boundline step".to_string()),
            explanation: "view is consistent".to_string(),
        }
    }

    fn build_derived_state_record(workspace_ref: &str) -> ActiveSessionRecord {
        let mut record = build_record(workspace_ref);
        let task = record.active_task.as_mut().unwrap();
        task.context.state.insert("latest_changed_files".to_string(), json!(["src/lib.rs"]));
        task.context.state.insert(
            "latest_workspace_slice".to_string(),
            json!({"selected_targets": ["src/lib.rs", "tests/red_to_green.rs"]}),
        );
        task.context
            .state
            .insert("latest_selection_headline".to_string(), json!("selected src/lib.rs"));
        task.context.state.insert(
            "latest_attempt_lineage".to_string(),
            json!({
                "previous_attempt_id": "attempt-1",
                "current_attempt_id": "attempt-2",
                "transition_kind": "retried_from",
            }),
        );
        task.context.state.insert("latest_validation_status".to_string(), json!("passed"));
        task.context.state.insert("latest_review_trigger".to_string(), json!("pr_ready"));
        task.context.state.insert("latest_review_vote".to_string(), json!("accepted"));
        task.context.state.insert("latest_review_outcome".to_string(), json!("accepted"));
        task.context.state.insert(
            "latest_review_findings".to_string(),
            json!([{
                "reviewer_id": "safety",
                "disposition": "approve",
                "summary": "No blockers"
            }]),
        );
        task.context
            .set_latest_governance_stage(&crate::domain::governance::GovernedStageRecord {
                stage_key: "bug-fix:investigate".to_string(),
                runtime: crate::domain::governance::GovernanceRuntimeKind::Canon,
                lifecycle_state:
                    crate::domain::governance::GovernanceLifecycleState::AwaitingApproval,
                required: true,
                autopilot_enabled: true,
                approval_state: crate::domain::governance::ApprovalState::Requested,
                canon_run_ref: Some("canon-run-1".to_string()),
                governance_attempt_id: "attempt-governance-1".to_string(),
                previous_governance_attempt_id: None,
                packet_ref: Some(".canon/runs/canon-run-1".to_string()),
                decision_ref: Some("decision-1".to_string()),
                blocked_reason: None,
            })
            .unwrap();
        task.context
            .set_latest_governance_packet(&crate::domain::governance::GovernedStagePacket {
                packet_ref: ".canon/runs/canon-run-1".to_string(),
                runtime: crate::domain::governance::GovernanceRuntimeKind::Canon,
                canon_mode: Some(crate::domain::governance::CanonMode::Discovery),
                expected_document_refs: vec![".canon/runs/canon-run-1/discovery.md".to_string()],
                document_refs: vec![".canon/runs/canon-run-1/discovery.md".to_string()],
                readiness: crate::domain::governance::PacketReadiness::Reusable,
                missing_sections: Vec::new(),
                headline: "governed discovery packet".to_string(),
                reason_code: None,
                authority_governance: None,
                adaptive_governance: None,
            })
            .unwrap();
        task.context
            .set_latest_governance_packet_reuse(&crate::domain::governance::PacketReuseBinding {
                upstream_stage_key: "bug-fix:investigate".to_string(),
                downstream_stage_key: "bug-fix:implement".to_string(),
                packet_ref: ".canon/runs/canon-run-1".to_string(),
                binding_reason:
                    crate::domain::governance::PacketReuseBindingReason::UpstreamStageContext,
            })
            .unwrap();
        task.context
            .set_latest_governance_decision(&crate::domain::governance::AutopilotDecisionRecord {
                decision_id: "decision-1".to_string(),
                stage_key: "bug-fix:investigate".to_string(),
                candidate_actions: vec![
                    crate::domain::governance::AutopilotAction::SelectMode,
                    crate::domain::governance::AutopilotAction::AwaitApproval,
                ],
                candidate_modes: vec![crate::domain::governance::CanonMode::Discovery],
                selected_action: Some(crate::domain::governance::AutopilotAction::SelectMode),
                selected_mode: Some(crate::domain::governance::CanonMode::Discovery),
                selected_target_stage_key: None,
                rationale: "autopilot selected Canon mode Discovery for bug-fix:investigate"
                    .to_string(),
                blocked_reason: None,
            })
            .unwrap();

        record
    }

    fn build_derived_view(record: &ActiveSessionRecord) -> SessionStatusView {
        let mut view = build_view(record);
        let task = record.active_task.as_ref().unwrap();
        view.latest_changed_files = task_state_strings(task, "latest_changed_files");
        view.latest_checkpoint_id = task_state_string(task, "latest_checkpoint_id");
        view.latest_checkpoint_scope = task_state_string(task, "latest_checkpoint_scope");
        view.latest_checkpoint_restore_command =
            task_state_string(task, "latest_checkpoint_restore_command");
        view.latest_workspace_slice = task_state_workspace_slice_summary(task);
        view.clarification_headline =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_headline());
        view.clarification_prompt =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_prompt());
        view.clarification_missing_fields =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_missing_fields());
        view.latest_selection_headline = task_state_string(task, "latest_selection_headline");
        view.latest_candidate_family = task_state_string(task, "latest_candidate_family");
        view.latest_selection_reason = task_state_string(task, "latest_selection_reason");
        view.latest_rejected_candidates = task_state_strings(task, "latest_rejected_candidates");
        view.latest_attempt_lineage = task_state_attempt_lineage_summary(task);
        view.latest_validation_status = task_state_string(task, "latest_validation_status");
        view.latest_exhaustion_reason = task_state_string(task, "latest_exhaustion_reason");
        view.latest_review_trigger = task_state_string(task, "latest_review_trigger");
        view.latest_review_vote = task_state_string(task, "latest_review_vote");
        view.latest_review_outcome = task_state_string(task, "latest_review_outcome");
        view.latest_review_headline = task_state_review_headline(task);
        view.latest_governance_stage = super::task_state_governance_stage_key(task);
        view.latest_governance_runtime = super::task_state_governance_runtime_text(task);
        view.latest_governance_mode = super::task_state_governance_mode_text(task);
        view.latest_governance_run_ref = super::task_state_governance_canon_run_ref(task);
        view.latest_governance_state = super::task_state_governance_state_text(task);
        view.latest_governance_runtime_state =
            super::task_state_governance_runtime_state_text(task);
        view.latest_governance_rollout_profile =
            super::task_state_governance_rollout_profile_text(task);
        view.latest_governance_reason = super::task_state_governance_reason(task);
        view.latest_governance_contract_lines = super::task_state_governance_contract_lines(task);
        view.latest_governance_approval_provenance =
            super::task_state_governance_approval_provenance(task);
        view.latest_governance_blocked_reason = super::task_state_governance_blocked_reason(task);
        view.latest_governance_packet_ref = super::task_state_governance_packet_ref(task);
        view.latest_governance_packet_source_stage =
            super::task_state_governance_packet_source_stage(task);
        view.latest_governance_packet_binding_reason =
            super::task_state_governance_packet_binding_reason(task);
        view.latest_governance_approval = super::task_state_governance_approval_text(task);
        view.latest_governance_decision = super::task_state_governance_decision_headline(task);
        view.latest_governance_candidates = super::task_state_governance_candidate_actions(task);
        view.governance_next_action = super::task_state_governance_next_action(task);
        view
    }

    fn build_context_goal_plan() -> GoalPlan {
        let mut goal_plan = GoalPlan::new(
            "Deliver a session-backed CLI",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Implement the session runtime".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("session runtime works".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_context_pack(ContextPack {
            pack_id: "cp-session".to_string(),
            summary: "bounded context from src/lib.rs".to_string(),
            credibility: ContextPackCredibility::Stale,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "contains the runtime entry point".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            staleness_reason: Some("trace snapshot is stale".to_string()),
        })
        .with_flow(InferredFlow {
            flow_name: "bug-fix".to_string(),
            confidence_reason: "goal contains fix evidence".to_string(),
            confirmed: false,
        });
        goal_plan.confirm().unwrap();
        goal_plan
    }

    #[test]
    fn status_view_rejects_stage_count_trace_and_step_index_mismatches() {
        let workspace = "/tmp/boundline-session-domain";
        let record = build_record(workspace);

        let mut wrong_stage_index = build_view(&record);
        wrong_stage_index.current_stage_index = Some(1);
        assert!(matches!(
            wrong_stage_index.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewStageIndexMismatch { .. }
        ));

        let mut wrong_stage_count = build_view(&record);
        wrong_stage_count.total_stages = Some(99);
        assert!(matches!(
            wrong_stage_count.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewStageCountMismatch { .. }
        ));
        let mut wrong_trace = build_view(&record);
        wrong_trace.latest_trace_ref = Some("/tmp/other/trace.json".to_string());
        assert!(matches!(
            wrong_trace.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewTraceMismatch { .. }
        ));

        let mut wrong_step_index = build_view(&record);
        wrong_step_index.current_step_index = Some(99);
        assert!(matches!(
            wrong_step_index.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewStepIndexMismatch { .. }
        ));
    }

    #[test]
    fn governance_packet_provenance_text_formats_source_and_binding_reason() {
        assert_eq!(
            super::governance_packet_provenance_text(
                Some("bug-fix:investigate"),
                Some("upstream_stage_context")
            ),
            Some("bug-fix:investigate (upstream_stage_context)".to_string())
        );
        assert_eq!(
            super::governance_packet_provenance_text(Some("bug-fix:investigate"), None),
            Some("bug-fix:investigate".to_string())
        );
        assert_eq!(
            super::governance_packet_provenance_text(None, Some("same_stage_rerun")),
            Some("same_stage_rerun".to_string())
        );
        assert_eq!(super::governance_packet_provenance_text(None, None), None);
    }

    #[test]
    fn task_state_canon_memory_provenance_includes_adaptive_lines() {
        let mut task = build_task("/tmp/workspace");
        task.context
            .set_latest_compacted_canon_memory(&crate::domain::governance::CompactedCanonMemory {
                headline: "Canon verification packet".to_string(),
                credibility: crate::domain::governance::MemoryCredibilityState::Stale,
                stage_key: Some("change:verify".to_string()),
                run_ref: Some("run-10".to_string()),
                packet_ref: Some(".canon/runs/run-10".to_string()),
                reason_code: Some("refresh_required".to_string()),
                artifact_refs: vec![".canon/runs/run-10/verification.md".to_string()],
                mode_summary: None,
                possible_actions: Vec::new(),
                recommended_next_action: None,
                evidence_summary: None,
                authority_provenance_lines: vec![
                    "authority_contract_line: authority-governance-v1".to_string(),
                ],
                adaptive_provenance_lines: vec![
                    "adaptive_contract_line: adaptive-governance-v1".to_string(),
                ],
            })
            .unwrap();

        let provenance = super::task_state_canon_memory_provenance(&task).unwrap();

        assert!(
            provenance
                .iter()
                .any(|line| line == "authority_contract_line: authority-governance-v1")
        );
        assert!(
            provenance.iter().any(|line| line == "adaptive_contract_line: adaptive-governance-v1")
        );
    }

    #[test]
    fn delegation_packet_validation_rejects_invalid_supersession_and_empty_evidence() {
        let packet = DelegationPacket {
            packet_id: "packet-1".to_string(),
            kind: DelegationPacketKind::Handoff,
            state: DelegationPacketState::Superseded,
            created_at: 10,
            resolved_at: None,
            source_route_owner: "native".to_string(),
            target_owner: "codex".to_string(),
            continuity_reason: String::new(),
            recommended_next_action: "boundline status".to_string(),
            evidence_refs: Vec::new(),
            capability_summary: None,
            stuck_marker: None,
            superseded_by_packet_id: None,
        };

        let error = packet.validate().unwrap_err();
        assert!(error.contains("decisive evidence") || error.contains("successor packet"));
    }

    #[test]
    fn delegation_continuity_validation_requires_active_packet_for_handoff() {
        let continuity = DelegationContinuityState {
            active_packet_id: None,
            mode: DelegationContinuityMode::HandoffRequired,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: "boundline status".to_string(),
            headline: "handoff required: implementation route cannot continue".to_string(),
            evidence_summary: "routing policy requires a handoff".to_string(),
        };

        let error = continuity.validate(&[]).unwrap_err();
        assert!(error.contains("requires an active_packet_id"));
    }

    #[test]
    fn helper_functions_cover_relative_trace_paths_and_state_extractors() {
        assert!(trace_within_workspace("/tmp/workspace", "trace.json"));
        assert!(!trace_within_workspace("/tmp/workspace", "../outside.json"));

        let mut task = build_task("/tmp/workspace");
        task.context.state.insert("latest_validation_status".to_string(), json!("passed"));
        task.context.state.insert("latest_changed_files".to_string(), json!(["src/lib.rs"]));
        task.context.state.insert(
            "latest_workspace_slice".to_string(),
            json!({"selected_targets": ["src/lib.rs"]}),
        );
        task.context.state.insert(
            "latest_selection_headline".to_string(),
            json!("selected src/lib.rs for adaptive delivery"),
        );
        task.context.state.insert(
            "latest_attempt_lineage".to_string(),
            json!({
                "previous_attempt_id": "adaptive-attempt-1",
                "current_attempt_id": "adaptive-attempt-2",
                "transition_kind": "replaced",
            }),
        );
        task.context.state.insert("latest_review_trigger".to_string(), json!("pr_ready"));
        task.context.state.insert(
            "latest_review_findings".to_string(),
            json!([{
                "reviewer_id": "safety",
                "disposition": "approve",
                "summary": "No blockers"
            }]),
        );

        assert_eq!(
            task_state_string(&task, "latest_validation_status"),
            Some("passed".to_string())
        );
        assert_eq!(
            task_state_strings(&task, "latest_changed_files"),
            Some(vec!["src/lib.rs".to_string()])
        );
        assert_eq!(task_state_workspace_slice_summary(&task), Some("src/lib.rs".to_string()));
        assert_eq!(
            task_state_attempt_lineage_summary(&task),
            Some("adaptive-attempt-2 replaced adaptive-attempt-1".to_string())
        );
        assert_eq!(task_state_string(&task, "latest_review_trigger"), Some("pr_ready".to_string()));
        assert_eq!(
            task_state_review_headline(&task),
            Some("safety approve: No blockers".to_string())
        );
    }

    #[test]
    fn status_view_rejects_derived_state_mismatches_and_blank_metadata() {
        let record = build_derived_state_record("/tmp/boundline-session-domain-derived");
        let view = build_derived_view(&record);

        macro_rules! assert_view_error {
            ($candidate:expr, $pattern:pat) => {{
                let error = $candidate.validate(&record).unwrap_err();
                assert!(matches!(error, $pattern), "unexpected error: {error:?}");
            }};
        }

        let mut wrong_changed_files = view.clone();
        wrong_changed_files.latest_changed_files = Some(vec!["src/other.rs".to_string()]);
        assert_view_error!(
            wrong_changed_files,
            SessionValidationError::StatusViewChangedFilesMismatch { .. }
        );

        let mut wrong_workspace_slice = view.clone();
        wrong_workspace_slice.latest_workspace_slice = Some("tests/red_to_green.rs".to_string());
        assert_view_error!(
            wrong_workspace_slice,
            SessionValidationError::StatusViewWorkspaceSliceMismatch { .. }
        );

        let mut wrong_selection_headline = view.clone();
        wrong_selection_headline.latest_selection_headline = Some("different headline".to_string());
        assert_view_error!(
            wrong_selection_headline,
            SessionValidationError::StatusViewSelectionHeadlineMismatch { .. }
        );

        let mut wrong_attempt_lineage = view.clone();
        wrong_attempt_lineage.latest_attempt_lineage =
            Some("attempt-3 retried_from attempt-2".to_string());
        assert_view_error!(
            wrong_attempt_lineage,
            SessionValidationError::StatusViewAttemptLineageMismatch { .. }
        );

        let mut wrong_validation_status = view.clone();
        wrong_validation_status.latest_validation_status = Some("failed".to_string());
        assert_view_error!(
            wrong_validation_status,
            SessionValidationError::StatusViewValidationStatusMismatch { .. }
        );

        let mut wrong_review_trigger = view.clone();
        wrong_review_trigger.latest_review_trigger = Some("manual".to_string());
        assert_view_error!(
            wrong_review_trigger,
            SessionValidationError::StatusViewReviewTriggerMismatch { .. }
        );

        let mut wrong_review_vote = view.clone();
        wrong_review_vote.latest_review_vote = Some("rejected".to_string());
        assert_view_error!(
            wrong_review_vote,
            SessionValidationError::StatusViewReviewVoteMismatch { .. }
        );

        let mut wrong_review_outcome = view.clone();
        wrong_review_outcome.latest_review_outcome = Some("blocked".to_string());
        assert_view_error!(
            wrong_review_outcome,
            SessionValidationError::StatusViewReviewOutcomeMismatch { .. }
        );

        let mut wrong_review_headline = view.clone();
        wrong_review_headline.latest_review_headline =
            Some("reviewer blocked: missing test".to_string());
        assert_view_error!(
            wrong_review_headline,
            SessionValidationError::StatusViewReviewHeadlineMismatch { .. }
        );

        let mut wrong_governance_stage = view.clone();
        wrong_governance_stage.latest_governance_stage = Some("bug-fix:implement".to_string());
        assert_view_error!(
            wrong_governance_stage,
            SessionValidationError::StatusViewGovernanceStageMismatch { .. }
        );

        let mut wrong_governance_runtime = view.clone();
        wrong_governance_runtime.latest_governance_runtime = Some("local".to_string());
        assert_view_error!(
            wrong_governance_runtime,
            SessionValidationError::StatusViewGovernanceRuntimeMismatch { .. }
        );

        let mut wrong_governance_mode = view.clone();
        wrong_governance_mode.latest_governance_mode = Some("implementation".to_string());
        assert_view_error!(
            wrong_governance_mode,
            SessionValidationError::StatusViewGovernanceModeMismatch { .. }
        );

        let mut wrong_governance_run_ref = view.clone();
        wrong_governance_run_ref.latest_governance_run_ref = Some("canon-run-2".to_string());
        assert_view_error!(
            wrong_governance_run_ref,
            SessionValidationError::StatusViewGovernanceRunRefMismatch { .. }
        );

        let mut wrong_governance_state = view.clone();
        wrong_governance_state.latest_governance_state = Some("blocked".to_string());
        assert_view_error!(
            wrong_governance_state,
            SessionValidationError::StatusViewGovernanceStateMismatch { .. }
        );

        let mut wrong_governance_blocked_reason = view.clone();
        wrong_governance_blocked_reason.latest_governance_blocked_reason =
            Some("unexpected blocked reason".to_string());
        assert_view_error!(
            wrong_governance_blocked_reason,
            SessionValidationError::StatusViewGovernanceBlockedReasonMismatch { .. }
        );

        let mut wrong_governance_packet_ref = view.clone();
        wrong_governance_packet_ref.latest_governance_packet_ref =
            Some(".canon/runs/canon-run-2".to_string());
        assert_view_error!(
            wrong_governance_packet_ref,
            SessionValidationError::StatusViewGovernancePacketRefMismatch { .. }
        );

        let mut wrong_governance_packet_source = view.clone();
        wrong_governance_packet_source.latest_governance_packet_source_stage =
            Some("bug-fix:verify".to_string());
        assert_view_error!(
            wrong_governance_packet_source,
            SessionValidationError::StatusViewGovernancePacketSourceMismatch { .. }
        );

        let mut wrong_governance_packet_binding = view.clone();
        wrong_governance_packet_binding.latest_governance_packet_binding_reason =
            Some("same_stage_rerun".to_string());
        assert_view_error!(
            wrong_governance_packet_binding,
            SessionValidationError::StatusViewGovernancePacketBindingMismatch { .. }
        );

        let mut wrong_governance_approval = view.clone();
        wrong_governance_approval.latest_governance_approval = Some("granted".to_string());
        assert_view_error!(
            wrong_governance_approval,
            SessionValidationError::StatusViewGovernanceApprovalMismatch { .. }
        );

        let mut wrong_governance_decision = view.clone();
        wrong_governance_decision.latest_governance_decision =
            Some("different decision".to_string());
        assert_view_error!(
            wrong_governance_decision,
            SessionValidationError::StatusViewGovernanceDecisionMismatch { .. }
        );

        let mut wrong_governance_candidates = view.clone();
        wrong_governance_candidates.latest_governance_candidates =
            Some(vec!["block_stage".to_string()]);
        assert_view_error!(
            wrong_governance_candidates,
            SessionValidationError::StatusViewGovernanceCandidatesMismatch { .. }
        );

        let mut missing_explanation = view.clone();
        missing_explanation.explanation = "  ".to_string();
        assert_view_error!(missing_explanation, SessionValidationError::MissingStatusExplanation);

        let mut missing_next_command = view.clone();
        missing_next_command.next_command = Some(" ".to_string());
        assert_view_error!(missing_next_command, SessionValidationError::MissingNextCommand);
    }

    #[test]
    fn task_persistence_errors_convert_to_session_validation_errors() {
        let error = SessionValidationError::from(TaskPersistenceError::MissingGoal);
        assert!(
            matches!(error, SessionValidationError::InvalidTask(message) if message.contains("task goal must not be empty"))
        );
    }

    #[test]
    fn active_session_validation_covers_goal_plan_and_workflow_branches() {
        let workspace = "/tmp/boundline-session-domain-goal-plan";

        let mut missing_goal = build_record(workspace);
        missing_goal.goal = None;
        assert!(matches!(
            missing_goal.validate().unwrap_err(),
            SessionValidationError::MissingGoal(SessionStatus::Planned)
        ));

        let mut planned_without_task = build_record(workspace);
        planned_without_task.active_task = None;
        planned_without_task.goal_plan = Some(build_context_goal_plan());
        planned_without_task.latest_status = SessionStatus::Planned;
        planned_without_task.validate().unwrap();

        let mut invalid_workflow = build_record(workspace);
        invalid_workflow.workflow_progress = Some(WorkflowProgressState {
            workflow_name: " ".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Run),
            completed_phases: Vec::new(),
            blocked_reason: None,
            next_action: Some("boundline step".to_string()),
            routing_summary: None,
        });
        assert!(matches!(
            invalid_workflow.validate().unwrap_err(),
            SessionValidationError::InvalidWorkflowProgress(_)
        ));

        assert_eq!(super::ContinuityAuthority::NativeSession.as_str(), "native_session");
        assert_eq!(super::ContinuityAuthority::CompatibilityTrace.as_str(), "compatibility_trace");
        assert_eq!(super::CompatibilityFollowUpMode::InspectOnly.as_str(), "inspect_only");
        assert_eq!(super::CompatibilityFollowUpMode::Superseded.as_str(), "superseded");
    }

    #[test]
    fn status_view_rejects_negotiation_and_context_projection_mismatches() {
        let workspace = "/tmp/boundline-session-domain-context";
        let mut record = build_record(workspace);
        record.negotiation_packet = Some(NegotiatedDeliveryPacket::from_goal(
            &record.session_id,
            &record.workspace_ref,
            record.goal.as_deref().unwrap(),
        ));
        record.goal_plan = Some(build_context_goal_plan());

        let mut view = build_view(&record);
        let packet = record.negotiation_packet.as_ref().unwrap();
        view.negotiation_goal_summary = Some(packet.goal_summary.clone());
        view.negotiation_resolution = Some(packet.resolution_state.as_str().to_string());
        view.negotiation_acceptance_boundary =
            Some(packet.acceptance_boundary.success_headline.clone());
        view.validate(&record).unwrap();

        macro_rules! assert_view_error {
            ($candidate:expr, $pattern:pat) => {{
                let error = $candidate.validate(&record).unwrap_err();
                assert!(matches!(error, $pattern), "unexpected error: {error:?}");
            }};
        }

        let mut wrong_negotiation_goal_summary = view.clone();
        wrong_negotiation_goal_summary.negotiation_goal_summary = Some("other goal".to_string());
        assert_view_error!(
            wrong_negotiation_goal_summary,
            SessionValidationError::StatusViewNegotiationGoalSummaryMismatch { .. }
        );

        let mut wrong_negotiation_resolution = view.clone();
        wrong_negotiation_resolution.negotiation_resolution = Some("blocked".to_string());
        assert_view_error!(
            wrong_negotiation_resolution,
            SessionValidationError::StatusViewNegotiationResolutionMismatch { .. }
        );

        let mut wrong_negotiation_boundary = view.clone();
        wrong_negotiation_boundary.negotiation_acceptance_boundary =
            Some("different boundary".to_string());
        assert_view_error!(
            wrong_negotiation_boundary,
            SessionValidationError::StatusViewNegotiationAcceptanceBoundaryMismatch { .. }
        );

        let mut wrong_context_summary = view.clone();
        wrong_context_summary.context_summary = Some("other context".to_string());
        assert_view_error!(
            wrong_context_summary,
            SessionValidationError::StatusViewContextSummaryMismatch { .. }
        );

        let mut wrong_context_credibility = view.clone();
        wrong_context_credibility.context_credibility = Some("credible".to_string());
        assert_view_error!(
            wrong_context_credibility,
            SessionValidationError::StatusViewContextCredibilityMismatch { .. }
        );

        let mut wrong_context_inputs = view.clone();
        wrong_context_inputs.context_primary_inputs = Some(vec!["README.md".to_string()]);
        assert_view_error!(
            wrong_context_inputs,
            SessionValidationError::StatusViewContextPrimaryInputsMismatch { .. }
        );

        let mut wrong_context_provenance = view.clone();
        wrong_context_provenance.context_provenance =
            Some(vec!["workspace_file: README.md (wrong)".to_string()]);
        assert_view_error!(
            wrong_context_provenance,
            SessionValidationError::StatusViewContextProvenanceMismatch { .. }
        );

        let mut wrong_context_staleness = view;
        wrong_context_staleness.context_staleness_reason = Some("different reason".to_string());
        assert_view_error!(
            wrong_context_staleness,
            SessionValidationError::StatusViewContextStalenessReasonMismatch { .. }
        );
    }

    #[test]
    fn status_view_rejects_project_scale_and_voting_projection_mismatches() {
        let workspace = "/tmp/boundline-session-domain-project-scale";
        let mut record = build_record(workspace);
        record.project_scale = Some(ProjectScaleSessionState {
            path: ProjectScalePath {
                kind: ProjectScalePathKind::IdeaToCode,
                goal: record.goal.clone().unwrap(),
                stages: vec![
                    ProjectScaleStage {
                        kind: ProjectScaleStageKind::Discovery,
                        reason: "problem framing is incomplete".to_string(),
                    },
                    ProjectScaleStage {
                        kind: ProjectScaleStageKind::Requirements,
                        reason: "product scope must be bounded".to_string(),
                    },
                ],
                requires_confirmation: true,
                next_action: "confirm_project_scale_path".to_string(),
                unbounded_autonomy: false,
            },
            active_stage_index: 0,
            active_work_unit_id: Some("stage-001-discovery".to_string()),
            checkpoint_refs: vec!["checkpoint-1".to_string()],
            trace_refs: Vec::new(),
            next_action: "repair_context".to_string(),
        });
        record.latest_voting = Some(VotingSessionState {
            trigger: "high_impact_architecture".to_string(),
            reviewed_evidence_ref: Some("govern:architecture".to_string()),
            result: "pending".to_string(),
            reviewer_findings: vec!["needs ADR".to_string()],
            adjudication_result: Some("escalate".to_string()),
            blocking: true,
            next_action: "resolve_voting_boundary".to_string(),
        });

        let mut view = build_view(&record);
        let project_scale = record.project_scale.as_ref().unwrap();
        let vote = record.latest_voting.as_ref().unwrap();
        view.project_scale_path = Some(project_scale.path.stage_names());
        view.project_scale_current_stage = project_scale.active_stage_text();
        view.project_scale_next_action = Some(project_scale.next_action.clone());
        view.project_scale_checkpoint_refs = Some(project_scale.checkpoint_refs.clone());
        view.latest_voting_trigger = Some(vote.trigger.clone());
        view.latest_voting_result = Some(vote.result.clone());
        view.latest_voting_adjudication = vote.adjudication_result.clone();
        view.latest_voting_reviewed_evidence = vote.reviewed_evidence_ref.clone();
        view.latest_voting_blocking = Some(vote.blocking);
        view.latest_voting_next_action = Some(vote.next_action.clone());
        view.validate(&record).unwrap();

        macro_rules! assert_view_error {
            ($candidate:expr, $pattern:pat) => {{
                let error = $candidate.validate(&record).unwrap_err();
                assert!(matches!(error, $pattern), "unexpected error: {error:?}");
            }};
        }

        let mut wrong_project_scale_path = view.clone();
        wrong_project_scale_path.project_scale_path =
            Some("discovery -> implementation".to_string());
        assert_view_error!(
            wrong_project_scale_path,
            SessionValidationError::StatusViewProjectScalePathMismatch { .. }
        );

        let mut wrong_project_scale_stage = view.clone();
        wrong_project_scale_stage.project_scale_current_stage = Some("requirements".to_string());
        assert_view_error!(
            wrong_project_scale_stage,
            SessionValidationError::StatusViewProjectScaleStageMismatch { .. }
        );

        let mut wrong_project_scale_next = view.clone();
        wrong_project_scale_next.project_scale_next_action =
            Some("confirm_project_scale_path".to_string());
        assert_view_error!(
            wrong_project_scale_next,
            SessionValidationError::StatusViewProjectScaleNextActionMismatch { .. }
        );

        let mut wrong_project_scale_checkpoints = view.clone();
        wrong_project_scale_checkpoints.project_scale_checkpoint_refs =
            Some(vec!["checkpoint-2".to_string()]);
        assert_view_error!(
            wrong_project_scale_checkpoints,
            SessionValidationError::StatusViewProjectScaleCheckpointRefsMismatch { .. }
        );

        let mut wrong_vote_trigger = view.clone();
        wrong_vote_trigger.latest_voting_trigger = Some("pr_ready".to_string());
        assert_view_error!(
            wrong_vote_trigger,
            SessionValidationError::StatusViewVotingTriggerMismatch { .. }
        );

        let mut wrong_vote_result = view.clone();
        wrong_vote_result.latest_voting_result = Some("approved".to_string());
        assert_view_error!(
            wrong_vote_result,
            SessionValidationError::StatusViewVotingResultMismatch { .. }
        );

        let mut wrong_vote_adjudication = view.clone();
        wrong_vote_adjudication.latest_voting_adjudication = Some("override".to_string());
        assert_view_error!(
            wrong_vote_adjudication,
            SessionValidationError::StatusViewVotingAdjudicationMismatch { .. }
        );

        let mut wrong_vote_evidence = view.clone();
        wrong_vote_evidence.latest_voting_reviewed_evidence = Some("govern:pr-review".to_string());
        assert_view_error!(
            wrong_vote_evidence,
            SessionValidationError::StatusViewVotingEvidenceMismatch { .. }
        );

        let mut wrong_vote_blocking = view.clone();
        wrong_vote_blocking.latest_voting_blocking = Some(false);
        assert_view_error!(
            wrong_vote_blocking,
            SessionValidationError::StatusViewVotingBlockingMismatch { .. }
        );

        let mut wrong_vote_next_action = view;
        wrong_vote_next_action.latest_voting_next_action =
            Some("continue_architecture_stage".to_string());
        assert_view_error!(
            wrong_vote_next_action,
            SessionValidationError::StatusViewVotingNextActionMismatch { .. }
        );
    }

    #[test]
    fn project_scale_active_stage_text_returns_none_when_index_is_out_of_bounds() {
        let state = ProjectScaleSessionState {
            path: ProjectScalePath {
                kind: ProjectScalePathKind::IdeaToCode,
                goal: "Build onboarding".to_string(),
                stages: vec![ProjectScaleStage {
                    kind: ProjectScaleStageKind::Discovery,
                    reason: "problem framing is incomplete".to_string(),
                }],
                requires_confirmation: true,
                next_action: "confirm_project_scale_path".to_string(),
                unbounded_autonomy: false,
            },
            active_stage_index: 99,
            active_work_unit_id: None,
            checkpoint_refs: Vec::new(),
            trace_refs: Vec::new(),
            next_action: "repair_context".to_string(),
        };

        assert_eq!(state.active_stage_text(), None);
    }

    #[test]
    fn status_view_rejects_flow_workflow_and_participant_projection_mismatches() {
        let workspace = "/tmp/boundline-session-domain-workflow";
        let mut record = build_record(workspace);
        record.goal_plan = Some(build_context_goal_plan());
        record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "developer-ux".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Review),
            completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            blocked_reason: None,
            next_action: Some("boundline review".to_string()),
            routing_summary: Some("routing: native (goal_plan)".to_string()),
        });

        let view = build_view(&record);

        macro_rules! assert_view_error {
            ($candidate:expr, $pattern:pat) => {{
                let error = $candidate.validate(&record).unwrap_err();
                assert!(matches!(error, $pattern), "unexpected error: {error:?}");
            }};
        }

        let mut wrong_active_flow = view.clone();
        wrong_active_flow.active_flow = Some("delivery".to_string());
        assert_view_error!(
            wrong_active_flow,
            SessionValidationError::StatusViewFlowMismatch { .. }
        );

        let mut wrong_flow_state = view.clone();
        wrong_flow_state.flow_state = Some("confirmed (delivery)".to_string());
        assert_view_error!(
            wrong_flow_state,
            SessionValidationError::StatusViewFlowStateMismatch { .. }
        );

        let mut wrong_workflow = view.clone();
        wrong_workflow.active_workflow = Some("release".to_string());
        assert_view_error!(
            wrong_workflow,
            SessionValidationError::StatusViewWorkflowMismatch { .. }
        );

        let mut wrong_workflow_phase = view.clone();
        wrong_workflow_phase.workflow_phase = Some("govern".to_string());
        assert_view_error!(
            wrong_workflow_phase,
            SessionValidationError::StatusViewWorkflowPhaseMismatch { .. }
        );

        let mut wrong_workflow_next_action = view;
        wrong_workflow_next_action.workflow_next_action = Some("boundline inspect".to_string());
        assert_view_error!(
            wrong_workflow_next_action,
            SessionValidationError::StatusViewWorkflowNextActionMismatch { .. }
        );

        let mut participant_task = build_task(workspace);
        participant_task.context.state.insert(
            "latest_review_participants".to_string(),
            json!([
                {"reviewer_id": "safety", "status": "pending"},
                {"reviewer_id": "maintainability"}
            ]),
        );
        assert_eq!(
            task_state_review_headline(&participant_task),
            Some("participants: safety pending, maintainability unknown".to_string())
        );
    }

    #[test]
    fn delegation_helpers_cover_headline_evidence_mutation_and_stuck_signals() {
        assert_eq!(StuckRecoveryAction::Replan.as_str(), "replan");
        assert_eq!(StuckRecoveryAction::ResolvePacket.as_str(), "resolve_packet");
        assert_eq!(StuckRecoveryAction::UpdateConfig.as_str(), "update_config");
        assert_eq!(StuckRecoveryAction::RerunValidation.as_str(), "rerun_validation");
        assert_eq!(StuckRecoveryAction::Escalate.as_str(), "escalate");

        let invalid_marker = StuckEvidenceMarker {
            repeated_attempts: 0,
            same_reason_count: 0,
            unchanged_workspace_signal: false,
            stale_route_policy: false,
            recommended_recovery: StuckRecoveryAction::Replan,
        };
        assert!(
            invalid_marker
                .validate()
                .unwrap_err()
                .contains("at least one repeated or unchanged signal")
        );

        let mut packet = DelegationPacket {
            packet_id: "packet-1".to_string(),
            kind: DelegationPacketKind::Handoff,
            state: DelegationPacketState::Active,
            created_at: 10,
            resolved_at: None,
            source_route_owner: "native".to_string(),
            target_owner: "codex".to_string(),
            continuity_reason: String::new(),
            recommended_next_action: "boundline status".to_string(),
            evidence_refs: vec!["trace:delegation:packet-1".to_string()],
            capability_summary: Some("continuation=unsupported".to_string()),
            stuck_marker: None,
            superseded_by_packet_id: None,
        };
        assert_eq!(packet.headline(), "handoff required: trace:delegation:packet-1".to_string());
        assert_eq!(packet.evidence_summary(), "trace:delegation:packet-1".to_string());

        packet.mark_superseded("packet-2");
        assert_eq!(packet.state, DelegationPacketState::Superseded);
        assert_eq!(packet.superseded_by_packet_id.as_deref(), Some("packet-2"));

        packet.mark_resolved();
        assert_eq!(packet.state, DelegationPacketState::Resolved);
        assert!(packet.resolved_at.is_some());
        assert!(packet.superseded_by_packet_id.is_none());

        let capability_packet = DelegationPacket {
            packet_id: "packet-3".to_string(),
            kind: DelegationPacketKind::Escalation,
            state: DelegationPacketState::Exhausted,
            created_at: 10,
            resolved_at: Some(20),
            source_route_owner: "native".to_string(),
            target_owner: "operator".to_string(),
            continuity_reason: "verification route cannot validate directly".to_string(),
            recommended_next_action: "boundline inspect".to_string(),
            evidence_refs: Vec::new(),
            capability_summary: Some("validation=unsupported".to_string()),
            stuck_marker: None,
            superseded_by_packet_id: None,
        };
        assert_eq!(
            capability_packet.headline(),
            "escalation required: verification route cannot validate directly".to_string()
        );
        assert_eq!(capability_packet.evidence_summary(), "validation=unsupported".to_string());
    }

    #[test]
    fn delegation_status_and_next_command_fall_back_to_task_context_and_validate_modes() {
        let workspace = "/tmp/boundline-session-domain-delegation-context";
        let mut record = build_record(workspace);
        record.goal_plan = None;

        let packet = DelegationPacket {
            packet_id: "packet-stuck".to_string(),
            kind: DelegationPacketKind::Escalation,
            state: DelegationPacketState::Stuck,
            created_at: 10,
            resolved_at: None,
            source_route_owner: "native".to_string(),
            target_owner: "operator".to_string(),
            continuity_reason: "verification route cannot continue".to_string(),
            recommended_next_action: "boundline inspect".to_string(),
            evidence_refs: vec!["trace:delegation:packet-stuck".to_string()],
            capability_summary: Some("validation=unsupported".to_string()),
            stuck_marker: Some(StuckEvidenceMarker {
                repeated_attempts: 3,
                same_reason_count: 3,
                unchanged_workspace_signal: true,
                stale_route_policy: false,
                recommended_recovery: StuckRecoveryAction::RerunValidation,
            }),
            superseded_by_packet_id: None,
        };
        let continuity = DelegationContinuityState {
            active_packet_id: Some(packet.packet_id.clone()),
            mode: DelegationContinuityMode::Stuck,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: "boundline inspect".to_string(),
            headline: "stuck delegated continuity: verification route cannot continue".to_string(),
            evidence_summary: "trace:delegation:packet-stuck".to_string(),
        };
        record
            .active_task
            .as_mut()
            .unwrap()
            .context
            .set_delegation_packet_history(std::slice::from_ref(&packet))
            .unwrap();
        record
            .active_task
            .as_mut()
            .unwrap()
            .context
            .set_delegation_continuity_state(&continuity)
            .unwrap();

        let status = delegation_status_view(&record).unwrap();
        assert_eq!(status.mode, DelegationContinuityMode::Stuck);
        assert_eq!(status.packet_id.as_deref(), Some("packet-stuck"));
        assert_eq!(status.packet_kind, Some(DelegationPacketKind::Escalation));
        assert_eq!(status.packet_state, Some(DelegationPacketState::Stuck));
        assert_eq!(status.target_owner.as_deref(), Some("operator"));
        assert_eq!(delegation_next_command(&record), Some("boundline inspect".to_string()));

        let resolved = DelegationContinuityState {
            active_packet_id: None,
            mode: DelegationContinuityMode::Resolved,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: "boundline run".to_string(),
            headline: "delegation resolved after routing update".to_string(),
            evidence_summary: "routing policy now supports direct continuation".to_string(),
        };
        let resolved_view =
            DelegationStatusView::from_continuity(&resolved, std::slice::from_ref(&packet))
                .unwrap();
        assert_eq!(resolved_view.packet_id.as_deref(), Some("packet-stuck"));

        let invalid_active_packet = DelegationContinuityState {
            active_packet_id: Some(packet.packet_id.clone()),
            mode: DelegationContinuityMode::Resolved,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: "boundline run".to_string(),
            headline: "delegation resolved".to_string(),
            evidence_summary: "routing policy changed".to_string(),
        };
        assert!(
            invalid_active_packet
                .validate(std::slice::from_ref(&packet))
                .unwrap_err()
                .contains("must not keep an active_packet_id")
        );

        let invalid_authority = DelegationContinuityState {
            active_packet_id: None,
            mode: DelegationContinuityMode::Resolved,
            authority_source: ContinuityAuthority::NoFollowUpState,
            next_command: "boundline run".to_string(),
            headline: "delegation resolved".to_string(),
            evidence_summary: "routing policy changed".to_string(),
        };
        assert!(
            invalid_authority
                .validate(&[])
                .unwrap_err()
                .contains("may only be used with delegation mode none")
        );

        assert!(DelegationContinuityMode::EscalationRequired.requires_active_packet());
        assert!(!DelegationContinuityMode::InspectOnly.requires_active_packet());
        assert_eq!(CompatibilityFollowUpMode::Resumable.as_str(), "resumable");
    }

    #[test]
    fn routing_outcome_covers_delegation_goal_plan_compatibility_goal_capture_and_empty_session() {
        let workspace = "/tmp/boundline-session-domain-routing";

        let packet = DelegationPacket {
            packet_id: "packet-routing".to_string(),
            kind: DelegationPacketKind::Handoff,
            state: DelegationPacketState::Active,
            created_at: 10,
            resolved_at: None,
            source_route_owner: "native".to_string(),
            target_owner: "codex".to_string(),
            continuity_reason: "implementation route requires handoff".to_string(),
            recommended_next_action: "boundline status".to_string(),
            evidence_refs: vec!["trace:delegation:packet-routing".to_string()],
            capability_summary: Some("continuation=unsupported".to_string()),
            stuck_marker: None,
            superseded_by_packet_id: None,
        };
        let continuity = DelegationContinuityState {
            active_packet_id: Some(packet.packet_id.clone()),
            mode: DelegationContinuityMode::HandoffRequired,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: "boundline status".to_string(),
            headline: packet.headline(),
            evidence_summary: packet.evidence_summary(),
        };

        let delegated_goal_plan = GoalPlan::new(
            "Deliver a session-backed CLI",
            vec![PlannedTask {
                task_id: "planned-task-routing".to_string(),
                description: "Implement the session runtime".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("session runtime works".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_delegation_state(vec![packet], continuity)
        .unwrap();
        let mut delegated_record = build_record(workspace);
        delegated_record.active_task = None;
        delegated_record.goal_plan = Some(delegated_goal_plan);

        let delegated_outcome = routing_outcome(&delegated_record);
        assert_eq!(delegated_outcome.mode, RoutingMode::Blocked);
        assert_eq!(delegated_outcome.source, RoutingSource::SessionState);
        assert!(delegated_outcome.reason.contains("handoff required"));
        assert_eq!(execution_path_text(&delegated_record), None);

        let mut pending_record = build_record(workspace);
        pending_record.active_task = None;
        pending_record.goal_plan = Some(
            GoalPlan::new(
                "Deliver a session-backed CLI",
                vec![PlannedTask {
                    task_id: "planned-task-pending".to_string(),
                    description: "Confirm the plan".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some("plan confirmed".to_string()),
                    decision_type_hint: None,
                }],
            )
            .unwrap(),
        );
        let pending_outcome = routing_outcome(&pending_record);
        assert_eq!(pending_outcome.mode, RoutingMode::Blocked);
        assert_eq!(pending_outcome.source, RoutingSource::GoalPlan);
        assert_eq!(
            execution_path_text(&pending_record),
            Some("native_goal_plan_pending_plan_confirmation".to_string())
        );

        let mut native_record = build_record(workspace);
        native_record.active_task = None;
        native_record.goal_plan = Some(build_context_goal_plan());
        let native_outcome = routing_outcome(&native_record);
        assert_eq!(native_outcome.mode, RoutingMode::Native);
        assert_eq!(native_outcome.source, RoutingSource::GoalPlan);
        assert_eq!(execution_path_text(&native_record), Some("native_goal_plan".to_string()));

        let compatibility_record = build_record(workspace);
        let compatibility_outcome = routing_outcome(&compatibility_record);
        assert_eq!(compatibility_outcome.mode, RoutingMode::Compatibility);
        assert_eq!(compatibility_outcome.source, RoutingSource::ExecutionProfile);
        assert_eq!(
            execution_path_text(&compatibility_record),
            Some("fixture_compatibility".to_string())
        );

        let mut goal_capture_record = build_record(workspace);
        goal_capture_record.active_task = None;
        goal_capture_record.goal_plan = None;
        let goal_capture_outcome = routing_outcome(&goal_capture_record);
        assert_eq!(goal_capture_outcome.mode, RoutingMode::Blocked);
        assert_eq!(goal_capture_outcome.source, RoutingSource::GoalCapture);
        assert_eq!(
            execution_path_text(&goal_capture_record),
            Some("native_session_pending_plan".to_string())
        );

        let mut empty_record = build_record(workspace);
        empty_record.goal = None;
        empty_record.active_task = None;
        empty_record.goal_plan = None;
        let empty_outcome = routing_outcome(&empty_record);
        assert_eq!(empty_outcome.mode, RoutingMode::Blocked);
        assert_eq!(empty_outcome.source, RoutingSource::SessionState);
        assert!(empty_outcome.reason.contains("no goal plan"));
        assert_eq!(execution_path_text(&empty_record), None);

        assert_eq!(RoutingMode::Native.as_str(), "native");
        assert_eq!(RoutingMode::Compatibility.as_str(), "compatibility");
        assert_eq!(RoutingMode::Blocked.as_str(), "blocked");
        assert_eq!(RoutingSource::GoalPlan.as_str(), "goal_plan");
        assert_eq!(RoutingSource::ExecutionProfile.as_str(), "execution_profile");
        assert_eq!(RoutingSource::GoalCapture.as_str(), "goal_capture");
        assert_eq!(RoutingSource::SessionState.as_str(), "session_state");
    }
}
