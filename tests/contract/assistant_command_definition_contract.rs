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
            asset_path("assistant/claude/commands/synod-start.md"),
            &["cargo run --bin synod -- start --workspace <workspace>", "/synod-plan"][..],
            &["cargo run --bin synod -- doctor --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-start.md"),
            &["cargo run --bin synod -- start --workspace <workspace>", "/synod-plan"][..],
            &["cargo run --bin synod -- doctor --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-start.prompt.md"),
            &["cargo run --bin synod -- start --workspace <workspace>", "/synod-plan"][..],
            &["cargo run --bin synod -- doctor --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/claude/commands/synod-plan.md"),
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin synod -- plan --workspace <workspace>",
                "authored_input_summary",
                "/synod-step",
            ][..],
            &["No direct CLI invocation is required"][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-plan.md"),
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin synod -- plan --workspace <workspace>",
                "authored_input_summary",
                "/synod-step",
            ][..],
            &["No direct CLI invocation is required"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-plan.prompt.md"),
            &[
                "cargo run --bin synod -- capture --workspace <workspace> --goal \"<goal>\"",
                "cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]",
                "cargo run --bin synod -- plan --workspace <workspace>",
                "authored_input_summary",
                "/synod-step",
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
            asset_path("assistant/claude/commands/synod-step.md"),
            &[
                "cargo run --bin synod -- step --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-step.md"),
            &[
                "cargo run --bin synod -- step --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-step.prompt.md"),
            &[
                "cargo run --bin synod -- step --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["No direct CLI invocation is required by default"][..],
        ),
        (
            asset_path("assistant/claude/commands/synod-run.md"),
            &[
                "cargo run --bin synod -- run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/synod-inspect",
            ][..],
            &["cargo run --bin synod -- run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-run.md"),
            &[
                "cargo run --bin synod -- run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/synod-inspect",
            ][..],
            &["cargo run --bin synod -- run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-run.prompt.md"),
            &[
                "cargo run --bin synod -- run --workspace <workspace>",
                "next_command",
                "governance wait-or-block guidance",
                "/synod-inspect",
            ][..],
            &["cargo run --bin synod -- run --workspace <workspace> --goal \"<goal>\""][..],
        ),
        (
            asset_path("assistant/claude/commands/synod-status.md"),
            &[
                "cargo run --bin synod -- status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "next_command",
            ][..],
            &["cargo run --bin synod -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-status.md"),
            &[
                "cargo run --bin synod -- status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "next_command",
            ][..],
            &["cargo run --bin synod -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-status.prompt.md"),
            &[
                "cargo run --bin synod -- status --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "next_command",
            ][..],
            &["cargo run --bin synod -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/claude/commands/synod-next.md"),
            &[
                "cargo run --bin synod -- next --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["cargo run --bin synod -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-next.md"),
            &[
                "cargo run --bin synod -- next --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["cargo run --bin synod -- inspect --workspace <workspace>"][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-next.prompt.md"),
            &[
                "cargo run --bin synod -- next --workspace <workspace>",
                "latest_trace_ref",
                "next_command",
            ][..],
            &["cargo run --bin synod -- inspect --workspace <workspace>"][..],
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
            asset_path("assistant/claude/commands/synod-inspect.md"),
            &[
                "cargo run --bin synod -- inspect --trace <trace>",
                "cargo run --bin synod -- inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "corrected_command",
                "/synod-start",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/codex/commands/synod-inspect.md"),
            &[
                "cargo run --bin synod -- inspect --trace <trace>",
                "cargo run --bin synod -- inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "corrected_command",
                "/synod-start",
                "next_command",
            ][..],
        ),
        (
            asset_path("assistant/copilot/prompts/synod-inspect.prompt.md"),
            &[
                "cargo run --bin synod -- inspect --trace <trace>",
                "cargo run --bin synod -- inspect --workspace <workspace>",
                "latest_trace_ref",
                "authored_input_summary",
                "authored_input_sources",
                "authored_input_deduplicated_sources",
                "governance_next_action",
                "corrected_command",
                "/synod-start",
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
