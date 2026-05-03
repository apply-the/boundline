use std::process::Command;

use crate::workspace_fixture::terminal_text;

#[test]
fn doctor_install_reports_repair_needed_when_canon_is_missing() {
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["doctor", "--install"])
        .env("PATH", "")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    let text = terminal_text(&output);

    assert_ne!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("companion_state: repair_needed"), "{text}");
    assert!(text.contains("canon_companion: failed"), "{text}");
    assert!(text.contains("actions:"), "{text}");
}
