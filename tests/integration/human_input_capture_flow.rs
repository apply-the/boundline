use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use serde_json::Value;
use std::fs;

fn stdout_json_lines(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).unwrap_or_else(|error| {
                panic!("failed to parse orchestrate JSON line `{line}`: {error}")
            })
        })
        .collect()
}

#[test]
fn capture_with_brief_only_succeeds_and_persists_session() {
    let workspace = temp_fixture_workspace("boundline-h-input-capture");
    let brief = workspace.join("brief.md");
    fs::write(&brief, "# Goal\n\nFix the failing add test\n").unwrap();

    let capture =
        run_boundline_in(&workspace, &["goal", "--brief", brief.to_string_lossy().as_ref()]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{text}");
    assert!(text.contains("latest_status: goal_captured"), "{text}");
    assert!(text.contains("Fix the failing add test"), "{text}");

    // Session is persisted; plan must succeed.
    let plan = run_boundline_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));
}

#[test]
fn brief_only_goal_uses_brief_content_for_session_slug() {
    let workspace = temp_fixture_workspace("boundline-h-input-brief-slug");
    let brief = workspace.join("plan.md");
    fs::write(
        &brief,
        "# Rust service brief\n\nImplement a rust microservice for user management.\n",
    )
    .unwrap();

    let capture = run_boundline_in(&workspace, &["goal", "--brief", "plan.md"]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{text}");

    let session_json: Value = serde_json::from_str(
        &fs::read_to_string(workspace.join(".boundline/session.json")).unwrap(),
    )
    .unwrap();
    let session_id = session_json["session_id"].as_str().unwrap_or_default();
    assert!(session_id.ends_with("rust-service-brief"), "{session_id}");
    assert!(!session_id.ends_with("plan-md"), "{session_id}");
}

#[test]
fn capture_rejects_brief_outside_workspace() {
    let workspace = temp_fixture_workspace("boundline-h-input-out-ws");
    let foreign =
        std::env::temp_dir().join(format!("boundline-h-input-foreign-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&foreign).unwrap();
    let brief = foreign.join("evil.md");
    fs::write(&brief, "outside\n").unwrap();

    let capture =
        run_boundline_in(&workspace, &["goal", "--brief", brief.to_string_lossy().as_ref()]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains("must be inside the workspace"), "{text}");
}

#[test]
fn capture_rejects_unsupported_extension() {
    let workspace = temp_fixture_workspace("boundline-h-input-bad-ext");
    let brief = workspace.join("notes.txt");
    fs::write(&brief, "no markdown\n").unwrap();

    let capture =
        run_boundline_in(&workspace, &["goal", "--brief", brief.to_string_lossy().as_ref()]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains(".md or .markdown"), "{text}");
}

#[test]
fn capture_without_goal_or_brief_returns_clear_error() {
    let workspace = temp_fixture_workspace("boundline-h-input-empty");

    let capture = run_boundline_in(&workspace, &["goal"]);
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains("at least one of --goal or --brief is required"), "{text}");
}

#[test]
fn capture_persists_resolved_source_provenance_in_session_file() {
    let workspace = temp_fixture_workspace("boundline-h-input-session-provenance");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    let explicit = workspace.join("docs/explicit.md");
    let referenced = workspace.join("docs/referenced.md");
    fs::write(&explicit, "Explicit context\n").unwrap();
    fs::write(&referenced, "Referenced context\n").unwrap();

    let capture = run_boundline_in(
        &workspace,
        &[
            "goal",
            "--goal",
            "Use docs/referenced.md alongside the explicit brief",
            "--brief",
            "docs/explicit.md",
        ],
    );
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{text}");

    let session_path = workspace.join(".boundline/session.json");
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
fn status_reports_authored_input_source_order_after_goal_capture() {
    let workspace = temp_fixture_workspace("boundline-h-input-status-provenance");
    fs::create_dir_all(workspace.join("docs")).unwrap();
    fs::write(workspace.join("docs/explicit.md"), "Explicit context\n").unwrap();
    fs::write(workspace.join("docs/referenced.md"), "Referenced context\n").unwrap();

    assert_eq!(
        run_boundline_in(
            &workspace,
            &[
                "goal",
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

    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("authored_input_sources: direct_text: developer goal, attached_markdown: docs/explicit.md, referenced_markdown: docs/referenced.md"), "{text}");
    assert!(text.contains("latest_status: goal_captured"), "{text}");
    assert!(text.contains("next_command: boundline plan"), "{text}");
}

#[test]
fn capture_records_clarification_for_unbounded_request_and_blocks_plan() {
    let workspace = temp_fixture_workspace("boundline-h-input-clarification-flow");

    let capture = run_boundline_in(
        &workspace,
        &["goal", "--goal", "Improve the platform docs and fix whatever tests are broken"],
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

    let session_json: Value = serde_json::from_str(
        &fs::read_to_string(workspace.join(".boundline/session.json")).unwrap(),
    )
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

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("clarification_headline: clarification required: narrow the request to one bounded outcome"), "{status_text}");
    assert!(
        status_text.contains("next_command: boundline goal --goal <narrower goal>"),
        "{status_text}"
    );

    let plan = run_boundline_in(&workspace, &["plan"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(1), "{plan_text}");
    assert!(
        plan_text.contains("clarification required: narrow the request to one bounded outcome"),
        "{plan_text}"
    );
    assert!(
        plan_text.contains("next_command: boundline goal --goal <narrower goal>"),
        "{plan_text}"
    );
}

#[test]
fn orchestrate_brief_only_reuses_existing_goal_as_planning_input() {
    let workspace = temp_fixture_workspace("boundline-h-input-orchestrate-plan-brief");
    let plan_input = workspace.join("plan.md");
    fs::write(
        &plan_input,
        concat!(
            "# Rust service brief\n\n",
            "Implement a rust microservice for user management.\n\n",
            "- API surface: create and read users over HTTP.\n",
            "- Authoritative persistence store: SQLite for the first slice.\n",
            "- Authentication boundary: OAuth2 token validation stops at the edge; service authorization begins in the application layer.\n",
            "- Validation target: cargo test.\n",
        ),
    )
    .unwrap();

    let goal = run_boundline_in(
        &workspace,
        &["goal", "--goal", "Build a bounded user management microservice"],
    );
    assert_eq!(goal.status.code(), Some(0), "{}", terminal_text(&goal));

    let orchestrate = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--brief",
            "plan.md",
            "--assistant-host",
            "copilot",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let text = terminal_text(&orchestrate);
    assert_eq!(orchestrate.status.code(), Some(0), "{text}");

    let frames = stdout_json_lines(&String::from_utf8_lossy(&orchestrate.stdout));
    assert!(
        frames.iter().any(|frame| frame["event_kind"] == "artifact_recorded"
            && frame.get("artifact").and_then(|a| a.get("artifact_kind")).and_then(|k| k.as_str())
                == Some("plan_brief")),
        "expected plan_brief artifact in output: {text}"
    );
    assert!(
        !frames
            .iter()
            .any(|frame| frame["event_kind"] == "phase_request" && frame["stage_key"] == "goal"),
        "goal stage phase_request should not appear when the existing goal plus brief already answer the planning questions: {text}"
    );

    let session_json: Value = serde_json::from_str(
        &fs::read_to_string(workspace.join(".boundline/session.json")).unwrap(),
    )
    .unwrap();
    let authored_brief = &session_json["authored_brief"];
    assert_eq!(
        authored_brief["primary_goal_text"].as_str(),
        Some("Build a bounded user management microservice")
    );
    let source_labels = authored_brief["sources"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|source| {
            let kind = source.get("kind").and_then(Value::as_str)?;
            let label = source.get("workspace_path").and_then(Value::as_str).unwrap_or_else(|| {
                source.get("display_name").and_then(Value::as_str).unwrap_or_default()
            });
            Some(format!("{kind}:{label}"))
        })
        .collect::<Vec<_>>();
    assert!(source_labels.contains(&"direct_text:developer goal".to_string()), "{source_labels:?}");
    assert!(source_labels.contains(&"attached_markdown:plan.md".to_string()), "{source_labels:?}");
}

#[test]
fn orchestrate_goal_clarification_accepts_request_id_and_answer() {
    let workspace = temp_fixture_workspace("boundline-h-input-orchestrate-goal-answer");

    let first = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Build a bounded user management microservice",
            "--assistant-host",
            "copilot",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let first_text = terminal_text(&first);
    assert_eq!(first.status.code(), Some(0), "{first_text}");

    let first_frames = stdout_json_lines(&String::from_utf8_lossy(&first.stdout));
    let first_request = first_frames
        .iter()
        .find(|frame| frame["event_kind"] == "phase_request" && frame["stage_key"] == "goal")
        .expect("expected a goal clarification phase_request");
    assert_eq!(
        first_request["phase_request"]["question"].as_str(),
        Some("Which persistence store is authoritative for the first slice?")
    );
    assert_eq!(
        first_request["phase_request"]["expected_answer"]["type"].as_str(),
        Some("suggested_choice")
    );
    let option_labels = first_request["phase_request"]["expected_answer"]["options"]
        .as_array()
        .expect("suggested_choice should include selectable options")
        .iter()
        .filter_map(|option| option["label"].as_str())
        .collect::<Vec<_>>();
    assert!(
        option_labels.contains(&"PostgreSQL")
            && option_labels.contains(&"SQLite")
            && option_labels.contains(&"in-memory"),
        "expected persistence-store suggested options, got {option_labels:?}"
    );
    let first_request_id = first_request["phase_request"]["request_id"]
        .as_str()
        .expect("phase_request.request_id should be present")
        .to_string();

    let second = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--assistant-host",
            "copilot",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
            "--request-id",
            &first_request_id,
            "--answer",
            "Postgres",
        ],
    );
    let second_text = terminal_text(&second);
    assert_eq!(second.status.code(), Some(0), "{second_text}");

    let second_frames = stdout_json_lines(&String::from_utf8_lossy(&second.stdout));
    assert!(
        second_frames.iter().any(|frame| frame["event_kind"] == "session_updated"
            && frame["message"] == "applied the clarification answer to the active goal"),
        "expected a session_updated event after answering the goal clarification: {second_text}"
    );
    let second_request = second_frames
        .iter()
        .find(|frame| frame["event_kind"] == "phase_request" && frame["stage_key"] == "goal")
        .expect("expected the next goal clarification phase_request");
    assert_ne!(
        second_request["phase_request"]["request_id"].as_str(),
        Some(first_request_id.as_str())
    );
    assert_eq!(
        second_request["phase_request"]["question"].as_str(),
        Some("Where does OAuth2 or authentication stop and service-level authorization begin?")
    );

    let session_json: Value = serde_json::from_str(
        &fs::read_to_string(workspace.join(".boundline/session.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        session_json["authored_brief"]["primary_goal_text"].as_str(),
        Some("Build a bounded user management microservice\n\nClarification answer: Postgres")
    );
}

#[test]
fn orchestrate_with_slug_embeds_slug_in_session_id() {
    let workspace = temp_fixture_workspace("boundline-h-input-slug");

    let output = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Fix the failing add test",
            "--slug",
            "fix-add-test",
            "--assistant-host",
            "copilot",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(0), "{text}");

    let frames = stdout_json_lines(&String::from_utf8_lossy(&output.stdout));
    let opened = frames
        .iter()
        .find(|frame| frame["event_kind"] == "session_opened")
        .expect("expected a session_opened event");

    let session_ref = opened["session_ref"].as_str().unwrap_or("");
    assert!(
        session_ref.contains("fix-add-test"),
        "session_ref should embed the slug override, got: {session_ref}"
    );
}

#[test]
fn orchestrate_brief_only_on_fresh_workspace_bootstraps_via_brief_only_path() {
    let workspace = temp_fixture_workspace("boundline-h-input-brief-bootstrap");
    let brief = workspace.join("goal-brief.md");
    fs::write(&brief, "# Goal\n\nFix the failing add test\n").unwrap();

    let output = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--brief",
            brief.to_string_lossy().as_ref(),
            "--assistant-host",
            "copilot",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(0), "{text}");

    let frames = stdout_json_lines(&String::from_utf8_lossy(&output.stdout));
    assert!(
        frames.iter().any(|frame| frame["event_kind"] == "session_opened"),
        "expected a session_opened event when bootstrapping from brief on fresh workspace; got: {text}"
    );
    assert!(
        !frames.iter().any(|frame| {
            frame["event_kind"] == "phase_request"
                && frame["message"]
                    .as_str()
                    .map(|m| m.contains("provide a goal or brief"))
                    .unwrap_or(false)
        }),
        "orchestrate should not emit 'provide a goal or brief' phase_request when a brief is supplied; got: {text}"
    );
}
