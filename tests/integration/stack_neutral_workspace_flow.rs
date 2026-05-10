use crate::workspace_fixture::{
    run_boundline_in, temp_empty_workspace, temp_python_workspace, terminal_text,
};

#[test]
fn doctor_accepts_initialized_stack_neutral_workspace_without_cargo_manifest() {
    let workspace = temp_empty_workspace("boundline-stack-neutral-doctor");

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let doctor = run_boundline_in(
        &workspace,
        &["doctor", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let doctor_text = terminal_text(&doctor);

    assert_eq!(doctor.status.code(), Some(0), "{doctor_text}");
    assert!(!doctor_text.contains("Cargo.toml"), "{doctor_text}");
    assert!(!doctor_text.contains("repository_root"), "{doctor_text}");
    assert!(doctor_text.contains("doctor: ready for workspace"), "{doctor_text}");
}

#[test]
fn native_direct_run_does_not_fail_on_missing_cargo_manifest() {
    let workspace = temp_python_workspace("boundline-stack-neutral-run");

    let run = run_boundline_in(
        &workspace,
        &[
            "run",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--goal",
            "Bootstrap a bounded Python CLI package with a tested add command",
        ],
    );
    let run_text = terminal_text(&run);

    assert!(!run_text.contains("Cargo.toml"), "{run_text}");
    assert!(!run_text.contains("repository_root"), "{run_text}");
    assert!(
        run_text.contains("routing:")
            || run_text.contains("latest_status:")
            || run_text.contains("context_credibility:")
            || run_text.contains("goal_plan_state:"),
        "{run_text}"
    );
}
