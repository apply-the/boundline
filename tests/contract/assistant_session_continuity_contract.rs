use std::fs;
use std::path::{Path, PathBuf};

fn asset_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn read_asset(relative_path: &str) -> String {
    fs::read_to_string(asset_path(relative_path)).unwrap()
}

#[test]
fn assistant_readme_documents_session_native_continuity_rules() {
    let content = read_asset("assistant/README.md");

    for snippet in [
        "cargo run --bin synod -- start --workspace <workspace>",
        "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
        "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
        "cargo run --bin synod -- plan --workspace <workspace>",
        "cargo run --bin synod -- step --workspace <workspace>",
        "cargo run --bin synod -- run --workspace <workspace>",
        "cargo run --bin synod -- status --workspace <workspace>",
        "cargo run --bin synod -- next --workspace <workspace>",
        "Preserve confirmed `workspace_ref`, captured goal, confirmed brief paths, authored input summary, and latest trace reference across assistant turns.",
        "continuity_authority",
        "compatibility_follow_up",
        "compatibility_follow_up_command",
        "Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.",
        "When CLI output includes `next_command`, prefer that route instead of inventing a follow-up.",
        "When `status` or `next` reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/synod-inspect` instead of `/synod-start`.",
        "governance_next_action",
    ] {
        assert!(content.contains(snippet), "assistant/README.md missing {snippet}");
    }
}

#[test]
fn assistant_readme_documents_clustered_session_guidance() {
    let content = read_asset("assistant/README.md");

    for snippet in [
        "cargo run --bin synod -- start --cluster <primary-workspace>",
        "cargo run --bin synod -- capture --cluster <primary-workspace> --goal \"<goal>\"",
        "cargo run --bin synod -- plan --cluster <primary-workspace>",
        "cargo run --bin synod -- run --cluster <primary-workspace>",
        "cargo run --bin synod -- status --cluster <primary-workspace>",
        "cargo run --bin synod -- next --cluster <primary-workspace>",
        "cargo run --bin synod -- inspect --cluster <primary-workspace>",
        "cluster_route_owner",
        "cluster_authoritative_workspace",
        "cluster_execution_condition",
        "cluster_participating_workspaces",
        "cluster_blocking_workspace",
    ] {
        assert!(content.contains(snippet), "assistant/README.md missing {snippet}");
    }
}

#[test]
fn inspect_assets_document_session_trace_reuse_and_start_recovery() {
    let assets = [
        "assistant/claude/commands/synod-inspect.md",
        "assistant/codex/commands/synod-inspect.md",
        "assistant/copilot/prompts/synod-inspect.prompt.md",
    ];

    for path in assets {
        let content = read_asset(path);
        for snippet in [
            "latest_trace_ref",
            "/synod-start",
            "authored_input_summary",
            "authored_input_sources",
            "authored_input_deduplicated_sources",
            "governance_next_action",
            "corrected_command",
        ] {
            assert!(content.contains(snippet), "{path} missing {snippet}");
        }
    }
}

#[test]
fn assistant_command_packs_expose_session_native_backend_mappings() {
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
                "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin synod -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/codex/commands/synod-plan.md",
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin synod -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/copilot/prompts/synod-plan.prompt.md",
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
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
            &[
                "cargo run --bin synod -- status --workspace <workspace>",
                "authored_input_summary",
                "continuity_authority",
                "compatibility_follow_up",
                "governance_next_action",
            ][..],
        ),
        (
            "assistant/codex/commands/synod-status.md",
            &[
                "cargo run --bin synod -- status --workspace <workspace>",
                "authored_input_summary",
                "continuity_authority",
                "compatibility_follow_up",
                "governance_next_action",
            ][..],
        ),
        (
            "assistant/copilot/prompts/synod-status.prompt.md",
            &[
                "cargo run --bin synod -- status --workspace <workspace>",
                "authored_input_summary",
                "continuity_authority",
                "compatibility_follow_up",
                "governance_next_action",
            ][..],
        ),
        (
            "assistant/claude/commands/synod-next.md",
            &[
                "cargo run --bin synod -- next --workspace <workspace>",
                "continuity_authority",
                "compatibility_follow_up",
            ][..],
        ),
        (
            "assistant/codex/commands/synod-next.md",
            &[
                "cargo run --bin synod -- next --workspace <workspace>",
                "continuity_authority",
                "compatibility_follow_up",
            ][..],
        ),
        (
            "assistant/copilot/prompts/synod-next.prompt.md",
            &[
                "cargo run --bin synod -- next --workspace <workspace>",
                "continuity_authority",
                "compatibility_follow_up",
            ][..],
        ),
    ];

    for (path, snippets) in assets {
        let content = read_asset(path);
        for snippet in snippets {
            assert!(content.contains(snippet), "{path} missing {snippet}");
        }
    }
}
