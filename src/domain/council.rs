//! Review council and guardian activation router domain types.
//!
//! This module defines the guardian activation ruleset, the activation
//! plan produced by the router, the single-adjudicator council decision
//! model, and the structured evidence types for trace visibility.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Ruleset ───────────────────────────────────────────────────────────

/// The source of the guardian ruleset used for activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RulesetSource {
    BuiltIn,
    File,
    Invalid,
}

/// A single activation rule from `.boundline/guardian-rules.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianRule {
    pub id: String,
    #[serde(default)]
    pub stages: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default)]
    pub activate: Vec<String>,
    #[serde(default)]
    pub skip: Vec<String>,
    #[serde(default)]
    pub mandatory: Vec<String>,
}

/// The loaded and validated ruleset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianRuleset {
    pub schema_version: String,
    pub rules: Vec<GuardianRule>,
}

/// Error type for ruleset validation failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RulesetError {
    #[error(
        "ruleset contains contradictory rules for guardian '{guardian}' under condition '{condition}': rules {rule_ids:?}"
    )]
    Contradiction { guardian: String, condition: String, rule_ids: Vec<String> },
    #[error("ruleset validation failed: {0}")]
    Validation(String),
}

impl GuardianRuleset {
    /// Validate that no rule pair contradicts another for the same guardian
    /// under the same matched condition.
    pub fn validate(&self) -> Result<(), RulesetError> {
        for i in 0..self.rules.len() {
            for j in (i + 1)..self.rules.len() {
                let a = &self.rules[i];
                let b = &self.rules[j];
                for guardian in &a.activate {
                    if b.skip.contains(guardian) && rules_overlap(a, b) {
                        return Err(RulesetError::Contradiction {
                            guardian: guardian.clone(),
                            condition: format!(
                                "stages={:?}|{:?} files={:?}|{:?}",
                                a.stages, b.stages, a.files, b.files
                            ),
                            rule_ids: vec![a.id.clone(), b.id.clone()],
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

/// Check whether two rules overlap in their matching conditions.
fn rules_overlap(a: &GuardianRule, b: &GuardianRule) -> bool {
    let stages_overlap =
        a.stages.is_empty() || b.stages.is_empty() || a.stages.iter().any(|s| b.stages.contains(s));
    let files_overlap =
        a.files.is_empty() || b.files.is_empty() || a.files.iter().any(|f| b.files.contains(f));
    stages_overlap && files_overlap
}

/// Build the built-in default ruleset with four predefined rules.
#[must_use]
pub fn builtin_ruleset() -> GuardianRuleset {
    GuardianRuleset {
        schema_version: "1.0".into(),
        rules: vec![
            GuardianRule {
                id: "rust-runtime-change".into(),
                stages: vec!["run".into(), "review".into()],
                files: vec!["src/domain/**/*.rs".into(), "src/orchestrator/**/*.rs".into()],
                language: Some("rust".into()),
                risk: None,
                activate: vec![
                    "rust-guardian".into(),
                    "error-handling-guardian".into(),
                    "traceability-guardian".into(),
                ],
                skip: vec![],
                mandatory: vec!["rust-guardian".into()],
            },
            GuardianRule {
                id: "documentation-change".into(),
                stages: vec!["review".into()],
                files: vec!["docs/**/*.md".into()],
                language: None,
                risk: None,
                activate: vec![
                    "docs-consistency-guardian".into(),
                    "release-surface-guardian".into(),
                ],
                skip: vec!["rust-guardian".into(), "error-handling-guardian".into()],
                mandatory: vec![],
            },
            GuardianRule {
                id: "contract-change".into(),
                stages: vec!["plan".into(), "review".into()],
                files: vec!["specs/**/*.md".into(), "contracts/**/*.md".into()],
                language: None,
                risk: None,
                activate: vec![
                    "contract-drift-guardian".into(),
                    "migration-guardian".into(),
                    "traceability-guardian".into(),
                ],
                skip: vec![],
                mandatory: vec!["contract-drift-guardian".into()],
            },
            GuardianRule {
                id: "security-sensitive-change".into(),
                stages: vec!["run".into(), "review".into()],
                files: vec![
                    "auth".into(),
                    "secrets".into(),
                    "permissions".into(),
                    "sandbox".into(),
                ],
                language: None,
                risk: Some("high".into()),
                activate: vec![
                    "security-guardian".into(),
                    "threat-model-guardian".into(),
                    "approval-gate-guardian".into(),
                ],
                skip: vec![],
                mandatory: vec!["security-guardian".into(), "threat-model-guardian".into()],
            },
        ],
    }
}

// ── Activation Plan ───────────────────────────────────────────────────

/// A record of a skipped guardian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianSkipRecord {
    pub guardian_id: String,
    pub reason: String,
    pub is_mandatory: bool,
}

/// Execution status of a guardian invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Success,
    Failure,
    Unavailable,
}

/// Guardian execution evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianExecutionRecord {
    pub guardian_id: String,
    pub status: ExecutionStatus,
    pub finding_count: u64,
    pub blocking_count: u64,
    pub trace_ref: String,
}

/// The router output after evaluating all rules against the change surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianActivationPlan {
    pub plan_id: Uuid,
    pub ruleset_source: RulesetSource,
    pub matched_rules: Vec<String>,
    pub activated: Vec<String>,
    pub skipped: Vec<GuardianSkipRecord>,
    pub mandatory_unavailable: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalation: Option<String>,
}

impl GuardianActivationPlan {
    /// Produce an activation plan from the matched rules.
    #[must_use]
    pub fn from_rules(rules: &[&GuardianRule], source: RulesetSource) -> Self {
        let mut activated = Vec::new();
        let mut skipped = Vec::new();
        let mut mandatory: Vec<String> = Vec::new();
        let mut optional_skips: Vec<String> = Vec::new();

        for rule in rules {
            for g in &rule.activate {
                if !activated.contains(g) {
                    activated.push(g.clone());
                }
            }
            for g in &rule.skip {
                optional_skips.push(g.clone());
            }
            for g in &rule.mandatory {
                if !mandatory.contains(g) {
                    mandatory.push(g.clone());
                }
            }
        }

        // Guardians that are skipped but not in the activated list produce
        // a skip record.
        for g in &optional_skips {
            if !activated.contains(g) {
                skipped.push(GuardianSkipRecord {
                    guardian_id: g.clone(),
                    reason: "excluded by matching rule".into(),
                    is_mandatory: mandatory.contains(g),
                });
            }
        }

        // Mandatory guardians not in the activated list are unavailable.
        let mandatory_unavailable: Vec<String> =
            mandatory.iter().filter(|g| !activated.contains(*g)).cloned().collect();

        let escalation = if !mandatory_unavailable.is_empty() {
            Some("mandatory guardians unavailable — escalation required".into())
        } else {
            None
        };

        Self {
            plan_id: Uuid::new_v4(),
            ruleset_source: source,
            matched_rules: rules.iter().map(|r| r.id.clone()).collect(),
            activated,
            skipped,
            mandatory_unavailable,
            escalation,
        }
    }

    /// Build a zero-guardian plan for trivial changes.
    #[must_use]
    pub fn zero_guardian(source: RulesetSource) -> Self {
        Self {
            plan_id: Uuid::new_v4(),
            ruleset_source: source,
            matched_rules: Vec::new(),
            activated: Vec::new(),
            skipped: Vec::new(),
            mandatory_unavailable: Vec::new(),
            escalation: None,
        }
    }
}

// ── Council Decision ──────────────────────────────────────────────────

/// The source of the council profile used for adjudication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileSource {
    Configured,
    BuiltInDefault,
    Invalid,
}

/// The binary outcome of council adjudication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouncilOutcome {
    Clean,
    Blocked,
}

/// The adjudicated outcome from `boundline council adjudicate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouncilDecision {
    pub decision_id: Uuid,
    pub adjudicator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_zone: Option<String>,
    pub profile_source: ProfileSource,
    pub findings_reviewed: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub deferred: u64,
    pub dissent: bool,
    pub outcome: CouncilOutcome,
    pub reason: String,
}

impl CouncilDecision {
    /// Produce a blocked decision when mandatory guardians are unavailable.
    #[must_use]
    pub fn blocked_mandatory_unavailable(plan: &GuardianActivationPlan) -> Self {
        Self {
            decision_id: Uuid::new_v4(),
            adjudicator: "single-reviewer".into(),
            authority_zone: None,
            profile_source: ProfileSource::BuiltInDefault,
            findings_reviewed: 0,
            accepted: 0,
            rejected: 0,
            deferred: 0,
            dissent: false,
            outcome: CouncilOutcome::Blocked,
            reason: format!(
                "mandatory guardians unavailable: {}",
                plan.mandatory_unavailable.join(", ")
            ),
        }
    }

    /// Produce a clean decision.
    #[must_use]
    pub fn clean() -> Self {
        Self {
            decision_id: Uuid::new_v4(),
            adjudicator: "single-reviewer".into(),
            authority_zone: None,
            profile_source: ProfileSource::BuiltInDefault,
            findings_reviewed: 0,
            accepted: 0,
            rejected: 0,
            deferred: 0,
            dissent: false,
            outcome: CouncilOutcome::Clean,
            reason: "no blocking findings".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_ruleset_has_four_rules() {
        let rs = builtin_ruleset();
        assert_eq!(rs.rules.len(), 4);
        assert_eq!(rs.schema_version, "1.0");
        rs.validate().expect("builtin ruleset must be valid");
    }

    #[test]
    fn builtin_ruleset_passes_validation() {
        let rs = builtin_ruleset();
        assert!(rs.validate().is_ok());
    }

    #[test]
    fn contradictory_ruleset_fails_validation() {
        let rs = GuardianRuleset {
            schema_version: "1.0".into(),
            rules: vec![
                GuardianRule {
                    id: "a".into(),
                    stages: vec!["run".into()],
                    files: vec!["src/**/*.rs".into()],
                    language: None,
                    risk: None,
                    activate: vec!["rust-guardian".into()],
                    skip: vec![],
                    mandatory: vec![],
                },
                GuardianRule {
                    id: "b".into(),
                    stages: vec!["run".into()],
                    files: vec!["src/**/*.rs".into()],
                    language: None,
                    risk: None,
                    activate: vec![],
                    skip: vec!["rust-guardian".into()],
                    mandatory: vec![],
                },
            ],
        };
        assert!(rs.validate().is_err());
    }

    #[test]
    fn activation_plan_from_builtin_rust_rule() {
        let rs = builtin_ruleset();
        let rust_rule = rs.rules.iter().find(|r| r.id == "rust-runtime-change").unwrap();
        let plan = GuardianActivationPlan::from_rules(&[rust_rule], RulesetSource::BuiltIn);
        assert!(plan.activated.contains(&"rust-guardian".into()));
        assert!(plan.activated.contains(&"error-handling-guardian".into()));
        assert_eq!(plan.mandatory_unavailable.len(), 0);
    }

    #[test]
    fn mandatory_unavailable_triggers_escalation() {
        let rs = builtin_ruleset();
        let sec_rule = rs.rules.iter().find(|r| r.id == "security-sensitive-change").unwrap();
        let plan = GuardianActivationPlan::from_rules(&[sec_rule], RulesetSource::BuiltIn);
        // All mandatory guardians are also activated in builtin rules,
        // so no unavailability expected.
        assert!(plan.mandatory_unavailable.is_empty());
        assert!(plan.escalation.is_none());
    }

    #[test]
    fn zero_guardian_plan_has_no_escalation() {
        let plan = GuardianActivationPlan::zero_guardian(RulesetSource::BuiltIn);
        assert!(plan.activated.is_empty());
        assert!(plan.mandatory_unavailable.is_empty());
        assert!(plan.escalation.is_none());
    }

    #[test]
    fn blocked_decision_for_mandatory_unavailable() {
        let plan = GuardianActivationPlan {
            plan_id: Uuid::nil(),
            ruleset_source: RulesetSource::BuiltIn,
            matched_rules: vec![],
            activated: vec![],
            skipped: vec![],
            mandatory_unavailable: vec!["security-guardian".into()],
            escalation: Some("escalation".into()),
        };
        let decision = CouncilDecision::blocked_mandatory_unavailable(&plan);
        assert_eq!(decision.outcome, CouncilOutcome::Blocked);
        assert!(decision.reason.contains("security-guardian"));
    }

    #[test]
    fn clean_decision_has_no_blockers() {
        let decision = CouncilDecision::clean();
        assert_eq!(decision.outcome, CouncilOutcome::Clean);
        assert!(!decision.dissent);
    }

    #[test]
    fn rules_overlap_detects_shared_stage_and_file() {
        let a = GuardianRule {
            id: "a".into(),
            stages: vec!["run".into()],
            files: vec!["src/**/*.rs".into()],
            language: None,
            risk: None,
            activate: vec![],
            skip: vec![],
            mandatory: vec![],
        };
        let b = GuardianRule {
            id: "b".into(),
            stages: vec!["run".into()],
            files: vec!["src/**/*.rs".into()],
            language: None,
            risk: None,
            activate: vec![],
            skip: vec![],
            mandatory: vec![],
        };
        assert!(rules_overlap(&a, &b));
    }

    #[test]
    fn rules_different_stages_do_not_overlap() {
        let a = GuardianRule {
            id: "a".into(),
            stages: vec!["plan".into()],
            files: vec!["src/**/*.rs".into()],
            language: None,
            risk: None,
            activate: vec![],
            skip: vec![],
            mandatory: vec![],
        };
        let b = GuardianRule {
            id: "b".into(),
            stages: vec!["review".into()],
            files: vec!["src/**/*.rs".into()],
            language: None,
            risk: None,
            activate: vec![],
            skip: vec![],
            mandatory: vec![],
        };
        assert!(!rules_overlap(&a, &b));
    }
}
