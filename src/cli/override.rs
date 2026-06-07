//! `boundline override` CLI command.
//!
//! Writes an explicit override record to `.boundline/overrides.json` that
//! `boundline run` and `boundline continue` consume before adjudication.
//! Overrides are tied to a specific finding, control, guardian, and operator.

use std::path::Path;

use crate::domain::calibration::{ControlLevel, OverrideRecord, load_calibration_policy};

/// Error type for the override CLI command.
#[derive(Debug, thiserror::Error)]
pub enum OverrideCliError {
    #[error("invalid control level '{0}': must be advisory, catch, or rule")]
    InvalidLevel(String),
    #[error("hook-level findings cannot be overridden via `boundline override`")]
    HookNotOverridable,
    #[error("missing required argument: {0}")]
    MissingArgument(String),
    #[error("failed to read calibration policy: {0}")]
    PolicyRead(String),
    #[error("failed to write override record: {0}")]
    WriteError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Run the override command.
pub fn run(
    workspace_root: &Path,
    guardian_id: &str,
    control_id: &str,
    level: &str,
    reason: &str,
    expiry: Option<&str>,
) -> Result<i32, OverrideCliError> {
    let requested_level = parse_level(level)?;

    // Hook bypass is not permitted via `boundline override`.
    if requested_level == ControlLevel::Hook {
        return Err(OverrideCliError::HookNotOverridable);
    }

    // Load calibration policy to check override policy.
    let policy = load_calibration_policy(workspace_root)
        .map_err(|e| OverrideCliError::PolicyRead(e.to_string()))?;

    let finding_id = uuid::Uuid::new_v4().to_string();
    let timestamp = {
        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        format!("{}", now.as_secs())
    };

    // Check if the override satisfies the configured policy.
    let satisfies_policy = policy.entries.iter().any(|entry| {
        entry.rule_id == guardian_id
            && (entry.override_policy.allowed_roles.is_empty()
                || entry.override_policy.allowed_roles.contains(&"operator".to_string()))
    }) || policy.entries.is_empty();

    let override_record = OverrideRecord {
        finding_id: finding_id.clone(),
        control_id: control_id.to_string(),
        guardian_id: guardian_id.to_string(),
        requested_level,
        reason: reason.to_string(),
        operator_identity: std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok()),
        timestamp: timestamp.clone(),
        expiry: expiry.map(|s| s.to_string()),
        satisfies_policy,
    };

    // Write the override record to `.boundline/overrides.json`.
    let overrides_path = workspace_root.join(".boundline").join("overrides.json");
    if let Some(parent) = overrides_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut existing: Vec<OverrideRecord> = if overrides_path.exists() {
        let content = std::fs::read_to_string(&overrides_path)
            .map_err(|e| OverrideCliError::WriteError(e.to_string()))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    existing.push(override_record);

    let json_content = serde_json::to_string_pretty(&existing)
        .map_err(|e| OverrideCliError::WriteError(e.to_string()))?;
    std::fs::write(&overrides_path, &json_content)?;

    println!(
        "Override record written for guardian '{}' (control: {}, level: {:?})",
        guardian_id, control_id, requested_level
    );
    println!("  Finding ID: {}", finding_id);
    println!("  Satisfies policy: {}", satisfies_policy);
    if !satisfies_policy {
        println!(
            "  Warning: override does not satisfy the configured policy and may be rejected at adjudication."
        );
    }

    Ok(0)
}

/// Parse a control level string into a `ControlLevel`.
fn parse_level(level: &str) -> Result<ControlLevel, OverrideCliError> {
    match level.to_lowercase().as_str() {
        "advisory" => Ok(ControlLevel::Advisory),
        "catch" => Ok(ControlLevel::Catch),
        "rule" => Ok(ControlLevel::Rule),
        "hook" => Ok(ControlLevel::Hook),
        other => Err(OverrideCliError::InvalidLevel(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_level_valid() {
        assert_eq!(parse_level("advisory").unwrap(), ControlLevel::Advisory);
        assert_eq!(parse_level("catch").unwrap(), ControlLevel::Catch);
        assert_eq!(parse_level("rule").unwrap(), ControlLevel::Rule);
        assert_eq!(parse_level("hook").unwrap(), ControlLevel::Hook);
    }

    #[test]
    fn parse_level_invalid() {
        assert!(parse_level("none").is_err());
        assert!(parse_level("block").is_err());
    }

    #[test]
    fn parse_level_case_insensitive() {
        assert_eq!(parse_level("ADVISORY").unwrap(), ControlLevel::Advisory);
        assert_eq!(parse_level("Catch").unwrap(), ControlLevel::Catch);
    }

    #[test]
    fn hook_bypass_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result = run(tmp.path(), "test-guardian", "ctrl-1", "hook", "reason", None);
        assert!(result.is_err());
    }

    #[test]
    fn write_override_with_existing_valid_overrides() {
        let tmp = tempfile::tempdir().unwrap();
        let overrides_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&overrides_dir).unwrap();
        let overrides_path = overrides_dir.join("overrides.json");
        std::fs::write(&overrides_path, "[{\"finding_id\":\"existing-1\",\"control_id\":\"ctrl-0\",\"guardian_id\":\"test-guardian\",\"requested_level\":\"advisory\",\"reason\":\"reason\",\"operator_identity\":null,\"timestamp\":\"123\",\"expiry\":null,\"satisfies_policy\":true}]").unwrap();

        let result = run(tmp.path(), "test-guardian", "ctrl-1", "catch", "new reason", None);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&overrides_path).unwrap();
        let records: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn write_override_with_existing_invalid_overrides() {
        let tmp = tempfile::tempdir().unwrap();
        let overrides_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&overrides_dir).unwrap();
        let overrides_path = overrides_dir.join("overrides.json");
        std::fs::write(&overrides_path, "not json").unwrap();

        let result = run(tmp.path(), "test-guardian", "ctrl-1", "catch", "reason", None);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&overrides_path).unwrap();
        let records: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(records.len(), 1); // The invalid JSON was replaced by an empty vector which then got the new record
    }

    #[test]
    fn write_override_violates_policy() {
        let tmp = tempfile::tempdir().unwrap();
        let overrides_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&overrides_dir).unwrap();
        let policy_path = overrides_dir.join("calibration-policy.toml");
        let policy_content = r#"
schema_version = "1.0"
evidence_window = 5
minimum_evidence_threshold = 3

[[entries]]
rule_id = "other-guardian"
authority_zone = "green"
risk_level = "low"
default_level = "catch"
green_level = "catch"
yellow_level = "rule"
red_level = "rule"
confidence_threshold = 0.85

[entries.override_policy]
allowed_roles = ["admin"]
required_evidence = ["security_review"]
time_limited = true
max_duration_hours = 24
"#;
        std::fs::write(&policy_path, policy_content).unwrap();

        // rule_id won't match, so it won't satisfy policy
        let result = run(tmp.path(), "test-guardian", "ctrl-1", "catch", "reason", None);
        assert!(result.is_ok());

        let overrides_path = overrides_dir.join("overrides.json");
        let content = std::fs::read_to_string(&overrides_path).unwrap();
        assert!(content.contains("\"satisfies_policy\": false"));
    }
}
