//! Trace inspection and summary rehydration for operator-facing CLI output.

#[path = "inspect/projections.rs"]
mod projections;
#[path = "inspect/resolve.rs"]
mod resolve;
#[path = "inspect/timeline.rs"]
mod timeline;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

use crate::adapters::audit_store::SessionAuditStoreError;
use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::cluster::ClusterDeliveryStory;
use crate::domain::completion_verification::CompletionVerificationProjection;
use crate::domain::context_intelligence::AdvancedContextProjection;
use crate::domain::execution::StageRoutingDecisionRecord;
use crate::domain::framework_adapter::AdapterExecutionSource;
use crate::domain::goal_plan::GoalPlanFlowState;
use crate::domain::guidance::GuidanceGuardianProjection;
use crate::domain::reasoning::ProfileActivationRecord;
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::governance_next_action_for_state;
use crate::domain::session::{
    DelightFeedbackSignal, FrameworkAdapterStageFailureDetails, RoutingMode, RoutingOutcome,
    RoutingSource,
};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskStatus, TerminalReason};
use crate::domain::trace::{
    ExecutionTrace, InspectClosureKind, InspectClosureView, TraceEvent, TraceEventType,
    TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
};

use self::projections::{
    TraceContextProjection, TraceGovernanceProjection, TraceInputProjection,
    merge_guidance_projection_from_payload,
};
use self::resolve::{ResolvedTraceArtifacts, load_trace, resolve_trace_artifacts};
use self::timeline::{
    adaptive_evidence_lines, decision_failure_evidence, decision_timeline_lines, failure_headline,
    governance_timeline_line, review_timeline_line, success_headline,
    synthesized_in_progress_reason,
};

const UNKNOWN_VALIDATION_EXIT_CODE: i64 = -1;
const UNKNOWN_DECISION_ID: &str = "unknown-decision";
const UNKNOWN_TARGET: &str = "unknown";
const KEY_ACTION_RESULT: &str = "action_result";
const KEY_FAILURE_REASON: &str = "failure_reason";
const KEY_PLAN_QUALITY_ASSUMPTIONS: &str = "plan_quality_assumptions";
const KEY_PLAN_QUALITY_FINDINGS: &str = "plan_quality_findings";
const KEY_PLAN_QUALITY_STATE: &str = "plan_quality_state";

fn string_list_from_payload(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

fn advanced_context_from_payload(payload: &Value) -> Option<AdvancedContextProjection> {
    payload.get("advanced_context").cloned().and_then(|value| serde_json::from_value(value).ok())
}

fn framework_adapter_stage_failure_evidence(payload: &Value) -> Option<String> {
    let failure = payload
        .get("framework_adapter_stage_failure")
        .and_then(FrameworkAdapterStageFailureDetails::from_value)?;
    let failure_class = failure
        .execution
        .failure_class
        .map(|failure_class| failure_class.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    Some(format!(
        "framework_adapter stage={} claim={} failure_class={} summary={}{}",
        failure.execution.stage_key.as_str(),
        failure.claim_state.as_str(),
        failure_class,
        failure.summary,
        failure
            .protocol_error_code
            .as_deref()
            .map(|code| format!(" protocol_error_code={code}"))
            .unwrap_or_default()
    ))
}

fn framework_adapter_stage_routing_from_payload(
    payload: &Value,
) -> Option<StageRoutingDecisionRecord> {
    payload
        .get("framework_adapter_stage_routing")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
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
    audit_only: bool,
) -> Result<InspectCommandReport, InspectCommandError> {
    let (inspection_target, trace_ref, trace) = load_trace(trace, workspace, session_id)?;
    let summary = match summarize_trace(&trace_ref, &trace) {
        Ok(summary) => summary,
        Err(
            TraceSummaryError::MissingTerminalStatus | TraceSummaryError::MissingTerminalReason,
        ) => fallback_trace_summary_for_active_completion_verification(
            workspace, session_id, &trace_ref, &trace,
        )?
        .ok_or(InspectCommandError::Summary(TraceSummaryError::MissingTerminalStatus))?,
        Err(error) => return Err(InspectCommandError::Summary(error)),
    };
    let exit_status = if summary.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };
    let mut terminal_output = if audit_only {
        output::render_trace_audit_summary(
            &summary,
            inspection_target.as_str(),
            output::next_command_after_inspect(summary.terminal_status),
        )
    } else {
        output::render_trace_summary(
            &summary,
            inspection_target.as_str(),
            output::next_command_after_inspect(summary.terminal_status),
        )
    };
    let completion_lines =
        inspect_completion_verification_lines(workspace, session_id, &trace_ref, &trace)?;
    if !completion_lines.is_empty() {
        terminal_output.push('\n');
        terminal_output.push_str(&completion_lines.join("\n"));
    }

    Ok(InspectCommandReport {
        exit_status,
        terminal_output,
        inspection_target: Some(inspection_target.as_str().to_string()),
        trace_location: Some(trace_ref.to_string_lossy().into_owned()),
        trace_summary: Some(summary),
    })
}

fn fallback_trace_summary_for_active_completion_verification(
    workspace: Option<&Path>,
    session_id: Option<&str>,
    trace_ref: &Path,
    trace: &ExecutionTrace,
) -> Result<Option<TraceSummaryView>, InspectCommandError> {
    let resolved_workspace =
        workspace.map(Path::to_path_buf).or_else(|| workspace_from_trace_ref(trace_ref));
    let Some(workspace) = resolved_workspace.as_deref() else {
        return Ok(None);
    };
    let store = FileSessionStore::for_workspace(workspace);
    let session = if let Some(session_id) = session_id {
        store.load_session(session_id).map_err(InspectCommandError::SessionStore)?
    } else {
        store.load().map_err(InspectCommandError::SessionStore)?
    };
    let Some(session) = session else {
        return Ok(None);
    };
    let Some(latest_trace_ref) = session.latest_trace_ref.as_deref() else {
        return Ok(None);
    };
    if latest_trace_ref != trace_ref.to_string_lossy() {
        return Ok(None);
    }
    let Some(task) = session.active_task.as_ref() else {
        return Ok(None);
    };
    let projection = task
        .context
        .completion_verification_projection()
        .map_err(|error| InspectCommandError::InvalidSession(error.to_string()))?;
    let Some(projection) = projection else {
        return Ok(None);
    };
    let terminal_reason = TerminalReason::new(
        crate::domain::limits::TerminalCondition::GoalSatisfied,
        projection
            .completion_verification_findings
            .first()
            .map(|finding| finding.message.clone())
            .unwrap_or_else(|| "completion verification blocked closeout".to_string()),
        None,
    );

    Ok(Some(TraceSummaryView {
        trace_ref: trace_ref.to_string_lossy().into_owned(),
        goal: trace.goal.clone(),
        trace_started_at: Some(trace.started_at),
        advanced_context: task
            .context
            .latest_advanced_context()
            .map_err(|error| InspectCommandError::InvalidSession(error.to_string()))?,
        cluster_delivery_story: task.context.cluster_delivery_story().ok().flatten(),
        terminal_status: TaskStatus::Running,
        terminal_reason,
        duration: trace.duration_millis(),
        ..TraceSummaryView::default()
    }))
}

fn inspect_completion_verification_lines(
    workspace: Option<&Path>,
    session_id: Option<&str>,
    trace_ref: &Path,
    trace: &ExecutionTrace,
) -> Result<Vec<String>, InspectCommandError> {
    let resolved_workspace =
        workspace.map(Path::to_path_buf).or_else(|| workspace_from_trace_ref(trace_ref));
    let Some(workspace) = resolved_workspace.as_deref() else {
        return Ok(Vec::new());
    };
    let store = FileSessionStore::for_workspace(workspace);
    let session = if let Some(session_id) = session_id {
        store.load_session(session_id).map_err(InspectCommandError::SessionStore)?
    } else {
        store.load().map_err(InspectCommandError::SessionStore)?
    };
    let Some(session) = session else {
        return Ok(Vec::new());
    };
    let Some(latest_trace_ref) = session.latest_trace_ref.as_deref() else {
        return Ok(Vec::new());
    };
    if latest_trace_ref != trace_ref.to_string_lossy() {
        return Ok(Vec::new());
    }
    let Some(task) = session.active_task.as_ref() else {
        return Ok(Vec::new());
    };
    let projection = task
        .context
        .completion_verification_projection()
        .map_err(|error| InspectCommandError::InvalidSession(error.to_string()))?;
    let Some(projection) = projection else {
        return Ok(Vec::new());
    };
    Ok(render_completion_verification_lines(&projection, Some(&trace.task_id)))
}

fn workspace_from_trace_ref(trace_ref: &Path) -> Option<PathBuf> {
    trace_ref.ancestors().find_map(|path| {
        if path.file_name().is_some_and(|name| name == ".boundline") {
            path.parent().map(Path::to_path_buf)
        } else {
            None
        }
    })
}

fn render_completion_verification_lines(
    projection: &CompletionVerificationProjection,
    default_task_id: Option<&str>,
) -> Vec<String> {
    let mut lines = vec![format!(
        "completion_verification_state: {}",
        projection.completion_verification_state.as_str()
    )];
    if let Some(claim) = projection.claim.as_ref() {
        lines.push(format!("completion_claim_kind: {}", claim.kind.as_str()));
        lines.push(format!("completion_claim_source: {}", claim.source.as_str()));
        lines.push(format!("completion_claim_summary: {}", claim.summary));
    }
    if !projection.completion_blocked_claims.is_empty() {
        lines.push(format!(
            "completion_blocked_claims: {}",
            projection
                .completion_blocked_claims
                .iter()
                .map(|claim| claim.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !projection.completion_evidence_refs.is_empty() {
        lines.push(format!(
            "completion_evidence_refs: {}",
            projection.completion_evidence_refs.join(", ")
        ));
    }
    for finding in &projection.completion_verification_findings {
        lines.push(format!(
            "completion_verification_finding: {} | {} | {}",
            finding.kind.as_str(),
            finding.severity.as_str(),
            finding.message
        ));
        if let Some(task_id) = finding.task_id.as_deref().or(default_task_id) {
            lines.push(format!("completion_verification_task_id: {task_id}"));
        }
        if !finding.changed_paths.is_empty() {
            lines.push(format!(
                "completion_verification_changed_paths: {}",
                finding.changed_paths.join(", ")
            ));
        }
        lines.push(format!(
            "completion_verification_required_action: {}",
            finding.required_action.as_str()
        ));
    }
    lines
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

#[derive(Debug, Default)]
struct TraceSummaryFold {
    input_projection: TraceInputProjection,
    cluster_delivery_story: Option<ClusterDeliveryStory>,
    routing_summary: Option<String>,
    routing_projection: RoutingDecisionProjection,
    goal_plan_summary: Option<String>,
    plan_quality_state: Option<String>,
    plan_quality_findings: Vec<String>,
    plan_quality_assumptions: Vec<String>,
    advanced_context: Option<AdvancedContextProjection>,
    context_projection: TraceContextProjection,
    guidance_guardian: GuidanceGuardianProjection,
    governance_projection: TraceGovernanceProjection,
    decision_timeline: Vec<String>,
    failure_evidence: Vec<String>,
    adaptive_evidence: Vec<String>,
    latest_checkpoint_id: Option<String>,
    latest_checkpoint_scope: Option<String>,
    latest_checkpoint_restore_command: Option<String>,
    delegation: Option<crate::domain::session::DelegationStatusView>,
    saw_native_routing_signal: bool,
    step_indexes: HashMap<String, usize>,
    executed_steps: Vec<TraceStepSummary>,
    recovery_events: Vec<TraceRecoveryEvent>,
    review_timeline: Vec<String>,
    reasoning_profile: Option<ProfileActivationRecord>,
    delight_feedback: Option<DelightFeedbackSignal>,
    framework_adapter_stage_routing: Option<StageRoutingDecisionRecord>,
}

impl TraceSummaryFold {
    fn merge_event(
        &mut self,
        event: &TraceEvent,
        trace_goal: &str,
    ) -> Result<(), TraceSummaryError> {
        if let Some(signal) = event
            .payload
            .get(KEY_DELIGHT_FEEDBACK)
            .cloned()
            .and_then(|value| serde_json::from_value::<DelightFeedbackSignal>(value).ok())
            .filter(|signal| signal.validate().is_ok())
        {
            self.delight_feedback = Some(signal);
        }

        if let Some(record) = event
            .payload
            .get("reasoning_profile_record")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
        {
            self.reasoning_profile = Some(record);
        }

        if let Some(stage_routing) = framework_adapter_stage_routing_from_payload(&event.payload) {
            if stage_routing.execution_source == AdapterExecutionSource::Adapter {
                self.routing_summary = event
                    .payload
                    .get(KEY_SUMMARY)
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| self.routing_summary.take());
            }
            self.framework_adapter_stage_routing = Some(stage_routing);
        }

        // Guidance and guardian projection is persisted incrementally across
        // planning, execution, and verification events; inspect rebuilds the
        // latest authoritative view by folding those payload snapshots.
        merge_guidance_projection_from_payload(&mut self.guidance_guardian, &event.payload);

        if self.routing_projection.is_empty()
            && let Some(projection) = RoutingDecisionProjection::from_event_payload(&event.payload)
        {
            self.routing_projection = projection;
        }

        if self.delegation.is_none() {
            self.delegation = event
                .payload
                .get("delegation")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok());
        }

        if event.event_type.is_decision_loop_event() {
            self.saw_native_routing_signal = true;
        }

        match event.event_type {
            TraceEventType::TaskStarted => {
                self.input_projection.merge_task_started_payload(&event.payload);
                self.context_projection.merge_task_started_payload(&event.payload);
            }
            TraceEventType::TerminalRecorded => {
                self.cluster_delivery_story = event
                    .payload
                    .get("cluster_delivery_story")
                    .cloned()
                    .and_then(|value| serde_json::from_value(value).ok());
            }
            TraceEventType::ReviewerStarted => {}
            TraceEventType::CheckpointCreated => {
                self.latest_checkpoint_id = event
                    .payload
                    .get("checkpoint_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(self.latest_checkpoint_id.take());
                self.latest_checkpoint_scope = event
                    .payload
                    .get("checkpoint_scope")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(self.latest_checkpoint_scope.take());
                self.latest_checkpoint_restore_command = event
                    .payload
                    .get("checkpoint_restore_command")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(self.latest_checkpoint_restore_command.take());
            }
            TraceEventType::FlowSelected => {
                self.recovery_events.push(TraceRecoveryEvent {
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
            TraceEventType::StageRouted => {}
            TraceEventType::StageTransitioned => {
                self.recovery_events.push(TraceRecoveryEvent {
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

                if let Some(index) = self.step_indexes.get(&step_id) {
                    self.executed_steps[*index].attempts += 1;
                } else {
                    self.step_indexes.insert(step_id.clone(), self.executed_steps.len());
                    self.executed_steps.push(TraceStepSummary {
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

                let index = *self
                    .step_indexes
                    .get(&step_id)
                    .ok_or_else(|| TraceSummaryError::MissingStartedStep(step_id.clone()))?;
                let headline = match final_status {
                    StepStatus::Succeeded => {
                        success_headline(&event.payload, self.executed_steps[index].attempts)
                    }
                    StepStatus::Failed => {
                        failure_headline(&event.payload, self.executed_steps[index].attempts)
                    }
                    _ => "completed".to_string(),
                };
                self.executed_steps[index].final_status = final_status;
                self.executed_steps[index].headline = headline;
                for line in adaptive_evidence_lines(&event.payload) {
                    if !self.adaptive_evidence.contains(&line) {
                        self.adaptive_evidence.push(line);
                    }
                }
            }
            TraceEventType::RetryScheduled
            | TraceEventType::StageRetryScheduled
            | TraceEventType::Replanned
            | TraceEventType::StageReplanned
            | TraceEventType::StageFailed => {
                self.recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: event
                        .payload
                        .get("reason")
                        .and_then(|value| value.as_str())
                        .unwrap_or("recovery event")
                        .to_string(),
                    related_step_id: event.step_id.clone(),
                });
                if event.event_type == TraceEventType::StageFailed
                    && let Some(evidence) = framework_adapter_stage_failure_evidence(&event.payload)
                {
                    self.failure_evidence.push(evidence);
                }
            }
            TraceEventType::GovernanceSelected
            | TraceEventType::GovernanceStarted
            | TraceEventType::GovernanceDecisionRecorded
            | TraceEventType::GovernanceAwaitingApproval
            | TraceEventType::GovernanceCompleted
            | TraceEventType::GovernanceBlocked
            | TraceEventType::GovernancePacketRejected => {
                self.saw_native_routing_signal = true;
                self.governance_projection.merge_event(
                    event.event_type,
                    &event.payload,
                    &mut self.context_projection,
                );
            }
            TraceEventType::ReviewStarted
            | TraceEventType::ReviewTriggerIgnored
            | TraceEventType::ReviewerCompleted
            | TraceEventType::ReviewCouncilAssembled
            | TraceEventType::ReviewStopSemanticsRecorded
            | TraceEventType::ReviewVoteResolved
            | TraceEventType::ReviewAdjudicated
            | TraceEventType::ReviewTerminalRecorded => {
                if let Some(line) = review_timeline_line(event.event_type, &event.payload) {
                    self.review_timeline.push(line);
                }
            }
            TraceEventType::GoalPlanCreated => {
                self.plan_quality_state = event
                    .payload
                    .get(KEY_PLAN_QUALITY_STATE)
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(self.plan_quality_state.take());
                if self.plan_quality_findings.is_empty() {
                    self.plan_quality_findings =
                        string_list_from_payload(&event.payload, KEY_PLAN_QUALITY_FINDINGS);
                }
                if self.plan_quality_assumptions.is_empty() {
                    self.plan_quality_assumptions =
                        string_list_from_payload(&event.payload, KEY_PLAN_QUALITY_ASSUMPTIONS);
                }
                if self.routing_summary.is_none() {
                    self.routing_summary = Some(output::render_route_outcome(&RoutingOutcome {
                        mode: RoutingMode::Native,
                        source: RoutingSource::GoalPlan,
                        reason: "goal plan trace came from the session-native runtime".to_string(),
                    }));
                }
                if self.goal_plan_summary.is_none() {
                    let task_count = event
                        .payload
                        .get("task_count")
                        .and_then(|value| value.as_u64())
                        .unwrap_or_default();
                    let goal = event
                        .payload
                        .get("goal")
                        .and_then(|value| value.as_str())
                        .unwrap_or(trace_goal);
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
                    self.advanced_context = self
                        .advanced_context
                        .take()
                        .or_else(|| advanced_context_from_payload(&event.payload));
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
                    self.goal_plan_summary = Some(format!(
                        "{task_count} bounded task(s) for {goal}{state_suffix}{flow_suffix}{verification_suffix}{rationale_suffix}"
                    ));
                }
                self.input_projection.merge_goal_plan_payload(&event.payload);
                self.context_projection.merge_goal_plan_payload(&event.payload);
            }
            TraceEventType::FlowInferred => {
                if let Some(flow_name) =
                    event.payload.get("flow_name").and_then(|value| value.as_str())
                {
                    self.decision_timeline.push(format!("flow_inferred: {flow_name}"));
                }
            }
            TraceEventType::RefinementRoundCompleted => {
                // Refinement round packets are surfaced via the refinement
                // inspection projection (render_refinement_inspection).
            }
            TraceEventType::DecisionCreated
            | TraceEventType::DecisionDispatched
            | TraceEventType::DecisionVerified
            | TraceEventType::DecisionFailed
            | TraceEventType::DecisionRecovered => {
                self.decision_timeline.extend(decision_timeline_lines(
                    event.event_type,
                    event.step_id.as_deref(),
                    &event.payload,
                ));
                if event.event_type == TraceEventType::DecisionFailed
                    && let Some(evidence) =
                        decision_failure_evidence(event.step_id.as_deref(), &event.payload)
                {
                    self.failure_evidence.push(evidence);
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

        Ok(())
    }
}

fn fold_trace_events(trace: &ExecutionTrace) -> Result<TraceSummaryFold, TraceSummaryError> {
    let mut fold = TraceSummaryFold::default();
    for event in &trace.events {
        fold.merge_event(event, &trace.goal)?;
    }
    Ok(fold)
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
    let TraceSummaryFold {
        input_projection,
        cluster_delivery_story,
        mut routing_summary,
        routing_projection,
        goal_plan_summary,
        plan_quality_state,
        plan_quality_findings,
        plan_quality_assumptions,
        advanced_context,
        context_projection,
        guidance_guardian,
        governance_projection,
        decision_timeline,
        failure_evidence,
        adaptive_evidence,
        latest_checkpoint_id,
        latest_checkpoint_scope,
        latest_checkpoint_restore_command,
        delegation,
        saw_native_routing_signal,
        executed_steps,
        recovery_events,
        review_timeline,
        reasoning_profile,
        delight_feedback,
        framework_adapter_stage_routing,
        ..
    } = fold_trace_events(trace)?;

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

    let ResolvedTraceTerminal { terminal_status, terminal_reason, governance_next_action } =
        resolve_trace_terminal(
            persisted_terminal_status,
            persisted_terminal_reason,
            &governance_projection,
        )?;

    let governance_timeline = governance_projection.timeline;
    let governance_runtime_state = governance_projection.runtime_state;
    let governance_rollout_profile = governance_projection.rollout_profile;
    let governance_reason = governance_projection.reason;
    let governance_approval_provenance = governance_projection.approval_provenance;
    let terminal_projection = InspectTerminalProjection {
        terminal_status,
        terminal_reason: &terminal_reason,
        next_action: governance_next_action.as_deref(),
    };
    let InspectSummaryViews { inspect_context, inspect_council, inspect_timeline } =
        build_inspect_summary_views(InspectSummaryInputs {
            context_projection: &context_projection,
            review_timeline: &review_timeline,
            governance_timeline: &governance_timeline,
            reasoning_profile: reasoning_profile.as_ref(),
            decision_timeline: &decision_timeline,
            executed_steps: &executed_steps,
            recovery_events: &recovery_events,
            terminal: terminal_projection,
        });
    let ResolvedTraceArtifacts {
        goal_brief_ref,
        session_plan_brief_ref,
        run_brief_ref,
        session_audit,
        latest_framework_adapter_hook_dispatch,
    } = resolve_trace_artifacts(trace_ref.as_ref(), &trace.session_id)?;

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
        plan_quality_state,
        plan_quality_findings,
        plan_quality_assumptions,
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
        inspect_context: Some(inspect_context),
        inspect_council: Some(inspect_council),
        inspect_timeline: Some(inspect_timeline),
        review_timeline,
        session_audit,
        delight_feedback,
        framework_adapter_stage_routing,
        framework_adapter_hook_dispatch: latest_framework_adapter_hook_dispatch,
        framework_adapter_stage_failure: FrameworkAdapterStageFailureDetails::from_terminal_reason(
            &terminal_reason,
        ),
        capability_provider_trace: None,
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

struct InspectSummaryViews {
    inspect_context: InspectClosureView,
    inspect_council: InspectClosureView,
    inspect_timeline: InspectClosureView,
}

struct InspectSummaryInputs<'a> {
    context_projection: &'a TraceContextProjection,
    review_timeline: &'a [String],
    governance_timeline: &'a [String],
    reasoning_profile: Option<&'a ProfileActivationRecord>,
    decision_timeline: &'a [String],
    executed_steps: &'a [TraceStepSummary],
    recovery_events: &'a [TraceRecoveryEvent],
    terminal: InspectTerminalProjection<'a>,
}

struct ResolvedTraceTerminal {
    terminal_status: TaskStatus,
    terminal_reason: TerminalReason,
    governance_next_action: Option<String>,
}

/// Resolves the terminal projection for inspect, synthesizing a running state
/// when a governed trace paused before persisting a terminal snapshot.
fn resolve_trace_terminal(
    persisted_terminal_status: Option<TaskStatus>,
    persisted_terminal_reason: Option<TerminalReason>,
    governance_projection: &TraceGovernanceProjection,
) -> Result<ResolvedTraceTerminal, TraceSummaryError> {
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

    Ok(ResolvedTraceTerminal {
        terminal_status,
        terminal_reason,
        governance_next_action: governance_projection.next_action.clone().or_else(|| {
            governance_next_action_for_state(governance_projection.latest_state.as_deref())
        }),
    })
}

/// Builds the operator-facing closure views that `inspect` renders alongside
/// the raw trace summary fields.
fn build_inspect_summary_views(input: InspectSummaryInputs<'_>) -> InspectSummaryViews {
    let InspectSummaryInputs {
        context_projection,
        review_timeline,
        governance_timeline,
        reasoning_profile,
        decision_timeline,
        executed_steps,
        recovery_events,
        terminal,
    } = input;
    InspectSummaryViews {
        inspect_context: build_inspect_context_view(
            context_projection.summary.as_deref(),
            context_projection.credibility.as_deref(),
            &context_projection.primary_inputs,
            &context_projection.provenance,
            context_projection.staleness_reason.as_deref(),
            terminal,
        ),
        inspect_council: build_inspect_council_view(
            review_timeline,
            governance_timeline,
            reasoning_profile,
            terminal,
        ),
        inspect_timeline: build_inspect_timeline_view(
            decision_timeline,
            review_timeline,
            governance_timeline,
            executed_steps,
            recovery_events,
            terminal,
        ),
    }
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
            "boundline inspect --trace <trace>"
        }
        TraceResolutionTarget::LatestWorkspaceTrace => "boundline inspect --workspace <workspace>",
    }
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
    #[error("failed to read session audit log: {0}")]
    SessionAuditStore(SessionAuditStoreError),
    #[error("trace event {0:?} is missing a step id")]
    MissingStepId(TraceEventType),
    #[error("trace step '{0}' is missing its step kind payload")]
    MissingStepKind(String),
    #[error("trace step '{0}' completed without a matching start event")]
    MissingStartedStep(String),
    #[error("trace step kind '{0}' is unknown")]
    UnknownStepKind(String),
}

// ── Calibration Inspection ────────────────────────────────────────────

use crate::domain::calibration::{
    AuthorityZone, ControlLevelAssignment, GuardianTrustRecord, OverrideRecord, RiskLevel,
    load_calibration_policy,
};

/// A summary of calibration state for inspect output.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CalibrationInspectionView {
    pub policy_loaded: bool,
    pub policy_path: String,
    pub schema_version: String,
    pub evidence_window: u32,
    pub minimum_evidence_threshold: u32,
    pub assignments: Vec<ControlLevelAssignment>,
    pub trust_records: Vec<GuardianTrustRecord>,
    pub override_records: Vec<OverrideRecord>,
}

/// Load calibration policy and render control-level assignments for inspect.
pub fn render_calibration_inspection(
    workspace_root: &std::path::Path,
    activated_guardian_ids: &[String],
    json: bool,
) -> String {
    let policy_result = load_calibration_policy(workspace_root);
    let policy = match policy_result {
        Ok(p) => p,
        Err(_) => {
            if json {
                return r#"{"calibration":{"error":"failed to load calibration policy"}}"#
                    .to_string();
            }
            return "Calibration: failed to load calibration policy\n".to_string();
        }
    };

    // For now, assign default levels to all activated guardians.
    // Full authority zone and risk level resolution happens in council adjudication.
    let assignments: Vec<ControlLevelAssignment> = activated_guardian_ids
        .iter()
        .map(|id| policy.resolve_level(id, AuthorityZone::Green, RiskLevel::Low, None))
        .collect();

    let trust_records: Vec<GuardianTrustRecord> =
        activated_guardian_ids.iter().map(|id| GuardianTrustRecord::new(id)).collect();

    if json {
        let view = CalibrationInspectionView {
            policy_loaded: true,
            policy_path: workspace_root
                .join(".boundline")
                .join("calibration-policy.toml")
                .to_string_lossy()
                .to_string(),
            schema_version: policy.schema_version.clone(),
            evidence_window: policy.evidence_window,
            minimum_evidence_threshold: policy.minimum_evidence_threshold,
            assignments: assignments.clone(),
            trust_records: trust_records.clone(),
            override_records: Vec::new(),
        };
        serde_json::to_string_pretty(&view)
            .unwrap_or_else(|_| r#"{"calibration":{"error":"serialization failed"}}"#.to_string())
    } else {
        let mut lines = Vec::new();
        lines.push("Guardian Calibration".to_string());
        lines.push(format!(
            "  Policy: {} (schema v{})",
            if policy.entries.is_empty() {
                "built-in all-advisory default"
            } else {
                "loaded from .boundline/calibration-policy.toml"
            },
            policy.schema_version
        ));
        lines.push(format!(
            "  Evidence window: {} sessions (min sample: {})",
            policy.evidence_window, policy.minimum_evidence_threshold
        ));
        lines.push(String::new());

        for assignment in &assignments {
            lines.push(format!(
                "  {}: {:?} (guardian confidence: {:.2}, calibrated: {:.2})",
                assignment.rule_id,
                assignment.assigned_level,
                assignment.guardian_confidence,
                assignment.calibrated_confidence
            ));
            lines.push(format!("    Reason: {}", assignment.reason));
            if let Some(from) = assignment.degraded_from {
                lines.push(format!(
                    "    Degraded from: {from:?} — {}",
                    assignment.degradation_reason.as_deref().unwrap_or("unknown")
                ));
            }
            lines.push(String::new());
        }

        for record in &trust_records {
            if record.adjudicated_count() > 0 {
                let tpr_str = record
                    .true_positive_rate()
                    .map(|r| format!("{:.2}", r))
                    .unwrap_or_else(|| "N/A".to_string());
                lines.push(format!(
                    "  Trust: {} — TP={}, FP={}, deferred={}, TPR={}",
                    record.rule_id,
                    record.true_positive_count,
                    record.false_positive_count,
                    record.deferred_count,
                    tpr_str
                ));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::projections::string_array_field;
    use super::resolve::resolve_session_trace_ref;
    use super::timeline::reviewer_line;
    use super::{
        InspectCommandError, TraceResolutionTarget, TraceSummaryError, adaptive_evidence_lines,
        corrected_command, decision_failure_evidence, decision_timeline_lines, failure_headline,
        governance_timeline_line, inspection_target_for, merge_guidance_projection_from_payload,
        parse_step_kind, render_error, resolve_trace_path, review_timeline_line, success_headline,
        summarize_trace, synthesized_in_progress_reason,
    };
    use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
    use crate::adapters::trace_store::{FileTraceStore, TraceStore};
    use crate::domain::completion_verification::{
        ClaimInferenceConfidence, CompletionClaim, CompletionClaimKind, CompletionClaimSource,
        CompletionRequiredAction, CompletionVerificationFinding, CompletionVerificationFindingKind,
        CompletionVerificationFindingSeverity, CompletionVerificationProjection,
        CompletionVerificationScope, CompletionVerificationState,
    };
    use crate::domain::guidance::GuidanceGuardianProjection;
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::plan::Plan;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::step::Step;
    use crate::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};
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
    fn load_trace_resolves_relative_session_trace_ref_against_workspace() {
        let workspace = temp_workspace("boundline-inspect-relative-session-trace");
        let relative_trace_ref = PathBuf::from("relative/trace.json");
        let persisted_trace_path = workspace.join(&relative_trace_ref);
        fs::create_dir_all(persisted_trace_path.parent().unwrap()).unwrap();
        fs::write(&persisted_trace_path, serde_json::to_vec_pretty(&terminal_trace()).unwrap())
            .unwrap();

        let store = FileSessionStore::for_workspace(&workspace);
        let record = ActiveSessionRecord {
            session_id: "relative-session".to_string(),
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
            latest_trace_ref: Some(relative_trace_ref.to_string_lossy().into_owned()),
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        };
        store.persist(&record).unwrap();

        let (target, loaded_path, loaded_trace) =
            super::load_trace(None, Some(&workspace), None).unwrap();

        assert_eq!(target, TraceResolutionTarget::SessionTraceRef);
        assert_eq!(loaded_path, persisted_trace_path);
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
            active_execution_run_id: None,
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
            active_execution_run_id: None,
        };

        store.persist(&active_record).map_err(|error| error.to_string())?;
        store.persist_without_select(&selected_record).map_err(|error| error.to_string())?;

        let selected_trace = resolve_session_trace_ref(&workspace, Some("selected-session"))
            .map_err(|error| error.to_string())?;
        assert_eq!(selected_trace.as_deref(), Some("selected/trace.json"));

        let active_trace =
            resolve_session_trace_ref(&workspace, None).map_err(|error| error.to_string())?;
        assert_eq!(active_trace.as_deref(), Some("active/trace.json"));

        let active_after = store
            .load()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected active session record".to_string())?;
        assert_eq!(active_after.session_id, active_record.session_id);

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
            active_execution_run_id: None,
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
            "boundline inspect --trace <trace>"
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

    #[test]
    fn calibration_inspection_view_handles_missing_policy() {
        let ws = temp_workspace("calib");
        let guardians = vec!["g1".to_string(), "g2".to_string()];

        let plain = super::render_calibration_inspection(&ws, &guardians, false);
        assert!(plain.contains("built-in all-advisory default"));

        let json_out = super::render_calibration_inspection(&ws, &guardians, true);
        assert!(json_out.contains("\"policy_loaded\""));
    }

    #[test]
    fn calibration_inspection_view_handles_valid_policy() {
        let ws = temp_workspace("calib-valid");
        let guardians = vec!["test-guardian".to_string()];

        // Write a valid calibration policy
        let policy_path = ws.join(".boundline").join("calibration-policy.toml");
        std::fs::write(
            &policy_path,
            r#"schema_version = "1.0"
evidence_window = 100
minimum_evidence_threshold = 10

[[entries]]
rule_id = "test-guardian"
authority_zone = "green"
risk_level = "low"
default_level = "catch"
green_level = "catch"
yellow_level = "rule"
red_level = "rule"
confidence_threshold = 0.85

[entries.override_policy]
allowed_roles = ["operator"]
required_evidence = ["reason"]
time_limited = false
"#,
        )
        .unwrap();

        // Write a mock trust record
        let trust_path = ws.join(".boundline").join("trust-records.json");
        std::fs::write(
            &trust_path,
            r#"
[
  {
    "rule_id": "test-guardian",
    "true_positive_count": 5,
    "false_positive_count": 1,
    "deferred_count": 0,
    "last_updated": "123"
  }
]
"#,
        )
        .unwrap();

        let plain = super::render_calibration_inspection(&ws, &guardians, false);
        assert!(plain.contains("Guardian Calibration"));
        assert!(plain.contains("loaded from .boundline/calibration-policy.toml"));
        assert!(plain.contains("test-guardian: Catch"));

        let json_out = super::render_calibration_inspection(&ws, &guardians, true);
        let parsed: serde_json::Value = serde_json::from_str(&json_out).unwrap();
        assert!(parsed.get("policy_loaded").unwrap().as_bool().unwrap());
        assert_eq!(parsed.get("schema_version").unwrap().as_str().unwrap(), "1.0");
        assert_eq!(parsed.get("evidence_window").unwrap().as_u64().unwrap(), 100);
    }

    #[test]
    fn execute_inspect_falls_back_to_active_completion_verification_summary() {
        let workspace = temp_workspace("boundline-inspect-completion-fallback");
        let session_id = "session-completion-fallback";
        let mut trace = ExecutionTrace::new("task-completion-fallback", session_id, "Fix the bug");
        trace.record_event(
            TraceEventType::StepStarted,
            Some("verify".to_string()),
            1,
            json!({"step_kind": "agent"}),
        );
        let trace_path = match FileTraceStore::for_workspace(&workspace).persist(&trace) {
            Ok(path) => path,
            Err(error) => panic!("failed to persist trace fixture: {error}"),
        };

        let mut task = completion_task(&workspace, session_id, "task-completion-fallback");
        let projection = CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::ProofRequired,
            scope: CompletionVerificationScope::Task,
            claim: Some(CompletionClaim {
                claim_id: "claim-fallback".to_string(),
                kind: CompletionClaimKind::BugFixed,
                scope: CompletionVerificationScope::Task,
                source: CompletionClaimSource::RuntimeInference,
                confidence: Some(ClaimInferenceConfidence::High),
                summary: "bug fix remains unproven".to_string(),
                supporting_signals: vec!["goal_text".to_string(), "changed_files".to_string()],
            }),
            completion_blocked_claims: vec![CompletionClaimKind::BugFixed],
            completion_evidence_refs: vec!["evidence-proof-1".to_string()],
            completion_verification_findings: vec![CompletionVerificationFinding {
                kind: CompletionVerificationFindingKind::StaleProof,
                severity: CompletionVerificationFindingSeverity::Blocking,
                message: "The previously passing proof is stale because workspace content changed after proof execution.".to_string(),
                proof_ref: Some("proof-1".to_string()),
                task_id: Some("task-completion-fallback".to_string()),
                changed_paths: vec!["src/lib.rs".to_string(), "Cargo.toml".to_string()],
                required_action: CompletionRequiredAction::RerunProof,
            }],
            child_summary: None,
        };
        if let Err(error) = task.context.set_completion_verification_projection(&projection) {
            panic!("failed to attach completion verification projection: {error}");
        }

        let session = ActiveSessionRecord {
            session_id: session_id.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Fix the bug".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: Some(task),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Blocked,
            latest_terminal_reason: None,
            latest_trace_ref: Some(trace_path.to_string_lossy().into_owned()),
            created_at: 1,
            updated_at: 2,
            governance_lifecycle: None,
            project_scale: None,
            delight_feedback: None,
            latest_voting: None,
            active_execution_run_id: None,
        };
        if let Err(error) = FileSessionStore::for_workspace(&workspace).persist(&session) {
            panic!("failed to persist session fixture: {error}");
        }

        let report = match super::execute_inspect(None, Some(&workspace), Some(session_id), false) {
            Ok(report) => report,
            Err(error) => panic!("inspect should succeed with fallback summary: {error}"),
        };

        assert_eq!(report.exit_status, crate::cli::CommandExitStatus::NonSuccess);
        assert!(
            report
                .terminal_output
                .contains("terminal_reason: The previously passing proof is stale because workspace content changed after proof execution."),
            "{}",
            report.terminal_output
        );
        assert!(
            report.terminal_output.contains("completion_verification_state: proof_required"),
            "{}",
            report.terminal_output
        );
        assert!(
            report
                .terminal_output
                .contains("completion_verification_changed_paths: src/lib.rs, Cargo.toml"),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    fn render_completion_verification_lines_uses_default_task_id_and_evidence_refs() {
        let projection = CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::Blocked,
            scope: CompletionVerificationScope::Task,
            claim: Some(CompletionClaim {
                claim_id: "claim-render".to_string(),
                kind: CompletionClaimKind::BuildClean,
                scope: CompletionVerificationScope::Task,
                source: CompletionClaimSource::OperatorConfirmed,
                confidence: Some(ClaimInferenceConfidence::Medium),
                summary: "build cleanliness still needs proof".to_string(),
                supporting_signals: vec!["selected_proof_command".to_string()],
            }),
            completion_blocked_claims: vec![CompletionClaimKind::BuildClean],
            completion_evidence_refs: vec!["evidence-1".to_string(), "evidence-2".to_string()],
            completion_verification_findings: vec![CompletionVerificationFinding {
                kind: CompletionVerificationFindingKind::MissingProof,
                severity: CompletionVerificationFindingSeverity::Blocking,
                message: "No proving command is available for the requested claim.".to_string(),
                proof_ref: None,
                task_id: None,
                changed_paths: vec!["Cargo.toml".to_string()],
                required_action: CompletionRequiredAction::RunProof,
            }],
            child_summary: None,
        };

        let lines =
            super::render_completion_verification_lines(&projection, Some("task-render-default"));
        let rendered = lines.join("\n");

        assert!(rendered.contains("completion_verification_state: blocked"), "{rendered}");
        assert!(rendered.contains("completion_claim_kind: build_clean"), "{rendered}");
        assert!(
            rendered.contains("completion_evidence_refs: evidence-1, evidence-2"),
            "{rendered}"
        );
        assert!(
            rendered.contains("completion_verification_task_id: task-render-default"),
            "{rendered}"
        );
        assert!(
            rendered.contains("completion_verification_required_action: run_proof"),
            "{rendered}"
        );
    }

    fn completion_task(workspace: &Path, session_id: &str, task_id: &str) -> Task {
        let request = TaskRunRequest {
            goal: "Fix the bug".to_string(),
            input: json!({}),
            session_id: session_id.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let step = match Step::agent("verify", "tester", json!({"goal": "Fix the bug"})) {
            Ok(step) => step,
            Err(error) => panic!("failed to build step fixture: {error}"),
        };
        let plan = match Plan::new(vec![step]) {
            Ok(plan) => plan,
            Err(error) => panic!("failed to build plan fixture: {error}"),
        };
        match Task::new(task_id, &request, plan) {
            Ok(task) => task,
            Err(error) => panic!("failed to build task fixture: {error}"),
        }
    }
}
