use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace =
        std::env::temp_dir().join(format!("synod-assistant-chat-fallback-{}", Uuid::new_v4()));
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

fn read_asset(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read assistant asset {}: {error}", path.display())
    })
}

fn assert_wait_for_paste(path: &str, content: &str) {
    let normalized = content.to_ascii_lowercase();
    assert!(
        normalized.contains("wait for pasted output")
            || normalized.contains("wait for the user to paste the output")
            || normalized.contains("paste the outputs before continuing")
            || normalized.contains("paste the output before continuing"),
        "{path} should tell the user to paste command output"
    );
}

fn bootstrap_session(workspace: &Path, goal: &str) {
    let workspace_ref = workspace.to_string_lossy().into_owned();

    let start_output = run_synod(&["start", "--workspace", &workspace_ref]);
    let start_text = terminal_text(&start_output);
    assert_eq!(start_output.status.code(), Some(0), "{start_text}");

    let capture_output = run_synod(&["capture", "--workspace", &workspace_ref, "--goal", goal]);
    let capture_text = terminal_text(&capture_output);
    assert_eq!(capture_output.status.code(), Some(0), "{capture_text}");

    let plan_output = run_synod(&["plan", "--workspace", &workspace_ref]);
    let plan_text = terminal_text(&plan_output);
    assert_eq!(plan_output.status.code(), Some(0), "{plan_text}");
}

#[test]
fn chat_fallback_assets_offer_repo_root_copyable_commands_for_session_native_flow() {
    let assets = [
        (
            "assistant/claude/commands/synod-start.md",
            &["cargo run --bin synod -- start --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/synod-start.md",
            &["cargo run --bin synod -- start --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/synod-start.prompt.md",
            &["cargo run --bin synod -- start --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/synod-plan.md",
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/codex/commands/synod-plan.md",
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/copilot/prompts/synod-plan.prompt.md",
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/claude/commands/synod-step.md",
            &["cargo run --bin synod -- step --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/synod-step.md",
            &["cargo run --bin synod -- step --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/synod-step.prompt.md",
            &["cargo run --bin synod -- step --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/synod-run.md",
            &["cargo run --bin synod -- run --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/synod-run.md",
            &["cargo run --bin synod -- run --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/synod-run.prompt.md",
            &["cargo run --bin synod -- run --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/synod-status.md",
            &["cargo run --bin synod -- status --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/synod-status.md",
            &["cargo run --bin synod -- status --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/synod-status.prompt.md",
            &["cargo run --bin synod -- status --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/synod-next.md",
            &["cargo run --bin synod -- next --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/synod-next.md",
            &["cargo run --bin synod -- next --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/synod-next.prompt.md",
            &["cargo run --bin synod -- next --workspace <workspace>"][..],
        ),
    ];

    for (path, commands) in assets {
        let content = read_asset(path);
        for command in commands {
            assert!(content.contains(command), "{path} missing fallback command {command}");
        }
        assert_wait_for_paste(path, &content);
    }
}

#[test]
fn chat_fallback_assets_offer_repo_root_copyable_commands_for_us3() {
    let assets = [
        (
            "assistant/claude/commands/synod-inspect.md",
            "cargo run --bin synod -- inspect --trace <trace>",
        ),
        (
            "assistant/codex/commands/synod-inspect.md",
            "cargo run --bin synod -- inspect --trace <trace>",
        ),
        (
            "assistant/copilot/prompts/synod-inspect.prompt.md",
            "cargo run --bin synod -- inspect --trace <trace>",
        ),
    ];

    for (path, command) in assets {
        let content = read_asset(path);
        assert!(content.contains(command), "{path} missing fallback command {command}");
        assert!(
            content.contains("cargo run --bin synod -- inspect --workspace <workspace>"),
            "{path} should include workspace-based inspect fallback"
        );
        assert!(
            content.contains("latest_trace_ref"),
            "{path} should mention latest_trace_ref reuse"
        );
        assert!(
            content.contains("/synod-start"),
            "{path} should route session errors to /synod-start"
        );
        assert_wait_for_paste(path, &content);
        assert!(
            content.to_ascii_lowercase().contains("corrected trace reference")
                || content.to_ascii_lowercase().contains("replacement inspect command"),
            "{path}"
        );
    }
}

#[test]
fn chat_fallback_non_success_session_native_run_output_preserves_trace_and_next_step_cues() {
    let workspace = temp_workspace();
    bootstrap_session(&workspace, "Force a non-success failure for the default developer flow");
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let run_output = run_synod(&["run", "--workspace", &workspace_ref]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_status:"), "{run_text}");
    assert!(run_text.contains("terminal_reason:"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");
    assert!(run_text.contains("next_command: synod inspect"), "{run_text}");

    let status_output = run_synod(&["status", "--workspace", &workspace_ref]);
    let status_text = terminal_text(&status_output);

    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref:"), "{status_text}");
    assert!(status_text.contains("next_command: synod inspect"), "{status_text}");

    let next_output = run_synod(&["next", "--workspace", &workspace_ref]);
    let next_text = terminal_text(&next_output);

    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: synod inspect"), "{next_text}");
}

#[test]
fn chat_fallback_inspect_trace_read_failures_preserve_correction_cues() {
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
