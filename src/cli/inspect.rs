//! Trace inspection and summary rehydration for operator-facing CLI output.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::cluster::ClusterDeliveryStory;
use crate::domain::context_intelligence::AdvancedContextProjection;
use crate::domain::goal_plan::GoalPlanFlowState;
use crate::domain::guidance::{GuardianFinding, GuidanceGuardianProjection};
use crate::domain::limits::TerminalCondition;
use crate::domain::reasoning::ProfileActivationRecord;
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::{DelightFeedbackSignal, RoutingMode, RoutingOutcome, RoutingSource};
use crate::domain::session::{
    governance_next_action_for_state, governance_packet_provenance_text, session_goal_brief_ref,
    session_plan_brief_ref, session_run_brief_ref,
};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskStatus, TerminalReason};
use crate::domain::tool_result::ToolResult;
use crate::domain::trace::{
    ExecutionTrace, InspectClosureKind, InspectClosureView, TraceEventType, TraceRecoveryEvent,
    TraceStepSummary, TraceSummaryView,
};

const UNKNOWN_VALIDATION_EXIT_CODE: i64 = -1;
const UNKNOWN_DECISION_ID: &str = "unknown-decision";
const UNKNOWN_TARGET: &str = "unknown";
const KEY_ACTION_RESULT: &str = "action_result";
const KEY_FAILURE_REASON: &str = "failure_reason";

fn advanced_context_from_payload(payload: &Value) -> Option<AdvancedContextProjection> {
    payload.get("advanced_context").cloned().and_then(|value| serde_json::from_value(value).ok())
}
const KEY_FINDING: &str = "finding";
const KEY_DELIGHT_FEEDBACK: &str = "delight_feedback";
const KEY_REVIEW_OUTCOME: &str = "review_outcome";
const KEY_REVIEW_TRIGGER: &str = "review_trigger";
const KEY_REVIEWER_ID: &str = "reviewer_id";
const KEY_REVIEWER_ROLE: &str = "reviewer_role";
const KEY_SUMMARY: &str = "summary";
const KEY_TARGET: &str = "target";
const KEY_VOTE_RESOLUTION: &str = "vote_resolution";

fn trace_workspace_root(trace_ref: &Path) -> Option<PathBuf> {
    let mut current = trace_ref.parent()?;
    loop {
        if current.file_name().and_then(|name| name.to_str()) == Some(".boundline") {
            return current.parent().map(Path::to_path_buf);
        }
        current = current.parent()?;
    }
}

fn persisted_session_brief_ref(workspace: &Path, brief_ref: &str) -> Option<String> {
    workspace.join(brief_ref).is_file().then(|| brief_ref.to_string())
}

/// Result returned by `inspect` after loading a trace, summarizing it, and
/// rendering the terminal-facing output.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub inspection_target: Option<String>,
    pub trace_location: Option<String>,
    pub trace_summary: Option<TraceSummaryView>,
}

/// Source used to resolve which trace `inspect` should open.
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

/// Loads the requested trace, summarizes persisted execution state, and renders
/// the same flattened operator view used by the CLI.
pub fn execute_inspect(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    session_id: Option<&str>,
) -> Result<InspectCommandReport, InspectCommandError> {
    let (inspection_target, trace_ref, trace) = load_trace(trace, workspace, session_id)?;
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
        inspection_target: Some(inspection_target.as_str().to_string()),
        trace_location: Some(trace_ref.to_string_lossy().into_owned()),
        trace_summary: Some(summary),
    })
}

/// Renders a user-facing inspect failure using the same target-resolution rules
/// as the successful path.
pub fn render_error(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    session_id: Option<&str>,
    error: &InspectCommandError,
) -> String {
    if let InspectCommandError::UnknownSession(selected_session_id) = error {
        let session_scope =
            workspace.map(|path| format!(" in {}", path.display())).unwrap_or_default();
        return output::render_session_error(
            "inspect",
            &format!("session `{selected_session_id}` does not exist{session_scope}"),
            Some("boundline session list"),
        );
    }

    if let InspectCommandError::InvalidSession(message) = error {
        let next_command = if session_id.is_some() {
            Some("boundline session list")
        } else {
            Some("boundline goal --goal <goal>")
        };
        return output::render_session_error("inspect", message, next_command);
    }

    let inspection_target = inspection_target_for(trace, workspace);
    let trace_ref = trace.map(|path| path.to_string_lossy().into_owned());
    let workspace_ref = workspace.map(|path| path.to_string_lossy().into_owned());
    let terminal_reason = match error {
        InspectCommandError::MissingTraceReference => "inspect requires --trace or --workspace",
        InspectCommandError::MissingLatestTrace | InspectCommandError::TraceStore(_) => {
            "failed to read the requested trace"
        }
        InspectCommandError::SessionStore(_) => {
            if session_id.is_some() {
                "failed to read the selected session"
            } else {
                "failed to read the active session"
            }
        }
        InspectCommandError::UnknownSession(_) => "selected session does not exist",
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

/// Projects routing and optional flow state into the compact summary lines used
/// by inspect-style output surfaces.
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

/// Rehydrates a persisted trace into the flattened `TraceSummaryView` consumed
/// by CLI output. The function prefers persisted projection fields over
/// recomputation so inspect explains exactly what the run recorded.
pub fn summarize_trace(
    trace_ref: impl AsRef<Path>,
    trace: &ExecutionTrace,
) -> Result<TraceSummaryView, TraceSummaryError> {
    let persisted_terminal_status = trace.terminal_status;
    let persisted_terminal_reason = trace.terminal_reason.clone();
    let mut input_projection = TraceInputProjection::default();
    let mut cluster_delivery_story: Option<ClusterDeliveryStory> = None;
    let mut routing_summary: Option<String> = None;
    let mut routing_projection = RoutingDecisionProjection::default();
    let mut goal_plan_summary: Option<String> = None;
    let mut advanced_context: Option<AdvancedContextProjection> = None;
    let mut context_projection = TraceContextProjection::default();
    let mut guidance_guardian = GuidanceGuardianProjection::default();
    let mut governance_projection = TraceGovernanceProjection::default();
    let mut decision_timeline: Vec<String> = Vec::new();
    let mut failure_evidence: Vec<String> = Vec::new();
    let mut adaptive_evidence: Vec<String> = Vec::new();
    let mut latest_checkpoint_id: Option<String> = None;
    let mut latest_checkpoint_scope: Option<String> = None;
    let mut latest_checkpoint_restore_command: Option<String> = None;
    let mut delegation: Option<crate::domain::session::DelegationStatusView> = None;
    let mut saw_native_routing_signal = false;
    let mut step_indexes: HashMap<String, usize> = HashMap::new();
    let mut executed_steps: Vec<TraceStepSummary> = Vec::new();
    let mut recovery_events: Vec<TraceRecoveryEvent> = Vec::new();
    let mut review_timeline: Vec<String> = Vec::new();
    let mut reasoning_profile: Option<ProfileActivationRecord> = None;
    let mut delight_feedback: Option<DelightFeedbackSignal> = None;

    for event in &trace.events {
        if let Some(signal) = event
            .payload
            .get(KEY_DELIGHT_FEEDBACK)
            .cloned()
            .and_then(|value| serde_json::from_value::<DelightFeedbackSignal>(value).ok())
            .filter(|signal| signal.validate().is_ok())
        {
            delight_feedback = Some(signal);
        }

        if let Some(record) = event
            .payload
            .get("reasoning_profile_record")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
        {
            reasoning_profile = Some(record);
        }

        // Guidance and guardian projection is persisted incrementally across
        // planning, execution, and verification events; inspect rebuilds the
        // latest authoritative view by folding those payload snapshots.
        merge_guidance_projection_from_payload(&mut guidance_guardian, &event.payload);

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
                input_projection.merge_task_started_payload(&event.payload);
                context_projection.merge_task_started_payload(&event.payload);
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
                governance_projection.merge_event(
                    event.event_type,
                    &event.payload,
                    &mut context_projection,
                );
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
                    advanced_context =
                        advanced_context.or_else(|| advanced_context_from_payload(&event.payload));
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
                input_projection.merge_goal_plan_payload(&event.payload);
                context_projection.merge_goal_plan_payload(&event.payload);
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
            TraceEventType::ReasoningProfileActivated
            | TraceEventType::ReasoningParticipantStarted
            | TraceEventType::ReasoningParticipantCompleted
            | TraceEventType::ReasoningDisagreementRecorded
            | TraceEventType::ReasoningDebateRoundCompleted
            | TraceEventType::ReasoningReflexionRevisionCompleted
            | TraceEventType::ReasoningAdjudicationRecorded
            | TraceEventType::ReasoningConfidenceRecorded
            | TraceEventType::ReasoningProfileBlocked
            | TraceEventType::ReasoningProfileInterrupted
            | TraceEventType::ReasoningProfileEscalated => {}
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
                if governance_projection.latest_state.is_some() {
                    (
                        TaskStatus::Running,
                        synthesized_in_progress_reason(
                            governance_projection.latest_state.as_deref(),
                        ),
                    )
                } else {
                    return Err(TraceSummaryError::MissingTerminalStatus);
                }
            }
            (None, Some(_)) => return Err(TraceSummaryError::MissingTerminalStatus),
            (Some(_), None) => return Err(TraceSummaryError::MissingTerminalReason),
        };

    let governance_timeline = governance_projection.timeline;
    let governance_runtime_state = governance_projection.runtime_state;
    let governance_rollout_profile = governance_projection.rollout_profile;
    let governance_reason = governance_projection.reason;
    let governance_approval_provenance = governance_projection.approval_provenance;
    let governance_next_action = governance_projection.next_action.or_else(|| {
        governance_next_action_for_state(governance_projection.latest_state.as_deref())
    });
    let terminal_projection = InspectTerminalProjection {
        terminal_status,
        terminal_reason: &terminal_reason,
        next_action: governance_next_action.as_deref(),
    };
    let inspect_context = Some(build_inspect_context_view(
        context_projection.summary.as_deref(),
        context_projection.credibility.as_deref(),
        &context_projection.primary_inputs,
        &context_projection.provenance,
        context_projection.staleness_reason.as_deref(),
        terminal_projection,
    ));
    let inspect_council = Some(build_inspect_council_view(
        &review_timeline,
        &governance_timeline,
        reasoning_profile.as_ref(),
        terminal_projection,
    ));
    let inspect_timeline = Some(build_inspect_timeline_view(
        &decision_timeline,
        &review_timeline,
        &governance_timeline,
        &executed_steps,
        &recovery_events,
        terminal_projection,
    ));
    let workspace_root = trace_workspace_root(trace_ref.as_ref());
    let goal_brief_ref = workspace_root.as_ref().and_then(|workspace| {
        persisted_session_brief_ref(workspace, &session_goal_brief_ref(&trace.session_id))
    });
    let session_plan_brief_ref = workspace_root.as_ref().and_then(|workspace| {
        persisted_session_brief_ref(workspace, &session_plan_brief_ref(&trace.session_id))
    });
    let run_brief_ref = workspace_root.as_ref().and_then(|workspace| {
        persisted_session_brief_ref(workspace, &session_run_brief_ref(&trace.session_id))
    });

    Ok(TraceSummaryView {
        trace_ref: trace_ref.as_ref().to_string_lossy().into_owned(),
        goal: trace.goal.clone(),
        trace_started_at: Some(trace.started_at),
        advanced_context,
        negotiation_goal_summary: input_projection.negotiation_goal_summary,
        negotiation_resolution: input_projection.negotiation_resolution,
        negotiation_acceptance_boundary: input_projection.negotiation_acceptance_boundary,
        cluster_delivery_story,
        routing_summary,
        routing_projection,
        goal_plan_summary,
        authored_input_summary: input_projection.authored_input_summary,
        authored_input_sources: input_projection.authored_input_sources,
        authored_input_deduplicated_sources: input_projection.authored_input_deduplicated_sources,
        goal_brief_ref,
        session_plan_brief_ref,
        run_brief_ref,
        context_summary: context_projection.summary,
        context_credibility: context_projection.credibility,
        context_primary_inputs: context_projection.primary_inputs,
        context_provenance: context_projection.provenance,
        context_staleness_reason: context_projection.staleness_reason,
        guidance_guardian,
        clarification_headline: input_projection.clarification_headline,
        clarification_prompt: input_projection.clarification_prompt,
        clarification_missing_fields: input_projection.clarification_missing_fields,
        requested_governance_runtime: input_projection.requested_governance_runtime,
        requested_governance_risk: input_projection.requested_governance_risk,
        requested_governance_zone: input_projection.requested_governance_zone,
        requested_governance_owner: input_projection.requested_governance_owner,
        decision_timeline,
        failure_evidence,
        adaptive_evidence,
        latest_checkpoint_id,
        latest_checkpoint_scope,
        latest_checkpoint_restore_command,
        executed_steps,
        recovery_events,
        governance_timeline,
        governance_runtime_state,
        governance_rollout_profile,
        governance_reason,
        governance_approval_provenance,
        governance_next_action,
        reasoning_profile,
        delegation,
        inspect_context,
        inspect_council,
        inspect_timeline,
        review_timeline,
        delight_feedback,
        terminal_status,
        terminal_reason,
        duration: trace.duration_millis(),
    })
}

#[derive(Clone, Copy)]
struct InspectTerminalProjection<'a> {
    terminal_status: TaskStatus,
    terminal_reason: &'a TerminalReason,
    next_action: Option<&'a str>,
}

fn build_inspect_context_view(
    context_summary: Option<&str>,
    context_credibility: Option<&str>,
    context_primary_inputs: &[String],
    context_provenance: &[String],
    context_staleness_reason: Option<&str>,
    terminal: InspectTerminalProjection<'_>,
) -> InspectClosureView {
    let mut narrative_lines = Vec::new();
    if let Some(context_summary) = context_summary {
        narrative_lines.push(format!("context_summary: {context_summary}"));
    }
    if let Some(context_credibility) = context_credibility {
        narrative_lines.push(format!("context_credibility: {context_credibility}"));
    }
    if !context_primary_inputs.is_empty() {
        narrative_lines
            .push(format!("context_primary_inputs: {}", context_primary_inputs.join(", ")));
    }
    narrative_lines.extend(context_provenance.iter().cloned());
    if let Some(context_staleness_reason) = context_staleness_reason {
        narrative_lines.push(format!("context_staleness_reason: {context_staleness_reason}"));
    }

    let mut missing_inputs = Vec::new();
    if context_summary.is_none() {
        missing_inputs.push("context_summary".to_string());
    }
    if context_primary_inputs.is_empty() {
        missing_inputs.push("context_primary_inputs".to_string());
    }
    if context_provenance.is_empty() {
        missing_inputs.push("context_provenance".to_string());
    }

    let headline = if let Some(context_summary) = context_summary {
        context_summary.to_string()
    } else if let Some(context_staleness_reason) = context_staleness_reason {
        format!("context needs refresh: {context_staleness_reason}")
    } else {
        "context evidence is not yet available from the authoritative trace".to_string()
    };

    InspectClosureView {
        view_kind: InspectClosureKind::Context,
        headline,
        narrative_lines,
        source_attribution: context_provenance.to_vec(),
        missing_inputs,
        terminal_status: terminal.terminal_status,
        terminal_reason: terminal.terminal_reason.message.clone(),
        next_action: terminal.next_action.map(str::to_string),
    }
}

fn build_inspect_council_view(
    review_timeline: &[String],
    _governance_timeline: &[String],
    reasoning_profile: Option<&ProfileActivationRecord>,
    terminal: InspectTerminalProjection<'_>,
) -> InspectClosureView {
    let mut narrative_lines = review_timeline.to_vec();
    if let Some(reasoning_profile) = reasoning_profile {
        narrative_lines.push(format!(
            "reasoning_profile: {} ({})",
            reasoning_profile.profile_id,
            reasoning_profile.status.as_str()
        ));
    }

    let mut source_attribution = Vec::new();
    if !review_timeline.is_empty() {
        source_attribution.push("review_timeline".to_string());
    }
    if reasoning_profile.is_some() {
        source_attribution.push("reasoning_profile".to_string());
    }

    let mut missing_inputs = Vec::new();
    if review_timeline.is_empty() {
        missing_inputs.push("review_timeline".to_string());
    }

    let headline = if !review_timeline.is_empty() || reasoning_profile.is_some() {
        "council activity was recorded for this trace".to_string()
    } else {
        "no council activity was recorded".to_string()
    };

    InspectClosureView {
        view_kind: InspectClosureKind::Council,
        headline,
        narrative_lines,
        source_attribution,
        missing_inputs,
        terminal_status: terminal.terminal_status,
        terminal_reason: terminal.terminal_reason.message.clone(),
        next_action: terminal.next_action.map(str::to_string),
    }
}

fn build_inspect_timeline_view(
    decision_timeline: &[String],
    review_timeline: &[String],
    governance_timeline: &[String],
    executed_steps: &[TraceStepSummary],
    recovery_events: &[TraceRecoveryEvent],
    terminal: InspectTerminalProjection<'_>,
) -> InspectClosureView {
    let mut narrative_lines = Vec::new();
    narrative_lines.extend(decision_timeline.iter().cloned());
    narrative_lines.extend(review_timeline.iter().cloned());
    narrative_lines.extend(governance_timeline.iter().cloned());
    narrative_lines.extend(executed_steps.iter().map(step_timeline_line));
    narrative_lines.extend(recovery_events.iter().map(recovery_timeline_line));

    let mut missing_inputs = Vec::new();
    if decision_timeline.is_empty()
        && review_timeline.is_empty()
        && governance_timeline.is_empty()
        && executed_steps.is_empty()
        && recovery_events.is_empty()
    {
        missing_inputs.push("decision_review_governance_step_recovery_timeline".to_string());
    }

    let headline = if narrative_lines.is_empty() {
        "timeline details are not yet available for this trace".to_string()
    } else {
        format!("timeline preserves {} recorded transition(s)", narrative_lines.len())
    };

    let source_attribution = vec![
        "decision_timeline".to_string(),
        "review_timeline".to_string(),
        "governance_timeline".to_string(),
        "executed_steps".to_string(),
        "recovery_events".to_string(),
    ];

    InspectClosureView {
        view_kind: InspectClosureKind::Timeline,
        headline,
        narrative_lines,
        source_attribution,
        missing_inputs,
        terminal_status: terminal.terminal_status,
        terminal_reason: terminal.terminal_reason.message.clone(),
        next_action: terminal.next_action.map(str::to_string),
    }
}

fn step_timeline_line(step: &TraceStepSummary) -> String {
    format!(
        "step: {} ({}) {} [{} attempt(s)] - {}",
        step.step_id,
        step_kind_label(step.step_kind),
        step_status_label(step.final_status),
        step.attempts,
        step.headline,
    )
}

fn recovery_timeline_line(event: &TraceRecoveryEvent) -> String {
    let label = match event.event_type {
        TraceEventType::RetryScheduled => "retry",
        TraceEventType::StageRetryScheduled => "stage_retry",
        TraceEventType::Replanned => "replan",
        TraceEventType::StageReplanned => "stage_replan",
        TraceEventType::FlowSelected => "flow",
        TraceEventType::StageTransitioned => "stage",
        TraceEventType::StageFailed => "stage_failure",
        _ => "recovery",
    };
    match event.related_step_id.as_deref() {
        Some(step_id) => format!("{label}: {} [{step_id}]", event.trigger),
        None => format!("{label}: {}", event.trigger),
    }
}

fn step_kind_label(step_kind: StepKind) -> &'static str {
    match step_kind {
        StepKind::Agent => "agent",
        StepKind::Tool => "tool",
        StepKind::Decision => "decision",
    }
}

fn step_status_label(step_status: StepStatus) -> &'static str {
    match step_status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
    }
}

// Fold one event payload into the flattened guidance/guardian projection.
// Planning-era fields latch on first value; execution-era fields are refreshed
// whenever a later event publishes a newer non-empty snapshot.
fn merge_guidance_projection_from_payload(
    projection: &mut GuidanceGuardianProjection,
    payload: &Value,
) {
    let Some(object) = payload.as_object() else {
        return;
    };

    if projection.capability_resolution_summary.is_none() {
        projection.capability_resolution_summary = object
            .get("capability_resolution_summary")
            .and_then(|value| value.as_str().map(str::to_string));
    }
    if projection.loaded_packs.is_empty() {
        projection.loaded_packs = string_array_field(object, "loaded_packs");
    }
    if projection.skipped_packs.is_empty() {
        projection.skipped_packs = string_array_field(object, "skipped_packs");
    }
    if projection.catalog_validation_findings.is_empty() {
        projection.catalog_validation_findings =
            string_array_field(object, "catalog_validation_findings");
    }
    if projection.loaded_guidance_sources.is_empty() {
        projection.loaded_guidance_sources = string_array_field(object, "loaded_guidance_sources");
    }
    if projection.skipped_guidance_sources.is_empty() {
        projection.skipped_guidance_sources =
            string_array_field(object, "skipped_guidance_sources");
    }

    let loaded_guardian_sources = string_array_field(object, "loaded_guardian_sources");
    if !loaded_guardian_sources.is_empty() {
        projection.loaded_guardian_sources = loaded_guardian_sources;
    }

    let skipped_guardian_sources = string_array_field(object, "skipped_guardian_sources");
    if !skipped_guardian_sources.is_empty() {
        projection.skipped_guardian_sources = skipped_guardian_sources;
    }

    let guardian_timeline = string_array_field(object, "guardian_timeline");
    if !guardian_timeline.is_empty() {
        projection.guardian_timeline = guardian_timeline;
    }

    if let Some(summary) =
        object.get("guardian_findings_summary").and_then(|value| value.as_str().map(str::to_string))
    {
        projection.guardian_findings_summary = Some(summary);
    }

    if let Some(findings) = object
        .get("guardian_findings")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<GuardianFinding>>(value).ok())
        && !findings.is_empty()
    {
        projection.guardian_findings = findings;
    }

    let guardian_degradations = string_array_field(object, "guardian_degradations");
    if !guardian_degradations.is_empty() {
        projection.guardian_degradations = guardian_degradations;
    }

    if let Some(outcome) =
        object.get("guardian_blocking_outcome").and_then(|value| value.as_str().map(str::to_string))
    {
        projection.guardian_blocking_outcome = Some(outcome);
    }
}

#[derive(Debug, Default)]
struct TraceInputProjection {
    authored_input_summary: Option<String>,
    authored_input_sources: Vec<String>,
    authored_input_deduplicated_sources: Vec<String>,
    clarification_headline: Option<String>,
    clarification_prompt: Option<String>,
    clarification_missing_fields: Vec<String>,
    requested_governance_runtime: Option<String>,
    requested_governance_risk: Option<String>,
    requested_governance_zone: Option<String>,
    requested_governance_owner: Option<String>,
    negotiation_goal_summary: Option<String>,
    negotiation_resolution: Option<String>,
    negotiation_acceptance_boundary: Option<String>,
}

impl TraceInputProjection {
    fn merge_task_started_payload(&mut self, payload: &Value) {
        if self.authored_input_summary.is_none() {
            self.authored_input_summary =
                nested_payload_string(payload, "input", "authored_input_summary");
        }
        if self.authored_input_sources.is_empty() {
            self.authored_input_sources =
                nested_payload_string_array(payload, "input", "authored_input_sources");
        }
        if self.authored_input_deduplicated_sources.is_empty() {
            self.authored_input_deduplicated_sources = nested_payload_string_array(
                payload,
                "input",
                "authored_input_deduplicated_sources",
            );
        }
        if self.clarification_headline.is_none() {
            self.clarification_headline =
                nested_payload_string(payload, "input", "clarification_headline");
        }
        if self.clarification_prompt.is_none() {
            self.clarification_prompt =
                nested_payload_string(payload, "input", "clarification_prompt");
        }
        if self.clarification_missing_fields.is_empty() {
            self.clarification_missing_fields =
                nested_payload_string_array(payload, "input", "clarification_missing_fields");
        }
        if self.requested_governance_runtime.is_none() {
            self.requested_governance_runtime =
                nested_payload_string(payload, "input", "requested_governance_runtime");
        }
        if self.requested_governance_risk.is_none() {
            self.requested_governance_risk =
                nested_payload_string(payload, "input", "requested_governance_risk");
        }
        if self.requested_governance_zone.is_none() {
            self.requested_governance_zone =
                nested_payload_string(payload, "input", "requested_governance_zone");
        }
        if self.requested_governance_owner.is_none() {
            self.requested_governance_owner =
                nested_payload_string(payload, "input", "requested_governance_owner");
        }
        if self.negotiation_goal_summary.is_none() {
            self.negotiation_goal_summary =
                nested_payload_string(payload, "input", "negotiation_goal_summary");
        }
        if self.negotiation_resolution.is_none() {
            self.negotiation_resolution =
                nested_payload_string(payload, "input", "negotiation_resolution");
        }
        if self.negotiation_acceptance_boundary.is_none() {
            self.negotiation_acceptance_boundary =
                nested_payload_string(payload, "input", "negotiation_acceptance_boundary");
        }
    }

    fn merge_goal_plan_payload(&mut self, payload: &Value) {
        if self.negotiation_goal_summary.is_none() {
            self.negotiation_goal_summary = payload_string(payload, "negotiation_goal_summary");
        }
        if self.negotiation_resolution.is_none() {
            self.negotiation_resolution = payload_string(payload, "negotiation_resolution");
        }
        if self.negotiation_acceptance_boundary.is_none() {
            self.negotiation_acceptance_boundary =
                payload_string(payload, "negotiation_acceptance_boundary");
        }
    }
}

#[derive(Debug, Default)]
struct TraceContextProjection {
    summary: Option<String>,
    credibility: Option<String>,
    primary_inputs: Vec<String>,
    provenance: Vec<String>,
    staleness_reason: Option<String>,
}

impl TraceContextProjection {
    fn merge_task_started_payload(&mut self, payload: &Value) {
        if self.summary.is_none() {
            self.summary = nested_payload_string(payload, "input", "context_summary");
        }
        if self.credibility.is_none() {
            self.credibility = nested_payload_string(payload, "input", "context_credibility");
        }
        if self.primary_inputs.is_empty() {
            self.primary_inputs =
                nested_payload_string_array(payload, "input", "context_primary_inputs");
        }
        if self.provenance.is_empty() {
            self.provenance = nested_payload_string_array(payload, "input", "context_provenance");
        }
        if self.staleness_reason.is_none() {
            self.staleness_reason =
                nested_payload_string(payload, "input", "context_staleness_reason");
        }
    }

    fn merge_goal_plan_payload(&mut self, payload: &Value) {
        if self.summary.is_none() {
            self.summary = payload_string(payload, "context_summary");
        }
        if self.credibility.is_none() {
            self.credibility = payload_string(payload, "context_credibility");
        }
        if self.primary_inputs.is_empty() {
            self.primary_inputs = payload_string_array(payload, "context_primary_inputs");
        }
        if self.provenance.is_empty() {
            self.provenance = payload_string_array(payload, "context_provenance");
        }
        if self.staleness_reason.is_none() {
            self.staleness_reason = payload_string(payload, "context_staleness_reason");
        }
    }

    fn merge_governance_payload(&mut self, payload: &Value) {
        if self.summary.is_none() {
            self.summary = payload_string(payload, "canon_memory_summary");
        }
        if self.credibility.is_none() {
            self.credibility = payload_string(payload, "canon_memory_credibility");
        }
        if self.primary_inputs.is_empty() {
            self.primary_inputs = payload_string_array(payload, "document_refs");
        }

        self.push_optional_line(
            payload_string(payload, "canon_memory_summary")
                .map(|value| format!("canon_memory: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_compatibility")
                .map(|value| format!("canon_memory_compatibility: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_run_ref")
                .or_else(|| payload_string(payload, "run_ref"))
                .map(|value| format!("canon_memory_run_ref: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_packet_ref")
                .or_else(|| payload_string(payload, "packet_ref"))
                .map(|value| format!("canon_memory_packet: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_reason_code")
                .map(|value| format!("canon_memory_reason: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_next_action")
                .map(|value| format!("canon_memory_next_action: {value}")),
        );

        for line in payload_string_array(payload, "authority_provenance_lines") {
            self.push_line(line);
        }
        for line in payload_string_array(payload, "adaptive_provenance_lines") {
            self.push_line(line);
        }

        if self.staleness_reason.is_none()
            && payload_string(payload, "canon_memory_credibility")
                .is_some_and(|credibility| credibility != "credible")
        {
            self.staleness_reason = payload_string(payload, "reason")
                .or_else(|| payload_string(payload, "canon_memory_summary"));
        }
    }

    fn push_optional_line(&mut self, line: Option<String>) {
        if let Some(line) = line {
            self.push_line(line);
        }
    }

    fn push_line(&mut self, line: String) {
        if !self.provenance.contains(&line) {
            self.provenance.push(line);
        }
    }
}

#[derive(Debug, Default)]
struct TraceGovernanceProjection {
    latest_state: Option<String>,
    next_action: Option<String>,
    runtime_state: Option<String>,
    rollout_profile: Option<String>,
    reason: Option<String>,
    approval_provenance: Option<String>,
    timeline: Vec<String>,
}

impl TraceGovernanceProjection {
    fn merge_event(
        &mut self,
        event_type: TraceEventType,
        payload: &Value,
        context_projection: &mut TraceContextProjection,
    ) {
        match event_type {
            TraceEventType::GovernanceAwaitingApproval => {
                self.latest_state = Some("awaiting_approval".to_string());
            }
            TraceEventType::GovernanceCompleted => {
                self.latest_state = Some("governed_ready".to_string());
            }
            TraceEventType::GovernanceBlocked | TraceEventType::GovernancePacketRejected => {
                self.latest_state = Some("blocked".to_string());
            }
            _ => {}
        }

        context_projection.merge_governance_payload(payload);

        if self.next_action.is_none() {
            self.next_action = payload_string(payload, "canon_next_action");
        }
        if self.runtime_state.is_none() {
            self.runtime_state = payload_string(payload, "latest_governance_runtime_state");
        }
        if self.rollout_profile.is_none() {
            self.rollout_profile = payload_string(payload, "latest_governance_rollout_profile");
        }
        if self.reason.is_none() {
            self.reason = payload_string(payload, "latest_governance_reason");
        }
        if self.approval_provenance.is_none() {
            self.approval_provenance =
                payload_string(payload, "latest_governance_approval_provenance");
        }
        if let Some(line) = governance_timeline_line(event_type, payload) {
            self.timeline.push(line);
        }
    }
}

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(|value| value.as_str().map(str::to_string))
}

fn payload_string_array(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn nested_payload_string(payload: &Value, container: &str, key: &str) -> Option<String> {
    payload
        .get(container)
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str().map(str::to_string))
}

fn nested_payload_string_array(payload: &Value, container: &str, key: &str) -> Vec<String> {
    payload
        .get(container)
        .and_then(|value| value.get(key))
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

// Extract a string array from a JSON payload without failing the overall
// inspection path when older or partial payloads omit the key.
fn string_array_field(object: &serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

// Normalize decision-loop events into human-readable timeline lines while
// preserving the persisted decision id, target, rationale, and evidence refs.
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

// Load the requested trace using explicit trace path first, then active-session
// trace ref, then the latest workspace trace.
fn load_trace(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    session_id: Option<&str>,
) -> Result<(TraceResolutionTarget, PathBuf, ExecutionTrace), InspectCommandError> {
    let session_trace_ref = workspace
        .map(|workspace_path| resolve_session_trace_ref(workspace_path, session_id))
        .transpose()?
        .flatten();
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

fn resolve_session_trace_ref(
    workspace: &Path,
    session_id: Option<&str>,
) -> Result<Option<String>, InspectCommandError> {
    let store = FileSessionStore::for_workspace(workspace);
    match session_id {
        Some(session_id) => match store.load_session(session_id) {
            Ok(Some(record)) => Ok(record.latest_trace_ref),
            Ok(None) => Err(InspectCommandError::UnknownSession(session_id.to_string())),
            Err(SessionStoreError::InvalidRecord(message)) => {
                Err(InspectCommandError::InvalidSession(format!(
                    "session `{session_id}` is invalid: {message}"
                )))
            }
            Err(error) => Err(InspectCommandError::SessionStore(error)),
        },
        None => match store.load() {
            Ok(Some(record)) => Ok(record.latest_trace_ref),
            Ok(None) => Ok(None),
            Err(SessionStoreError::InvalidRecord(message)) => {
                Err(InspectCommandError::InvalidSession(format!(
                    "active session is invalid: {message}"
                )))
            }
            Err(error) => Err(InspectCommandError::SessionStore(error)),
        },
    }
}

/// Resolves which trace path `inspect` should open. Precedence is explicit
/// `--trace`, then the active session trace ref, then the latest workspace trace.
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

/// Errors surfaced while resolving and loading traces for `inspect`.
#[derive(Debug, Error)]
pub enum InspectCommandError {
    #[error("inspect requires --trace or --workspace")]
    MissingTraceReference,
    #[error("no persisted trace could be found for the selected workspace")]
    MissingLatestTrace,
    #[error("failed to read the active session: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("session `{0}` does not exist")]
    UnknownSession(String),
    #[error("{0}")]
    InvalidSession(String),
    #[error("failed to read the requested trace: {0}")]
    TraceStore(#[from] TraceStoreError),
    #[error("failed to summarize the requested trace: {0}")]
    Summary(#[from] TraceSummaryError),
}

/// Errors surfaced while rehydrating a persisted trace into a summary view.
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
        corrected_command, decision_failure_evidence, decision_timeline_lines, failure_headline,
        governance_timeline_line, inspection_target_for, merge_guidance_projection_from_payload,
        parse_step_kind, render_error, resolve_session_trace_ref, resolve_trace_path,
        review_timeline_line, reviewer_line, string_array_field, success_headline, summarize_trace,
        synthesized_in_progress_reason,
    };
    use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
    use crate::adapters::trace_store::{FileTraceStore, TraceStore};
    use crate::domain::guidance::GuidanceGuardianProjection;
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
        let session_text = render_error(None, Some(workspace.as_path()), None, &session_error);
        assert!(session_text.contains("failed to read the active session"), "{session_text}");

        let summary_error = InspectCommandError::Summary(TraceSummaryError::MissingTerminalStatus);
        let summary_text = render_error(None, Some(workspace.as_path()), None, &summary_error);
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
    fn summarize_trace_merges_guidance_projection_from_trace_payloads() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            1,
            json!({
                "loaded_packs": [
                    "assistant/packs/guidance-catalog (pack=boundline-guidance-catalog, catalog=boundline-guidance-catalog)"
                ],
                "loaded_guidance_sources": [
                    "assistant/packs/shared/guidance/clean-code.md",
                    7
                ],
                "skipped_guidance_sources": [
                    ".canon/boundline/guidance (missing)"
                ],
                "loaded_guardian_sources": [
                    "assistant/packs/shared/guardians/verification.toml"
                ],
                "guardian_timeline": ["verification_guardian: planned"]
            }),
        );
        trace.record_event(
            TraceEventType::TerminalRecorded,
            None,
            2,
            json!({
                "capability_resolution_summary": "resolved 1 guidance capability entries from 1 source(s) for verification",
                "catalog_validation_findings": [
                    "warning: assistant/packs/guidance-catalog/catalog/guidance-index.toml (legacy alias normalized)"
                ],
                "loaded_guardian_sources": [".boundline/guardians/verification.toml"],
                "skipped_guardian_sources": [
                    "assistant/packs/shared/guardians/verification.toml (shadowed)"
                ],
                "guardian_timeline": ["verification_guardian: completed"],
                "guardian_findings_summary": "1 guardian finding(s); blocking=false",
                "guardian_findings": [{
                    "finding_id": "finding-1",
                    "guardian_id": "verification_guardian",
                    "rule_id": "verification",
                    "disposition": "warn",
                    "summary": "verification evidence is stale",
                    "evidence_refs": ["tests/red_to_green.rs"],
                    "confidence": "medium",
                    "recommended_action": "rerun the bounded verification command",
                    "authority_source": "workspace_override",
                    "source_ref": ".boundline/guardians/verification.toml",
                    "phase": "verification"
                }],
                "guardian_degradations": ["verification route unavailable"],
                "guardian_blocking_outcome": "guardian findings recorded without a blocking outcome"
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(
            summary.guidance_guardian.capability_resolution_summary.as_deref(),
            Some("resolved 1 guidance capability entries from 1 source(s) for verification")
        );
        assert_eq!(
            summary.guidance_guardian.loaded_packs,
            vec![
                "assistant/packs/guidance-catalog (pack=boundline-guidance-catalog, catalog=boundline-guidance-catalog)".to_string()
            ]
        );
        assert_eq!(
            summary.guidance_guardian.loaded_guidance_sources,
            vec!["assistant/packs/shared/guidance/clean-code.md".to_string()]
        );
        assert_eq!(
            summary.guidance_guardian.catalog_validation_findings,
            vec![
                "warning: assistant/packs/guidance-catalog/catalog/guidance-index.toml (legacy alias normalized)".to_string()
            ]
        );
        assert_eq!(
            summary.guidance_guardian.loaded_guardian_sources,
            vec![".boundline/guardians/verification.toml".to_string()]
        );
        assert_eq!(
            summary.guidance_guardian.guardian_timeline,
            vec!["verification_guardian: completed".to_string()]
        );
        assert_eq!(
            summary.guidance_guardian.guardian_findings_summary.as_deref(),
            Some("1 guardian finding(s); blocking=false")
        );
        assert_eq!(summary.guidance_guardian.guardian_findings.len(), 1);
        assert_eq!(
            summary.guidance_guardian.guardian_blocking_outcome.as_deref(),
            Some("guardian findings recorded without a blocking outcome")
        );
    }

    #[test]
    fn merge_guidance_projection_preserves_existing_values_on_partial_payloads() {
        let mut projection = GuidanceGuardianProjection {
            capability_resolution_summary: Some("existing".to_string()),
            loaded_packs: vec!["assistant/packs/guidance-catalog".to_string()],
            skipped_packs: vec!["assistant/packs/legacy-pack (skipped)".to_string()],
            catalog_validation_findings: vec!["warning: existing finding".to_string()],
            loaded_guidance_sources: vec![".canon/boundline/guidance/clean-code.md".to_string()],
            skipped_guidance_sources: vec!["assistant/packs/shared (shadowed)".to_string()],
            ..GuidanceGuardianProjection::default()
        };

        merge_guidance_projection_from_payload(&mut projection, &json!(["not-an-object"]));
        merge_guidance_projection_from_payload(
            &mut projection,
            &json!({
                "loaded_packs": [],
                "catalog_validation_findings": [],
                "loaded_guardian_sources": ["assistant/packs/guidance-catalog/guardians/catalog-review.md"],
                "guardian_timeline": ["catalog_review: completed"],
                "guardian_degradations": ["verification route unavailable"],
                "guardian_blocking_outcome": "guardian findings recorded without a blocking outcome"
            }),
        );

        assert_eq!(projection.capability_resolution_summary.as_deref(), Some("existing"));
        assert_eq!(projection.loaded_packs, vec!["assistant/packs/guidance-catalog".to_string()]);
        assert_eq!(
            projection.skipped_packs,
            vec!["assistant/packs/legacy-pack (skipped)".to_string()]
        );
        assert_eq!(
            projection.catalog_validation_findings,
            vec!["warning: existing finding".to_string()]
        );
        assert_eq!(
            projection.loaded_guardian_sources,
            vec!["assistant/packs/guidance-catalog/guardians/catalog-review.md".to_string()]
        );
        assert_eq!(projection.guardian_timeline, vec!["catalog_review: completed".to_string()]);
        assert_eq!(
            projection.guardian_degradations,
            vec!["verification route unavailable".to_string()]
        );
        assert_eq!(
            projection.guardian_blocking_outcome.as_deref(),
            Some("guardian findings recorded without a blocking outcome")
        );
    }

    #[test]
    fn string_array_field_filters_non_string_values() {
        let payload = json!({
            "loaded_packs": ["assistant/packs/guidance-catalog", 7, true, "assistant/packs/legacy"],
            "missing": "not-an-array"
        });
        let object = payload.as_object().unwrap();

        assert_eq!(
            string_array_field(object, "loaded_packs"),
            vec![
                "assistant/packs/guidance-catalog".to_string(),
                "assistant/packs/legacy".to_string(),
            ]
        );
        assert!(string_array_field(object, "missing").is_empty());
        assert!(string_array_field(object, "absent").is_empty());
    }

    #[test]
    fn summarize_trace_collects_task_started_and_goal_plan_input_projection() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            0,
            json!({
                "input": {
                    "authored_input_summary": "authored brief narrowed to src/lib.rs",
                    "authored_input_sources": ["brief.md", 7],
                    "authored_input_deduplicated_sources": ["brief.md", "README.md"],
                    "clarification_headline": "Need bounded scope",
                    "clarification_prompt": "Confirm the failing file before planning",
                    "clarification_missing_fields": ["target_file", true],
                    "requested_governance_runtime": "canon",
                    "requested_governance_risk": "medium",
                    "requested_governance_zone": "engineering",
                    "requested_governance_owner": "platform",
                    "negotiation_goal_summary": "repair the failing arithmetic path",
                    "negotiation_resolution": "accepted",
                    "negotiation_acceptance_boundary": "bounded fix only",
                    "context_summary": "bounded context from src/lib.rs",
                    "context_credibility": "credible",
                    "context_primary_inputs": ["src/lib.rs", false],
                    "context_provenance": ["workspace_file: src/lib.rs", {"ignored": true}],
                    "context_staleness_reason": "context snapshot is stale"
                }
            }),
        );
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            1,
            json!({
                "task_count": 2,
                "goal": "Repair arithmetic",
                "goal_plan_state": "confirmed",
                "goal_plan_revision": 3,
                "flow_state": "bug-fix/implement",
                "verification_strategy": "cargo test --quiet",
                "planning_rationale": "focus on src/lib.rs"
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(
            summary.authored_input_summary.as_deref(),
            Some("authored brief narrowed to src/lib.rs")
        );
        assert_eq!(summary.authored_input_sources, vec!["brief.md".to_string()]);
        assert_eq!(
            summary.authored_input_deduplicated_sources,
            vec!["brief.md".to_string(), "README.md".to_string()]
        );
        assert_eq!(summary.clarification_headline.as_deref(), Some("Need bounded scope"));
        assert_eq!(
            summary.clarification_prompt.as_deref(),
            Some("Confirm the failing file before planning")
        );
        assert_eq!(summary.clarification_missing_fields, vec!["target_file".to_string()]);
        assert_eq!(summary.requested_governance_runtime.as_deref(), Some("canon"));
        assert_eq!(summary.requested_governance_risk.as_deref(), Some("medium"));
        assert_eq!(summary.requested_governance_zone.as_deref(), Some("engineering"));
        assert_eq!(summary.requested_governance_owner.as_deref(), Some("platform"));
        assert_eq!(
            summary.negotiation_goal_summary.as_deref(),
            Some("repair the failing arithmetic path")
        );
        assert_eq!(summary.negotiation_resolution.as_deref(), Some("accepted"));
        assert_eq!(summary.negotiation_acceptance_boundary.as_deref(), Some("bounded fix only"));
        assert_eq!(summary.context_summary.as_deref(), Some("bounded context from src/lib.rs"));
        assert_eq!(summary.context_credibility.as_deref(), Some("credible"));
        assert_eq!(summary.context_primary_inputs, vec!["src/lib.rs".to_string()]);
        assert_eq!(summary.context_provenance, vec!["workspace_file: src/lib.rs".to_string()]);
        assert_eq!(summary.context_staleness_reason.as_deref(), Some("context snapshot is stale"));
        assert_eq!(
            summary.goal_plan_summary.as_deref(),
            Some(
                "2 bounded task(s) for Repair arithmetic [confirmed rev 3] | flow: bug-fix/implement | verification: cargo test --quiet | rationale: focus on src/lib.rs"
            )
        );
    }

    #[test]
    fn decision_review_and_governance_helpers_cover_fallback_paths() {
        let created = decision_timeline_lines(
            TraceEventType::DecisionCreated,
            Some("decision-1"),
            &json!({
                "status": "pending",
                "selector": "modify",
                "decision_type": "code",
                "target": "src/lib.rs",
                "rationale": "repair the failing implementation",
                "expected_outcome": "tests pass",
                "evidence_inputs": [
                    {"kind": "file", "reference": "src/lib.rs"},
                    {"kind": "tool_output", "reference": "cargo test --quiet"}
                ]
            }),
        );
        assert!(
            created.contains(
                &"decision: decision-1 modify (code) -> src/lib.rs [pending]".to_string()
            )
        );
        assert!(created.contains(&"selector: modify".to_string()));
        assert!(created.contains(&"rationale: repair the failing implementation".to_string()));
        assert!(created.contains(&"verification_intent: tests pass".to_string()));
        assert!(created.contains(
            &"evidence_inputs: file:src/lib.rs, tool_output:cargo test --quiet".to_string()
        ));

        let recovered = decision_timeline_lines(
            TraceEventType::DecisionRecovered,
            Some("decision-1"),
            &json!({"status": "recovered", "recovery_decision_id": "decision-2"}),
        );
        assert_eq!(
            recovered,
            vec!["decision_status: decision-1 recovered via decision-2".to_string()]
        );

        assert_eq!(
            review_timeline_line(
                TraceEventType::ReviewerCompleted,
                &json!({"reviewer_id": "safety", "failure_reason": "tool crashed"}),
            )
            .as_deref(),
            Some("reviewer safety failed: tool crashed")
        );
        assert_eq!(
            review_timeline_line(
                TraceEventType::ReviewVoteResolved,
                &json!({"vote_resolution": {"decision": "accepted"}}),
            )
            .as_deref(),
            Some("review_vote: {\"decision\":\"accepted\"}")
        );
        assert_eq!(
            governance_timeline_line(
                TraceEventType::GovernanceDecisionRecorded,
                &json!({"blocked_reason": "approval missing"}),
            )
            .as_deref(),
            Some("governance_decision_blocked: approval missing")
        );

        assert_eq!(
            success_headline(&json!({"output": {"changed_files": ["src/lib.rs"]}}), 2),
            "updated src/lib.rs after 2 attempt(s)"
        );
        assert_eq!(
            decision_failure_evidence(
                Some("decision-1"),
                &json!({
                    "target": "src/lib.rs",
                    "action_result": {
                        "tool_id": "cargo-test",
                        "invocation": "cargo test --quiet",
                        "success": false,
                        "duration_ms": 12,
                        "stdout": "",
                        "stderr": "tests failed"
                    }
                }),
            )
            .as_deref(),
            Some("decision-1 src/lib.rs: tests failed")
        );
    }

    #[test]
    fn trace_loading_helpers_cover_explicit_session_and_latest_workspace_paths() {
        let workspace = temp_workspace("boundline-inspect-load-trace");
        let trace_store = FileTraceStore::for_workspace(&workspace);
        let trace_path = trace_store.persist(&terminal_trace()).unwrap();

        let (target, latest_path) = resolve_trace_path(None, Some(&workspace), None).unwrap();
        assert_eq!(target, TraceResolutionTarget::LatestWorkspaceTrace);
        assert_eq!(latest_path, trace_path);

        let (target, session_path) =
            resolve_trace_path(None, Some(&workspace), Some("relative/trace.json")).unwrap();
        assert_eq!(target, TraceResolutionTarget::SessionTraceRef);
        assert_eq!(session_path, PathBuf::from("relative/trace.json"));

        let (target, loaded_path, loaded_trace) =
            super::load_trace(Some(trace_path.as_path()), Some(&workspace), None).unwrap();
        assert_eq!(target, TraceResolutionTarget::ExplicitTrace);
        assert_eq!(loaded_path, trace_path);
        assert_eq!(loaded_trace.goal, "Inspect trace");
    }

    #[test]
    fn resolve_session_trace_ref_prefers_selected_session_without_switching_active_pointer()
    -> Result<(), String> {
        let workspace = temp_workspace("boundline-inspect-selected-session");
        let store = FileSessionStore::for_workspace(&workspace);
        let active_record = ActiveSessionRecord {
            session_id: "active-session".to_string(),
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
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: Some("active/trace.json".to_string()),
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        };
        let selected_record = ActiveSessionRecord {
            session_id: "selected-session".to_string(),
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
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: Some("selected/trace.json".to_string()),
            created_at: 2,
            updated_at: 2,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        };

        store.persist(&active_record).map_err(|error| error.to_string())?;
        store.persist_without_select(&selected_record).map_err(|error| error.to_string())?;

        let selected_trace = resolve_session_trace_ref(&workspace, Some("selected-session"))
            .map_err(|error| error.to_string())?;
        if selected_trace.as_deref() != Some("selected/trace.json") {
            return Err(format!("expected selected trace ref, got {selected_trace:?}"));
        }

        let active_trace =
            resolve_session_trace_ref(&workspace, None).map_err(|error| error.to_string())?;
        if active_trace.as_deref() != Some("active/trace.json") {
            return Err(format!("expected active trace ref, got {active_trace:?}"));
        }

        let active_after = store
            .load()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected active session record".to_string())?;
        if active_after.session_id != active_record.session_id {
            return Err(format!(
                "expected active session {} to remain selected, got {}",
                active_record.session_id, active_after.session_id
            ));
        }

        Ok(())
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
            delight_feedback: None,
        };
        fs::write(
            workspace.join(".boundline/session.json"),
            serde_json::to_vec_pretty(&invalid_record).unwrap(),
        )
        .unwrap();

        assert!(matches!(
            resolve_session_trace_ref(&workspace, None).unwrap_err(),
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
                "canon_next_action": "refresh: refresh the governed packet and reassess its credibility",
                "authority_provenance_lines": ["authority_control_class: council_review"],
                "adaptive_provenance_lines": ["adaptive_contract_line: adaptive-governance-v1"]
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
        assert!(
            summary
                .context_provenance
                .contains(&"authority_control_class: council_review".to_string())
        );
        assert!(
            summary
                .context_provenance
                .contains(&"adaptive_contract_line: adaptive-governance-v1".to_string())
        );
        assert_eq!(summary.context_staleness_reason.as_deref(), Some("refresh_required"));
        assert_eq!(
            summary.governance_next_action.as_deref(),
            Some("refresh: refresh the governed packet and reassess its credibility")
        );
    }

    #[test]
    fn summarize_trace_rehydrates_reasoning_profile_from_governance_payload() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::GovernanceCompleted,
            Some("step-1".to_string()),
            0,
            json!({
                "stage_key": "bug-fix:verify",
                "runtime": "canon",
                "headline": "reasoning profile bounded_reflexion degraded",
                "reasoning_profile_record": {
                    "activation_id": "reasoning-attempt-2",
                    "stage_key": "bug-fix:verify",
                    "profile_id": "bounded_reflexion",
                    "trigger": "canon_required_challenge",
                    "activation_reason": "Canon governance activated stronger challenge",
                    "status": "degraded",
                    "participants": [],
                    "budget": {
                        "max_participants": 1,
                        "max_branches": 1,
                        "max_debate_rounds": 0,
                        "max_reflexion_revisions": 2,
                        "max_calls": 2,
                        "max_tokens": 6000,
                        "max_adjudication_steps": 1
                    },
                    "independence": {
                        "requested_floor": {
                            "route_distinct": false,
                            "provider_distinct": false,
                            "context_distinct": false,
                            "prompt_pattern_distinct": false,
                            "minimum_participants": 1
                        },
                        "observed_distinctions": {
                            "distinct_routes": 1,
                            "distinct_providers": 1,
                            "distinct_contexts": 1,
                            "distinct_prompt_patterns": 1
                        },
                        "result": "degraded",
                        "reason": "reflexion remained bounded but shared one runtime"
                    },
                    "outcome": {
                        "outcome_kind": "degraded",
                        "headline": "bounded reflexion degraded",
                        "disagreement_summary": "shared runtime reduced independence",
                        "next_action": "escalate to blind review if confidence remains low",
                        "iterations": []
                    }
                }
            }),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        let reasoning_profile = summary.reasoning_profile.expect("reasoning profile should exist");
        assert_eq!(reasoning_profile.profile_id.as_str(), "bounded_reflexion");
        assert_eq!(reasoning_profile.status.as_str(), "degraded");
        assert_eq!(
            reasoning_profile.outcome.as_ref().and_then(|outcome| outcome.next_action.as_deref()),
            Some("escalate to blind review if confidence remains low")
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
            governance_timeline_line(
                TraceEventType::GovernanceSelected,
                &json!({"stage_key": "bug-fix:review", "selected_runtime": "canon"}),
            ),
            Some("governance_selected: bug-fix:review -> canon".to_string())
        );
        assert_eq!(
            governance_timeline_line(
                TraceEventType::GovernanceStarted,
                &json!({
                    "stage_key": "bug-fix:review",
                    "canon_mode": "direct",
                    "run_ref": "canon-run-7",
                    "packet_source_stage": "bug-fix:implement",
                    "packet_binding_reason": "stage_context"
                }),
            ),
            Some(
                "governance_started: bug-fix:review (direct) [canon-run-7] from bug-fix:implement (stage_context)"
                    .to_string(),
            )
        );
        assert_eq!(
            reviewer_line(&json!({
                "reviewer_id": "safety",
                "failure_reason": "tool timeout"
            })),
            Some("reviewer safety failed: tool timeout".to_string())
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

    #[test]
    fn inspect_helpers_cover_progress_and_fallback_paths() {
        assert_eq!(
            decision_timeline_lines(
                TraceEventType::DecisionRecovered,
                Some("decision-1"),
                &json!({
                    "status": "recovered",
                    "target": "src/lib.rs",
                    "recovery_decision_id": "decision-0"
                }),
            ),
            vec!["decision_status: decision-1 recovered via decision-0".to_string()]
        );
        assert!(
            decision_timeline_lines(TraceEventType::TerminalRecorded, None, &json!({})).is_empty()
        );
        assert_eq!(review_timeline_line(TraceEventType::TerminalRecorded, &json!({})), None);
        assert_eq!(governance_timeline_line(TraceEventType::TerminalRecorded, &json!({})), None);

        assert_eq!(
            synthesized_in_progress_reason(Some("awaiting_approval")).message,
            "governance approval is still pending"
        );
        assert_eq!(
            synthesized_in_progress_reason(Some("blocked")).message,
            "governed work is blocked pending intervention"
        );
        assert_eq!(
            synthesized_in_progress_reason(Some("governed_ready")).message,
            "governed work is ready for the next bounded step"
        );
        assert_eq!(synthesized_in_progress_reason(None).message, "trace is still in progress");

        assert_eq!(
            success_headline(
                &json!({
                    "output": {
                        "workspace_slice": {"headline": "focused src/lib.rs"},
                        "selection_evidence": {"reason": "closest failing target"}
                    }
                }),
                1,
            ),
            "adaptive slice focused src/lib.rs: closest failing target"
        );
        assert_eq!(
            success_headline(
                &json!({"output": {"changed_files": ["src/lib.rs", "src/main.rs"]}}),
                2,
            ),
            "updated src/lib.rs, src/main.rs after 2 attempt(s)"
        );
    }
}
