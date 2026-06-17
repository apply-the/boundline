use std::fs;
use std::path::Path;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::adapters::trace_store::{FileTraceStore, TraceStore};
use boundline::domain::limits::TerminalCondition;
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::task::{TaskStatus, TerminalReason};
use boundline::domain::trace::{ExecutionTrace, TraceEventType};
use serde_json::json;
use uuid::Uuid;

fn persist_initialized_session(workspace: &Path) {
    let now = 10;
    let session = ActiveSessionRecord {
        session_id: Uuid::new_v4().to_string(),
        workspace_ref: workspace.canonicalize().unwrap().to_string_lossy().into_owned(),
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
        latest_trace_ref: None,
        created_at: now,
        updated_at: now,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join(".boundline").join("session.json"),
        serde_json::to_vec_pretty(&session).unwrap(),
    )
    .unwrap();
}

#[test]
fn initialized_session_without_a_goal_guides_follow_up_commands_from_current_workspace() {
    let workspace = temp_fixture_workspace("boundline-session-flow");
    persist_initialized_session(&workspace);

    let plan = run_boundline_in(&workspace, &["plan"]);
    let plan_text = terminal_text(&plan);

    assert_eq!(plan.status.code(), Some(1), "{plan_text}");
    assert!(plan_text.contains("plan: session error"), "{plan_text}");
    assert!(plan_text.contains("reason: active session has no goal"), "{plan_text}");
    assert!(plan_text.contains("next_command: boundline goal --goal <goal>"), "{plan_text}");
}

#[test]
fn goal_plan_and_run_keep_session_state_and_trace_synchronized() {
    let workspace = temp_fixture_workspace("boundline-session-flow-state");

    let goal = run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing add test"]);
    assert_eq!(goal.status.code(), Some(0), "{}", terminal_text(&goal));

    let plan = run_boundline_in(&workspace, &["plan", "--flow", "bug-fix"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("terminal_status: succeeded"), "{run_text}");
    assert!(run_text.contains("next_command: boundline checkpoint restore"), "{run_text}");
    assert!(run_text.contains("trace="), "{run_text}");
}

#[test]
fn status_next_and_inspect_reuse_the_active_session_view_and_trace_reference() {
    let workspace = temp_fixture_workspace("boundline-session-flow-inspect");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing add test"],)
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: planned"), "{status_text}");
    assert!(status_text.contains("current_stage: investigate"), "{status_text}");
    assert!(status_text.contains("next_command: boundline run"), "{status_text}");

    let next = run_boundline_in(&workspace, &["next"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: boundline run"), "{next_text}");

    let run = run_boundline_in(&workspace, &["run"]);
    assert_eq!(run.status.code(), Some(0), "{}", terminal_text(&run));

    let store = FileTraceStore::for_workspace(&workspace);
    let mut foreign_trace =
        ExecutionTrace::new("foreign-task", "foreign-session", "Foreign latest trace");
    foreign_trace.started_at = u64::MAX - 10;
    foreign_trace.record_event(
        TraceEventType::TaskStarted,
        None,
        0,
        json!({"goal": foreign_trace.goal}),
    );
    foreign_trace.finalize(
        TaskStatus::Failed,
        TerminalReason::new(TerminalCondition::UnrecoverableError, "foreign trace", None),
    );
    foreign_trace.ended_at = Some(u64::MAX - 1);
    store.persist(&foreign_trace).unwrap();

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("goal: Fix the failing add test"), "{inspect_text}");
    assert!(inspect_text.contains("latest_status: succeeded"), "{inspect_text}");
    assert!(!inspect_text.contains("goal: Foreign latest trace"), "{inspect_text}");
}
