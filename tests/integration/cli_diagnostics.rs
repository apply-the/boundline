use crate::workspace_fixture::{run_synod, temp_fixture_workspace, terminal_text};

#[test]
fn doctor_reports_a_ready_workspace_and_actionable_checks() {
    let workspace = temp_fixture_workspace("synod-cli-doctor");
    let output = run_synod(&["doctor", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("ready"), "{text}");
    assert!(text.contains("workspace_execution_profile"), "{text}");
    assert!(text.contains("trace_store"), "{text}");
}
