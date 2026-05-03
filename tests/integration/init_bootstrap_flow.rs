use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use crate::workspace_fixture::{run_boundline_in, terminal_text};

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

#[test]
fn init_scaffolds_execution_and_config_files() {
    let workspace = empty_workspace("boundline-init-bootstrap");

    let init = run_boundline_in(
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

    assert!(workspace.join(".boundline/execution.json").is_file());
    assert!(workspace.join(".boundline/config.toml").is_file());

    let config = fs::read_to_string(workspace.join(".boundline/config.toml")).unwrap();
    assert!(config.contains("assistant_runtimes"));
    assert!(config.contains("copilot"));
    assert!(config.contains("domain_templates"));
    assert!(config.contains("systems"));
}

#[test]
fn init_seeds_explicit_domain_templates_and_bindings() {
    let workspace = empty_workspace("boundline-init-bootstrap-domain");

    let init = run_boundline_in(
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
