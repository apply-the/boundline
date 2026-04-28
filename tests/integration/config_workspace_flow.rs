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
fn config_set_show_and_unset_workspace_slot() {
    let workspace = empty_workspace("synod-config-workspace");

    let init = run_synod_in(
        &workspace,
        &[
            "init",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--template",
            "change",
            "--assistant",
            "copilot",
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set = run_synod_in(
        &workspace,
        &[
            "config",
            "set",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "planning",
            "--runtime",
            "codex",
            "--model",
            "gpt-5-codex",
        ],
    );
    let set_text = terminal_text(&set);
    assert_eq!(set.status.code(), Some(0), "{set_text}");
    assert!(set_text.contains("config: updated workspace config"), "{set_text}");

    let show = run_synod_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
        ],
    );
    let show_text = terminal_text(&show);
    assert_eq!(show.status.code(), Some(0), "{show_text}");
    assert!(show_text.contains("planning: codex:gpt-5-codex"), "{show_text}");

    let unset = run_synod_in(
        &workspace,
        &[
            "config",
            "unset",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "planning",
        ],
    );
    let unset_text = terminal_text(&unset);
    assert_eq!(unset.status.code(), Some(0), "{unset_text}");

    let show_after = run_synod_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
        ],
    );
    let show_after_text = terminal_text(&show_after);
    assert_eq!(show_after.status.code(), Some(0), "{show_after_text}");
    assert!(show_after_text.contains("planning: <unset>"), "{show_after_text}");
}
