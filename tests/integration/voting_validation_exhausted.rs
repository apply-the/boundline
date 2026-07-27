use boundline::cli::govern::{GovernRequest, execute_govern};
use boundline::domain::governance::CanonMode;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn validation_exhaustion_triggers_blocking_voting_for_implementation() -> Result<(), String> {
    let workspace = temp_fixture_workspace("boundline-voting-validation");
    execute_govern(GovernRequest {
        workspace: Some(&workspace),
        mode: Some(CanonMode::Implementation),
        goal: Some("Implement the next bounded onboarding slice"),
        brief: &[],
        base: None,
        head: None,
        risk: None,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: true,
        pr_ready: false,
        preserved_behavior_evidence: false,
    })
    .map_err(|error| error.to_string())?;
    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    if !text.contains("latest_voting_trigger: validation_exhausted")
        || !text.contains("latest_voting_blocking: true")
    {
        return Err(format!("validation voting projection changed: {text}"));
    }
    Ok(())
}
