use std::path::{Path, PathBuf};

use crate::adapters::dashboard_state::DashboardStateAssembler;

const DASHBOARD_UNAVAILABLE: &str = "dashboard_unavailable";

pub fn execute_dashboard_launcher(workspace: Option<&Path>, no_color: bool) -> String {
    let workspace_path = workspace
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let snapshot_status = DashboardStateAssembler::for_workspace(&workspace_path)
        .snapshot(no_color)
        .map(|snapshot| if snapshot.degraded_state.is_some() { "degraded" } else { "ready" })
        .unwrap_or("degraded");
    let mut output = render_launcher_unavailable(&workspace_path, no_color);
    output.push_str(&format!("snapshot_status: {snapshot_status}\n"));
    output
}

pub fn render_launcher_unavailable(workspace: &Path, no_color: bool) -> String {
    let workspace_arg = workspace.display();
    let color_mode = if no_color { "monochrome" } else { "color" };
    [
        format!("outcome: {DASHBOARD_UNAVAILABLE}"),
        format!("workspace: {workspace_arg}"),
        format!("color_mode: {color_mode}"),
        "message: the dedicated boundline-dashboard entrypoint is not available from this launcher path".to_string(),
        format!("fallback: boundline status --workspace {workspace_arg}"),
        format!("fallback: boundline inspect --workspace {workspace_arg}"),
        format!("fallback: boundline run --workspace {workspace_arg}"),
    ]
    .join("\n")
        + "\n"
}
