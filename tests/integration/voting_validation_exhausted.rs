use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn validation_exhaustion_triggers_blocking_voting_for_implementation() {
    let workspace = temp_fixture_workspace("boundline-voting-validation");

    let govern = run_boundline_in(
        &workspace,
        &[
            "govern",
            "--mode",
            "implementation",
            "--goal",
            "Implement the next bounded onboarding slice",
            "--validation-exhausted",
        ],
    );
    let govern_text = terminal_text(&govern);
    assert_eq!(govern.status.code(), Some(0), "{govern_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("latest_voting_trigger: validation_exhausted"), "{text}");
    assert!(text.contains("latest_voting_blocking: true"), "{text}");
}
