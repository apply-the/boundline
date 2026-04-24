use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use synod::adapters::trace_store::{FileTraceStore, TraceStore};
use synod::cli::inspect::summarize_trace;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-trace-summary-{}", Uuid::new_v4()));
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

fn extract_trace_path(text: &str) -> PathBuf {
    text.split_whitespace()
        .find_map(|token| {
            let cleaned = token.trim_matches(|ch: char| ch == '"' || ch == ',' || ch == ':');
            if cleaned.ends_with(".json") { Some(PathBuf::from(cleaned)) } else { None }
        })
        .expect(text)
}

#[test]
fn trace_summary_preserves_step_order_and_terminal_reason() {
    let workspace = temp_workspace();
    let output = run_synod(&["demo", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert_eq!(summary.trace_ref, trace_path.to_string_lossy());
    assert_eq!(summary.executed_steps[0].step_id, "analyze");
    assert_eq!(summary.executed_steps[1].step_id, "code");
    assert_eq!(summary.terminal_status, trace.terminal_status.unwrap());
    assert_eq!(summary.terminal_reason, trace.terminal_reason.unwrap());
}

#[test]
fn trace_summary_keeps_recovery_events_explicit() {
    let workspace = temp_workspace();
    let output = run_synod(&["demo", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);
    let store = FileTraceStore::for_workspace(&workspace);
    let trace = store.load(&trace_path).unwrap();
    let summary = summarize_trace(&trace_path, &trace).unwrap();

    assert!(!summary.recovery_events.is_empty());
    assert!(summary.recovery_events.iter().all(|event| !event.trigger.trim().is_empty()));
    assert!(summary.duration.is_some());
}
