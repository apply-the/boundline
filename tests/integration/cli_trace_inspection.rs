use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-cli-inspect-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.5.0\"\nedition = \"2024\"\n",
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

fn extract_trace_path(text: &str) -> PathBuf {
    text.split_whitespace()
        .find_map(|token| {
            let cleaned = token.trim_matches(|ch: char| ch == '"' || ch == ',' || ch == ':');
            if cleaned.ends_with(".json") { Some(PathBuf::from(cleaned)) } else { None }
        })
        .expect(text)
}

#[test]
fn inspect_command_reconstructs_step_order_and_recovery_from_a_successful_trace() {
    let workspace = temp_workspace();
    let demo_output = run_synod(&["demo", "--workspace", workspace.to_string_lossy().as_ref()]);
    let trace_path = extract_trace_path(&terminal_text(&demo_output));
    let inspect_output = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{text}");
    assert!(text.contains("analyze"), "{text}");
    assert!(text.contains("code"), "{text}");
    assert!(text.contains("verify"), "{text}");
    assert!(text.contains("retry") || text.contains("replan"), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
}

#[test]
fn inspect_command_highlights_non_success_terminal_reasons() {
    let workspace = temp_workspace();
    let run_output = run_synod(&[
        "run",
        "--goal",
        "Force a non-success failure for the default developer flow",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let trace_path = extract_trace_path(&terminal_text(&run_output));
    let inspect_output = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{text}");
    assert!(text.contains("terminal_reason"), "{text}");
    assert!(text.contains("failed") || text.contains("exhausted"), "{text}");
}
