use boundline::cli::govern::{GovernRequest, execute_govern};
use boundline::domain::governance::CanonMode;

use crate::workspace_fixture::temp_empty_workspace;

#[test]
fn govern_without_mode_lists_supported_choices() -> Result<(), String> {
    let workspace = temp_empty_workspace("boundline-govern-no-mode");
    let report = execute_govern(GovernRequest {
        workspace: Some(&workspace),
        mode: None,
        goal: None,
        brief: &[],
        base: None,
        head: None,
        risk: None,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready: false,
        preserved_behavior_evidence: false,
    })
    .map_err(|error| error.to_string())?;
    if !report.terminal_output.contains("mode_choices:")
        || !report.terminal_output.contains("- architecture")
        || !report.terminal_output.contains("- pr-review")
    {
        return Err(format!("internal govern choices changed: {}", report.terminal_output));
    }
    Ok(())
}

#[test]
fn govern_with_mode_stops_when_session_state_is_missing() -> Result<(), String> {
    let workspace = temp_empty_workspace("boundline-govern-missing-session");
    let error = execute_govern(GovernRequest {
        workspace: Some(&workspace),
        mode: Some(CanonMode::Architecture),
        goal: None,
        brief: &[],
        base: None,
        head: None,
        risk: None,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready: false,
        preserved_behavior_evidence: false,
    })
    .err()
    .ok_or_else(|| "internal govern unexpectedly accepted missing session state".to_string())?;
    let rendered = error.to_string();
    if !rendered.contains(".boundline/session.json") || !rendered.contains("boundline goal") {
        return Err(format!("internal govern missing-session diagnostic changed: {rendered}"));
    }
    Ok(())
}
