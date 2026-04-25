use crate::workspace_fixture::{
    extract_trace_path, run_synod, temp_broken_fixture_workspace, temp_fixture_workspace,
    terminal_text,
};
use synod::adapters::trace_store::{FileTraceStore, TraceStore};
use synod::cli::inspect::summarize_trace;

#[test]
fn trace_summary_preserves_step_order_and_terminal_reason() {
    let workspace = temp_fixture_workspace("synod-trace-summary");
    let output = run_synod(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert_eq!(summary.trace_ref, trace_path.to_string_lossy());
    assert_eq!(summary.executed_steps[0].step_id, "analyze");
    assert_eq!(summary.executed_steps[1].step_id, "code");
    assert_eq!(summary.executed_steps[2].step_id, "verify");
    assert_eq!(summary.terminal_status, trace.terminal_status.unwrap());
    assert_eq!(summary.terminal_reason, trace.terminal_reason.unwrap());
}

#[test]
fn trace_summary_handles_fixture_terminal_failures_without_fake_recovery_events() {
    let workspace = temp_broken_fixture_workspace("synod-trace-summary-broken");
    let output = run_synod(&[
        "run",
        "--goal",
        "Attempt the fixture patch on a broken workspace",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert!(summary.recovery_events.is_empty());
    assert!(summary.duration.is_some());
    assert_eq!(summary.terminal_status, trace.terminal_status.unwrap());
}
