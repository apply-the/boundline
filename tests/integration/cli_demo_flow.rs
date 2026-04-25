use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-cli-demo-{}", Uuid::new_v4()));
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

fn extract_trace_path(text: &str) -> Option<PathBuf> {
    text.split_whitespace().find_map(|token| {
        let cleaned = token.trim_matches(|ch: char| ch == '"' || ch == ',' || ch == ':');
        if cleaned.ends_with(".json") { Some(PathBuf::from(cleaned)) } else { None }
    })
}

#[test]
fn guided_demo_shows_progress_recovery_terminal_status_and_trace_location() {
    let workspace = temp_workspace();
    let output = run_synod(&["demo", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("analyze"), "{text}");
    assert!(text.contains("code"), "{text}");
    assert!(text.contains("verify"), "{text}");
    assert!(text.contains("retry") || text.contains("replan"), "{text}");
    assert!(text.contains("succeeded") || text.contains("terminal"), "{text}");
    assert!(trace_path.as_ref().is_some_and(|path| path.exists()), "{text}");
}
