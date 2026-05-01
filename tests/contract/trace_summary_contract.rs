use crate::workspace_fixture::{
    extract_trace_path, run_synod, temp_adaptive_ordering_boundary_workspace,
    temp_broken_fixture_workspace, temp_fixture_workspace, terminal_text, write_markdown_brief,
};
use synod::adapters::trace_store::{FileTraceStore, TraceStore};
use synod::cli::inspect::summarize_trace;
use synod::cli::output::trace_execution_condition_text;

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
    assert_eq!(summary.executed_steps[1].step_id, "code-fix-add");
    assert_eq!(summary.executed_steps[2].step_id, "verify-fix-add");
    assert!(summary.executed_steps[1].headline.contains("src/lib.rs"));
    assert!(summary.executed_steps[2].headline.contains("validation passed"));
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

#[test]
fn trace_summary_carries_authored_input_summary_and_source_order() {
    let workspace = temp_fixture_workspace("synod-trace-summary-human-input");
    write_markdown_brief(&workspace, "docs/explicit.md", "Explicit context\n");
    write_markdown_brief(&workspace, "docs/referenced.md", "Referenced context\n");

    let output = run_synod(&[
        "run",
        "--goal",
        "Use docs/referenced.md with the explicit brief",
        "--brief",
        "docs/explicit.md",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert_eq!(
        summary.authored_input_summary.as_deref(),
        Some("direct_text + 2 markdown source(s)")
    );
    assert_eq!(
        summary.authored_input_sources,
        vec![
            "direct_text: developer goal".to_string(),
            "attached_markdown: docs/explicit.md".to_string(),
            "referenced_markdown: docs/referenced.md".to_string(),
        ]
    );
}

#[test]
fn trace_summary_uses_shared_route_and_execution_condition_vocabulary_for_compatibility_traces() {
    let workspace = temp_fixture_workspace("synod-trace-summary-shared-vocabulary");
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

    assert_eq!(
        summary.routing_summary.as_deref(),
        Some(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
        )
    );
    assert_eq!(
        trace_execution_condition_text(&summary),
        format!("terminal - {}", summary.terminal_reason.message)
    );
}

#[test]
fn trace_summary_surfaces_broader_adaptive_family_evidence() {
    let workspace = temp_adaptive_ordering_boundary_workspace("synod-trace-summary-ordering");
    let output = run_synod(&[
        "run",
        "--goal",
        "Fix the inclusive threshold boundary",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text).expect(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert!(
        summary
            .adaptive_evidence
            .iter()
            .any(|line| line == "candidate_family: ordering_boundary_flip"),
        "{:?}",
        summary.adaptive_evidence
    );
    assert!(
        summary.adaptive_evidence.iter().any(|line| {
            line.contains("selection_reason: selected src/lib.rs via ordering_boundary_flip")
        }),
        "{:?}",
        summary.adaptive_evidence
    );
}
