//! Contract tests for calibration output.
//!
//! Validates human-readable and JSON output formats for control level
//! assignments, override records, and council adjudication results.

use std::fs;
use std::process::Command;

#[test]
fn council_without_calibration_policy_defaults_all_advisory() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["council", "adjudicate"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(text.contains("Control Level Assignments"));
    assert!(text.contains("built-in all-advisory default"));
    assert!(text.contains("Advisory"));
}

#[test]
fn council_with_json_output_includes_calibration_fields() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["council", "adjudicate", "--json"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(text.contains("\"calibration\""));
    assert!(text.contains("\"control_levels\""));
}

#[test]
fn override_rejects_hook_level() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args([
            "override",
            "--workspace",
            dir.to_str().unwrap(),
            "--guardian-id",
            "test",
            "--control-id",
            "ctrl-1",
            "--level",
            "hook",
            "--reason",
            "testing",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn override_writes_record() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let boundline_dir = &dir.join(".boundline");
    fs::create_dir_all(boundline_dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args([
            "override",
            "--workspace",
            dir.to_str().unwrap(),
            "--guardian-id",
            "rust-guardian",
            "--control-id",
            "ctrl-1",
            "--level",
            "catch",
            "--reason",
            "false positive test",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(boundline_dir.join("overrides.json").exists());
}
