use crate::workspace_fixture::{run_boundline_in, temp_empty_workspace, terminal_text};

#[test]
fn broad_goal_with_insufficient_context_stops_with_project_scale_repair_guidance() {
    let workspace = temp_empty_workspace("boundline-project-scale-context-stop");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(
            &workspace,
            &["capture", "--goal", "Build a customer onboarding capability with audit logging",],
        )
        .status
        .code(),
        Some(0)
    );

    let plan = run_boundline_in(&workspace, &["plan"]);
    let text = terminal_text(&plan);

    assert_eq!(plan.status.code(), Some(1), "{text}");
    assert!(text.contains("bounded context required before planning"), "{text}");
    assert!(text.contains("next_command: boundline capture --goal <narrower goal>"), "{text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("project_scale_path:"), "{status_text}");
    assert!(status_text.contains("project_scale_current_stage: discovery"), "{status_text}");
    assert!(status_text.contains("project_scale_next_action: repair_context"), "{status_text}");
}
