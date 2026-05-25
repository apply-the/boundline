use std::fs;
use std::path::{Path, PathBuf};

fn asset_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn read_asset(relative_path: &str) -> String {
    fs::read_to_string(asset_path(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

#[test]
fn host_support_paths_are_explicit_across_repo_guidance() {
    for (path, snippets) in [
        (
            "assistant/README.md",
            &[
                "Cursor is `copy-ready-assets`",
                "Antigravity is `repo-local-full`",
                "all hosts must treat CLI output plus\n`.boundline/session.json` as authoritative",
            ][..],
        ),
        (
            "assistant/global/cursor/README.md",
            &[
                "Support mode: `copy-ready-assets`.",
                "CLI remains the\nauthoritative runtime surface",
                ".boundline/session.json",
                "generated assets are copy-ready",
            ][..],
        ),
        (
            "assistant/global/antigravity/README.md",
            &[
                "Support mode: `manual-fallback`.",
                "repo-local Antigravity package surface",
                ".boundline/session.json",
                "Fallback CLI:",
            ][..],
        ),
        (
            "assistant/antigravity/README.md",
            &[
                "Support mode: `repo-local-full`.",
                "repo-local package surface through",
                "the CLI remains authoritative\nfor status, inspect, explain-plan",
                "Compatibility remains an explicit subordinate route.",
            ][..],
        ),
    ] {
        let content = read_asset(path);
        for snippet in snippets {
            assert!(content.contains(snippet), "{path} missing {snippet}");
        }
    }
}

#[test]
fn explain_plan_guidance_preserves_host_boundaries_and_delight_signals() {
    for path in [
        "assistant/claude/commands/boundline-explain-plan.md",
        "assistant/codex/commands/boundline-explain-plan.md",
        "assistant/antigravity/commands/boundline-explain-plan.md",
        "assistant/copilot/prompts/boundline-explain-plan.prompt.md",
    ] {
        let content = read_asset(path);
        for snippet in [
            "Cursor remains `copy-ready-assets`",
            "repo-local-full",
            "manual-fallback",
            "time_to_first_useful_answer_ms",
            "time_to_first_useful_answer_command",
            "explanation_attribution_rate",
            "next_action_acceptance_rate",
            "latest_next_action_outcome",
        ] {
            assert!(content.contains(snippet), "{path} missing {snippet}");
        }
    }
}
