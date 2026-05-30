use crate::workspace_fixture::{
    run_boundline, run_boundline_in, temp_empty_workspace, temp_git_workspace, terminal_text,
};

#[test]
fn global_bootstrap_commands_are_actionable_before_workspace_init() {
    let workspace = temp_git_workspace("boundline-global-bootstrap");

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
        &["init", "--non-interactive", "--workspace", ".", "--assistant", "copilot", "--force"],
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
        initialized_status_text.contains("next_command: boundline session list"),
        "{initialized_status_text}"
    );
    assert!(
        initialized_status_text.contains("repair_command: boundline goal"),
        "{initialized_status_text}"
    );
}

#[test]
fn global_continue_does_not_invent_a_session_before_init_or_goal() {
    let workspace = temp_empty_workspace("boundline-global-continue");
    let output = run_boundline(&["continue", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("no active session"), "{text}");
    assert!(
        text.contains("source_of_truth: .boundline/active-session -> .boundline/sessions/<session_ref>/session.json"),
        "{text}"
    );
    assert!(text.contains("next_command: boundline init --workspace"), "{text}");
    assert!(text.contains("repair_command: boundline doctor --workspace"), "{text}");
    assert!(!text.contains("latest_status:"), "{text}");
}

#[test]
fn doctor_workspace_output_surfaces_contextual_s7_gaps_before_init() {
    let workspace = temp_empty_workspace("boundline-global-doctor-context");
    let output = run_boundline(&["doctor", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(2), "{text}");
    assert!(text.contains("boundline_config"), "{text}");
    assert!(text.contains("provider_readiness"), "{text}");
    assert!(text.contains("advanced_context_index"), "{text}");
    assert!(text.contains("session_evidence"), "{text}");
    assert!(text.contains("Canon project memory"), "{text}");
    assert!(
        text.contains("boundline config show --workspace")
            || text.contains("boundline init --workspace"),
        "{text}"
    );
    assert!(
        text.contains(
            "boundline config set-semantic-acceleration --scope workspace --policy local"
        ),
        "{text}"
    );
    assert!(text.contains("boundline goal --workspace"), "{text}");
}
