use std::fs;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::domain::session::ActiveSessionRecord;
use boundline::domain::workflow::{ProjectScalePathKind, ProjectScaleStageKind};

#[test]
fn session_json_persists_project_scale_path_stage_work_unit_and_trace_refs() {
    let workspace = temp_fixture_workspace("boundline-project-scale-contract");

    assert_eq!(
        run_boundline_in(
            &workspace,
            &["goal", "--goal", "Build a customer onboarding capability with audit logging",],
        )
        .status
        .code(),
        Some(0)
    );
    let plan = run_boundline_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let session_path = workspace.join(".boundline/session.json");
    let record: ActiveSessionRecord =
        serde_json::from_slice(&fs::read(&session_path).unwrap()).unwrap();
    record.validate().unwrap();

    let project_scale = record.project_scale.expect("project scale state should be persisted");
    assert_eq!(project_scale.path.kind, ProjectScalePathKind::IdeaToCode);
    assert_eq!(project_scale.path.stages[0].kind, ProjectScaleStageKind::Discovery);
    assert!(
        project_scale.path.stages.iter().any(|stage| stage.kind == ProjectScaleStageKind::PrReview)
    );
    assert_eq!(project_scale.active_stage_index, 0);
    assert_eq!(project_scale.active_work_unit_id.as_deref(), Some("stage-001-discovery"));
    assert_eq!(project_scale.next_action, "confirm_project_scale_path");
    assert!(project_scale.checkpoint_refs.is_empty());
    assert!(project_scale.trace_refs.is_empty());
}
