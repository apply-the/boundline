use crate::dashboard_fixture::{DashboardTestResult, require, require_eq};
use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::adapters::dashboard_state::DashboardStateAssembler;

#[test]
fn inspection_panels_are_projected_from_existing_session_state() -> DashboardTestResult {
    let workspace = temp_fixture_workspace("dashboard-inspection-flow");
    require_eq(run_boundline_in(&workspace, &["start"]).status.code(), Some(0), "start")?;
    require_eq(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing add test"])
            .status
            .code(),
        Some(0),
        "capture",
    )?;
    let plan = run_boundline_in(&workspace, &["plan"]);
    require_eq(plan.status.code(), Some(0), &terminal_text(&plan))?;

    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    require(snapshot.panels.goal_plan.is_some(), "goal plan panel must be available")?;
    require(
        !snapshot.panels.diagnostics.is_empty(),
        "dashboard diagnostics panel must be available",
    )?;
    require(
        !workspace.join(".canon").exists(),
        "dashboard inspection must not write governed artifact state",
    )
}
