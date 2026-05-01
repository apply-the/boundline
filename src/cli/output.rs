use serde_json::Value;

use crate::cli::diagnostics::{DiagnosticsReport, DiagnosticsStatus};
use crate::cli::{CliValidationError, CommandExitStatus, DeveloperCommand};
use crate::domain::cluster::{ClusterInspectReport, ClusterMemberState};
use crate::domain::goal_plan::GoalPlanFlowState;
use crate::domain::session::RoutingOutcome;
use crate::domain::session::{
    CompatibilityFollowUpView, ContinuityAuthority, RoutingMode, RoutingSource, SessionStatus,
    SessionStatusView, governance_packet_provenance_text,
};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskRunResponse, TaskStatus};
use crate::domain::trace::{ExecutionTrace, TraceEventType, TraceSummaryView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExitCode {
    Success,
    NonSuccess,
    InvalidInvocation,
    TraceReadFailure,
}

impl CommandExitCode {
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::NonSuccess => 1,
            Self::InvalidInvocation => 2,
            Self::TraceReadFailure => 3,
        }
    }

    pub const fn for_status(status: CommandExitStatus) -> Self {
        match status {
            CommandExitStatus::Succeeded => Self::Success,
            CommandExitStatus::NonSuccess => Self::NonSuccess,
            CommandExitStatus::InvalidInvocation => Self::InvalidInvocation,
            CommandExitStatus::TraceReadFailure => Self::TraceReadFailure,
        }
    }
}

pub fn unimplemented_message(command: &DeveloperCommand) -> String {
    format!("`{}` is not implemented yet", command_name(command))
}

pub fn command_name(command: &DeveloperCommand) -> &'static str {
    match command {
        DeveloperCommand::Doctor { .. } => "doctor",
        DeveloperCommand::Start { .. } => "start",
        DeveloperCommand::Capture { .. } => "capture",
        DeveloperCommand::Flow { .. } => "flow",
        DeveloperCommand::Plan { .. } => "plan",
        DeveloperCommand::Step { .. } => "step",
        DeveloperCommand::Run { .. } => "run",
        DeveloperCommand::Workflow { .. } => "workflow",
        DeveloperCommand::Inspect { .. } => "inspect",
        DeveloperCommand::Status { .. } => "status",
        DeveloperCommand::Next { .. } => "next",
        DeveloperCommand::Init { .. } => "init",
        DeveloperCommand::Config { .. } => "config",
        DeveloperCommand::Cluster { .. } => "cluster",
    }
}

pub fn render_cluster_init(cluster_id: &str, cluster_path: &str, members: &[String]) -> String {
    let mut lines = vec![
        "cluster: initialized".to_string(),
        format!("cluster_id: {cluster_id}"),
        format!("cluster_file: {cluster_path}"),
        "members:".to_string(),
    ];
    for member in members {
        lines.push(format!("- {member}"));
    }
    lines.join("\n")
}

pub fn render_cluster_status(report: &ClusterInspectReport) -> String {
    let mut lines = vec![
        "cluster: status".to_string(),
        format!("cluster_id: {}", report.cluster_id),
        format!("primary_workspace: {}", report.primary_workspace_ref),
        "members:".to_string(),
    ];

    for member in &report.members {
        let mut line =
            format!("- {} [{}]", member.workspace_ref, cluster_member_state_text(member.state));
        if let Some(status) = member.latest_status {
            line.push_str(&format!(" status={}", session_status_text(status)));
        }
        line.push_str(&format!(" {}", member.headline));
        lines.push(line);
    }

    lines.join("\n")
}

pub fn render_cluster_inspect(report: &ClusterInspectReport) -> String {
    let mut lines = vec![
        "cluster: inspect".to_string(),
        format!("cluster_id: {}", report.cluster_id),
        format!("primary_workspace: {}", report.primary_workspace_ref),
        "members:".to_string(),
    ];

    for member in &report.members {
        let trace_text = member.latest_trace_ref.as_deref().unwrap_or("<missing>");
        lines.push(format!(
            "- {} [{}] trace={} {}",
            member.workspace_ref,
            cluster_member_state_text(member.state),
            trace_text,
            member.headline
        ));
    }

    lines.join("\n")
}

pub fn validation_error_message(error: &CliValidationError) -> String {
    error.to_string()
}

pub fn render_route_outcome(outcome: &RoutingOutcome) -> String {
    format!("routing: {} ({}) - {}", outcome.mode.as_str(), outcome.source.as_str(), outcome.reason)
}

pub fn render_goal_plan_flow_state(flow_state: &GoalPlanFlowState) -> String {
    format!("flow_state: {}", flow_state.summary_text())
}

pub fn render_diagnostics(report: &DiagnosticsReport) -> String {
    let readiness = if report.ready { "ready" } else { "not ready" };
    let mut lines = vec![
        format!("doctor: {readiness} for workspace {}", report.workspace_ref),
        format!("assistant_hint: Diagnostic output format is optimized for chat parsing."),
    ];

    for check in &report.checks {
        let status = match check.status {
            DiagnosticsStatus::Passed => "passed",
            DiagnosticsStatus::Failed => "failed",
        };
        lines.push(format!("- {}: {} - {}", check.name, status, check.message));
    }

    if !report.suggested_actions.is_empty() {
        lines.push("actions:".to_string());
        for action in &report.suggested_actions {
            lines.push(format!("- {action}"));
        }
    }

    lines.join("\n")
}

pub fn render_run_trace(
    command_name: &str,
    trace: Option<&ExecutionTrace>,
    response: &TaskRunResponse,
    next_command: &str,
) -> String {
    let mut lines = vec![format!("{command_name}: {}", response.terminal_reason.message)];

    if let Some(trace) = trace {
        lines.insert(0, format!("goal: {}", trace.goal));

        if let Some(input) = trace.events.iter().find_map(|event| {
            (event.event_type == TraceEventType::TaskStarted)
                .then(|| event.payload.get("input"))
                .flatten()
        }) {
            if let Some(authored_input_summary) =
                input.get("authored_input_summary").and_then(Value::as_str)
            {
                lines.push(format!("authored_input_summary: {authored_input_summary}"));
            }
            if let Some(clarification_headline) =
                input.get("clarification_headline").and_then(Value::as_str)
            {
                lines.push(format!("clarification_headline: {clarification_headline}"));
            }
            if let Some(clarification_prompt) =
                input.get("clarification_prompt").and_then(Value::as_str)
            {
                lines.push(format!("clarification_prompt: {clarification_prompt}"));
            }
        }

        for event in &trace.events {
            match event.event_type {
                TraceEventType::TaskStarted
                | TraceEventType::TerminalRecorded
                | TraceEventType::ReviewerStarted => {}
                TraceEventType::FlowSelected => {
                    let flow_name = event
                        .payload
                        .get("flow_name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-flow");
                    let stage_id = event
                        .payload
                        .get("current_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("flow {flow_name} selected at {stage_id}"));
                }
                TraceEventType::StageTransitioned => {
                    let from_stage = event
                        .payload
                        .get("from_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let to_stage = event
                        .payload
                        .get("to_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("stage {from_stage} -> {to_stage}"));
                }
                TraceEventType::StepStarted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let step_kind =
                        event.payload.get("step_kind").and_then(Value::as_str).unwrap_or("step");
                    lines.push(format!("step {step_id} ({step_kind}) started"));
                }
                TraceEventType::StepCompleted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let status =
                        event.payload.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("step {step_id} {status}"));

                    if let Some(changed_files) = event
                        .payload
                        .get("output")
                        .and_then(|output| output.get("changed_files"))
                        .and_then(value_as_string_list)
                        && !changed_files.is_empty()
                    {
                        lines.push(format!("changed_files: {}", changed_files.join(", ")));
                    }

                    if let Some(validation_line) = validation_line_from_event(&event.payload) {
                        lines.push(validation_line);
                    }
                }
                TraceEventType::DecisionCreated => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    let decision_type = event
                        .payload
                        .get("decision_type")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let target =
                        event.payload.get("target").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!(
                        "decision {decision_id} created: {decision_type} -> {target}"
                    ));
                }
                TraceEventType::DecisionDispatched => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    let target =
                        event.payload.get("target").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("decision {decision_id} dispatched: {target}"));
                }
                TraceEventType::DecisionVerified => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} verified"));
                }
                TraceEventType::DecisionFailed => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} failed"));
                }
                TraceEventType::DecisionRecovered => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} recovered"));
                }
                TraceEventType::GovernanceSelected
                | TraceEventType::GovernanceStarted
                | TraceEventType::GovernanceDecisionRecorded
                | TraceEventType::GovernanceAwaitingApproval
                | TraceEventType::GovernanceCompleted
                | TraceEventType::GovernanceBlocked
                | TraceEventType::GovernancePacketRejected => {
                    if let Some(line) = governance_event_line(event.event_type, &event.payload) {
                        lines.push(line);
                    }
                }
                TraceEventType::RetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("retry for {step_id}: {reason}"));
                }
                TraceEventType::StageRetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("stage retry for {step_id}: {reason}"));
                }
                TraceEventType::Replanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("replan after {step_id}: {reason}"));
                }
                TraceEventType::StageReplanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("stage replan after {step_id}: {reason}"));
                }
                TraceEventType::StageFailed => {
                    let stage_id = event
                        .payload
                        .get("stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("stage failed");
                    lines.push(format!("stage {stage_id} failed: {reason}"));
                }
                TraceEventType::ReviewStarted
                | TraceEventType::ReviewTriggerIgnored
                | TraceEventType::ReviewerCompleted
                | TraceEventType::ReviewVoteResolved
                | TraceEventType::ReviewAdjudicated
                | TraceEventType::ReviewTerminalRecorded => {
                    if let Some(line) = review_event_line(event.event_type, &event.payload) {
                        lines.push(line);
                    }
                }
                TraceEventType::GoalPlanCreated => {
                    let goal =
                        event.payload.get("goal").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("goal plan created: {goal}"));
                }
                TraceEventType::FlowInferred => {
                    let flow =
                        event.payload.get("flow_name").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("flow inferred: {flow}"));
                }
            }
        }

        lines.push(render_run_execution_condition(response));
    }

    if trace.is_none() {
        lines.push(render_run_execution_condition(response));
    }

    if let Some(workspace_slice) = adaptive_workspace_slice_summary(&response.final_context.state) {
        lines.push(format!("workspace_slice: {workspace_slice}"));
    }

    if let Some(attempt_lineage) = adaptive_attempt_lineage_summary(&response.final_context.state) {
        lines.push(format!("attempt_lineage: {attempt_lineage}"));
    }

    if let Some(candidate_family) = adaptive_candidate_family_summary(&response.final_context.state)
    {
        lines.push(format!("candidate_family: {candidate_family}"));
    }

    if let Some(selection_reason) = adaptive_selection_reason_summary(&response.final_context.state)
    {
        lines.push(format!("selection_reason: {selection_reason}"));
    }

    if let Some(rejected_candidates) =
        adaptive_rejected_candidates_summary(&response.final_context.state)
    {
        lines.push(format!("rejected_candidates: {rejected_candidates}"));
    }

    if let Some(exhaustion_reason) =
        adaptive_exhaustion_reason_summary(&response.final_context.state)
    {
        lines.push(format!("adaptive_exhaustion: {exhaustion_reason}"));
    }

    lines.push(format!("terminal_status: {}", task_status_text(response.terminal_status)));
    lines.push(format!("terminal_reason: {}", response.terminal_reason.message));
    lines.push(format!("trace: {}", response.trace_location));
    lines.push(format!("next_command: {next_command}"));
    lines.join("\n")
}

pub fn render_trace_summary(
    summary: &TraceSummaryView,
    inspection_target: &str,
    next_command: &str,
) -> String {
    let mut lines = vec![
        format!("inspection_target: {inspection_target}"),
        format!("trace: {}", summary.trace_ref),
        format!("goal: {}", summary.goal),
    ];

    if let Some(routing_summary) = &summary.routing_summary {
        lines.push(routing_summary.clone());
    }

    lines.push(render_trace_execution_condition(summary));

    if let Some(goal_plan_summary) = &summary.goal_plan_summary {
        lines.push(format!("goal_plan_summary: {goal_plan_summary}"));
    }

    if let Some(authored_input_summary) = &summary.authored_input_summary {
        lines.push(format!("authored_input_summary: {authored_input_summary}"));
    }

    if !summary.authored_input_sources.is_empty() {
        lines
            .push(format!("authored_input_sources: {}", summary.authored_input_sources.join(", ")));
    }

    if !summary.authored_input_deduplicated_sources.is_empty() {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            summary.authored_input_deduplicated_sources.join(", ")
        ));
    }

    if let Some(clarification_headline) = &summary.clarification_headline {
        lines.push(format!("clarification_headline: {clarification_headline}"));
    }

    if let Some(clarification_prompt) = &summary.clarification_prompt {
        lines.push(format!("clarification_prompt: {clarification_prompt}"));
    }

    if !summary.clarification_missing_fields.is_empty() {
        lines.push(format!(
            "clarification_missing_fields: {}",
            summary.clarification_missing_fields.join(", ")
        ));
    }

    if let Some(requested_governance_runtime) = &summary.requested_governance_runtime {
        lines.push(format!("requested_governance_runtime: {requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &summary.requested_governance_risk {
        lines.push(format!("requested_governance_risk: {requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &summary.requested_governance_zone {
        lines.push(format!("requested_governance_zone: {requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &summary.requested_governance_owner {
        lines.push(format!("requested_governance_owner: {requested_governance_owner}"));
    }

    if !summary.decision_timeline.is_empty() {
        lines.push("decision_timeline:".to_string());
        lines.extend(summary.decision_timeline.iter().cloned());
    }

    if !summary.failure_evidence.is_empty() {
        lines.push("failure_evidence:".to_string());
        lines.extend(summary.failure_evidence.iter().cloned());
    }

    if !summary.adaptive_evidence.is_empty() {
        lines.push("adaptive_evidence:".to_string());
        lines.extend(summary.adaptive_evidence.iter().cloned());
    }

    for step in &summary.executed_steps {
        lines.push(format!(
            "step {} ({}) {} [{} attempt(s)] - {}",
            step.step_id,
            step_kind_text(step.step_kind),
            step_status_text(step.final_status),
            step.attempts,
            step.headline,
        ));
    }

    for recovery in &summary.recovery_events {
        let label = match recovery.event_type {
            TraceEventType::RetryScheduled => "retry",
            TraceEventType::StageRetryScheduled => "stage_retry",
            TraceEventType::Replanned => "replan",
            TraceEventType::StageReplanned => "stage_replan",
            TraceEventType::FlowSelected => "flow",
            TraceEventType::StageTransitioned => "stage",
            TraceEventType::StageFailed => "stage_failure",
            _ => "recovery",
        };
        lines.push(format!("{label}: {}", recovery.trigger));
    }

    lines.extend(summary.governance_timeline.iter().cloned());

    if let Some(governance_next_action) = &summary.governance_next_action {
        lines.push(format!("governance_next_action: {governance_next_action}"));
    }

    lines.extend(summary.review_timeline.iter().cloned());

    lines.push(format!("terminal_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("terminal_reason: {}", summary.terminal_reason.message));
    lines.push(format!("next_command: {next_command}"));

    if let Some(duration) = summary.duration {
        lines.push(format!("duration_ms: {duration}"));
    }

    lines.join("\n")
}

pub fn render_inspect_failure(
    inspection_target: &str,
    trace_ref: Option<&str>,
    workspace_ref: Option<&str>,
    terminal_reason: &str,
    corrected_command: &str,
) -> String {
    let mut lines = vec![
        "inspect: trace read failure".to_string(),
        format!("inspection_target: {inspection_target}"),
        format!("terminal_reason: {terminal_reason}"),
    ];

    if let Some(trace_ref) = trace_ref {
        lines.push(format!("trace: {trace_ref}"));
    }

    if let Some(workspace_ref) = workspace_ref {
        lines.push(format!("workspace_ref: {workspace_ref}"));
    }

    lines.push("next_command: /synod-inspect".to_string());
    lines.push(format!("corrected_command: {corrected_command}"));
    lines.join("\n")
}

pub fn render_session_projection_prefix(view: &SessionStatusView) -> String {
    [
        render_route_outcome(&routing_outcome_for_status_view(view)),
        render_session_execution_condition(view),
    ]
    .join("\n")
}

pub fn render_session_status(view: &SessionStatusView) -> String {
    let mut lines = vec![
        format!("session_id: {}", view.session_id),
        format!("workspace_ref: {}", view.workspace_ref),
    ];

    if let Some(goal) = &view.goal {
        lines.push(format!("goal: {goal}"));
    }

    lines.extend(render_session_projection_prefix(view).lines().map(str::to_string));

    if let Some(continuity_authority) = view.continuity_authority {
        lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    }

    if let Some(compatibility_follow_up) = &view.compatibility_follow_up {
        lines.extend(render_compatibility_follow_up_lines(
            compatibility_follow_up,
            "compatibility_routing",
            "compatibility_execution_condition",
            "compatibility_follow_up_command",
        ));
    }

    if let Some(authored_input_summary) = &view.authored_input_summary {
        lines.push(format!("authored_input_summary: {authored_input_summary}"));
    }

    if let Some(authored_input_sources) = &view.authored_input_sources
        && !authored_input_sources.is_empty()
    {
        lines.push(format!("authored_input_sources: {}", authored_input_sources.join(", ")));
    }

    if let Some(authored_input_deduplicated_sources) = &view.authored_input_deduplicated_sources
        && !authored_input_deduplicated_sources.is_empty()
    {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            authored_input_deduplicated_sources.join(", ")
        ));
    }

    if let Some(clarification_headline) = &view.clarification_headline {
        lines.push(format!("clarification_headline: {clarification_headline}"));
    }

    if let Some(clarification_prompt) = &view.clarification_prompt {
        lines.push(format!("clarification_prompt: {clarification_prompt}"));
    }

    if let Some(clarification_missing_fields) = &view.clarification_missing_fields
        && !clarification_missing_fields.is_empty()
    {
        lines.push(format!(
            "clarification_missing_fields: {}",
            clarification_missing_fields.join(", ")
        ));
    }

    if let Some(requested_governance_runtime) = &view.requested_governance_runtime {
        lines.push(format!("requested_governance_runtime: {requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &view.requested_governance_risk {
        lines.push(format!("requested_governance_risk: {requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &view.requested_governance_zone {
        lines.push(format!("requested_governance_zone: {requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &view.requested_governance_owner {
        lines.push(format!("requested_governance_owner: {requested_governance_owner}"));
    }

    if let Some(active_flow) = &view.active_flow {
        lines.push(format!("active_flow: {active_flow}"));
    }

    if let Some(flow_state) = &view.flow_state {
        lines.push(format!("flow_state: {flow_state}"));
    }

    if let Some(active_workflow) = &view.active_workflow {
        lines.push(format!("workflow: {active_workflow}"));
    }

    if let Some(workflow_phase) = &view.workflow_phase {
        lines.push(format!("workflow_phase: {workflow_phase}"));
    }

    if let Some(current_stage_id) = &view.current_stage_id {
        lines.push(format!("current_stage: {current_stage_id}"));
    }

    if let (Some(current_stage_index), Some(total_stages)) =
        (view.current_stage_index, view.total_stages)
    {
        lines.push(format!("stage_progress: {}/{}", current_stage_index + 1, total_stages));
    }

    if let Some(plan_revision) = view.plan_revision {
        lines.push(format!("plan_revision: {plan_revision}"));
    }

    if let Some(current_step_index) = view.current_step_index {
        lines.push(format!("current_step_index: {current_step_index}"));
    }

    if let Some(current_step_id) = &view.current_step_id {
        lines.push(format!("current_step_id: {current_step_id}"));
    }

    lines.push(format!("latest_status: {}", session_status_text(view.latest_status)));

    if let Some(execution_path) = &view.execution_path {
        lines.push(format!("execution_path: {execution_path}"));
    }

    if let Some(latest_trace_ref) = &view.latest_trace_ref {
        lines.push(format!("latest_trace_ref: {latest_trace_ref}"));
    }

    if let Some(latest_decision_status) = &view.latest_decision_status {
        lines.push(format!("latest_decision_status: {latest_decision_status}"));
    }

    if let Some(latest_decision_target) = &view.latest_decision_target {
        lines.push(format!("latest_decision_target: {latest_decision_target}"));
    }

    if let Some(latest_changed_files) = &view.latest_changed_files
        && !latest_changed_files.is_empty()
    {
        lines.push(format!("latest_changed_files: {}", latest_changed_files.join(", ")));
    }

    if let Some(latest_workspace_slice) = &view.latest_workspace_slice {
        lines.push(format!("latest_workspace_slice: {latest_workspace_slice}"));
    }

    if let Some(latest_selection_headline) = &view.latest_selection_headline {
        lines.push(format!("latest_selection_headline: {latest_selection_headline}"));
    }

    if let Some(latest_candidate_family) = &view.latest_candidate_family {
        lines.push(format!("latest_candidate_family: {latest_candidate_family}"));
    }

    if let Some(latest_selection_reason) = &view.latest_selection_reason {
        lines.push(format!("latest_selection_reason: {latest_selection_reason}"));
    }

    if let Some(latest_rejected_candidates) = &view.latest_rejected_candidates
        && !latest_rejected_candidates.is_empty()
    {
        lines.push(format!(
            "latest_rejected_candidates: {}",
            latest_rejected_candidates.join(" | ")
        ));
    }

    if let Some(latest_attempt_lineage) = &view.latest_attempt_lineage {
        lines.push(format!("latest_attempt_lineage: {latest_attempt_lineage}"));
    }

    if let Some(latest_validation_status) = &view.latest_validation_status {
        lines.push(format!("latest_validation_status: {latest_validation_status}"));
    }

    if let Some(latest_exhaustion_reason) = &view.latest_exhaustion_reason {
        lines.push(format!("latest_exhaustion_reason: {latest_exhaustion_reason}"));
    }

    if let Some(latest_review_trigger) = &view.latest_review_trigger {
        lines.push(format!("latest_review_trigger: {latest_review_trigger}"));
    }

    if let Some(latest_review_vote) = &view.latest_review_vote {
        lines.push(format!("latest_review_vote: {latest_review_vote}"));
    }

    if let Some(latest_review_outcome) = &view.latest_review_outcome {
        lines.push(format!("latest_review_outcome: {latest_review_outcome}"));
    }

    if let Some(latest_review_headline) = &view.latest_review_headline {
        lines.push(format!("latest_review_headline: {latest_review_headline}"));
    }

    if let Some(latest_governance_stage) = &view.latest_governance_stage {
        lines.push(format!("latest_governance_stage: {latest_governance_stage}"));
    }

    if let Some(latest_governance_runtime) = &view.latest_governance_runtime {
        lines.push(format!("latest_governance_runtime: {latest_governance_runtime}"));
    }

    if let Some(latest_governance_mode) = &view.latest_governance_mode {
        lines.push(format!("latest_governance_mode: {latest_governance_mode}"));
    }

    if let Some(latest_governance_run_ref) = &view.latest_governance_run_ref {
        lines.push(format!("latest_governance_run_ref: {latest_governance_run_ref}"));
    }

    if let Some(latest_governance_state) = &view.latest_governance_state {
        lines.push(format!("latest_governance_state: {latest_governance_state}"));
    }

    if let Some(latest_governance_blocked_reason) = &view.latest_governance_blocked_reason {
        lines.push(format!("latest_governance_blocked_reason: {latest_governance_blocked_reason}"));
    }

    if let Some(latest_governance_packet_ref) = &view.latest_governance_packet_ref {
        lines.push(format!("latest_governance_packet_ref: {latest_governance_packet_ref}"));
    }

    if let Some(latest_governance_packet_source_stage) = &view.latest_governance_packet_source_stage
    {
        lines.push(format!(
            "latest_governance_packet_source_stage: {latest_governance_packet_source_stage}"
        ));
    }

    if let Some(latest_governance_packet_binding_reason) =
        &view.latest_governance_packet_binding_reason
    {
        lines.push(format!(
            "latest_governance_packet_binding_reason: {latest_governance_packet_binding_reason}"
        ));
    }

    if let Some(latest_governance_approval) = &view.latest_governance_approval {
        lines.push(format!("latest_governance_approval: {latest_governance_approval}"));
    }

    if let Some(latest_governance_decision) = &view.latest_governance_decision {
        lines.push(format!("latest_governance_decision: {latest_governance_decision}"));
    }

    if let Some(latest_governance_candidates) = &view.latest_governance_candidates
        && !latest_governance_candidates.is_empty()
    {
        lines.push(format!(
            "latest_governance_candidates: {}",
            latest_governance_candidates.join(", ")
        ));
    }

    if let Some(governance_next_action) = &view.governance_next_action {
        lines.push(format!("governance_next_action: {governance_next_action}"));
    }

    if let Some(next_command) = view.next_command.as_ref().or(view.workflow_next_action.as_ref()) {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.push(format!("explanation: {}", view.explanation));
    lines.join("\n")
}

pub fn render_compatibility_follow_up_status(
    workspace_ref: &str,
    continuity_authority: ContinuityAuthority,
    follow_up: &CompatibilityFollowUpView,
    explanation: impl Into<String>,
) -> String {
    let mut lines = vec![format!("workspace_ref: {workspace_ref}")];
    lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    lines.extend(render_compatibility_follow_up_lines(
        follow_up,
        "routing",
        "execution_condition",
        "next_command",
    ));
    lines.push(format!("explanation: {}", explanation.into()));
    lines.join("\n")
}

pub fn render_session_error(action: &str, message: &str, next_command: Option<&str>) -> String {
    let mut lines = vec![format!("{action}: session error"), format!("reason: {message}")];

    if let Some(next_command) = next_command {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.join("\n")
}

pub const fn next_command_after_run(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Succeeded => "/synod-status",
        TaskStatus::Planned
        | TaskStatus::Running
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "/synod-next",
    }
}

fn adaptive_workspace_slice_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    let slice = state.get("latest_workspace_slice")?;
    let targets = slice.get("selected_targets")?.as_array()?;
    let targets = targets.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if targets.is_empty() { None } else { Some(targets.join(", ")) }
}

fn adaptive_attempt_lineage_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    let lineage = state.get("latest_attempt_lineage")?;
    let current = lineage.get("current_attempt_id")?.as_str()?;
    let transition = lineage.get("transition_kind")?.as_str()?;
    let previous = lineage.get("previous_attempt_id").and_then(Value::as_str);
    previous.map_or_else(
        || Some(format!("{current} ({transition})")),
        |previous| Some(format!("{current} {transition} {previous}")),
    )
}

fn adaptive_candidate_family_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    state.get("latest_candidate_family")?.as_str().map(str::to_string)
}

fn adaptive_selection_reason_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    state.get("latest_selection_reason")?.as_str().map(str::to_string)
}

fn adaptive_rejected_candidates_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    let rejected = state.get("latest_rejected_candidates")?.as_array()?;
    let rejected = rejected.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if rejected.is_empty() { None } else { Some(rejected.join(" | ")) }
}

fn adaptive_exhaustion_reason_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    state.get("latest_exhaustion_reason")?.as_str().map(str::to_string)
}

pub const fn next_command_after_inspect(_: TaskStatus) -> &'static str {
    "/synod-next"
}

pub fn trace_execution_condition_text(summary: &TraceSummaryView) -> String {
    let (kind, reason) = trace_execution_condition_parts(summary);
    format!("{kind} - {reason}")
}

fn value_as_string_list(value: &Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
    })
}

fn validation_line_from_event(payload: &Value) -> Option<String> {
    let validation =
        payload.get("output").and_then(|output| output.get("validation")).or_else(|| {
            payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
        })?;
    let command = validation.get("command").and_then(Value::as_str).unwrap_or("validation");
    let succeeded = validation.get("succeeded").and_then(Value::as_bool).unwrap_or(false);
    let exit_code = validation.get("exit_code").and_then(Value::as_i64).unwrap_or(-1);
    Some(format!(
        "validation: {} ({command}, exit_code={exit_code})",
        if succeeded { "passed" } else { "failed" }
    ))
}

fn review_event_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::ReviewStarted => payload
            .get("review_trigger")
            .and_then(Value::as_str)
            .map(|trigger| format!("review_trigger: {trigger}")),
        TraceEventType::ReviewTriggerIgnored => payload
            .get("review_trigger")
            .and_then(Value::as_str)
            .map(|trigger| format!("review_trigger_ignored: {trigger}")),
        TraceEventType::ReviewerCompleted => reviewer_event_line(payload),
        TraceEventType::ReviewVoteResolved => payload
            .get("summary")
            .and_then(Value::as_str)
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
            reviewer_event_line(payload).map(|line| format!("review_adjudication: {line}"))
        }
        TraceEventType::ReviewTerminalRecorded => payload
            .get("review_outcome")
            .and_then(Value::as_str)
            .map(|outcome| format!("review_outcome: {outcome}"))
            .or_else(|| {
                payload
                    .get("failure_reason")
                    .and_then(Value::as_str)
                    .map(|reason| format!("review_reason: {reason}"))
            }),
        _ => None,
    }
}

fn governance_event_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::GovernanceSelected => Some(format!(
            "governance_selected: {} -> {}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload.get("selected_runtime").and_then(Value::as_str).unwrap_or("unknown-runtime")
        )),
        TraceEventType::GovernanceStarted => Some(format!(
            "governance_started: {}{}{}{}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload
                .get("canon_mode")
                .and_then(Value::as_str)
                .map(|mode| format!(" ({mode})"))
                .unwrap_or_default(),
            payload
                .get("run_ref")
                .and_then(Value::as_str)
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceDecisionRecorded => payload
            .get("selected_action")
            .and_then(Value::as_str)
            .map(|action| format!("governance_decision: {action}"))
            .or_else(|| {
                payload
                    .get("blocked_reason")
                    .and_then(Value::as_str)
                    .map(|reason| format!("governance_decision_blocked: {reason}"))
            }),
        TraceEventType::GovernanceAwaitingApproval => Some(format!(
            "governance_awaiting_approval: {} ({}){}{}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload.get("approval_state").and_then(Value::as_str).unwrap_or("unknown"),
            payload
                .get("run_ref")
                .and_then(Value::as_str)
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceCompleted => Some(format!(
            "governance_completed: {}{}{}",
            payload.get("headline").and_then(Value::as_str).unwrap_or("governed packet ready"),
            payload
                .get("packet_ref")
                .and_then(Value::as_str)
                .map(|packet_ref| format!(" [{packet_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceBlocked => Some(format!(
            "governance_blocked: {}{}",
            payload.get("reason").and_then(Value::as_str).unwrap_or("blocked"),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernancePacketRejected => Some(format!(
            "governance_packet_rejected: {}{}",
            payload.get("reason").and_then(Value::as_str).unwrap_or("packet rejected"),
            governance_packet_provenance_suffix(payload)
        )),
        _ => None,
    }
}

pub(crate) fn governance_packet_provenance_suffix(payload: &Value) -> String {
    governance_packet_provenance_text(
        payload.get("packet_source_stage").and_then(Value::as_str),
        payload.get("packet_binding_reason").and_then(Value::as_str),
    )
    .map(|provenance| format!(" from {provenance}"))
    .unwrap_or_default()
}

fn reviewer_event_line(payload: &Value) -> Option<String> {
    let reviewer_id = payload.get("reviewer_id").and_then(Value::as_str)?;

    if let Some(finding) = payload.get("finding") {
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        let role = payload.get("reviewer_role").and_then(Value::as_str);
        return Some(match role {
            Some(role) => format!("reviewer {reviewer_id} ({role}) {disposition}: {summary}"),
            None => format!("reviewer {reviewer_id} {disposition}: {summary}"),
        });
    }

    payload
        .get("failure_reason")
        .and_then(Value::as_str)
        .map(|reason| format!("reviewer {reviewer_id} failed: {reason}"))
}

fn task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Exhausted => "exhausted",
        TaskStatus::Aborted => "aborted",
    }
}

fn render_compatibility_follow_up_lines(
    follow_up: &CompatibilityFollowUpView,
    routing_label: &str,
    execution_condition_label: &str,
    next_command_label: &str,
) -> Vec<String> {
    let routing_summary =
        follow_up.routing_summary.strip_prefix("routing: ").unwrap_or(&follow_up.routing_summary);

    vec![
        format!("compatibility_follow_up: {}", follow_up.follow_up_mode.as_str()),
        format!("compatibility_trace_ref: {}", follow_up.trace_ref),
        format!("{routing_label}: {routing_summary}"),
        format!("{execution_condition_label}: {}", follow_up.execution_condition),
        format!("compatibility_terminal_status: {}", task_status_text(follow_up.terminal_status)),
        format!("compatibility_terminal_reason: {}", follow_up.terminal_reason),
        format!("{next_command_label}: {}", follow_up.next_command),
    ]
}

fn step_kind_text(kind: StepKind) -> &'static str {
    match kind {
        StepKind::Agent => "agent",
        StepKind::Tool => "tool",
        StepKind::Decision => "decision",
    }
}

fn step_status_text(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
    }
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn routing_outcome_for_status_view(view: &SessionStatusView) -> RoutingOutcome {
    match view.execution_path.as_deref() {
        Some("native_goal_plan") => RoutingOutcome {
            mode: RoutingMode::Native,
            source: RoutingSource::GoalPlan,
            reason: "goal plan is ready for native execution".to_string(),
        },
        Some("fixture_compatibility") => RoutingOutcome {
            mode: RoutingMode::Compatibility,
            source: RoutingSource::ExecutionProfile,
            reason: "compatibility execution remains active from the persisted task".to_string(),
        },
        Some("native_goal_plan_pending_flow_confirmation") => RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalPlan,
            reason: "flow confirmation is still pending before native execution".to_string(),
        },
        Some("native_session_pending_plan") => RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalCapture,
            reason: "goal captured but a goal plan is not ready yet".to_string(),
        },
        _ => match view.latest_status {
            SessionStatus::Initialized => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "capture a goal before planning or execution can begin".to_string(),
            },
            SessionStatus::GoalCaptured => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::GoalCapture,
                reason: "goal captured but a goal plan is not ready yet".to_string(),
            },
            SessionStatus::Invalid => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "active session state is invalid and must be recreated".to_string(),
            },
            _ => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "session has no goal plan or compatibility task to route".to_string(),
            },
        },
    }
}

fn render_session_execution_condition(view: &SessionStatusView) -> String {
    let (kind, reason) = session_execution_condition_parts(view);
    format!("execution_condition: {kind} - {reason}")
}

fn session_execution_condition_parts(view: &SessionStatusView) -> (&'static str, String) {
    if let Some(governance_state) = view.latest_governance_state.as_deref() {
        match governance_state {
            "awaiting_approval" => {
                return (
                    "waiting",
                    "governance approval is still pending before execution can continue"
                        .to_string(),
                );
            }
            "blocked" => {
                return (
                    "blocked",
                    view.latest_governance_blocked_reason.clone().unwrap_or_else(|| {
                        "governance blocked further execution until the blocker is resolved"
                            .to_string()
                    }),
                );
            }
            _ => {}
        }
    }

    if let Some(workflow_phase) = view.workflow_phase.as_deref() {
        match workflow_phase {
            "capture" if view.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() => {
                return (
                    "waiting",
                    "workflow is waiting for a captured goal before it can continue".to_string(),
                );
            }
            "clarify"
                if view.clarification_headline.is_some()
                    || view.clarification_prompt.is_some()
                    || view
                        .clarification_missing_fields
                        .as_ref()
                        .is_some_and(|fields| !fields.is_empty()) =>
            {
                return (
                    "waiting",
                    "clarification is still required before workflow planning can continue"
                        .to_string(),
                );
            }
            "review" => {
                if matches!(
                    view.latest_status,
                    SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted
                ) {
                    return ("terminal", "work stopped after a non-success result".to_string());
                }

                if view.latest_review_trigger.is_some() && view.latest_review_outcome.is_none() {
                    return (
                        "waiting",
                        "review outcome is still pending before workflow can continue".to_string(),
                    );
                }

                return (
                    "blocked",
                    "workflow review phase requires review evidence from the active session"
                        .to_string(),
                );
            }
            "govern" if view.latest_governance_state.is_none() => {
                if matches!(
                    view.latest_status,
                    SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted
                ) {
                    return ("terminal", "work stopped after a non-success result".to_string());
                }

                return (
                    "blocked",
                    "workflow govern phase requires governance evidence from the active session"
                        .to_string(),
                );
            }
            "govern"
                if matches!(
                    view.latest_governance_state.as_deref(),
                    Some("governed_ready" | "completed")
                ) && !view.latest_status.is_terminal() =>
            {
                return ("waiting", "governance is ready and workflow can resume".to_string());
            }
            _ => {}
        }
    }

    match view.execution_path.as_deref() {
        Some("native_goal_plan_pending_flow_confirmation") => {
            return (
                "blocked",
                "flow confirmation is still pending before native execution".to_string(),
            );
        }
        Some("native_session_pending_plan") => {
            return ("blocked", "goal captured but a goal plan is not ready yet".to_string());
        }
        _ => {}
    }

    match view.latest_status {
        SessionStatus::Initialized => {
            ("blocked", "capture a goal before planning or execution can begin".to_string())
        }
        SessionStatus::GoalCaptured => {
            ("blocked", "goal captured but a goal plan is not ready yet".to_string())
        }
        SessionStatus::Planned => (
            "waiting",
            if view.current_step_id.is_some() {
                "a bounded task is ready for the next execution step".to_string()
            } else {
                "planning is complete and execution can begin".to_string()
            },
        ),
        SessionStatus::Running => ("running", running_condition_reason(view).to_string()),
        SessionStatus::Succeeded => ("terminal", "work completed successfully".to_string()),
        SessionStatus::Failed => {
            if let Some(reason) = view.latest_exhaustion_reason.clone() {
                return ("terminal", reason);
            }
            ("terminal", "work stopped after a non-success result".to_string())
        }
        SessionStatus::Exhausted => {
            if let Some(reason) = view.latest_exhaustion_reason.clone() {
                return ("terminal", reason);
            }
            ("terminal", "retry or recovery limits were exhausted".to_string())
        }
        SessionStatus::Aborted => ("terminal", "work was aborted before completion".to_string()),
        SessionStatus::Invalid => {
            ("blocked", "active session state is invalid and must be recreated".to_string())
        }
    }
}

fn running_condition_reason(view: &SessionStatusView) -> &'static str {
    match view.latest_decision_status.as_deref() {
        Some("pending") => "a bounded decision is pending dispatch",
        Some("dispatched") => "the latest bounded decision is in flight",
        Some("verified") => "the latest bounded decision was verified and more work may remain",
        Some("failed") => "the latest bounded decision failed and recovery is in progress",
        Some("recovered") => "the latest bounded decision recovered and execution can continue",
        _ if view.latest_review_trigger.is_some() => {
            "review is in progress as part of the active session"
        }
        _ => "bounded execution is in progress",
    }
}

fn render_trace_execution_condition(summary: &TraceSummaryView) -> String {
    let (kind, reason) = trace_execution_condition_parts(summary);
    format!("execution_condition: {kind} - {reason}")
}

fn trace_execution_condition_parts(summary: &TraceSummaryView) -> (&'static str, String) {
    let governance_waiting =
        summary.governance_timeline.iter().any(|line| line.contains("awaiting_approval"));
    let governance_blocked = summary.governance_timeline.iter().any(|line| {
        line.contains("governance_blocked") || line.contains("governance_packet_rejected")
    });

    if governance_waiting {
        return (
            "waiting",
            "governance approval is still pending before execution can continue".to_string(),
        );
    }

    if governance_blocked {
        return (
            "blocked",
            summary.governance_next_action.clone().unwrap_or_else(|| {
                "governance blocked further execution until the blocker is resolved".to_string()
            }),
        );
    }

    match summary.terminal_status {
        TaskStatus::Failed | TaskStatus::Exhausted
            if trace_adaptive_exhaustion_reason(summary).is_some() =>
        {
            ("terminal", trace_adaptive_exhaustion_reason(summary).unwrap())
        }
        TaskStatus::Planned => ("waiting", summary.terminal_reason.message.clone()),
        TaskStatus::Running => ("running", summary.terminal_reason.message.clone()),
        TaskStatus::Succeeded
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => ("terminal", summary.terminal_reason.message.clone()),
    }
}

fn trace_adaptive_exhaustion_reason(summary: &TraceSummaryView) -> Option<String> {
    summary
        .adaptive_evidence
        .iter()
        .find_map(|line| line.strip_prefix("adaptive_exhaustion: ").map(str::to_string))
}

fn render_run_execution_condition(response: &TaskRunResponse) -> String {
    let kind = match response.terminal_status {
        TaskStatus::Planned => "waiting",
        TaskStatus::Running => {
            let message = response.terminal_reason.message.to_ascii_lowercase();
            if message.contains("approval")
                || message.contains("wait")
                || message.contains("blocked")
            {
                "waiting"
            } else {
                "running"
            }
        }
        TaskStatus::Succeeded
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "terminal",
    };

    format!("execution_condition: {kind} - {}", response.terminal_reason.message)
}

fn cluster_member_state_text(state: ClusterMemberState) -> &'static str {
    match state {
        ClusterMemberState::Healthy => "healthy",
        ClusterMemberState::MissingSession => "missing-session",
        ClusterMemberState::MissingTrace => "missing-trace",
        ClusterMemberState::Blocked => "blocked",
        ClusterMemberState::Invalid => "invalid",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{command_name, render_run_trace, render_session_status, render_trace_summary};
    use crate::cli::DeveloperCommand;
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::session::{SessionStatus, SessionStatusView};
    use crate::domain::step::{StepKind, StepStatus};
    use crate::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{
        ExecutionTrace, TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
    };

    #[test]
    fn command_name_covers_every_developer_subcommand() {
        let commands = [
            (DeveloperCommand::Doctor { workspace: "/tmp/workspace".into() }, "doctor"),
            (DeveloperCommand::Start { workspace: None }, "start"),
            (
                DeveloperCommand::Capture {
                    workspace: None,
                    goal: Some("goal".to_string()),
                    brief: Vec::new(),
                    governance: None,
                    risk: None,
                    zone: None,
                    owner: None,
                },
                "capture",
            ),
            (DeveloperCommand::Flow { name: "bug-fix".to_string(), workspace: None }, "flow"),
            (DeveloperCommand::Plan { workspace: None, flow: None, no_flow: false }, "plan"),
            (DeveloperCommand::Step { workspace: None }, "step"),
            (
                DeveloperCommand::Run {
                    workspace: None,
                    goal: None,
                    brief: Vec::new(),
                    governance: None,
                    risk: None,
                    zone: None,
                    owner: None,
                },
                "run",
            ),
            (
                DeveloperCommand::Workflow {
                    command: crate::cli::WorkflowSubcommand::Run {
                        name: "default".to_string(),
                        workspace: None,
                        goal: None,
                    },
                },
                "workflow",
            ),
            (DeveloperCommand::Inspect { trace: None, workspace: None }, "inspect"),
            (DeveloperCommand::Status { workspace: None }, "status"),
            (DeveloperCommand::Next { workspace: None }, "next"),
        ];

        for (command, expected) in commands {
            assert_eq!(command_name(&command), expected);
        }
    }

    #[test]
    fn render_run_trace_covers_stage_replan_and_stage_failure_fallbacks() {
        let mut trace = ExecutionTrace::new("task-output", "session-output", "Render output");
        trace.record_event(TraceEventType::StageReplanned, None, 0, json!({}));
        trace.record_event(TraceEventType::StageFailed, None, 0, json!({}));

        let response = TaskRunResponse {
            task_id: "task-output".to_string(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "stage failed",
                None,
            ),
            final_context: TaskContext::new(
                "session-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.synod/traces/task-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/synod-next");

        assert!(text.contains("stage replan after unknown-step: replan scheduled"), "{text}");
        assert!(text.contains("stage unknown-stage failed: stage failed"), "{text}");
    }

    #[test]
    fn render_trace_summary_labels_flow_stage_and_stage_failure_events() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.synod/traces/task-output.json".to_string(),
            goal: "Render trace summary".to_string(),
            routing_summary: None,
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
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
            executed_steps: vec![TraceStepSummary {
                step_id: "verify".to_string(),
                step_kind: StepKind::Tool,
                attempts: 1,
                final_status: StepStatus::Succeeded,
                headline: "validation passed".to_string(),
            }],
            recovery_events: vec![
                TraceRecoveryEvent {
                    event_type: TraceEventType::FlowSelected,
                    trigger: "bug-fix @ investigate".to_string(),
                    related_step_id: None,
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::StageTransitioned,
                    trigger: "investigate -> implement".to_string(),
                    related_step_id: None,
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::StageFailed,
                    trigger: "verify failed".to_string(),
                    related_step_id: Some("verify".to_string()),
                },
            ],
            governance_timeline: Vec::new(),
            governance_next_action: None,
            review_timeline: Vec::new(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed",
                None,
            ),
            duration: None,
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/synod-next");

        assert!(text.contains("flow: bug-fix @ investigate"), "{text}");
        assert!(text.contains("stage: investigate -> implement"), "{text}");
        assert!(text.contains("stage_failure: verify failed"), "{text}");
    }

    #[test]
    fn render_session_status_covers_invalid_status_without_changed_files() {
        let view = SessionStatusView {
            session_id: "session-output".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: None,
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
            active_flow: None,
            flow_state: None,
            active_workflow: None,
            workflow_phase: None,
            workflow_next_action: None,
            continuity_authority: None,
            compatibility_follow_up: None,
            current_stage_id: None,
            current_stage_index: None,
            total_stages: None,
            plan_revision: None,
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Invalid,
            execution_path: None,
            latest_trace_ref: None,
            latest_decision_status: None,
            latest_decision_target: None,
            latest_changed_files: Some(Vec::new()),
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
            next_command: None,
            explanation: "session is invalid".to_string(),
        };

        let text = render_session_status(&view);

        assert!(text.contains("latest_status: invalid"), "{text}");
        assert!(!text.contains("latest_changed_files:"), "{text}");
    }

    #[test]
    fn render_run_trace_surfaces_review_events() {
        let mut trace =
            ExecutionTrace::new("task-review-output", "session-review-output", "Render review");
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

        let response = TaskRunResponse {
            task_id: "task-review-output".to_string(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            final_context: TaskContext::new(
                "session-review-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.synod/traces/task-review-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/synod-next");

        assert!(text.contains("review_trigger: pr_ready"), "{text}");
        assert!(text.contains("reviewer safety (Safety) approve: No blockers"), "{text}");
        assert!(
            text.contains(
                "review_vote: strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"
            ),
            "{text}"
        );
        assert!(text.contains("review_outcome: accepted"), "{text}");
    }

    #[test]
    fn render_session_status_surfaces_review_projection() {
        let view = SessionStatusView {
            session_id: "session-review-status".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: Some("Ship review output".to_string()),
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
            active_flow: None,
            flow_state: None,
            active_workflow: None,
            workflow_phase: None,
            workflow_next_action: None,
            continuity_authority: None,
            compatibility_follow_up: None,
            current_stage_id: None,
            current_stage_index: None,
            total_stages: None,
            plan_revision: None,
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Running,
            execution_path: Some("fixture_compatibility".to_string()),
            latest_trace_ref: None,
            latest_decision_status: None,
            latest_decision_target: None,
            latest_changed_files: None,
            latest_workspace_slice: None,
            latest_selection_headline: None,
            latest_candidate_family: None,
            latest_selection_reason: None,
            latest_rejected_candidates: None,
            latest_attempt_lineage: None,
            latest_validation_status: Some("passed".to_string()),
            latest_exhaustion_reason: None,
            latest_review_trigger: Some("pr_ready".to_string()),
            latest_review_vote: Some(
                "strategy=majority approvals=2 concerns=0 blocks=0 decision=accepted".to_string(),
            ),
            latest_review_outcome: Some("accepted".to_string()),
            latest_review_headline: Some("safety approve: No blockers".to_string()),
            latest_governance_stage: Some("bug-fix:implement".to_string()),
            latest_governance_runtime: Some("canon".to_string()),
            latest_governance_mode: Some("implementation".to_string()),
            latest_governance_run_ref: Some("canon-run-1".to_string()),
            latest_governance_state: Some("awaiting_approval".to_string()),
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: Some(".canon/runs/canon-run-1".to_string()),
            latest_governance_packet_source_stage: Some("bug-fix:investigate".to_string()),
            latest_governance_packet_binding_reason: Some("upstream_stage_context".to_string()),
            latest_governance_approval: Some("requested".to_string()),
            latest_governance_decision: Some(
                "await approval for governed implementation".to_string(),
            ),
            latest_governance_candidates: Some(vec![
                "await_approval".to_string(),
                "block_stage".to_string(),
            ]),
            governance_next_action: Some("wait for approval and rerun synod status".to_string()),
            next_command: Some("synod step".to_string()),
            explanation: "review is in progress".to_string(),
        };

        let text = render_session_status(&view);

        assert!(
            text.contains(
                "routing: compatibility (execution_profile) - compatibility execution remains active from the persisted task"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "execution_condition: waiting - governance approval is still pending before execution can continue"
            ),
            "{text}"
        );
        assert!(text.contains("latest_review_trigger: pr_ready"), "{text}");
        assert!(text.contains("latest_review_vote: strategy=majority approvals=2 concerns=0 blocks=0 decision=accepted"), "{text}");
        assert!(text.contains("latest_review_outcome: accepted"), "{text}");
        assert!(text.contains("latest_review_headline: safety approve: No blockers"), "{text}");
        assert!(text.contains("latest_governance_mode: implementation"), "{text}");
        assert!(text.contains("latest_governance_run_ref: canon-run-1"), "{text}");
        assert!(text.contains("latest_governance_state: awaiting_approval"), "{text}");
        assert!(text.contains("execution_path: fixture_compatibility"), "{text}");
        assert!(
            text.contains("latest_governance_candidates: await_approval, block_stage"),
            "{text}"
        );
        assert!(
            text.contains("governance_next_action: wait for approval and rerun synod status"),
            "{text}"
        );
    }

    #[test]
    fn render_trace_summary_includes_review_timeline() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.synod/traces/task-review-output.json".to_string(),
            goal: "Render trace summary".to_string(),
            routing_summary: None,
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
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
            executed_steps: vec![],
            recovery_events: vec![],
            governance_timeline: vec![
                "governance_selected: bug-fix:implement -> canon".to_string(),
                "governance_awaiting_approval: bug-fix:implement (requested)".to_string(),
            ],
            governance_next_action: Some("wait for approval and rerun synod status".to_string()),
            review_timeline: vec![
                "review_trigger: pr_ready".to_string(),
                "reviewer safety (Safety) approve: No blockers".to_string(),
                "review_vote: strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"
                    .to_string(),
                "review_outcome: accepted".to_string(),
            ],
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            duration: Some(42),
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/synod-next");

        assert!(text.contains("governance_selected: bug-fix:implement -> canon"), "{text}");
        assert!(
            text.contains("governance_next_action: wait for approval and rerun synod status"),
            "{text}"
        );
        assert!(text.contains("review_trigger: pr_ready"), "{text}");
        assert!(text.contains("review_outcome: accepted"), "{text}");
    }
}
