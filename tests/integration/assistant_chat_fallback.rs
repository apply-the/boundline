use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::workspace_fixture::{run_boundline, temp_broken_fixture_workspace, terminal_text};

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

    let goal_output = run_boundline(&["goal", "--workspace", &workspace_ref, "--goal", goal]);
    let goal_text = terminal_text(&goal_output);
    assert_eq!(goal_output.status.code(), Some(0), "{goal_text}");

    let plan_output = run_boundline(&["plan", "--workspace", &workspace_ref, "--flow", "bug-fix"]);
    let plan_text = terminal_text(&plan_output);
    assert_eq!(plan_output.status.code(), Some(0), "{plan_text}");
}

#[test]
fn chat_fallback_assets_offer_repo_root_copyable_commands_for_session_native_flow() {
    let assets = [
        (
            "assistant/claude/commands/boundline-goal.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "phase_request",
                "## Host Capabilities",
                "Boundline needs one answer before it can continue",
            ][..],
        ),
        (
            "assistant/codex/commands/boundline-goal.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "phase_request",
                "## Host Capabilities",
                "Boundline needs one answer before it can continue",
            ][..],
        ),
        (
            "assistant/antigravity/commands/boundline-goal.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "phase_request",
                "## Host Capabilities",
                "Boundline needs one answer before it can continue",
            ][..],
        ),
        (
            "assistant/copilot/prompts/boundline-goal.prompt.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream",
                "phase_request",
                "Agent Mode Override",
            ][..],
        ),
        (
            "assistant/claude/commands/boundline-plan.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline plan --workspace <workspace> --input <path> --json",
                "phase_request",
            ][..],
        ),
        (
            "assistant/codex/commands/boundline-plan.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline plan --workspace <workspace> --input <path> --json",
                "phase_request",
            ][..],
        ),
        (
            "assistant/copilot/prompts/boundline-plan.prompt.md",
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline plan --workspace <workspace> --input <path> --json",
            ][..],
        ),
        (
            "assistant/claude/commands/boundline-step.md",
            &["boundline step --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/boundline-step.md",
            &["boundline step --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/boundline-step.prompt.md",
            &["boundline step --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/boundline-run.md",
            &[
                "boundline orchestrate --workspace <workspace> --until terminal --json-stream",
                "phase_request",
            ][..],
        ),
        (
            "assistant/codex/commands/boundline-run.md",
            &[
                "boundline orchestrate --workspace <workspace> --until terminal --json-stream",
                "phase_request",
            ][..],
        ),
        (
            "assistant/copilot/prompts/boundline-run.prompt.md",
            &["boundline orchestrate --workspace <workspace> --until terminal --json-stream"][..],
        ),
        (
            "assistant/claude/commands/boundline-status.md",
            &["boundline status --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/boundline-status.md",
            &["boundline status --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/boundline-status.prompt.md",
            &["boundline status --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/boundline-next.md",
            &["boundline next --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/boundline-next.md",
            &["boundline next --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/boundline-next.prompt.md",
            &["boundline next --workspace <workspace>"][..],
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
        ("assistant/claude/commands/boundline-inspect.md", "boundline inspect --trace <trace>"),
        ("assistant/codex/commands/boundline-inspect.md", "boundline inspect --trace <trace>"),
        (
            "assistant/copilot/prompts/boundline-inspect.prompt.md",
            "boundline inspect --trace <trace>",
        ),
    ];

    for (path, command) in assets {
        let content = read_asset(path);
        assert!(content.contains(command), "{path} missing fallback command {command}");
        assert!(
            content.contains("boundline inspect --workspace <workspace>"),
            "{path} should include workspace-based inspect fallback"
        );
        assert!(
            content.contains("latest_trace_ref"),
            "{path} should mention latest_trace_ref reuse"
        );
        assert!(
            content.contains("/boundline-goal"),
            "{path} should route session errors to /boundline-goal"
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
fn chat_fallback_session_native_run_output_preserves_trace_and_next_step_cues() {
    let workspace = temp_broken_fixture_workspace("boundline-assistant-chat-fallback-broken");
    bootstrap_session(&workspace, "Attempt the fixture patch on a broken workspace");
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let run_output = run_boundline(&["run", "--workspace", &workspace_ref]);
    let run_text = terminal_text(&run_output);

    assert_eq!(run_output.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("trace="), "{run_text}");
    assert!(run_text.contains("next_command: boundline inspect"), "{run_text}");

    let status_output = run_boundline(&["status", "--workspace", &workspace_ref]);
    let status_text = terminal_text(&status_output);

    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref: "), "{status_text}");
    assert!(status_text.contains("next_command: boundline inspect"), "{status_text}");

    let next_output = run_boundline(&["next", "--workspace", &workspace_ref]);
    let next_text = terminal_text(&next_output);

    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: boundline inspect"), "{next_text}");
}

#[test]
fn chat_fallback_inspect_trace_read_failures_preserve_correction_cues() {
    let missing_trace =
        std::env::temp_dir().join(format!("boundline-missing-trace-{}.json", Uuid::new_v4()));
    let inspect_output =
        run_boundline(&["inspect", "--trace", missing_trace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect_output);

    assert_eq!(inspect_output.status.code(), Some(3), "{inspect_text}");
    assert!(inspect_text.contains("inspect: trace read failure"), "{inspect_text}");
    assert!(
        inspect_text.contains("terminal_reason: failed to read the requested trace"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("next_command: /boundline-inspect"), "{inspect_text}");
    assert!(
        inspect_text.contains("corrected_command: boundline inspect --trace"),
        "{inspect_text}"
    );
}
