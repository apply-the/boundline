use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use crate::workspace_fixture::{run_synod_in, terminal_text};

fn empty_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    workspace
}

#[test]
fn init_scaffolds_execution_and_config_files() {
    let workspace = empty_workspace("synod-init-bootstrap");

    let init = run_synod_in(
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

    assert!(workspace.join(".synod/execution.json").is_file());
    assert!(workspace.join(".synod/config.toml").is_file());

    let config = fs::read_to_string(workspace.join(".synod/config.toml")).unwrap();
    assert!(config.contains("assistant_runtimes"));
    assert!(config.contains("copilot"));
}
