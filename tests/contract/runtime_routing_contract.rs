use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use boundline::FileConfigStore;
use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::adapters::{
    FrameworkAdapterEnvelopeError, FrameworkAdapterErrorEnvelope, FrameworkAdapterFailureClass,
    FrameworkAdapterResponseEnvelope, FrameworkAdapterStageExecutionStatus,
};
use boundline::cli::CommandExitStatus;
use boundline::cli::inspect::execute_inspect;
use boundline::cli::run::{execute_custom_run, execute_native_direct_run};
use boundline::cli::session::{
    execute_goal, execute_next, execute_plan, execute_run, execute_status,
};
use boundline::domain::configuration::{
    AdapterSelectionRecord, ConfigFile, ModelRoute, PersistedAdapterConfiguration, RoutingConfig,
    RuntimeKind,
};
use boundline::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
};
use boundline::fixture::{
    sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_preflight_ready_response, sample_framework_adapter_success_envelope,
};
use serde_json::json;
use uuid::Uuid;

use crate::runtime_refoundation::{
    temp_runtime_refoundation_compat_workspace, temp_runtime_refoundation_governed_workspace,
};

#[test]
fn confirmed_goal_plan_takes_precedence_over_execution_profile_for_session_run() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-native");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

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
        None,
        false,
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

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

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
    execute_goal(
        Some(&native_workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&native_workspace), Some("bug-fix"), false).unwrap();

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
                model: "o4-mini".to_string(),
            }),
            implementation: Some(ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "gpt-4o".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(status.terminal_output.contains("route_owner: native"), "{}", status.terminal_output);
    assert!(
        status
            .terminal_output
            .contains("route_config_projection: workspace_routing: planning=codex/o4-mini, implementation=copilot/gpt-4o"),
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

    let inspect = execute_inspect(Some(Path::new(&trace_ref)), None, None, false).unwrap();

    assert!(
        inspect
            .terminal_output
            .contains("effective_routing: planning=codex/o4-mini [built-in], implementation=codex/o4-mini [built-in], verification=copilot/gpt-4.1 [built-in], review=claude/reviewer-before [workspace], adjudication=codex/o4-mini [built-in]"),
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

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
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

    let inspect = execute_inspect(Some(Path::new(&trace_ref)), None, None, false).unwrap();

    assert!(
        inspect
            .terminal_output
            .contains("effective_routing: planning=codex/o4-mini [built-in], implementation=codex/o4-mini [built-in], verification=copilot/gpt-4.1 [built-in], review=claude/reviewer-before [workspace], adjudication=codex/o4-mini [built-in]"),
        "{}",
        inspect.terminal_output
    );
    assert!(!inspect.terminal_output.contains("reviewer-after"), "{}", inspect.terminal_output);
}

#[test]
fn claimed_run_stage_failures_are_visible_in_direct_session_reports() {
    for behavior in [
        RuntimeRoutingExecuteStageBehavior::InBandFailed,
        RuntimeRoutingExecuteStageBehavior::ProtocolError,
        RuntimeRoutingExecuteStageBehavior::TransportFailure,
    ] {
        let workspace = temp_runtime_refoundation_compat_workspace(&format!(
            "runtime-routing-contract-adapter-{}",
            behavior.label()
        ));
        let adapter_path = write_runtime_routing_run_adapter(&workspace, behavior).unwrap();
        FileConfigStore::for_workspace(&workspace)
            .save_local(&ConfigFile {
                adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
                ..ConfigFile::default()
            })
            .unwrap();

        execute_goal(
            Some(&workspace),
            Some("fix the failing add test"),
            &[],
            None,
            None,
            None,
            None,
        )
        .unwrap();
        execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

        let run = execute_run(Some(&workspace)).unwrap();
        assert!(run.terminal_output.contains("terminal_status: failed"), "{}", run.terminal_output);
        assert!(
            run.terminal_output.contains(&format!(
                "framework_adapter_failure_class: {}",
                behavior.expected_failure_class()
            )),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("framework_adapter_stage_claim: failed_after_claim"),
            "{}",
            run.terminal_output
        );

        let status = execute_status(Some(&workspace)).unwrap();
        assert!(
            status.terminal_output.contains("latest_status: failed"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains(&format!(
                "framework_adapter_failure_class: {}",
                behavior.expected_failure_class()
            )),
            "{}",
            status.terminal_output
        );

        let inspect = execute_inspect(None, Some(&workspace), None, false).unwrap();
        assert!(
            inspect.terminal_output.contains("terminal_status: failed"),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains(&format!(
                "framework_adapter_failure_class: {}",
                behavior.expected_failure_class()
            )),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains("event=stage_failed"),
            "{}",
            inspect.terminal_output
        );
    }
}

#[test]
fn claimed_plan_stage_success_is_visible_in_plan_report_and_status() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-plan-stage-success");
    let adapter_path =
        write_runtime_routing_manifest_adapter(&workspace, &["plan"], &[], None, None).unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(
        status.terminal_output.contains("framework_adapter_execution_source: adapter"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage: plan"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_claim: completed"),
        "{}",
        status.terminal_output
    );
}

#[test]
fn claimed_run_stage_success_does_not_report_native_goal_plan_routing() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-run-stage-success");
    let adapter_path =
        write_runtime_routing_manifest_adapter(&workspace, &["run"], &[], None, None).unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(
        !run.terminal_output.contains("routing: native (goal_plan)"),
        "{}",
        run.terminal_output
    );
    assert!(
        run.terminal_output
            .contains("framework-adapter routed run: adapter / completed / declared_override"),
        "{}",
        run.terminal_output
    );
}

#[test]
fn adapter_without_run_override_leaves_run_stage_native() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-no-run-override");
    let run_marker = workspace.join("adapter-run-stage-marker.txt");
    let hook_marker = workspace.join("adapter-hook-marker.txt");
    let adapter_path = write_runtime_routing_manifest_adapter(
        &workspace,
        &["plan"],
        &["stage_completed"],
        Some(&run_marker),
        Some(&hook_marker),
    )
    .unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("terminal_status: succeeded"), "{}", run.terminal_output);
    assert!(!run_marker.exists(), "{}", run.terminal_output);
    assert!(
        run.terminal_output
            .contains("framework-adapter routed run: built_in / not_claimed / undeclared_stage"),
        "{}",
        run.terminal_output
    );

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(
        status.terminal_output.contains("latest_status: succeeded"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_execution_source: built_in"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_claim: not_claimed"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_routing_reason: undeclared_stage"),
        "{}",
        status.terminal_output
    );
}

#[test]
fn adapter_without_hook_subscription_skips_hook_delivery_after_claimed_success() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-no-hook-subscription");
    let run_marker = workspace.join("adapter-run-stage-marker.txt");
    let hook_marker = workspace.join("adapter-hook-marker.txt");
    let adapter_path = write_runtime_routing_manifest_adapter(
        &workspace,
        &["run"],
        &[],
        Some(&run_marker),
        Some(&hook_marker),
    )
    .unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("terminal_status: succeeded"), "{}", run.terminal_output);
    assert!(run_marker.is_file(), "{}", run.terminal_output);
    assert!(!hook_marker.exists(), "{}", run.terminal_output);
}

#[test]
fn claimed_plan_stage_failures_persist_terminal_state_for_status_and_inspect() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-plan-stage-failure");
    let adapter_path = write_runtime_routing_stage_adapter(
        &workspace,
        &["plan"],
        RuntimeRoutingExecuteStageBehavior::InBandFailed,
    )
    .unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    let error = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap_err();
    assert!(
        error.to_string().contains("framework-adapter plan stage execution failed after claim"),
        "{error}"
    );

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(status.terminal_output.contains("latest_status: failed"), "{}", status.terminal_output);
    assert!(
        status.terminal_output.contains("framework_adapter_stage: plan"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_claim: failed_after_claim"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_failure_class: adapter_runtime"),
        "{}",
        status.terminal_output
    );

    let inspect = execute_inspect(None, Some(&workspace), None, false).unwrap();
    assert!(
        inspect.terminal_output.contains("terminal_status: failed"),
        "{}",
        inspect.terminal_output
    );
    assert!(
        inspect.terminal_output.contains("framework_adapter_stage: plan"),
        "{}",
        inspect.terminal_output
    );
    assert!(
        inspect.terminal_output.contains("framework_adapter_failure_class: adapter_runtime"),
        "{}",
        inspect.terminal_output
    );
}

#[test]
fn claimed_plan_stage_blocked_returns_blocked_status_without_marking_completion() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-plan-stage-blocked");
    let adapter_path = write_runtime_routing_stage_adapter(
        &workspace,
        &["plan"],
        RuntimeRoutingExecuteStageBehavior::Blocked,
    )
    .unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    assert_eq!(plan.exit_status, CommandExitStatus::NonSuccess);
    assert!(plan.terminal_output.contains("latest_status: blocked"), "{}", plan.terminal_output);
    assert!(
        plan.terminal_output.contains("framework_adapter_stage: plan"),
        "{}",
        plan.terminal_output
    );
    assert!(
        plan.terminal_output.contains("framework_adapter_stage_claim: claimed"),
        "{}",
        plan.terminal_output
    );
    assert!(
        plan.terminal_output.contains("framework_adapter_stage_status: blocked"),
        "{}",
        plan.terminal_output
    );

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(
        status.terminal_output.contains("latest_status: blocked"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage: plan"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_claim: claimed"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_status: blocked"),
        "{}",
        status.terminal_output
    );
}

#[test]
fn claimed_run_stage_blocked_returns_blocked_status_without_marking_completion() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-run-stage-blocked");
    let adapter_path =
        write_runtime_routing_run_adapter(&workspace, RuntimeRoutingExecuteStageBehavior::Blocked)
            .unwrap();

    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            adapter: Some(runtime_routing_persisted_adapter(&adapter_path)),
            ..ConfigFile::default()
        })
        .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert_eq!(run.exit_status, CommandExitStatus::NonSuccess);
    assert!(run.terminal_output.contains("latest_status: blocked"), "{}", run.terminal_output);
    assert!(
        run.terminal_output.contains("framework_adapter_stage: run"),
        "{}",
        run.terminal_output
    );
    assert!(
        run.terminal_output.contains("framework_adapter_stage_claim: claimed"),
        "{}",
        run.terminal_output
    );
    assert!(
        run.terminal_output.contains("framework_adapter_stage_status: blocked"),
        "{}",
        run.terminal_output
    );

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(
        status.terminal_output.contains("latest_status: blocked"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage: run"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_claim: claimed"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("framework_adapter_stage_status: blocked"),
        "{}",
        status.terminal_output
    );
}

#[derive(Clone, Copy)]
enum RuntimeRoutingExecuteStageBehavior {
    Blocked,
    InBandFailed,
    ProtocolError,
    TransportFailure,
}

impl RuntimeRoutingExecuteStageBehavior {
    fn label(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::InBandFailed => "in-band-failed",
            Self::ProtocolError => "protocol-error",
            Self::TransportFailure => "transport-failure",
        }
    }

    fn expected_failure_class(self) -> &'static str {
        match self {
            Self::Blocked => "adapter_runtime",
            Self::InBandFailed => "adapter_runtime",
            Self::ProtocolError => "protocol_error",
            Self::TransportFailure => "transport_failure",
        }
    }
}

fn runtime_routing_persisted_adapter(command: &Path) -> PersistedAdapterConfiguration {
    PersistedAdapterConfiguration {
        selection: AdapterSelectionRecord {
            selection_mode: AdapterSelectionMode::Custom,
            adapter_id: "runtime-routing-test-adapter".to_string(),
            display_name: "Runtime Routing Test Adapter".to_string(),
            command: command.to_string_lossy().into_owned(),
            args: Vec::new(),
            registration_source: AdapterRegistrationSource::AdapterAdd,
            discovery_state: AdapterDiscoveryState::ExplicitCommand,
            compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
            updated_at: 1,
        },
        schema_fingerprint: "runtime-routing-contract".to_string(),
        completeness_state: AdapterConfigCompletenessState::Complete,
        interactive_resolution: false,
        last_validated_at: Some(1),
        value_count: 0,
        values: Vec::new(),
    }
}

fn write_runtime_routing_run_adapter(
    workspace: &Path,
    behavior: RuntimeRoutingExecuteStageBehavior,
) -> Result<std::path::PathBuf, String> {
    write_runtime_routing_stage_adapter(workspace, &["run"], behavior)
}

fn write_runtime_routing_stage_adapter(
    workspace: &Path,
    declared_stage_overrides: &[&str],
    behavior: RuntimeRoutingExecuteStageBehavior,
) -> Result<std::path::PathBuf, String> {
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())
        .map_err(|error| error.to_string())?;
    describe["declared_stage_overrides"] = json!(declared_stage_overrides);
    let describe_json = serde_json::to_string(&sample_framework_adapter_success_envelope(describe))
        .map_err(|error| error.to_string())?;
    let preflight_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_preflight_ready_response(),
    ))
    .map_err(|error| error.to_string())?;
    let execute_stage_script = runtime_routing_execute_stage_script(behavior)?;

    let binary_path = workspace.join(format!("adapter-{}.sh", behavior.label()));
    let script = format!(
        "#!/bin/sh\nset -eu\ncase \"$1\" in\n  describe)\n    cat <<'BOUNDLINE_JSON'\n{describe_json}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    cat <<'BOUNDLINE_JSON'\n{preflight_json}\nBOUNDLINE_JSON\n    ;;\n  execute-stage)\n{execute_stage_script}\n    ;;\n  emit-hook)\n    cat <<'BOUNDLINE_JSON'\n{}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n",
        serde_json::to_string(&sample_framework_adapter_success_envelope(json!({
            "status": "ignored",
            "summary": "hook ignored in runtime routing contract"
        })))
        .map_err(|error| error.to_string())?
    );
    fs::write(&binary_path, script).map_err(|error| error.to_string())?;
    let mut permissions =
        fs::metadata(&binary_path).map_err(|error| error.to_string())?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&binary_path, permissions).map_err(|error| error.to_string())?;
    Ok(binary_path)
}

fn write_runtime_routing_manifest_adapter(
    workspace: &Path,
    declared_stage_overrides: &[&str],
    declared_hook_subscriptions: &[&str],
    run_marker: Option<&Path>,
    hook_marker: Option<&Path>,
) -> Result<std::path::PathBuf, String> {
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())
        .map_err(|error| error.to_string())?;
    describe["declared_stage_overrides"] = json!(declared_stage_overrides);
    describe["declared_hook_subscriptions"] = json!(declared_hook_subscriptions);
    let describe_json = serde_json::to_string(&sample_framework_adapter_success_envelope(describe))
        .map_err(|error| error.to_string())?;
    let preflight_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_preflight_ready_response(),
    ))
    .map_err(|error| error.to_string())?;
    let execute_stage_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_execute_stage_success_response(),
    ))
    .map_err(|error| error.to_string())?;

    let run_marker_line = run_marker
        .map(|path| {
            format!(
                "    if printf '%s' \"$request\" | grep -q '\"stage_key\":\"run\"'; then : > \"{}\"; fi\n",
                path.display()
            )
        })
        .unwrap_or_default();
    let hook_marker_line = hook_marker
        .map(|path| format!("    cat > \"{}\"\n", path.display()))
        .unwrap_or_else(|| "    cat >/dev/null\n".to_string());

    let binary_path = workspace.join(format!("adapter-manifest-{}.sh", Uuid::new_v4()));
    let script = format!(
        "#!/bin/sh\nset -eu\ncase \"$1\" in\n  describe)\n    cat <<'BOUNDLINE_JSON'\n{describe_json}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    cat <<'BOUNDLINE_JSON'\n{preflight_json}\nBOUNDLINE_JSON\n    ;;\n  execute-stage)\n    request=\"$(cat)\"\n{run_marker_line}    cat <<'BOUNDLINE_JSON'\n{execute_stage_json}\nBOUNDLINE_JSON\n    ;;\n  emit-hook)\n{hook_marker_line}    cat <<'BOUNDLINE_JSON'\n{}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n",
        serde_json::to_string(&sample_framework_adapter_success_envelope(json!({
            "status": "delivered",
            "summary": "hook delivered"
        })))
        .map_err(|error| error.to_string())?
    );
    fs::write(&binary_path, script).map_err(|error| error.to_string())?;
    let mut permissions =
        fs::metadata(&binary_path).map_err(|error| error.to_string())?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&binary_path, permissions).map_err(|error| error.to_string())?;
    Ok(binary_path)
}

fn runtime_routing_execute_stage_script(
    behavior: RuntimeRoutingExecuteStageBehavior,
) -> Result<String, String> {
    match behavior {
        RuntimeRoutingExecuteStageBehavior::Blocked => {
            let mut response = sample_framework_adapter_execute_stage_success_response();
            response.status = FrameworkAdapterStageExecutionStatus::Blocked;
            response.summary =
                "framework-adapter blocked the claimed stage pending operator action".to_string();
            response.next_action = Some(
                "repair the adapter-owned stage inputs and rerun the claimed stage".to_string(),
            );
            let response_json =
                serde_json::to_string(&sample_framework_adapter_success_envelope(response))
                    .map_err(|error| error.to_string())?;
            Ok(format!(
                "    printf '%s\\n' 'adapter stderr: blocked claimed outcome' >&2\n    cat <<'BOUNDLINE_JSON'\n{response_json}\nBOUNDLINE_JSON"
            ))
        }
        RuntimeRoutingExecuteStageBehavior::InBandFailed => {
            let mut response = sample_framework_adapter_execute_stage_success_response();
            response.status = FrameworkAdapterStageExecutionStatus::Failed;
            response.failure_class = Some(FrameworkAdapterFailureClass::AdapterRuntime);
            response.summary = "adapter reported a claimed run-stage failure".to_string();
            let response_json =
                serde_json::to_string(&sample_framework_adapter_success_envelope(response))
                    .map_err(|error| error.to_string())?;
            Ok(format!(
                "    printf '%s\\n' 'adapter stderr: in-band failed outcome' >&2\n    cat <<'BOUNDLINE_JSON'\n{response_json}\nBOUNDLINE_JSON"
            ))
        }
        RuntimeRoutingExecuteStageBehavior::ProtocolError => {
            let response_json = serde_json::to_string(&FrameworkAdapterResponseEnvelope::<
                serde_json::Value,
            >::Error(
                FrameworkAdapterErrorEnvelope {
                    success: false,
                    error: FrameworkAdapterEnvelopeError {
                        code: "invalid_request".to_string(),
                        message: "execute-stage payload was rejected".to_string(),
                        details: None,
                    },
                },
            ))
            .map_err(|error| error.to_string())?;
            Ok(format!(
                "    printf '%s\\n' 'adapter stderr: protocol error envelope' >&2\n    cat <<'BOUNDLINE_JSON'\n{response_json}\nBOUNDLINE_JSON"
            ))
        }
        RuntimeRoutingExecuteStageBehavior::TransportFailure => {
            Ok("    printf '%s\\n' 'adapter stderr: transport failure' >&2\n    exit 23"
                .to_string())
        }
    }
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

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

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

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), None, true).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);
    assert!(!run.terminal_output.contains("governance_selected:"), "{}", run.terminal_output);
}
