use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn broad_capability_goal_surfaces_a_bounded_idea_to_code_path() {
    let workspace = temp_fixture_workspace("boundline-project-scale-idea");

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

    assert_eq!(plan.status.code(), Some(0), "{text}");
    assert!(text.contains("project_scale_path:"), "{text}");
    assert!(
        text.contains(
            "discovery -> requirements -> domain-language -> domain-model -> system-shaping -> architecture -> backlog -> implementation -> verification -> pr-review"
        ),
        "{text}"
    );
    assert!(text.contains("project_scale_current_stage: discovery"), "{text}");
    assert!(text.contains("project_scale_next_action: confirm_project_scale_path"), "{text}");
    assert!(text.contains("next_command: boundline plan --confirm"), "{text}");
}
