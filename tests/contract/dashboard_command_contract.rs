use std::env;
use std::sync::Mutex;

use crate::dashboard_fixture::{
    DashboardTestResult, dashboard_workspace, require_contains, require_eq,
};
use boundline::adapters::dashboard_state::DashboardStateAssembler;
use boundline::domain::dashboard::DegradedReason;
use boundline_dashboard::app::{DashboardCli, run};

static CURRENT_DIR_LOCK: Mutex<()> = Mutex::new(());

struct CurrentDirGuard {
    original: std::path::PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let original = env::current_dir()?;
        env::set_current_dir(path)?;
        Ok(Self { original })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

#[test]
fn snapshot_json_for_workspace_without_session_reports_launch_fallback() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-command-no-session")?;
    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    let degraded =
        snapshot.degraded_state.as_ref().ok_or("no-session snapshot should be degraded")?;

    require_eq(degraded.reason, DegradedReason::MissingActiveSession, "degraded reason")?;
    require_contains(&degraded.available_commands.join("\n"), "boundline start", "fallback command")
}

#[test]
fn normal_launcher_reports_dashboard_unavailable_with_fallbacks() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-command-launcher")?;
    let output = boundline::cli::dashboard::render_launcher_unavailable(&workspace, true);
    require_contains(&output, "dashboard_unavailable", "launcher outcome")?;
    require_contains(&output, "boundline status", "status fallback")?;
    require_contains(&output, "boundline inspect", "inspect fallback")
}

#[test]
fn launcher_snapshot_status_reports_ready_for_session_workspace() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-command-ready")?;
    crate::dashboard_fixture::write_session(&workspace, "goal_captured", 7)?;

    let output = boundline::cli::dashboard::execute_dashboard_launcher(Some(&workspace), false);
    require_contains(&output, "snapshot_status: ready", "snapshot status")?;
    require_contains(&output, "color_mode: color", "color mode")
}

#[test]
fn launcher_snapshot_status_reports_degraded_without_session() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-command-degraded")?;

    let output = boundline::cli::dashboard::execute_dashboard_launcher(Some(&workspace), true);
    require_contains(&output, "snapshot_status: degraded", "snapshot status")?;
    require_contains(&output, "color_mode: monochrome", "color mode")
}

#[test]
fn launcher_uses_current_directory_when_workspace_is_omitted() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-command-current-dir")?;
    crate::dashboard_fixture::write_session(&workspace, "goal_captured", 7)?;

    let _current_dir_lock = CURRENT_DIR_LOCK.lock().map_err(|error| error.to_string())?;
    let _guard = CurrentDirGuard::change_to(&workspace)?;
    let resolved_current_dir = env::current_dir()?;
    let output = boundline::cli::dashboard::execute_dashboard_launcher(None, true);

    require_contains(&output, "snapshot_status: ready", "snapshot status")?;
    require_contains(&output, "color_mode: monochrome", "color mode")?;
    require_contains(
        &output,
        &format!("workspace: {}", resolved_current_dir.display()),
        "workspace path",
    )
}

#[test]
fn dedicated_entrypoint_snapshot_json_emits_contract_snapshot() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-command-entrypoint")?;
    let output =
        run(DashboardCli { workspace: Some(workspace), no_color: true, snapshot_json: true })?;

    require_contains(&output, "\"snapshot_id\"", "snapshot id")?;
    require_contains(&output, "\"degraded_state\"", "degraded state")
}
