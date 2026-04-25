use std::env;
use std::fs;
use std::path::PathBuf;

use synod::demo::workspace::{
    BUG_MARKER, DemoWorkspaceError, reset_demo_workspace, seed_demo_workspace,
};

fn temp_root(suffix: &str) -> PathBuf {
    let mut base = env::temp_dir();
    base.push(format!(
        "synod-demo-ext-{}-{}-{}",
        suffix,
        std::process::id(),
        synod::domain::trace::current_timestamp_millis()
    ));
    base.push(".synod");
    base.push("demo-workspace");
    base
}

#[test]
fn seed_demo_workspace_creates_seeded_files_with_marker() {
    let root = temp_root("seed");
    let ws = seed_demo_workspace(&root).expect("seed succeeds");
    let body = fs::read_to_string(&ws.target_file).unwrap();
    assert!(body.contains(BUG_MARKER));
    assert!(ws.test_file.exists());
    let _ = fs::remove_dir_all(root.parent().unwrap());
}

#[test]
fn reset_demo_workspace_restores_buggy_state() {
    let root = temp_root("reset");
    let ws = seed_demo_workspace(&root).expect("seed");
    fs::write(&ws.target_file, "fixed!\n").unwrap();
    let ws2 = reset_demo_workspace(&root).expect("reset");
    assert!(fs::read_to_string(&ws2.target_file).unwrap().contains(BUG_MARKER));
    let _ = fs::remove_dir_all(root.parent().unwrap());
}

#[test]
fn rejects_unsafe_root() {
    let mut bad = env::temp_dir();
    bad.push("synod-bad");
    bad.push("not-demo-workspace");
    assert!(matches!(seed_demo_workspace(&bad), Err(DemoWorkspaceError::UnsafeRoot(_))));
}
