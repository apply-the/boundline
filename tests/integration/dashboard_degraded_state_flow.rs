use std::fs;

use crate::dashboard_fixture::{
    DashboardTestResult, dashboard_workspace, require_contains, require_eq,
};
use boundline::adapters::dashboard_state::DashboardStateAssembler;
use boundline::domain::dashboard::DegradedReason;

#[test]
fn invalid_workspace_degrades_to_init_fallback() -> DashboardTestResult {
    let workspace = std::env::temp_dir().join("boundline-dashboard-missing-workspace");
    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    let degraded = snapshot.degraded_state.as_ref().ok_or("invalid workspace must degrade")?;

    require_eq(degraded.reason, DegradedReason::InvalidWorkspace, "reason")?;
    require_contains(&degraded.available_commands.join("\n"), "boundline init", "fallback")
}

#[test]
fn invalid_session_json_degrades_without_dashboard_state_write() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-invalid-session")?;
    let session_path = workspace.join(".boundline").join("session.json");
    fs::write(&session_path, b"not json")?;

    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    let degraded = snapshot.degraded_state.as_ref().ok_or("invalid session json must degrade")?;

    require_eq(degraded.reason, DegradedReason::InvalidSessionJson, "reason")?;
    require_contains(&degraded.available_commands.join("\n"), "boundline status", "fallback")
}
