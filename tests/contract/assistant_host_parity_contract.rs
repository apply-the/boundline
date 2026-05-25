use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const HOST_SUPPORT_MODES: &[(&str, &str)] = &[
    ("claude", "repo-local-full"),
    ("codex", "repo-local-full"),
    ("cursor", "copy-ready-assets"),
    ("copilot", "repo-local-full"),
    ("antigravity", "repo-local-full"),
];

fn asset_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn read_json(relative_path: &str) -> Value {
    let path = asset_path(relative_path);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

#[test]
fn support_modes_are_aligned_across_manifest_metadata_and_workflow() {
    let global_manifest = read_json("assistant/global/manifest.json");
    let plugin_metadata = read_json("assistant/plugin-metadata.json");
    let workflow = read_json("assistant/commands/session-workflow.json");

    for (host, expected_mode) in HOST_SUPPORT_MODES {
        assert_eq!(
            global_manifest["support_modes"][host], *expected_mode,
            "global manifest drift for {host}"
        );
        assert_eq!(
            plugin_metadata["supportModes"][host], *expected_mode,
            "plugin metadata drift for {host}"
        );
        assert_eq!(workflow["hostSupportModes"][host], *expected_mode, "workflow drift for {host}");
    }
}

#[test]
fn cursor_and_antigravity_support_mode_notes_stay_explicit() {
    let global_manifest = read_json("assistant/global/manifest.json");
    let plugin_metadata = read_json("assistant/plugin-metadata.json");

    for (path, field, host, required_snippets) in [
        (
            "assistant/global/manifest.json",
            "support_mode_notes",
            "cursor",
            ["copy-ready", "CLI remains authoritative"],
        ),
        (
            "assistant/global/manifest.json",
            "support_mode_notes",
            "antigravity",
            ["repo-local", "manual fallback"],
        ),
        (
            "assistant/plugin-metadata.json",
            "supportModeNotes",
            "cursor",
            ["copy-ready", "CLI stays authoritative"],
        ),
        (
            "assistant/plugin-metadata.json",
            "supportModeNotes",
            "antigravity",
            ["repo-local", "manual fallback"],
        ),
    ] {
        let source = if path == "assistant/global/manifest.json" {
            &global_manifest
        } else {
            &plugin_metadata
        };
        let note = source[field][host]
            .as_str()
            .unwrap_or_else(|| panic!("{path} missing {field}.{host} string"));

        for snippet in required_snippets {
            assert!(note.contains(snippet), "{path} {field}.{host} missing {snippet}");
        }
    }
}
