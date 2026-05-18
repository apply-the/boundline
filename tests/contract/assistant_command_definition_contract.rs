use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED_SECTIONS: &[&str] = &[
    "## Intent",
    "## Required Context",
    "## Shell-Enabled Path",
    "## Chat-Only Path",
    "## Output Interpretation",
    "## Next-Step Routing",
];

#[test]
fn test_start_and_plan_definition_sections_and_backend_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-start.md"),
            &["cargo run --bin boundline -- start --workspace <workspace>", "/boundline-plan"][..],
            &["cargo run --bin boundline -- doctor --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-start.md"),
            &["cargo run --bin boundline -- start --workspace <workspace>", "/boundline-plan"][..],
            &["cargo run --bin boundline -- doctor --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-start.prompt.md"),
            &["cargo run --bin boundline -- start --workspace <workspace>", "/boundline-plan"][..],
            &["cargo run --bin boundline -- doctor --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-plan.md"),
            &[
                "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin boundline -- plan --workspace <workspace>",
                "authored_input_summary",
                "/boundline-step",
            ][..],
            &["No direct CLI invocation is required"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-plan.md"),
            &[
                "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin boundline -- plan --workspace <workspace>",
                "authored_input_summary",
                "/boundline-step",
            ][..],
            &["No direct CLI invocation is required"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
            &[
                "cargo run --bin boundline -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin boundline -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin boundline -- plan --workspace <workspace>",
                "authored_input_summary",
                "/boundline-step",
            ][..],
            &["No direct CLI invocation is required"][..],
        ),
    ];

    for (path, required, forbidden) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, required);
        assert_forbidden_snippets(&path, &content, forbidden);
    }
}

#[test]
fn test_step_run_status_and_next_definition_sections_and_backend_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-step.md"),
            &[
                "cargo run --bin boundline -- step --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-step.md"),
            &[
                "cargo run --bin boundline -- step --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-step.prompt.md"),
            &[
                "cargo run --bin boundline -- step --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-run.md"),
            &[
                "cargo run --bin boundline -- run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/boundline-inspect",
            ][..],
            &["cargo run --bin boundline -- run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-run.md"),
            &[
                "cargo run --bin boundline -- run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/boundline-inspect",
            ][..],
            &["cargo run --bin boundline -- run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-run.prompt.md"),
            &[
                "cargo run --bin boundline -- run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/boundline-inspect",
            ][..],
            &["cargo run --bin boundline -- run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-status.md"),
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["cargo run --bin boundline -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-status.md"),
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["cargo run --bin boundline -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-status.prompt.md"),
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["cargo run --bin boundline -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-next.md"),
            &[
                "cargo run --bin boundline -- next --workspace <workspace>",
                "latest_trace_ref",
                "follow_through_guidance",
                "next_command",
            ][..],
            &["cargo run --bin boundline -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-next.md"),
            &[
                "cargo run --bin boundline -- next --workspace <workspace>",
                "latest_trace_ref",
                "follow_through_guidance",
                "next_command",
            ][..],
            &["cargo run --bin boundline -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-next.prompt.md"),
            &[
                "cargo run --bin boundline -- next --workspace <workspace>",
                "latest_trace_ref",
                "follow_through_guidance",
                "next_command",
            ][..],
            &["cargo run --bin boundline -- inspect --workspace <workspace>"][..],
        ),
    ];

    for (path, required, forbidden) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, required);
        assert_forbidden_snippets(&path, &content, forbidden);
    }
}

#[test]
fn test_inspect_definition_sections_and_trace_read_failure_expectations() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-inspect.md"),
            &[
                "cargo run --bin boundline -- inspect --trace <trace>",
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "corrected_command",
                "/boundline-start",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-inspect.md"),
            &[
                "cargo run --bin boundline -- inspect --trace <trace>",
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "corrected_command",
                "/boundline-start",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-inspect.prompt.md"),
            &[
                "cargo run --bin boundline -- inspect --trace <trace>",
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "corrected_command",
                "/boundline-start",
                "next_command",
            ][..],
        ),
    ];

    for (path, snippets) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, snippets);
    }
}

#[test]
fn test_workflow_definition_sections_and_backend_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-workflow-list.md"),
            &[
                "cargo run --bin boundline -- workflow list --workspace <workspace>",
                "workflow registry status",
                "/boundline-workflow-run",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-workflow-list.md"),
            &[
                "cargo run --bin boundline -- workflow list --workspace <workspace>",
                "workflow registry status",
                "/boundline-workflow-run",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-workflow-list.prompt.md"),
            &[
                "cargo run --bin boundline -- workflow list --workspace <workspace>",
                "workflow registry status",
                "/boundline-workflow-run",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-workflow-run.md"),
            &[
                "cargo run --bin boundline -- workflow run <name> --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-workflow-run.md"),
            &[
                "cargo run --bin boundline -- workflow run <name> --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-workflow-run.prompt.md"),
            &[
                "cargo run --bin boundline -- workflow run <name> --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-workflow-status.md"),
            &[
                "cargo run --bin boundline -- workflow status --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-workflow-status.md"),
            &[
                "cargo run --bin boundline -- workflow status --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-workflow-status.prompt.md"),
            &[
                "cargo run --bin boundline -- workflow status --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-workflow-resume.md"),
            &[
                "cargo run --bin boundline -- workflow resume --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-workflow-resume.md"),
            &[
                "cargo run --bin boundline -- workflow resume --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-workflow-resume.prompt.md"),
            &[
                "cargo run --bin boundline -- workflow resume --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-workflow-inspect.md"),
            &[
                "cargo run --bin boundline -- workflow inspect --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-workflow-inspect.md"),
            &[
                "cargo run --bin boundline -- workflow inspect --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-workflow-inspect.prompt.md"),
            &[
                "cargo run --bin boundline -- workflow inspect --workspace <workspace>",
                "workflow_phase",
                "route_config_projection",
                "next_command",
            ][..],
        ),
    ];

    for (path, snippets) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, snippets);
    }
}

#[test]
fn test_s7_mvp_definition_sections_exist() {
    let assets = [
        asset_path("assistant/claude/commands/boundline-why.md"),
        asset_path("assistant/codex/commands/boundline-why.md"),
        asset_path("assistant/copilot/prompts/boundline-why.prompt.md"),
        asset_path("assistant/claude/commands/boundline-risk.md"),
        asset_path("assistant/codex/commands/boundline-risk.md"),
        asset_path("assistant/copilot/prompts/boundline-risk.prompt.md"),
        asset_path("assistant/claude/commands/boundline-evidence.md"),
        asset_path("assistant/codex/commands/boundline-evidence.md"),
        asset_path("assistant/copilot/prompts/boundline-evidence.prompt.md"),
        asset_path("assistant/claude/commands/boundline-next-best.md"),
        asset_path("assistant/codex/commands/boundline-next-best.md"),
        asset_path("assistant/copilot/prompts/boundline-next-best.prompt.md"),
    ];

    for path in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
    }
}

#[test]
fn s7_us2_definition_sections_and_backend_mappings_exist() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-assumptions.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "assumptions_summary",
                "assumption_group",
                "fallback_disclosure",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-assumptions.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "assumptions_summary",
                "assumption_group",
                "fallback_disclosure",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-assumptions.prompt.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "assumptions_summary",
                "assumption_group",
                "fallback_disclosure",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-hidden-impact.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "hidden_impact_summary",
                "hidden_impact_fallback_disclosure",
                "challenge_required_review",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-hidden-impact.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "hidden_impact_summary",
                "hidden_impact_fallback_disclosure",
                "challenge_required_review",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-hidden-impact.prompt.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "hidden_impact_summary",
                "hidden_impact_fallback_disclosure",
                "challenge_required_review",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-challenge.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "challenge_strongest_objection",
                "challenge_required_review",
                "challenge_council_required",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-challenge.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "challenge_strongest_objection",
                "challenge_required_review",
                "challenge_council_required",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-challenge.prompt.md"),
            &[
                "cargo run --bin boundline -- inspect --workspace <workspace>",
                "challenge_strongest_objection",
                "challenge_required_review",
                "challenge_council_required",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-explain-plan.md"),
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "explain_plan_summary",
                "explain_plan_validation",
                "explain_plan_governance",
                "explain_plan_recovery",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-explain-plan.md"),
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "explain_plan_summary",
                "explain_plan_validation",
                "explain_plan_governance",
                "explain_plan_recovery",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-explain-plan.prompt.md"),
            &[
                "cargo run --bin boundline -- status --workspace <workspace>",
                "explain_plan_summary",
                "explain_plan_validation",
                "explain_plan_governance",
                "explain_plan_recovery",
            ][..],
        ),
    ];

    for (path, snippets) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, snippets);
    }
}

fn assert_forbidden_snippets(path: &Path, content: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !content.contains(snippet),
            "assistant asset {} still contains deprecated mapping snippet {snippet}",
            path.display()
        );
    }
}

fn asset_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn read_asset(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read assistant asset {}: {error}", path.display())
    })
}

fn assert_required_sections(path: &Path, content: &str) {
    for section in REQUIRED_SECTIONS {
        assert!(
            content.contains(section),
            "assistant asset {} is missing required section {section}",
            path.display()
        );
    }
}

fn assert_required_snippets(path: &Path, content: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            content.contains(snippet),
            "assistant asset {} is missing required mapping snippet {snippet}",
            path.display()
        );
    }
}
