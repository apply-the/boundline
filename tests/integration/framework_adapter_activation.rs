//! Integration coverage for explicit framework-adapter activation and pre-claim fallback.

use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use boundline::adapters::{
    FrameworkAdapterImplementationStatus, FrameworkAdapterPlanningFinding,
    FrameworkAdapterPlanningFindingSeverity, FrameworkAdapterPlanningReadinessStatus,
    FrameworkAdapterStageExecutionStatus,
};
use boundline::domain::framework_adapter::AdapterRegistrationSource;
use boundline::domain::session::SessionStatus;
use boundline::fixture::{
    sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_hook_emission_response,
    sample_framework_adapter_preflight_blocked_response,
    sample_framework_adapter_preflight_ready_response, sample_framework_adapter_success_envelope,
};
use boundline::{FileConfigStore, FileSessionStore, SessionStore};
use serde_json::json;

use crate::framework_adapter::{SPECKIT_BINARY_NAME, optional_built_speckit_binary_dir};
use crate::workspace_fixture::{
    run_boundline_in_with_env, supported_canon_path, target_test_dir, temp_fixture_workspace,
    terminal_text,
};

const FALLBACK_UNAVAILABLE_BINARY: &str = "adapter_fallback_reason: unavailable_binary";
const FALLBACK_PREFLIGHT_BLOCKED: &str = "adapter_fallback_reason: preflight_blocked";
const FALLBACK_UNSUPPORTED_TRANSPORT: &str = "adapter_fallback_reason: unsupported_transport";
const NATIVE_ROUTING_SUMMARY: &str = "routing: native (goal_plan)";
const BUG_FIX_FLOW: &str = "bug-fix";
const FIX_GOAL: &str = "fix the failing add test";

#[test]
fn discoverable_speckit_binary_is_not_auto_enabled_without_explicit_selection()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-no-auto-enable");
    let adapter_dir = target_test_dir("framework-adapter-discoverable-path");
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_ready_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains(NATIVE_ROUTING_SUMMARY), "{plan_text}");
    assert!(!stage_marker.exists(), "{plan_text}");

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(!stage_marker.exists(), "{run_text}");
    assert!(FileConfigStore::for_workspace(&workspace).local_adapter()?.is_none());

    Ok(())
}

#[test]
fn init_with_adapter_registers_speckit_profile_through_workspace_init_surface()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-init-registration");
    let adapter_dir = target_test_dir("framework-adapter-init-registration-bin");
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_ready_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let init = run_boundline_in_with_env(
        &workspace,
        &["init", "--non-interactive", "--force", "--adapter", "speckit"],
        &[("PATH", path_env.as_str())],
    );
    let init_text = terminal_text(&init);
    assert_eq!(init.status.code(), Some(0), "{init_text}");
    assert!(init_text.contains("framework_adapter_registration:"), "{init_text}");
    assert!(init_text.contains("status: ready"), "{init_text}");
    assert!(init_text.contains("adapter_id: speckit"), "{init_text}");
    assert!(init_text.contains("supported_transports: stdio/json/stdin->stdout"), "{init_text}");

    let adapter = FileConfigStore::for_workspace(&workspace)
        .local_adapter()?
        .ok_or("expected init to persist a workspace adapter selection")?;
    assert_eq!(adapter.selection.adapter_id, "speckit");
    assert_eq!(adapter.selection.registration_source, AdapterRegistrationSource::Init);

    Ok(())
}

#[test]
fn explicit_speckit_add_activates_declared_plan_stage_in_lifecycle_flow()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-explicit-activation");
    let adapter_dir = target_test_dir("framework-adapter-explicit-activation-bin");
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_ready_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let add_text = terminal_text(&add);
    assert_eq!(add.status.code(), Some(0), "{add_text}");
    assert!(FileConfigStore::for_workspace(&workspace).local_adapter()?.is_some());

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(stage_marker.exists(), "{plan_text}");
    assert!(!plan_text.contains(NATIVE_ROUTING_SUMMARY), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_execution_source: adapter"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage: plan"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage_claim: completed"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage_status: succeeded"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_workflow_id: speckit-planning"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_planning_readiness: ready"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_analyze_pass_count: 1"), "{plan_text}");
    assert!(
        plan_text.contains(
            "framework_adapter_executed_commands: speckit.specify, speckit.plan, speckit.tasks, speckit.analyze"
        ),
        "{plan_text}"
    );

    let status = run_boundline_in_with_env(&workspace, &["status"], &[("PATH", path_env.as_str())]);
    let status_text = terminal_text(&status);
    assert!(status_text.contains("framework_adapter_execution_source: adapter"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage: plan"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage_claim: completed"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage_status: succeeded"), "{status_text}");
    assert!(
        status_text.contains("framework_adapter_workflow_id: speckit-planning"),
        "{status_text}"
    );
    assert!(status_text.contains("framework_adapter_planning_readiness: ready"), "{status_text}");

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    Ok(())
}

#[test]
fn blocked_plan_stage_leaves_session_blocked_and_incomplete() -> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-blocked-plan-stage");
    let adapter_dir = target_test_dir("framework-adapter-blocked-plan-stage-bin");
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_blocked_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(1), "{plan_text}");
    assert!(stage_marker.exists(), "{plan_text}");
    assert!(!plan_text.contains(NATIVE_ROUTING_SUMMARY), "{plan_text}");
    assert!(plan_text.contains("latest_status: blocked"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage: plan"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage_claim: claimed"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage_status: blocked"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_workflow_id: speckit-planning"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_planning_readiness: blocked"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_remediation_cycles_used: 2"), "{plan_text}");

    let session = FileSessionStore::for_workspace(&workspace)
        .load()?
        .ok_or("expected a blocked session after plan")?;
    assert_eq!(session.latest_status, SessionStatus::Blocked);
    let goal_plan = session.goal_plan.ok_or("expected goal plan after blocked plan stage")?;
    assert!(goal_plan.confirmed_at.is_none(), "{plan_text}");

    Ok(())
}

#[test]
fn explicit_speckit_add_activates_declared_run_stage_in_lifecycle_flow()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-explicit-run-activation");
    let adapter_dir = target_test_dir("framework-adapter-explicit-run-activation-bin");
    let stage_marker = workspace.join("adapter-run-marker.txt");
    write_ready_run_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let add_text = terminal_text(&add);
    assert_eq!(add.status.code(), Some(0), "{add_text}");

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(!stage_marker.exists(), "{plan_text}");

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(stage_marker.exists(), "{run_text}");
    assert!(
        run_text.contains("framework_adapter_workflow_id: speckit-implementation"),
        "{run_text}"
    );
    assert!(run_text.contains("framework_adapter_implementation_status: completed"), "{run_text}");
    assert!(
        run_text.contains("framework_adapter_validation_refs: validation/run.md"),
        "{run_text}"
    );
    assert!(
        run_text.contains("framework_adapter_executed_commands: speckit.implement"),
        "{run_text}"
    );
    assert!(!run_text.contains("speckit.analyze"), "{run_text}");

    let status = run_boundline_in_with_env(&workspace, &["status"], &[("PATH", path_env.as_str())]);
    let status_text = terminal_text(&status);
    assert!(
        status_text.contains("framework_adapter_workflow_id: speckit-implementation"),
        "{status_text}"
    );
    assert!(
        status_text.contains("framework_adapter_implementation_status: completed"),
        "{status_text}"
    );
    assert!(
        status_text.contains("framework_adapter_validation_refs: validation/run.md"),
        "{status_text}"
    );

    let inspect = run_boundline_in_with_env(
        &workspace,
        &["inspect", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("framework_adapter_workflow_id: speckit-implementation"),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains("framework_adapter_implementation_status: completed"),
        "{inspect_text}"
    );

    Ok(())
}

#[test]
fn cross_repo_speckit_binary_smoke_bridges_real_specify_plan_and_completes_run()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_specify_workspace_fixture("framework-adapter-cross-repo-smoke")?;
    let Some(speckit_binary_dir) = optional_built_speckit_binary_dir()? else {
        return Ok(());
    };
    let path_env = adapter_path(&speckit_binary_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let add_text = terminal_text(&add);
    assert_eq!(add.status.code(), Some(0), "{add_text}");

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage: plan"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage_status: succeeded"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_workflow_id: speckit-planning"), "{plan_text}");
    assert!(
        plan_text.contains(
            "framework_adapter_produced_artifacts: specs/066-agentic-framework-integration/spec.md, specs/066-agentic-framework-integration/plan.md, specs/066-agentic-framework-integration/tasks.md, .specify/workflows/speckit/planning.yml"
        ),
        "{plan_text}"
    );
    assert!(plan_text.contains("framework_adapter_planning_readiness: ready"), "{plan_text}");
    assert!(!workspace.join("speckit-plan-claimed.txt").exists(), "{plan_text}");

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("latest_status: succeeded"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage: run"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage_claim: completed"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage_status: succeeded"), "{run_text}");
    assert!(
        run_text.contains("framework_adapter_workflow_id: speckit-implementation"),
        "{run_text}"
    );
    assert!(run_text.contains("framework_adapter_implementation_status: completed"), "{run_text}");
    assert!(
        run_text.contains(
            "framework_adapter_executed_commands: sh .specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks, specify workflow run .specify/workflows/speckit/implementation.yml"
        ),
        "{run_text}"
    );
    assert!(!workspace.join("speckit-run-claimed.txt").exists(), "{run_text}");

    Ok(())
}

#[test]
fn missing_selected_adapter_binary_falls_back_before_plan_stage_claim() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_fixture_workspace("framework-adapter-missing-binary-fallback");
    let adapter_dir = target_test_dir("framework-adapter-missing-binary-bin");
    let binary_path = adapter_dir.join(SPECKIT_BINARY_NAME);
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_ready_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));
    fs::remove_file(binary_path)?;

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains(NATIVE_ROUTING_SUMMARY), "{plan_text}");
    assert!(plan_text.contains(FALLBACK_UNAVAILABLE_BINARY), "{plan_text}");
    assert!(!stage_marker.exists(), "{plan_text}");

    Ok(())
}

#[test]
fn missing_selected_adapter_binary_falls_back_before_run_stage_claim() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_fixture_workspace("framework-adapter-missing-run-binary-fallback");
    let adapter_dir = target_test_dir("framework-adapter-missing-run-binary-bin");
    let binary_path = adapter_dir.join(SPECKIT_BINARY_NAME);
    let stage_marker = workspace.join("adapter-run-marker.txt");
    write_ready_run_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;
    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    ))?;

    fs::remove_file(binary_path)?;

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(!stage_marker.exists(), "{run_text}");

    let session = FileSessionStore::for_workspace(&workspace)
        .load()?
        .ok_or("expected an active session after run")?;
    let rationale =
        session.goal_plan.and_then(|goal_plan| goal_plan.planning_rationale).unwrap_or_default();
    assert!(rationale.contains(FALLBACK_UNAVAILABLE_BINARY), "{rationale}");

    Ok(())
}

#[test]
fn blocked_adapter_preflight_falls_back_before_plan_stage_claim() -> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-preflight-fallback");
    let adapter_dir = target_test_dir("framework-adapter-preflight-fallback-bin");
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_ready_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    write_blocked_preflight_plan_only_adapter(&adapter_dir, &stage_marker)?;

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains(NATIVE_ROUTING_SUMMARY), "{plan_text}");
    assert!(plan_text.contains(FALLBACK_PREFLIGHT_BLOCKED), "{plan_text}");
    assert!(!stage_marker.exists(), "{plan_text}");

    Ok(())
}

#[test]
fn unsupported_transport_adapter_falls_back_before_plan_stage_claim() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_fixture_workspace("framework-adapter-plan-transport-fallback");
    let adapter_dir = target_test_dir("framework-adapter-plan-transport-fallback-bin");
    let stage_marker = workspace.join("adapter-plan-marker.txt");
    write_ready_plan_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    write_unsupported_transport_plan_only_adapter(&adapter_dir, &stage_marker)?;

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains(NATIVE_ROUTING_SUMMARY), "{plan_text}");
    assert!(plan_text.contains(FALLBACK_UNSUPPORTED_TRANSPORT), "{plan_text}");
    assert!(!stage_marker.exists(), "{plan_text}");

    Ok(())
}

#[test]
fn blocked_adapter_preflight_falls_back_before_run_stage_claim() -> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-run-preflight-fallback");
    let adapter_dir = target_test_dir("framework-adapter-run-preflight-fallback-bin");
    let stage_marker = workspace.join("adapter-run-marker.txt");
    write_ready_run_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;
    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    ))?;

    write_blocked_preflight_run_only_adapter(&adapter_dir, &stage_marker)?;

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(!stage_marker.exists(), "{run_text}");

    let session = FileSessionStore::for_workspace(&workspace)
        .load()?
        .ok_or("expected an active session after run")?;
    let rationale =
        session.goal_plan.and_then(|goal_plan| goal_plan.planning_rationale).unwrap_or_default();
    assert!(rationale.contains(FALLBACK_PREFLIGHT_BLOCKED), "{rationale}");

    Ok(())
}

#[test]
fn unsupported_transport_adapter_falls_back_before_run_stage_claim() -> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-run-transport-fallback");
    let adapter_dir = target_test_dir("framework-adapter-run-transport-fallback-bin");
    let stage_marker = workspace.join("adapter-run-marker.txt");
    write_ready_run_only_adapter(&adapter_dir, &stage_marker)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;
    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    ))?;

    write_unsupported_transport_run_only_adapter(&adapter_dir, &stage_marker)?;

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(!stage_marker.exists(), "{run_text}");

    let session = FileSessionStore::for_workspace(&workspace)
        .load()?
        .ok_or("expected an active session after run")?;
    let rationale =
        session.goal_plan.and_then(|goal_plan| goal_plan.planning_rationale).unwrap_or_default();
    assert!(rationale.contains(FALLBACK_UNSUPPORTED_TRANSPORT), "{rationale}");

    Ok(())
}

fn assert_command_succeeds(output: std::process::Output) -> Result<(), Box<dyn Error>> {
    let rendered = terminal_text(&output);
    if output.status.code() == Some(0) {
        Ok(())
    } else {
        Err(format!("command failed: {rendered}").into())
    }
}

fn adapter_path(adapter_dir: &Path) -> String {
    format!("{}:{}", adapter_dir.display(), supported_canon_path())
}

fn write_ready_plan_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_fixture_adapter_with_execute_json(
        adapter_dir,
        stage_marker,
        ready_stage_only_describe_json("plan")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_ready_response(),
        ))?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            ready_plan_stage_execute_response(),
        ))?,
    )
}

fn write_blocked_preflight_plan_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_fixture_adapter(
        adapter_dir,
        stage_marker,
        ready_stage_only_describe_json("plan")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_blocked_response(),
        ))?,
    )
}

fn write_blocked_plan_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut execute_stage_response = sample_framework_adapter_execute_stage_success_response();
    execute_stage_response.status = FrameworkAdapterStageExecutionStatus::Blocked;
    execute_stage_response.summary =
        "framework-adapter blocked the claimed stage pending operator action".to_string();
    execute_stage_response.workflow_id = Some("speckit-planning".to_string());
    execute_stage_response.executed_commands = vec![
        "speckit.specify".to_string(),
        "speckit.plan".to_string(),
        "speckit.tasks".to_string(),
        "speckit.analyze".to_string(),
    ];
    execute_stage_response.planning_findings = vec![FrameworkAdapterPlanningFinding {
        finding_id: "F-001".to_string(),
        summary: "Blocking planning finding".to_string(),
        severity: FrameworkAdapterPlanningFindingSeverity::Blocking,
    }];
    execute_stage_response.remaining_blocking_findings =
        execute_stage_response.planning_findings.clone();
    execute_stage_response.final_planning_readiness_status =
        Some(FrameworkAdapterPlanningReadinessStatus::Blocked);
    execute_stage_response.analyze_pass_count = Some(3);
    execute_stage_response.remediation_cycles_used = Some(2);

    write_fixture_adapter_with_execute_json(
        adapter_dir,
        stage_marker,
        ready_stage_only_describe_json("plan")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_ready_response(),
        ))?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(execute_stage_response))?,
    )
}

fn write_unsupported_transport_plan_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_fixture_adapter(
        adapter_dir,
        stage_marker,
        unsupported_transport_stage_only_describe_json("plan")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_ready_response(),
        ))?,
    )
}

fn write_ready_run_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_fixture_adapter_with_execute_json(
        adapter_dir,
        stage_marker,
        ready_stage_only_describe_json("run")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_ready_response(),
        ))?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            ready_run_stage_execute_response(),
        ))?,
    )
}

fn ready_plan_stage_execute_response() -> boundline::adapters::FrameworkAdapterExecuteStageResponse
{
    let mut response = sample_framework_adapter_execute_stage_success_response();
    response.workflow_id = Some("speckit-planning".to_string());
    response.executed_commands = vec![
        "speckit.specify".to_string(),
        "speckit.plan".to_string(),
        "speckit.tasks".to_string(),
        "speckit.analyze".to_string(),
    ];
    response.planning_findings = vec![FrameworkAdapterPlanningFinding {
        finding_id: "NB-001".to_string(),
        summary: "Informational planning note".to_string(),
        severity: FrameworkAdapterPlanningFindingSeverity::NonBlocking,
    }];
    response.final_planning_readiness_status = Some(FrameworkAdapterPlanningReadinessStatus::Ready);
    response.analyze_pass_count = Some(1);
    response.remediation_cycles_used = Some(0);
    response
}

fn ready_run_stage_execute_response() -> boundline::adapters::FrameworkAdapterExecuteStageResponse {
    let mut response = sample_framework_adapter_execute_stage_success_response();
    response.workflow_id = Some("speckit-implementation".to_string());
    response.executed_commands = vec!["speckit.implement".to_string()];
    response.implementation_status = Some(FrameworkAdapterImplementationStatus::Completed);
    response.validation_refs = vec!["validation/run.md".to_string()];
    response
}

fn write_blocked_preflight_run_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_fixture_adapter(
        adapter_dir,
        stage_marker,
        ready_stage_only_describe_json("run")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_blocked_response(),
        ))?,
    )
}

fn write_unsupported_transport_run_only_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_fixture_adapter(
        adapter_dir,
        stage_marker,
        unsupported_transport_stage_only_describe_json("run")?,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_preflight_ready_response(),
        ))?,
    )
}

fn ready_stage_only_describe_json(stage_key: &str) -> Result<String, Box<dyn Error>> {
    let mut document = serde_json::to_value(sample_framework_adapter_describe_response())?;
    document["declared_stage_overrides"] = json!([stage_key]);
    Ok(serde_json::to_string(&sample_framework_adapter_success_envelope(document))?)
}

fn unsupported_transport_stage_only_describe_json(
    stage_key: &str,
) -> Result<String, Box<dyn Error>> {
    let mut document = serde_json::to_value(sample_framework_adapter_describe_response())?;
    document["declared_stage_overrides"] = json!([stage_key]);
    document["supported_transports"] = json!([
        {
            "transport": "stdio",
            "encoding": "json",
            "request_channel": "stdout",
            "response_channel": "stdout"
        }
    ]);
    Ok(serde_json::to_string(&sample_framework_adapter_success_envelope(document))?)
}

fn write_fixture_adapter(
    adapter_dir: &Path,
    stage_marker: &Path,
    describe_json: String,
    preflight_json: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let execute_stage_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_execute_stage_success_response(),
    ))?;
    write_fixture_adapter_with_execute_json(
        adapter_dir,
        stage_marker,
        describe_json,
        preflight_json,
        execute_stage_json,
    )
}

fn write_fixture_adapter_with_execute_json(
    adapter_dir: &Path,
    stage_marker: &Path,
    describe_json: String,
    preflight_json: String,
    execute_stage_json: String,
) -> Result<PathBuf, Box<dyn Error>> {
    fs::create_dir_all(adapter_dir)?;
    let emit_hook_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_hook_emission_response(),
    ))?;
    let binary_path = adapter_dir.join(SPECKIT_BINARY_NAME);
    let stage_marker_path = stage_marker.to_string_lossy();
    let script = format!(
        "#!/bin/sh\nset -eu\ncase \"$1\" in\n  describe)\n    cat <<'BOUNDLINE_JSON'\n{describe_json}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    cat <<'BOUNDLINE_JSON'\n{preflight_json}\nBOUNDLINE_JSON\n    ;;\n  execute-stage)\n    : > \"{stage_marker_path}\"\n    cat <<'BOUNDLINE_JSON'\n{execute_stage_json}\nBOUNDLINE_JSON\n    ;;\n  emit-hook)\n    cat <<'BOUNDLINE_JSON'\n{emit_hook_json}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );
    fs::write(&binary_path, script)?;
    let mut permissions = fs::metadata(&binary_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&binary_path, permissions)?;
    Ok(binary_path)
}

fn temp_specify_workspace_fixture(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let workspace = temp_fixture_workspace(prefix);
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("rsync")
        .args([
            "-a",
            "--exclude",
            ".git",
            "--exclude",
            "target",
            "--exclude",
            ".boundline",
            &format!("{}/", repo_root.display()),
            &format!("{}/", workspace.display()),
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "failed to copy Spec Kit workspace fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(workspace)
}
