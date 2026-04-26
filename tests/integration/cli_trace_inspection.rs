use crate::workspace_fixture::{
    extract_trace_path, run_synod, temp_broken_fixture_workspace, temp_fixture_workspace,
    terminal_text,
};

#[test]
fn inspect_command_reconstructs_step_order_from_a_successful_fixture_trace() {
    let workspace = temp_fixture_workspace("synod-cli-inspect");
    let run_output = run_synod(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output)).expect("trace path");
    let inspect_output = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{text}");
    assert!(text.contains("analyze"), "{text}");
    assert!(text.contains("code"), "{text}");
    assert!(text.contains("verify"), "{text}");
    assert!(text.contains("src/lib.rs"), "{text}");
    assert!(text.contains("validation passed"), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
}

#[test]
fn inspect_command_highlights_non_success_terminal_reasons() {
    let workspace = temp_broken_fixture_workspace("synod-cli-inspect-broken");
    let run_output = run_synod(&[
        "run",
        "--goal",
        "Attempt the fixture patch on a broken workspace",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output)).expect("trace path");
    let inspect_output = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
    assert!(text.contains("failed") || text.contains("exhausted"), "{text}");
}
