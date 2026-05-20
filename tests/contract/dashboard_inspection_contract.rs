use crate::dashboard_fixture::{DashboardTestResult, require, require_eq};
use boundline::domain::dashboard::{
    ContextPackPanelItem, DashboardPanels, GoalPlanPanel, GovernedReferencePanelItem,
};

#[test]
fn inspection_panels_distinguish_goal_context_diagnostics_and_governed_refs() -> DashboardTestResult
{
    let mut panels = DashboardPanels::empty();
    panels.goal_plan = Some(GoalPlanPanel {
        revision: 2,
        state: "confirmed".to_string(),
        verification_strategy: Some("run targeted validation".to_string()),
        targets: vec!["src/lib.rs".to_string()],
    });
    panels.context_pack.push(ContextPackPanelItem {
        reason: "target was changed recently".to_string(),
        source: "workspace".to_string(),
        budget: Some("local".to_string()),
        authority: "workspace_file".to_string(),
        evidence_ref: "src/lib.rs".to_string(),
    });
    panels.governed_references.push(GovernedReferencePanelItem {
        reference: ".canon/packet.json".to_string(),
        readiness: "available".to_string(),
        provenance: "canon".to_string(),
        approval_cue: Some("not_needed".to_string()),
        read_only: true,
    });

    let value = serde_json::to_value(&panels)?;
    require_eq(value["goal_plan"]["revision"].as_u64(), Some(2), "goal revision")?;
    require_eq(
        value["context_pack"][0]["authority"].as_str(),
        Some("workspace_file"),
        "context authority",
    )?;
    require(
        value["governed_references"][0]["read_only"].as_bool() == Some(true),
        "governed references must remain read-only",
    )
}

#[test]
fn empty_panels_serialize_as_available_empty_lists() -> DashboardTestResult {
    let value = serde_json::to_value(DashboardPanels::empty())?;
    require_eq(value["evidence"].as_array().map(Vec::len), Some(0), "evidence empty")?;
    require_eq(
        value["governed_references"].as_array().map(Vec::len),
        Some(0),
        "governed refs empty",
    )
}
