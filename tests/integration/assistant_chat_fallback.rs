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

fn read_asset(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read assistant asset {}: {error}", path.display())
    })
}

#[test]
fn chat_fallback_assets_offer_repo_root_copyable_commands_for_us2() {
    let assets = [
        (
            "assistant/claude/commands/synod-run.md",
            "cargo run --bin synod -- run --workspace <workspace> --goal \"<goal>\"",
        ),
        (
            "assistant/codex/commands/synod-run.md",
            "cargo run --bin synod -- run --workspace <workspace> --goal \"<goal>\"",
        ),
        (
            "assistant/copilot/prompts/synod-run.prompt.md",
            "cargo run --bin synod -- run --workspace <workspace> --goal \"<goal>\"",
        ),
        (
            "assistant/claude/commands/synod-status.md",
            "cargo run --bin synod -- inspect --workspace <workspace>",
        ),
        (
            "assistant/codex/commands/synod-status.md",
            "cargo run --bin synod -- inspect --workspace <workspace>",
        ),
        (
            "assistant/copilot/prompts/synod-status.prompt.md",
            "cargo run --bin synod -- inspect --workspace <workspace>",
        ),
        (
            "assistant/claude/commands/synod-next.md",
            "cargo run --bin synod -- inspect --workspace <workspace>",
        ),
        (
            "assistant/codex/commands/synod-next.md",
            "cargo run --bin synod -- inspect --workspace <workspace>",
        ),
        (
            "assistant/copilot/prompts/synod-next.prompt.md",
            "cargo run --bin synod -- inspect --workspace <workspace>",
        ),
    ];

    for (path, command) in assets {
        let content = read_asset(path);
        let normalized = content.to_ascii_lowercase();
        assert!(content.contains(command), "{path} missing fallback command {command}");
        assert!(
            normalized.contains("wait for pasted output")
                || normalized.contains("wait for the user to paste the output"),
            "{path} should tell the user to paste command output"
        );
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
        let normalized = content.to_ascii_lowercase();
        assert!(content.contains(command), "{path} missing fallback command {command}");
        assert!(
            content.contains("cargo run --bin synod -- inspect --workspace <workspace>"),
            "{path} should include workspace-based inspect fallback"
        );
        assert!(
            normalized.contains("wait for pasted output")
                || normalized.contains("wait for the user to paste the output"),
            "{path} should tell the user to paste command output"
        );
        assert!(
            normalized.contains("corrected trace reference")
                || normalized.contains("replacement inspect command"),
            "{path}"
        );
    }
}

#[test]
fn chat_fallback_non_success_run_output_preserves_trace_and_next_step_cues() {
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
    assert!(run_text.contains("terminal_status:"), "{run_text}");
    assert!(run_text.contains("terminal_reason:"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");
    assert!(run_text.contains("next_command: /synod-next"), "{run_text}");

    let inspect_output =
        run_synod(&["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("terminal_reason:"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /synod-next"), "{inspect_text}");
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
