use boundline::cli::govern::{GovernRequest, execute_govern};
use boundline::domain::governance::CanonMode;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

fn govern<'a>(
    workspace: &'a std::path::Path,
    mode: CanonMode,
    goal: &'a str,
    risk: Option<&'a str>,
    pr_ready: bool,
    preserved_behavior_evidence: bool,
) -> GovernRequest<'a> {
    GovernRequest {
        workspace: Some(workspace),
        mode: Some(mode),
        goal: Some(goal),
        brief: &[],
        base: None,
        head: None,
        risk,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready,
        preserved_behavior_evidence,
    }
}

#[test]
fn pr_ready_triggers_voting_but_low_risk_refactor_with_evidence_skips_it() -> Result<(), String> {
    let workspace = temp_fixture_workspace("boundline-voting-pr-ready");
    execute_govern(govern(
        &workspace,
        CanonMode::PrReview,
        "Review the merge-ready onboarding diff",
        None,
        true,
        false,
    ))
    .map_err(|error| error.to_string())?;
    let status = terminal_text(&run_boundline_in(&workspace, &["status"]));
    if !status.contains("latest_voting_trigger: pr_ready")
        || !status.contains("latest_voting_blocking: true")
    {
        return Err(format!("PR voting projection changed: {status}"));
    }

    execute_govern(govern(
        &workspace,
        CanonMode::Refactor,
        "Refactor the local helper without behavior changes",
        Some("low"),
        false,
        true,
    ))
    .map_err(|error| error.to_string())?;
    let skipped = terminal_text(&run_boundline_in(&workspace, &["status"]));
    if !skipped.contains("latest_voting_trigger: low_risk_preserved_behavior")
        || !skipped.contains("latest_voting_result: skipped")
        || !skipped.contains("latest_voting_blocking: false")
    {
        return Err(format!("low-risk voting skip projection changed: {skipped}"));
    }
    Ok(())
}
