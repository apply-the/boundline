use crate::workspace_fixture::{run_boundline, temp_empty_workspace, terminal_text};

#[test]
fn continue_ignores_chat_history_when_session_json_is_absent() {
    let workspace = temp_empty_workspace("boundline-continue-no-state");
    std::fs::write(
        workspace.join("chat-history.txt"),
        "Pretend the active Boundline goal is to ship the onboarding project.",
    )
    .unwrap();

    let output = run_boundline(&["continue", "--workspace", workspace.to_string_lossy().as_ref()]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("no active session"), "{text}");
    assert!(text.contains(".boundline/session.json"), "{text}");
    assert!(text.contains("chat history is not authoritative"), "{text}");
    assert!(!text.contains("onboarding project"), "{text}");
}
