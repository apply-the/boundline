use boundline::cli::govern::{GovernRequest, execute_govern};
use boundline::domain::governance::CanonMode;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn govern_routes_explicit_canon_modes_through_boundline_session_state() -> Result<(), String> {
    let workspace = temp_fixture_workspace("boundline-govern-modes");
    for mode in [
        CanonMode::Architecture,
        CanonMode::Requirements,
        CanonMode::SecurityAssessment,
        CanonMode::Migration,
        CanonMode::SupplyChainAnalysis,
        CanonMode::PrReview,
    ] {
        let report = execute_govern(GovernRequest {
            workspace: Some(&workspace),
            mode: Some(mode),
            goal: Some("Shape the governed stage for the current delivery boundary"),
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
        if !report.terminal_output.contains("govern: staged")
            || !report.terminal_output.contains(mode.as_str())
        {
            return Err(format!(
                "internal govern mode routing changed: {}",
                report.terminal_output
            ));
        }
    }
    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    if !status.status.success() || !text.contains("governance_lifecycle_selected_mode: pr-review") {
        return Err(format!("governance session projection changed: {text}"));
    }
    Ok(())
}
