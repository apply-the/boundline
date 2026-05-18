use crate::workspace_fixture::{run_boundline, terminal_text};

#[test]
fn assistant_install_user_scope_reports_global_bootstrap_commands_for_supported_hosts() {
    for host in ["claude", "codex", "cursor", "copilot", "gemini"] {
        let output = run_boundline(&["assistant", "install", "--host", host, "--scope", "user"]);
        let text = terminal_text(&output);

        assert_eq!(output.status.code(), Some(0), "{text}");
        assert!(text.contains("assistant_global_package:"), "{text}");
        assert!(text.contains(&format!("host: {host}")), "{text}");
        assert!(text.contains("scope: user"), "{text}");
        assert!(text.contains("/boundline:init"), "{text}");
        assert!(text.contains("/boundline:doctor"), "{text}");
        assert!(text.contains("/boundline:status"), "{text}");
        assert!(text.contains("/boundline:continue"), "{text}");
        assert!(text.contains("boundline init --workspace"), "{text}");
    }
}

#[test]
fn assistant_install_user_scope_is_explicit_when_host_needs_manual_fallback() {
    let output = run_boundline(&["assistant", "install", "--host", "gemini", "--scope", "user"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("install_mode: manual_fallback"), "{text}");
    assert!(text.contains("global command installation is not claimed for this host"), "{text}");
    assert!(text.contains("boundline doctor --workspace"), "{text}");
}

#[test]
fn assistant_install_user_scope_reports_contextual_doctor_context_follow_up() {
    let output = run_boundline(&["assistant", "install", "--host", "claude", "--scope", "user"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("contextual_commands:"), "{text}");
    assert!(text.contains("/boundline:doctor-context"), "{text}");
    assert!(text.contains("/boundline:explain-plan"), "{text}");
    assert!(text.contains("boundline doctor --workspace"), "{text}");
}
