use std::path::{Path, PathBuf};

const US1_COMMANDS: &[&str] = &["boundline-goal", "boundline-plan"];
const US2_COMMANDS: &[&str] =
    &["boundline-step", "boundline-run", "boundline-status", "boundline-next"];
const US3_COMMANDS: &[&str] = &["boundline-inspect"];
const DELIGHT_MVP_COMMANDS: &[&str] =
    &["boundline-why", "boundline-risk", "boundline-evidence", "boundline-next-best"];
const DELIGHT_FOLLOW_UP_COMMANDS: &[&str] = &[
    "boundline-assumptions",
    "boundline-hidden-impact",
    "boundline-challenge",
    "boundline-explain-plan",
];

#[test]
fn test_command_pack_covers_goal_and_plan_commands() {
    assert_pack_commands_exist(US1_COMMANDS);
}

#[test]
fn test_command_pack_covers_step_run_status_and_next_commands() {
    assert_pack_commands_exist(US2_COMMANDS);
}

#[test]
fn test_command_pack_covers_inspect_commands() {
    assert_pack_commands_exist(US3_COMMANDS);
}

#[test]
fn test_command_pack_covers_delight_mvp_commands() {
    assert_pack_commands_exist(DELIGHT_MVP_COMMANDS);
}

#[test]
fn delight_follow_up_command_pack_covers_cognitive_commands() {
    assert_pack_commands_exist(DELIGHT_FOLLOW_UP_COMMANDS);
}

#[test]
fn plan_and_status_assets_document_the_compact_operator_brief_contract() {
    assert_command_assets_contain(
        &["boundline-plan", "boundline-status"],
        "compact operator brief",
    );
    assert_command_assets_contain(&["boundline-plan", "boundline-status"], "execution_condition");
    assert_command_assets_contain(&["boundline-plan", "boundline-status"], "latest_status");
    assert_command_assets_contain(&["boundline-plan", "boundline-status"], "next_command");
}

#[test]
fn run_inspect_and_recover_assets_document_the_compact_operator_brief_contract() {
    assert_command_assets_contain(
        &["boundline-run", "boundline-inspect", "boundline-recover"],
        "compact operator brief",
    );
    assert_command_assets_contain(
        &["boundline-run", "boundline-inspect", "boundline-recover"],
        "execution_condition",
    );
    assert_command_assets_contain(
        &["boundline-run", "boundline-inspect", "boundline-recover"],
        "latest_status",
    );
    assert_command_assets_contain(
        &["boundline-run", "boundline-inspect", "boundline-recover"],
        "next_command",
    );
    assert_command_assets_contain(
        &["boundline-run", "boundline-inspect", "boundline-recover"],
        "--verbose",
    );
}

#[test]
fn init_assets_document_the_compact_operator_brief_contract() {
    assert_command_assets_contain(&["boundline-init"], "compact operator brief");
    assert_command_assets_contain(&["boundline-init"], "latest_status");
    assert_command_assets_contain(&["boundline-init"], "next_command");
    assert_command_assets_contain(&["boundline-init"], "--verbose");
}

#[test]
fn test_antigravity_repo_local_package_exists() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join(".antigravity-plugin/manifest.json");
    assert!(manifest.is_file(), "missing Antigravity package manifest: {}", manifest.display());
}

#[test]
fn test_host_support_modes_are_documented_in_shared_guidance() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(manifest_dir.join("assistant/README.md"))
        .expect("assistant README should be readable");

    for snippet in [
        "Cursor is `copy-ready-assets`",
        "Antigravity is `repo-local-full`",
        "all hosts must treat CLI output plus\n`.boundline/session.json` as authoritative",
    ] {
        assert!(readme.contains(snippet), "assistant/README.md missing {snippet}");
    }
}

fn asset_path(root: &Path, command: &str, suffix: &str) -> PathBuf {
    root.join(format!("{command}{suffix}"))
}

fn assert_pack_commands_exist(commands: &[&str]) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let packs = [
        ("claude", manifest_dir.join("assistant/claude/commands"), ".md"),
        ("codex", manifest_dir.join("assistant/codex/commands"), ".md"),
        ("antigravity", manifest_dir.join("assistant/antigravity/commands"), ".md"),
        ("copilot", manifest_dir.join("assistant/copilot/prompts"), ".prompt.md"),
    ];

    for (assistant, root, suffix) in packs {
        assert!(root.is_dir(), "missing {assistant} pack root: {}", root.display());

        for command in commands {
            let asset = asset_path(&root, command, suffix);
            assert!(
                asset.is_file(),
                "missing {assistant} asset for {command}: {}",
                asset.display()
            );
        }
    }
}

fn assert_command_assets_contain(commands: &[&str], needle: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let packs = [
        ("claude", manifest_dir.join("assistant/claude/commands"), ".md"),
        ("codex", manifest_dir.join("assistant/codex/commands"), ".md"),
        ("antigravity", manifest_dir.join("assistant/antigravity/commands"), ".md"),
        ("copilot", manifest_dir.join("assistant/copilot/prompts"), ".prompt.md"),
    ];

    for (assistant, root, suffix) in packs {
        for command in commands {
            let asset = asset_path(&root, command, suffix);
            let contents = std::fs::read_to_string(&asset).unwrap_or_else(|error| {
                panic!("failed to read {} asset {}: {error}", assistant, asset.display())
            });
            assert!(
                contents.contains(needle),
                "{assistant} asset for {command} missing {needle}: {}",
                asset.display()
            );
        }
    }
}
