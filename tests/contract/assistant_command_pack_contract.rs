use std::path::{Path, PathBuf};

const US1_COMMANDS: &[&str] = &["synod-start", "synod-plan"];
const US2_COMMANDS: &[&str] = &["synod-step", "synod-run", "synod-status", "synod-next"];
const US3_COMMANDS: &[&str] = &["synod-inspect"];

#[test]
fn test_command_pack_covers_start_and_plan_commands() {
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
fn test_gemini_cli_fallback_notes_exist() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = manifest_dir.join("assistant/gemini/README.md");
    assert!(readme.is_file(), "missing gemini fallback notes: {}", readme.display());
}

fn asset_path(root: &Path, command: &str, suffix: &str) -> PathBuf {
    root.join(format!("{command}{suffix}"))
}

fn assert_pack_commands_exist(commands: &[&str]) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let packs = [
        ("claude", manifest_dir.join("assistant/claude/commands"), ".md"),
        ("codex", manifest_dir.join("assistant/codex/commands"), ".md"),
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
