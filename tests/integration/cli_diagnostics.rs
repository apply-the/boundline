use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-cli-doctor-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    workspace
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
fn doctor_reports_a_ready_workspace_and_actionable_checks() {
    let workspace = temp_workspace();
    let output = run_synod(&["doctor", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("ready"), "{text}");
    assert!(text.contains("built_in_flow"), "{text}");
    assert!(text.contains("trace_store"), "{text}");
}
