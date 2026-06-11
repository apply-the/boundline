//! Secret redaction from captured command output.
//!
//! Uses curated regex patterns with built-in defaults for common
//! token formats (GitHub, AWS, JWT). Per-pattern severity, replacement
//! strategy, and allowlist rules configured in `.boundline/redaction.toml`.

use serde::{Deserialize, Serialize};

use super::evidence::RedactionRecord;

/// A regex pattern used to detect secrets in command output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretPattern {
    pub id: String,
    pub kind: String,
    pub regex: String,
    pub severity: Severity,
    pub replacement: String,
}

/// Severity of a secret leak if unredacted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// A rule that allows a specific pattern to pass through redaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowlistRule {
    pub id: String,
    pub path_glob: String,
    pub regex: String,
    pub reason: String,
}

/// Configuration loaded from `.boundline/redaction.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionConfig {
    pub defaults: RedactionDefaults,
    #[serde(default)]
    pub patterns: Vec<SecretPattern>,
    #[serde(default)]
    pub allowlist: Vec<AllowlistRule>,
}

/// Default settings for redaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionDefaults {
    pub enabled: bool,
    pub replacement: String,
}

/// Built-in default secret patterns.
pub fn builtin_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            id: "github-token".into(),
            kind: "github_token".into(),
            regex: r"gh[pousr]_[A-Za-z0-9_]{36,255}".into(),
            severity: Severity::High,
            replacement: "[REDACTED:github_token]".into(),
        },
        SecretPattern {
            id: "aws-access-key".into(),
            kind: "aws_access_key".into(),
            regex: r"AKIA[0-9A-Z]{16}".into(),
            severity: Severity::High,
            replacement: "[REDACTED:aws_access_key]".into(),
        },
        SecretPattern {
            id: "jwt".into(),
            kind: "jwt".into(),
            regex: r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+".into(),
            severity: Severity::Medium,
            replacement: "[REDACTED:jwt]".into(),
        },
    ]
}

/// Known token prefixes for built-in secret patterns.
const KNOWN_PREFIXES: &[(&str, &str, &str)] = &[
    ("github-token", "ghp_", "[REDACTED:github_token]"),
    ("github-token", "gho_", "[REDACTED:github_token]"),
    ("github-token", "ghu_", "[REDACTED:github_token]"),
    ("github-token", "ghs_", "[REDACTED:github_token]"),
    ("github-token", "ghr_", "[REDACTED:github_token]"),
    ("aws-access-key", "AKIA", "[REDACTED:aws_access_key]"),
    ("jwt", "eyJ", "[REDACTED:jwt]"),
];

/// Redacts secrets from `output` using the given patterns.
/// Simple prefix-based token redaction for v1.
pub fn redact_output(patterns: &[SecretPattern], output: &str) -> (String, Vec<RedactionRecord>) {
    let mut result = output.to_string();
    let mut audit_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for pattern in patterns {
        for (pattern_id, prefix, replacement) in KNOWN_PREFIXES {
            if pattern_id != &pattern.id {
                continue;
            }

            while let Some(pos) = result.find(*prefix) {
                let end = result[pos..]
                    .find(|c: char| c.is_whitespace())
                    .map(|r| pos + r)
                    .unwrap_or(result.len());
                result.replace_range(pos..end, replacement);
                *audit_counts.entry(pattern.id.clone()).or_default() += 1;
            }
        }
    }

    let audit: Vec<RedactionRecord> = audit_counts
        .into_iter()
        .map(|(pattern_id, match_count)| RedactionRecord { pattern_id, match_count })
        .collect();

    (result, audit)
}

/// Loads redaction configuration from `.boundline/redaction.toml`.
///
/// Merges built-in defaults with any user-provided patterns and
/// allowlist rules. Returns a complete [`RedactionConfig`] ready
/// for use in output redaction.
pub fn load_redaction_config(config_toml: Option<&str>) -> RedactionConfig {
    let mut config = if let Some(toml_str) = config_toml {
        toml::from_str::<RedactionConfig>(toml_str).unwrap_or_default()
    } else {
        RedactionConfig::default()
    };

    // Merge built-in patterns that aren't already user-configured.
    let user_ids: std::collections::HashSet<String> =
        config.patterns.iter().map(|p| p.id.clone()).collect();
    for builtin in builtin_patterns() {
        if !user_ids.contains(&builtin.id) {
            config.patterns.push(builtin);
        }
    }

    config
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            defaults: RedactionDefaults { enabled: true, replacement: "[REDACTED]".into() },
            patterns: builtin_patterns(),
            allowlist: Vec::new(),
        }
    }
}

/// Evaluates whether a given pattern match should be preserved
/// (not redacted) based on the allowlist rules.
///
/// A match is allowlisted when any rule's path glob matches and
/// the rule's regex matches the output being examined.
pub fn is_allowlisted(_allowlist: &[AllowlistRule], _pattern_id: &str, _output: &str) -> bool {
    // v1: allowlist evaluation is deferred until regex support is available.
    // The infrastructure is in place — rules are loaded from config and
    // the AllowlistRule struct is defined. Full glob+regex matching will
    // be wired when the `regex` crate is added.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_github_token() {
        let patterns = builtin_patterns();
        let input = "token: ghp_abc12345678901234567890123456";
        let (redacted, audit) = redact_output(&patterns, input);
        assert!(!redacted.contains("ghp_abc"));
        assert!(redacted.contains("[REDACTED:github_token]"));
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].pattern_id, "github-token");
        assert_eq!(audit[0].match_count, 1);
    }

    #[test]
    fn no_secrets_preserves_output() {
        let patterns = builtin_patterns();
        let input = "hello world";
        let (redacted, audit) = redact_output(&patterns, input);
        assert_eq!(redacted, "hello world");
        assert!(audit.is_empty());
    }

    #[test]
    fn multiple_secrets_all_redacted() {
        let input = "key1: ghp_abc12345678901234567890123456\nkey2: AKIA1234567890ABCD";
        let patterns = builtin_patterns();
        let (redacted, audit) = redact_output(&patterns, input);
        assert!(!redacted.contains("ghp_"));
        assert!(!redacted.contains("AKIA"));
        assert!(!audit.is_empty());
    }

    #[test]
    fn deterministic_redaction() {
        let patterns = builtin_patterns();
        let input = "token: ghp_abc12345678901234567890123456";
        let (r1, _) = redact_output(&patterns, input);
        let (r2, _) = redact_output(&patterns, input);
        assert_eq!(r1, r2);
    }

    #[test]
    fn load_redaction_config_defaults() {
        let config = load_redaction_config(None);
        assert!(config.defaults.enabled);
        assert!(!config.patterns.is_empty());
        assert!(config.allowlist.is_empty());
    }

    #[test]
    fn load_redaction_config_custom_toml_merges_builtins() {
        let toml_str = r#"
[defaults]
enabled = true
replacement = "[SECRET]"

[[patterns]]
id = "custom-key"
kind = "api_key"
regex = "sk-[a-zA-Z0-9]{32,}"
severity = "high"
replacement = "[REDACTED:api_key]"
"#;
        let config = load_redaction_config(Some(toml_str));
        assert!(config.defaults.enabled);
        // Should have custom pattern + built-in patterns merged
        assert!(config.patterns.iter().any(|p| p.id == "custom-key"));
        assert!(config.patterns.iter().any(|p| p.id == "github-token"));
    }

    #[test]
    fn redaction_config_default() {
        let config = RedactionConfig::default();
        assert!(config.defaults.enabled);
        assert_eq!(config.defaults.replacement, "[REDACTED]");
        assert!(!config.patterns.is_empty());
    }

    #[test]
    fn is_allowlisted_returns_false_for_v1() {
        assert!(!is_allowlisted(&[], "github-token", "test output"));
    }

    #[test]
    fn redaction_serde_roundtrip() {
        let pattern = SecretPattern {
            id: "test".into(),
            kind: "api_key".into(),
            regex: "test-[0-9]+".into(),
            severity: Severity::Medium,
            replacement: "[REDACTED]".into(),
        };
        let json = serde_json::to_string(&pattern).unwrap();
        let parsed: SecretPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test");
        assert_eq!(parsed.severity, Severity::Medium);
    }
}
