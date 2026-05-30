use std::path::PathBuf;
use std::process::Command;

use crate::workspace_fixture::{target_test_cwd, target_test_dir, terminal_text};
use uuid::Uuid;

fn isolated_homebrew_prefix() -> PathBuf {
    target_test_dir(&format!("boundline-doctor-homebrew-disabled-{}", Uuid::new_v4()))
}

#[test]
fn doctor_install_reports_repair_needed_when_canon_is_missing() {
    let homebrew_prefix = isolated_homebrew_prefix();
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["doctor", "--install"])
        .env("PATH", "")
        .env("HOMEBREW_PREFIX", &homebrew_prefix)
        .current_dir(target_test_cwd("boundline-doctor-blocked-cwd"))
        .output()
        .unwrap();
    let text = terminal_text(&output);

    assert_ne!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("companion_state: repair_needed"), "{text}");
    assert!(text.contains("canon_companion: failed"), "{text}");
    assert!(text.contains("actions:"), "{text}");
}
