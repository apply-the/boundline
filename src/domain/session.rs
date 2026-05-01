use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::domain::brief::AuthoredBriefBundle;
use crate::domain::decision::{Decision, DecisionStatus};
use crate::domain::flow::SessionFlowState;
use crate::domain::flow_policy::FlowPolicy;
use crate::domain::goal_plan::{GoalPlan, GoalPlanFlowMode};
use crate::domain::governance::{
    AutopilotDecisionRecord, GovernedStagePacket, GovernedStageRecord, PacketReuseBinding,
};
use crate::domain::task::{Task, TaskPersistenceError, TaskStatus, TerminalReason};
use crate::domain::task_context::{
    LATEST_GOVERNANCE_DECISION_KEY, LATEST_GOVERNANCE_PACKET_KEY,
    LATEST_GOVERNANCE_PACKET_REUSE_KEY, LATEST_GOVERNANCE_STAGE_KEY,
};
use crate::domain::workflow::WorkflowProgressState;

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
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Exhausted | Self::Aborted)
    }
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveSessionRecord {
    pub session_id: String,
    pub workspace_ref: String,
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_brief: Option<AuthoredBriefBundle>,
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
}

impl ActiveSessionRecord {
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

        if let Some(trace_ref) = &self.latest_trace_ref
            && !trace_within_workspace(&self.workspace_ref, trace_ref)
        {
            return Err(SessionValidationError::TraceOutsideWorkspace {
                workspace_ref: self.workspace_ref.clone(),
                trace_ref: trace_ref.clone(),
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

            if !task.context.belongs_to_workspace(&self.workspace_ref) {
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

        Ok(())
    }

    pub fn active_workflow_progress(&self) -> Option<&WorkflowProgressState> {
        self.workflow_progress.as_ref().or_else(|| {
            self.goal_plan.as_ref().and_then(|goal_plan| goal_plan.workflow_progress.as_ref())
        })
    }

    pub fn active_workflow_name(&self) -> Option<String> {
        self.active_workflow_progress().map(|workflow| workflow.workflow_name.clone())
    }

    pub fn active_workflow_phase_text(&self) -> Option<String> {
        self.active_workflow_progress().and_then(WorkflowProgressState::current_phase_text)
    }

    pub fn active_workflow_next_action(&self) -> Option<String> {
        self.active_workflow_progress().and_then(WorkflowProgressState::next_action_text)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTransition {
    pub trigger_command: SessionCommand,
    pub from_status: Option<SessionStatus>,
    pub to_status: SessionStatus,
    pub trace_ref: Option<String>,
    pub reason: String,
}

impl SessionTransition {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStatusView {
    pub session_id: String,
    pub workspace_ref: String,
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_sources: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_input_deduplicated_sources: Option<Vec<String>>,
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
    pub active_workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_authority: Option<ContinuityAuthority>,
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
    pub latest_workspace_slice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_selection_headline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_attempt_lineage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_validation_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_vote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_outcome: Option<String>,
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
    pub next_command: Option<String>,
    pub explanation: String,
}

impl SessionStatusView {
    pub fn validate(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingOutcome {
    pub mode: RoutingMode,
    pub source: RoutingSource,
    pub reason: String,
}

impl RoutingOutcome {
    pub fn execution_path_key(&self) -> Option<&'static str> {
        match (self.mode, self.source) {
            (RoutingMode::Native, RoutingSource::GoalPlan) => Some("native_goal_plan"),
            (RoutingMode::Compatibility, RoutingSource::ExecutionProfile) => {
                Some("fixture_compatibility")
            }
            (RoutingMode::Blocked, RoutingSource::GoalPlan) => {
                Some("native_goal_plan_pending_flow_confirmation")
            }
            (RoutingMode::Blocked, RoutingSource::GoalCapture) => {
                Some("native_session_pending_plan")
            }
            _ => None,
        }
    }
}

pub fn routing_outcome(record: &ActiveSessionRecord) -> RoutingOutcome {
    if let Some(goal_plan) = record.goal_plan.as_ref() {
        if goal_plan.flow_state().mode == GoalPlanFlowMode::Proposed {
            return RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::GoalPlan,
                reason: "flow confirmation is still pending before native execution".to_string(),
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

pub fn execution_path_text(record: &ActiveSessionRecord) -> Option<String> {
    routing_outcome(record).execution_path_key().map(str::to_string)
}

pub fn decision_status_text(status: DecisionStatus) -> &'static str {
    match status {
        DecisionStatus::Pending => "pending",
        DecisionStatus::Dispatched => "dispatched",
        DecisionStatus::Verified => "verified",
        DecisionStatus::Failed => "failed",
        DecisionStatus::Recovered => "recovered",
    }
}

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

fn task_state_string(task: &Task, key: &str) -> Option<String> {
    task.context.state.get(key).and_then(|value| value.as_str().map(str::to_string))
}

fn task_state_json<T: DeserializeOwned>(task: &Task, key: &str) -> Option<T> {
    task.context.state.get(key).cloned().and_then(|value| serde_json::from_value(value).ok())
}

fn task_state_strings(task: &Task, key: &str) -> Option<Vec<String>> {
    task.context.state.get(key).and_then(|value| {
        value.as_array().map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
    })
}

pub(crate) fn task_state_governed_stage(task: &Task) -> Option<GovernedStageRecord> {
    task_state_json(task, LATEST_GOVERNANCE_STAGE_KEY)
}

pub(crate) fn task_state_governed_packet(task: &Task) -> Option<GovernedStagePacket> {
    task_state_json(task, LATEST_GOVERNANCE_PACKET_KEY)
}

pub(crate) fn task_state_governance_packet_reuse(task: &Task) -> Option<PacketReuseBinding> {
    task_state_json(task, LATEST_GOVERNANCE_PACKET_REUSE_KEY)
}

pub(crate) fn task_state_governance_decision(task: &Task) -> Option<AutopilotDecisionRecord> {
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

pub(crate) fn task_state_governance_stage_key(task: &Task) -> Option<String> {
    task_state_governed_stage(task).map(|record| record.stage_key)
}

pub(crate) fn task_state_governance_runtime_text(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| encoded_text(&record.runtime))
}

pub(crate) fn task_state_governance_mode_text(task: &Task) -> Option<String> {
    task_state_governed_packet(task)
        .and_then(|packet| packet.canon_mode)
        .and_then(|mode| encoded_text(&mode))
}

pub(crate) fn task_state_governance_canon_run_ref(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| record.canon_run_ref)
}

pub(crate) fn task_state_governance_state_text(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| encoded_text(&record.lifecycle_state))
}

pub(crate) fn task_state_governance_blocked_reason(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| record.blocked_reason)
}

pub(crate) fn task_state_governance_packet_ref(task: &Task) -> Option<String> {
    task_state_governed_packet(task)
        .map(|packet| packet.packet_ref)
        .or_else(|| task_state_governed_stage(task).and_then(|record| record.packet_ref))
}

pub(crate) fn task_state_governance_packet_source_stage(task: &Task) -> Option<String> {
    task_state_governance_packet_reuse(task).map(|binding| binding.upstream_stage_key)
}

pub(crate) fn task_state_governance_packet_binding_reason(task: &Task) -> Option<String> {
    task_state_governance_packet_reuse(task).map(|binding| binding.binding_reason)
}

pub(crate) fn governance_packet_provenance_text(
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

pub(crate) fn task_state_governance_approval_text(task: &Task) -> Option<String> {
    task_state_governed_stage(task).and_then(|record| encoded_text(&record.approval_state))
}

pub(crate) fn task_state_governance_decision_headline(task: &Task) -> Option<String> {
    task_state_governance_decision(task).map(|decision| decision.rationale)
}

pub(crate) fn task_state_governance_candidate_actions(task: &Task) -> Option<Vec<String>> {
    task_state_governance_decision(task).map(|decision| {
        decision
            .candidate_actions
            .into_iter()
            .map(|action| autopilot_action_text(action).to_string())
            .collect::<Vec<_>>()
    })
}

pub(crate) fn governance_next_action_for_state(governance_state: Option<&str>) -> Option<String> {
    match governance_state {
        Some("awaiting_approval") => Some("wait for approval and rerun synod status".to_string()),
        Some("blocked") => {
            Some("resolve the governance blocker, then rerun synod step".to_string())
        }
        _ => None,
    }
}

pub(crate) fn task_state_governance_next_action(task: &Task) -> Option<String> {
    let governance_state = task_state_governance_state_text(task);
    governance_next_action_for_state(governance_state.as_deref())
}

pub(crate) fn task_state_workspace_slice_summary(task: &Task) -> Option<String> {
    let slice = task.context.state.get("latest_workspace_slice")?;
    let selected_targets = slice.get("selected_targets")?.as_array()?;
    let targets = selected_targets.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();

    if targets.is_empty() { None } else { Some(targets.join(", ")) }
}

pub(crate) fn task_state_attempt_lineage_summary(task: &Task) -> Option<String> {
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
        ActiveSessionRecord, SessionStatus, SessionStatusView, SessionValidationError,
        execution_path_text, task_state_attempt_lineage_summary, task_state_review_headline,
        task_state_string, task_state_strings, task_state_workspace_slice_summary,
        trace_within_workspace,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::Plan;
    use crate::domain::step::Step;
    use crate::domain::task::{Task, TaskPersistenceError, TaskRunRequest};

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
            latest_trace_ref: Some(format!("{workspace_ref}/.synod/traces/task-1.json")),
            created_at: 10,
            updated_at: 20,
        }
    }

    fn build_view(record: &ActiveSessionRecord) -> SessionStatusView {
        SessionStatusView {
            session_id: record.session_id.clone(),
            workspace_ref: record.workspace_ref.clone(),
            goal: record.goal.clone(),
            authored_input_summary: None,
            authored_input_sources: None,
            authored_input_deduplicated_sources: None,
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
            active_workflow: record.active_workflow_name(),
            workflow_phase: record.active_workflow_phase_text(),
            workflow_next_action: record.active_workflow_next_action(),
            continuity_authority: None,
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
            latest_workspace_slice: None,
            latest_selection_headline: None,
            latest_attempt_lineage: None,
            latest_validation_status: None,
            latest_review_trigger: None,
            latest_review_vote: None,
            latest_review_outcome: None,
            latest_review_headline: None,
            latest_governance_stage: None,
            latest_governance_runtime: None,
            latest_governance_mode: None,
            latest_governance_run_ref: None,
            latest_governance_state: None,
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: None,
            latest_governance_packet_source_stage: None,
            latest_governance_packet_binding_reason: None,
            latest_governance_approval: None,
            latest_governance_decision: None,
            latest_governance_candidates: None,
            governance_next_action: None,
            next_command: Some("synod step".to_string()),
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
            })
            .unwrap();
        task.context
            .set_latest_governance_packet_reuse(&crate::domain::governance::PacketReuseBinding {
                upstream_stage_key: "bug-fix:investigate".to_string(),
                downstream_stage_key: "bug-fix:implement".to_string(),
                packet_ref: ".canon/runs/canon-run-1".to_string(),
                binding_reason: "upstream_stage_context".to_string(),
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
        view.latest_workspace_slice = task_state_workspace_slice_summary(task);
        view.clarification_headline =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_headline());
        view.clarification_prompt =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_prompt());
        view.clarification_missing_fields =
            record.authored_brief.as_ref().and_then(|bundle| bundle.clarification_missing_fields());
        view.latest_selection_headline = task_state_string(task, "latest_selection_headline");
        view.latest_attempt_lineage = task_state_attempt_lineage_summary(task);
        view.latest_validation_status = task_state_string(task, "latest_validation_status");
        view.latest_review_trigger = task_state_string(task, "latest_review_trigger");
        view.latest_review_vote = task_state_string(task, "latest_review_vote");
        view.latest_review_outcome = task_state_string(task, "latest_review_outcome");
        view.latest_review_headline = task_state_review_headline(task);
        view.latest_governance_stage = super::task_state_governance_stage_key(task);
        view.latest_governance_runtime = super::task_state_governance_runtime_text(task);
        view.latest_governance_mode = super::task_state_governance_mode_text(task);
        view.latest_governance_run_ref = super::task_state_governance_canon_run_ref(task);
        view.latest_governance_state = super::task_state_governance_state_text(task);
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

    #[test]
    fn status_view_rejects_stage_count_trace_and_step_index_mismatches() {
        let workspace = "/tmp/synod-session-domain";
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
        let record = build_derived_state_record("/tmp/synod-session-domain-derived");
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
}
