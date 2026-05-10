use boundline::CompanionState;
use boundline::cli::diagnostics::{
    DiagnosticsCheck, DiagnosticsReport, DiagnosticsStatus, DiagnosticsSubject,
};
use boundline::cli::output::render_diagnostics;

#[test]
fn render_diagnostics_includes_install_specific_fields() {
    let report = DiagnosticsReport {
        subject: DiagnosticsSubject::Install,
        workspace_ref: None,
        installation_ref: Some("/tmp/boundline".to_string()),
        checks: vec![DiagnosticsCheck {
            name: "canon_companion".to_string(),
            status: DiagnosticsStatus::Passed,
            message: "Bundled Canon 0.43.0 is available".to_string(),
        }],
        ready: true,
        missing_prerequisites: Vec::new(),
        suggested_actions: Vec::new(),
        boundline_version: Some("0.39.0".to_string()),
        supported_canon_version: Some("0.43.0".to_string()),
        companion_state: Some(CompanionState::Ready),
        channel_candidates: vec!["homebrew".to_string(), "source".to_string()],
    };

    let rendered = render_diagnostics(&report);

    assert!(rendered.contains("doctor: ready for installation /tmp/boundline"));
    assert!(rendered.contains("boundline_version: 0.39.0"));
    assert!(rendered.contains("supported_canon_version: 0.43.0"));
    assert!(rendered.contains("companion_state: ready"));
    assert!(rendered.contains("channel_candidates: homebrew, source"));
}
