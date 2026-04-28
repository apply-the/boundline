use crate::workspace_fixture::{
    extract_trace_path, run_synod, run_synod_in, temp_fixture_workspace, terminal_text,
    write_markdown_brief,
};

#[test]
fn status_surface_reports_compact_authored_input_summary_and_deduplicated_source_order() {
    let workspace = temp_fixture_workspace("synod-human-session-contract-status");
    write_markdown_brief(&workspace, "docs/explicit.md", "Explicit context\n");
    write_markdown_brief(&workspace, "docs/referenced.md", "Referenced context\n");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    let capture = run_synod_in(
        &workspace,
        &[
            "capture",
            "--goal",
            "Use docs/referenced.md and docs/explicit.md together",
            "--brief",
            "docs/explicit.md",
            "--brief",
            "docs/explicit.md",
        ],
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let status = run_synod_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("authored_input_summary: direct_text + 2 markdown source(s)"), "{text}");
    assert!(text.contains("authored_input_sources: direct_text: developer goal, attached_markdown: docs/explicit.md, referenced_markdown: docs/referenced.md"), "{text}");
    assert!(text.contains("authored_input_deduplicated_sources: docs/explicit.md"), "{text}");
    assert!(!text.contains("docs/explicit.md, attached_markdown: docs/explicit.md"), "{text}");
}

#[test]
fn inspect_surface_reports_authored_input_provenance_for_direct_run() {
    let workspace = temp_fixture_workspace("synod-human-session-contract-inspect");
    write_markdown_brief(&workspace, "docs/explicit.md", "Explicit context\n");
    write_markdown_brief(&workspace, "docs/referenced.md", "Referenced context\n");

    let run = run_synod(&[
        "run",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
        "--goal",
        "Use docs/referenced.md with the explicit brief",
        "--brief",
        "docs/explicit.md",
        "--brief",
        "docs/explicit.md",
    ]);
    assert_eq!(run.status.code(), Some(0), "{}", terminal_text(&run));
    let trace_path = extract_trace_path(&terminal_text(&run)).expect("trace path");

    let inspect = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{text}");
    assert!(text.contains("authored_input_summary: direct_text + 2 markdown source(s)"), "{text}");
    assert!(text.contains("attached_markdown: docs/explicit.md"), "{text}");
    assert!(text.contains("referenced_markdown: docs/referenced.md"), "{text}");
    assert!(text.contains("authored_input_deduplicated_sources: docs/explicit.md"), "{text}");
}

#[test]
fn inspect_surface_reports_clarification_for_direct_run_blocked_before_planning() {
    let workspace = temp_fixture_workspace("synod-human-session-contract-clarification");

    let run = run_synod(&[
        "run",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
        "--goal",
        "Improve the platform docs and fix whatever tests are broken",
    ]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("clarification_headline: clarification required: narrow the request to one bounded outcome"), "{run_text}");
    let trace_path = extract_trace_path(&run_text).expect("trace path");

    let inspect = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{text}");
    assert!(text.contains("clarification_headline: clarification required: narrow the request to one bounded outcome"), "{text}");
    assert!(text.contains("clarification_prompt: Narrow the request to one bounded bug-fix, change, or delivery outcome."), "{text}");
    assert!(text.contains("clarification_missing_fields: bounded_scope"), "{text}");
}
