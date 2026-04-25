use crate::workspace_fixture::temp_fixture_workspace;
use synod::cli::diagnostics::{DiagnosticsStatus, diagnose_workspace};
use uuid::Uuid;

#[test]
fn diagnostics_report_covers_the_required_fields_for_a_ready_workspace() {
    let workspace = temp_fixture_workspace("synod-diagnostics-contract");
    let report = diagnose_workspace(&workspace);

    assert_eq!(report.workspace_ref, workspace.to_string_lossy());
    assert!(report.ready);
    assert!(report.checks.len() >= 4);
    assert!(report.missing_prerequisites.is_empty());
    assert!(report.suggested_actions.is_empty());
    assert!(report.checks.iter().all(|check| check.status == DiagnosticsStatus::Passed));
}

#[test]
fn diagnostics_report_keeps_failed_checks_actionable() {
    let workspace =
        std::env::temp_dir().join(format!("synod-diagnostics-missing-{}", Uuid::new_v4()));
    let report = diagnose_workspace(&workspace);

    assert!(!report.ready);
    assert!(!report.missing_prerequisites.is_empty());
    assert_eq!(report.missing_prerequisites.len(), report.suggested_actions.len());
    assert!(report.checks.iter().any(|check| check.status == DiagnosticsStatus::Failed));
    assert!(report.suggested_actions.iter().all(|message| !message.trim().is_empty()));
}
