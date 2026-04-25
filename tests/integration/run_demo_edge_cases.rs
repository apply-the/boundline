use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("synod-run-demo-edge-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir.join(".synod").join("demo-workspace")
}

fn run_synod(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap()
}

fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn run_demo_rejects_an_unsafe_workspace_root() {
    let dir = std::env::temp_dir().join(format!("synod-not-safe-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("not-demo-workspace");
    let output = run_synod(&["run-demo", "--workspace", bad.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);
    assert_ne!(output.status.code(), Some(0), "expected non-zero exit, got: {text}");
    assert!(text.to_lowercase().contains("demo-workspace"), "{text}");
}

#[test]
fn run_demo_recovers_a_pre_existing_workspace_via_reset() {
    let root = temp_root();
    // Pre-populate with junk to make sure `reset_demo_workspace` removes it.
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("leftover.txt"), "from a previous run").unwrap();

    let output = run_synod(&["run-demo", "--workspace", root.to_string_lossy().as_ref()]);
    assert_eq!(output.status.code(), Some(0), "{}", terminal_text(&output));
    assert!(!root.join("leftover.txt").exists(), "reset did not remove prior contents");
    assert!(root.join("src").join("buggy.rs").exists(), "buggy.rs missing after run");
}
