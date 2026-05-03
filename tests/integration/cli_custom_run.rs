use crate::workspace_fixture::{
    extract_trace_path, run_boundline, temp_broken_fixture_workspace, temp_fixture_workspace,
    temp_replanning_execution_workspace, terminal_text,
};

#[test]
fn custom_run_executes_the_fixture_vertical_slice_and_persists_a_trace() {
    let workspace = temp_fixture_workspace("boundline-cli-run");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("analyze"), "{text}");
    assert!(text.contains("code"), "{text}");
    assert!(text.contains("verify"), "{text}");
    assert!(text.contains("changed_files: src/lib.rs"), "{text}");
    assert!(text.contains("validation: passed"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
    assert!(trace_path.as_ref().is_some_and(|path| path.exists()), "{text}");
}

#[test]
fn custom_run_reports_non_success_and_keeps_the_trace_for_inspection() {
    let workspace = temp_broken_fixture_workspace("boundline-cli-run-broken");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Attempt the fixture patch on a broken workspace",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
    assert!(text.contains("patch") || text.contains("failed"), "{text}");
    assert!(trace_path.as_ref().is_some_and(|path| path.exists()), "{text}");
}

#[test]
fn custom_run_replans_to_a_later_execution_attempt_after_failed_validation() {
    let workspace = temp_replanning_execution_workspace("boundline-cli-run-replan");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Recover after the first validation fails",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("replan after verify-bad-fix"), "{text}");
    assert!(text.contains("changed_files: src/lib.rs"), "{text}");
    assert!(text.contains("validation: passed"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}
