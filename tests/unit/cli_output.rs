use std::path::PathBuf;

use serde_json::Map;
use serde_json::json;
use synod::FileConfigStore;
use synod::FileTraceStore;
use synod::adapters::trace_store::TraceStore;
use synod::cli::diagnostics::{DiagnosticsCheck, DiagnosticsReport, DiagnosticsStatus};
use synod::cli::inspect::{
    InspectCommandError, TraceResolutionTarget, TraceSummaryError, execute_inspect, render_error,
    render_inspection_routing_summary, resolve_trace_path, summarize_trace,
};
use synod::cli::output::{
    CommandExitCode, command_name, next_command_after_inspect, next_command_after_run,
    render_diagnostics, render_goal_plan_flow_state, render_inspect_failure, render_route_outcome,
    render_run_trace, render_session_status, render_trace_summary, validation_error_message,
};
use synod::cli::{
    CliValidationError, CommandExitStatus, CommandName, DeveloperCommand, DeveloperCommandSession,
};
use synod::domain::configuration::{ConfigFile, ModelRoute, RoutingConfig, RuntimeKind};
use synod::domain::goal_plan::{GoalPlanFlowMode, GoalPlanFlowState};
use synod::domain::governance::GovernanceRuntimeKind;
use synod::domain::limits::{RunLimits, TerminalCondition};
use synod::domain::session::{
    RoutingMode, RoutingOutcome, RoutingSource, SessionStatus, SessionStatusView,
};
use synod::domain::step::{StepKind, StepStatus};
use synod::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
use synod::domain::task_context::TaskContext;
use synod::domain::trace::{
    ExecutionTrace, TraceEvent, TraceEventType, TraceRecoveryEvent, TraceStepSummary,
    TraceSummaryView,
};
use uuid::Uuid;

#[test]
fn exit_codes_match_the_command_contract() {
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::Succeeded).code(), 0);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::NonSuccess).code(), 1);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::InvalidInvocation).code(), 2);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::TraceReadFailure).code(), 3);
}

#[test]
fn command_names_render_from_subcommands() {
    let command = DeveloperCommand::Flow {
        name: "bug-fix".to_string(),
        workspace: Some(PathBuf::from("/tmp/workspace")),
    };
    assert_eq!(command_name(&command), "flow");
    assert_eq!(command.name(), CommandName::Flow);

    let command = DeveloperCommand::Run {
        workspace: Some(PathBuf::from("/tmp/workspace")),
        goal: None,
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
    };
    assert_eq!(command_name(&command), "run");
    assert_eq!(command.name(), CommandName::Run);
}

#[test]
fn run_session_requires_a_non_empty_goal() {
    let command = DeveloperCommand::Run {
        workspace: Some(PathBuf::from("/tmp/workspace")),
        goal: Some("   ".to_string()),
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
    };
    let session = DeveloperCommandSession::from_command(&command);

    assert_eq!(session.validate(), Err(CliValidationError::MissingGoal(CommandName::Run)));
}

#[test]
fn run_without_legacy_flags_is_valid_for_session_native_execution() {
    let command = DeveloperCommand::Run {
        workspace: None,
        goal: None,
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
    };
    let session = DeveloperCommandSession::from_command(&command);

    assert_eq!(session.validate(), Ok(()));
}

#[test]
fn inspect_session_requires_trace_or_workspace() {
    let session = DeveloperCommandSession {
        command_name: CommandName::Inspect,
        workspace_ref: None,
        goal: None,
        trace_ref: None,
        started_at: 0,
        completed_at: None,
        exit_status: None,
        trace_location: None,
    };

    assert_eq!(session.validate(), Err(CliValidationError::MissingTraceSelection));
    assert_eq!(
        validation_error_message(&CliValidationError::MissingTraceSelection),
        "inspect requires --trace or --workspace"
    );
}

#[test]
fn diagnostics_renderer_lists_check_names_and_actions() {
    let report = DiagnosticsReport {
        workspace_ref: "/tmp/workspace".to_string(),
        checks: vec![
            DiagnosticsCheck {
                name: "workspace_exists".to_string(),
                status: DiagnosticsStatus::Passed,
                message: "workspace exists".to_string(),
            },
            DiagnosticsCheck {
                name: "trace_store".to_string(),
                status: DiagnosticsStatus::Failed,
                message: "fix the trace directory".to_string(),
            },
        ],
        ready: false,
        missing_prerequisites: vec!["trace_store".to_string()],
        suggested_actions: vec!["fix the trace directory".to_string()],
    };

    let rendered = render_diagnostics(&report);

    assert!(rendered.contains("doctor: not ready"));
    assert!(rendered.contains("workspace_exists"));
    assert!(rendered.contains("trace_store"));
    assert!(rendered.contains("actions:"));
    assert!(rendered.contains("fix the trace directory"));
}

#[test]
fn next_command_helpers_match_assistant_routing_expectations() {
    assert_eq!(next_command_after_run(TaskStatus::Succeeded), "/synod-status");
    assert_eq!(next_command_after_run(TaskStatus::Failed), "/synod-next");
    assert_eq!(next_command_after_inspect(TaskStatus::Succeeded), "/synod-next");
}

#[test]
fn inspect_failure_renderer_exposes_correction_cues() {
    let rendered = render_inspect_failure(
        "explicit-trace",
        Some("/tmp/missing-trace.json"),
        None,
        "failed to read the requested trace",
        "cargo run --bin synod -- inspect --trace <trace>",
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("inspection_target: explicit-trace"));
    assert!(rendered.contains("trace: /tmp/missing-trace.json"));
    assert!(rendered.contains("next_command: /synod-inspect"));
    assert!(
        rendered.contains("corrected_command: cargo run --bin synod -- inspect --trace <trace>")
    );
}

#[test]
fn route_and_flow_render_helpers_expose_foundational_runtime_cues() {
    let route = RoutingOutcome {
        mode: RoutingMode::Blocked,
        source: RoutingSource::GoalPlan,
        reason: "flow confirmation is still pending".to_string(),
    };
    let flow_state = GoalPlanFlowState {
        mode: GoalPlanFlowMode::Proposed,
        flow_name: Some("bug-fix".to_string()),
        confidence_reason: Some("goal contains keyword 'fix'".to_string()),
    };

    assert_eq!(
        render_route_outcome(&route),
        "routing: blocked (goal_plan) - flow confirmation is still pending"
    );
    assert_eq!(
        render_goal_plan_flow_state(&flow_state),
        "flow_state: proposed (bug-fix) - goal contains keyword 'fix'"
    );

    let summary = render_inspection_routing_summary(&route, Some(&flow_state));
    assert_eq!(summary[0], "routing: blocked (goal_plan) - flow confirmation is still pending");
    assert_eq!(summary[1], "flow_state: proposed (bug-fix) - goal contains keyword 'fix'");
}

#[test]
fn inspect_invalid_session_errors_reuse_session_guidance() {
    let rendered = render_error(
        None,
        Some(std::path::Path::new("/tmp/workspace")),
        &InspectCommandError::InvalidSession(
            "active session is invalid: workspace_ref must not be empty".to_string(),
        ),
    );

    assert!(rendered.contains("inspect: session error"), "{rendered}");
    assert!(rendered.contains("reason: active session is invalid:"), "{rendered}");
    assert!(rendered.contains("next_command: synod start"), "{rendered}");
}

#[test]
fn trace_summary_renderer_mentions_steps_recovery_and_terminal_reason() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/workspace/.synod/traces/task.json".to_string(),
        goal: "Inspect a recorded run".to_string(),
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
        executed_steps: vec![
            TraceStepSummary {
                step_id: "analyze".to_string(),
                step_kind: StepKind::Agent,
                attempts: 1,
                final_status: StepStatus::Succeeded,
                headline: "succeeded after 1 attempt(s)".to_string(),
            },
            TraceStepSummary {
                step_id: "code".to_string(),
                step_kind: StepKind::Agent,
                attempts: 2,
                final_status: StepStatus::Succeeded,
                headline: "succeeded after 2 attempt(s)".to_string(),
            },
        ],
        recovery_events: vec![TraceRecoveryEvent {
            event_type: TraceEventType::RetryScheduled,
            trigger: "retrying step code within remaining retry budget".to_string(),
            related_step_id: Some("code".to_string()),
        }],
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "goal satisfied after step verify",
            None,
        ),
        duration: Some(42),
    };

    let rendered = render_trace_summary(
        &summary,
        "explicit-trace",
        next_command_after_inspect(summary.terminal_status),
    );

    assert!(rendered.contains("inspection_target: explicit-trace"));
    assert!(rendered.contains("trace: /tmp/workspace/.synod/traces/task.json"));
    assert!(
        rendered.contains("execution_condition: terminal - goal satisfied after step verify"),
        "{rendered}"
    );
    assert!(rendered.contains("step analyze (agent) succeeded [1 attempt(s)]"));
    assert!(rendered.contains("step code (agent) succeeded [2 attempt(s)]"));
    assert!(rendered.contains("retry: retrying step code within remaining retry budget"));
    assert!(rendered.contains("terminal_reason: goal satisfied after step verify"));
    assert!(rendered.contains("next_command: /synod-next"));
    assert!(rendered.contains("duration_ms: 42"));
}

#[test]
fn inspect_failure_renderer_includes_workspace_ref_when_provided() {
    let rendered = render_inspect_failure(
        "latest-workspace-trace",
        None,
        Some("/tmp/my-workspace"),
        "failed to read the requested trace",
        "cargo run --bin synod -- inspect --workspace <workspace>",
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("inspection_target: latest-workspace-trace"));
    assert!(rendered.contains("workspace_ref: /tmp/my-workspace"));
    assert!(rendered.contains("next_command: /synod-inspect"));
    assert!(
        rendered.contains(
            "corrected_command: cargo run --bin synod -- inspect --workspace <workspace>"
        )
    );
}

#[test]
fn render_error_with_missing_trace_reference_uses_explicit_trace_correction() {
    let rendered = render_error(None, None, &InspectCommandError::MissingTraceReference);

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("terminal_reason: inspect requires --trace or --workspace"));
    assert!(rendered.contains("next_command: /synod-inspect"));
    assert!(rendered.contains("corrected_command: cargo run --bin synod -- inspect --trace"));
}

#[test]
fn render_error_with_workspace_path_uses_workspace_correction_cues() {
    let rendered = render_error(
        None,
        Some(std::path::Path::new("/tmp/my-workspace")),
        &InspectCommandError::MissingLatestTrace,
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("inspection_target: latest-workspace-trace"));
    assert!(rendered.contains("terminal_reason: failed to read the requested trace"));
    assert!(rendered.contains("workspace_ref: /tmp/my-workspace"));
    assert!(rendered.contains("next_command: /synod-inspect"));
    assert!(
        rendered.contains(
            "corrected_command: cargo run --bin synod -- inspect --workspace <workspace>"
        )
    );
}

#[test]
fn render_error_with_summary_failure_uses_summary_terminal_reason() {
    let rendered = render_error(
        Some(std::path::Path::new("/tmp/trace.json")),
        None,
        &InspectCommandError::Summary(TraceSummaryError::MissingTerminalStatus),
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("terminal_reason: failed to summarize the requested trace"));
    assert!(rendered.contains("next_command: /synod-inspect"));
}

fn minimal_trace(task_id: &str) -> ExecutionTrace {
    let mut trace = ExecutionTrace::new(task_id, "session-unit", "Unit test goal");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason = Some(TerminalReason::new(
        TerminalCondition::GoalSatisfied,
        "goal satisfied in unit test",
        None,
    ));
    trace
}

fn minimal_response(status: TaskStatus, reason_msg: &str) -> TaskRunResponse {
    TaskRunResponse {
        task_id: "task-unit".to_string(),
        terminal_status: status,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, reason_msg, None),
        final_context: TaskContext::new(
            "session-unit",
            "/tmp/workspace",
            RunLimits::default(),
            Map::new(),
        ),
        plan_revision: 1,
        trace_location: "/tmp/.synod/traces/task-unit.json".to_string(),
    }
}

#[test]
fn render_run_trace_includes_next_command_and_trace_fields() {
    let response = minimal_response(TaskStatus::Succeeded, "goal satisfied");
    let rendered = render_run_trace("run", None, &response, "/synod-status");

    assert!(rendered.contains("execution_condition: terminal - goal satisfied"), "{rendered}");
    assert!(rendered.contains("next_command: /synod-status"), "{rendered}");
    assert!(rendered.contains("terminal_status: succeeded"), "{rendered}");
    assert!(rendered.contains("trace: /tmp/.synod/traces/task-unit.json"), "{rendered}");
}

#[test]
fn render_run_trace_with_trace_events_includes_retry_and_replan_lines() {
    let mut trace = ExecutionTrace::new("task-events", "session", "Goal with events");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::RetryScheduled,
        step_id: Some("analyze".to_string()),
        plan_revision: 0,
        payload: json!({"reason": "transient error, retrying"}),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::Replanned,
        step_id: Some("analyze".to_string()),
        plan_revision: 1,
        payload: json!({"reason": "goal shifted, replanning"}),
        recorded_at: 1,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/synod-status");

    assert!(rendered.contains("retry for analyze: transient error, retrying"), "{rendered}");
    assert!(rendered.contains("replan after analyze: goal shifted, replanning"), "{rendered}");
    assert!(rendered.contains("next_command: /synod-status"), "{rendered}");
}

#[test]
fn render_run_trace_surfaces_security_assessment_packet_provenance() {
    let mut trace = ExecutionTrace::new("task-governance", "session", "Governed goal");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GovernanceStarted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "bug-fix:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "canon_mode": "security-assessment",
            "run_ref": "canon-run-security",
            "packet_source_stage": "bug-fix:implement",
            "packet_binding_reason": "upstream_stage_context"
        }),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::GovernanceCompleted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "bug-fix:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "headline": "security assessment packet ready",
            "packet_ref": ".canon/runs/canon-run-security",
            "packet_source_stage": "bug-fix:implement",
            "packet_binding_reason": "upstream_stage_context"
        }),
        recorded_at: 1,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/synod-status");

    assert!(
        rendered.contains(
            "governance_started: bug-fix:verify (security-assessment) [canon-run-security] from bug-fix:implement (upstream_stage_context)"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "governance_completed: security assessment packet ready [.canon/runs/canon-run-security] from bug-fix:implement (upstream_stage_context)"
        ),
        "{rendered}"
    );
}

#[test]
fn execute_inspect_explicit_trace_covers_inspection_target_and_next_command() {
    use std::fs;

    let dir = std::env::temp_dir().join(format!("synod-unit-inspect-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();

    let trace = minimal_trace("task-explicit");
    let store = FileTraceStore::new(&dir);
    let trace_path = store.persist(&trace).unwrap();

    let report = execute_inspect(Some(&trace_path), None).unwrap();
    let output = &report.terminal_output;

    assert!(output.contains("inspection_target: explicit-trace"), "{output}");
    assert!(output.contains("next_command: /synod-next"), "{output}");
}

#[test]
fn execute_inspect_workspace_covers_latest_workspace_trace_target() {
    use std::fs;

    let workspace = std::env::temp_dir().join(format!("synod-unit-ws-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let trace = minimal_trace("task-workspace");
    let store = FileTraceStore::for_workspace(&workspace);
    store.persist(&trace).unwrap();

    let report = execute_inspect(None, Some(&workspace)).unwrap();
    let output = &report.terminal_output;

    assert!(output.contains("inspection_target: latest-workspace-trace"), "{output}");
    assert!(output.contains("next_command: /synod-next"), "{output}");
}

#[test]
fn summarize_trace_handles_tool_and_decision_step_kinds() {
    let mut trace = ExecutionTrace::new("task-steps", "session", "Steps test");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "all steps done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("fetch".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "tool"}),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::StepCompleted,
        step_id: Some("fetch".to_string()),
        plan_revision: 0,
        payload: json!({"status": "succeeded"}),
        recorded_at: 1,
    });
    trace.events.push(TraceEvent {
        event_id: "e3".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("decide".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "decision"}),
        recorded_at: 2,
    });
    trace.events.push(TraceEvent {
        event_id: "e4".to_string(),
        event_type: TraceEventType::StepCompleted,
        step_id: Some("decide".to_string()),
        plan_revision: 0,
        payload: json!({"status": "failed"}),
        recorded_at: 3,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let fetch = summary.executed_steps.iter().find(|s| s.step_id == "fetch").unwrap();
    let decide = summary.executed_steps.iter().find(|s| s.step_id == "decide").unwrap();

    assert_eq!(fetch.step_kind, StepKind::Tool);
    assert_eq!(decide.step_kind, StepKind::Decision);
    assert_eq!(decide.final_status, StepStatus::Failed);
}

#[test]
fn summarize_trace_with_unknown_step_status_yields_running_final_status_and_completed_headline() {
    let mut trace = ExecutionTrace::new("task-unk", "session", "Unknown status test");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "agent"}),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::StepCompleted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({"status": "unknown_status"}),
        recorded_at: 1,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let step = &summary.executed_steps[0];

    assert_eq!(step.final_status, StepStatus::Running);
    assert_eq!(step.headline, "completed");
}

#[test]
fn render_session_status_includes_goal_trace_and_next_command() {
    let view = SessionStatusView {
        session_id: "session-status".to_string(),
        workspace_ref: "/tmp/session-workspace".to_string(),
        goal: Some("Ship a bounded change".to_string()),
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
        plan_revision: Some(2),
        current_step_id: Some("verify".to_string()),
        current_step_index: Some(1),
        latest_status: SessionStatus::Running,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: Some("/tmp/session-workspace/.synod/traces/task.json".to_string()),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
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
        next_command: Some("synod next".to_string()),
        explanation: "the active session can keep executing from the current step".to_string(),
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("session_id: session-status"), "{rendered}");
    assert!(rendered.contains("goal: Ship a bounded change"), "{rendered}");
    assert!(rendered.contains("latest_status: running"), "{rendered}");
    assert!(rendered.contains("execution_path: native_goal_plan"), "{rendered}");
    assert!(
        rendered.contains("latest_trace_ref: /tmp/session-workspace/.synod/traces/task.json"),
        "{rendered}"
    );
    assert!(rendered.contains("next_command: synod next"), "{rendered}");
}

#[test]
fn render_session_status_surfaces_security_assessment_projection() {
    let view = SessionStatusView {
        session_id: "session-governed".to_string(),
        workspace_ref: "/tmp/session-workspace".to_string(),
        goal: Some("Verify a governed change".to_string()),
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
        active_flow: Some("bug-fix".to_string()),
        flow_state: Some("confirmed".to_string()),
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: Some("verify".to_string()),
        current_stage_index: Some(3),
        total_stages: Some(4),
        plan_revision: Some(2),
        current_step_id: Some("verify".to_string()),
        current_step_index: Some(2),
        latest_status: SessionStatus::Running,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: Some("/tmp/session-workspace/.synod/traces/task.json".to_string()),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
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
        latest_governance_stage: Some("bug-fix:verify".to_string()),
        latest_governance_runtime: Some("canon".to_string()),
        latest_governance_mode: Some("security-assessment".to_string()),
        latest_governance_run_ref: Some("canon-run-security".to_string()),
        latest_governance_state: Some("governed_ready".to_string()),
        latest_governance_blocked_reason: None,
        latest_governance_packet_ref: Some(".canon/runs/canon-run-security".to_string()),
        latest_governance_packet_source_stage: Some("bug-fix:implement".to_string()),
        latest_governance_packet_binding_reason: Some("upstream_stage_context".to_string()),
        latest_governance_approval: Some("not_needed".to_string()),
        latest_governance_decision: Some(
            "autopilot selected Canon mode SecurityAssessment for bug-fix:verify".to_string(),
        ),
        latest_governance_candidates: Some(vec![
            "select_mode".to_string(),
            "escalate_pr_review".to_string(),
        ]),
        governance_next_action: None,
        next_command: Some("synod inspect".to_string()),
        explanation: "governance completed for the current verification stage".to_string(),
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("latest_governance_mode: security-assessment"), "{rendered}");
    assert!(
        rendered.contains("latest_governance_packet_ref: .canon/runs/canon-run-security"),
        "{rendered}"
    );
    assert!(
        rendered.contains("latest_governance_packet_source_stage: bug-fix:implement"),
        "{rendered}"
    );
    assert!(
        rendered.contains("latest_governance_packet_binding_reason: upstream_stage_context"),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_workflow_phase_and_pause_reason() {
    let view = SessionStatusView {
        session_id: "session-workflow-status".to_string(),
        workspace_ref: "/tmp/session-workflow".to_string(),
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
        active_workflow: Some("default".to_string()),
        workflow_phase: Some("capture".to_string()),
        workflow_next_action: Some(
            "synod capture --workspace /tmp/session-workflow --goal <goal>".to_string(),
        ),
        continuity_authority: None,
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
        explanation: "workflow is paused until a goal is captured".to_string(),
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("workflow: default"), "{rendered}");
    assert!(rendered.contains("workflow_phase: capture"), "{rendered}");
    assert!(
        rendered.contains(
            "execution_condition: waiting - workflow is waiting for a captured goal before it can continue"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "next_command: synod capture --workspace /tmp/session-workflow --goal <goal>"
        ),
        "{rendered}"
    );
}

#[test]
fn resolve_trace_path_prefers_session_trace_ref_when_available() {
    use std::fs;

    let workspace =
        std::env::temp_dir().join(format!("synod-unit-session-trace-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let explicit_session_trace = workspace.join(".synod").join("traces").join("session-trace.json");
    fs::create_dir_all(explicit_session_trace.parent().unwrap()).unwrap();
    let store = FileTraceStore::new(explicit_session_trace.parent().unwrap());
    let trace = minimal_trace("task-session-trace");
    let persisted = store.persist(&trace).unwrap();

    let (target, path) =
        resolve_trace_path(None, Some(&workspace), Some(persisted.to_str().unwrap())).unwrap();

    assert_eq!(target, TraceResolutionTarget::SessionTraceRef);
    assert_eq!(path, persisted);
}

#[test]
fn execute_inspect_with_no_args_returns_missing_trace_reference_error() {
    let result = execute_inspect(None, None);
    assert!(matches!(result, Err(InspectCommandError::MissingTraceReference)), "{result:?}");
}

#[test]
fn execute_inspect_with_empty_workspace_returns_missing_latest_trace_error() {
    use std::fs;
    let workspace = std::env::temp_dir().join(format!("synod-unit-empty-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let result = execute_inspect(None, Some(&workspace));
    assert!(matches!(result, Err(InspectCommandError::MissingLatestTrace)), "{result:?}");
}

#[test]
fn summarize_trace_errors_on_unknown_step_kind() {
    use serde_json::json;
    use synod::domain::trace::TraceEvent;

    let mut trace = ExecutionTrace::new("task-badkind", "session", "Bad kind test");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "invalid_kind"}),
        recorded_at: 0,
    });

    let result = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace);
    assert!(matches!(result, Err(TraceSummaryError::UnknownStepKind(_))), "{result:?}");
}

#[test]
fn summarize_trace_errors_when_step_kind_payload_is_missing() {
    use serde_json::json;
    use synod::domain::trace::TraceEvent;

    let mut trace = ExecutionTrace::new("task-nokind", "session", "Missing kind test");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({}),
        recorded_at: 0,
    });

    let result = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace);
    assert!(matches!(result, Err(TraceSummaryError::MissingStepKind(_))), "{result:?}");
}

#[test]
fn unimplemented_message_formats_the_command_name() {
    use synod::cli::output::unimplemented_message;

    let msg = unimplemented_message(&DeveloperCommand::Doctor { workspace: PathBuf::from("/tmp") });
    assert_eq!(msg, "`doctor` is not implemented yet");
}

#[test]
fn command_names_render_for_all_four_subcommands() {
    assert_eq!(
        command_name(&DeveloperCommand::Doctor { workspace: PathBuf::from("/tmp") }),
        "doctor"
    );
    assert_eq!(
        command_name(&DeveloperCommand::Run {
            workspace: Some(PathBuf::from("/tmp")),
            goal: Some("x".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
        }),
        "run"
    );
    assert_eq!(
        command_name(&DeveloperCommand::Inspect { trace: None, workspace: None }),
        "inspect"
    );
}

#[test]
fn render_trace_summary_handles_all_terminal_status_variants() {
    let statuses = [
        (TaskStatus::Planned, "planned"),
        (TaskStatus::Running, "running"),
        (TaskStatus::Exhausted, "exhausted"),
        (TaskStatus::Aborted, "aborted"),
        (TaskStatus::Failed, "failed"),
    ];

    for (status, expected) in statuses {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/trace.json".to_string(),
            goal: "test".to_string(),
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
            governance_timeline: Vec::new(),
            governance_next_action: None,
            review_timeline: Vec::new(),
            terminal_status: status,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "reason text",
                None,
            ),
            duration: None,
        };
        let rendered = render_trace_summary(&summary, "explicit-trace", "/synod-next");
        assert!(
            rendered.contains(&format!("terminal_status: {expected}")),
            "status {status:?}: {rendered}"
        );
    }
}

#[test]
fn render_trace_summary_surfaces_route_owner_and_config_projection() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/trace.json".to_string(),
        goal: "test".to_string(),
        routing_summary: Some(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
                .to_string(),
        ),
        goal_plan_summary: None,
        authored_input_summary: None,
        authored_input_sources: Vec::new(),
        authored_input_deduplicated_sources: Vec::new(),
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: Vec::new(),
        requested_governance_runtime: Some("canon".to_string()),
        requested_governance_risk: Some("high".to_string()),
        requested_governance_zone: None,
        requested_governance_owner: None,
        decision_timeline: Vec::new(),
        failure_evidence: Vec::new(),
        adaptive_evidence: Vec::new(),
        executed_steps: Vec::new(),
        recovery_events: Vec::new(),
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        duration: None,
    };

    let rendered = render_trace_summary(&summary, "explicit-trace", "/synod-next");

    assert!(rendered.contains("route_owner: compatibility"), "{rendered}");
    assert!(
        rendered.contains(
            "route_config_projection: requested_governance_runtime=canon | requested_governance_risk=high"
        ),
        "{rendered}"
    );
}

#[test]
fn render_session_status_projects_workspace_routing_defaults() {
    let workspace =
        std::env::temp_dir().join(format!("synod-route-config-status-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let config = ConfigFile {
        routing: RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "gpt-5-codex".to_string(),
            }),
            implementation: Some(ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "gpt-5.4".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    let rendered = render_session_status(&SessionStatusView {
        session_id: "session-config-projection".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Project route defaults".to_string()),
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
        latest_status: SessionStatus::Initialized,
        execution_path: None,
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
        next_command: Some("synod capture --goal <goal>".to_string()),
        explanation: "session is waiting for a goal".to_string(),
    });

    assert!(
        rendered.contains(
            "route_config_projection: workspace_routing: planning=codex/gpt-5-codex, implementation=copilot/gpt-5.4"
        ),
        "{rendered}"
    );
}

#[test]
fn render_trace_summary_projects_workspace_routing_defaults() {
    let workspace =
        std::env::temp_dir().join(format!("synod-route-config-trace-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let config = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-1".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    let trace_ref = workspace.join(".synod").join("traces").join("trace.json");
    let summary = TraceSummaryView {
        trace_ref: trace_ref.to_string_lossy().into_owned(),
        goal: "test".to_string(),
        routing_summary: Some(
            "routing: native (goal_plan) - trace came from the session-native runtime".to_string(),
        ),
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
        executed_steps: Vec::new(),
        recovery_events: Vec::new(),
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        duration: None,
    };

    let rendered = render_trace_summary(&summary, "explicit-trace", "/synod-next");

    assert!(
        rendered.contains("route_config_projection: workspace_routing: review=claude/reviewer-1"),
        "{rendered}"
    );
}

#[test]
fn render_trace_summary_covers_replan_recovery_label_and_decision_step_kind() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/trace.json".to_string(),
        goal: "test".to_string(),
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
            step_id: "decide".to_string(),
            step_kind: StepKind::Decision,
            attempts: 1,
            final_status: StepStatus::Succeeded,
            headline: "succeeded after 1 attempt(s)".to_string(),
        }],
        recovery_events: vec![TraceRecoveryEvent {
            event_type: TraceEventType::Replanned,
            trigger: "goal shifted".to_string(),
            related_step_id: None,
        }],
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        duration: None,
    };

    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/synod-next");

    assert!(rendered.contains("step decide (decision)"), "{rendered}");
    assert!(rendered.contains("replan: goal shifted"), "{rendered}");
}

#[test]
fn render_trace_summary_includes_security_assessment_packet_provenance() {
    let mut trace = ExecutionTrace::new("task-summary-governance", "session", "Governed goal");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GovernanceStarted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "change:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "canon_mode": "security-assessment",
            "run_ref": "canon-run-security",
            "packet_source_stage": "change:implement",
            "packet_binding_reason": "same_stage_rerun"
        }),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::GovernanceCompleted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "change:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "headline": "security assessment packet ready",
            "packet_ref": ".canon/runs/canon-run-security",
            "packet_source_stage": "change:implement",
            "packet_binding_reason": "same_stage_rerun"
        }),
        recorded_at: 1,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let rendered = render_trace_summary(&summary, "explicit-trace", "/synod-next");

    assert!(
        rendered.contains(
            "governance_started: change:verify (security-assessment) [canon-run-security] from change:implement (same_stage_rerun)"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "governance_completed: security assessment packet ready [.canon/runs/canon-run-security] from change:implement (same_stage_rerun)"
        ),
        "{rendered}"
    );
}

#[test]
fn render_trace_summary_covers_pending_running_and_skipped_step_statuses() {
    let statuses = [
        (StepStatus::Pending, "pending"),
        (StepStatus::Running, "running"),
        (StepStatus::Skipped, "skipped"),
    ];

    for (status, expected) in statuses {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/trace.json".to_string(),
            goal: "test".to_string(),
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
                step_id: "step1".to_string(),
                step_kind: StepKind::Agent,
                attempts: 1,
                final_status: status,
                headline: format!("{expected} after 1 attempt(s)"),
            }],
            recovery_events: vec![],
            governance_timeline: Vec::new(),
            governance_next_action: None,
            review_timeline: Vec::new(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            duration: None,
        };
        let rendered = render_trace_summary(&summary, "explicit-trace", "/synod-next");
        assert!(
            rendered.contains(&format!("(agent) {expected} [1")),
            "status {status:?}: {rendered}"
        );
    }
}
