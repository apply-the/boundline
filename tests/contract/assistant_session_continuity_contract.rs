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
        "cargo run --bin boundline -- start --workspace <workspace>",
        "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
        "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
        "cargo run --bin boundline -- plan --workspace <workspace>",
        "cargo run --bin boundline -- step --workspace <workspace>",
        "cargo run --bin boundline -- run --workspace <workspace>",
        "run --compatibility --goal ...",
        "cargo run --bin boundline -- status --workspace <workspace>",
        "cargo run --bin boundline -- next --workspace <workspace>",
        "assistants should prefer the native\nroute by default",
        "Preserve confirmed `workspace_ref`, captured goal, confirmed brief paths, authored input summary, and latest trace reference across assistant turns.",
        "continuity_authority",
        "compatibility_follow_up",
        "compatibility_follow_up_command",
        "effective_routing",
        "assistant_bindings",
        "assistant_runtimes",
        "follow_through_guidance",
        "follow_through_evidence_source",
        "follow_through_next_action",
        "follow_through_stop_reason",
        "Workspace-based inspect may reuse the active session's `latest_trace_ref` before falling back to the latest workspace trace.",
        "When CLI output includes `next_command`, prefer that route instead of inventing a follow-up.",
        "When `status` or `next` reports `continuity_authority: compatibility_trace` or `compatibility_follow_up: inspect_only`, route to `/boundline-inspect` instead of `/boundline-start`.",
        "governance_next_action",
        "latest_changed_files",
        "latest_validation_status",
    ] {
        assert!(content.contains(snippet), "assistant/README.md missing {snippet}");
    }
}

#[test]
fn assistant_readme_documents_clustered_session_guidance() {
    let content = read_asset("assistant/README.md");

    for snippet in [
        "cargo run --bin boundline -- start --cluster <primary-workspace>",
        "cargo run --bin boundline -- capture --cluster <primary-workspace> --goal \"<goal>\"",
        "cargo run --bin boundline -- plan --cluster <primary-workspace>",
        "cargo run --bin boundline -- run --cluster <primary-workspace>",
        "cargo run --bin boundline -- status --cluster <primary-workspace>",
        "cargo run --bin boundline -- next --cluster <primary-workspace>",
        "cargo run --bin boundline -- inspect --cluster <primary-workspace>",
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
fn assistant_readme_documents_workflow_first_guidance() {
    let content = read_asset("assistant/README.md");

    for snippet in [
        "/boundline-workflow-list",
        "/boundline-workflow-run",
        "/boundline-workflow-status",
        "/boundline-workflow-resume",
        "/boundline-workflow-inspect",
        "cargo run --bin boundline -- workflow list --workspace <workspace>",
        "cargo run --bin boundline -- workflow run <name> --workspace <workspace>",
        "cargo run --bin boundline -- workflow status --workspace <workspace>",
        "cargo run --bin boundline -- workflow resume --workspace <workspace>",
        "cargo run --bin boundline -- workflow inspect --workspace <workspace>",
        "workflows and direct runs are primary surfaces",
        "compatibility remains explicit and subordinate",
    ] {
        assert!(content.contains(snippet), "assistant/README.md missing {snippet}");
    }
}

#[test]
fn gemini_guidance_uses_workflow_first_vocabulary() {
    let content = read_asset("assistant/gemini/README.md");

    for snippet in [
        "cargo run --bin boundline -- workflow list --workspace <workspace>",
        "cargo run --bin boundline -- workflow run <name> --workspace <workspace>",
        "cargo run --bin boundline -- workflow status --workspace <workspace>",
        "cargo run --bin boundline -- workflow resume --workspace <workspace>",
        "cargo run --bin boundline -- workflow inspect --workspace <workspace>",
        "primary Boundline workflow surface",
        "compatibility remains an explicit subordinate route",
    ] {
        assert!(content.contains(snippet), "assistant/gemini/README.md missing {snippet}");
    }
}

#[test]
fn inspect_assets_document_session_trace_reuse_and_start_recovery() {
    let assets = [
        "assistant/claude/commands/boundline-inspect.md",
        "assistant/codex/commands/boundline-inspect.md",
        "assistant/copilot/prompts/boundline-inspect.prompt.md",
    ];

    for path in assets {
        let content = read_asset(path);
        for snippet in [
            "latest_trace_ref",
            "route_config_projection",
            "changed_files",
            "validation",
            "/boundline-start",
            "prior direct run opted into `--compatibility`",
            "authored_input_summary",
            "authored_input_sources",
            "authored_input_deduplicated_sources",
            "governance_next_action",
            "follow_through_guidance",
            "follow_through_evidence_source",
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
            "assistant/claude/commands/boundline-start.md",
            &["cargo run --bin boundline -- start --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/boundline-start.md",
            &["cargo run --bin boundline -- start --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/boundline-start.prompt.md",
            &["cargo run --bin boundline -- start --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/boundline-plan.md",
            &[
                "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin boundline -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/codex/commands/boundline-plan.md",
            &[
                "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin boundline -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/copilot/prompts/boundline-plan.prompt.md",
            &[
                "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin boundline -- plan --workspace <workspace>",
            ][..],
        ),
        (
            "assistant/claude/commands/boundline-step.md",
            &["cargo run --bin boundline -- step --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/boundline-step.md",
            &["cargo run --bin boundline -- step --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/boundline-step.prompt.md",
            &["cargo run --bin boundline -- step --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/boundline-run.md",
            &["cargo run --bin boundline -- run --workspace <workspace>"][..],
        ),
        (
            "assistant/codex/commands/boundline-run.md",
            &["cargo run --bin boundline -- run --workspace <workspace>"][..],
        ),
        (
            "assistant/copilot/prompts/boundline-run.prompt.md",
            &["cargo run --bin boundline -- run --workspace <workspace>"][..],
        ),
        (
            "assistant/claude/commands/boundline-status.md",
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "authored_input_summary",
                "continuity_authority",
                "compatibility_follow_up",
                "governance_next_action",
                "latest_changed_files",
                "latest_validation_status",
            ][..],
        ),
        (
            "assistant/codex/commands/boundline-status.md",
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "authored_input_summary",
                "continuity_authority",
                "compatibility_follow_up",
                "governance_next_action",
                "latest_changed_files",
                "latest_validation_status",
            ][..],
        ),
        (
            "assistant/copilot/prompts/boundline-status.prompt.md",
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "authored_input_summary",
                "continuity_authority",
                "compatibility_follow_up",
                "governance_next_action",
                "latest_changed_files",
                "latest_validation_status",
            ][..],
        ),
        (
            "assistant/claude/commands/boundline-next.md",
            &[
                "cargo run --bin boundline -- next --workspace <workspace>",
                "continuity_authority",
                "compatibility_follow_up",
            ][..],
        ),
        (
            "assistant/codex/commands/boundline-next.md",
            &[
                "cargo run --bin boundline -- next --workspace <workspace>",
                "continuity_authority",
                "compatibility_follow_up",
            ][..],
        ),
        (
            "assistant/copilot/prompts/boundline-next.prompt.md",
            &[
                "cargo run --bin boundline -- next --workspace <workspace>",
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

#[test]
fn assistant_command_packs_expose_canon_default_mode_aliases() {
    let modes = [
        "requirements",
        "discovery",
        "system-shaping",
        "architecture",
        "backlog",
        "change",
        "implementation",
        "refactor",
        "review",
        "verification",
        "incident",
        "security-assessment",
        "system-assessment",
        "migration",
        "supply-chain-analysis",
    ];

    let run_assets = [
        "assistant/copilot/prompts/boundline-run.prompt.md",
        "assistant/codex/commands/boundline-run.md",
        "assistant/claude/commands/boundline-run.md",
        "assistant/gemini/README.md",
    ];
    for path in run_assets {
        let content = read_asset(path);
        assert!(content.contains("boundline run --mode <mode>"), "{path} missing mode shorthand");
        assert!(content.contains("Canon-default"), "{path} missing Canon-default wording");
        assert!(
            !content.contains("--governance canon"),
            "{path} should not require explicit Canon governance"
        );
    }

    for mode in modes {
        let path = format!("assistant/copilot/prompts/boundline-{mode}.prompt.md");
        let content = read_asset(&path);
        assert!(content.contains(&format!("/boundline-{mode}")), "{path} missing alias command");
        assert!(
            content.contains(&format!("boundline run --mode {mode}")),
            "{path} missing canonical CLI mapping"
        );
        assert!(content.contains("governance_runtime"), "{path} missing lifecycle fields");
        assert!(content.contains("mode_selection_preference"), "{path} missing lifecycle fields");
        assert!(content.contains("selected_mode"), "{path} missing lifecycle fields");
        assert!(content.contains("approval_state"), "{path} missing lifecycle fields");
        assert!(content.contains("next_action"), "{path} missing lifecycle fields");
    }
}

#[test]
fn copilot_prompts_preserve_canon_default_cli_boundaries() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let prompt_root = manifest_dir.join("assistant/copilot/prompts");

    for entry in std::fs::read_dir(&prompt_root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", prompt_root.display()))
    {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert!(
            !content.contains("--governance canon"),
            "{} should not require explicit Canon governance",
            path.display()
        );
        assert!(
            !content.contains("edit manifest") && !content.contains("manual manifest"),
            "{} should not instruct manual manifest editing as the primary path",
            path.display()
        );
    }

    for command in [
        "boundline-init",
        "boundline-doctor",
        "boundline-config-show",
        "boundline-config-set-canon",
        "boundline-capture",
        "boundline-run",
    ] {
        let path = format!("assistant/copilot/prompts/{command}.prompt.md");
        let content = read_asset(&path);
        assert!(content.contains("boundline "), "{path} should map back to the Boundline CLI");
    }
}
