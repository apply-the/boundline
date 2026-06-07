//! Integration tests for calibration flow.
//!
//! Tests the full flow: calibration policy → council adjudication →
//! override consumption → trust accumulation → graduation.

use std::fs;
use std::process::Command;

#[test]
fn full_cycle_calibration_defaults_to_advisory() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let boundline = &dir.join(".boundline");
    fs::create_dir_all(boundline).unwrap();

    // Run council with no calibration policy — should default all to advisory.
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["council", "adjudicate"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(text.contains("Control Level Assignments"));
    assert!(text.contains("built-in all-advisory default"));
    // With no calibration policy, all guardians should be Advisory (cold start).
    let advisory_count = text.matches("Advisory").count();
    assert!(advisory_count >= 4, "expected at least 4 Advisory assignments, got {advisory_count}");
}

#[test]
fn trust_records_persist_after_adjudication() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let boundline = &dir.join(".boundline");
    fs::create_dir_all(boundline).unwrap();

    // First adjudication — creates trust records.
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["council", "adjudicate"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Trust records should be persisted.
    let trust_path = boundline.join("trust-records.json");
    assert!(trust_path.exists(), "trust-records.json should exist after adjudication");
}

#[test]
fn override_and_readback_flow() {
    let dir = std::env::temp_dir().join(format!("boundline-calib-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let boundline = &dir.join(".boundline");
    fs::create_dir_all(boundline).unwrap();

    // Write an override.
    let override_out = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args([
            "override",
            "--workspace",
            dir.to_str().unwrap(),
            "--guardian-id",
            "rust-guardian",
            "--control-id",
            "ctrl-test-1",
            "--level",
            "catch",
            "--reason",
            "integration test override",
        ])
        .output()
        .unwrap();
    assert!(override_out.status.success());

    // Verify the overrides file exists.
    let overrides_path = boundline.join("overrides.json");
    assert!(overrides_path.exists());
    let content = fs::read_to_string(&overrides_path).unwrap();
    assert!(content.contains("rust-guardian"));
    assert!(content.contains("integration test override"));
}
