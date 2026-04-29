use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};
use serde_json::json;
use synod::adapters::trace_store::{FileTraceStore, TraceStore};
use synod::domain::limits::TerminalCondition;
use synod::domain::task::{TaskStatus, TerminalReason};
use synod::domain::trace::{ExecutionTrace, TraceEventType};

#[test]
fn start_persists_an_active_session_that_follow_up_commands_reuse_from_current_workspace() {
    let workspace = temp_fixture_workspace("synod-session-flow");
    let start = run_synod_in(&workspace, &["start"]);
    let start_text = terminal_text(&start);

    assert_eq!(start.status.code(), Some(0), "{start_text}");
    assert!(start_text.contains("latest_status: initialized"), "{start_text}");

    let plan = run_synod_in(&workspace, &["plan"]);
    let plan_text = terminal_text(&plan);

    assert_eq!(plan.status.code(), Some(1), "{plan_text}");
    assert!(plan_text.contains("plan: session error"), "{plan_text}");
    assert!(plan_text.contains("reason: active session has no captured goal"), "{plan_text}");
    assert!(plan_text.contains("next_command: synod capture --goal <goal>"), "{plan_text}");
}

#[test]
fn capture_plan_and_run_keep_session_state_and_trace_synchronized() {
    let workspace = temp_fixture_workspace("synod-session-flow-state");

    let start = run_synod_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_synod_in(&workspace, &["capture", "--goal", "Fix the failing add test"]);
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let plan = run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(run_text.contains("terminal_status: succeeded"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");
}

#[test]
fn status_next_and_inspect_reuse_the_active_session_view_and_trace_reference() {
    let workspace = temp_fixture_workspace("synod-session-flow-inspect");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing add test"],).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: planned"), "{status_text}");
    assert!(status_text.contains("current_stage: investigate"), "{status_text}");
    assert!(status_text.contains("next_command: synod run"), "{status_text}");

    let next = run_synod_in(&workspace, &["next"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: synod run"), "{next_text}");

    let run = run_synod_in(&workspace, &["run"]);
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

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("goal: Fix the failing add test"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(!inspect_text.contains("goal: Foreign latest trace"), "{inspect_text}");
}
