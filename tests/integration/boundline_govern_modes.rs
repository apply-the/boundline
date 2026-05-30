use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn govern_routes_explicit_canon_modes_through_boundline_session_state() {
    let workspace = temp_fixture_workspace("boundline-govern-modes");

    for mode in [
        "architecture",
        "requirements",
        "security-assessment",
        "migration",
        "supply-chain-analysis",
        "pr-review",
    ] {
        let output = run_boundline_in(
            &workspace,
            &[
                "govern",
                "--mode",
                mode,
                "--goal",
                "Shape the governed stage for the current delivery boundary",
            ],
        );
        let text = terminal_text(&output);

        assert_eq!(output.status.code(), Some(0), "{text}");
        assert!(text.contains("govern: staged"), "{text}");
        assert!(text.contains(&format!("mode: {mode}")), "{text}");
        assert!(text.contains("governed_stage_ref:"), "{text}");
        assert!(text.contains("next_command: boundline status"), "{text}");
    }

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("governance_lifecycle_runtime: canon"), "{status_text}");
    assert!(status_text.contains("governance_lifecycle_selected_mode: pr-review"), "{status_text}");
}
