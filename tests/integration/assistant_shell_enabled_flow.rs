use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace =
        std::env::temp_dir().join(format!("synod-assistant-shell-enabled-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.3.0\"\nedition = \"2024\"\n",
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

fn extract_trace_path(text: &str) -> Option<PathBuf> {
    text.split_whitespace().find_map(|token| {
        let cleaned = token.trim_matches(|ch: char| ch == '"' || ch == ',' || ch == ':');
        if cleaned.ends_with(".json") { Some(PathBuf::from(cleaned)) } else { None }
    })
}

#[test]
fn shell_enabled_run_and_status_outputs_include_assistant_routing_cues() {
    let workspace = temp_workspace();
    let run_output = run_synod(&[
        "run",
        "--goal",
        "Summarize the current bounded developer flow",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("terminal_status: succeeded"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");
    assert!(run_text.contains("next_command: /synod-status"), "{run_text}");

    let inspect_output =
        run_synod(&["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(inspect_text.contains("trace:"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_status_surfaces_non_success_outcomes_for_next_step_routing() {
    let workspace = temp_workspace();
    let run_output = run_synod(&[
        "run",
        "--goal",
        "Force a non-success failure for the default developer flow",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_reason:"), "{run_text}");
    assert!(run_text.contains("next_command: /synod-next"), "{run_text}");

    let inspect_output =
        run_synod(&["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("terminal_reason:"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_explicit_inspect_outputs_selection_mode_and_next_step_cues() {
    let workspace = temp_workspace();
    let demo_output = run_synod(&["demo", "--workspace", workspace.to_string_lossy().as_ref()]);
    let demo_text = terminal_text(&demo_output);
    let trace_path = extract_trace_path(&demo_text).expect(&demo_text);

    let inspect_output = run_synod(&["inspect", "--trace", trace_path.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: explicit-trace"), "{inspect_text}");
    assert!(inspect_text.contains(&format!("trace: {}", trace_path.display())), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
}

#[test]
fn shell_enabled_inspect_trace_read_failures_include_correction_cues() {
    let missing_trace =
        std::env::temp_dir().join(format!("synod-missing-trace-{}.json", Uuid::new_v4()));
    let inspect_output =
        run_synod(&["inspect", "--trace", missing_trace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(3), "{inspect_text}");
    assert!(inspect_text.contains("inspect: trace read failure"), "{inspect_text}");
    assert!(
        inspect_text.contains("terminal_reason: failed to read the requested trace"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("next_command: /synod-inspect"), "{inspect_text}");
    assert!(
        inspect_text.contains("corrected_command: cargo run --bin synod -- inspect --trace"),
        "{inspect_text}"
    );
}
