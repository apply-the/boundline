use boundline::cli::output::render_host_command_json;
use boundline::domain::configuration::ConfigFile;
use boundline::domain::session::SessionStatusView;

use crate::workspace_fixture::temp_git_workspace;

#[test]
fn host_output_surfaces_capability_provider_projection_fields() {
    let workspace = temp_git_workspace("boundline-provider-host-output");
    let config = ConfigFile::default();
    let save_result =
        boundline::FileConfigStore::for_workspace(workspace.path()).save_local(&config);
    assert!(save_result.is_ok());

    let view = SessionStatusView {
        workspace_ref: workspace.path().to_string_lossy().into_owned(),
        session_id: "session-provider-output".to_string(),
        ..SessionStatusView::default()
    };

    let rendered = render_host_command_json(
        "status",
        boundline::cli::CommandExitStatus::Succeeded,
        "ok",
        None,
        Some(&view),
        None,
    );

    assert!(rendered.contains("\"capability_provider_status\""));
}
