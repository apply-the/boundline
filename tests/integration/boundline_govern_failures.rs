use crate::workspace_fixture::{run_boundline_in, temp_empty_workspace, terminal_text};

#[test]
fn govern_without_mode_lists_supported_choices() {
    let workspace = temp_empty_workspace("boundline-govern-no-mode");

    let output = run_boundline_in(&workspace, &["govern"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("govern: mode required"), "{text}");
    assert!(text.contains("mode_choices:"), "{text}");
    assert!(text.contains("- architecture"), "{text}");
    assert!(text.contains("- supply-chain-analysis"), "{text}");
    assert!(text.contains("- pr-review"), "{text}");
}

#[test]
fn govern_with_mode_stops_when_session_state_is_missing() {
    let workspace = temp_empty_workspace("boundline-govern-missing-session");

    let output = run_boundline_in(&workspace, &["govern", "--mode", "architecture"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("govern error:"), "{text}");
    assert!(text.contains(".boundline/session.json"), "{text}");
    assert!(text.contains("boundline start"), "{text}");
}
