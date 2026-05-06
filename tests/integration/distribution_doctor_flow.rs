use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use uuid::Uuid;

use crate::workspace_fixture::terminal_text;

const FULL_CAPABILITIES: &str = include_str!("../fixtures/canon_capabilities_full.json");
const MISSING_OPERATION_CAPABILITIES: &str =
    include_str!("../fixtures/canon_capabilities_missing_operation.json");
const MISSING_MODE_CAPABILITIES: &str =
    include_str!("../fixtures/canon_capabilities_missing_mode.json");

#[test]
fn doctor_install_reports_a_ready_install_when_canon_matches() {
    let canon_dir = fake_canon_directory("0.40.0");
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["doctor", "--install"])
        .env("PATH", &canon_dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("doctor: ready for installation"), "{text}");
    assert!(text.contains("canon_companion: passed"), "{text}");
    assert!(text.contains("canon_path: passed"), "{text}");
    assert!(text.contains("canon_governance_surface: passed"), "{text}");
    assert!(text.contains("canon_modes: passed"), "{text}");
    assert!(text.contains("companion_state: already_satisfied"), "{text}");
}

#[test]
fn doctor_install_reports_missing_canon_governance_operation() {
    let canon_dir =
        fake_canon_directory_with_capabilities("0.40.0", MISSING_OPERATION_CAPABILITIES);
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["doctor", "--install"])
        .env("PATH", &canon_dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    let text = terminal_text(&output);

    assert!(!output.status.success(), "{text}");
    assert!(text.contains("canon_governance_surface: failed"), "{text}");
    assert!(text.contains("start") && text.contains("refresh"), "{text}");
    assert!(text.contains("actions:"), "{text}");
}

#[test]
fn doctor_install_reports_missing_canon_mode() {
    let canon_dir = fake_canon_directory_with_capabilities("0.40.0", MISSING_MODE_CAPABILITIES);
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["doctor", "--install"])
        .env("PATH", &canon_dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    let text = terminal_text(&output);

    assert!(!output.status.success(), "{text}");
    assert!(text.contains("canon_modes: failed"), "{text}");
    assert!(text.contains("supply-chain-analysis"), "{text}");
    assert!(text.contains("actions:"), "{text}");
}

fn fake_canon_directory(version: &str) -> PathBuf {
    fake_canon_directory_with_capabilities(version, FULL_CAPABILITIES)
}

fn fake_canon_directory_with_capabilities(version: &str, capabilities: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("boundline-distribution-flow-{}", Uuid::new_v4()));
    fs::create_dir_all(&directory).unwrap();
    let canon = directory.join("canon");
    fs::write(
        &canon,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf 'canon version {version}\\n'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{}'\n  exit 0\nfi\nexit 1\n",
            capabilities
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&canon).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&canon, permissions).unwrap();
    directory
}
