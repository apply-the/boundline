use boundline::assistant_plugin_validation::{
    REQUIRED_COMMANDS, capability_ids, command_ids, manifest_errors, string_array,
    workspace_version_from_toml,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFESTS: &[&str] = &[
    ".claude-plugin/manifest.json",
    ".codex-plugin/plugin.json",
    ".cursor-plugin/manifest.json",
    ".copilot-prompts/pack.json",
];

const PACKAGE_FILES: &[&str] = &[
    ".claude-plugin/manifest.json",
    ".claude-plugin/commands.json",
    ".codex-plugin/plugin.json",
    ".cursor-plugin/manifest.json",
    ".cursor-plugin/commands.json",
    ".copilot-prompts/README.md",
    ".copilot-prompts/pack.json",
    "assistant/plugin-metadata.json",
    "assistant/commands/session-workflow.json",
    "assistant/prompts/starter-prompts.md",
    "assistant/prompts/copilot-command-pack.md",
    "assistant/assets/boundline-plugin-icon.svg",
    "assistant/assets/boundline-plugin-logo.svg",
    "docs/guides/assistant-plugin-packages.md",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_text(relative_path: &str) -> String {
    let path = repo_root().join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"))
}

fn read_json(relative_path: &str) -> Value {
    serde_json::from_str(&read_text(relative_path))
        .unwrap_or_else(|error| panic!("invalid JSON in {relative_path}: {error}"))
}

fn workspace_version() -> String {
    workspace_version_from_toml(&read_text("Cargo.toml")).expect("workspace version must parse")
}

#[test]
fn package_folders_and_docs_are_present() {
    let root = repo_root();
    for folder in
        [".claude-plugin", ".codex-plugin", ".cursor-plugin", ".copilot-prompts", "assistant"]
    {
        assert!(root.join(folder).is_dir(), "missing package folder {folder}");
    }
    for file in PACKAGE_FILES {
        assert!(root.join(file).is_file(), "missing package file {file}");
    }

    let guide = read_text("docs/guides/assistant-plugin-packages.md");
    for expected in [
        ".claude-plugin/",
        ".codex-plugin/",
        ".cursor-plugin/",
        ".copilot-prompts/",
        "assistant/prompts/copilot-command-pack.md",
        ".boundline/session.json remains authoritative",
    ] {
        assert!(guide.contains(expected), "guide must mention {expected}");
    }

    let readme = read_text("README.md");
    for expected in [
        "## Use Boundline from chat",
        "## Use Boundline from CLI",
        "## How chat commands map to CLI/runtime state",
    ] {
        assert!(readme.contains(expected), "README must include {expected}");
    }
}

#[test]
fn manifests_expose_required_boundline_commands() {
    let metadata = read_json("assistant/plugin-metadata.json");
    let commands = read_json("assistant/commands/session-workflow.json");
    let required_capabilities = capability_ids(&metadata).expect("metadata capabilities parse");
    let command_ids = command_ids(&commands).expect("command ids parse");

    for required in REQUIRED_COMMANDS {
        assert!(
            required_capabilities.iter().any(|capability| capability == required),
            "shared metadata must include {required}"
        );
        assert!(
            command_ids.iter().any(|command| command == required),
            "shared command definitions must include {required}"
        );
    }

    for manifest_path in MANIFESTS {
        let manifest = read_json(manifest_path);
        let ids = capability_ids(&manifest).expect("manifest capabilities parse");
        for required in REQUIRED_COMMANDS {
            assert!(ids.iter().any(|id| id == required), "{manifest_path} must include {required}");
        }
    }
}

#[test]
fn metadata_paths_and_versions_are_aligned() {
    let root = repo_root();
    let version = workspace_version();
    let metadata = read_json("assistant/plugin-metadata.json");

    assert_eq!(metadata["version"], version);
    assert_eq!(metadata["description"], "Local delivery orchestrator for bounded engineering work");

    for path in string_array(&metadata, "requiredPaths").expect("required paths parse") {
        assert!(root.join(path).exists(), "shared metadata path is missing: {path}");
    }

    for manifest_path in MANIFESTS {
        let manifest = read_json(manifest_path);
        assert!(
            manifest_errors(&manifest, &version, &root).is_empty(),
            "{manifest_path} failed validation"
        );
    }
}

#[test]
fn command_guidance_preserves_session_state() {
    let commands = read_text("assistant/commands/session-workflow.json");
    let guide = read_text("docs/guides/assistant-plugin-packages.md");
    let assistant_readme = read_text("assistant/README.md");
    let copilot_pack = read_text("assistant/prompts/copilot-command-pack.md");

    for text in [&commands, &guide, &assistant_readme, &copilot_pack] {
        assert!(text.contains(".boundline/session.json"), "missing session authority in {text}");
        assert!(text.contains("next_command"), "missing next_command guidance in {text}");
        for state in ["blocked", "clarification-required", "failed", "exhausted", "terminal"] {
            assert!(text.contains(state), "missing state {state} in {text}");
        }
    }

    assert!(guide.contains("Canon governance is conditional"));
    assert!(copilot_pack.contains("does not claim a universal Copilot plugin format"));
}

#[test]
fn validation_rejects_drift_and_prohibited_positioning() {
    assert!(serde_json::from_str::<Value>("{").is_err(), "invalid JSON must be rejected");

    let root = repo_root();
    let valid = json!({
        "name": "boundline",
        "displayName": "Boundline Assistant Support",
        "version": "0.49.1",
        "description": "Local delivery orchestrator for bounded engineering work",
        "author": {"name": "Apply The", "url": "https://github.com/apply-the"},
        "homepage": "https://github.com/apply-the/boundline",
        "repository": "https://github.com/apply-the/boundline",
        "license": "MIT",
        "keywords": ["boundline", "delivery"],
        "capabilities": REQUIRED_COMMANDS
            .iter()
            .map(|id| json!({"id": id, "label": id.trim_start_matches("/boundline:")}))
            .collect::<Vec<_>>(),
        "paths": {
            "commands": "assistant/commands/session-workflow.json",
            "starterPrompts": "assistant/prompts/starter-prompts.md"
        }
    });

    let mut missing_fields = valid.clone();
    missing_fields.as_object_mut().expect("valid manifest must be an object").remove("author");
    assert!(
        manifest_errors(&missing_fields, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("missing required field"))
    );

    let mut version_drift = valid.clone();
    version_drift["version"] = json!("0.0.0");
    assert!(
        manifest_errors(&version_drift, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("version"))
    );

    let mut missing_path = valid.clone();
    missing_path["paths"]["commands"] = json!("missing/commands.json");
    assert!(
        manifest_errors(&missing_path, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("referenced path"))
    );

    let mut non_string_path = valid.clone();
    non_string_path["paths"]["commands"] = json!(false);
    assert!(
        manifest_errors(&non_string_path, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("path references must be strings"))
    );

    let mut missing_paths = valid.clone();
    missing_paths.as_object_mut().expect("valid manifest must be an object").remove("paths");
    assert!(
        manifest_errors(&missing_paths, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("missing paths object"))
    );

    let mut missing_command = valid.clone();
    missing_command["capabilities"] = json!([]);
    assert!(
        manifest_errors(&missing_command, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("missing required Boundline command"))
    );

    let mut invalid_capability = valid.clone();
    invalid_capability["capabilities"] = json!([{"label": "missing id"}]);
    assert!(
        manifest_errors(&invalid_capability, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("capability id must be a string"))
    );

    let mut unsupported_capability = valid.clone();
    unsupported_capability["capabilities"] = json!([{"id": "generic-agent", "label": "agent"}]);
    assert!(
        manifest_errors(&unsupported_capability, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("unsupported capability"))
    );

    let mut prohibited = valid;
    prohibited["description"] = json!("Boundline is a generic agent framework");
    assert!(
        manifest_errors(&prohibited, "0.49.1", &root)
            .iter()
            .any(|error| error.contains("prohibited positioning"))
    );
}

#[test]
fn validation_helpers_report_malformed_inputs() {
    assert!(workspace_version_from_toml("not = [toml").is_err());
    assert!(workspace_version_from_toml("[workspace]\n[workspace.package]\nversion = 49").is_err());
    assert!(string_array(&json!({"requiredPaths": [1]}), "requiredPaths").is_err());
    assert!(string_array(&json!({}), "requiredPaths").is_err());
    assert!(command_ids(&json!({"commands": [{"label": "missing id"}]})).is_err());
    assert!(command_ids(&json!({})).is_err());
}

#[test]
fn starter_prompts_are_present() {
    let prompts = read_text("assistant/prompts/starter-prompts.md");
    for expected in [
        "I want to turn this idea into a bounded implementation plan.",
        "Help me fix a failing test with Boundline.",
        "Continue the active Boundline session.",
        "Inspect the latest Boundline trace and tell me the next safe action.",
    ] {
        assert!(prompts.contains(expected), "missing starter prompt: {expected}");
    }
}

#[test]
fn command_reference_paths_exist() {
    let commands = read_json("assistant/commands/session-workflow.json");
    let root = repo_root();
    let entries = commands["commands"].as_array().expect("commands must be an array");
    for entry in entries {
        let refs = entry["skillRefs"].as_object().expect("skillRefs must be an object");
        for value in refs.values() {
            let path = value.as_str().expect("skill ref must be a string");
            assert!(Path::new(path).is_relative(), "skill ref must be relative: {path}");
            assert!(root.join(path).is_file(), "skill ref is missing: {path}");
        }
    }
}

#[test]
fn s7_mvp_commands_are_registered_in_shared_metadata() {
    let metadata = read_json("assistant/plugin-metadata.json");
    let commands = read_json("assistant/commands/session-workflow.json");
    let capability_ids = capability_ids(&metadata).expect("metadata capabilities parse");
    let command_ids = command_ids(&commands).expect("command ids parse");

    for expected in
        ["/boundline:why", "/boundline:risk", "/boundline:evidence", "/boundline:next-best"]
    {
        assert!(
            capability_ids.iter().any(|capability| capability == expected),
            "shared metadata must include {expected}"
        );
        assert!(
            command_ids.iter().any(|command| command == expected),
            "shared commands must include {expected}"
        );
    }
}

#[test]
fn s7_us2_commands_are_registered_in_shared_metadata() {
    let metadata = read_json("assistant/plugin-metadata.json");
    let commands = read_json("assistant/commands/session-workflow.json");
    let capability_ids = capability_ids(&metadata).expect("metadata capabilities parse");
    let command_ids = command_ids(&commands).expect("command ids parse");

    for expected in [
        "/boundline:assumptions",
        "/boundline:hidden-impact",
        "/boundline:challenge",
        "/boundline:explain-plan",
    ] {
        assert!(
            capability_ids.iter().any(|capability| capability == expected),
            "shared metadata must include {expected}"
        );
        assert!(
            command_ids.iter().any(|command| command == expected),
            "shared commands must include {expected}"
        );
    }
}
