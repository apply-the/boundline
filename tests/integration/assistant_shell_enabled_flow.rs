use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::workspace_fixture::{
    extract_trace_path, run_synod, temp_broken_fixture_workspace, temp_fixture_workspace,
    terminal_text,
};

fn bootstrap_session(workspace: &Path, goal: &str) {
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let start_output = run_synod(&["start", "--workspace", &workspace_ref]);
    let start_text = terminal_text(&start_output);
    assert_eq!(start_output.status.code(), Some(0), "{start_text}");

    let capture_output = run_synod(&["capture", "--workspace", &workspace_ref, "--goal", goal]);
    let capture_text = terminal_text(&capture_output);
    assert_eq!(capture_output.status.code(), Some(0), "{capture_text}");

    let plan_output = run_synod(&["plan", "--workspace", &workspace_ref]);
    let plan_text = terminal_text(&plan_output);
    assert_eq!(plan_output.status.code(), Some(0), "{plan_text}");
}

fn write_invalid_session(workspace: &Path) {
    let session_dir = workspace.join(".synod");
    fs::create_dir_all(&session_dir).unwrap();
    fs::write(
        session_dir.join("session.json"),
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
    let workspace = temp_fixture_workspace("synod-assistant-shell-enabled");
    bootstrap_session(&workspace, "Fix the failing add test");
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let run_output = run_synod(&["run", "--workspace", &workspace_ref]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("terminal_status: succeeded"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");
    assert!(run_text.contains("next_command: synod inspect"), "{run_text}");

    let status_output = run_synod(&["status", "--workspace", &workspace_ref]);
    let status_text = terminal_text(&status_output);

    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref:"), "{status_text}");
    assert!(status_text.contains("next_command: synod inspect"), "{status_text}");

    let next_output = run_synod(&["next", "--workspace", &workspace_ref]);
    let next_text = terminal_text(&next_output);

    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: synod inspect"), "{next_text}");

    let inspect_output = run_synod(&["inspect", "--workspace", &workspace_ref]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(inspect_text.contains("trace:"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_status_and_next_surface_non_success_session_outcomes_for_routing() {
    let workspace = temp_broken_fixture_workspace("synod-assistant-shell-enabled-broken");
    bootstrap_session(&workspace, "Attempt the fixture patch on a broken workspace");
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let run_output = run_synod(&["run", "--workspace", &workspace_ref]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_reason:"), "{run_text}");
    assert!(run_text.contains("next_command: synod inspect"), "{run_text}");

    let status_output = run_synod(&["status", "--workspace", &workspace_ref]);
    let status_text = terminal_text(&status_output);

    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref:"), "{status_text}");
    assert!(status_text.contains("next_command: synod inspect"), "{status_text}");

    let next_output = run_synod(&["next", "--workspace", &workspace_ref]);
    let next_text = terminal_text(&next_output);

    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: synod inspect"), "{next_text}");

    let inspect_output = run_synod(&["inspect", "--workspace", &workspace_ref]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_reason:"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_explicit_inspect_outputs_selection_mode_and_next_step_cues() {
    let workspace = temp_fixture_workspace("synod-assistant-shell-enabled-inspect");
    let run_output = run_synod(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let run_text = terminal_text(&run_output);
    let trace_path = extract_trace_path(&run_text).expect(&run_text);

    let inspect_output = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: explicit-trace"), "{inspect_text}");
    assert!(inspect_text.contains(&format!("trace: {}", trace_path.display())), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_inspect_trace_read_failures_include_correction_cues() {
    let missing_trace =
        std::env::temp_dir().join(format!("synod-missing-trace-{}.json", Uuid::new_v4()));
    let inspect_output =
        run_synod(&["inspect", "--trace", missing_trace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(3), "{inspect_text}");
    assert!(inspect_text.contains("inspect: trace read failure"), "{inspect_text}");
    assert!(
        inspect_text.contains("terminal_reason: failed to read the requested trace"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("next_command: /synod-inspect"), "{inspect_text}");
    assert!(
        inspect_text.contains("corrected_command: cargo run --bin synod -- inspect --trace"),
        "{inspect_text}"
    );
}

#[test]
fn shell_enabled_workspace_inspect_session_errors_route_back_to_start() {
    let workspace = temp_fixture_workspace("synod-assistant-shell-enabled-invalid");
    write_invalid_session(&workspace);

    let inspect_output =
        run_synod(&["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspect: session error"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: synod start"), "{inspect_text}");
}
