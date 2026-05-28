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

#[test]
fn inspect_output_surfaces_canon_aware_explanation_lines() {
    let workspace = temp_fixture_workspace("boundline-cli-inspect-explanation");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    fs::write(
        workspace.join("docs/explanation-brief.md"),
        concat!(
            "# Explanation brief\n\n",
            "Intended outcome: explain the current delivery state for the first fixture slice.\n\n",
            "Domain entities: delivery state, validation evidence, and review evidence are the in-scope domain entities; delivery state depends on validation and review evidence.\n\n",
            "Authoritative persistence store: workspace-local .boundline/session.json and persisted trace files under .boundline/traces/.\n\n",
            "Auth boundary: OAuth2 stops at the API gateway; Boundline service-level authorization begins when the runtime reads persisted trace and session evidence.\n\n",
            "API operations: inspect the persisted trace and session-owned evidence for the run.\n\n",
            "Success criteria: inspect reports source attribution, fallback disclosure, and next-best-action lines for the first fixture slice.\n\n",
            "Validation target: inspect output surfaces explanation attribution, fallback disclosure, and next-best-action lines.\n",
        ),
    )
    .unwrap();
    let run_output = run_boundline(&[
        "run",
        "--goal",
        "Explain the active delivery state",
        "--brief",
        "docs/explanation-brief.md",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output)).expect("trace path");
    let inspect_output =
        run_boundline(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{text}");
    assert!(text.contains("source_attribution: runtime="), "{text}");
    assert!(
        text.contains(
            "fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only"
        ),
        "{text}"
    );
    assert!(text.contains("next_best_action:"), "{text}");
}
