use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn high_impact_architecture_govern_stage_persists_blocking_voting_state() {
    let workspace = temp_fixture_workspace("boundline-voting-architecture");

    let govern = run_boundline_in(
        &workspace,
        &[
            "govern",
            "--mode",
            "architecture",
            "--goal",
            "Choose architecture for the onboarding capability",
            "--risk",
            "high",
            "--structural-impact",
        ],
    );
    let govern_text = terminal_text(&govern);
    assert_eq!(govern.status.code(), Some(0), "{govern_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("latest_voting_trigger: high_impact_architecture"), "{text}");
    assert!(text.contains("latest_voting_result: pending"), "{text}");
    assert!(text.contains("latest_voting_blocking: true"), "{text}");
    assert!(text.contains("latest_voting_next_action: resolve_voting_boundary"), "{text}");
}
