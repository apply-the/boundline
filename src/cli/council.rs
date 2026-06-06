//! `boundline council adjudicate` CLI command and output rendering.
//!
//! Reads the guardian ruleset, produces an activation plan, and
//! adjudicates findings into a clean/blocked council decision.

use std::path::Path;

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
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Run the council adjudication command.
pub fn run(workspace_root: &Path, json_output: bool) -> Result<i32, CouncilCliError> {
    let (ruleset, source) = load_ruleset(workspace_root)?;

    // For V1, produce a simple activation plan from all matching rules
    // (in a real implementation this would evaluate the change surface).
    let rules: Vec<&crate::domain::council::GuardianRule> = ruleset.rules.iter().collect();
    let plan = if rules.is_empty() {
        GuardianActivationPlan::zero_guardian(source)
    } else {
        GuardianActivationPlan::from_rules(&rules, source)
    };

    let decision = if plan.mandatory_unavailable.is_empty() {
        CouncilDecision::clean()
    } else {
        CouncilDecision::blocked_mandatory_unavailable(&plan)
    };

    if json_output {
        render_json(&plan, &decision)?;
    } else {
        render_human(&plan, &decision);
    }

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

fn render_json(
    plan: &GuardianActivationPlan,
    decision: &CouncilDecision,
) -> Result<(), CouncilCliError> {
    let output = serde_json::json!({
        "activation_plan": plan,
        "council_decision": decision,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

impl From<serde_json::Error> for CouncilCliError {
    fn from(e: serde_json::Error) -> Self {
        CouncilCliError::RulesetRead(e.to_string())
    }
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
}
