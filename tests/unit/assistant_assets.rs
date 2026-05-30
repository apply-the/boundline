use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

const CORE_COMMANDS: &[&str] = &[
    "boundline-goal",
    "boundline-plan",
    "boundline-step",
    "boundline-run",
    "boundline-status",
    "boundline-next",
    "boundline-inspect",
];

const REQUIRED_COMMANDS: &[&str] = &[
    "boundline-goal",
    "boundline-plan",
    "boundline-step",
    "boundline-run",
    "boundline-status",
    "boundline-next",
    "boundline-inspect",
    "boundline-update",
    "boundline-doctor",
    "boundline-config-show",
    "boundline-config-set-canon",
    "boundline-goal",
    "boundline-recover",
    "boundline-govern",
    "boundline-why",
    "boundline-risk",
    "boundline-evidence",
    "boundline-next-best",
    "boundline-assumptions",
    "boundline-hidden-impact",
    "boundline-challenge",
    "boundline-explain-plan",
    "boundline-doctor-context",
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
        "assistant command surface should expose the full session-native command pack"
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
fn assistant_assets_use_installed_cli_commands() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assistant_root = manifest_dir.join("assistant");
    let mut offenders = Vec::new();
    collect_cargo_run_references(&assistant_root, &assistant_root, &mut offenders);

    assert!(
        offenders.is_empty(),
        "assistant assets should use the installed `boundline` CLI, not repo-local Cargo commands: {}",
        offenders.join(", ")
    );
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
        quickstart.contains("boundline inspect --trace \"$PWD/.boundline/traces/<task-id>.json\""),
        "{quickstart}"
    );
    assert!(quickstart.contains("boundline inspect --workspace \"$PWD\""), "{quickstart}");
}

fn collect_cargo_run_references(root: &Path, base: &Path, offenders: &mut Vec<String>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| {
        panic!("failed to read assistant asset root {}: {error}", root.display())
    }) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_cargo_run_references(&path, base, offenders);
            continue;
        }

        let content = read_text(&path);
        if content.contains("cargo run --bin boundline --") {
            let relative = path.strip_prefix(base).unwrap_or(&path).display().to_string();
            offenders.push(relative);
        }
    }
}

#[test]
fn global_bootstrap_manifest_keeps_doctor_context_contextual() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = manifest_dir.join("assistant/global/manifest.json");
    let manifest: Value = serde_json::from_str(&read_text(&manifest_path))
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", manifest_path.display()));

    let commands = manifest["commands"]
        .as_array()
        .unwrap_or_else(|| panic!("{} missing commands array", manifest_path.display()))
        .iter()
        .map(|value| {
            value.as_str().unwrap_or_else(|| {
                panic!("{} command entries must be strings", manifest_path.display())
            })
        })
        .collect::<Vec<_>>();
    let contextual_commands = manifest["contextualCommands"]
        .as_array()
        .unwrap_or_else(|| panic!("{} missing contextualCommands array", manifest_path.display()))
        .iter()
        .map(|value| {
            value.as_str().unwrap_or_else(|| {
                panic!("{} contextual command entries must be strings", manifest_path.display())
            })
        })
        .collect::<Vec<_>>();

    assert!(
        !commands.contains(&"/boundline:doctor-context"),
        "global bootstrap palette should stay compact"
    );
    assert!(
        contextual_commands.contains(&"/boundline:doctor-context"),
        "doctor-context should stay available as a contextual global follow-up"
    );
    assert!(
        contextual_commands.contains(&"/boundline:explain-plan"),
        "explain-plan should also remain contextual rather than always visible"
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
