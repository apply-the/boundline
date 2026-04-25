use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("synod-run-demo-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir.join(".synod").join("demo-workspace")
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
fn run_demo_drives_a_failing_test_to_passing_with_retry_and_replan() {
    let root = temp_workspace_root();
    let output = run_synod(&["run-demo", "--workspace", root.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "synod run-demo failed: {text}");

    // Step kinds appear in the rendered output.
    assert!(text.contains("analyze"), "missing analyze in: {text}");
    assert!(text.contains("code"), "missing code in: {text}");
    assert!(text.contains("verify"), "missing verify in: {text}");

    // Trace path is printed and the file exists.
    let trace_path = extract_trace_path(&text).expect("trace path printed");
    assert!(trace_path.exists(), "trace file missing at {trace_path:?}: {text}");

    // Final source file line is appended.
    assert!(text.contains("final source file"), "expected `final source file` line in: {text}");

    // The seeded buggy file is now fixed: bug marker absent.
    let target_file = root.join("src").join("buggy.rs");
    let body = fs::read_to_string(&target_file).expect("buggy.rs exists");
    assert!(
        !body.contains("// TODO-BUG: returns 1 instead of 0"),
        "bug marker still present after run-demo:\n{body}"
    );
    assert!(body.contains("fn answer() -> i32"));
    assert!(body.contains("0"), "fixed source must return 0:\n{body}");

    // Trace contains both retry and replan events plus a succeeded terminal_status.
    let trace_text = fs::read_to_string(&trace_path).expect("read trace");
    assert!(trace_text.contains("retry_scheduled"), "trace lacks retry: {trace_text}");
    assert!(trace_text.contains("replanned"), "trace lacks replan: {trace_text}");
    assert!(
        trace_text.contains("\"terminal_status\": \"succeeded\"")
            || trace_text.contains("\"terminal_status\":\"succeeded\""),
        "trace lacks succeeded terminal_status:\n{trace_text}"
    );
}

#[test]
fn run_demo_is_idempotent_across_consecutive_invocations() {
    let root = temp_workspace_root();
    let first = run_synod(&["run-demo", "--workspace", root.to_string_lossy().as_ref()]);
    assert_eq!(first.status.code(), Some(0), "{}", terminal_text(&first));
    let second = run_synod(&["run-demo", "--workspace", root.to_string_lossy().as_ref()]);
    assert_eq!(second.status.code(), Some(0), "{}", terminal_text(&second));

    let target_file = root.join("src").join("buggy.rs");
    let body = fs::read_to_string(&target_file).expect("buggy.rs exists");
    assert!(!body.contains("// TODO-BUG: returns 1 instead of 0"));
}
