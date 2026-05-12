use crate::workspace_fixture::{
    run_boundline, run_boundline_in, temp_empty_workspace, terminal_text,
};

#[test]
fn global_bootstrap_commands_are_actionable_before_workspace_init() {
    let workspace = temp_empty_workspace("boundline-global-bootstrap");

    let status = run_boundline(&["status", "--workspace", workspace.to_string_lossy().as_ref()]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("no active session"), "{status_text}");
    assert!(status_text.contains("workspace_initialized: false"), "{status_text}");
    assert!(status_text.contains("next_command: boundline init --workspace"), "{status_text}");

    let doctor = run_boundline(&["doctor", "--workspace", workspace.to_string_lossy().as_ref()]);
    let doctor_text = terminal_text(&doctor);
    assert!(doctor_text.contains("workspace"), "{doctor_text}");
    assert!(doctor_text.contains("boundline init --workspace"), "{doctor_text}");

    let init = run_boundline_in(
        &workspace,
        &["init", "--non-interactive", "--assistant", "codex", "--force"],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("init: workspace initialized"), "{init_text}");
    assert!(init_text.contains("assistant_package_scope: repo-local"), "{init_text}");
    assert!(
        init_text.contains("boundline assistant install --host <host> --scope user"),
        "{init_text}"
    );

    let initialized_status = run_boundline_in(&workspace, &["status"]);
    let initialized_status_text = terminal_text(&initialized_status);
    assert_eq!(initialized_status.status.code(), Some(0), "{initialized_status_text}");
    assert!(
        initialized_status_text.contains("workspace_initialized: true"),
        "{initialized_status_text}"
    );
    assert!(initialized_status_text.contains("no active session"), "{initialized_status_text}");
    assert!(
        initialized_status_text.contains("next_command: boundline start"),
        "{initialized_status_text}"
    );
}

#[test]
fn global_continue_does_not_invent_a_session_before_init_or_start() {
    let workspace = temp_empty_workspace("boundline-global-continue");
    let output = run_boundline(&["continue", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("no active session"), "{text}");
    assert!(text.contains("source_of_truth: .boundline/session.json"), "{text}");
    assert!(text.contains("next_command: boundline init --workspace"), "{text}");
    assert!(!text.contains("latest_status:"), "{text}");
}
