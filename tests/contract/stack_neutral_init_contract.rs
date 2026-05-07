use std::fs;

use crate::workspace_fixture::{
    run_boundline_in, run_boundline_in_with_env, temp_empty_workspace, terminal_text,
};

#[test]
fn doctor_workspace_output_stays_stack_neutral() {
    let workspace = temp_empty_workspace("boundline-stack-neutral-contract");

    let init = run_boundline_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let doctor = run_boundline_in(
        &workspace,
        &["doctor", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let doctor_text = terminal_text(&doctor);

    assert!(doctor_text.contains("workspace_exists: passed"), "{doctor_text}");
    assert!(doctor_text.contains("workspace_writable: passed"), "{doctor_text}");
    assert!(doctor_text.contains("workspace_execution_profile: passed"), "{doctor_text}");
    assert!(!doctor_text.contains("Cargo.toml"), "{doctor_text}");
}

#[test]
fn init_reports_seeded_routes_and_hygiene_actions() {
    let workspace = temp_empty_workspace("boundline-stack-neutral-init-contract");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    fs::write(workspace.join("package.json"), "{}\n").unwrap();
    fs::write(workspace.join("Dockerfile"), "FROM node:22\n").unwrap();

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "codex",
            "--domain",
            "react",
        ],
    );
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("seeded_route_defaults:"), "{init_text}");
    assert!(init_text.contains("workspace_hygiene:"), "{init_text}");
    assert!(init_text.contains(".gitignore: created"), "{init_text}");
    assert!(init_text.contains(".dockerignore: created"), "{init_text}");
    assert!(fs::read_to_string(workspace.join(".gitignore")).unwrap().contains("node_modules/"));
}

#[test]
fn init_reports_assistant_fallback_when_selected_runtime_is_unavailable() {
    let workspace = temp_empty_workspace("boundline-stack-neutral-init-fallback-contract");

    let init = run_boundline_in_with_env(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "codex",
            "--assistant",
            "copilot",
        ],
        &[("PATH", "/usr/bin:/bin")],
    );
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("- codex: missing from PATH or extension surface"), "{init_text}");
    assert!(
        init_text.contains(
            "planning: copilot:gpt-5.4 [assistant-default fallback-from=codex-unavailable]"
        ),
        "{init_text}"
    );
}
