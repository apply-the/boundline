use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, json};

use crate::dashboard_fixture::{DashboardTestResult, require, require_contains, require_eq};
use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::adapters::dashboard_state::DashboardStateAssembler;
use boundline::domain::dashboard::{DashboardActionKind, ExecutionCondition};
use boundline::domain::goal_plan::{GoalPlan, PlannedTask};
use boundline::domain::limits::{RunLimits, TerminalCondition};
use boundline::domain::plan::Plan;
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::step::Step;
use boundline::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};
use boundline::domain::trace::{ExecutionTrace, TraceEvent, TraceEventType};

fn write_record(workspace: &Path, record: &ActiveSessionRecord) -> DashboardTestResult {
    let session_path = workspace.join(".boundline").join("session.json");
    fs::write(session_path, serde_json::to_vec_pretty(record)?)?;
    Ok(())
}

fn write_trace(
    workspace: &Path,
    file_name: &str,
    trace: &ExecutionTrace,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let trace_path = workspace.join(".boundline").join("traces").join(file_name);
    fs::write(&trace_path, serde_json::to_vec_pretty(trace)?)?;
    Ok(trace_path)
}

fn goal_plan(requires_confirmation: bool) -> Result<GoalPlan, Box<dyn std::error::Error>> {
    let mut plan = GoalPlan::new(
        "Fix the failing checkout flow",
        vec![PlannedTask {
            task_id: "task-1".to_string(),
            description: "Run the dashboard validation suite".to_string(),
            target: "tests/contract/dashboard_render_contract.rs".to_string(),
            expected_outcome: Some("Coverage stays above threshold".to_string()),
            decision_type_hint: None,
        }],
    )?;
    if !requires_confirmation {
        plan.confirm()?;
    }
    Ok(plan)
}

fn base_record(
    workspace: &Path,
    status: SessionStatus,
) -> Result<ActiveSessionRecord, Box<dyn std::error::Error>> {
    let needs_goal = !matches!(status, SessionStatus::Initialized | SessionStatus::Invalid);
    let goal_plan = if matches!(
        status,
        SessionStatus::Planned
            | SessionStatus::Running
            | SessionStatus::Succeeded
            | SessionStatus::Failed
            | SessionStatus::Exhausted
            | SessionStatus::Aborted
    ) {
        Some(goal_plan(false)?)
    } else {
        None
    };
    let latest_terminal_reason = if matches!(
        status,
        SessionStatus::Succeeded
            | SessionStatus::Failed
            | SessionStatus::Exhausted
            | SessionStatus::Aborted
    ) {
        Some(TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            format!("{status:?} terminal outcome"),
            None,
        ))
    } else {
        None
    };

    Ok(ActiveSessionRecord {
        session_id: format!("session-{status:?}").to_lowercase(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: if needs_goal { Some("Fix the failing checkout flow".to_string()) } else { None },
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: status,
        latest_terminal_reason,
        latest_trace_ref: None,
        created_at: 1,
        updated_at: 7,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
    })
}

fn waiting_record(workspace: &Path) -> Result<ActiveSessionRecord, Box<dyn std::error::Error>> {
    Ok(ActiveSessionRecord {
        goal_plan: Some(goal_plan(true)?),
        latest_status: SessionStatus::GoalCaptured,
        goal: Some("Fix the failing checkout flow".to_string()),
        ..base_record(workspace, SessionStatus::GoalCaptured)?
    })
}

fn governed_record(workspace: &Path) -> Result<ActiveSessionRecord, Box<dyn std::error::Error>> {
    let request = TaskRunRequest {
        goal: "Fix the failing checkout flow".to_string(),
        input: json!({"kind": "dashboard"}),
        session_id: "session-governed".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: RunLimits::default(),
        initial_context: Some(Map::new()),
    };
    let plan = Plan::new(vec![Step::decision("inspect-dashboard", json!({"mode": "inspect"}))?])?;
    let mut task = Task::new("task-governed", &request, plan)?;
    task.mark_running();
    task.context.state.insert(
        "latest_governance_packet_ref".to_string(),
        json!("/workspace/.canon/packet-1.json"),
    );
    task.context.state.insert("latest_governance_approval".to_string(), json!("approved"));

    Ok(ActiveSessionRecord {
        session_id: "session-governed".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Fix the failing checkout flow".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(task),
        goal_plan: Some(goal_plan(false)?),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Running,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 1,
        updated_at: 11,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
    })
}

fn trace_event(event_id: &str, event_type: TraceEventType, recorded_at: u64) -> TraceEvent {
    TraceEvent {
        event_id: event_id.to_string(),
        event_type,
        step_id: Some("inspect-dashboard".to_string()),
        plan_revision: 1,
        payload: json!({"recorded_at": recorded_at}),
        recorded_at,
    }
}

fn coverage_trace() -> ExecutionTrace {
    ExecutionTrace {
        task_id: "task-governed".to_string(),
        session_id: "session-governed".to_string(),
        goal: "Fix the failing checkout flow".to_string(),
        started_at: 1,
        ended_at: Some(2),
        terminal_status: Some(TaskStatus::Succeeded),
        terminal_reason: Some(TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "goal satisfied",
            None,
        )),
        events: vec![
            trace_event("event-1", TraceEventType::GoalPlanCreated, 1),
            trace_event("event-2", TraceEventType::DecisionVerified, 2),
            trace_event("event-3", TraceEventType::DecisionFailed, 3),
            trace_event("event-4", TraceEventType::DecisionRecovered, 4),
            trace_event("event-5", TraceEventType::CheckpointCreated, 5),
            trace_event("event-6", TraceEventType::TerminalRecorded, 6),
            trace_event("event-7", TraceEventType::RetryScheduled, 7),
            trace_event("event-8", TraceEventType::TaskStarted, 8),
        ],
        trace_location: None,
    }
}

#[test]
fn dashboard_snapshot_matches_status_for_planned_workspace() -> DashboardTestResult {
    let workspace = temp_fixture_workspace("dashboard-snapshot-flow");

    let start = run_boundline_in(&workspace, &["start"]);
    require_eq(start.status.code(), Some(0), &terminal_text(&start))?;
    let capture = run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing add test"]);
    require_eq(capture.status.code(), Some(0), &terminal_text(&capture))?;
    let plan = run_boundline_in(&workspace, &["plan"]);
    require_eq(plan.status.code(), Some(0), &terminal_text(&plan))?;

    let status = run_boundline_in(&workspace, &["status"]);
    require_eq(status.status.code(), Some(0), &terminal_text(&status))?;
    let status_text = terminal_text(&status);

    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    let session = snapshot.session.as_ref().ok_or("planned snapshot requires session")?;
    require_eq(session.latest_status.as_str(), "planned", "latest status")?;
    require_contains(&status_text, "planned", "status output")?;
    require(
        session.next_command.contains("boundline plan")
            || session.next_command.contains("boundline run"),
        "dashboard next command must stay on normal command surface",
    )
}

#[test]
fn dashboard_snapshot_refresh_observes_external_session_changes() -> DashboardTestResult {
    let workspace = temp_fixture_workspace("dashboard-refresh-flow");
    require_eq(run_boundline_in(&workspace, &["start"]).status.code(), Some(0), "start")?;
    require_eq(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix refresh flow"]).status.code(),
        Some(0),
        "capture",
    )?;

    let before = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    require_eq(
        before.session.as_ref().map(|session| session.latest_status.as_str()),
        Some("goal_captured"),
        "before status",
    )?;

    require_eq(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0), "plan")?;
    let after = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    require_eq(
        after.session.as_ref().map(|session| session.latest_status.as_str()),
        Some("planned"),
        "after status",
    )
}

#[test]
fn dashboard_snapshot_projects_status_variants_and_recommended_actions() -> DashboardTestResult {
    let workspace = crate::dashboard_fixture::dashboard_workspace("dashboard-status-variants")?;
    let scenarios = vec![
        (
            base_record(&workspace, SessionStatus::Initialized)?,
            ExecutionCondition::Ready,
            "initialized",
            "Continue bounded execution",
            "boundline capture",
            DashboardActionKind::Continue,
            None,
        ),
        (
            waiting_record(&workspace)?,
            ExecutionCondition::Waiting,
            "goal_captured",
            "Confirm proposed plan",
            "boundline plan --confirm",
            DashboardActionKind::Confirm,
            Some("Plan confirmation is required"),
        ),
        (
            base_record(&workspace, SessionStatus::Running)?,
            ExecutionCondition::Ready,
            "running",
            "Continue bounded execution",
            "boundline run",
            DashboardActionKind::Continue,
            None,
        ),
        (
            base_record(&workspace, SessionStatus::Succeeded)?,
            ExecutionCondition::Complete,
            "succeeded",
            "Inspect completed session",
            "boundline inspect",
            DashboardActionKind::InspectOnly,
            None,
        ),
        (
            base_record(&workspace, SessionStatus::Failed)?,
            ExecutionCondition::Failed,
            "failed",
            "Recover or inspect",
            "boundline status",
            DashboardActionKind::Recover,
            Some("Failed terminal outcome"),
        ),
        (
            base_record(&workspace, SessionStatus::Exhausted)?,
            ExecutionCondition::Exhausted,
            "exhausted",
            "Recover or inspect",
            "boundline status",
            DashboardActionKind::Recover,
            Some("Exhausted terminal outcome"),
        ),
        (
            base_record(&workspace, SessionStatus::Aborted)?,
            ExecutionCondition::Invalid,
            "aborted",
            "Inspect current state",
            "boundline status",
            DashboardActionKind::InspectOnly,
            Some("Aborted terminal outcome"),
        ),
        (
            base_record(&workspace, SessionStatus::Invalid)?,
            ExecutionCondition::Invalid,
            "invalid",
            "Inspect current state",
            "boundline status",
            DashboardActionKind::InspectOnly,
            Some("Session is not ready for normal progression"),
        ),
    ];

    for (
        record,
        expected_condition,
        expected_status,
        expected_label,
        expected_command,
        expected_action,
        expected_blocking_reason,
    ) in scenarios
    {
        write_record(&workspace, &record)?;
        let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
        let session = snapshot.session.as_ref().ok_or("status variant must produce a session")?;
        let action = snapshot.actions.first().ok_or("status variant must produce an action")?;

        require_eq(session.latest_status.as_str(), expected_status, "latest status")?;
        require_eq(session.execution_condition, expected_condition, "execution condition")?;
        require_contains(&session.next_action_label, expected_label, "next action label")?;
        require_contains(&session.next_command, expected_command, "next command")?;
        require_eq(action.action_kind, expected_action, "action kind")?;

        if let Some(expected_reason) = expected_blocking_reason {
            require_contains(
                session.blocking_reason.as_deref().unwrap_or_default(),
                expected_reason,
                "blocking reason",
            )?;
        } else {
            require_eq(session.blocking_reason.is_none(), true, "ready/complete blocking reason")?;
        }
    }

    Ok(())
}

#[test]
fn dashboard_snapshot_projects_trace_events_from_latest_and_stale_references() -> DashboardTestResult
{
    let workspace = crate::dashboard_fixture::dashboard_workspace("dashboard-trace-events")?;
    let trace = coverage_trace();
    let trace_path = write_trace(&workspace, "task-governed.json", &trace)?;

    let latest_record = governed_record(&workspace)?;
    write_record(&workspace, &latest_record)?;
    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    require_eq(snapshot.timeline.len(), 8, "timeline length")?;
    require(
        snapshot.timeline.iter().any(|event| event.event_kind == "plan"),
        "timeline should project plan events",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.event_kind == "action"),
        "timeline should project action events",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.event_kind == "checkpoint"),
        "timeline should project checkpoint events",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.event_kind == "terminal"),
        "timeline should project terminal events",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.event_kind == "replan"),
        "timeline should project replan events",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.event_kind == "session"),
        "timeline should project fallback session events",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Goal plan created"),
        "timeline should include goal plan headline",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Decision verified"),
        "timeline should include verified headline",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Decision failed"),
        "timeline should include failed headline",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Decision recovered"),
        "timeline should include recovered headline",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Checkpoint created"),
        "timeline should include checkpoint headline",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Terminal outcome recorded"),
        "timeline should include terminal headline",
    )?;
    require(
        snapshot.timeline.iter().any(|event| event.headline == "Runtime event recorded"),
        "timeline should include fallback headline",
    )?;

    let mut stale_record = governed_record(&workspace)?;
    stale_record.latest_trace_ref =
        Some(trace_path.with_file_name("missing.json").to_string_lossy().into_owned());
    write_record(&workspace, &stale_record)?;
    let stale_snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    require_contains(
        &stale_snapshot.timeline.first().ok_or("stale trace must yield a degraded event")?.headline,
        "Latest trace reference is unavailable",
        "stale trace headline",
    )
}

#[test]
fn dashboard_snapshot_projects_governed_references_and_compatibility_context() -> DashboardTestResult
{
    let workspace = crate::dashboard_fixture::dashboard_workspace("dashboard-governed-panels")?;
    let record = governed_record(&workspace)?;
    write_record(&workspace, &record)?;

    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    let session = snapshot.session.as_ref().ok_or("governed snapshot requires a session")?;
    let governed_reference = snapshot
        .panels
        .governed_references
        .first()
        .ok_or("governed snapshot requires a governed reference")?;

    require_contains(
        governed_reference.reference.as_str(),
        ".canon/packet-1.json",
        "governed reference path",
    )?;
    require_eq(governed_reference.approval_cue.as_deref(), Some("approved"), "approval cue")?;
    require_eq(governed_reference.read_only, true, "read only flag")?;
    require_contains(
        session.compatibility_context.as_deref().unwrap_or_default(),
        "task-governed remains tied to session session-governed",
        "compatibility context",
    )?;
    require_eq(session.current_step_id.as_deref(), Some("inspect-dashboard"), "current step id")
}
