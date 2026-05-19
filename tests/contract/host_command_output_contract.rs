use serde_json::Value;

use crate::workspace_fixture::{
    run_boundline_in, stdout_json, temp_fixture_workspace, terminal_text,
};

#[test]
fn session_lifecycle_commands_can_emit_structured_host_output() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-session");

    let start = run_boundline_in(&workspace, &["start", "--json"]);
    let start_text = terminal_text(&start);
    assert_eq!(start.status.code(), Some(0), "{start_text}");
    let start_json: Value = stdout_json(&start);
    assert_eq!(start_json["command_name"], "start", "{start_text}");
    assert_eq!(start_json["exit_status"], "succeeded", "{start_text}");
    assert_eq!(start_json["session_status"]["latest_status"], "initialized", "{start_text}");
    assert!(
        start_json["rendered_output"]
            .as_str()
            .unwrap_or_default()
            .contains("latest_status: initialized"),
        "{start_text}"
    );

    let capture =
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing add test", "--json"]);
    let capture_text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{capture_text}");
    let capture_json: Value = stdout_json(&capture);
    assert_eq!(capture_json["command_name"], "capture", "{capture_text}");
    assert_eq!(
        capture_json["session_status"]["goal"], "Fix the failing add test",
        "{capture_text}"
    );

    let plan = run_boundline_in(&workspace, &["plan", "--json"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    let plan_json: Value = stdout_json(&plan);
    assert_eq!(plan_json["command_name"], "plan", "{plan_text}");
    assert_eq!(plan_json["session_status"]["latest_status"], "planned", "{plan_text}");

    let status = run_boundline_in(&workspace, &["status", "--json"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    let status_json: Value = stdout_json(&status);
    assert_eq!(status_json["command_name"], "status", "{status_text}");
    assert!(status_json["session_status"]["next_command"].is_string(), "{status_text}");

    let next = run_boundline_in(&workspace, &["next", "--json"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    let next_json: Value = stdout_json(&next);
    assert_eq!(next_json["command_name"], "next", "{next_text}");
    assert!(next_json["session_status"]["next_command"].is_string(), "{next_text}");
}

#[test]
fn run_and_inspect_can_emit_structured_trace_output() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-trace");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing add test"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan", "--confirm"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run", "--json"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    let run_json: Value = stdout_json(&run);
    assert_eq!(run_json["command_name"], "run", "{run_text}");
    assert_eq!(run_json["exit_status"], "succeeded", "{run_text}");
    assert_eq!(run_json["trace_summary"]["terminal_status"], "succeeded", "{run_text}");
    assert!(run_json["trace_location"].is_string(), "{run_text}");

    let inspect = run_boundline_in(&workspace, &["inspect", "--json"]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    let inspect_json: Value = stdout_json(&inspect);
    assert_eq!(inspect_json["command_name"], "inspect", "{inspect_text}");
    assert_eq!(inspect_json["trace_summary"]["terminal_status"], "succeeded", "{inspect_text}");
    assert!(inspect_json["trace_summary"]["trace_ref"].is_string(), "{inspect_text}");
}

#[test]
fn invalid_invocations_can_emit_structured_host_output() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-invalid");

    let capture = run_boundline_in(&workspace, &["capture", "--goal", "   ", "--json"]);
    let capture_text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(2), "{capture_text}");

    let capture_json: Value = stdout_json(&capture);
    assert_eq!(capture_json["command_name"], "capture", "{capture_text}");
    assert_eq!(capture_json["exit_status"], "invalid_invocation", "{capture_text}");
    assert_eq!(
        capture_json["rendered_output"], "capture requires a non-empty --goal",
        "{capture_text}"
    );
    assert!(capture_json["trace_location"].is_null(), "{capture_text}");
    assert!(capture_json["session_status"].is_null(), "{capture_text}");
    assert!(capture_json["trace_summary"].is_null(), "{capture_text}");
}

#[test]
fn partial_setup_status_output_surfaces_fallback_disclosure_and_next_best_action() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-partial");

    let start = run_boundline_in(&workspace, &["start", "--json"]);
    let start_text = terminal_text(&start);
    assert_eq!(start.status.code(), Some(0), "{start_text}");

    let status = run_boundline_in(&workspace, &["status", "--json"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");

    let status_json: Value = stdout_json(&status);
    let rendered = status_json["rendered_output"].as_str().unwrap_or_default();
    assert!(rendered.contains("source_attribution: runtime="), "{status_text}");
    assert!(
        rendered.contains(
            "fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only"
        ),
        "{status_text}"
    );
    assert!(rendered.contains("next_best_action:"), "{status_text}");
}

#[test]
fn inspect_output_surfaces_runtime_source_attribution() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-inspect");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Explain why this delivery is safe"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan", "--confirm"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect = run_boundline_in(&workspace, &["inspect", "--json"]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");

    let inspect_json: Value = stdout_json(&inspect);
    let rendered = inspect_json["rendered_output"].as_str().unwrap_or_default();
    assert!(rendered.contains("source_attribution: runtime="), "{inspect_text}");
    assert!(
        rendered.contains(
            "fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only"
        ),
        "{inspect_text}"
    );
    assert!(rendered.contains("next_best_action:"), "{inspect_text}");
}
