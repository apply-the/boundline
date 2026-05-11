use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::workspace_fixture::{run_boundline_in, run_boundline_in_with_env, terminal_text};

fn empty_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"boundline-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    workspace
}

fn run_init_in(workspace: &Path, args: &[&str]) -> std::process::Output {
    assert_eq!(args.first(), Some(&"init"));
    let mut command = Vec::with_capacity(args.len() + 1);
    command.push("init");
    command.push("--non-interactive");
    command.extend_from_slice(&args[1..]);
    run_boundline_in(workspace, &command)
}

fn run_init_in_with_env(
    workspace: &Path,
    args: &[&str],
    env: &[(&str, &str)],
) -> std::process::Output {
    assert_eq!(args.first(), Some(&"init"));
    let mut command = Vec::with_capacity(args.len() + 1);
    command.push("init");
    command.push("--non-interactive");
    command.extend_from_slice(&args[1..]);
    run_boundline_in_with_env(workspace, &command, env)
}

#[test]
fn init_scaffolds_execution_and_config_files() {
    let workspace = empty_workspace("boundline-init-bootstrap");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--template",
            "bug-fix",
            "--assistant",
            "copilot",
        ],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("init: workspace initialized"), "{init_text}");
    assert!(init_text.contains("route_setup:"), "{init_text}");
    assert!(init_text.contains("assistant_setup:"), "{init_text}");
    assert!(init_text.contains("next_steps:"), "{init_text}");
    assert!(
        init_text.contains("inspect effective config: boundline config show --workspace"),
        "{init_text}"
    );

    assert!(workspace.join(".boundline/execution.json").is_file());
    assert!(workspace.join(".boundline/config.toml").is_file());
    assert!(workspace.join("assistant/README.md").is_file());
    assert!(workspace.join("assistant/copilot/prompts/boundline-start.prompt.md").is_file());

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("assistant_runtimes"));
    assert!(config.contains("copilot"));
    assert!(config.contains("domain_templates"));
    assert!(config.contains("systems"));
}

#[test]
fn init_supports_relative_workspace_dot_without_existing_boundline() {
    let workspace = empty_workspace("boundline-init-relative-workspace");

    let init = run_init_in(&workspace, &["init", "--workspace", "."]);
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("init: workspace initialized"), "{init_text}");
    assert!(workspace.join(".boundline/execution.json").is_file(), "{init_text}");
    assert!(workspace.join(".boundline/config.toml").is_file(), "{init_text}");
}

#[test]
fn init_docs_export_is_create_only_by_default_when_targets_exist() {
    let workspace = empty_workspace("boundline-init-docs-create-only");

    let initial = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
        ],
    );
    let initial_text = terminal_text(&initial);
    assert_eq!(initial.status.code(), Some(0), "{initial_text}");

    let canon_doc = workspace.join("docs/boundline/canon.md");
    fs::write(&canon_doc, "stale canon doc\n").unwrap();

    let blocked = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
        ],
    );
    let blocked_text = terminal_text(&blocked);

    assert_ne!(blocked.status.code(), Some(0), "{blocked_text}");
    assert!(blocked_text.contains("documentation export blocked"), "{blocked_text}");
    assert!(blocked_text.contains("docs_export_root: docs/boundline"), "{blocked_text}");
    assert!(blocked_text.contains("--refresh"), "{blocked_text}");
    assert!(blocked_text.contains("--diff"), "{blocked_text}");
    assert!(blocked_text.contains("--to <path>"), "{blocked_text}");
    assert_eq!(fs::read_to_string(&canon_doc).unwrap(), "stale canon doc\n");
}

#[test]
fn init_docs_export_diff_reports_changes_without_writing() {
    let workspace = empty_workspace("boundline-init-docs-diff");

    let initial = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
        ],
    );
    let initial_text = terminal_text(&initial);
    assert_eq!(initial.status.code(), Some(0), "{initial_text}");

    let canon_doc = workspace.join("docs/boundline/canon.md");
    fs::write(&canon_doc, "stale canon doc\n").unwrap();

    let diff = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
            "--diff",
        ],
    );
    let diff_text = terminal_text(&diff);

    assert_eq!(diff.status.code(), Some(0), "{diff_text}");
    assert!(diff_text.contains("documentation export diff"), "{diff_text}");
    assert!(diff_text.contains("update docs/boundline/canon.md"), "{diff_text}");
    assert_eq!(fs::read_to_string(&canon_doc).unwrap(), "stale canon doc\n");
}

#[test]
fn init_docs_export_refresh_updates_existing_docs_without_force() {
    let workspace = empty_workspace("boundline-init-docs-refresh");

    let initial = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
        ],
    );
    let initial_text = terminal_text(&initial);
    assert_eq!(initial.status.code(), Some(0), "{initial_text}");

    let canon_doc = workspace.join("docs/boundline/canon.md");
    fs::write(&canon_doc, "stale canon doc\n").unwrap();

    let refresh = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
            "--refresh",
        ],
    );
    let refresh_text = terminal_text(&refresh);

    assert_eq!(refresh.status.code(), Some(0), "{refresh_text}");
    assert!(refresh_text.contains("docs_export:"), "{refresh_text}");
    assert!(
        refresh_text.contains("Canon reference docs: 0 created, 1 updated, 0 unchanged"),
        "{refresh_text}"
    );
    assert_ne!(fs::read_to_string(&canon_doc).unwrap(), "stale canon doc\n");
}

#[test]
fn init_docs_export_to_custom_root_writes_under_requested_directory() {
    let workspace = empty_workspace("boundline-init-docs-custom-root");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
            "--to",
            "docs/reference/boundline",
        ],
    );
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("docs_export:"), "{init_text}");
    assert!(init_text.contains("root: docs/reference/boundline"), "{init_text}");
    assert!(workspace.join("docs/reference/boundline/canon.md").is_file());
    assert!(workspace.join("docs/reference/boundline/assistant/README.md").is_file());
    assert!(!workspace.join("docs/boundline/canon.md").exists());
}

#[test]
fn init_docs_export_to_custom_root_works_after_workspace_is_initialized() {
    let workspace = empty_workspace("boundline-init-docs-custom-root-rerun");

    let initial = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
        ],
    );
    let initial_text = terminal_text(&initial);
    assert_eq!(initial.status.code(), Some(0), "{initial_text}");

    let rerun = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--export-docs",
            "--to",
            "docs/reference/boundline",
        ],
    );
    let rerun_text = terminal_text(&rerun);

    assert_eq!(rerun.status.code(), Some(0), "{rerun_text}");
    assert!(rerun_text.contains("root: docs/reference/boundline"), "{rerun_text}");
    assert!(workspace.join("docs/reference/boundline/canon.md").is_file());
    assert!(workspace.join("docs/reference/boundline/assistant/README.md").is_file());
}

#[test]
fn init_previews_existing_assistant_assets_without_force() {
    let workspace = empty_workspace("boundline-init-assistant-preview");
    fs::create_dir_all(workspace.join("assistant/copilot/prompts")).unwrap();
    fs::write(
        workspace.join("assistant/copilot/prompts/boundline-start.prompt.md"),
        "outdated command pack",
    )
    .unwrap();

    let init = run_init_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    let init_text = terminal_text(&init);

    assert_ne!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("preview only"), "{init_text}");
    assert!(init_text.contains("planned_changes:"), "{init_text}");
    assert!(init_text.contains("next_steps:"), "{init_text}");
    assert!(init_text.contains("refresh Copilot prompt pack"), "{init_text}");
}

#[test]
fn init_auto_seeds_routes_from_selected_assistant() {
    let workspace = empty_workspace("boundline-init-assistant-defaults");

    let init = run_init_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("route_setup:"), "{init_text}");
    assert!(init_text.contains("assistant_defaults: copilot"), "{init_text}");
    assert!(
        init_text.contains("seeded planning: copilot:gpt-5.5 [assistant-default]"),
        "{init_text}"
    );

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("assistant_runtimes = [\"copilot\"]"), "{config}");
    assert!(config.contains("[routing.planning]"), "{config}");
    assert!(config.contains("runtime = \"copilot\""), "{config}");
    assert!(config.contains("model = \"gpt-5.5\""), "{config}");
}

#[test]
fn init_falls_back_to_available_selected_assistant_when_preferred_runtime_is_unavailable() {
    let workspace = empty_workspace("boundline-init-assistant-fallback");

    let init = run_init_in_with_env(
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
    assert!(
        init_text.contains(
            "seeded planning: copilot:gpt-5.5 [assistant-default fallback-from=codex-unavailable]"
        ),
        "{init_text}"
    );

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("assistant_runtimes"), "{config}");
    assert!(config.contains("\"codex\""), "{config}");
    assert!(config.contains("\"copilot\""), "{config}");
    assert!(config.contains("[routing.planning]"), "{config}");
    assert!(config.contains("runtime = \"copilot\""), "{config}");
}

#[test]
fn init_stops_when_selected_assistant_defaults_are_unavailable() {
    let workspace = empty_workspace("boundline-init-assistant-unavailable");

    let init = run_init_in_with_env(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "codex"],
        &[("PATH", "/usr/bin:/bin")],
    );
    let init_text = terminal_text(&init);

    assert_ne!(init.status.code(), Some(0), "{init_text}");
    assert!(
        init_text.contains("init error: no available assistant defaults remain"),
        "{init_text}"
    );
    assert!(init_text.contains("--route planning=copilot:gpt-5.5"), "{init_text}");
}

#[test]
fn init_keeps_explicit_route_and_seeds_remaining_slots() {
    let workspace = empty_workspace("boundline-init-assistant-partial");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--route",
            "planning=copilot:gpt-4o",
        ],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("route_setup:"), "{init_text}");
    assert!(init_text.contains("explicit planning: copilot:gpt-4o [explicit]"), "{init_text}");
    assert!(
        init_text.contains("seeded verification: copilot:gpt-5.5 [assistant-default]"),
        "{init_text}"
    );
    assert!(
        init_text.contains("inspect_or_edit: boundline config show --workspace"),
        "{init_text}"
    );

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("model = \"gpt-4o\""), "{config}");
    assert!(config.contains("model = \"gpt-5.5\""), "{config}");
}

#[test]
fn init_reports_when_no_workspace_local_routes_are_recorded() {
    let workspace = empty_workspace("boundline-init-no-local-routes");

    let init =
        run_init_in(&workspace, &["init", "--workspace", workspace.to_string_lossy().as_ref()]);
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("route_setup:"), "{init_text}");
    assert!(
        init_text.contains(
            "workspace-local routes: none recorded; add --assistant or --route later to pin workspace-specific defaults"
        ),
        "{init_text}"
    );
    assert!(init_text.contains("next_steps:"), "{init_text}");
}

#[test]
fn init_rejects_malformed_route_with_actionable_example_and_no_mutation() {
    let workspace = empty_workspace("boundline-init-malformed-route");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--assistant",
            "copilot",
            "--route",
            "planning-copilot-gpt-4o",
        ],
    );
    let init_text = terminal_text(&init);

    assert_ne!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("init error:"), "{init_text}");
    assert!(init_text.contains("SLOT=RUNTIME:MODEL"), "{init_text}");
    assert!(init_text.contains("planning=copilot:gpt-5.5"), "{init_text}");
    assert!(!workspace.join(".boundline/config.toml").exists(), "{init_text}");
    assert!(!workspace.join(".boundline/execution.json").exists(), "{init_text}");
}

#[test]
fn init_requires_non_interactive_flag_when_guided_values_need_a_tty() {
    let workspace = empty_workspace("boundline-init-no-tty-guidance");

    let init = run_boundline_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    let init_text = terminal_text(&init);

    assert_ne!(init.status.code(), Some(0), "{init_text}");
    assert!(
        init_text.contains(
            "Terminal interaction is unavailable. Rerun with --non-interactive and explicit flags."
        ),
        "{init_text}"
    );
}

#[test]
fn init_writes_canon_preferences_when_flags_are_supplied() {
    let workspace = empty_workspace("boundline-init-canon");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--canon-mode-selection",
            "auto-confirm",
            "--risk",
            "medium",
            "--zone",
            "engineering",
            "--owner",
            "platform",
        ],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("canon_mode_selection: auto-confirm"), "{init_text}");

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("[canon]"), "{config}");
    assert!(config.contains("mode_selection = \"auto-confirm\""), "{config}");
    assert!(config.contains("default_risk = \"medium\""), "{config}");
    assert!(config.contains("default_zone = \"engineering\""), "{config}");
    assert!(config.contains("default_owner = \"platform\""), "{config}");
}

#[test]
fn init_writes_canon_preferences_and_model_routes_when_flags_are_supplied() {
    let workspace = empty_workspace("boundline-init-canon-routes");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--canon-mode-selection",
            "auto-confirm",
            "--assistant",
            "copilot",
            "--route",
            "planning=copilot:gpt-4o",
            "--route",
            "implementation=codex:gpt-5-codex",
        ],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("[canon]"), "{config}");
    assert!(config.contains("mode_selection = \"auto-confirm\""), "{config}");
    assert!(config.contains("assistant_runtimes = [\"copilot\"]"), "{config}");
    assert!(config.contains("[routing.planning]"), "{config}");
    assert!(config.contains("runtime = \"copilot\""), "{config}");
    assert!(config.contains("model = \"gpt-4o\""), "{config}");
    assert!(config.contains("[routing.implementation]"), "{config}");
    assert!(config.contains("runtime = \"codex\""), "{config}");
    assert!(config.contains("model = \"gpt-5-codex\""), "{config}");
}

#[test]
fn init_seeds_explicit_domain_templates_and_bindings() {
    let workspace = empty_workspace("boundline-init-bootstrap-domain");

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--domain",
            "react",
            "--domain-standard",
            "react=follow the shared ui system",
            "--context-binding",
            "react|design_system|mcp:design-system",
            "--required-context-binding",
            "react|design_reference|design/reference.md",
            "--force",
        ],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("domain_templates:"), "{init_text}");
    assert!(init_text.contains("- react: enabled=true"), "{init_text}");

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("react"));
    assert!(config.contains("follow the shared ui system"));
    assert!(config.contains("mcp:design-system"));
    assert!(config.contains("design/reference.md"));
}

#[test]
fn init_seeds_domain_hygiene_defaults_without_overwriting_custom_rules() {
    let workspace = empty_workspace("boundline-init-hygiene");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    fs::write(workspace.join("package.json"), "{\"scripts\":{\"build\":\"vite build\"}}\n")
        .unwrap();
    fs::write(workspace.join("Dockerfile"), "FROM node:22\n").unwrap();
    fs::write(workspace.join(".gitignore"), "custom-local-cache/\n").unwrap();

    let init = run_init_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--domain",
            "react",
            "--assistant",
            "copilot",
        ],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("workspace_hygiene:"), "{init_text}");
    assert!(init_text.contains(".gitignore: updated"), "{init_text}");
    assert!(init_text.contains(".dockerignore: created"), "{init_text}");

    let gitignore = fs::read_to_string(workspace.join(".gitignore")).unwrap();
    assert!(gitignore.contains("custom-local-cache/"), "{gitignore}");
    assert!(gitignore.contains(".boundline/traces/"), "{gitignore}");
    assert!(gitignore.contains("node_modules/"), "{gitignore}");
    assert!(gitignore.contains("dist/"), "{gitignore}");
    assert!(!gitignore.contains("target/"), "{gitignore}");

    let dockerignore = fs::read_to_string(workspace.join(".dockerignore")).unwrap();
    assert!(dockerignore.contains(".git"), "{dockerignore}");
    assert!(dockerignore.contains("node_modules/"), "{dockerignore}");
}

#[test]
fn init_creates_legacy_eslintignore_when_legacy_cues_are_present() {
    let workspace = empty_workspace("boundline-init-eslint-hygiene");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    fs::write(workspace.join(".eslintrc.json"), "{}\n").unwrap();

    let init = run_init_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains(".eslintignore: created"), "{init_text}");

    let eslintignore = fs::read_to_string(workspace.join(".eslintignore")).unwrap();
    assert!(eslintignore.contains(".boundline/traces/"), "{eslintignore}");
    assert!(eslintignore.contains("dist/"), "{eslintignore}");
}

#[test]
fn init_adds_kubernetes_related_gitignore_defaults_when_cues_are_present() {
    let workspace = empty_workspace("boundline-init-kubernetes-hygiene");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    fs::write(workspace.join("kustomization.yaml"), "resources: []\n").unwrap();

    let init = run_init_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    let init_text = terminal_text(&init);

    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains(".gitignore: created"), "{init_text}");
    assert!(init_text.contains("tool:kubernetes"), "{init_text}");

    let gitignore = fs::read_to_string(workspace.join(".gitignore")).unwrap();
    assert!(gitignore.contains(".kube/"), "{gitignore}");
    assert!(gitignore.contains("*.secret.yaml"), "{gitignore}");
}

#[test]
fn init_uses_only_universal_hygiene_when_no_stack_is_credible() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-init-hygiene-empty-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".git")).unwrap();

    let init = run_init_in(
        &workspace,
        &["init", "--workspace", workspace.to_string_lossy().as_ref(), "--assistant", "copilot"],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains(".gitignore: created"), "{init_text}");
    assert!(!workspace.join(".dockerignore").exists());

    let gitignore = fs::read_to_string(workspace.join(".gitignore")).unwrap();
    assert!(gitignore.contains(".boundline/traces/"), "{gitignore}");
    assert!(!gitignore.contains("node_modules/"), "{gitignore}");
    assert!(!gitignore.contains("__pycache__/"), "{gitignore}");
}
