use std::fs;

use boundline::FileConfigStore;
use boundline::adapters::config_store::ConfigStoreError;
use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{
    execute_capture, execute_plan, execute_run, execute_start, execute_status,
};
use boundline::domain::configuration::{
    CapabilityState, ConfigFile, ModelRoute, RoutingConfig, RuntimeCapabilityProfile, RuntimeKind,
};

use crate::workspace_fixture::temp_fixture_workspace;

#[test]
fn status_surfaces_guardian_degradation_when_verification_route_lacks_validation() {
    let workspace = temp_fixture_workspace("boundline-cli-guardian-degradation");
    let config = ConfigFile {
        routing: RoutingConfig {
            verification: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "guardian-reviewer".to_string(),
            }),
            runtime_capabilities: std::iter::once((
                RuntimeKind::Claude,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Unsupported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("semantic validation is intentionally unavailable".to_string()),
                },
            ))
            .collect(),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    save_local_config(&workspace, &config).unwrap();

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();
    execute_run(Some(&workspace)).unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should remain persisted");

    assert!(
        plan.guidance_guardian
            .guardian_degradations
            .iter()
            .any(|line| { line.contains("validation support is unavailable") })
    );
    assert!(
        status_report.terminal_output.contains("guardian_degradations:"),
        "{}",
        status_report.terminal_output
    );
    assert!(
        status_report.terminal_output.contains("validation support is unavailable"),
        "{}",
        status_report.terminal_output
    );
}

#[test]
fn status_surfaces_semantic_skip_after_blocking_deterministic_findings() {
    let workspace = temp_fixture_workspace("boundline-cli-guardian-skip");
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 {\n    let total = Some(left + right).unwrap();\n    total\n}\n",
    )
    .unwrap();

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();
    execute_run(Some(&workspace)).unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should remain persisted");

    assert!(
        plan.guidance_guardian
            .guardian_timeline
            .iter()
            .any(|line| { line.contains("skipped after blocking deterministic findings") })
    );
    assert!(
        status_report.terminal_output.contains("skipped after blocking deterministic findings"),
        "{}",
        status_report.terminal_output
    );
}

fn save_local_config(
    workspace: &std::path::Path,
    config: &ConfigFile,
) -> Result<(), ConfigStoreError> {
    FileConfigStore::for_workspace(workspace).save_local(config).map(|_| ())
}
