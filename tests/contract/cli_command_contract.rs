use crate::workspace_fixture::{
    run_boundline, temp_broken_fixture_workspace, temp_fixture_workspace, terminal_text,
};

#[test]
fn doctor_uses_the_success_exit_code_for_a_ready_workspace() {
    let workspace = temp_fixture_workspace("boundline-cli-contract-doctor");
    let output = run_boundline(&["doctor", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("ready"), "{text}");
    assert!(text.contains("workspace"), "{text}");
}

#[test]
fn demo_surface_is_not_exposed_as_a_cli_subcommand() {
    let output = run_boundline(&["demo"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(2), "{text}");
    assert!(text.contains("unrecognized subcommand") || text.contains("error:"), "{text}");
}

#[test]
fn run_uses_the_success_exit_code_for_a_simple_bounded_goal() {
    let workspace = temp_fixture_workspace("boundline-cli-contract");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("trace"), "{text}");
    assert!(text.contains("terminal_status"), "{text}");
}

#[test]
fn run_uses_the_non_success_exit_code_when_execution_stops_before_success() {
    let workspace = temp_broken_fixture_workspace("boundline-cli-contract-broken");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Attempt the fixture patch on a broken workspace",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
    assert!(text.contains("trace"), "{text}");
}
