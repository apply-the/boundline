use std::fs;

use serde_json::Value;

use crate::workspace_fixture::{
    run_boundline_in, stdout_json, temp_empty_workspace, temp_fixture_workspace, terminal_text,
};

fn canonical_display(path: &std::path::Path) -> String {
    match fs::canonicalize(path) {
        Ok(canonical_path) => canonical_path.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn rendered_probe_report(output: &std::process::Output) -> Value {
    let text = terminal_text(output);
    let envelope: Value = stdout_json(output);
    let rendered = envelope["rendered_output"].as_str().unwrap_or_default();
    let parsed = serde_json::from_str::<Value>(rendered);

    assert!(parsed.is_ok(), "{text}");
    parsed.unwrap_or(Value::Null)
}

#[test]
fn probe_plain_output_reports_bootstrap_state_without_assistant_routes() {
    let workspace = temp_empty_workspace("boundline-probe-contract-bootstrap");
    let output = run_boundline_in(
        &workspace,
        &["probe", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let report: Value = stdout_json(&output);
    assert_eq!(report["workspace"]["initialized"], Value::Bool(false), "{text}");
    assert_eq!(report["recommended_next"]["command"], Value::String("boundline init".into()));
    assert!(report["recommended_next"].get("assistant_command").is_none(), "{text}");
    assert!(report.get("recommended_handoffs").is_none(), "{text}");
}

#[test]
fn probe_host_json_output_wraps_the_rendered_probe_report() {
    let workspace = temp_empty_workspace("boundline-probe-contract-json");
    let output = run_boundline_in(
        &workspace,
        &["probe", "--workspace", workspace.to_string_lossy().as_ref(), "--json"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let envelope: Value = stdout_json(&output);
    assert_eq!(envelope["command_name"], Value::String("probe".into()), "{text}");
    assert_eq!(envelope["exit_status"], Value::String("succeeded".into()), "{text}");

    let report = rendered_probe_report(&output);
    assert_eq!(report["recommended_next"]["command"], Value::String("boundline init".into()));
    assert!(report["recommended_next"].get("assistant_command").is_none(), "{text}");
}

#[test]
fn probe_can_resolve_the_current_workspace_when_flag_is_omitted() {
    let workspace = temp_fixture_workspace("boundline-probe-contract-auto-workspace");
    let output = run_boundline_in(&workspace, &["probe"]);
    let text = terminal_text(&output);
    let expected_path = canonical_display(&workspace);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let report: Value = stdout_json(&output);
    assert_eq!(report["workspace"]["path"], Value::String(expected_path), "{text}");
}

#[test]
fn probe_initialized_workspace_without_provider_credentials_routes_to_doctor() {
    let workspace = temp_fixture_workspace("boundline-probe-contract-doctor");
    let output = run_boundline_in(
        &workspace,
        &["probe", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let report: Value = stdout_json(&output);
    assert_eq!(report["workspace"]["initialized"], Value::Bool(true), "{text}");
    assert_eq!(report["providers"]["healthy"], Value::Bool(false), "{text}");
    assert_eq!(report["recommended_next"]["command"], Value::String("boundline doctor".into()));
    assert_eq!(
        report["recommended_next"]["assistant_command"],
        Value::String("/boundline-doctor".into()),
        "{text}"
    );
    assert_eq!(
        report["recommended_handoffs"][0]["command"],
        Value::String("/boundline-doctor".into()),
        "{text}"
    );
}

#[test]
fn probe_initialized_workspace_with_provider_credentials_routes_to_goal() {
    let workspace = temp_fixture_workspace("boundline-probe-contract-goal");
    let env_write = fs::write(workspace.join(".env"), b"OPENAI_API_KEY=test\n");
    assert!(env_write.is_ok(), "failed to write workspace .env: {env_write:?}");

    let output = run_boundline_in(
        &workspace,
        &["probe", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let report: Value = stdout_json(&output);
    assert_eq!(report["providers"]["healthy"], Value::Bool(true), "{text}");
    assert_eq!(report["session"]["active"], Value::Bool(false), "{text}");
    assert_eq!(report["recommended_next"]["command"], Value::String("boundline goal".into()));
    assert_eq!(
        report["recommended_next"]["assistant_command"],
        Value::String("/boundline-goal".into()),
        "{text}"
    );
}

#[test]
fn probe_reports_semantic_index_health_for_corrupt_index() {
    let workspace = temp_empty_workspace("boundline-probe-contract-semantic-health");
    let index_dir = workspace.join(".boundline/context-intelligence");
    let mkdir = fs::create_dir_all(&index_dir);
    assert!(mkdir.is_ok(), "failed to create semantic index dir: {mkdir:?}");
    let write_db = fs::write(index_dir.join("retrieval-index.sqlite3"), b"fake-db");
    assert!(write_db.is_ok(), "failed to write semantic index fixture: {write_db:?}");

    let output = run_boundline_in(
        &workspace,
        &["probe", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let report: Value = stdout_json(&output);
    assert_eq!(report["capabilities"]["semantic_index"], Value::Bool(true), "{text}");
    assert_eq!(
        report["capabilities"]["semantic_index_health"],
        Value::String("failed".into()),
        "{text}"
    );
}
