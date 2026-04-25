use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-cli-contract-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
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
fn doctor_uses_the_success_exit_code_for_a_ready_workspace() {
    let workspace = temp_workspace();
    let output = run_synod(&["doctor", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("ready"), "{text}");
    assert!(text.contains("workspace"), "{text}");
}

#[test]
fn demo_uses_the_success_exit_code_and_reports_trace_output() {
    let workspace = temp_workspace();
    let output = run_synod(&["demo", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("retry") || text.contains("replan"), "{text}");
    assert!(text.contains("trace"), "{text}");
}

#[test]
fn run_uses_the_success_exit_code_for_a_simple_bounded_goal() {
    let workspace = temp_workspace();
    let output = run_synod(&[
        "run",
        "--goal",
        "Summarize the current bounded developer flow",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("trace"), "{text}");
    assert!(text.contains("terminal_status"), "{text}");
}

#[test]
fn run_uses_the_non_success_exit_code_when_execution_stops_before_success() {
    let workspace = temp_workspace();
    let output = run_synod(&[
        "run",
        "--goal",
        "Force a non-success failure for the default developer flow",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
    assert!(text.contains("trace"), "{text}");
}
