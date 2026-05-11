use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const CORE_COMMANDS: &[&str] = &[
    "boundline-start",
    "boundline-plan",
    "boundline-step",
    "boundline-run",
    "boundline-status",
    "boundline-next",
    "boundline-inspect",
];

const REQUIRED_COMMANDS: &[&str] = &[
    "boundline-start",
    "boundline-plan",
    "boundline-step",
    "boundline-run",
    "boundline-status",
    "boundline-next",
    "boundline-inspect",
    "boundline-workflow-list",
    "boundline-workflow-run",
    "boundline-workflow-status",
    "boundline-workflow-resume",
    "boundline-workflow-inspect",
    "boundline-init",
    "boundline-doctor",
    "boundline-config-show",
    "boundline-config-set-canon",
    "boundline-capture",
    "boundline-recover",
    "boundline-govern",
    "boundline-requirements",
    "boundline-discovery",
    "boundline-system-shaping",
    "boundline-architecture",
    "boundline-backlog",
    "boundline-change",
    "boundline-implementation",
    "boundline-refactor",
    "boundline-review",
    "boundline-verification",
    "boundline-incident",
    "boundline-security-assessment",
    "boundline-system-assessment",
    "boundline-migration",
    "boundline-supply-chain-analysis",
];

#[test]
fn test_asset_filenames_and_surfaces() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let claude_commands =
        command_names_from_dir(&manifest_dir.join("assistant/claude/commands"), ".md");
    let codex_commands =
        command_names_from_dir(&manifest_dir.join("assistant/codex/commands"), ".md");
    let copilot_commands =
        command_names_from_dir(&manifest_dir.join("assistant/copilot/prompts"), ".prompt.md");

    assert_eq!(claude_commands, codex_commands, "Claude and Codex command surfaces drifted");
    assert_eq!(
        claude_commands, copilot_commands,
        "Copilot prompt surface drifted from slash-command packs"
    );

    let expected: BTreeSet<String> =
        REQUIRED_COMMANDS.iter().map(|command| (*command).to_string()).collect();
    assert_eq!(
        claude_commands, expected,
        "assistant command surface should expose the full workflow-aware command pack"
    );
}

#[test]
fn test_shared_assistant_readme_covers_supported_surfaces_and_fallback_rules() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assistant/README.md");
    let readme = fs::read_to_string(&readme_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", readme_path.display()));

    assert!(readme.contains("Claude"), "{readme}");
    assert!(readme.contains("Codex"), "{readme}");
    assert!(readme.contains("Copilot"), "{readme}");
    assert!(readme.contains("Fallback Conventions"), "{readme}");
    assert!(readme.contains("Starting a Workflow (User Story 1)"), "{readme}");
    assert!(readme.contains("Continuing a Workflow (User Story 2)"), "{readme}");
    assert!(readme.contains("Inspecting Prior Runs (User Story 3)"), "{readme}");
    assert!(readme.contains("inspection_target"), "{readme}");
    assert!(readme.contains("continuity_authority"), "{readme}");
    assert!(readme.contains("compatibility_follow_up"), "{readme}");
    assert!(readme.contains("compatibility_follow_up_command"), "{readme}");
    assert!(readme.contains("corrected_command"), "{readme}");
    assert!(readme.contains("next_command"), "{readme}");
}

#[test]
fn test_cross_pack_assets_keep_expected_formatting_and_shared_guidance() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    for command in REQUIRED_COMMANDS {
        let claude_path = manifest_dir.join(format!("assistant/claude/commands/{command}.md"));
        let codex_path = manifest_dir.join(format!("assistant/codex/commands/{command}.md"));
        let copilot_path =
            manifest_dir.join(format!("assistant/copilot/prompts/{command}.prompt.md"));

        let claude = read_text(&claude_path);
        let codex = read_text(&codex_path);
        let copilot = read_text(&copilot_path);

        assert!(claude.starts_with(&format!("# Command: /{command}")), "{}", claude_path.display());
        assert!(codex.starts_with(&format!("# Command: /{command}")), "{}", codex_path.display());
        assert!(copilot.starts_with("---\n"), "{}", copilot_path.display());
        assert!(copilot.contains(&format!("# Command: /{command}")), "{}", copilot_path.display());

        assert!(
            claude.contains("Shared guidance: `assistant/README.md`"),
            "{}",
            claude_path.display()
        );
        assert!(
            codex.contains("Shared guidance: `assistant/README.md`"),
            "{}",
            codex_path.display()
        );
        assert!(
            copilot.contains("Shared guidance: `assistant/README.md`"),
            "{}",
            copilot_path.display()
        );
    }
}

#[test]
fn test_documented_flows_match_the_assistant_asset_surface() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assistant_readme = read_text(&manifest_dir.join("assistant/README.md"));
    let quickstart =
        read_text(&manifest_dir.join("specs/003-assistant-command-packs/quickstart.md"));

    for command in REQUIRED_COMMANDS {
        assert!(
            assistant_readme.contains(&format!("/{command}")),
            "assistant/README.md missing /{command}"
        );
    }

    for command in CORE_COMMANDS {
        assert!(quickstart.contains(&format!("/{command}")), "quickstart.md missing /{command}");
    }

    assert!(quickstart.contains("inspection_target"), "{quickstart}");
    assert!(quickstart.contains("corrected_command"), "{quickstart}");
    assert!(
        quickstart.contains(
            "cargo run --bin boundline -- inspect --trace \"$PWD/.boundline/traces/<task-id>.json\""
        ),
        "{quickstart}"
    );
    assert!(
        quickstart.contains("cargo run --bin boundline -- inspect --workspace \"$PWD\""),
        "{quickstart}"
    );
}

fn command_names_from_dir(root: &Path, suffix: &str) -> BTreeSet<String> {
    fs::read_dir(root)
        .unwrap_or_else(|error| {
            panic!("failed to read assistant asset root {}: {error}", root.display())
        })
        .map(|entry| entry.unwrap().path())
        .map(|path| strip_suffix(&path, suffix))
        .collect()
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn strip_suffix(path: &Path, suffix: &str) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_else(|| panic!("assistant asset path {} is not valid UTF-8", path.display()));
    file_name
        .strip_suffix(suffix)
        .unwrap_or_else(|| {
            panic!("assistant asset {file_name} does not end with expected suffix {suffix}")
        })
        .to_string()
}
