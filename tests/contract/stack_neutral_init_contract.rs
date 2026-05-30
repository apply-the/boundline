use std::fs;

use crate::workspace_fixture::{
    run_boundline_in, run_boundline_in_with_env, temp_git_workspace, terminal_text,
};

#[test]
fn doctor_workspace_output_stays_stack_neutral() {
    let workspace = temp_git_workspace("boundline-stack-neutral-contract");

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

    assert!(doctor_text.contains("workspace_exists: passed"), "{doctor_text}");
    assert!(doctor_text.contains("workspace_writable: passed"), "{doctor_text}");
    assert!(doctor_text.contains("workspace_execution_profile: passed"), "{doctor_text}");
    assert!(!doctor_text.contains("Cargo.toml"), "{doctor_text}");
}

#[test]
fn init_reports_seeded_routes_and_hygiene_actions() {
    let workspace = temp_git_workspace("boundline-stack-neutral-init-contract");
    fs::write(workspace.join("package.json"), "{}\n").unwrap();
    fs::write(workspace.join("Dockerfile"), "FROM node:22\n").unwrap();

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--domain",
            "react",
        ],
    );
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("route_setup:"), "{init_text}");
    assert!(
        init_text.contains("seeded planning: copilot:gpt-4.1 [assistant-default]"),
        "{init_text}"
    );
    assert!(init_text.contains("workspace_hygiene:"), "{init_text}");
    assert!(init_text.contains(".gitignore: created"), "{init_text}");
    assert!(init_text.contains(".dockerignore: created"), "{init_text}");
    assert!(fs::read_to_string(workspace.join(".gitignore")).unwrap().contains("node_modules/"));
}

#[test]
fn init_tracks_derived_index_manifest_and_wal_shm_hygiene() {
    let workspace = temp_git_workspace("boundline-stack-neutral-derived-index-hygiene");

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
    let init_text = terminal_text(&init);
    let gitignore = fs::read_to_string(workspace.join(".gitignore")).unwrap();

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(
        init_text.contains(
            "derived_index_hygiene: disposable retrieval DB, manifest, and SQLite WAL/SHM sidecars stay ignored"
        ),
        "{init_text}"
    );
    assert!(
        gitignore.contains(".boundline/context-intelligence/retrieval-index.sqlite3"),
        "{gitignore}"
    );
    assert!(gitignore.contains(".boundline/context-intelligence/manifest.json"), "{gitignore}");
    assert!(
        gitignore.contains(".boundline/context-intelligence/retrieval-index.sqlite3-wal"),
        "{gitignore}"
    );
    assert!(
        gitignore.contains(".boundline/context-intelligence/retrieval-index.sqlite3-shm"),
        "{gitignore}"
    );
}

#[test]
fn init_can_install_mark_stale_semantic_index_hooks() {
    let workspace = temp_git_workspace("boundline-stack-neutral-hook-init");

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--semantic-index-hook-action",
            "mark-stale",
        ],
    );
    let init_text = terminal_text(&init);
    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(config.contains("policy = \"local\""), "{config}");
    assert!(config.contains("index_hook_action = \"mark_stale\""), "{config}");
    assert!(workspace.join(".git/hooks/post-checkout").is_file());
    assert!(workspace.join(".git/hooks/post-merge").is_file());
    assert!(workspace.join(".git/hooks/post-rewrite").is_file());
}

#[test]
fn init_reports_assistant_fallback_when_selected_runtime_is_unavailable() {
    let workspace = temp_git_workspace("boundline-stack-neutral-init-fallback-contract");

    let init = run_boundline_in_with_env(
        &workspace,
        &[
            "init",
            "--non-interactive",
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
            "seeded planning: copilot:gpt-4.1 [assistant-default fallback-from=codex-unavailable]"
        ),
        "{init_text}"
    );
}
