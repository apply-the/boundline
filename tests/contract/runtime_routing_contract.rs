use std::path::Path;

use boundline::FileConfigStore;
use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::inspect::execute_inspect;
use boundline::cli::run::{execute_custom_run, execute_native_direct_run};
use boundline::cli::session::{
    execute_capture, execute_next, execute_plan, execute_run, execute_start, execute_status,
};
use boundline::domain::configuration::{ConfigFile, ModelRoute, RoutingConfig, RuntimeKind};

use crate::runtime_refoundation::{
    temp_runtime_refoundation_compat_workspace, temp_runtime_refoundation_governed_workspace,
};

#[test]
fn confirmed_goal_plan_takes_precedence_over_execution_profile_for_session_run() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-native");

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

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);
    assert!(run.terminal_output.contains("routing: native (goal_plan)"), "{}", run.terminal_output);
    assert!(!run.terminal_output.contains("routing: compatibility"), "{}", run.terminal_output);
}

#[test]
fn native_direct_run_stays_native_even_when_execution_profile_exists() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-direct");

    let report = execute_native_direct_run(
        &workspace,
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(
        report.terminal_output.contains("routing: native (goal_plan)"),
        "{}",
        report.terminal_output
    );
    assert!(
        !report.terminal_output.contains("routing: compatibility"),
        "{}",
        report.terminal_output
    );
}

#[test]
fn explicit_compatibility_run_is_visible_and_preserves_native_session_state() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-compat");

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

    let before = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let report = execute_custom_run(
        &workspace,
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(
        report.terminal_output.contains("routing: compatibility"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("execution_path: fixture_compatibility"),
        "{}",
        report.terminal_output
    );

    let after = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert_eq!(after.goal_plan, before.goal_plan);
    assert_eq!(after.decisions, before.decisions);
}

#[test]
fn native_and_compatibility_follow_up_keep_shared_routing_and_execution_condition_labels() {
    let compatibility_workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-shared-summary");

    let compatibility_next = execute_next(Some(&compatibility_workspace)).unwrap_err();
    assert!(compatibility_next.to_string().contains("no active session found"));

    let compatibility_run = execute_custom_run(
        &compatibility_workspace,
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(
        compatibility_run.terminal_output.contains("routing: compatibility"),
        "{}",
        compatibility_run.terminal_output
    );
    assert!(
        compatibility_run.terminal_output.contains("route_owner: compatibility"),
        "{}",
        compatibility_run.terminal_output
    );
    assert!(
        compatibility_run.terminal_output.contains("execution_condition: terminal -"),
        "{}",
        compatibility_run.terminal_output
    );

    let compatibility_follow_up = execute_next(Some(&compatibility_workspace)).unwrap();
    assert!(
        compatibility_follow_up
            .terminal_output
            .contains("routing: compatibility (execution_profile)"),
        "{}",
        compatibility_follow_up.terminal_output
    );
    assert!(
        compatibility_follow_up.terminal_output.contains("route_owner: compatibility"),
        "{}",
        compatibility_follow_up.terminal_output
    );
    assert!(
        compatibility_follow_up.terminal_output.contains("execution_condition: terminal -"),
        "{}",
        compatibility_follow_up.terminal_output
    );

    let native_workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-native-summary");
    execute_start(Some(&native_workspace)).unwrap();
    execute_capture(
        Some(&native_workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&native_workspace), Some("bug-fix"), false, false).unwrap();

    let native_run = execute_run(Some(&native_workspace)).unwrap();
    assert!(
        native_run.terminal_output.contains("routing: native (goal_plan)"),
        "{}",
        native_run.terminal_output
    );
    assert!(
        native_run.terminal_output.contains("route_owner: native"),
        "{}",
        native_run.terminal_output
    );
    assert!(
        native_run.terminal_output.contains("execution_condition: terminal -"),
        "{}",
        native_run.terminal_output
    );
}

#[test]
fn status_projects_workspace_routing_defaults_for_native_follow_up() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-config");

    let config = ConfigFile {
        routing: RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "gpt-5-codex".to_string(),
            }),
            implementation: Some(ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "gpt-5.4".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

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

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(status.terminal_output.contains("route_owner: native"), "{}", status.terminal_output);
    assert!(
        status
            .terminal_output
            .contains("route_config_projection: workspace_routing: planning=codex/gpt-5-codex, implementation=copilot/gpt-5.4"),
        "{}",
        status.terminal_output
    );
}

#[test]
fn compatibility_inspect_uses_persisted_routing_snapshot_after_config_changes() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-compat-snapshot");

    let before = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-before".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&before).unwrap();

    let report = execute_custom_run(
        &workspace,
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let trace_ref = report.trace_location.unwrap();

    let after = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-after".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&after).unwrap();

    let inspect = execute_inspect(Some(Path::new(&trace_ref)), None).unwrap();

    assert!(
        inspect
            .terminal_output
            .contains("effective_routing: planning=codex/gpt-5-codex [built-in], implementation=codex/gpt-5-codex [built-in], verification=copilot/gpt-5.4 [built-in], review=claude/reviewer-before [workspace], adjudication=codex/gpt-5-codex [built-in]"),
        "{}",
        inspect.terminal_output
    );
    assert!(!inspect.terminal_output.contains("reviewer-after"), "{}", inspect.terminal_output);
}

#[test]
fn native_inspect_uses_persisted_routing_snapshot_after_config_changes() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-native-snapshot");

    let before = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-before".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&before).unwrap();

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

    let trace_ref = FileSessionStore::for_workspace(&workspace)
        .load()
        .unwrap()
        .unwrap()
        .latest_trace_ref
        .unwrap();

    let after = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-after".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&after).unwrap();

    let inspect = execute_inspect(Some(Path::new(&trace_ref)), None).unwrap();

    assert!(
        inspect
            .terminal_output
            .contains("effective_routing: planning=codex/gpt-5-codex [built-in], implementation=codex/gpt-5-codex [built-in], verification=copilot/gpt-5.4 [built-in], review=claude/reviewer-before [workspace], adjudication=codex/gpt-5-codex [built-in]"),
        "{}",
        inspect.terminal_output
    );
    assert!(!inspect.terminal_output.contains("reviewer-after"), "{}", inspect.terminal_output);
}

#[test]
fn native_run_persists_delegation_when_route_runtime_missing_from_declared_assistant_capabilities()
{
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-unsupported-binding");

    let config = ConfigFile {
        routing: RoutingConfig {
            implementation: Some(ModelRoute {
                runtime: RuntimeKind::Gemini,
                model: "gemini-2.5-pro".to_string(),
            }),
            assistant_runtimes: vec![RuntimeKind::Codex],
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

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

    let run = execute_run(Some(&workspace)).unwrap();
    let record = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let goal_plan = record.goal_plan.as_ref().expect("goal plan should persist after blocked run");
    let continuity =
        goal_plan.delegation_continuity().expect("delegation continuity should be recorded");

    assert!(
        run.terminal_output.contains(
            "delegation_headline: escalation required: implementation route requires gemini, but available assistant runtimes are: codex"
        ),
        "{}",
        run.terminal_output
    );
    assert!(
        run.terminal_output.contains("delegation_packet_kind: escalation"),
        "{}",
        run.terminal_output
    );
    assert_eq!(continuity.mode.as_str(), "escalation_required");
    assert_eq!(continuity.next_command, "boundline inspect");
}

#[test]
fn canon_artifacts_remain_bounded_evidence_for_native_runs() {
    let workspace =
        temp_runtime_refoundation_governed_workspace("runtime-routing-contract-governed");

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
    execute_plan(Some(&workspace), None, true, false).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);
    assert!(!run.terminal_output.contains("governance_selected:"), "{}", run.terminal_output);
}
