use serde_json::Value;

use crate::workspace_fixture::{
    run_boundline_in, stdout_json, temp_fixture_workspace, terminal_text,
};

#[test]
fn structured_run_and_inspect_output_preserve_trace_and_terminal_reasoning() {
    let workspace = temp_fixture_workspace("boundline-host-trace-runtime");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing add test"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan", "--confirm"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run", "--json"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    let run_json: Value = stdout_json(&run);
    assert_eq!(run_json["trace_summary"]["terminal_status"], "succeeded", "{run_text}");
    assert!(run_json["trace_summary"]["terminal_reason"].is_object(), "{run_text}");
    assert!(run_json["trace_location"].is_string(), "{run_text}");

    let inspect = run_boundline_in(&workspace, &["inspect", "--json"]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    let inspect_json: Value = stdout_json(&inspect);
    assert_eq!(inspect_json["trace_summary"]["terminal_status"], "succeeded", "{inspect_text}");
    assert!(inspect_json["trace_summary"]["executed_steps"].is_array(), "{inspect_text}");
    assert!(
        inspect_json["rendered_output"]
            .as_str()
            .unwrap_or_default()
            .contains("terminal_status: succeeded"),
        "{inspect_text}"
    );
}

#[test]
fn structured_inspect_failure_keeps_non_success_exit_and_text_fallback() {
    let workspace = temp_fixture_workspace("boundline-host-trace-runtime-failure");
    let missing_trace = workspace.join("missing-trace.json");

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--trace", missing_trace.to_string_lossy().as_ref(), "--json"],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(3), "{inspect_text}");

    let inspect_json: Value = stdout_json(&inspect);
    assert_eq!(inspect_json["command_name"], "inspect", "{inspect_text}");
    assert_eq!(inspect_json["exit_status"], "trace_read_failure", "{inspect_text}");
    assert!(inspect_json["trace_summary"].is_null(), "{inspect_text}");
    assert!(
        inspect_json["rendered_output"]
            .as_str()
            .unwrap_or_default()
            .contains("inspect: trace read failure"),
        "{inspect_text}"
    );
}

#[test]
fn s7_host_status_advises_partial_setup_with_explicit_fallback() {
    let workspace = temp_fixture_workspace("boundline-host-trace-runtime-s7-partial");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));

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
