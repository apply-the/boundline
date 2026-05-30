use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use uuid::Uuid;

use crate::workspace_fixture::{
    run_boundline, temp_broken_fixture_workspace, temp_fixture_workspace, terminal_text,
};

fn bootstrap_session(workspace: &Path, goal: &str) {
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let goal_output = run_boundline(&["goal", "--workspace", &workspace_ref, "--goal", goal]);
    let goal_text = terminal_text(&goal_output);
    assert_eq!(goal_output.status.code(), Some(0), "{goal_text}");

    let plan_output = run_boundline(&["plan", "--workspace", &workspace_ref, "--flow", "bug-fix"]);
    let plan_text = terminal_text(&plan_output);
    assert_eq!(plan_output.status.code(), Some(0), "{plan_text}");
}

fn latest_trace_path(workspace: &Path) -> PathBuf {
    let session_id = fs::read_to_string(workspace.join(".boundline/active-session")).unwrap();
    let session_id = session_id.trim();
    let session: Value = serde_json::from_slice(
        &fs::read(workspace.join(format!(".boundline/sessions/{session_id}/session.json")))
            .unwrap(),
    )
    .unwrap();
    workspace.join(
        session["latest_trace_ref"]
            .as_str()
            .unwrap_or_else(|| panic!("missing latest_trace_ref in session: {session}")),
    )
}

fn write_invalid_session(workspace: &Path) {
    let session_dir = workspace.join(".boundline");
    fs::create_dir_all(&session_dir).unwrap();
    fs::write(session_dir.join("active-session"), "invalid-session").unwrap();
    let persisted_session_dir = session_dir.join("sessions/invalid-session");
    fs::create_dir_all(&persisted_session_dir).unwrap();
    fs::write(
        persisted_session_dir.join("session.json"),
        format!(
            concat!(
                "{{\n",
                "  \"session_id\": \"\",\n",
                "  \"workspace_ref\": \"{}\",\n",
                "  \"goal\": null,\n",
                "  \"active_task\": null,\n",
                "  \"latest_status\": \"initialized\",\n",
                "  \"latest_terminal_reason\": null,\n",
                "  \"latest_trace_ref\": null,\n",
                "  \"created_at\": 1,\n",
                "  \"updated_at\": 1\n",
                "}}\n"
            ),
            workspace.display()
        ),
    )
    .unwrap();
}

#[test]
fn shell_enabled_session_native_run_status_next_and_workspace_inspect_include_assistant_routing_cues()
 {
    let workspace = temp_fixture_workspace("boundline-assistant-shell-enabled");
    bootstrap_session(&workspace, "Fix the failing add test");
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let run_output = run_boundline(&["run", "--workspace", &workspace_ref]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("latest_status: succeeded"), "{run_text}");
    assert!(run_text.contains("trace="), "{run_text}");
    assert!(run_text.contains("next_command: boundline inspect"), "{run_text}");

    let status_output = run_boundline(&["status", "--workspace", &workspace_ref]);
    let status_text = terminal_text(&status_output);

    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref: "), "{status_text}");
    assert!(status_text.contains("next_command: boundline inspect"), "{status_text}");

    let next_output = run_boundline(&["next", "--workspace", &workspace_ref]);
    let next_text = terminal_text(&next_output);

    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: boundline inspect"), "{next_text}");

    let inspect_output = run_boundline(&["inspect", "--workspace", &workspace_ref]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("execution_condition: terminal -"), "{inspect_text}");
    assert!(inspect_text.contains("trace="), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /boundline-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_status_and_next_surface_session_outcomes_for_routing() {
    let workspace = temp_broken_fixture_workspace("boundline-assistant-shell-enabled-broken");
    bootstrap_session(&workspace, "Attempt the fixture patch on a broken workspace");
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let run_output = run_boundline(&["run", "--workspace", &workspace_ref]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("latest_status: succeeded"), "{run_text}");
    assert!(run_text.contains("trace="), "{run_text}");
    assert!(run_text.contains("next_command: boundline inspect"), "{run_text}");

    let status_output = run_boundline(&["status", "--workspace", &workspace_ref]);
    let status_text = terminal_text(&status_output);

    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref: "), "{status_text}");
    assert!(status_text.contains("next_command: boundline inspect"), "{status_text}");

    let next_output = run_boundline(&["next", "--workspace", &workspace_ref]);
    let next_text = terminal_text(&next_output);

    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: boundline inspect"), "{next_text}");

    let inspect_output = run_boundline(&["inspect", "--workspace", &workspace_ref]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("execution_condition: terminal -"), "{inspect_text}");
    assert!(inspect_text.contains("trace="), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /boundline-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_explicit_inspect_outputs_selection_mode_and_next_step_cues() {
    let workspace = temp_fixture_workspace("boundline-assistant-shell-enabled-inspect");
    bootstrap_session(&workspace, "Fix the failing add test");
    let run_output = run_boundline(&["run", "--workspace", workspace.to_string_lossy().as_ref()]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(0), "{run_text}");
    let trace_path = latest_trace_path(&workspace);

    let inspect_output =
        run_boundline(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: explicit-trace"), "{inspect_text}");
    assert!(inspect_text.contains(&format!("trace={}", trace_path.display())), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /boundline-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_inspect_trace_read_failures_include_correction_cues() {
    let missing_trace =
        std::env::temp_dir().join(format!("boundline-missing-trace-{}.json", Uuid::new_v4()));
    let inspect_output =
        run_boundline(&["inspect", "--trace", missing_trace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(3), "{inspect_text}");
    assert!(inspect_text.contains("inspect: trace read failure"), "{inspect_text}");
    assert!(
        inspect_text.contains("terminal_reason: failed to read the requested trace"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("next_command: /boundline-inspect"), "{inspect_text}");
    assert!(
        inspect_text.contains("corrected_command: boundline inspect --trace"),
        "{inspect_text}"
    );
}

#[test]
fn shell_enabled_workspace_inspect_session_errors_route_back_to_goal() {
    let workspace = temp_fixture_workspace("boundline-assistant-shell-enabled-invalid");
    write_invalid_session(&workspace);

    let inspect_output =
        run_boundline(&["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspect: session error"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: boundline goal --goal <goal>"), "{inspect_text}");
}
