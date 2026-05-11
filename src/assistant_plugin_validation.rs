use serde_json::Value;
use std::path::Path;

pub const REQUIRED_COMMANDS: &[&str] = &[
    "/boundline:start",
    "/boundline:capture",
    "/boundline:plan",
    "/boundline:run",
    "/boundline:status",
    "/boundline:inspect",
    "/boundline:recover",
    "/boundline:govern",
];

pub const REQUIRED_METADATA_FIELDS: &[&str] = &[
    "name",
    "displayName",
    "version",
    "description",
    "author",
    "homepage",
    "repository",
    "license",
    "keywords",
    "capabilities",
];

pub const PROHIBITED_POSITIONING: &[&str] = &[
    "agent framework",
    "debate cli",
    "prompt library",
    "governance runtime",
    "replacement for canon",
    "raw cli",
];

pub fn workspace_version_from_toml(cargo_toml: &str) -> Result<String, String> {
    let parsed: toml::Value =
        toml::from_str(cargo_toml).map_err(|error| format!("Cargo.toml parse error: {error}"))?;
    parsed["workspace"]["package"]["version"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "workspace package version must be a string".to_string())
}

pub fn string_array<'a>(value: &'a Value, field: &str) -> Result<Vec<&'a str>, String> {
    let entries = value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{field} must be an array"))?;

    entries
        .iter()
        .map(|entry| entry.as_str().ok_or_else(|| format!("{field} entries must be strings")))
        .collect()
}

pub fn capability_ids(value: &Value) -> Result<Vec<String>, String> {
    id_array(value, "capabilities", "capability")
}

pub fn command_ids(value: &Value) -> Result<Vec<String>, String> {
    id_array(value, "commands", "command")
}

fn id_array(value: &Value, field: &str, label: &str) -> Result<Vec<String>, String> {
    let entries = value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{field} must be an array"))?;

    entries
        .iter()
        .map(|entry| {
            entry
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("{label} id must be a string"))
        })
        .collect()
}

pub fn string_contains_any(value: &Value, prohibited: &[&str]) -> Option<String> {
    match value {
        Value::String(text) => prohibited
            .iter()
            .find(|term| text.to_ascii_lowercase().contains(&term.to_ascii_lowercase()))
            .map(|term| (*term).to_string()),
        Value::Array(entries) => {
            entries.iter().find_map(|entry| string_contains_any(entry, prohibited))
        }
        Value::Object(map) => map.values().find_map(|entry| string_contains_any(entry, prohibited)),
        _ => None,
    }
}

pub fn manifest_errors(manifest: &Value, version: &str, root: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    for field in REQUIRED_METADATA_FIELDS {
        if manifest.get(field).is_none() {
            errors.push(format!("missing required field: {field}"));
        }
    }

    if manifest.get("version").and_then(Value::as_str) != Some(version) {
        errors.push("manifest version does not match workspace version".to_string());
    }

    match capability_ids(manifest) {
        Ok(capabilities) => {
            for required in REQUIRED_COMMANDS {
                if !capabilities.iter().any(|capability| capability == required) {
                    errors.push(format!("missing required Boundline command: {required}"));
                }
            }
            for capability in capabilities {
                if !REQUIRED_COMMANDS.iter().any(|required| required == &capability) {
                    errors.push(format!("unsupported capability claim: {capability}"));
                }
            }
        }
        Err(error) => errors.push(error),
    }

    if let Some(paths) = manifest.get("paths").and_then(Value::as_object) {
        for value in paths.values() {
            let Some(path) = value.as_str() else {
                errors.push("path references must be strings".to_string());
                continue;
            };
            if !root.join(path).exists() {
                errors.push(format!("referenced path does not exist: {path}"));
            }
        }
    } else {
        errors.push("missing paths object".to_string());
    }

    if let Some(term) = string_contains_any(manifest, PROHIBITED_POSITIONING) {
        errors.push(format!("prohibited positioning term found: {term}"));
    }

    errors
}
