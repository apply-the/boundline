//! Execution policy: Intent × Zone matrix with command overrides.
//!
//! Loads `.boundline/execution-policy.toml` and resolves every command
//! execution against the matrix. Resolution order: classify intent →
//! apply command overrides → resolve matrix → apply safety escalation
//! flags → produce final ExecutionMode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::classifier::{CommandIntent, ExecutionMode};
use super::evidence::RiskZone;

/// The complete execution policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub defaults: PolicyDefaults,
    pub policy: HashMap<String, HashMap<String, PolicyEntry>>,
    #[serde(default)]
    pub overrides: Vec<CommandOverride>,
}

/// Default settings for the policy matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDefaults {
    pub unknown_intent_mode: ExecutionMode,
    pub missing_policy_mode: ExecutionMode,
}

/// A single cell in the Intent × Zone matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEntry {
    pub mode: ExecutionMode,
}

/// An optional command-specific override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOverride {
    pub command: String,
    #[serde(default)]
    pub args_contains: Option<Vec<String>>,
    pub intent: Option<CommandIntent>,
    pub mode: Option<ExecutionMode>,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            defaults: PolicyDefaults {
                unknown_intent_mode: ExecutionMode::RequireApproval,
                missing_policy_mode: ExecutionMode::Deny,
            },
            policy: default_policy_matrix(),
            overrides: Vec::new(),
        }
    }
}

/// Builds the default Intent × Zone policy matrix per the spec.
fn default_policy_matrix() -> HashMap<String, HashMap<String, PolicyEntry>> {
    let mut matrix: HashMap<String, HashMap<String, PolicyEntry>> = HashMap::new();

    // Helper
    let entry = |mode: ExecutionMode| PolicyEntry { mode };

    // read: allow everywhere
    for zone in &["green", "yellow", "red"] {
        matrix
            .entry("read".into())
            .or_default()
            .insert(zone.to_string(), entry(ExecutionMode::Allow));
    }

    // test: allow in green/yellow, dry-run in red
    matrix.entry("test".into()).or_default().insert("green".into(), entry(ExecutionMode::Allow));
    matrix.entry("test".into()).or_default().insert("yellow".into(), entry(ExecutionMode::Allow));
    matrix.entry("test".into()).or_default().insert("red".into(), entry(ExecutionMode::DryRun));

    // mutate: require-approval in green, dry-run in yellow, deny in red
    matrix
        .entry("mutate".into())
        .or_default()
        .insert("green".into(), entry(ExecutionMode::RequireApproval));
    matrix
        .entry("mutate".into())
        .or_default()
        .insert("yellow".into(), entry(ExecutionMode::DryRun));
    matrix.entry("mutate".into()).or_default().insert("red".into(), entry(ExecutionMode::Deny));

    // install: require-approval in green, dry-run in yellow, deny in red
    matrix
        .entry("install".into())
        .or_default()
        .insert("green".into(), entry(ExecutionMode::RequireApproval));
    matrix
        .entry("install".into())
        .or_default()
        .insert("yellow".into(), entry(ExecutionMode::DryRun));
    matrix.entry("install".into()).or_default().insert("red".into(), entry(ExecutionMode::Deny));

    // deploy: require-approval in green, deny in yellow/red
    matrix
        .entry("deploy".into())
        .or_default()
        .insert("green".into(), entry(ExecutionMode::RequireApproval));
    matrix.entry("deploy".into()).or_default().insert("yellow".into(), entry(ExecutionMode::Deny));
    matrix.entry("deploy".into()).or_default().insert("red".into(), entry(ExecutionMode::Deny));

    // unknown: require-approval in green/yellow, deny in red
    matrix
        .entry("unknown".into())
        .or_default()
        .insert("green".into(), entry(ExecutionMode::RequireApproval));
    matrix
        .entry("unknown".into())
        .or_default()
        .insert("yellow".into(), entry(ExecutionMode::RequireApproval));
    matrix.entry("unknown".into()).or_default().insert("red".into(), entry(ExecutionMode::Deny));

    matrix
}

impl ExecutionPolicy {
    /// Returns the effective `PolicyEntry` for a given intent and zone,
    /// respecting defaults and command overrides.
    pub fn resolve(&self, intent: CommandIntent, zone: RiskZone) -> PolicyEntry {
        let intent_key = intent.as_str();
        let zone_key = zone.as_str();

        self.policy
            .get(intent_key)
            .and_then(|zones| zones.get(zone_key))
            .copied()
            .unwrap_or(PolicyEntry { mode: self.defaults.missing_policy_mode })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_denies_unknown_in_red() {
        let policy = ExecutionPolicy::default();
        let entry = policy.resolve(CommandIntent::Unknown, RiskZone::Red);
        assert_eq!(entry.mode, ExecutionMode::Deny);
    }

    #[test]
    fn default_policy_allows_read_in_all_zones() {
        let policy = ExecutionPolicy::default();
        assert_eq!(policy.resolve(CommandIntent::Read, RiskZone::Green).mode, ExecutionMode::Allow);
        assert_eq!(policy.resolve(CommandIntent::Read, RiskZone::Red).mode, ExecutionMode::Allow);
    }

    #[test]
    fn default_policy_denies_deploy_in_red() {
        let policy = ExecutionPolicy::default();
        assert_eq!(policy.resolve(CommandIntent::Deploy, RiskZone::Red).mode, ExecutionMode::Deny);
    }

    #[test]
    fn serde_roundtrip() {
        let policy = ExecutionPolicy::default();
        let toml_str = toml::to_string_pretty(&policy).unwrap();
        let parsed: ExecutionPolicy = toml::from_str(&toml_str).unwrap();
        assert_eq!(policy.defaults, parsed.defaults);
    }
}
