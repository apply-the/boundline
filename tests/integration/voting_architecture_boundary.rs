use boundline::cli::govern::{GovernRequest, execute_govern};
use boundline::domain::governance::CanonMode;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn high_impact_architecture_govern_stage_persists_blocking_voting_state() -> Result<(), String> {
    let workspace = temp_fixture_workspace("boundline-voting-architecture");
    execute_govern(GovernRequest {
        workspace: Some(&workspace),
        mode: Some(CanonMode::Architecture),
        goal: Some("Choose architecture for the onboarding capability"),
        brief: &[],
        base: None,
        head: None,
        risk: Some("high"),
        structural_impact: true,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready: false,
        preserved_behavior_evidence: false,
    })
    .map_err(|error| error.to_string())?;
    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    for expected in [
        "latest_voting_trigger: high_impact_architecture",
        "latest_voting_result: pending",
        "latest_voting_blocking: true",
        "latest_voting_next_action: resolve_voting_boundary",
    ] {
        if !text.contains(expected) {
            return Err(format!("architecture voting projection omitted {expected}: {text}"));
        }
    }
    Ok(())
}
