use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::session::governance_next_action_for_state;
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::TaskStatus;
use crate::domain::trace::{
    ExecutionTrace, TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
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
    })
}

pub fn render_error(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    error: &InspectCommandError,
) -> String {
    if let InspectCommandError::InvalidSession(message) = error {
        return output::render_session_error("inspect", message, Some("synod start"));
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
        InspectCommandError::InvalidSession(_) => {
            unreachable!("invalid sessions are rendered separately")
        }
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

pub fn summarize_trace(
    trace_ref: impl AsRef<Path>,
    trace: &ExecutionTrace,
) -> Result<TraceSummaryView, TraceSummaryError> {
    let terminal_status = trace.terminal_status.ok_or(TraceSummaryError::MissingTerminalStatus)?;
    let terminal_reason =
        trace.terminal_reason.clone().ok_or(TraceSummaryError::MissingTerminalReason)?;
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
    let mut latest_governance_state: Option<String> = None;
    let mut step_indexes: HashMap<String, usize> = HashMap::new();
    let mut executed_steps: Vec<TraceStepSummary> = Vec::new();
    let mut recovery_events: Vec<TraceRecoveryEvent> = Vec::new();
    let mut governance_timeline: Vec<String> = Vec::new();
    let mut review_timeline: Vec<String> = Vec::new();

    for event in &trace.events {
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
            }
            TraceEventType::TerminalRecorded | TraceEventType::ReviewerStarted => {}
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
            TraceEventType::DecisionCreated
            | TraceEventType::DecisionDispatched
            | TraceEventType::DecisionVerified
            | TraceEventType::DecisionFailed
            | TraceEventType::DecisionRecovered
            | TraceEventType::GoalPlanCreated
            | TraceEventType::FlowInferred => {
                // Decision loop events — handled by decision-specific inspection
            }
        }
    }

    Ok(TraceSummaryView {
        trace_ref: trace_ref.as_ref().to_string_lossy().into_owned(),
        goal: trace.goal.clone(),
        authored_input_summary,
        authored_input_sources,
        authored_input_deduplicated_sources,
        clarification_headline,
        clarification_prompt,
        clarification_missing_fields,
        requested_governance_runtime,
        requested_governance_risk,
        requested_governance_zone,
        requested_governance_owner,
        executed_steps,
        recovery_events,
        governance_timeline,
        governance_next_action: governance_next_action_for_state(
            latest_governance_state.as_deref(),
        ),
        review_timeline,
        terminal_status,
        terminal_reason,
        duration: trace.duration_millis(),
    })
}

fn load_trace(
    trace: Option<&Path>,
    workspace: Option<&Path>,
) -> Result<(TraceResolutionTarget, PathBuf, ExecutionTrace), InspectCommandError> {
    let session_trace_ref = workspace.map(resolve_session_trace_ref).transpose()?.flatten();
    let (target, trace_path) = resolve_trace_path(trace, workspace, session_trace_ref.as_deref())?;

    let trace = match target {
        TraceResolutionTarget::LatestWorkspaceTrace => {
            let workspace_path =
                workspace.expect("workspace is required for latest workspace trace resolution");
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
            "cargo run --bin synod -- inspect --trace <trace>"
        }
        TraceResolutionTarget::LatestWorkspaceTrace => {
            "cargo run --bin synod -- inspect --workspace <workspace>"
        }
    }
}

fn review_timeline_line(event_type: TraceEventType, payload: &serde_json::Value) -> Option<String> {
    match event_type {
        TraceEventType::ReviewStarted => payload
            .get("review_trigger")
            .and_then(|value| value.as_str())
            .map(|trigger| format!("review_trigger: {trigger}")),
        TraceEventType::ReviewTriggerIgnored => payload
            .get("review_trigger")
            .and_then(|value| value.as_str())
            .map(|trigger| format!("review_trigger_ignored: {trigger}")),
        TraceEventType::ReviewerCompleted => reviewer_line(payload),
        TraceEventType::ReviewVoteResolved => payload
            .get("summary")
            .and_then(|value| value.as_str())
            .map(|summary| format!("review_vote: {summary}"))
            .or_else(|| {
                payload.get("vote_resolution").map(|resolution| {
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
            .get("review_outcome")
            .and_then(|value| value.as_str())
            .map(|outcome| format!("review_outcome: {outcome}"))
            .or_else(|| {
                payload
                    .get("failure_reason")
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
            "governance_started: {}{}{}",
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
                .unwrap_or_default()
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
            "governance_awaiting_approval: {} ({}){}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload.get("approval_state").and_then(|value| value.as_str()).unwrap_or("unknown"),
            payload
                .get("run_ref")
                .and_then(|value| value.as_str())
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default()
        )),
        TraceEventType::GovernanceCompleted => Some(format!(
            "governance_completed: {}{}",
            payload
                .get("headline")
                .and_then(|value| value.as_str())
                .unwrap_or("governed packet ready"),
            payload
                .get("packet_ref")
                .and_then(|value| value.as_str())
                .map(|packet_ref| format!(" [{packet_ref}]"))
                .unwrap_or_default()
        )),
        TraceEventType::GovernanceBlocked => Some(format!(
            "governance_blocked: {}",
            payload.get("reason").and_then(|value| value.as_str()).unwrap_or("blocked")
        )),
        TraceEventType::GovernancePacketRejected => Some(format!(
            "governance_packet_rejected: {}",
            payload.get("reason").and_then(|value| value.as_str()).unwrap_or("packet rejected")
        )),
        _ => None,
    }
}

fn reviewer_line(payload: &serde_json::Value) -> Option<String> {
    let reviewer_id = payload.get("reviewer_id").and_then(|value| value.as_str())?;

    if let Some(finding) = payload.get("finding") {
        let disposition =
            finding.get("disposition").and_then(|value| value.as_str()).unwrap_or("unknown");
        let summary =
            finding.get("summary").and_then(|value| value.as_str()).unwrap_or("review finding");
        let role = payload.get("reviewer_role").and_then(|value| value.as_str());
        return Some(match role {
            Some(role) => format!("reviewer {reviewer_id} ({role}) {disposition}: {summary}"),
            None => format!("reviewer {reviewer_id} {disposition}: {summary}"),
        });
    }

    payload
        .get("failure_reason")
        .and_then(|value| value.as_str())
        .map(|reason| format!("reviewer {reviewer_id} failed: {reason}"))
}

fn success_headline(payload: &serde_json::Value, attempts: usize) -> String {
    if let Some(headline) = payload
        .get("output")
        .and_then(|output| output.get("workspace_slice"))
        .and_then(|slice| slice.get("headline"))
        .and_then(|value| value.as_str())
    {
        return format!("adaptive slice {headline}");
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

    if let Some(validation) = payload.get("output").and_then(|output| output.get("validation")) {
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
    if let Some(validation) =
        payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
    {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let exit_code = validation.get("exit_code").and_then(|value| value.as_i64()).unwrap_or(-1);
        return format!(
            "validation failed after {attempts} attempt(s) via {command} (exit_code={exit_code})"
        );
    }

    format!("failed after {attempts} attempt(s)")
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
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        InspectCommandError, TraceResolutionTarget, TraceSummaryError, corrected_command,
        failure_headline, inspection_target_for, render_error, resolve_session_trace_ref,
        success_headline, summarize_trace,
    };
    use crate::adapters::session_store::SessionStoreError;
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::task::{TaskStatus, TerminalReason};
    use crate::domain::trace::{ExecutionTrace, TraceEventType};

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".synod")).unwrap();
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
    fn resolve_session_trace_ref_maps_invalid_records_to_invalid_session_errors() {
        let workspace = temp_workspace("synod-inspect-invalid-session");
        let invalid_record = ActiveSessionRecord {
            session_id: "session-inspect".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::GoalCaptured,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 20,
        };
        fs::write(
            workspace.join(".synod/session.json"),
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
            "cargo run --bin synod -- inspect --trace <trace>"
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
}
