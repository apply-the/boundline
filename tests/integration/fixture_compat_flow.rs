use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

/// Verify direct `boundline run --goal` now prefers the native session path even when
/// a compatibility execution profile exists.
#[test]
fn direct_run_with_execution_profile_prefers_native_path_by_default() {
    let workspace = temp_fixture_workspace("fixture-compat");

    let output = run_boundline_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert!(output.status.success(), "direct run should complete: {text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
    assert!(
        text.contains("next_command: boundline checkpoint restore"),
        "direct run should emit checkpoint recovery guidance: {text}"
    );
    assert!(text.contains("routing: native (goal_plan)"), "{text}");
}
