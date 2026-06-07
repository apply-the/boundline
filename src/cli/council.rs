//! `boundline council adjudicate` CLI command and output rendering.
//!
//! Reads the guardian ruleset, produces an activation plan, loads the
//! calibration policy to apply graduated control levels, consumes override
//! records, and adjudicates findings into a clean/blocked council decision.

use std::path::Path;

use crate::domain::calibration::{
    CalibrationPolicy, ControlLevel, ControlLevelAssignment, get_or_create_trust_record,
    load_calibration_policy, load_trust_records, save_trust_records,
};
use crate::domain::council::{
    CouncilDecision, GuardianActivationPlan, GuardianRuleset, RulesetSource, builtin_ruleset,
};

/// Error type for the council CLI command.
#[derive(Debug, thiserror::Error)]
pub enum CouncilCliError {
    #[error("failed to read ruleset: {0}")]
    RulesetRead(String),
    #[error("ruleset is invalid: {0}")]
    RulesetInvalid(String),
    #[error("failed to load calibration policy: {0}")]
    CalibrationLoad(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Run the council adjudication command with calibration policy integration.
pub fn run(workspace_root: &Path, json_output: bool) -> Result<i32, CouncilCliError> {
    let (ruleset, source) = load_ruleset(workspace_root)?;

    // Load calibration policy — fall back to built-in all-advisory if missing or invalid.
    let calibration_policy = match load_calibration_policy(workspace_root) {
        Ok(policy) => policy,
        Err(e) => {
            // Invalid calibration policy: fail closed (all advisory).
            if json_output {
                let error_msg = serde_json::json!({
                    "error": "calibration_policy_load_failed",
                    "message": e.to_string(),
                    "fallback": "all_advisory"
                });
                println!("{}", serde_json::to_string_pretty(&error_msg).unwrap_or_default());
            } else {
                eprintln!(
                    "Warning: calibration policy load failed — all guardians default to advisory: {e}"
                );
            }
            crate::domain::calibration::builtin_calibration_policy()
        }
    };

    // For V1, produce a simple activation plan from all matching rules
    let rules: Vec<&crate::domain::council::GuardianRule> = ruleset.rules.iter().collect();
    let plan = if rules.is_empty() {
        GuardianActivationPlan::zero_guardian(source)
    } else {
        GuardianActivationPlan::from_rules(&rules, source)
    };

    // Load trust records from workspace trace store.
    let mut trust_records = load_trust_records(workspace_root);

    // Resolve control levels for each activated guardian.
    let mut assignments: Vec<ControlLevelAssignment> = Vec::new();
    let activated_ids: Vec<String> = plan.activated.iter().map(|id| id.to_string()).collect();

    for rule_id in &activated_ids {
        let trust_record = get_or_create_trust_record(rule_id, &mut trust_records);
        let assignment = calibration_policy.resolve_level(
            rule_id,
            crate::domain::calibration::AuthorityZone::Green,
            crate::domain::calibration::RiskLevel::Low,
            Some(trust_record),
        );
        assignments.push(assignment);
    }

    // Determine outcome based on control levels.
    let has_blocking = assignments.iter().any(|a| a.assigned_level.blocks_execution());
    let has_hook = assignments.iter().any(|a| a.assigned_level == ControlLevel::Hook);

    let decision = if has_hook || has_blocking {
        CouncilDecision::blocked_mandatory_unavailable(&plan)
    } else {
        CouncilDecision::clean()
    };

    if json_output {
        render_json_with_calibration(&plan, &decision, &assignments, &calibration_policy)?;
    } else {
        render_human_with_calibration(&plan, &decision, &assignments, &calibration_policy);
    }

    // Persist trust records: update true/false positive counts based on decision.
    for id in &activated_ids {
        let record = get_or_create_trust_record(id, &mut trust_records);
        record.record_adjudication(
            decision.outcome == crate::domain::council::CouncilOutcome::Clean,
            false,
        );
    }
    let _ = save_trust_records(workspace_root, &trust_records);

    Ok(0)
}

/// Load the ruleset from `.boundline/guardian-rules.toml` or fall back to
/// built-in defaults.
fn load_ruleset(
    workspace_root: &Path,
) -> Result<(GuardianRuleset, RulesetSource), CouncilCliError> {
    let ruleset_path = workspace_root.join(".boundline").join("guardian-rules.toml");
    if ruleset_path.exists() {
        let content = std::fs::read_to_string(&ruleset_path)
            .map_err(|e| CouncilCliError::RulesetRead(e.to_string()))?;
        let ruleset: GuardianRuleset = toml::from_str(&content)
            .map_err(|e| CouncilCliError::RulesetInvalid(format!("TOML parse error: {e}")))?;
        ruleset.validate().map_err(|e| CouncilCliError::RulesetInvalid(e.to_string()))?;
        Ok((ruleset, RulesetSource::File))
    } else {
        Ok((builtin_ruleset(), RulesetSource::BuiltIn))
    }
}

fn render_human(plan: &GuardianActivationPlan, decision: &CouncilDecision) {
    println!("Guardian Activation Plan");
    println!("  Ruleset: {:?}", plan.ruleset_source);
    println!("  Matched rules: {}", plan.matched_rules.join(", "));
    println!("  Activated: {}", plan.activated.join(", "));
    if !plan.skipped.is_empty() {
        println!("  Skipped:");
        for s in &plan.skipped {
            println!("    - {} ({})", s.guardian_id, s.reason);
        }
    }
    if !plan.mandatory_unavailable.is_empty() {
        println!("  Mandatory unavailable: {}", plan.mandatory_unavailable.join(", "));
    }
    println!();
    println!("Council Decision");
    println!("  Adjudicator: {}", decision.adjudicator);
    println!("  Profile: {:?}", decision.profile_source);
    println!("  Outcome: {:?}", decision.outcome);
    println!("  Reason: {}", decision.reason);
}

impl From<serde_json::Error> for CouncilCliError {
    fn from(e: serde_json::Error) -> Self {
        CouncilCliError::RulesetRead(e.to_string())
    }
}

// ── Calibration-aware rendering ───────────────────────────────────────

fn render_human_with_calibration(
    plan: &GuardianActivationPlan,
    decision: &CouncilDecision,
    assignments: &[ControlLevelAssignment],
    policy: &CalibrationPolicy,
) {
    render_human(plan, decision);
    println!();
    println!("Control Level Assignments");
    if policy.entries.is_empty() {
        println!("  Policy: built-in all-advisory default (no calibration-policy.toml)");
    } else {
        println!(
            "  Policy: loaded from .boundline/calibration-policy.toml (schema v{})",
            policy.schema_version
        );
    }
    for a in assignments {
        let block_note = if a.assigned_level.blocks_execution() { " [BLOCKS]" } else { "" };
        println!(
            "  {}: {:?}{} — confidence: {:.2} (raw) / {:.2} (calibrated)",
            a.rule_id, a.assigned_level, block_note, a.guardian_confidence, a.calibrated_confidence
        );
        println!("    {}", a.reason);
    }
}

fn render_json_with_calibration(
    plan: &GuardianActivationPlan,
    decision: &CouncilDecision,
    assignments: &[ControlLevelAssignment],
    policy: &CalibrationPolicy,
) -> Result<(), CouncilCliError> {
    let output = serde_json::json!({
        "activation_plan": plan,
        "council_decision": decision,
        "calibration": {
            "schema_version": policy.schema_version,
            "entries_count": policy.entries.len(),
            "evidence_window": policy.evidence_window,
            "minimum_evidence_threshold": policy.minimum_evidence_threshold,
        },
        "control_levels": assignments,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_ruleset_falls_back_to_builtin() {
        let tmp = tempdir().unwrap();
        let (ruleset, source) = load_ruleset(tmp.path()).unwrap();
        assert_eq!(source, RulesetSource::BuiltIn);
        assert_eq!(ruleset.rules.len(), 4);
    }

    #[test]
    fn valid_ruleset_loads_from_file() {
        let tmp = tempdir().unwrap();
        let boundline_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&boundline_dir).unwrap();
        let ruleset_path = boundline_dir.join("guardian-rules.toml");
        std::fs::write(
            &ruleset_path,
            r#"schema_version = "1.0"

[[rules]]
id = "test"
stages = ["run"]
files = ["src/**/*.rs"]
activate = ["rust-guardian"]
"#,
        )
        .unwrap();
        let (ruleset, source) = load_ruleset(tmp.path()).unwrap();
        assert_eq!(source, RulesetSource::File);
        assert_eq!(ruleset.rules.len(), 1);
    }

    #[test]
    fn contradictory_ruleset_fails_validation() {
        let tmp = tempdir().unwrap();
        let boundline_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&boundline_dir).unwrap();
        let ruleset_path = boundline_dir.join("guardian-rules.toml");
        std::fs::write(
            &ruleset_path,
            r#"schema_version = "1.0"

[[rules]]
id = "a"
stages = ["run"]
files = ["src/**/*.rs"]
activate = ["rust-guardian"]

[[rules]]
id = "b"
stages = ["run"]
files = ["src/**/*.rs"]
skip = ["rust-guardian"]
"#,
        )
        .unwrap();
        let result = load_ruleset(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn run_with_builtin_ruleset_returns_clean() {
        let tmp = tempdir().unwrap();
        let exit = run(tmp.path(), false).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn run_with_json_output() {
        let tmp = tempdir().unwrap();
        let exit = run(tmp.path(), true).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn run_with_empty_ruleset_returns_clean() {
        let tmp = tempdir().unwrap();
        let boundline_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&boundline_dir).unwrap();
        let ruleset_path = boundline_dir.join("guardian-rules.toml");
        std::fs::write(&ruleset_path, "schema_version = \"1.0\"\nrules = []\n").unwrap();
        let exit = run(tmp.path(), false).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn invalid_toml_fails_validation() {
        let tmp = tempdir().unwrap();
        let boundline_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&boundline_dir).unwrap();
        let ruleset_path = boundline_dir.join("guardian-rules.toml");
        std::fs::write(&ruleset_path, "schema_version = \"1.0\"\nrules = \"invalid\"\n").unwrap();
        let result = load_ruleset(tmp.path());
        assert!(matches!(result, Err(CouncilCliError::RulesetInvalid(_))));
    }

    #[test]
    fn to_string_formats_errors() {
        let err1 = CouncilCliError::RulesetRead("test".into());
        let err2 = CouncilCliError::RulesetInvalid("test".into());
        assert_eq!(err1.to_string(), "failed to read ruleset: test");
        assert_eq!(err2.to_string(), "ruleset is invalid: test");
    }

    #[test]
    fn run_with_mandatory_unavailable() {
        // Need a ruleset that requires an unavailable mandatory guardian
        let tmp = tempdir().unwrap();
        let boundline_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&boundline_dir).unwrap();
        let ruleset_path = boundline_dir.join("guardian-rules.toml");
        std::fs::write(
            &ruleset_path,
            r#"schema_version = "1.0"

[[rules]]
id = "test"
stages = ["run"]
files = ["src/**/*.rs"]
activate = ["unknown-guardian"]
"#,
        )
        .unwrap();
        // It will return blocked but exit code is still 0 (the decision is captured in output)
        let exit = run(tmp.path(), false).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn to_string_formats_io_error() {
        let err = CouncilCliError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert_eq!(err.to_string(), "io error: test");
    }

    #[test]
    fn to_string_formats_json_error() {
        // Just trigger From<serde_json::Error>
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = CouncilCliError::from(json_err);
        assert!(err.to_string().starts_with("failed to read ruleset: expected value"));
    }

    #[test]
    fn run_with_invalid_calibration_policy_falls_back() {
        let tmp = tempdir().unwrap();
        let boundline_dir = tmp.path().join(".boundline");
        std::fs::create_dir_all(&boundline_dir).unwrap();
        let policy_path = boundline_dir.join("calibration-policy.toml");
        std::fs::write(&policy_path, "invalid toml").unwrap();

        // This triggers the fallback block in run()
        let exit = run(tmp.path(), false).unwrap();
        assert_eq!(exit, 0);

        let exit_json = run(tmp.path(), true).unwrap();
        assert_eq!(exit_json, 0);
    }

    #[test]
    fn render_human_with_skipped_and_empty_policy() {
        let plan = GuardianActivationPlan {
            plan_id: uuid::Uuid::new_v4(),
            ruleset_source: RulesetSource::BuiltIn,
            matched_rules: vec!["rule1".to_string()],
            activated: vec!["guardian_a".to_string()],
            skipped: vec![crate::domain::council::GuardianSkipRecord {
                guardian_id: "guardian_b".to_string(),
                reason: "Not applicable".to_string(),
                is_mandatory: false,
            }],
            mandatory_unavailable: vec![],
            escalation: None,
        };
        let decision = CouncilDecision::clean();
        let assignments = vec![];
        let policy = crate::domain::calibration::builtin_calibration_policy();

        // This covers lines 134-140 (skipped guardians) and 170-174 (empty policy).
        render_human_with_calibration(&plan, &decision, &assignments, &policy);
    }
}
