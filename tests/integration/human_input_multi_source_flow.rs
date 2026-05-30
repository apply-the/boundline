use crate::workspace_fixture::{
    run_boundline_in, temp_fixture_workspace, terminal_text, write_markdown_brief,
};
use serde_json::Value;
use std::fs;

#[test]
fn capture_combines_explicit_and_referenced_markdown_sources_in_stable_order() {
    let workspace = temp_fixture_workspace("boundline-human-multi-source");
    write_markdown_brief(&workspace, "docs/explicit.md", "Explicit context\n");
    write_markdown_brief(&workspace, "docs/referenced.md", "Referenced context\n");

    let capture = run_boundline_in(
        &workspace,
        &[
            "goal",
            "--goal",
            "Use docs/referenced.md with docs/explicit.md as the primary brief",
            "--brief",
            "docs/explicit.md",
        ],
    );
    let capture_text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{capture_text}");

    let session_json: Value = serde_json::from_str(
        &fs::read_to_string(workspace.join(".boundline/session.json")).unwrap(),
    )
    .unwrap();
    let sources = session_json
        .get("authored_brief")
        .and_then(|bundle| bundle.get("sources"))
        .and_then(Value::as_array)
        .expect("sources");

    let file_backed_paths = sources
        .iter()
        .filter_map(|source| source.get("workspace_path").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(file_backed_paths, vec!["docs/explicit.md", "docs/referenced.md"]);
    assert_eq!(
        session_json
            .get("authored_brief")
            .and_then(|bundle| bundle.get("deduplicated_sources"))
            .and_then(Value::as_array)
            .map(|items| { items.iter().filter_map(Value::as_str).collect::<Vec<_>>() })
            .unwrap_or_default(),
        vec!["docs/explicit.md"]
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(
        status_text.contains("authored_input_summary: direct_text + 2 markdown source(s)"),
        "{status_text}"
    );
    assert!(
        status_text.contains("authored_input_deduplicated_sources: docs/explicit.md"),
        "{status_text}"
    );
}

#[test]
fn capture_fails_when_goal_text_references_a_missing_markdown_source() {
    let workspace = temp_fixture_workspace("boundline-human-missing-reference");

    let capture =
        run_boundline_in(&workspace, &["goal", "--goal", "Use docs/missing.md to drive the task"]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains("brief source"), "{text}");
    assert!(text.contains("missing"), "{text}");
}
