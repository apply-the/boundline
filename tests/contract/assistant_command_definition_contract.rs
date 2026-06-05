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
fn test_goal_and_plan_definition_sections_and_backend_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-goal.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "phase_request",
                "request_id",
                "/boundline-goal",
                "## Host Capabilities",
                "Boundline needs one answer before it can continue",
                "suggested_choice",
            ][..],
            &["boundline start --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-goal.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "phase_request",
                "request_id",
                "/boundline-goal",
                "## Host Capabilities",
                "Boundline needs one answer before it can continue",
                "suggested_choice",
            ][..],
            &["boundline start --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/antigravity/commands/boundline-goal.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "phase_request",
                "request_id",
                "/boundline-goal",
                "## Host Capabilities",
                "Boundline needs one answer before it can continue",
                "optional",
                "suggested_choice",
            ][..],
            &["boundline start --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-goal.prompt.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --slug <derived-slug> --assistant-host copilot --until phase-request --json-stream",
                "phase_request",
                "request_id",
                "/boundline-goal",
                "Agent Mode Override",
                "suggested_choice",
            ][..],
            &["boundline start --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-plan.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "boundline plan --workspace <workspace> --input <path> --json",
                "phase_request",
                "request_id",
                "authored_input_summary",
                "/boundline-step",
            ][..],
            &["No direct CLI invocation is required", "plan confirmation"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-plan.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --brief <path> [--brief <path> ...] --until phase-request --json-stream",
                "boundline plan --workspace <workspace> --input <path> --json",
                "phase_request",
                "request_id",
                "authored_input_summary",
                "/boundline-step",
            ][..],
            &["No direct CLI invocation is required", "plan confirmation"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
            &[
                "boundline orchestrate --workspace <workspace> --goal \"<goal>\" --until phase-request --json-stream",
                "boundline plan --workspace <workspace> --input <path> --json",
                "authored_input_summary",
                "/boundline-step",
            ][..],
            &[
                "No direct CLI invocation is required",
                "boundline orchestrate --workspace <workspace> --brief <path> [--brief <path> ...] --until phase-request --json-stream",
            ][..],
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
fn goal_quality_contract_sections_are_present_in_assistant_assets() {
    let goal_assets = [
        asset_path("assistant/claude/commands/boundline-goal.md"),
        asset_path("assistant/codex/commands/boundline-goal.md"),
        asset_path("assistant/antigravity/commands/boundline-goal.md"),
        asset_path("assistant/copilot/prompts/boundline-goal.prompt.md"),
    ];
    let goal_required = [
        "## User Input",
        "## Pre-Execution Checks",
        "## Execution Flow",
        "## Goal Quality Validation",
        "## Quick Guidelines",
        "## Reasonable Defaults",
        "## Success Criteria Guidelines",
        "## Done When",
        "goal_quality_state",
        "goal_quality_findings",
        "goal_quality_assumptions",
        "Do not read `.specify/extensions.yml`",
        "scope > security/privacy > user experience > technical details",
        "Maximum 3",
    ];

    for path in goal_assets {
        let content = read_asset(&path);
        assert_required_snippets(&path, &content, &goal_required);
    }

    let plan_assets = [
        asset_path("assistant/claude/commands/boundline-plan.md"),
        asset_path("assistant/codex/commands/boundline-plan.md"),
        asset_path("assistant/antigravity/commands/boundline-plan.md"),
        asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
    ];

    for path in plan_assets {
        let content = read_asset(&path);
        assert_required_snippets(
            &path,
            &content,
            &[
                "Do not proceed from chat-only assumptions when goal quality or plan quality is blocked",
                "goal_quality_state",
            ],
        );
    }

    let template_path = asset_path("assistant/prompts/goal-template.md");
    let template = read_asset(&template_path);
    assert_required_snippets(
        &template_path,
        &template,
        &[
            "## Success Criteria",
            "## Acceptance Scenarios",
            "## Edge Cases",
            "## Reasonable Defaults",
            "## Goal Quality Checklist",
            "## Done When",
        ],
    );
}

#[test]
fn plan_quality_contract_sections_are_present_in_assistant_assets() {
    let plan_assets = [
        asset_path("assistant/claude/commands/boundline-plan.md"),
        asset_path("assistant/codex/commands/boundline-plan.md"),
        asset_path("assistant/antigravity/commands/boundline-plan.md"),
        asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
    ];
    let required = [
        "## User Input",
        "## Pre-Execution Checks",
        "## Execution Flow",
        "## Plan Quality Validation",
        "## Gate Handling",
        "## Planning Analysis Gate",
        "## Reasonable Defaults",
        "## Quick Guidelines",
        "## Success Criteria Guidelines",
        "## Done When",
        "plan_quality_state",
        "plan_quality_findings",
        "plan_quality_assumptions",
        "Do not proceed from chat-only assumptions when goal quality or plan quality is blocked",
        "phase_request",
    ];

    for path in plan_assets {
        let content = read_asset(&path);
        assert_required_snippets(&path, &content, &required);
    }
}

#[test]
fn planning_analysis_contract_sections_are_present_in_assistant_assets() {
    let assets = [
        asset_path("assistant/claude/commands/boundline-plan.md"),
        asset_path("assistant/codex/commands/boundline-plan.md"),
        asset_path("assistant/antigravity/commands/boundline-plan.md"),
        asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
        asset_path("assistant/claude/commands/boundline-run.md"),
        asset_path("assistant/codex/commands/boundline-run.md"),
        asset_path("assistant/antigravity/commands/boundline-run.md"),
        asset_path("assistant/copilot/prompts/boundline-run.prompt.md"),
        asset_path("assistant/claude/commands/boundline-status.md"),
        asset_path("assistant/codex/commands/boundline-status.md"),
        asset_path("assistant/antigravity/commands/boundline-status.md"),
        asset_path("assistant/copilot/prompts/boundline-status.prompt.md"),
        asset_path("assistant/claude/commands/boundline-inspect.md"),
        asset_path("assistant/codex/commands/boundline-inspect.md"),
        asset_path("assistant/antigravity/commands/boundline-inspect.md"),
        asset_path("assistant/copilot/prompts/boundline-inspect.prompt.md"),
    ];
    let required =
        ["planning_analysis_state", "planning_analysis_findings", "planning_analysis_coverage"];

    for path in assets {
        let content = read_asset(&path);
        assert_required_snippets(&path, &content, &required);
    }
}

#[test]
fn large_codebase_context_contract_sections_are_present_in_assistant_assets() {
    let assets = [
        asset_path("assistant/claude/commands/boundline-plan.md"),
        asset_path("assistant/codex/commands/boundline-plan.md"),
        asset_path("assistant/antigravity/commands/boundline-plan.md"),
        asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
        asset_path("assistant/claude/commands/boundline-run.md"),
        asset_path("assistant/codex/commands/boundline-run.md"),
        asset_path("assistant/antigravity/commands/boundline-run.md"),
        asset_path("assistant/copilot/prompts/boundline-run.prompt.md"),
        asset_path("assistant/claude/commands/boundline-status.md"),
        asset_path("assistant/codex/commands/boundline-status.md"),
        asset_path("assistant/antigravity/commands/boundline-status.md"),
        asset_path("assistant/copilot/prompts/boundline-status.prompt.md"),
        asset_path("assistant/claude/commands/boundline-inspect.md"),
        asset_path("assistant/codex/commands/boundline-inspect.md"),
        asset_path("assistant/antigravity/commands/boundline-inspect.md"),
        asset_path("assistant/copilot/prompts/boundline-inspect.prompt.md"),
    ];
    let required = [
        "repository_map_state",
        "snapshot_cache_state",
        "context_pack_entries",
        "omission_findings",
        "patch_safe_edit_attempts",
    ];

    for path in assets {
        let content = read_asset(&path);
        assert_required_snippets(&path, &content, &required);
    }
}

#[test]
fn backlog_quality_contract_sections_are_present_in_assistant_assets() {
    let plan_and_run_assets = [
        asset_path("assistant/claude/commands/boundline-plan.md"),
        asset_path("assistant/codex/commands/boundline-plan.md"),
        asset_path("assistant/antigravity/commands/boundline-plan.md"),
        asset_path("assistant/copilot/prompts/boundline-plan.prompt.md"),
        asset_path("assistant/claude/commands/boundline-run.md"),
        asset_path("assistant/codex/commands/boundline-run.md"),
        asset_path("assistant/antigravity/commands/boundline-run.md"),
        asset_path("assistant/copilot/prompts/boundline-run.prompt.md"),
    ];
    let required = [
        "backlog_quality_state",
        "backlog_quality_findings",
        "backlog_task_count",
        "backlog_mvp_scope",
        "backlog_unmapped_items",
        "do not route to",
        "Canon backlog is governed source material",
    ];

    for path in plan_and_run_assets {
        let content = read_asset(&path);
        assert_required_snippets(&path, &content, &required);
    }
}

#[test]
fn test_step_run_status_and_next_definition_sections_and_backend_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-step.md"),
            &["boundline step --workspace <workspace>", "latest_trace_ref", "next_command"][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-step.md"),
            &["boundline step --workspace <workspace>", "latest_trace_ref", "next_command"][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-step.prompt.md"),
            &["boundline step --workspace <workspace>", "latest_trace_ref", "next_command"][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-run.md"),
            &[
                "boundline orchestrate --workspace <workspace> --until terminal --json-stream",
                "phase_request",
                "resume_command",
                "next_command",
                "governance wait-or-block guidance",
                "/boundline-inspect",
            ][..],
            &["boundline run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-run.md"),
            &[
                "boundline orchestrate --workspace <workspace> --until terminal --json-stream",
                "phase_request",
                "resume_command",
                "next_command",
                "governance wait-or-block guidance",
                "/boundline-inspect",
            ][..],
            &["boundline run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-run.prompt.md"),
            &[
                "boundline run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/boundline-inspect",
            ][..],
            &["boundline run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-status.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-status.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/antigravity/commands/boundline-status.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-status.prompt.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-next.md"),
            &[
                "boundline next --workspace <workspace>",
                "latest_trace_ref",
                "follow_through_guidance",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-next.md"),
            &[
                "boundline next --workspace <workspace>",
                "latest_trace_ref",
                "follow_through_guidance",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-next.prompt.md"),
            &[
                "boundline next --workspace <workspace>",
                "latest_trace_ref",
                "follow_through_guidance",
                "next_command",
            ][..],
            &["boundline inspect --workspace <workspace>"][..],
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
fn status_definition_sections_and_probe_preflight_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-status.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline init",
                "/boundline-doctor",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-status.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline init",
                "/boundline-doctor",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/antigravity/commands/boundline-status.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline init",
                "/boundline-doctor",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-status.prompt.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline init",
                "/boundline-doctor",
                "boundline status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "next_command",
            ][..],
        ),
    ];

    for (path, required) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, required);
    }
}

#[test]
fn recover_definition_sections_and_backend_mappings() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-recover.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace> --json",
                "latest_checkpoint_restore_command",
                "corrected_command",
                "next_command",
                "/boundline-inspect",
                "/boundline-goal",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-recover.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace> --json",
                "latest_checkpoint_restore_command",
                "corrected_command",
                "next_command",
                "/boundline-inspect",
                "/boundline-goal",
            ][..],
        ),
        (
            asset_path("assistant/antigravity/commands/boundline-recover.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace> --json",
                "latest_checkpoint_restore_command",
                "corrected_command",
                "next_command",
                "/boundline-inspect",
                "/boundline-goal",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-recover.prompt.md"),
            &[
                "boundline probe --workspace <workspace> --json",
                "boundline status --workspace <workspace> --json",
                "latest_checkpoint_restore_command",
                "corrected_command",
                "next_command",
                "/boundline-inspect",
                "/boundline-goal",
            ][..],
        ),
    ];

    for (path, required) in assets {
        let content = read_asset(&path);
        assert_required_sections(&path, &content);
        assert_required_snippets(&path, &content, required);
    }
}

#[test]
fn session_step_action_routing_uses_host_native_buttons_and_runtime_precedence() {
    let target_commands = ["goal", "plan", "step", "next", "run"];

    for command in target_commands {
        let copilot_path =
            asset_path(&format!("assistant/copilot/prompts/boundline-{command}.prompt.md"));
        let copilot = read_asset(&copilot_path);
        assert_required_snippets(
            &copilot_path,
            &copilot,
            &[
                "command:github.copilot.chat.execute",
                "assistant_resume_command",
                "assistant_next_command",
                "next_command",
            ],
        );

        for host in ["claude", "codex", "antigravity"] {
            let path = asset_path(&format!("assistant/{host}/commands/boundline-{command}.md"));
            let content = read_asset(&path);
            assert_required_snippets(
                &path,
                &content,
                &[
                    "host-native",
                    "/boundline:*",
                    "assistant_resume_command",
                    "assistant_next_command",
                    "next_command",
                ],
            );
            assert_forbidden_snippets(&path, &content, &["command:github.copilot.chat.execute"]);
        }
    }
}

#[test]
fn test_inspect_definition_sections_and_trace_read_failure_expectations() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-inspect.md"),
            &[
                "boundline inspect --trace <trace>",
                "boundline inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "corrected_command",
                "/boundline-goal",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-inspect.md"),
            &[
                "boundline inspect --trace <trace>",
                "boundline inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "corrected_command",
                "/boundline-goal",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-inspect.prompt.md"),
            &[
                "boundline inspect --trace <trace>",
                "boundline inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "follow_through_guidance",
                "follow_through_evidence_source",
                "corrected_command",
                "/boundline-goal",
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
fn reasoning_profile_fields_are_documented_in_delight_command_packs() {
    let assets = [
        asset_path("assistant/claude/commands/boundline-why.md"),
        asset_path("assistant/codex/commands/boundline-why.md"),
        asset_path("assistant/copilot/prompts/boundline-why.prompt.md"),
        asset_path("assistant/claude/commands/boundline-challenge.md"),
        asset_path("assistant/codex/commands/boundline-challenge.md"),
        asset_path("assistant/copilot/prompts/boundline-challenge.prompt.md"),
        asset_path("assistant/claude/commands/boundline-explain-plan.md"),
        asset_path("assistant/codex/commands/boundline-explain-plan.md"),
        asset_path("assistant/copilot/prompts/boundline-explain-plan.prompt.md"),
    ];

    for path in assets {
        let content = read_asset(&path);
        assert_required_snippets(
            &path,
            &content,
            &[
                "reasoning_selection_reason",
                "reasoning_contribution",
                "reasoning_fallback_disclosure",
            ],
        );
    }
}

#[test]
fn cognitive_follow_up_definition_sections_and_backend_mappings_exist() {
    let assets = [
        (
            asset_path("assistant/claude/commands/boundline-assumptions.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "assumptions_summary",
                "assumption_group",
                "fallback_disclosure",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-assumptions.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "assumptions_summary",
                "assumption_group",
                "fallback_disclosure",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-assumptions.prompt.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "assumptions_summary",
                "assumption_group",
                "fallback_disclosure",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-hidden-impact.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "hidden_impact_summary",
                "hidden_impact_fallback_disclosure",
                "challenge_required_review",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-hidden-impact.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "hidden_impact_summary",
                "hidden_impact_fallback_disclosure",
                "challenge_required_review",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-hidden-impact.prompt.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "hidden_impact_summary",
                "hidden_impact_fallback_disclosure",
                "challenge_required_review",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-challenge.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "challenge_strongest_objection",
                "challenge_required_review",
                "challenge_council_required",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-challenge.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "challenge_strongest_objection",
                "challenge_required_review",
                "challenge_council_required",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-challenge.prompt.md"),
            &[
                "boundline inspect --workspace <workspace>",
                "challenge_strongest_objection",
                "challenge_required_review",
                "challenge_council_required",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/claude/commands/boundline-explain-plan.md"),
            &[
                "boundline status --workspace <workspace>",
                "explain_plan_summary",
                "explain_plan_validation",
                "explain_plan_governance",
                "explain_plan_recovery",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/boundline-explain-plan.md"),
            &[
                "boundline status --workspace <workspace>",
                "explain_plan_summary",
                "explain_plan_validation",
                "explain_plan_governance",
                "explain_plan_recovery",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/boundline-explain-plan.prompt.md"),
            &[
                "boundline status --workspace <workspace>",
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
