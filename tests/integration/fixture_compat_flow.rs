use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};

/// Verify `synod run` with execution profile uses the existing fixture path
#[test]
fn fixture_run_with_execution_profile_uses_fixture_path() {
    let workspace = temp_fixture_workspace("fixture-compat");

    let output = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert!(output.status.success(), "fixture run should complete: {text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
    assert!(
        !text.contains("decision "),
        "fixture run should not emit native decision events: {text}"
    );
}
