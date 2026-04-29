use serde_json::json;
use synod::adapters::trace_store::{FileTraceStore, TraceStore};
use synod::domain::limits::TerminalCondition;
use synod::domain::task::{TaskStatus, TerminalReason};
use synod::domain::trace::{ExecutionTrace, TraceEventType};

use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};

#[test]
fn cluster_status_classifies_missing_session_members_explicitly() {
    let primary = temp_fixture_workspace("synod-cluster-status-primary");
    let secondary = temp_fixture_workspace("synod-cluster-status-secondary");

    let init = run_synod_in(
        &primary,
        &[
            "cluster",
            "init",
            "--workspace",
            primary.to_string_lossy().as_ref(),
            "--cluster-id",
            "delivery-a",
            "--member",
            primary.to_string_lossy().as_ref(),
            "--member",
            secondary.to_string_lossy().as_ref(),
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let start =
        run_synod_in(&primary, &["start", "--workspace", primary.to_string_lossy().as_ref()]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let status = run_synod_in(
        &primary,
        &["cluster", "status", "--workspace", primary.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("cluster: status"), "{text}");
    assert!(text.contains("healthy"), "{text}");
    assert!(text.contains("missing-session"), "{text}");
}

#[test]
fn cluster_inspect_surfaces_latest_trace_and_missing_trace_gaps() {
    let primary = temp_fixture_workspace("synod-cluster-inspect-primary");
    let secondary = temp_fixture_workspace("synod-cluster-inspect-secondary");

    let init = run_synod_in(
        &primary,
        &[
            "cluster",
            "init",
            "--workspace",
            primary.to_string_lossy().as_ref(),
            "--cluster-id",
            "delivery-a",
            "--member",
            primary.to_string_lossy().as_ref(),
            "--member",
            secondary.to_string_lossy().as_ref(),
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let start =
        run_synod_in(&primary, &["start", "--workspace", primary.to_string_lossy().as_ref()]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let start_secondary =
        run_synod_in(&secondary, &["start", "--workspace", secondary.to_string_lossy().as_ref()]);
    assert_eq!(start_secondary.status.code(), Some(0), "{}", terminal_text(&start_secondary));

    let mut trace = ExecutionTrace::new("task-primary", "session-primary", "cluster goal");
    trace.record_event(TraceEventType::TaskStarted, None, 0, json!({"goal": "cluster goal"}));
    trace.finalize(
        TaskStatus::Succeeded,
        TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
    );
    let trace_path = FileTraceStore::for_workspace(&primary).persist(&trace).unwrap();

    let inspect = run_synod_in(
        &primary,
        &["cluster", "inspect", "--workspace", primary.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{text}");
    assert!(text.contains("cluster: inspect"), "{text}");
    assert!(text.contains(trace_path.to_string_lossy().as_ref()), "{text}");
    assert!(text.contains("missing-trace"), "{text}");
}
