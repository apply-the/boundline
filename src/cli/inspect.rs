use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::cluster::ClusterDeliveryStory;
use crate::domain::goal_plan::GoalPlanFlowState;
use crate::domain::limits::TerminalCondition;
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::{RoutingMode, RoutingOutcome, RoutingSource};
use crate::domain::session::{governance_next_action_for_state, governance_packet_provenance_text};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskStatus, TerminalReason};
use crate::domain::tool_result::ToolResult;
use crate::domain::trace::{
    ExecutionTrace, TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
};

const UNKNOWN_VALIDATION_EXIT_CODE: i64 = -1;
const UNKNOWN_DECISION_ID: &str = "unknown-decision";
const UNKNOWN_TARGET: &str = "unknown";
const KEY_ACTION_RESULT: &str = "action_result";
const KEY_FAILURE_REASON: &str = "failure_reason";
const KEY_FINDING: &str = "finding";
const KEY_REVIEW_OUTCOME: &str = "review_outcome";
const KEY_REVIEW_TRIGGER: &str = "review_trigger";
const KEY_REVIEWER_ID: &str = "reviewer_id";
const KEY_REVIEWER_ROLE: &str = "reviewer_role";
const KEY_SUMMARY: &str = "summary";
const KEY_TARGET: &str = "target";
const KEY_VOTE_RESOLUTION: &str = "vote_resolution";

#[derive(Debug, Clone, PartialEq)]
pub struct InspectCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
    pub trace_summary: Option<TraceSummaryView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceResolutionTarget {
    ExplicitTrace,
    SessionTraceRef,
    LatestWorkspaceTrace,
}

impl TraceResolutionTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitTrace => "explicit-trace",
            Self::SessionTraceRef => "session-trace-ref",
            Self::LatestWorkspaceTrace => "latest-workspace-trace",
        }
    }
}

pub fn execute_inspect(
    trace: Option<&Path>,
    workspace: Option<&Path>,
) -> Result<InspectCommandReport, InspectCommandError> {
    let (inspection_target, trace_ref, trace) = load_trace(trace, workspace)?;
    let summary = summarize_trace(&trace_ref, &trace)?;
    let exit_status = if summary.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };

    Ok(InspectCommandReport {
        exit_status,
        terminal_output: output::render_trace_summary(
            &summary,
            inspection_target.as_str(),
            output::next_command_after_inspect(summary.terminal_status),
        ),
        trace_location: Some(trace_ref.to_string_lossy().into_owned()),
        trace_summary: Some(summary),
    })
}

pub fn render_error(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    error: &InspectCommandError,
) -> String {
    if let InspectCommandError::InvalidSession(message) = error {
        return output::render_session_error("inspect", message, Some("boundline start"));
    }

    let inspection_target = inspection_target_for(trace, workspace);
    let trace_ref = trace.map(|path| path.to_string_lossy().into_owned());
    let workspace_ref = workspace.map(|path| path.to_string_lossy().into_owned());
    let terminal_reason = match error {
        InspectCommandError::MissingTraceReference => "inspect requires --trace or --workspace",
        InspectCommandError::MissingLatestTrace | InspectCommandError::TraceStore(_) => {
            "failed to read the requested trace"
        }
        InspectCommandError::SessionStore(_) => "failed to read the active session",
        InspectCommandError::InvalidSession(_) => "active session is invalid",
        InspectCommandError::Summary(_) => "failed to summarize the requested trace",
    };

    output::render_inspect_failure(
        inspection_target.as_str(),
        trace_ref.as_deref(),
        workspace_ref.as_deref(),
        terminal_reason,
        corrected_command(inspection_target),
    )
}

pub fn render_inspection_routing_summary(
    outcome: &RoutingOutcome,
    flow_state: Option<&GoalPlanFlowState>,
) -> Vec<String> {
    let mut lines = vec![output::render_route_outcome(outcome)];
    if let Some(flow_state) = flow_state {
        lines.push(output::render_goal_plan_flow_state(flow_state));
    }
    lines
}

pub fn summarize_trace(
    trace_ref: impl AsRef<Path>,
    trace: &ExecutionTrace,
) -> Result<TraceSummaryView, TraceSummaryError> {
    let persisted_terminal_status = trace.terminal_status;
    let persisted_terminal_reason = trace.terminal_reason.clone();
    let mut authored_input_summary: Option<String> = None;
    let mut authored_input_sources: Vec<String> = Vec::new();
    let mut authored_input_deduplicated_sources: Vec<String> = Vec::new();
    let mut clarification_headline: Option<String> = None;
    let mut clarification_prompt: Option<String> = None;
    let mut clarification_missing_fields: Vec<String> = Vec::new();
    let mut requested_governance_runtime: Option<String> = None;
    let mut requested_governance_risk: Option<String> = None;
    let mut requested_governance_zone: Option<String> = None;
    let mut requested_governance_owner: Option<String> = None;
    let mut negotiation_goal_summary: Option<String> = None;
    let mut negotiation_resolution: Option<String> = None;
    let mut negotiation_acceptance_boundary: Option<String> = None;
    let mut cluster_delivery_story: Option<ClusterDeliveryStory> = None;
    let mut routing_summary: Option<String> = None;
    let mut routing_projection = RoutingDecisionProjection::default();
    let mut goal_plan_summary: Option<String> = None;
    let mut context_summary: Option<String> = None;
    let mut context_credibility: Option<String> = None;
    let mut context_primary_inputs: Vec<String> = Vec::new();
    let mut context_provenance: Vec<String> = Vec::new();
    let mut context_staleness_reason: Option<String> = None;
    let mut governance_next_action: Option<String> = None;
    let mut decision_timeline: Vec<String> = Vec::new();
    let mut failure_evidence: Vec<String> = Vec::new();
    let mut adaptive_evidence: Vec<String> = Vec::new();
    let mut latest_checkpoint_id: Option<String> = None;
    let mut latest_checkpoint_scope: Option<String> = None;
    let mut latest_checkpoint_restore_command: Option<String> = None;
    let mut latest_governance_state: Option<String> = None;
    let mut delegation: Option<crate::domain::session::DelegationStatusView> = None;
    let mut saw_native_routing_signal = false;
    let mut step_indexes: HashMap<String, usize> = HashMap::new();
    let mut executed_steps: Vec<TraceStepSummary> = Vec::new();
    let mut recovery_events: Vec<TraceRecoveryEvent> = Vec::new();
    let mut governance_timeline: Vec<String> = Vec::new();
    let mut review_timeline: Vec<String> = Vec::new();

    for event in &trace.events {
        if routing_projection.is_empty()
            && let Some(projection) = RoutingDecisionProjection::from_event_payload(&event.payload)
        {
            routing_projection = projection;
        }

        if delegation.is_none() {
            delegation = event
                .payload
                .get("delegation")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok());
        }

        if event.event_type.is_decision_loop_event() {
            saw_native_routing_signal = true;
        }

        match event.event_type {
            TraceEventType::TaskStarted => {
                if authored_input_summary.is_none() {
                    authored_input_summary = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("authored_input_summary"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if authored_input_sources.is_empty() {
                    authored_input_sources = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("authored_input_sources"))
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if authored_input_deduplicated_sources.is_empty() {
                    authored_input_deduplicated_sources = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("authored_input_deduplicated_sources"))
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if clarification_headline.is_none() {
                    clarification_headline = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("clarification_headline"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if clarification_prompt.is_none() {
                    clarification_prompt = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("clarification_prompt"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if clarification_missing_fields.is_empty() {
                    clarification_missing_fields = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("clarification_missing_fields"))
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if requested_governance_runtime.is_none() {
                    requested_governance_runtime = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("requested_governance_runtime"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if requested_governance_risk.is_none() {
                    requested_governance_risk = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("requested_governance_risk"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if requested_governance_zone.is_none() {
                    requested_governance_zone = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("requested_governance_zone"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if requested_governance_owner.is_none() {
                    requested_governance_owner = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("requested_governance_owner"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if negotiation_goal_summary.is_none() {
                    negotiation_goal_summary = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("negotiation_goal_summary"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if negotiation_resolution.is_none() {
                    negotiation_resolution = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("negotiation_resolution"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if negotiation_acceptance_boundary.is_none() {
                    negotiation_acceptance_boundary = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("negotiation_acceptance_boundary"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_summary.is_none() {
                    context_summary = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("context_summary"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_credibility.is_none() {
                    context_credibility = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("context_credibility"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_primary_inputs.is_empty() {
                    context_primary_inputs = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("context_primary_inputs"))
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if context_provenance.is_empty() {
                    context_provenance = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("context_provenance"))
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if context_staleness_reason.is_none() {
                    context_staleness_reason = event
                        .payload
                        .get("input")
                        .and_then(|input| input.get("context_staleness_reason"))
                        .and_then(|value| value.as_str().map(str::to_string));
                }
            }
            TraceEventType::TerminalRecorded => {
                cluster_delivery_story = event
                    .payload
                    .get("cluster_delivery_story")
                    .cloned()
                    .and_then(|value| serde_json::from_value(value).ok());
            }
            TraceEventType::ReviewerStarted => {}
            TraceEventType::CheckpointCreated => {
                latest_checkpoint_id = event
                    .payload
                    .get("checkpoint_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(latest_checkpoint_id);
                latest_checkpoint_scope = event
                    .payload
                    .get("checkpoint_scope")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(latest_checkpoint_scope);
                latest_checkpoint_restore_command = event
                    .payload
                    .get("checkpoint_restore_command")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(latest_checkpoint_restore_command);
            }
            TraceEventType::FlowSelected => {
                recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: format!(
                        "{} @ {}",
                        event
                            .payload
                            .get("flow_name")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-flow"),
                        event
                            .payload
                            .get("current_stage_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-stage")
                    ),
                    related_step_id: None,
                });
            }
            TraceEventType::StageTransitioned => {
                recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: format!(
                        "{} -> {}",
                        event
                            .payload
                            .get("from_stage_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-stage"),
                        event
                            .payload
                            .get("to_stage_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-stage")
                    ),
                    related_step_id: event.step_id.clone(),
                });
            }
            TraceEventType::StepStarted => {
                let step_id = event
                    .step_id
                    .clone()
                    .ok_or(TraceSummaryError::MissingStepId(event.event_type))?;
                let step_kind = parse_step_kind(
                    event
                        .payload
                        .get("step_kind")
                        .and_then(|value| value.as_str())
                        .ok_or_else(|| TraceSummaryError::MissingStepKind(step_id.clone()))?,
                )?;

                if let Some(index) = step_indexes.get(&step_id) {
                    executed_steps[*index].attempts += 1;
                } else {
                    step_indexes.insert(step_id.clone(), executed_steps.len());
                    executed_steps.push(TraceStepSummary {
                        step_id,
                        step_kind,
                        attempts: 1,
                        final_status: StepStatus::Running,
                        headline: "started".to_string(),
                    });
                }
            }
            TraceEventType::StepCompleted => {
                let step_id = event
                    .step_id
                    .clone()
                    .ok_or(TraceSummaryError::MissingStepId(event.event_type))?;
                let final_status = match event
                    .payload
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                {
                    "succeeded" => StepStatus::Succeeded,
                    "failed" => StepStatus::Failed,
                    _ => StepStatus::Running,
                };

                let index = *step_indexes
                    .get(&step_id)
                    .ok_or_else(|| TraceSummaryError::MissingStartedStep(step_id.clone()))?;
                let headline = match final_status {
                    StepStatus::Succeeded => {
                        success_headline(&event.payload, executed_steps[index].attempts)
                    }
                    StepStatus::Failed => {
                        failure_headline(&event.payload, executed_steps[index].attempts)
                    }
                    _ => "completed".to_string(),
                };
                executed_steps[index].final_status = final_status;
                executed_steps[index].headline = headline;
                for line in adaptive_evidence_lines(&event.payload) {
                    if !adaptive_evidence.contains(&line) {
                        adaptive_evidence.push(line);
                    }
                }
            }
            TraceEventType::RetryScheduled
            | TraceEventType::StageRetryScheduled
            | TraceEventType::Replanned
            | TraceEventType::StageReplanned
            | TraceEventType::StageFailed => {
                recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: event
                        .payload
                        .get("reason")
                        .and_then(|value| value.as_str())
                        .unwrap_or("recovery event")
                        .to_string(),
                    related_step_id: event.step_id.clone(),
                });
            }
            TraceEventType::GovernanceSelected
            | TraceEventType::GovernanceStarted
            | TraceEventType::GovernanceDecisionRecorded
            | TraceEventType::GovernanceAwaitingApproval
            | TraceEventType::GovernanceCompleted
            | TraceEventType::GovernanceBlocked
            | TraceEventType::GovernancePacketRejected => {
                saw_native_routing_signal = true;
                match event.event_type {
                    TraceEventType::GovernanceAwaitingApproval => {
                        latest_governance_state = Some("awaiting_approval".to_string());
                    }
                    TraceEventType::GovernanceCompleted => {
                        latest_governance_state = Some("governed_ready".to_string());
                    }
                    TraceEventType::GovernanceBlocked
                    | TraceEventType::GovernancePacketRejected => {
                        latest_governance_state = Some("blocked".to_string());
                    }
                    _ => {}
                }
                if context_summary.is_none() {
                    context_summary = event
                        .payload
                        .get("canon_memory_summary")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_credibility.is_none() {
                    context_credibility = event
                        .payload
                        .get("canon_memory_credibility")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_primary_inputs.is_empty() {
                    context_primary_inputs = event
                        .payload
                        .get("document_refs")
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if let Some(canon_memory_summary) =
                    event.payload.get("canon_memory_summary").and_then(|value| value.as_str())
                {
                    let line = format!("canon_memory: {canon_memory_summary}");
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
                if let Some(canon_memory_compatibility) =
                    event.payload.get("canon_memory_compatibility").and_then(|value| value.as_str())
                {
                    let line = format!("canon_memory_compatibility: {canon_memory_compatibility}");
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
                if let Some(canon_memory_run_ref) = event
                    .payload
                    .get("canon_memory_run_ref")
                    .or_else(|| event.payload.get("run_ref"))
                    .and_then(|value| value.as_str())
                {
                    let line = format!("canon_memory_run_ref: {canon_memory_run_ref}");
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
                if let Some(canon_memory_packet_ref) = event
                    .payload
                    .get("canon_memory_packet_ref")
                    .or_else(|| event.payload.get("packet_ref"))
                    .and_then(|value| value.as_str())
                {
                    let line = format!("canon_memory_packet: {canon_memory_packet_ref}");
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
                if let Some(canon_memory_reason_code) =
                    event.payload.get("canon_memory_reason_code").and_then(|value| value.as_str())
                {
                    let line = format!("canon_memory_reason: {canon_memory_reason_code}");
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
                if let Some(canon_next_action) =
                    event.payload.get("canon_next_action").and_then(|value| value.as_str())
                {
                    let line = format!("canon_memory_next_action: {canon_next_action}");
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
                if context_staleness_reason.is_none()
                    && event
                        .payload
                        .get("canon_memory_credibility")
                        .and_then(|value| value.as_str())
                        .is_some_and(|credibility| credibility != "credible")
                {
                    context_staleness_reason = event
                        .payload
                        .get("reason")
                        .and_then(|value| value.as_str().map(str::to_string))
                        .or_else(|| {
                            event
                                .payload
                                .get("canon_memory_summary")
                                .and_then(|value| value.as_str().map(str::to_string))
                        });
                }
                if governance_next_action.is_none() {
                    governance_next_action = event
                        .payload
                        .get("canon_next_action")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if let Some(line) = governance_timeline_line(event.event_type, &event.payload) {
                    governance_timeline.push(line);
                }
            }
            TraceEventType::ReviewStarted
            | TraceEventType::ReviewTriggerIgnored
            | TraceEventType::ReviewerCompleted
            | TraceEventType::ReviewVoteResolved
            | TraceEventType::ReviewAdjudicated
            | TraceEventType::ReviewTerminalRecorded => {
                if let Some(line) = review_timeline_line(event.event_type, &event.payload) {
                    review_timeline.push(line);
                }
            }
            TraceEventType::GoalPlanCreated => {
                if routing_summary.is_none() {
                    routing_summary = Some(output::render_route_outcome(&RoutingOutcome {
                        mode: RoutingMode::Native,
                        source: RoutingSource::GoalPlan,
                        reason: "goal plan trace came from the session-native runtime".to_string(),
                    }));
                }
                if goal_plan_summary.is_none() {
                    let task_count = event
                        .payload
                        .get("task_count")
                        .and_then(|value| value.as_u64())
                        .unwrap_or_default();
                    let goal = event
                        .payload
                        .get("goal")
                        .and_then(|value| value.as_str())
                        .unwrap_or(&trace.goal);
                    let state_suffix = event
                        .payload
                        .get("goal_plan_state")
                        .and_then(|value| value.as_str())
                        .map(|state| {
                            format!(
                                " [{state} rev {}]",
                                event
                                    .payload
                                    .get("goal_plan_revision")
                                    .and_then(|value| value.as_u64())
                                    .unwrap_or(1)
                            )
                        })
                        .unwrap_or_default();
                    let flow_suffix = event
                        .payload
                        .get("flow_state")
                        .and_then(|value| value.as_str())
                        .map(|flow_state| format!(" | flow: {flow_state}"))
                        .unwrap_or_default();
                    let verification_suffix = event
                        .payload
                        .get("verification_strategy")
                        .and_then(|value| value.as_str())
                        .map(|verification_strategy| {
                            format!(" | verification: {verification_strategy}")
                        })
                        .unwrap_or_default();
                    let rationale_suffix = event
                        .payload
                        .get("planning_rationale")
                        .and_then(|value| value.as_str())
                        .map(|planning_rationale| format!(" | rationale: {planning_rationale}"))
                        .unwrap_or_default();
                    goal_plan_summary = Some(format!(
                        "{task_count} bounded task(s) for {goal}{state_suffix}{flow_suffix}{verification_suffix}{rationale_suffix}"
                    ));
                }
                if negotiation_goal_summary.is_none() {
                    negotiation_goal_summary = event
                        .payload
                        .get("negotiation_goal_summary")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if negotiation_resolution.is_none() {
                    negotiation_resolution = event
                        .payload
                        .get("negotiation_resolution")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if negotiation_acceptance_boundary.is_none() {
                    negotiation_acceptance_boundary = event
                        .payload
                        .get("negotiation_acceptance_boundary")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_summary.is_none() {
                    context_summary = event
                        .payload
                        .get("context_summary")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_credibility.is_none() {
                    context_credibility = event
                        .payload
                        .get("context_credibility")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
                if context_primary_inputs.is_empty() {
                    context_primary_inputs = event
                        .payload
                        .get("context_primary_inputs")
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if context_provenance.is_empty() {
                    context_provenance = event
                        .payload
                        .get("context_provenance")
                        .and_then(|value| value.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                if context_staleness_reason.is_none() {
                    context_staleness_reason = event
                        .payload
                        .get("context_staleness_reason")
                        .and_then(|value| value.as_str().map(str::to_string));
                }
            }
            TraceEventType::FlowInferred => {
                if let Some(flow_name) =
                    event.payload.get("flow_name").and_then(|value| value.as_str())
                {
                    decision_timeline.push(format!("flow_inferred: {flow_name}"));
                }
            }
            TraceEventType::DecisionCreated
            | TraceEventType::DecisionDispatched
            | TraceEventType::DecisionVerified
            | TraceEventType::DecisionFailed
            | TraceEventType::DecisionRecovered => {
                decision_timeline.extend(decision_timeline_lines(
                    event.event_type,
                    event.step_id.as_deref(),
                    &event.payload,
                ));
                if event.event_type == TraceEventType::DecisionFailed
                    && let Some(evidence) =
                        decision_failure_evidence(event.step_id.as_deref(), &event.payload)
                {
                    failure_evidence.push(evidence);
                }
            }
            TraceEventType::ProjectScalePathProposed
            | TraceEventType::ProjectScaleStageTransitioned
            | TraceEventType::VotingDecisionRecorded => {}
        }
    }

    if routing_summary.is_none() {
        routing_summary = Some(output::render_route_outcome(&RoutingOutcome {
            mode: if saw_native_routing_signal {
                RoutingMode::Native
            } else {
                RoutingMode::Compatibility
            },
            source: if saw_native_routing_signal {
                RoutingSource::GoalPlan
            } else {
                RoutingSource::ExecutionProfile
            },
            reason: if saw_native_routing_signal {
                "trace came from the session-native runtime".to_string()
            } else {
                "trace came from the explicit compatibility runtime".to_string()
            },
        }));
    }

    let (terminal_status, terminal_reason) =
        match (persisted_terminal_status, persisted_terminal_reason) {
            (Some(terminal_status), Some(terminal_reason)) => (terminal_status, terminal_reason),
            (None, None) => {
                if latest_governance_state.is_some() {
                    (
                        TaskStatus::Running,
                        synthesized_in_progress_reason(latest_governance_state.as_deref()),
                    )
                } else {
                    return Err(TraceSummaryError::MissingTerminalStatus);
                }
            }
            (None, Some(_)) => return Err(TraceSummaryError::MissingTerminalStatus),
            (Some(_), None) => return Err(TraceSummaryError::MissingTerminalReason),
        };

    Ok(TraceSummaryView {
        trace_ref: trace_ref.as_ref().to_string_lossy().into_owned(),
        goal: trace.goal.clone(),
        negotiation_goal_summary,
        negotiation_resolution,
        negotiation_acceptance_boundary,
        cluster_delivery_story,
        routing_summary,
        routing_projection,
        goal_plan_summary,
        authored_input_summary,
        authored_input_sources,
        authored_input_deduplicated_sources,
        context_summary,
        context_credibility,
        context_primary_inputs,
        context_provenance,
        context_staleness_reason,
        clarification_headline,
        clarification_prompt,
        clarification_missing_fields,
        requested_governance_runtime,
        requested_governance_risk,
        requested_governance_zone,
        requested_governance_owner,
        decision_timeline,
        failure_evidence,
        adaptive_evidence,
        latest_checkpoint_id,
        latest_checkpoint_scope,
        latest_checkpoint_restore_command,
        executed_steps,
        recovery_events,
        governance_timeline,
        governance_next_action: governance_next_action
            .or_else(|| governance_next_action_for_state(latest_governance_state.as_deref())),
        delegation,
        review_timeline,
        terminal_status,
        terminal_reason,
        duration: trace.duration_millis(),
    })
}

fn decision_timeline_lines(
    event_type: TraceEventType,
    decision_id: Option<&str>,
    payload: &serde_json::Value,
) -> Vec<String> {
    let decision_id = decision_id.unwrap_or("unknown-decision");
    let status = payload.get("status").and_then(|value| value.as_str()).unwrap_or("unknown");

    match event_type {
        TraceEventType::DecisionCreated => {
            let selector = payload.get("selector").and_then(|value| value.as_str());
            let decision_type =
                payload.get("decision_type").and_then(|value| value.as_str()).unwrap_or("unknown");
            let target =
                payload.get("target").and_then(|value| value.as_str()).unwrap_or("unknown");
            let mut lines = vec![match selector {
                Some(selector) => {
                    format!(
                        "decision: {decision_id} {selector} ({decision_type}) -> {target} [{status}]"
                    )
                }
                None => format!("decision: {decision_id} {decision_type} -> {target} [{status}]"),
            }];

            if let Some(selector) = selector {
                lines.push(format!("selector: {selector}"));
            }

            if let Some(rationale) = payload.get("rationale").and_then(|value| value.as_str()) {
                lines.push(format!("rationale: {rationale}"));
            }
            if let Some(expected_outcome) =
                payload.get("expected_outcome").and_then(|value| value.as_str())
            {
                lines.push(format!("expected_outcome: {expected_outcome}"));
                lines.push(format!("verification_intent: {expected_outcome}"));
            }
            if let Some(inputs) = payload.get("evidence_inputs").and_then(|value| value.as_array())
            {
                let inputs = inputs.iter().filter_map(format_evidence_input).collect::<Vec<_>>();
                if !inputs.is_empty() {
                    lines.push(format!("evidence_inputs: {}", inputs.join(", ")));
                }
            }

            lines
        }
        TraceEventType::DecisionDispatched => {
            vec![format!("decision_status: {decision_id} {status}")]
        }
        TraceEventType::DecisionVerified | TraceEventType::DecisionFailed => {
            vec![format!("decision_status: {decision_id} {status}")]
        }
        TraceEventType::DecisionRecovered => {
            let recovery_decision_id = payload
                .get("recovery_decision_id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown-decision");
            vec![format!("decision_status: {decision_id} {status} via {recovery_decision_id}")]
        }
        _ => Vec::new(),
    }
}

fn decision_failure_evidence(
    decision_id: Option<&str>,
    payload: &serde_json::Value,
) -> Option<String> {
    let decision_id = decision_id.unwrap_or(UNKNOWN_DECISION_ID);
    let target = payload.get(KEY_TARGET).and_then(|value| value.as_str()).unwrap_or(UNKNOWN_TARGET);
    let action_result = payload.get(KEY_ACTION_RESULT)?;
    let typed_result = serde_json::from_value::<ToolResult>(action_result.clone()).ok();
    let message = typed_result
        .as_ref()
        .and_then(|tool_result| {
            first_non_empty(&[Some(tool_result.stderr.as_str()), Some(tool_result.stdout.as_str())])
        })
        .or_else(|| {
            first_non_empty(&[
                action_result.get("stderr").and_then(|value| value.as_str()),
                action_result.get("stdout").and_then(|value| value.as_str()),
            ])
        })?;

    Some(format!("{decision_id} {target}: {message}"))
}

fn first_non_empty<'a>(values: &[Option<&'a str>]) -> Option<&'a str> {
    values.iter().filter_map(|value| *value).find(|value| !value.trim().is_empty())
}

fn format_evidence_input(value: &serde_json::Value) -> Option<String> {
    let kind = value.get("kind")?.as_str()?;
    let reference = value.get("reference")?.as_str()?;
    Some(format!("{kind}:{reference}"))
}

fn load_trace(
    trace: Option<&Path>,
    workspace: Option<&Path>,
) -> Result<(TraceResolutionTarget, PathBuf, ExecutionTrace), InspectCommandError> {
    let session_trace_ref = workspace.map(resolve_session_trace_ref).transpose()?.flatten();
    let (target, trace_path) = resolve_trace_path(trace, workspace, session_trace_ref.as_deref())?;

    let trace = match target {
        TraceResolutionTarget::LatestWorkspaceTrace => {
            let Some(workspace_path) = workspace else {
                return Err(InspectCommandError::MissingTraceReference);
            };
            let store = FileTraceStore::for_workspace(workspace_path);
            store.load(&trace_path)?
        }
        TraceResolutionTarget::ExplicitTrace | TraceResolutionTarget::SessionTraceRef => {
            let store = FileTraceStore::new(trace_path.parent().unwrap_or_else(|| Path::new(".")));
            store.load(&trace_path)?
        }
    };

    Ok((target, trace_path, trace))
}

fn resolve_session_trace_ref(workspace: &Path) -> Result<Option<String>, InspectCommandError> {
    match FileSessionStore::for_workspace(workspace).load() {
        Ok(Some(record)) => Ok(record.latest_trace_ref),
        Ok(None) => Ok(None),
        Err(SessionStoreError::InvalidRecord(message)) => Err(InspectCommandError::InvalidSession(
            format!("active session is invalid: {message}"),
        )),
        Err(error) => Err(InspectCommandError::SessionStore(error)),
    }
}

pub fn resolve_trace_path(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    session_trace_ref: Option<&str>,
) -> Result<(TraceResolutionTarget, PathBuf), InspectCommandError> {
    if let Some(trace_path) = trace {
        return Ok((TraceResolutionTarget::ExplicitTrace, trace_path.to_path_buf()));
    }

    if let Some(session_trace_ref) = session_trace_ref {
        return Ok((TraceResolutionTarget::SessionTraceRef, PathBuf::from(session_trace_ref)));
    }

    let Some(workspace_path) = workspace else {
        return Err(InspectCommandError::MissingTraceReference);
    };

    let store = FileTraceStore::for_workspace(workspace_path);
    let Some(trace_path) = store.latest()? else {
        return Err(InspectCommandError::MissingLatestTrace);
    };
    Ok((TraceResolutionTarget::LatestWorkspaceTrace, trace_path))
}

fn inspection_target_for(trace: Option<&Path>, workspace: Option<&Path>) -> TraceResolutionTarget {
    if trace.is_some() {
        TraceResolutionTarget::ExplicitTrace
    } else if workspace.is_some() {
        TraceResolutionTarget::LatestWorkspaceTrace
    } else {
        TraceResolutionTarget::ExplicitTrace
    }
}

fn corrected_command(inspection_target: TraceResolutionTarget) -> &'static str {
    match inspection_target {
        TraceResolutionTarget::ExplicitTrace | TraceResolutionTarget::SessionTraceRef => {
            "cargo run --bin boundline -- inspect --trace <trace>"
        }
        TraceResolutionTarget::LatestWorkspaceTrace => {
            "cargo run --bin boundline -- inspect --workspace <workspace>"
        }
    }
}

fn review_timeline_line(event_type: TraceEventType, payload: &serde_json::Value) -> Option<String> {
    match event_type {
        TraceEventType::ReviewStarted => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(|value| value.as_str())
            .map(|trigger| format!("review_trigger: {trigger}")),
        TraceEventType::ReviewTriggerIgnored => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(|value| value.as_str())
            .map(|trigger| format!("review_trigger_ignored: {trigger}")),
        TraceEventType::ReviewerCompleted => reviewer_line(payload),
        TraceEventType::ReviewVoteResolved => payload
            .get(KEY_SUMMARY)
            .and_then(|value| value.as_str())
            .map(|summary| format!("review_vote: {summary}"))
            .or_else(|| {
                payload.get(KEY_VOTE_RESOLUTION).map(|resolution| {
                    format!(
                        "review_vote: {}",
                        serde_json::to_string(resolution).unwrap_or_default()
                    )
                })
            }),
        TraceEventType::ReviewAdjudicated => {
            reviewer_line(payload).map(|line| format!("review_adjudication: {line}"))
        }
        TraceEventType::ReviewTerminalRecorded => payload
            .get(KEY_REVIEW_OUTCOME)
            .and_then(|value| value.as_str())
            .map(|outcome| format!("review_outcome: {outcome}"))
            .or_else(|| {
                payload
                    .get(KEY_FAILURE_REASON)
                    .and_then(|value| value.as_str())
                    .map(|reason| format!("review_reason: {reason}"))
            }),
        _ => None,
    }
}

fn governance_timeline_line(
    event_type: TraceEventType,
    payload: &serde_json::Value,
) -> Option<String> {
    match event_type {
        TraceEventType::GovernanceSelected => Some(format!(
            "governance_selected: {} -> {}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload
                .get("selected_runtime")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown-runtime")
        )),
        TraceEventType::GovernanceStarted => Some(format!(
            "governance_started: {}{}{}{}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload
                .get("canon_mode")
                .and_then(|value| value.as_str())
                .map(|mode| format!(" ({mode})"))
                .unwrap_or_default(),
            payload
                .get("run_ref")
                .and_then(|value| value.as_str())
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceDecisionRecorded => payload
            .get("selected_action")
            .and_then(|value| value.as_str())
            .map(|selected_action| format!("governance_decision: {selected_action}"))
            .or_else(|| {
                payload
                    .get("blocked_reason")
                    .and_then(|value| value.as_str())
                    .map(|reason| format!("governance_decision_blocked: {reason}"))
            }),
        TraceEventType::GovernanceAwaitingApproval => Some(format!(
            "governance_awaiting_approval: {} ({}){}{}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload.get("approval_state").and_then(|value| value.as_str()).unwrap_or("unknown"),
            payload
                .get("run_ref")
                .and_then(|value| value.as_str())
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceCompleted => Some(format!(
            "governance_completed: {}{}{}",
            payload
                .get("headline")
                .and_then(|value| value.as_str())
                .unwrap_or("governed packet ready"),
            payload
                .get("packet_ref")
                .and_then(|value| value.as_str())
                .map(|packet_ref| format!(" [{packet_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceBlocked => Some(format!(
            "governance_blocked: {}{}",
            payload.get("reason").and_then(|value| value.as_str()).unwrap_or("blocked"),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernancePacketRejected => Some(format!(
            "governance_packet_rejected: {}{}",
            payload.get("reason").and_then(|value| value.as_str()).unwrap_or("packet rejected"),
            governance_packet_provenance_suffix(payload)
        )),
        _ => None,
    }
}

fn governance_packet_provenance_suffix(payload: &serde_json::Value) -> String {
    governance_packet_provenance_text(
        payload.get("packet_source_stage").and_then(|value| value.as_str()),
        payload.get("packet_binding_reason").and_then(|value| value.as_str()),
    )
    .map(|provenance| format!(" from {provenance}"))
    .unwrap_or_default()
}

fn reviewer_line(payload: &serde_json::Value) -> Option<String> {
    let reviewer_id = payload.get(KEY_REVIEWER_ID).and_then(|value| value.as_str())?;

    if let Some(finding) = payload.get(KEY_FINDING) {
        let disposition =
            finding.get("disposition").and_then(|value| value.as_str()).unwrap_or("unknown");
        let summary =
            finding.get("summary").and_then(|value| value.as_str()).unwrap_or("review finding");
        let role = payload.get(KEY_REVIEWER_ROLE).and_then(|value| value.as_str());
        return Some(match role {
            Some(role) => format!("reviewer {reviewer_id} ({role}) {disposition}: {summary}"),
            None => format!("reviewer {reviewer_id} {disposition}: {summary}"),
        });
    }

    payload
        .get(KEY_FAILURE_REASON)
        .and_then(|value| value.as_str())
        .map(|reason| format!("reviewer {reviewer_id} failed: {reason}"))
}

fn synthesized_in_progress_reason(latest_governance_state: Option<&str>) -> TerminalReason {
    let message = match latest_governance_state {
        Some("awaiting_approval") => "governance approval is still pending",
        Some("blocked") => "governed work is blocked pending intervention",
        Some("governed_ready") => "governed work is ready for the next bounded step",
        _ => "trace is still in progress",
    };

    TerminalReason::new(TerminalCondition::NoCredibleNextStep, message, None)
}

fn success_headline(payload: &serde_json::Value, attempts: usize) -> String {
    let selection_reason = adaptive_selection_reason(payload);
    if let Some(headline) = payload
        .get("output")
        .and_then(|output| output.get("workspace_slice"))
        .and_then(|slice| slice.get("headline"))
        .and_then(|value| value.as_str())
    {
        return selection_reason.map_or_else(
            || format!("adaptive slice {headline}"),
            |reason| format!("adaptive slice {headline}: {reason}"),
        );
    }

    if let Some(change) = payload
        .get("output")
        .and_then(|output| output.get("change_evidence"))
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
    {
        let path = change.get("path").and_then(|value| value.as_str()).unwrap_or("workspace");
        let before =
            change.get("before_excerpt").and_then(|value| value.as_str()).unwrap_or("before");
        let after = change.get("after_excerpt").and_then(|value| value.as_str()).unwrap_or("after");
        return format!("updated {path} from {before} to {after} after {attempts} attempt(s)");
    }

    if let Some(changed_files) = payload
        .get("output")
        .and_then(|output| output.get("changed_files"))
        .and_then(|value| value.as_array())
    {
        let changed_files =
            changed_files.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
        if !changed_files.is_empty() {
            return format!("updated {} after {attempts} attempt(s)", changed_files.join(", "));
        }
    }

    if let Some(validation) = payload
        .get("output")
        .and_then(|output| output.get("validation"))
        .or_else(|| payload.get("evidence").and_then(|evidence| evidence.get("validation_record")))
    {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let succeeded =
            validation.get("succeeded").and_then(|value| value.as_bool()).unwrap_or(false);
        return format!(
            "validation {} after {attempts} attempt(s) via {command}",
            if succeeded { "passed" } else { "failed" }
        );
    }

    format!("succeeded after {attempts} attempt(s)")
}

fn failure_headline(payload: &serde_json::Value, attempts: usize) -> String {
    if let Some(exhaustion_reason) = payload
        .get("evidence")
        .and_then(|evidence| evidence.get("exhaustion_reason"))
        .and_then(|value| value.as_str())
    {
        return format!(
            "adaptive repair exhausted after {attempts} attempt(s): {exhaustion_reason}"
        );
    }

    if let Some(validation) =
        payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
    {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let exit_code = validation
            .get("exit_code")
            .and_then(|value| value.as_i64())
            .unwrap_or(UNKNOWN_VALIDATION_EXIT_CODE);
        return adaptive_selection_reason(payload).map_or_else(
            || {
                format!(
                    "validation failed after {attempts} attempt(s) via {command} (exit_code={exit_code})"
                )
            },
            |reason| {
                format!(
                    "validation failed after {attempts} attempt(s) via {command} (exit_code={exit_code}) while {reason}"
                )
            },
        );
    }

    format!("failed after {attempts} attempt(s)")
}

fn adaptive_selection_reason(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("output")
        .and_then(|output| output.get("selection_evidence"))
        .or_else(|| payload.get("evidence").and_then(|evidence| evidence.get("selection_evidence")))
        .and_then(|selection| selection.get("reason"))
        .and_then(|value| value.as_str().map(str::to_string))
}

fn adaptive_evidence_lines(payload: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    let selection =
        payload.get("output").and_then(|output| output.get("selection_evidence")).or_else(|| {
            payload.get("evidence").and_then(|evidence| evidence.get("selection_evidence"))
        });

    if let Some(selection) = selection {
        if let Some(candidate_family) =
            selection.get("candidate_family").and_then(|value| value.as_str())
        {
            lines.push(format!("candidate_family: {candidate_family}"));
        }

        if let Some(reason) = selection.get("reason").and_then(|value| value.as_str()) {
            lines.push(format!("selection_reason: {reason}"));
        }

        if let Some(rejected_candidates) =
            selection.get("rejected_candidates").and_then(|value| value.as_array())
        {
            lines.extend(
                rejected_candidates
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(|item| format!("rejected_candidate: {item}")),
            );
        }
    }

    if let Some(exhaustion_reason) = payload
        .get("evidence")
        .and_then(|evidence| evidence.get("exhaustion_reason"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("adaptive_exhaustion: {exhaustion_reason}"));
    }

    lines
}

fn parse_step_kind(raw: &str) -> Result<StepKind, TraceSummaryError> {
    match raw {
        "agent" => Ok(StepKind::Agent),
        "tool" => Ok(StepKind::Tool),
        "decision" => Ok(StepKind::Decision),
        other => Err(TraceSummaryError::UnknownStepKind(other.to_string())),
    }
}

#[derive(Debug, Error)]
pub enum InspectCommandError {
    #[error("inspect requires --trace or --workspace")]
    MissingTraceReference,
    #[error("no persisted trace could be found for the selected workspace")]
    MissingLatestTrace,
    #[error("failed to read the active session: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("{0}")]
    InvalidSession(String),
    #[error("failed to read the requested trace: {0}")]
    TraceStore(#[from] TraceStoreError),
    #[error("failed to summarize the requested trace: {0}")]
    Summary(#[from] TraceSummaryError),
}

#[derive(Debug, Error)]
pub enum TraceSummaryError {
    #[error("trace is missing a terminal status")]
    MissingTerminalStatus,
    #[error("trace is missing a terminal reason")]
    MissingTerminalReason,
    #[error("trace event {0:?} is missing a step id")]
    MissingStepId(TraceEventType),
    #[error("trace step '{0}' is missing its step kind payload")]
    MissingStepKind(String),
    #[error("trace step '{0}' completed without a matching start event")]
    MissingStartedStep(String),
    #[error("trace step kind '{0}' is unknown")]
    UnknownStepKind(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        InspectCommandError, TraceResolutionTarget, TraceSummaryError, adaptive_evidence_lines,
        corrected_command, decision_failure_evidence, failure_headline, governance_timeline_line,
        inspection_target_for, parse_step_kind, render_error, resolve_session_trace_ref,
        resolve_trace_path, review_timeline_line, success_headline, summarize_trace,
    };
    use crate::adapters::session_store::SessionStoreError;
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::task::{TaskStatus, TerminalReason};
    use crate::domain::trace::{ExecutionTrace, TraceEventType};

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        workspace
    }

    fn terminal_trace() -> ExecutionTrace {
        let mut trace = ExecutionTrace::new("task-inspect", "session-inspect", "Inspect trace");
        trace.terminal_status = Some(TaskStatus::Failed);
        trace.terminal_reason =
            Some(TerminalReason::new(TerminalCondition::UnrecoverableError, "failed", None));
        trace.ended_at = Some(trace.started_at + 1);
        trace
    }

    #[test]
    fn render_error_maps_session_store_and_summary_failures() {
        let workspace = PathBuf::from("/tmp/workspace");
        let session_error = InspectCommandError::SessionStore(SessionStoreError::Read(
            std::io::Error::other("read failed"),
        ));
        let session_text = render_error(None, Some(workspace.as_path()), &session_error);
        assert!(session_text.contains("failed to read the active session"), "{session_text}");

        let summary_error = InspectCommandError::Summary(TraceSummaryError::MissingTerminalStatus);
        let summary_text = render_error(None, Some(workspace.as_path()), &summary_error);
        assert!(summary_text.contains("failed to summarize the requested trace"), "{summary_text}");
    }

    #[test]
    fn summarize_trace_reports_missing_step_id_and_step_kind() {
        let mut missing_step_id = terminal_trace();
        missing_step_id.record_event(TraceEventType::StepStarted, None, 0, json!({}));
        assert!(matches!(
            summarize_trace("/tmp/trace.json", &missing_step_id).unwrap_err(),
            TraceSummaryError::MissingStepId(TraceEventType::StepStarted)
        ));

        let mut missing_step_kind = terminal_trace();
        missing_step_kind.record_event(
            TraceEventType::StepStarted,
            Some("verify".to_string()),
            0,
            json!({}),
        );
        assert!(matches!(
            summarize_trace("/tmp/trace.json", &missing_step_kind).unwrap_err(),
            TraceSummaryError::MissingStepKind(step_id) if step_id == "verify"
        ));
    }

    #[test]
    fn summarize_trace_surfaces_delegation_projection_from_payload() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            1,
            json!({
                "delegation": {
                    "mode": "handoff_required",
                    "packet_id": "packet-1",
                    "packet_kind": "handoff",
                    "packet_state": "active",
                    "target_owner": "codex",
                    "headline": "handoff required: implementation route cannot continue",
                    "evidence_summary": "claude lacks continuation support for implementation"
                }
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();
        let delegation = summary.delegation.unwrap();

        assert_eq!(delegation.mode.as_str(), "handoff_required");
        assert_eq!(delegation.packet_id.as_deref(), Some("packet-1"));
        assert_eq!(delegation.target_owner.as_deref(), Some("codex"));
        assert!(delegation.headline.contains("handoff required"));
    }

    #[test]
    fn summarize_trace_surfaces_checkpoint_projection() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::CheckpointCreated,
            None,
            1,
            json!({
                "checkpoint_id": "checkpoint-123",
                "checkpoint_scope": "workspace",
                "checkpoint_restore_command": "boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(summary.latest_checkpoint_id.as_deref(), Some("checkpoint-123"));
        assert_eq!(summary.latest_checkpoint_scope.as_deref(), Some("workspace"));
        assert_eq!(
            summary.latest_checkpoint_restore_command.as_deref(),
            Some("boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace")
        );
    }

    #[test]
    fn summarize_trace_ignores_project_scale_and_voting_projection_events() {
        let mut trace = terminal_trace();
        trace.record_event(TraceEventType::ProjectScalePathProposed, None, 0, json!({}));
        trace.record_event(TraceEventType::ProjectScaleStageTransitioned, None, 0, json!({}));
        trace.record_event(TraceEventType::VotingDecisionRecorded, None, 0, json!({}));

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(summary.trace_ref, "/tmp/trace.json");
        assert_eq!(summary.terminal_status, TaskStatus::Failed);
        assert_eq!(summary.goal, "Inspect trace");
    }

    #[test]
    fn resolve_session_trace_ref_maps_invalid_records_to_invalid_session_errors() {
        let workspace = temp_workspace("boundline-inspect-invalid-session");
        let invalid_record = ActiveSessionRecord {
            session_id: "session-inspect".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::GoalCaptured,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 20,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };
        fs::write(
            workspace.join(".boundline/session.json"),
            serde_json::to_vec_pretty(&invalid_record).unwrap(),
        )
        .unwrap();

        assert!(matches!(
            resolve_session_trace_ref(&workspace).unwrap_err(),
            InspectCommandError::InvalidSession(message) if message.contains("active session is invalid")
        ));
    }

    #[test]
    fn inspection_helpers_cover_default_headlines_and_command_targets() {
        assert_eq!(
            inspection_target_for(Some(PathBuf::from("trace.json").as_path()), None),
            TraceResolutionTarget::ExplicitTrace
        );
        assert_eq!(
            inspection_target_for(None, Some(PathBuf::from("workspace").as_path())),
            TraceResolutionTarget::LatestWorkspaceTrace
        );
        assert_eq!(
            corrected_command(TraceResolutionTarget::SessionTraceRef),
            "cargo run --bin boundline -- inspect --trace <trace>"
        );
        assert_eq!(success_headline(&json!({}), 2), "succeeded after 2 attempt(s)");
        assert_eq!(failure_headline(&json!({}), 1), "failed after 1 attempt(s)");
    }

    #[test]
    fn summarize_trace_collects_recovery_events_and_headlines_from_validation_payloads() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::StepStarted,
            Some("verify".to_string()),
            0,
            json!({"step_kind": "tool"}),
        );
        trace.record_event(
            TraceEventType::StepCompleted,
            Some("verify".to_string()),
            0,
            json!({
                "status": "failed",
                "evidence": {
                    "validation_record": {
                        "command": "cargo test --quiet",
                        "exit_code": 101,
                        "stdout": "",
                        "stderr": "",
                        "succeeded": false
                    }
                }
            }),
        );
        trace.record_event(
            TraceEventType::StageRetryScheduled,
            Some("verify".to_string()),
            0,
            json!({}),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(
            summary.executed_steps[0].headline,
            "validation failed after 1 attempt(s) via cargo test --quiet (exit_code=101)"
        );
        assert_eq!(summary.recovery_events[0].related_step_id.as_deref(), Some("verify"));
        assert_eq!(summary.recovery_events[0].trigger, "recovery event");

        let success = success_headline(
            &json!({
                "output": {
                    "validation": {
                        "command": "cargo test --quiet",
                        "succeeded": true
                    }
                }
            }),
            2,
        );
        assert_eq!(success, "validation passed after 2 attempt(s) via cargo test --quiet");

        let success_from_evidence = success_headline(
            &json!({
                "evidence": {
                    "validation_record": {
                        "command": "cargo test --quiet",
                        "succeeded": true
                    }
                }
            }),
            2,
        );
        assert_eq!(
            success_from_evidence,
            "validation passed after 2 attempt(s) via cargo test --quiet"
        );
    }

    #[test]
    fn summarize_trace_collects_review_timeline_lines() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::ReviewStarted,
            Some("review-safety".to_string()),
            0,
            json!({"review_trigger": "pr_ready"}),
        );
        trace.record_event(
            TraceEventType::ReviewerCompleted,
            Some("review-safety".to_string()),
            0,
            json!({
                "reviewer_id": "safety",
                "reviewer_role": "Safety",
                "finding": {
                    "disposition": "approve",
                    "summary": "No blockers"
                }
            }),
        );
        trace.record_event(
            TraceEventType::ReviewVoteResolved,
            Some("review-vote".to_string()),
            0,
            json!({"summary": "strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"}),
        );
        trace.record_event(
            TraceEventType::ReviewTerminalRecorded,
            Some("review-finalize".to_string()),
            0,
            json!({"review_outcome": "accepted"}),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(
            summary.review_timeline,
            vec![
                "review_trigger: pr_ready".to_string(),
                "reviewer safety (Safety) approve: No blockers".to_string(),
                "review_vote: strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"
                    .to_string(),
                "review_outcome: accepted".to_string(),
            ]
        );
    }

    #[test]
    fn summarize_trace_synthesizes_running_summary_for_paused_governance_traces() {
        let mut trace = ExecutionTrace::new("task-inspect", "session-inspect", "Inspect trace");
        trace.record_event(
            TraceEventType::GovernanceStarted,
            Some("investigate".to_string()),
            0,
            json!({
                "stage_key": "bug-fix:investigate",
                "canon_mode": "discovery"
            }),
        );
        trace.record_event(
            TraceEventType::GovernanceBlocked,
            Some("investigate".to_string()),
            0,
            json!({
                "stage_key": "bug-fix:investigate",
                "reason": "governance blocked stage bug-fix:investigate"
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(summary.terminal_status, TaskStatus::Running);
        assert_eq!(
            summary.terminal_reason.message,
            "governed work is blocked pending intervention"
        );
        assert!(
            summary
                .governance_timeline
                .iter()
                .any(|line| { line == "governance_started: bug-fix:investigate (discovery)" })
        );
        assert_eq!(
            summary.governance_next_action.as_deref(),
            Some("resolve the governance blocker, then rerun boundline step")
        );
    }

    #[test]
    fn summarize_trace_collects_context_and_requested_governance_projection() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            0,
            json!({
                "input": {
                    "authored_input_summary": "Need bounded context evidence",
                    "authored_input_sources": ["brief.md"],
                    "authored_input_deduplicated_sources": ["brief.md"],
                    "clarification_headline": "clarification required: narrow the goal",
                    "clarification_prompt": "pick one bounded outcome",
                    "clarification_missing_fields": ["bounded_outcome"],
                    "requested_governance_runtime": "canon",
                    "requested_governance_risk": "high",
                    "requested_governance_zone": "payments",
                    "requested_governance_owner": "platform",
                    "negotiation_goal_summary": "ship the bounded context slice"
                }
            }),
        );
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            0,
            json!({
                "task_count": 1,
                "goal": "Inspect trace",
                "goal_plan_state": "proposed",
                "goal_plan_revision": 2,
                "flow_state": "proposed (bug-fix) - evidence suggests bug-fix because selected targets span existing tests and source files",
                "planning_rationale": "replan revision 2 supersedes revision 1 because flow, verification strategy",
                "verification_strategy": "run targeted verification against tests/red_to_green.rs",
                "negotiation_resolution": "credible",
                "negotiation_acceptance_boundary": "deliver the bounded outcome",
                "context_summary": "bounded context from src/lib.rs",
                "context_credibility": "stale",
                "context_primary_inputs": ["src/lib.rs"],
                "context_provenance": ["workspace_file: src/lib.rs (failing test target) [source=symbol_scan]"],
                "context_staleness_reason": "trace snapshot is stale"
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(
            summary.authored_input_summary.as_deref(),
            Some("Need bounded context evidence")
        );
        assert_eq!(summary.authored_input_sources, vec!["brief.md".to_string()]);
        assert_eq!(summary.authored_input_deduplicated_sources, vec!["brief.md".to_string()]);
        assert_eq!(
            summary.clarification_headline.as_deref(),
            Some("clarification required: narrow the goal")
        );
        assert_eq!(summary.clarification_prompt.as_deref(), Some("pick one bounded outcome"));
        assert_eq!(summary.clarification_missing_fields, vec!["bounded_outcome".to_string()]);
        assert_eq!(summary.requested_governance_runtime.as_deref(), Some("canon"));
        assert_eq!(summary.requested_governance_risk.as_deref(), Some("high"));
        assert_eq!(summary.requested_governance_zone.as_deref(), Some("payments"));
        assert_eq!(summary.requested_governance_owner.as_deref(), Some("platform"));
        assert_eq!(
            summary.negotiation_goal_summary.as_deref(),
            Some("ship the bounded context slice")
        );
        assert_eq!(summary.negotiation_resolution.as_deref(), Some("credible"));
        assert_eq!(
            summary.negotiation_acceptance_boundary.as_deref(),
            Some("deliver the bounded outcome")
        );
        assert_eq!(summary.context_summary.as_deref(), Some("bounded context from src/lib.rs"));
        assert_eq!(summary.context_credibility.as_deref(), Some("stale"));
        assert!(summary.goal_plan_summary.as_deref().unwrap().contains("[proposed rev 2]"));
        assert!(
            summary
                .goal_plan_summary
                .as_deref()
                .unwrap()
                .contains("verification: run targeted verification against tests/red_to_green.rs")
        );
        assert_eq!(summary.context_primary_inputs, vec!["src/lib.rs".to_string()]);
        assert_eq!(
            summary.context_provenance,
            vec![
                "workspace_file: src/lib.rs (failing test target) [source=symbol_scan]".to_string()
            ]
        );
        assert_eq!(summary.context_staleness_reason.as_deref(), Some("trace snapshot is stale"));
        assert!(
            summary.routing_summary.as_deref().unwrap().contains("routing: native (goal_plan)"),
            "{:?}",
            summary.routing_summary
        );
    }

    #[test]
    fn summarize_trace_surfaces_canon_memory_from_governance_events() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::GovernanceBlocked,
            Some("step-1".to_string()),
            0,
            json!({
                "stage_key": "change:verify",
                "runtime": "canon",
                "required": true,
                "reason": "refresh_required",
                "run_ref": "run-7",
                "packet_ref": ".canon/runs/run-7",
                "document_refs": [".canon/runs/run-7/verification.md"],
                "canon_memory_summary": "Canon verification packet [stale]",
                "canon_memory_credibility": "stale",
                "canon_memory_compatibility": "warning",
                "canon_memory_reason_code": "refresh_required",
                "canon_next_action": "refresh: refresh the governed packet and reassess its credibility"
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(summary.context_summary.as_deref(), Some("Canon verification packet [stale]"));
        assert_eq!(summary.context_credibility.as_deref(), Some("stale"));
        assert_eq!(
            summary.context_primary_inputs,
            vec![".canon/runs/run-7/verification.md".to_string()]
        );
        assert!(
            summary
                .context_provenance
                .contains(&"canon_memory: Canon verification packet [stale]".to_string())
        );
        assert!(
            summary.context_provenance.contains(&"canon_memory_compatibility: warning".to_string())
        );
        assert!(summary.context_provenance.contains(&"canon_memory_run_ref: run-7".to_string()));
        assert!(
            summary
                .context_provenance
                .contains(&"canon_memory_packet: .canon/runs/run-7".to_string())
        );
        assert_eq!(summary.context_staleness_reason.as_deref(), Some("refresh_required"));
        assert_eq!(
            summary.governance_next_action.as_deref(),
            Some("refresh: refresh the governed packet and reassess its credibility")
        );
    }

    #[test]
    fn decision_failure_evidence_falls_back_to_raw_action_result_fields() {
        let payload = json!({
            "target": "src/lib.rs",
            "action_result": {
                "stderr": "compiler exploded"
            }
        });

        assert_eq!(
            decision_failure_evidence(Some("decision-raw"), &payload),
            Some("decision-raw src/lib.rs: compiler exploded".to_string())
        );
    }

    #[test]
    fn inspect_helper_functions_cover_resolution_review_governance_and_adaptive_fallbacks() {
        assert!(matches!(
            resolve_trace_path(None, None, None).unwrap_err(),
            InspectCommandError::MissingTraceReference
        ));
        assert_eq!(
            resolve_trace_path(None, Some(Path::new("/tmp/workspace")), Some("/tmp/trace.json"))
                .unwrap()
                .0,
            TraceResolutionTarget::SessionTraceRef
        );
        assert!(matches!(
            parse_step_kind("mystery").unwrap_err(),
            TraceSummaryError::UnknownStepKind(kind) if kind == "mystery"
        ));

        assert_eq!(
            review_timeline_line(
                TraceEventType::ReviewTriggerIgnored,
                &json!({"review_trigger": "manual"}),
            ),
            Some("review_trigger_ignored: manual".to_string())
        );
        assert_eq!(
            review_timeline_line(
                TraceEventType::ReviewVoteResolved,
                &json!({"vote_resolution": {"decision": "accepted"}}),
            )
            .unwrap(),
            "review_vote: {\"decision\":\"accepted\"}"
        );
        assert_eq!(
            review_timeline_line(
                TraceEventType::ReviewAdjudicated,
                &json!({
                    "reviewer_id": "safety",
                    "finding": {"disposition": "approve", "summary": "No blockers"}
                }),
            ),
            Some("review_adjudication: reviewer safety approve: No blockers".to_string())
        );
        assert_eq!(
            review_timeline_line(
                TraceEventType::ReviewTerminalRecorded,
                &json!({"failure_reason": "timed out"}),
            ),
            Some("review_reason: timed out".to_string())
        );

        assert_eq!(
            governance_timeline_line(
                TraceEventType::GovernanceDecisionRecorded,
                &json!({"blocked_reason": "needs approval"}),
            ),
            Some("governance_decision_blocked: needs approval".to_string())
        );
        assert_eq!(
            governance_timeline_line(
                TraceEventType::GovernanceAwaitingApproval,
                &json!({
                    "stage_key": "bug-fix:implement",
                    "approval_state": "requested",
                    "run_ref": "canon-run-1",
                    "packet_source_stage": "bug-fix:investigate",
                    "packet_binding_reason": "upstream_stage_context"
                }),
            ),
            Some(
                "governance_awaiting_approval: bug-fix:implement (requested) [canon-run-1] from bug-fix:investigate (upstream_stage_context)"
                    .to_string(),
            )
        );
        assert_eq!(
            governance_timeline_line(
                TraceEventType::GovernanceCompleted,
                &json!({"packet_ref": ".canon/runs/canon-run-1"}),
            ),
            Some(
                "governance_completed: governed packet ready [.canon/runs/canon-run-1]".to_string()
            )
        );
        assert_eq!(
            governance_timeline_line(TraceEventType::GovernanceBlocked, &json!({})),
            Some("governance_blocked: blocked".to_string())
        );
        assert_eq!(
            governance_timeline_line(TraceEventType::GovernancePacketRejected, &json!({})),
            Some("governance_packet_rejected: packet rejected".to_string())
        );

        assert_eq!(
            adaptive_evidence_lines(&json!({
                "output": {
                    "selection_evidence": {
                        "candidate_family": "source",
                        "reason": "goal keywords matched src/lib.rs",
                        "rejected_candidates": ["tests/red.rs"]
                    }
                },
                "evidence": {
                    "exhaustion_reason": "limits exhausted"
                }
            })),
            vec![
                "candidate_family: source".to_string(),
                "selection_reason: goal keywords matched src/lib.rs".to_string(),
                "rejected_candidate: tests/red.rs".to_string(),
                "adaptive_exhaustion: limits exhausted".to_string(),
            ]
        );

        assert_eq!(
            success_headline(
                &json!({
                    "output": {
                        "change_evidence": [{
                            "path": "src/lib.rs",
                            "before_excerpt": "left - right",
                            "after_excerpt": "left + right"
                        }]
                    }
                }),
                2,
            ),
            "updated src/lib.rs from left - right to left + right after 2 attempt(s)"
        );
        assert_eq!(
            failure_headline(
                &json!({
                    "evidence": {
                        "exhaustion_reason": "limits exhausted"
                    }
                }),
                3,
            ),
            "adaptive repair exhausted after 3 attempt(s): limits exhausted"
        );
    }
}
