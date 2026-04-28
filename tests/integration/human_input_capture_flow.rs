use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};
use serde_json::Value;
use std::fs;

#[test]
fn capture_with_brief_only_succeeds_and_persists_session() {
    let workspace = temp_fixture_workspace("synod-h-input-capture");
    let brief = workspace.join("brief.md");
    fs::write(&brief, "# Goal\n\nFix the failing add test\n").unwrap();

    let start = run_synod_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture =
        run_synod_in(&workspace, &["capture", "--brief", brief.to_string_lossy().as_ref()]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{text}");
    assert!(text.contains("latest_status: goal_captured"), "{text}");
    assert!(text.contains("Fix the failing add test"), "{text}");

    // Session is persisted; plan must succeed.
    let plan = run_synod_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));
}

#[test]
fn capture_rejects_brief_outside_workspace() {
    let workspace = temp_fixture_workspace("synod-h-input-out-ws");
    let foreign =
        std::env::temp_dir().join(format!("synod-h-input-foreign-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&foreign).unwrap();
    let brief = foreign.join("evil.md");
    fs::write(&brief, "outside\n").unwrap();

    run_synod_in(&workspace, &["start"]);
    let capture =
        run_synod_in(&workspace, &["capture", "--brief", brief.to_string_lossy().as_ref()]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains("must be inside the workspace"), "{text}");
}

#[test]
fn capture_rejects_unsupported_extension() {
    let workspace = temp_fixture_workspace("synod-h-input-bad-ext");
    let brief = workspace.join("notes.txt");
    fs::write(&brief, "no markdown\n").unwrap();

    run_synod_in(&workspace, &["start"]);
    let capture =
        run_synod_in(&workspace, &["capture", "--brief", brief.to_string_lossy().as_ref()]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains(".md or .markdown"), "{text}");
}

#[test]
fn capture_without_goal_or_brief_returns_clear_error() {
    let workspace = temp_fixture_workspace("synod-h-input-empty");
    run_synod_in(&workspace, &["start"]);

    let capture = run_synod_in(&workspace, &["capture"]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains("at least one of --goal or --brief is required"), "{text}");
}

#[test]
fn capture_persists_resolved_source_provenance_in_session_file() {
    let workspace = temp_fixture_workspace("synod-h-input-session-provenance");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    let explicit = workspace.join("docs/explicit.md");
    let referenced = workspace.join("docs/referenced.md");
    fs::write(&explicit, "Explicit context\n").unwrap();
    fs::write(&referenced, "Referenced context\n").unwrap();

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));

    let capture = run_synod_in(
        &workspace,
        &[
            "capture",
            "--goal",
            "Use docs/referenced.md alongside the explicit brief",
            "--brief",
            "docs/explicit.md",
        ],
    );
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{text}");

    let session_path = workspace.join(".synod/session.json");
    let session_json: Value =
        serde_json::from_str(&fs::read_to_string(session_path).unwrap()).unwrap();
    let bundle = session_json
        .get("authored_brief")
        .expect("authored_brief should be persisted after capture");
    let sources = bundle
        .get("sources")
        .and_then(Value::as_array)
        .expect("authored_brief.sources should be an array");

    let file_backed_paths = sources
        .iter()
        .filter_map(|source| source.get("workspace_path").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(file_backed_paths, vec!["docs/explicit.md", "docs/referenced.md"]);
    assert_eq!(
        bundle.get("primary_goal_text").and_then(Value::as_str),
        Some("Use docs/referenced.md alongside the explicit brief")
    );
}

#[test]
fn status_reports_authored_input_summary_and_source_order() {
    let workspace = temp_fixture_workspace("synod-h-input-status-provenance");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    fs::write(workspace.join("docs/explicit.md"), "Explicit context\n").unwrap();
    fs::write(workspace.join("docs/referenced.md"), "Referenced context\n").unwrap();

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &[
                "capture",
                "--goal",
                "Use docs/referenced.md alongside the explicit brief",
                "--brief",
                "docs/explicit.md",
            ],
        )
        .status
        .code(),
        Some(0)
    );

    let status = run_synod_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("authored_input_summary: direct_text + 2 markdown source(s)"), "{text}");
    assert!(text.contains("authored_input_sources: direct_text: developer goal, attached_markdown: docs/explicit.md, referenced_markdown: docs/referenced.md"), "{text}");
}

#[test]
fn capture_records_clarification_for_unbounded_request_and_blocks_plan() {
    let workspace = temp_fixture_workspace("synod-h-input-clarification-flow");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));

    let capture = run_synod_in(
        &workspace,
        &["capture", "--goal", "Improve the platform docs and fix whatever tests are broken"],
    );
    let capture_text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{capture_text}");
    assert!(
        capture_text.contains(
            "clarification_headline: clarification required: narrow the request to one bounded outcome"
        ),
        "{capture_text}"
    );
    assert!(capture_text.contains("clarification_missing_fields: bounded_scope"), "{capture_text}");

    let session_json: Value =
        serde_json::from_str(&fs::read_to_string(workspace.join(".synod/session.json")).unwrap())
            .unwrap();
    let bundle = session_json.get("authored_brief").expect("authored brief should be persisted");
    assert_eq!(
        bundle.get("resolution_state").and_then(Value::as_str),
        Some("clarification_required")
    );
    assert_eq!(
        bundle
            .get("clarification")
            .and_then(|value| value.get("reason_kind"))
            .and_then(Value::as_str),
        Some("unbounded_request")
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("clarification_headline: clarification required: narrow the request to one bounded outcome"), "{status_text}");
    assert!(
        status_text.contains("next_command: synod capture --goal <narrower goal>"),
        "{status_text}"
    );

    let plan = run_synod_in(&workspace, &["plan"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(1), "{plan_text}");
    assert!(
        plan_text.contains("clarification required: narrow the request to one bounded outcome"),
        "{plan_text}"
    );
    assert!(
        plan_text.contains("next_command: synod capture --goal <narrower goal>"),
        "{plan_text}"
    );
}
