use crate::workspace_fixture::{
    extract_trace_path, run_boundline, temp_broken_fixture_workspace, temp_fixture_workspace,
    terminal_text,
};
use std::fs;

#[test]
fn inspect_command_reconstructs_step_order_from_a_successful_fixture_trace() {
    let workspace = temp_fixture_workspace("boundline-cli-inspect");
    let run_output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output)).expect("trace path");
    let inspect_output =
        run_boundline(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
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
    let workspace = temp_broken_fixture_workspace("boundline-cli-inspect-broken");
    let run_output = run_boundline(&[
        "run",
        "--goal",
        "Attempt the fixture patch on a broken workspace",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output)).expect("trace path");
    let inspect_output =
        run_boundline(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
    assert!(text.contains("failed") || text.contains("exhausted"), "{text}");
}

#[test]
fn inspect_command_surfaces_authored_input_summary_and_sources() {
    let workspace = temp_fixture_workspace("boundline-cli-inspect-human-input");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    fs::write(workspace.join("docs/explicit.md"), "Explicit context\n").unwrap();
    fs::write(workspace.join("docs/referenced.md"), "Referenced context\n").unwrap();

    let run_output = run_boundline(&[
        "run",
        "--goal",
        "Use docs/referenced.md alongside the explicit brief",
        "--brief",
        "docs/explicit.md",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output)).expect("trace path");
    let inspect_output =
        run_boundline(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{text}");
    assert!(text.contains("authored_input_summary: direct_text + 2 markdown source(s)"), "{text}");
    assert!(text.contains("authored_input_sources: direct_text: developer goal, attached_markdown: docs/explicit.md, referenced_markdown: docs/referenced.md"), "{text}");
}
