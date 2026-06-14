//! Test coverage for run-stage runtime support helpers.

use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::adapters::agent::FrameworkAdapterHost;
use crate::adapters::audit_store::{FileSessionAuditStore, FrameworkAdapterHookAuditStore};
use crate::adapters::config_store::FileConfigStore;
use crate::adapters::trace_store::TraceStore;
use crate::domain::audit::{SessionAuditOutcomeStatus, SessionAuditPhase};
use crate::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, ConfigFile, PersistedAdapterConfiguration,
    RoutingConfig, RuntimeKind,
};
use crate::domain::execution::{
    ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, WorkspaceChange,
    WorkspaceExecutionProfile,
};
use crate::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterExecutionSource,
    AdapterFailureClass, AdapterHookKey, AdapterLifecycleStageKey, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    HookDispatchStatus, LifecycleStageExecutionStatus, StageClaimState, StageRoutingDecisionReason,
    StoredAdapterConfigValueState,
};
use crate::domain::goal_plan::{GoalPlan, PlannedTask};
use crate::domain::governance::{CanonMode, SystemContextBinding};
use crate::domain::reasoning::{
    ParticipantAssignment, ProfileActivationRecord, ReasoningActivationStatus,
    ReasoningActivationTrigger, ReasoningBudget, ReasoningParticipantStatus, ReasoningProfileId,
};
use crate::domain::review::{ReviewerParticipation, ReviewerParticipationStatus};
use crate::domain::trace::{TraceEvent, TraceEventType};
use crate::fixture::{
    sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_failed_response,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_hook_emission_response,
    sample_framework_adapter_preflight_blocked_response,
    sample_framework_adapter_preflight_ready_response, sample_framework_adapter_success_envelope,
};
use crate::orchestrator::FrameworkHookKey;

use super::{
    ActiveSessionRecord, FrameworkAdapterClaimedStageRuntime, FrameworkAdapterHostError,
    FrameworkAdapterRunStageOutcome, SessionAuditActor, SessionAuditActorKind, SessionRuntime,
    SessionRuntimeError, SessionStatus, TaskContext, TaskStatus, TerminalCondition,
    UPSTREAM_EVIDENCE_MAX_CHARS, append_run_stage_adapter_fallback_reason,
    apply_route_text_to_actor, canon_workspace_scope_mismatch_reason, cluster_task_status_text,
    cluster_workspace_is_blocked, configured_framework_adapter_binding,
    default_planning_system_context, effective_assistant_runtimes,
    execution_governance_read_targets, framework_adapter_host_from_selection,
    framework_adapter_run_stage_not_claimed_record, framework_adapter_run_stage_routing_record,
    framework_adapter_run_stage_routing_record_from_blocked,
    framework_adapter_run_stage_routing_record_from_failure,
    framework_adapter_stage_failure_terminal_condition, framework_adapter_stage_routing_value,
    git_config_value, governance_audit_actor, governance_route_slot_for_stage_key,
    hook_dispatch_status_from_response, map_framework_adapter_failure_class,
    missing_planning_governance_field, parse_planning_system_context, parse_three_segment_route,
    payload_string, protocol_error_code_from_host_error, read_upstream_artifact_capped,
    run_stage_hook_keys_for_status, runtime_framework_adapter_config_values,
    session_audit_outcome_for_status, session_status_for_task_status, session_status_text,
    trace_event_audit_actor, trace_event_audit_algorithm, trace_event_audit_message,
    trace_event_audit_outcome, trace_event_type_text,
};
use uuid::Uuid;

const ADAPTER_ID: &str = "speckit";
const ADAPTER_COMMAND: &str = "definitely-missing-boundline-adapter";
const ADAPTER_DISPLAY_NAME: &str = "Speckit";
const CHECKPOINT_ID: &str = "checkpoint-123";
const CHECKPOINT_RESTORE_COMMAND: &str =
    "boundline checkpoint restore checkpoint-123 --workspace .";
const CHECKPOINT_SCOPE_WORKSPACE: &str = "workspace";
const DESCRIBE_RESPONSE_FILE_NAME: &str = "describe-response.json";
const EMIT_HOOK_RESPONSE_FILE_NAME: &str = "emit-hook-response.json";
const EXECUTE_RESPONSE_FILE_NAME: &str = "execute-stage-response.json";
const FALLBACK_REASON_PREFLIGHT_BLOCKED: &str = "preflight_blocked";
const FALLBACK_REASON_UNAVAILABLE_BINARY: &str = "unavailable_binary";
const FALLBACK_REASON_UNSUPPORTED_TRANSPORT: &str = "unsupported_transport";
const FRAMEWORK_ADAPTER_SCRIPT_FILE_NAME: &str = "framework-adapter.sh";
const BOUNDLINE_SYSTEM_ID: &str = "boundline";
const DECISION_LOOP_DISPLAY_NAME: &str = "Boundline Decision Loop";
const DECISION_LOOP_ID: &str = "boundline-decision-loop";
const DEFAULT_GOVERNANCE_RUNTIME: &str = "governance";
const GIT_CONFIG_KEY_MISSING: &str = "boundline.test.missing";
const GIT_CONFIG_KEY_USER_NAME: &str = "user.name";
const GIT_CONFIG_USER_NAME_VALUE: &str = "Boundline Runtime";
const IMPLEMENTATION_STAGE_KEY: &str = "run:implementation";
const PATH_FIELD_KEY: &str = "template_repo";
const PREFLIGHT_RESPONSE_FILE_NAME: &str = "preflight-response.json";
const PROTOCOL_ERROR_CODE: &str = "stage_contract_error";
const STRING_FIELD_KEY: &str = "workspace_slug";
const BOOLEAN_FIELD_KEY: &str = "use_cache";
const INTEGER_FIELD_KEY: &str = "max_steps";
const ENUM_FIELD_KEY: &str = "mode";
const REVIEW_COUNCIL_DISPLAY_NAME: &str = "Review Council";
const REVIEW_COUNCIL_ID: &str = "review-council";
const REVIEW_MODEL_O3: &str = "o3";
const REVIEW_ROUTE_COPILOT: &str = "review:copilot:gpt-5.4";
const ROUTE_SLOT_IMPLEMENTATION: &str = "implementation";
const ROUTE_SLOT_REVIEW: &str = "review";
const SIMPLE_ROUTE_COPILOT: &str = "copilot/gpt-5.4";
const SIMPLE_ROUTE_EMPTY_MODEL: &str = "copilot/ ";
const SIMPLE_ROUTE_EMPTY_RUNTIME: &str = " /gpt-5.4";
const RUN_ID: &str = "run-123";
const SCHEMA_FINGERPRINT: &str = "schema-v1";
const SESSION_ID: &str = "session-123";
const TRACE_LOCATION: &str = "traces/run-stage.json";
const UNKNOWN_PARTICIPANT_ID: &str = "unknown-participant";
const UNKNOWN_REVIEWER_ID: &str = "unknown-reviewer";
const UPDATED_AT: u64 = 42;

#[test]
fn run_stage_runtime_support_helpers_cover_failure_routing_and_payload_serialization()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-helpers")?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let stage_runtime = sample_stage_runtime(vec![FrameworkHookKey::StageFailed]);

    let succeeded_failure = runtime.framework_adapter_stage_failure_from_execute_response(
        &stage_runtime,
        crate::adapters::FrameworkAdapterExecuteStageResponse {
            status: crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded,
            summary: "run stage unexpectedly reported success in failure helper".to_string(),
            produced_artifacts: vec!["artifact.md".to_string()],
            workflow_id: None,
            executed_commands: Vec::new(),
            planning_findings: Vec::new(),
            remediation_tasks_attempted: Vec::new(),
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: Vec::new(),
            remaining_blocking_findings: Vec::new(),
            final_planning_readiness_status: None,
            analyze_pass_count: None,
            remediation_cycles_used: None,
            implementation_status: None,
            validation_refs: Vec::new(),
            next_action: None,
            failure_class: None,
        },
    );
    assert_eq!(succeeded_failure.execution.status, LifecycleStageExecutionStatus::Succeeded);
    assert_eq!(
        succeeded_failure.execution.failure_class,
        Some(AdapterFailureClass::AdapterRuntime)
    );
    assert_eq!(
        framework_adapter_stage_failure_terminal_condition(&succeeded_failure),
        TerminalCondition::TaskNotCredible
    );

    let failed_failure = runtime.framework_adapter_stage_failure_from_execute_response(
        &stage_runtime,
        crate::adapters::FrameworkAdapterExecuteStageResponse {
            status: crate::adapters::FrameworkAdapterStageExecutionStatus::Failed,
            summary: "adapter failed the run stage".to_string(),
            produced_artifacts: Vec::new(),
            workflow_id: None,
            executed_commands: Vec::new(),
            planning_findings: Vec::new(),
            remediation_tasks_attempted: Vec::new(),
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: Vec::new(),
            remaining_blocking_findings: Vec::new(),
            final_planning_readiness_status: None,
            analyze_pass_count: None,
            remediation_cycles_used: None,
            implementation_status: None,
            validation_refs: Vec::new(),
            next_action: None,
            failure_class: None,
        },
    );
    assert_eq!(failed_failure.execution.status, LifecycleStageExecutionStatus::Failed);
    assert_eq!(failed_failure.claim_state, StageClaimState::FailedAfterClaim);
    assert_eq!(
        framework_adapter_stage_failure_terminal_condition(&failed_failure),
        TerminalCondition::TaskNotCredible
    );

    let blocked_failure = runtime.framework_adapter_stage_failure_from_execute_response(
        &stage_runtime,
        crate::adapters::FrameworkAdapterExecuteStageResponse {
            status: crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked,
            summary: "adapter blocked the run stage".to_string(),
            produced_artifacts: vec!["artifact.md".to_string()],
            workflow_id: Some("speckit-implementation".to_string()),
            executed_commands: vec!["specify workflow run speckit-implementation".to_string()],
            planning_findings: Vec::new(),
            remediation_tasks_attempted: Vec::new(),
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: Vec::new(),
            remaining_blocking_findings: Vec::new(),
            final_planning_readiness_status: None,
            analyze_pass_count: None,
            remediation_cycles_used: None,
            implementation_status: Some(
                crate::adapters::FrameworkAdapterImplementationStatus::Blocked,
            ),
            validation_refs: vec!["validation/run.md".to_string()],
            next_action: Some("resume run stage".to_string()),
            failure_class: None,
        },
    );
    assert_eq!(blocked_failure.execution.status, LifecycleStageExecutionStatus::Blocked);

    let blocked = runtime.framework_adapter_stage_blocked_from_execute_response(
        &stage_runtime,
        crate::adapters::FrameworkAdapterExecuteStageResponse {
            status: crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked,
            summary: "adapter blocked the run stage".to_string(),
            produced_artifacts: vec!["artifact.md".to_string()],
            workflow_id: Some("speckit-implementation".to_string()),
            executed_commands: vec!["specify workflow run speckit-implementation".to_string()],
            planning_findings: Vec::new(),
            remediation_tasks_attempted: Vec::new(),
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: Vec::new(),
            remaining_blocking_findings: Vec::new(),
            final_planning_readiness_status: None,
            analyze_pass_count: None,
            remediation_cycles_used: None,
            implementation_status: Some(
                crate::adapters::FrameworkAdapterImplementationStatus::Blocked,
            ),
            validation_refs: vec!["validation/run.md".to_string()],
            next_action: Some("resume run stage".to_string()),
            failure_class: None,
        },
    );
    assert_eq!(blocked.claim_state, StageClaimState::Claimed);
    assert_eq!(blocked.execution.status, LifecycleStageExecutionStatus::Blocked);
    assert_eq!(blocked.detail.as_deref(), Some("resume run stage"));
    assert_eq!(
        framework_adapter_stage_failure_terminal_condition(&blocked),
        TerminalCondition::NoCredibleNextStep
    );

    let protocol_failure = runtime.framework_adapter_stage_failure_from_host_error(
        &stage_runtime,
        FrameworkAdapterHostError::ProtocolError {
            command: ADAPTER_COMMAND.to_string(),
            request_kind: "execute-stage".to_string(),
            code: PROTOCOL_ERROR_CODE.to_string(),
            message: "invalid stage envelope".to_string(),
            details: None,
        },
    );
    assert_eq!(protocol_failure.execution.failure_class, Some(AdapterFailureClass::ProtocolError));
    assert_eq!(protocol_failure.protocol_error_code.as_deref(), Some(PROTOCOL_ERROR_CODE));
    assert!(protocol_failure.summary.contains(&format!("code={PROTOCOL_ERROR_CODE}")));
    assert_eq!(
        framework_adapter_stage_failure_terminal_condition(&protocol_failure),
        TerminalCondition::UnrecoverableError
    );

    let transport_failure = runtime.framework_adapter_stage_failure_from_host_error(
        &stage_runtime,
        FrameworkAdapterHostError::ProcessFailed {
            command: ADAPTER_COMMAND.to_string(),
            request_kind: "execute-stage".to_string(),
            detail: "transport failed".to_string(),
        },
    );
    assert_eq!(
        transport_failure.execution.failure_class,
        Some(AdapterFailureClass::TransportFailure)
    );
    assert!(
        transport_failure
            .summary
            .contains("framework-adapter transport failed after claiming run stage")
    );
    assert_eq!(
        framework_adapter_stage_failure_terminal_condition(&transport_failure),
        TerminalCondition::UnrecoverableError
    );

    let success_routing = framework_adapter_run_stage_routing_record(
        &stage_runtime,
        StageClaimState::Completed,
        Some(LifecycleStageExecutionStatus::Succeeded),
        vec!["artifact.md".to_string()],
        None,
    );
    assert_eq!(success_routing.execution_source, AdapterExecutionSource::Adapter);
    assert_eq!(success_routing.claim_state, StageClaimState::Completed);
    assert_eq!(success_routing.stage_status, Some(LifecycleStageExecutionStatus::Succeeded));

    let failure_payload =
        framework_adapter_run_stage_routing_record_from_failure(&transport_failure);
    assert_eq!(
        failure_payload.framework_adapter_stage_routing.claim_state,
        StageClaimState::FailedAfterClaim
    );
    assert_eq!(
        failure_payload.framework_adapter_stage_routing.stage_status,
        Some(LifecycleStageExecutionStatus::Failed)
    );
    assert!(failure_payload.summary.contains("run stage as failed_after_claim"));
    let failure_value = framework_adapter_stage_routing_value(failure_payload)?;
    assert_eq!(
        failure_value["framework_adapter_stage_routing"]["stage_status"],
        serde_json::json!("failed")
    );

    let blocked_routing = framework_adapter_run_stage_routing_record_from_blocked(&blocked);
    assert_eq!(blocked_routing.stage_status, Some(LifecycleStageExecutionStatus::Blocked));
    assert_eq!(blocked_routing.claim_state, StageClaimState::Claimed);

    let not_claimed = framework_adapter_run_stage_not_claimed_record(
        &sample_session(workspace.as_path()),
        Some(ADAPTER_ID.to_string()),
        StageRoutingDecisionReason::CompatibilityBlocked,
    );
    assert_eq!(not_claimed.execution_source, AdapterExecutionSource::BuiltIn);
    assert_eq!(not_claimed.claim_state, StageClaimState::NotClaimed);
    assert_eq!(not_claimed.decision_reason, StageRoutingDecisionReason::CompatibilityBlocked);

    assert_eq!(
        run_stage_hook_keys_for_status(TaskStatus::Succeeded),
        Some((FrameworkHookKey::StageCompleted, AdapterHookKey::StageCompleted))
    );
    assert_eq!(run_stage_hook_keys_for_status(TaskStatus::Running), None);
    assert_eq!(
        hook_dispatch_status_from_response(
            crate::adapters::FrameworkAdapterHookDeliveryStatus::Delivered,
        ),
        HookDispatchStatus::Delivered
    );
    assert_eq!(
        hook_dispatch_status_from_response(
            crate::adapters::FrameworkAdapterHookDeliveryStatus::Ignored,
        ),
        HookDispatchStatus::Ignored
    );
    assert_eq!(
        hook_dispatch_status_from_response(
            crate::adapters::FrameworkAdapterHookDeliveryStatus::Warning,
        ),
        HookDispatchStatus::Warning
    );
    assert_eq!(
        hook_dispatch_status_from_response(
            crate::adapters::FrameworkAdapterHookDeliveryStatus::Failed,
        ),
        HookDispatchStatus::Failed
    );

    Ok(())
}

#[test]
fn run_stage_runtime_support_hook_dispatch_records_failures_for_subscribed_terminal_stages()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-hook-dispatch")?;
    FileConfigStore::for_workspace(workspace.as_path()).save_local(&ConfigFile {
        adapter: Some(sample_adapter_selection(ADAPTER_COMMAND)),
        ..ConfigFile::default()
    })?;

    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let session = sample_session(workspace.as_path());
    let audit_store = FileSessionAuditStore::for_session(workspace.as_path(), SESSION_ID);

    runtime.emit_framework_adapter_run_stage_hook(
        &session,
        &sample_stage_runtime(Vec::new()),
        TaskStatus::Succeeded,
        TRACE_LOCATION,
    )?;
    assert!(audit_store.load_hook_dispatches()?.is_empty());

    runtime.emit_framework_adapter_run_stage_hook(
        &session,
        &sample_stage_runtime(vec![FrameworkHookKey::StageFailed]),
        TaskStatus::Running,
        TRACE_LOCATION,
    )?;
    assert!(audit_store.load_hook_dispatches()?.is_empty());

    let no_selection_workspace = temp_workspace("boundline-runtime-support-hook-no-selection")?;
    let no_selection_runtime = SessionRuntime::for_workspace(no_selection_workspace.as_path());
    let no_selection_session = sample_session(no_selection_workspace.as_path());
    let no_selection_audit_store =
        FileSessionAuditStore::for_session(no_selection_workspace.as_path(), SESSION_ID);
    no_selection_runtime.emit_framework_adapter_run_stage_hook(
        &no_selection_session,
        &sample_stage_runtime(vec![FrameworkHookKey::StageFailed]),
        TaskStatus::Failed,
        TRACE_LOCATION,
    )?;
    assert!(no_selection_audit_store.load_hook_dispatches()?.is_empty());

    let success_workspace = temp_workspace("boundline-runtime-support-hook-success")?;
    let success_script = write_framework_adapter_script(
        success_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    save_local_adapter(success_workspace.as_path(), sample_adapter_selection(&success_script))?;
    let success_runtime = SessionRuntime::for_workspace(success_workspace.as_path());
    let success_session = sample_session(success_workspace.as_path());
    let success_audit_store =
        FileSessionAuditStore::for_session(success_workspace.as_path(), SESSION_ID);
    success_runtime.emit_framework_adapter_run_stage_hook(
        &success_session,
        &sample_stage_runtime(vec![FrameworkHookKey::StageCompleted]),
        TaskStatus::Succeeded,
        TRACE_LOCATION,
    )?;
    let success_records = success_audit_store.load_hook_dispatches()?;
    assert_eq!(success_records.len(), 1);
    assert_eq!(success_records[0].dispatch_status, HookDispatchStatus::Delivered);

    runtime.emit_framework_adapter_run_stage_hook(
        &session,
        &sample_stage_runtime(vec![FrameworkHookKey::StageFailed]),
        TaskStatus::Failed,
        TRACE_LOCATION,
    )?;

    let records = audit_store.load_hook_dispatches()?;
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.run_id, SESSION_ID);
    assert_eq!(record.hook_key, AdapterHookKey::StageFailed);
    assert_eq!(record.stage_key, AdapterLifecycleStageKey::Run);
    assert_eq!(record.adapter_id, ADAPTER_ID);
    assert_eq!(record.payload_ref, TRACE_LOCATION);
    assert_eq!(record.dispatch_status, HookDispatchStatus::Failed);
    assert!(record.summary.contains("framework-adapter hook delivery failed"));

    Ok(())
}

#[test]
fn run_stage_runtime_support_binding_failures_surface_execution_invariants()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-binding-errors")?;
    save_local_adapter(workspace.as_path(), sample_adapter_selection(""))?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let mut session = sample_goal_plan_session(workspace.as_path())?;

    let execute_error = runtime
        .maybe_execute_framework_adapter_run_stage(&mut session, None)
        .expect_err("expected binding error for empty adapter command");
    assert!(execute_error.to_string().contains("failed to load framework-adapter runtime binding"));

    let hook_error = runtime
        .emit_framework_adapter_run_stage_hook(
            &sample_session(workspace.as_path()),
            &sample_stage_runtime(vec![FrameworkHookKey::StageFailed]),
            TaskStatus::Failed,
            TRACE_LOCATION,
        )
        .expect_err("expected hook binding error for empty adapter command");
    assert!(hook_error.to_string().contains("failed to reload framework-adapter runtime binding"));

    Ok(())
}

#[test]
fn run_stage_runtime_support_records_not_claimed_routing_into_trace() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_workspace("boundline-runtime-support-not-claimed")?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let session = sample_goal_plan_session(workspace.as_path())?;
    let goal_plan = session.goal_plan.as_ref().ok_or("missing goal plan")?;
    let mut trace = runtime.build_goal_plan_trace(SESSION_ID, goal_plan);
    let trace_location = runtime.persist_trace(SESSION_ID, &mut trace)?;
    let routing_record = framework_adapter_run_stage_not_claimed_record(
        &session,
        Some(ADAPTER_ID.to_string()),
        StageRoutingDecisionReason::CompatibilityBlocked,
    );

    runtime.record_framework_adapter_run_stage_not_claimed_routing(
        &session,
        &trace_location,
        goal_plan.proposal_revision,
        routing_record,
    )?;

    let trace = runtime.trace_store().load(Path::new(&trace_location))?;
    let routed_event = trace.events.last().ok_or("missing routed event")?;
    assert_eq!(routed_event.event_type, TraceEventType::StageRouted);
    assert_eq!(routed_event.plan_revision, goal_plan.proposal_revision);
    assert_eq!(
        routed_event.payload["framework_adapter_stage_routing"]["claim_state"],
        serde_json::json!("not_claimed")
    );
    assert_eq!(
        routed_event.payload["framework_adapter_stage_routing"]["decision_reason"],
        serde_json::json!("compatibility_blocked")
    );

    Ok(())
}

#[test]
fn run_stage_runtime_support_persists_blocked_stage_terminal_state() -> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-blocked")?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let mut session = sample_goal_plan_session(workspace.as_path())?;
    let checkpoint_projection = sample_checkpoint_projection(workspace.as_path());
    let blocked = runtime.framework_adapter_stage_blocked_from_execute_response(
        &sample_stage_runtime(Vec::new()),
        crate::adapters::FrameworkAdapterExecuteStageResponse {
            status: crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked,
            summary: "operator approval required".to_string(),
            produced_artifacts: vec!["artifact.md".to_string()],
            workflow_id: Some("speckit-implementation".to_string()),
            executed_commands: vec!["specify workflow run speckit-implementation".to_string()],
            planning_findings: Vec::new(),
            remediation_tasks_attempted: Vec::new(),
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: Vec::new(),
            remaining_blocking_findings: Vec::new(),
            final_planning_readiness_status: None,
            analyze_pass_count: None,
            remediation_cycles_used: None,
            implementation_status: Some(
                crate::adapters::FrameworkAdapterImplementationStatus::Blocked,
            ),
            validation_refs: vec!["validation/run.md".to_string()],
            next_action: Some("grant approval".to_string()),
            failure_class: None,
        },
    );

    let response = runtime.persist_framework_adapter_run_stage_blocked(
        &mut session,
        Some(checkpoint_projection.clone()),
        blocked,
    )?;

    assert_eq!(response.terminal_status, TaskStatus::Failed);
    assert_eq!(response.terminal_reason.condition, TerminalCondition::NoCredibleNextStep);
    assert_eq!(session.latest_status, SessionStatus::Blocked);
    assert_eq!(session.latest_trace_ref.as_deref(), Some(response.trace_location.as_str()));
    assert!(session.active_task.is_none());
    assert_eq!(
        response.final_context.state.get("latest_checkpoint_id"),
        Some(&json!(CHECKPOINT_ID))
    );

    let trace = runtime.trace_store().load(Path::new(&response.trace_location))?;
    let routed_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::StageRouted)
        .ok_or("missing blocked routed event")?;
    assert_eq!(routed_event.event_type, TraceEventType::StageRouted);
    assert_eq!(
        routed_event.payload["framework_adapter_stage_routing"]["stage_status"],
        serde_json::json!("blocked")
    );
    assert_eq!(
        routed_event.payload["framework_adapter_stage_routing"]["claim_state"],
        serde_json::json!("claimed")
    );
    let checkpoint_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::CheckpointCreated)
        .ok_or("missing checkpoint event")?;
    assert_eq!(checkpoint_event.payload["checkpoint_id"], json!(CHECKPOINT_ID));

    Ok(())
}

#[test]
fn run_stage_runtime_support_persists_success_stage_terminal_state() -> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-success")?;
    write_goal_plan_target_file(workspace.as_path())?;
    write_execution_profile(workspace.as_path())?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let mut session = sample_goal_plan_session(workspace.as_path())?;

    let response = runtime.persist_framework_adapter_run_stage_success(
        &mut session,
        None,
        &sample_stage_runtime(Vec::new()),
        crate::adapters::FrameworkAdapterExecuteStageResponse {
            status: crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded,
            summary: "run stage completed".to_string(),
            produced_artifacts: vec!["artifact.md".to_string()],
            workflow_id: Some("speckit-implementation".to_string()),
            executed_commands: vec!["specify workflow run speckit-implementation".to_string()],
            planning_findings: Vec::new(),
            remediation_tasks_attempted: Vec::new(),
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: Vec::new(),
            remaining_blocking_findings: Vec::new(),
            final_planning_readiness_status: None,
            analyze_pass_count: None,
            remediation_cycles_used: None,
            implementation_status: Some(
                crate::adapters::FrameworkAdapterImplementationStatus::Completed,
            ),
            validation_refs: vec!["validation/run.md".to_string()],
            next_action: None,
            failure_class: None,
        },
    )?;

    assert_eq!(response.terminal_status, TaskStatus::Running);
    assert_eq!(response.terminal_reason.condition, TerminalCondition::GoalSatisfied);
    assert!(session.active_task.is_some());

    let trace = runtime.trace_store().load(Path::new(&response.trace_location))?;
    let routed_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::StageRouted)
        .ok_or("missing success routed event")?;
    assert_eq!(
        routed_event.payload["framework_adapter_stage_routing"]["stage_status"],
        serde_json::json!("succeeded")
    );

    Ok(())
}

#[test]
fn run_stage_runtime_support_persists_failure_stage_trace_payload() -> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-failure")?;
    write_goal_plan_target_file(workspace.as_path())?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let mut session = sample_goal_plan_session(workspace.as_path())?;
    let failure = runtime.framework_adapter_stage_failure_from_host_error(
        &sample_stage_runtime(Vec::new()),
        FrameworkAdapterHostError::ProtocolError {
            command: ADAPTER_COMMAND.to_string(),
            request_kind: "execute-stage".to_string(),
            code: PROTOCOL_ERROR_CODE.to_string(),
            message: "invalid stage envelope".to_string(),
            details: None,
        },
    );

    let response =
        runtime.persist_framework_adapter_run_stage_failure(&mut session, None, failure)?;

    assert_eq!(response.terminal_status, TaskStatus::Failed);
    assert_eq!(response.terminal_reason.condition, TerminalCondition::UnrecoverableError);
    assert_eq!(session.latest_status, SessionStatus::Failed);

    let trace = runtime.trace_store().load(Path::new(&response.trace_location))?;
    let routed_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::StageRouted)
        .ok_or("missing failure routed event")?;
    assert_eq!(
        routed_event.payload["framework_adapter_stage_routing"]["stage_status"],
        serde_json::json!("failed")
    );
    let failed_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::StageFailed)
        .ok_or("missing failure stage event")?;
    assert_eq!(
        failed_event.payload["framework_adapter_stage_failure"]["protocol_error_code"],
        serde_json::json!(PROTOCOL_ERROR_CODE)
    );

    Ok(())
}

#[test]
fn run_stage_runtime_support_maybe_execute_run_stage_covers_not_claimed_paths()
-> Result<(), Box<dyn Error>> {
    let missing_selection_workspace = temp_workspace("boundline-runtime-support-no-selection")?;
    let missing_selection_runtime =
        SessionRuntime::for_workspace(missing_selection_workspace.as_path());
    let mut missing_selection_session =
        sample_goal_plan_session(missing_selection_workspace.as_path())?;
    assert!(configured_framework_adapter_binding(missing_selection_workspace.as_path())?.is_none());
    assert!(matches!(
        missing_selection_runtime
            .maybe_execute_framework_adapter_run_stage(&mut missing_selection_session, None)?,
        FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: None }
    ));

    let missing_binary_workspace = temp_workspace("boundline-runtime-support-missing-binary")?;
    save_local_adapter(
        missing_binary_workspace.as_path(),
        sample_adapter_selection(ADAPTER_COMMAND),
    )?;
    let missing_binary_runtime = SessionRuntime::for_workspace(missing_binary_workspace.as_path());
    let mut missing_binary_session = sample_goal_plan_session(missing_binary_workspace.as_path())?;
    match missing_binary_runtime
        .maybe_execute_framework_adapter_run_stage(&mut missing_binary_session, None)?
    {
        FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: Some(routing_record) } => {
            assert_eq!(
                routing_record.decision_reason,
                StageRoutingDecisionReason::CompatibilityBlocked
            );
            assert_eq!(routing_record.adapter_id.as_deref(), Some(ADAPTER_ID));
        }
        _ => return Err("expected missing-binary fallback outcome".into()),
    }
    assert!(goal_plan_has_fallback_reason(
        &missing_binary_session,
        FALLBACK_REASON_UNAVAILABLE_BINARY,
    ));

    let undeclared_workspace = temp_workspace("boundline-runtime-support-undeclared")?;
    let mut undeclared_describe = sample_framework_adapter_describe_response();
    undeclared_describe
        .declared_stage_overrides
        .retain(|stage| *stage != crate::orchestrator::FrameworkStageKey::Run);
    let undeclared_script = write_framework_adapter_script(
        undeclared_workspace.as_path(),
        &undeclared_describe,
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    save_local_adapter(
        undeclared_workspace.as_path(),
        sample_adapter_selection(&undeclared_script),
    )?;
    let undeclared_runtime = SessionRuntime::for_workspace(undeclared_workspace.as_path());
    let mut undeclared_session = sample_goal_plan_session(undeclared_workspace.as_path())?;
    match undeclared_runtime
        .maybe_execute_framework_adapter_run_stage(&mut undeclared_session, None)?
    {
        FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: Some(routing_record) } => {
            assert_eq!(routing_record.decision_reason, StageRoutingDecisionReason::UndeclaredStage);
            assert_eq!(routing_record.adapter_id.as_deref(), Some(ADAPTER_ID));
        }
        _ => return Err("expected undeclared-stage fallback outcome".into()),
    }
    assert!(!goal_plan_has_any_fallback_reason(&undeclared_session));

    let unsupported_workspace = temp_workspace("boundline-runtime-support-unsupported-transport")?;
    let mut unsupported_describe = sample_framework_adapter_describe_response();
    unsupported_describe.supported_transports = Vec::new();
    let unsupported_script = write_framework_adapter_script(
        unsupported_workspace.as_path(),
        &unsupported_describe,
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    save_local_adapter(
        unsupported_workspace.as_path(),
        sample_adapter_selection(&unsupported_script),
    )?;
    let unsupported_runtime = SessionRuntime::for_workspace(unsupported_workspace.as_path());
    let mut unsupported_session = sample_goal_plan_session(unsupported_workspace.as_path())?;
    match unsupported_runtime
        .maybe_execute_framework_adapter_run_stage(&mut unsupported_session, None)?
    {
        FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: Some(routing_record) } => {
            assert_eq!(
                routing_record.decision_reason,
                StageRoutingDecisionReason::CompatibilityBlocked
            );
            assert_eq!(routing_record.adapter_id.as_deref(), Some(ADAPTER_ID));
        }
        _ => return Err("expected unsupported-transport fallback outcome".into()),
    }
    assert!(goal_plan_has_fallback_reason(
        &unsupported_session,
        FALLBACK_REASON_UNSUPPORTED_TRANSPORT,
    ));

    let preflight_failure_workspace =
        temp_workspace("boundline-runtime-support-preflight-failure")?;
    let preflight_failure_script = write_framework_adapter_script(
        preflight_failure_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::ProcessFailure,
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    save_local_adapter(
        preflight_failure_workspace.as_path(),
        sample_adapter_selection(&preflight_failure_script),
    )?;
    let preflight_failure_runtime =
        SessionRuntime::for_workspace(preflight_failure_workspace.as_path());
    let mut preflight_failure_session =
        sample_goal_plan_session(preflight_failure_workspace.as_path())?;
    match preflight_failure_runtime
        .maybe_execute_framework_adapter_run_stage(&mut preflight_failure_session, None)?
    {
        FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: Some(routing_record) } => {
            assert_eq!(
                routing_record.decision_reason,
                StageRoutingDecisionReason::CompatibilityBlocked
            );
            assert_eq!(routing_record.adapter_id.as_deref(), Some(ADAPTER_ID));
        }
        _ => return Err("expected preflight-failure fallback outcome".into()),
    }
    assert!(goal_plan_has_fallback_reason(
        &preflight_failure_session,
        FALLBACK_REASON_UNAVAILABLE_BINARY,
    ));

    let preflight_blocked_workspace =
        temp_workspace("boundline-runtime-support-preflight-blocked")?;
    let preflight_blocked_script = write_framework_adapter_script(
        preflight_blocked_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(sample_framework_adapter_preflight_blocked_response()),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    save_local_adapter(
        preflight_blocked_workspace.as_path(),
        sample_adapter_selection(&preflight_blocked_script),
    )?;
    let preflight_blocked_runtime =
        SessionRuntime::for_workspace(preflight_blocked_workspace.as_path());
    let mut preflight_blocked_session =
        sample_goal_plan_session(preflight_blocked_workspace.as_path())?;
    match preflight_blocked_runtime
        .maybe_execute_framework_adapter_run_stage(&mut preflight_blocked_session, None)?
    {
        FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: Some(routing_record) } => {
            assert_eq!(
                routing_record.decision_reason,
                StageRoutingDecisionReason::PreflightBlocked
            );
            assert_eq!(routing_record.adapter_id.as_deref(), Some(ADAPTER_ID));
        }
        _ => return Err("expected preflight-blocked fallback outcome".into()),
    }
    assert!(goal_plan_has_fallback_reason(
        &preflight_blocked_session,
        FALLBACK_REASON_PREFLIGHT_BLOCKED,
    ));

    Ok(())
}

#[test]
fn run_stage_runtime_support_maybe_execute_run_stage_covers_claimed_paths()
-> Result<(), Box<dyn Error>> {
    let success_workspace = temp_workspace("boundline-runtime-support-claimed-success")?;
    write_goal_plan_target_file(success_workspace.as_path())?;
    let mut empty_normalized_preflight = sample_framework_adapter_preflight_ready_response();
    empty_normalized_preflight.normalized_config_values.clear();
    let success_script = write_framework_adapter_script(
        success_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(empty_normalized_preflight),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    save_local_adapter(success_workspace.as_path(), sample_adapter_selection(&success_script))?;
    let success_runtime = SessionRuntime::for_workspace(success_workspace.as_path());
    let mut success_session = sample_goal_plan_session(success_workspace.as_path())?;
    match success_runtime.maybe_execute_framework_adapter_run_stage(&mut success_session, None)? {
        FrameworkAdapterRunStageOutcome::Completed { stage_runtime, response } => {
            assert_eq!(stage_runtime.adapter_id, ADAPTER_ID);
            assert_eq!(
                response.status,
                crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded
            );
            assert_eq!(
                response.produced_artifacts,
                vec![
                    "specs/066-agentic-framework-integration/plan.md".to_string(),
                    "specs/066-agentic-framework-integration/tasks.md".to_string(),
                ]
            );
        }
        _ => return Err("expected claimed success outcome".into()),
    }

    let blocked_workspace = temp_workspace("boundline-runtime-support-claimed-blocked")?;
    write_goal_plan_target_file(blocked_workspace.as_path())?;
    let blocked_script = write_framework_adapter_script(
        blocked_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::Response(blocked_execute_stage_response()),
    )?;
    save_local_adapter(blocked_workspace.as_path(), sample_adapter_selection(&blocked_script))?;
    let blocked_runtime = SessionRuntime::for_workspace(blocked_workspace.as_path());
    let mut blocked_session = sample_goal_plan_session(blocked_workspace.as_path())?;
    match blocked_runtime.maybe_execute_framework_adapter_run_stage(&mut blocked_session, None)? {
        FrameworkAdapterRunStageOutcome::Blocked(blocked) => {
            assert_eq!(blocked.claim_state, StageClaimState::Claimed);
            assert_eq!(blocked.execution.status, LifecycleStageExecutionStatus::Blocked);
            assert_eq!(blocked.detail.as_deref(), Some("resume run stage"));
        }
        _ => return Err("expected claimed blocked outcome".into()),
    }

    let failed_workspace = temp_workspace("boundline-runtime-support-claimed-failed")?;
    write_goal_plan_target_file(failed_workspace.as_path())?;
    let failed_script = write_framework_adapter_script(
        failed_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_failed_response()),
    )?;
    save_local_adapter(failed_workspace.as_path(), sample_adapter_selection(&failed_script))?;
    let failed_runtime = SessionRuntime::for_workspace(failed_workspace.as_path());
    let mut failed_session = sample_goal_plan_session(failed_workspace.as_path())?;
    match failed_runtime.maybe_execute_framework_adapter_run_stage(&mut failed_session, None)? {
        FrameworkAdapterRunStageOutcome::Terminal { stage_runtime, response } => {
            assert_eq!(stage_runtime.adapter_id, ADAPTER_ID);
            assert_eq!(response.terminal_status, TaskStatus::Failed);
            assert_eq!(response.terminal_reason.condition, TerminalCondition::TaskNotCredible);
            assert_eq!(failed_session.latest_status, SessionStatus::Failed);
            assert!(failed_session.latest_trace_ref.is_some());
        }
        _ => return Err("expected claimed failure outcome".into()),
    }

    let transport_failure_workspace =
        temp_workspace("boundline-runtime-support-claimed-transport-failure")?;
    write_goal_plan_target_file(transport_failure_workspace.as_path())?;
    let transport_failure_script = write_framework_adapter_script(
        transport_failure_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::ProcessFailure,
    )?;
    save_local_adapter(
        transport_failure_workspace.as_path(),
        sample_adapter_selection(&transport_failure_script),
    )?;
    let transport_failure_runtime =
        SessionRuntime::for_workspace(transport_failure_workspace.as_path());
    let mut transport_failure_session =
        sample_goal_plan_session(transport_failure_workspace.as_path())?;
    match transport_failure_runtime
        .maybe_execute_framework_adapter_run_stage(&mut transport_failure_session, None)?
    {
        FrameworkAdapterRunStageOutcome::Terminal { stage_runtime, response } => {
            assert_eq!(stage_runtime.adapter_id, ADAPTER_ID);
            assert_eq!(response.terminal_status, TaskStatus::Failed);
            assert_eq!(response.terminal_reason.condition, TerminalCondition::UnrecoverableError);
            assert_eq!(transport_failure_session.latest_status, SessionStatus::Failed);
        }
        _ => return Err("expected claimed transport-failure outcome".into()),
    }

    Ok(())
}

#[test]
fn run_stage_runtime_support_persistence_helpers_report_missing_goal_plan()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-missing-goal-plan")?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let mut session = sample_session(workspace.as_path());
    let stage_runtime = sample_stage_runtime(Vec::new());
    let blocked = runtime.framework_adapter_stage_blocked_from_execute_response(
        &stage_runtime,
        blocked_execute_stage_response(),
    );
    let failure = runtime.framework_adapter_stage_failure_from_host_error(
        &stage_runtime,
        FrameworkAdapterHostError::ProcessFailed {
            command: ADAPTER_COMMAND.to_string(),
            request_kind: "execute-stage".to_string(),
            detail: "transport failed".to_string(),
        },
    );

    assert!(matches!(
        runtime.persist_framework_adapter_run_stage_failure(&mut session, None, failure),
        Err(SessionRuntimeError::MissingGoalPlan)
    ));
    assert!(matches!(
        runtime.persist_framework_adapter_run_stage_success(
            &mut session,
            None,
            &stage_runtime,
            sample_framework_adapter_execute_stage_success_response(),
        ),
        Err(SessionRuntimeError::MissingGoalPlan)
    ));
    assert!(matches!(
        runtime.persist_framework_adapter_run_stage_blocked(&mut session, None, blocked),
        Err(SessionRuntimeError::MissingGoalPlan)
    ));

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_binding_config_and_git_helpers()
-> Result<(), Box<dyn Error>> {
    let binding_workspace = temp_workspace("boundline-runtime-support-binding-helpers")?;
    let adapter_script = write_framework_adapter_script(
        binding_workspace.as_path(),
        &sample_framework_adapter_describe_response(),
        PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
        ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
    )?;
    let mut selection = sample_adapter_selection(&adapter_script);
    selection.values = vec![
        AdapterConfigValueRecord {
            field_key: STRING_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::String,
            secret: false,
            string_value: Some("boundline-runtime".to_string()),
            path_value: None,
            bool_value: None,
            int_value: None,
            value_source: AdapterValueSource::CliFlag,
            resolution_state: StoredAdapterConfigValueState::Present,
        },
        AdapterConfigValueRecord {
            field_key: PATH_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::Path,
            secret: false,
            string_value: None,
            path_value: Some("../boundline-framework-template".to_string()),
            bool_value: None,
            int_value: None,
            value_source: AdapterValueSource::OperatorPrompt,
            resolution_state: StoredAdapterConfigValueState::Present,
        },
        AdapterConfigValueRecord {
            field_key: BOOLEAN_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::Boolean,
            secret: false,
            string_value: None,
            path_value: None,
            bool_value: Some(true),
            int_value: None,
            value_source: AdapterValueSource::KnownProfileDefault,
            resolution_state: StoredAdapterConfigValueState::Present,
        },
        AdapterConfigValueRecord {
            field_key: INTEGER_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::Integer,
            secret: false,
            string_value: None,
            path_value: None,
            bool_value: None,
            int_value: Some(7),
            value_source: AdapterValueSource::MigratedConfig,
            resolution_state: StoredAdapterConfigValueState::Present,
        },
        AdapterConfigValueRecord {
            field_key: ENUM_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::Enum,
            secret: false,
            string_value: Some("safe".to_string()),
            path_value: None,
            bool_value: None,
            int_value: None,
            value_source: AdapterValueSource::CliFlag,
            resolution_state: StoredAdapterConfigValueState::Present,
        },
    ];
    selection.value_count = selection.values.len();
    save_local_adapter(binding_workspace.as_path(), selection.clone())?;

    let binding = configured_framework_adapter_binding(binding_workspace.as_path())?
        .ok_or("missing configured binding")?;
    assert_eq!(binding.selection.selection.command, adapter_script);
    assert_eq!(binding.host.describe()?.adapter_id, ADAPTER_ID);

    let direct_host =
        framework_adapter_host_from_selection(binding_workspace.as_path(), &selection)?;
    assert_eq!(direct_host.describe()?.adapter_id, ADAPTER_ID);

    let workspace_marker = binding_workspace.as_path().join("workspace-marker.txt");
    fs::write(&workspace_marker, "marker")?;
    let no_directory_host = framework_adapter_host_from_selection(&workspace_marker, &selection)?;
    assert_eq!(no_directory_host.describe()?.adapter_id, ADAPTER_ID);

    let config_values = runtime_framework_adapter_config_values(&selection);
    assert_eq!(config_values.len(), selection.values.len());
    assert_eq!(config_values[0].field_key, STRING_FIELD_KEY);
    assert_eq!(config_values[0].string_value.as_deref(), Some("boundline-runtime"));
    assert_eq!(config_values[1].field_key, PATH_FIELD_KEY);
    assert_eq!(config_values[1].path_value.as_deref(), Some("../boundline-framework-template"));
    assert_eq!(config_values[2].field_key, BOOLEAN_FIELD_KEY);
    assert_eq!(config_values[2].bool_value, Some(true));
    assert_eq!(config_values[3].field_key, INTEGER_FIELD_KEY);
    assert_eq!(config_values[3].int_value, Some(7));
    assert_eq!(config_values[4].field_key, ENUM_FIELD_KEY);
    assert_eq!(config_values[4].string_value.as_deref(), Some("safe"));

    let mut rationale_session = sample_goal_plan_session(binding_workspace.as_path())?;
    append_run_stage_adapter_fallback_reason(
        &mut rationale_session,
        FALLBACK_REASON_UNAVAILABLE_BINARY,
    );
    append_run_stage_adapter_fallback_reason(
        &mut rationale_session,
        FALLBACK_REASON_UNSUPPORTED_TRANSPORT,
    );
    append_run_stage_adapter_fallback_reason(
        &mut rationale_session,
        FALLBACK_REASON_UNAVAILABLE_BINARY,
    );
    let rationale = rationale_session
        .goal_plan
        .as_ref()
        .and_then(|goal_plan| goal_plan.planning_rationale.as_deref())
        .ok_or("missing fallback rationale")?;
    assert_eq!(
        rationale.matches(&goal_plan_fallback_note(FALLBACK_REASON_UNAVAILABLE_BINARY)).count(),
        1
    );
    assert!(rationale.contains(&goal_plan_fallback_note(FALLBACK_REASON_UNSUPPORTED_TRANSPORT)));

    let mut empty_rationale_session = sample_session(binding_workspace.as_path());
    append_run_stage_adapter_fallback_reason(
        &mut empty_rationale_session,
        FALLBACK_REASON_UNAVAILABLE_BINARY,
    );
    assert!(empty_rationale_session.goal_plan.is_none());

    assert_eq!(
        map_framework_adapter_failure_class(
            crate::adapters::FrameworkAdapterFailureClass::Preflight,
        ),
        AdapterFailureClass::Preflight
    );
    assert_eq!(
        map_framework_adapter_failure_class(
            crate::adapters::FrameworkAdapterFailureClass::Manifest,
        ),
        AdapterFailureClass::Manifest
    );
    assert_eq!(
        map_framework_adapter_failure_class(
            crate::adapters::FrameworkAdapterFailureClass::MissingConfig,
        ),
        AdapterFailureClass::MissingConfig
    );
    assert_eq!(
        map_framework_adapter_failure_class(
            crate::adapters::FrameworkAdapterFailureClass::AdapterRuntime,
        ),
        AdapterFailureClass::AdapterRuntime
    );
    assert_eq!(
        map_framework_adapter_failure_class(
            crate::adapters::FrameworkAdapterFailureClass::Compatibility,
        ),
        AdapterFailureClass::Compatibility
    );

    let protocol_error = FrameworkAdapterHostError::ProtocolError {
        command: adapter_script.clone(),
        request_kind: "execute-stage".to_string(),
        code: PROTOCOL_ERROR_CODE.to_string(),
        message: "invalid envelope".to_string(),
        details: None,
    };
    assert_eq!(
        protocol_error_code_from_host_error(&protocol_error),
        Some(PROTOCOL_ERROR_CODE.to_string())
    );
    assert_eq!(protocol_error_code_from_host_error(&FrameworkAdapterHostError::EmptyCommand), None);

    let git_workspace = temp_workspace("boundline-runtime-support-git-config")?;
    assert!(
        Command::new("git").current_dir(git_workspace.as_path()).args(["init"]).status()?.success()
    );
    assert!(
        Command::new("git")
            .current_dir(git_workspace.as_path())
            .args(["config", GIT_CONFIG_KEY_USER_NAME, GIT_CONFIG_USER_NAME_VALUE])
            .status()?
            .success()
    );
    assert_eq!(
        git_config_value(git_workspace.as_path(), GIT_CONFIG_KEY_USER_NAME),
        Some(GIT_CONFIG_USER_NAME_VALUE.to_string())
    );
    assert_eq!(git_config_value(git_workspace.as_path(), GIT_CONFIG_KEY_MISSING), None);

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_audit_outcome_and_message_projection() {
    let initialized = session_audit_outcome_for_status(SessionStatus::Initialized);
    assert_eq!(initialized.status, SessionAuditOutcomeStatus::Recorded);
    assert_eq!(initialized.summary, "session initialized");
    assert!(!initialized.blocking);

    let blocked = session_audit_outcome_for_status(SessionStatus::Blocked);
    assert_eq!(blocked.status, SessionAuditOutcomeStatus::Blocked);
    assert_eq!(blocked.summary, "session blocked");
    assert!(blocked.blocking);

    let exhausted = session_audit_outcome_for_status(SessionStatus::Exhausted);
    assert_eq!(exhausted.status, SessionAuditOutcomeStatus::Failed);
    assert_eq!(exhausted.summary, "session exhausted its execution budget");

    let governance_event = TraceEvent {
        event_id: "governance-event".to_string(),
        event_type: TraceEventType::GovernanceBlocked,
        step_id: None,
        plan_revision: 1,
        payload: json!({"summary": "operator approval required"}),
        recorded_at: UPDATED_AT,
    };
    let governance_algorithm = trace_event_audit_algorithm(governance_event.event_type);
    assert_eq!(governance_algorithm.phase, SessionAuditPhase::Governance);
    assert_eq!(governance_algorithm.family, "governance");
    assert_eq!(governance_algorithm.name, "execute_governance_for_step");
    let governance_outcome = trace_event_audit_outcome(&governance_event);
    assert_eq!(governance_outcome.status, SessionAuditOutcomeStatus::Blocked);
    assert_eq!(governance_outcome.summary, "operator approval required");
    assert!(governance_outcome.blocking);
    assert_eq!(
        trace_event_audit_message(&governance_event),
        "governance blocked: operator approval required"
    );

    let retry_event = TraceEvent {
        event_id: "retry-event".to_string(),
        event_type: TraceEventType::RetryScheduled,
        step_id: None,
        plan_revision: 1,
        payload: json!({"reason": "planner requested a retry"}),
        recorded_at: UPDATED_AT,
    };
    let retry_algorithm = trace_event_audit_algorithm(retry_event.event_type);
    assert_eq!(retry_algorithm.phase, SessionAuditPhase::Recovery);
    assert_eq!(retry_algorithm.family, "recovery");
    assert_eq!(retry_algorithm.name, "decide_recovery");
    assert_eq!(trace_event_audit_outcome(&retry_event).status, SessionAuditOutcomeStatus::Retried);
    assert_eq!(
        trace_event_audit_message(&retry_event),
        "retry scheduled: planner requested a retry"
    );

    let review_event = TraceEvent {
        event_id: "review-stop".to_string(),
        event_type: TraceEventType::ReviewStopSemanticsRecorded,
        step_id: None,
        plan_revision: 1,
        payload: json!({"stop_semantics": "fail_closed"}),
        recorded_at: UPDATED_AT,
    };
    let review_algorithm = trace_event_audit_algorithm(review_event.event_type);
    assert_eq!(review_algorithm.phase, SessionAuditPhase::Review);
    assert_eq!(review_algorithm.family, "review_governance");
    assert_eq!(review_algorithm.name, "active_review_stop_semantics");
    assert_eq!(
        trace_event_audit_message(&review_event),
        "review stop semantics recorded: stop semantics fail_closed"
    );

    let target_event = TraceEvent {
        event_id: "target-event".to_string(),
        event_type: TraceEventType::StageRouted,
        step_id: None,
        plan_revision: 1,
        payload: json!({"target": "src/lib.rs"}),
        recorded_at: UPDATED_AT,
    };
    assert_eq!(trace_event_audit_message(&target_event), "stage routed: target src/lib.rs");

    let unlabeled_event = TraceEvent {
        event_id: "unlabeled-event".to_string(),
        event_type: TraceEventType::TerminalRecorded,
        step_id: None,
        plan_revision: 1,
        payload: json!({}),
        recorded_at: UPDATED_AT,
    };
    assert_eq!(trace_event_audit_message(&unlabeled_event), "terminal recorded");
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_all_status_variants() {
    let session_cases = [
        (
            SessionStatus::Initialized,
            SessionAuditOutcomeStatus::Recorded,
            "session initialized",
            false,
        ),
        (
            SessionStatus::GoalCaptured,
            SessionAuditOutcomeStatus::Recorded,
            "goal captured for active session",
            false,
        ),
        (SessionStatus::Planned, SessionAuditOutcomeStatus::Completed, "session planned", false),
        (SessionStatus::Blocked, SessionAuditOutcomeStatus::Blocked, "session blocked", true),
        (SessionStatus::Running, SessionAuditOutcomeStatus::Started, "session running", false),
        (
            SessionStatus::Succeeded,
            SessionAuditOutcomeStatus::Succeeded,
            "session succeeded",
            false,
        ),
        (SessionStatus::Failed, SessionAuditOutcomeStatus::Failed, "session failed", false),
        (
            SessionStatus::Exhausted,
            SessionAuditOutcomeStatus::Failed,
            "session exhausted its execution budget",
            false,
        ),
        (SessionStatus::Aborted, SessionAuditOutcomeStatus::Failed, "session aborted", false),
        (SessionStatus::Invalid, SessionAuditOutcomeStatus::Failed, "session invalid", false),
    ];

    for (status, expected_status, expected_summary, expected_blocking) in session_cases {
        let outcome = session_audit_outcome_for_status(status);
        assert_eq!(outcome.status, expected_status);
        assert_eq!(outcome.summary, expected_summary);
        assert_eq!(outcome.blocking, expected_blocking);
    }

    let task_cases = [
        (TaskStatus::Planned, SessionStatus::Planned, "planned"),
        (TaskStatus::Running, SessionStatus::Running, "running"),
        (TaskStatus::Succeeded, SessionStatus::Succeeded, "succeeded"),
        (TaskStatus::Failed, SessionStatus::Failed, "failed"),
        (TaskStatus::Exhausted, SessionStatus::Exhausted, "exhausted"),
        (TaskStatus::Aborted, SessionStatus::Aborted, "aborted"),
    ];

    for (task_status, expected_session_status, expected_cluster_text) in task_cases {
        assert_eq!(session_status_for_task_status(task_status), expected_session_status);
        assert_eq!(cluster_task_status_text(task_status), expected_cluster_text);
    }
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_trace_event_algorithm_and_outcome_variants() {
    let cases = [
        (
            TraceEventType::GoalPlanCreated,
            SessionAuditPhase::Plan,
            "goal_planner",
            "build_goal_plan_with_sources",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::FlowInferred,
            SessionAuditPhase::Plan,
            "session_runtime",
            "plan_goal_plan",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ProjectScalePathProposed,
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ProjectScaleStageTransitioned,
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::DecisionCreated,
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::DecisionDispatched,
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
            SessionAuditOutcomeStatus::Recorded,
            false,
        ),
        (
            TraceEventType::DecisionVerified,
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
            SessionAuditOutcomeStatus::Succeeded,
            false,
        ),
        (
            TraceEventType::DecisionFailed,
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
            SessionAuditOutcomeStatus::Failed,
            false,
        ),
        (
            TraceEventType::DecisionRecovered,
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReviewCouncilAssembled,
            SessionAuditPhase::Review,
            "review_council",
            "resolve_council_assembly",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReviewStopSemanticsRecorded,
            SessionAuditPhase::Review,
            "review_governance",
            "active_review_stop_semantics",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ReviewVoteResolved,
            SessionAuditPhase::Review,
            "review_vote",
            "VoteRuleDefinition::resolve",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::VotingDecisionRecorded,
            SessionAuditPhase::Review,
            "review_vote",
            "VoteRuleDefinition::resolve",
            SessionAuditOutcomeStatus::Recorded,
            false,
        ),
        (
            TraceEventType::ReviewStarted,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ReviewerStarted,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ReviewerCompleted,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReviewTriggerIgnored,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ReviewAdjudicated,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReviewTerminalRecorded,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningProfileActivated,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ReasoningParticipantStarted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::ReasoningParticipantCompleted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningDisagreementRecorded,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningDebateRoundCompleted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningReflexionRevisionCompleted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningAdjudicationRecorded,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningConfidenceRecorded,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::ReasoningProfileBlocked,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Blocked,
            true,
        ),
        (
            TraceEventType::ReasoningProfileInterrupted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Awaiting,
            false,
        ),
        (
            TraceEventType::ReasoningProfileEscalated,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
            SessionAuditOutcomeStatus::Failed,
            false,
        ),
        (
            TraceEventType::GovernanceDecisionRecorded,
            SessionAuditPhase::Governance,
            "governance",
            "build_autopilot_decision",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::GovernanceSelected,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::GovernanceStarted,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::GovernanceAwaitingApproval,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
            SessionAuditOutcomeStatus::Awaiting,
            false,
        ),
        (
            TraceEventType::GovernanceCompleted,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
            SessionAuditOutcomeStatus::Succeeded,
            false,
        ),
        (
            TraceEventType::GovernanceBlocked,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
            SessionAuditOutcomeStatus::Blocked,
            true,
        ),
        (
            TraceEventType::GovernancePacketRejected,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
            SessionAuditOutcomeStatus::Blocked,
            true,
        ),
        (
            TraceEventType::RetryScheduled,
            SessionAuditPhase::Recovery,
            "recovery",
            "decide_recovery",
            SessionAuditOutcomeStatus::Retried,
            false,
        ),
        (
            TraceEventType::StageRetryScheduled,
            SessionAuditPhase::Recovery,
            "recovery",
            "decide_recovery",
            SessionAuditOutcomeStatus::Retried,
            false,
        ),
        (
            TraceEventType::Replanned,
            SessionAuditPhase::Recovery,
            "recovery",
            "decide_recovery",
            SessionAuditOutcomeStatus::Replanned,
            false,
        ),
        (
            TraceEventType::StageReplanned,
            SessionAuditPhase::Recovery,
            "recovery",
            "decide_recovery",
            SessionAuditOutcomeStatus::Replanned,
            false,
        ),
        (
            TraceEventType::StageFailed,
            SessionAuditPhase::Recovery,
            "recovery",
            "decide_recovery",
            SessionAuditOutcomeStatus::Failed,
            false,
        ),
        (
            TraceEventType::StageRouted,
            SessionAuditPhase::Run,
            "framework_adapter",
            "record_framework_adapter_run_stage_routing",
            SessionAuditOutcomeStatus::Recorded,
            false,
        ),
        (
            TraceEventType::CheckpointCreated,
            SessionAuditPhase::Run,
            "checkpoint",
            "prepare_checkpoint_for_mutation",
            SessionAuditOutcomeStatus::Recorded,
            false,
        ),
        (
            TraceEventType::TerminalRecorded,
            SessionAuditPhase::Run,
            "session_runtime",
            "finalize_task",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
        (
            TraceEventType::TaskStarted,
            SessionAuditPhase::Run,
            "session_runtime",
            "advance_task",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::FlowSelected,
            SessionAuditPhase::Run,
            "session_runtime",
            "advance_task",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::StageTransitioned,
            SessionAuditPhase::Run,
            "session_runtime",
            "advance_task",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::StepStarted,
            SessionAuditPhase::Run,
            "session_runtime",
            "advance_task",
            SessionAuditOutcomeStatus::Started,
            false,
        ),
        (
            TraceEventType::StepCompleted,
            SessionAuditPhase::Run,
            "session_runtime",
            "advance_task",
            SessionAuditOutcomeStatus::Completed,
            false,
        ),
    ];

    for (index, case) in cases.into_iter().enumerate() {
        let (
            event_type,
            expected_phase,
            expected_family,
            expected_name,
            expected_status,
            expected_blocking,
        ) = case;
        let algorithm = trace_event_audit_algorithm(event_type);
        assert_eq!(algorithm.phase, expected_phase);
        assert_eq!(algorithm.family, expected_family);
        assert_eq!(algorithm.name, expected_name);

        let event = sample_trace_event(event_type, json!({"summary": format!("summary-{index}")}));
        let outcome = trace_event_audit_outcome(&event);
        assert_eq!(outcome.status, expected_status);
        assert_eq!(outcome.blocking, expected_blocking);
    }
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_audit_actor_routes()
-> Result<(), Box<dyn Error>> {
    let reviewer_actor = trace_event_audit_actor(&TraceEvent {
        event_id: "reviewer-event".to_string(),
        event_type: TraceEventType::ReviewerCompleted,
        step_id: None,
        plan_revision: 1,
        payload: json!({
            "reviewer_id": "reviewer-1",
            "reviewer_role": "security",
            "reviewer_source": "review:copilot:gpt-5.4"
        }),
        recorded_at: UPDATED_AT,
    });
    assert_eq!(reviewer_actor.kind, SessionAuditActorKind::Reviewer);
    assert_eq!(reviewer_actor.id, "reviewer-1");
    assert_eq!(reviewer_actor.role.as_deref(), Some("security"));
    assert_eq!(reviewer_actor.route_slot.as_deref(), Some("review"));
    assert_eq!(reviewer_actor.runtime_kind.as_deref(), Some("copilot"));
    assert_eq!(reviewer_actor.provider.as_deref(), Some("copilot"));
    assert_eq!(reviewer_actor.model_name.as_deref(), Some("gpt-5.4"));

    let review_council_actor = trace_event_audit_actor(&TraceEvent {
        event_id: "council-event".to_string(),
        event_type: TraceEventType::ReviewVoteResolved,
        step_id: None,
        plan_revision: 1,
        payload: json!({
            "vote_resolution": {
                "participants": serde_json::to_value(vec![
                    ReviewerParticipation {
                        reviewer_id: "reviewer-1".to_string(),
                        status: ReviewerParticipationStatus::Completed,
                        reason: None,
                        effective_route: Some("review:copilot:gpt-5.4".to_string()),
                    },
                    ReviewerParticipation {
                        reviewer_id: "reviewer-2".to_string(),
                        status: ReviewerParticipationStatus::Completed,
                        reason: None,
                        effective_route: Some("review:openai:o3".to_string()),
                    },
                    ReviewerParticipation {
                        reviewer_id: "reviewer-3".to_string(),
                        status: ReviewerParticipationStatus::Omitted,
                        reason: None,
                        effective_route: Some("review:copilot:gpt-5.4".to_string()),
                    },
                ])?
            }
        }),
        recorded_at: UPDATED_AT,
    });
    assert_eq!(review_council_actor.kind, SessionAuditActorKind::Reviewer);
    assert_eq!(review_council_actor.id, "review-council");
    assert_eq!(review_council_actor.role.as_deref(), Some("multi-reviewer"));
    assert_eq!(review_council_actor.route_slot.as_deref(), Some("review"));
    assert_eq!(review_council_actor.participant_routes.len(), 2);
    assert!(review_council_actor.mixed_routes);

    let reasoning_actor = trace_event_audit_actor(&TraceEvent {
        event_id: "reasoning-event".to_string(),
        event_type: TraceEventType::ReasoningParticipantCompleted,
        step_id: None,
        plan_revision: 1,
        payload: json!({
            "participant_id": "critic-1",
            "role": "critic",
            "reasoning_profile_record": serde_json::to_value(ProfileActivationRecord {
                activation_id: "activation-1".to_string(),
                stage_key: "plan:requirements".to_string(),
                profile_id: ReasoningProfileId::BoundedSelfConsistency,
                trigger: ReasoningActivationTrigger::LocalFixture,
                activation_reason: "exercise audit projection".to_string(),
                status: ReasoningActivationStatus::Completed,
                participants: vec![ParticipantAssignment {
                    role_id: "critic".to_string(),
                    participant_id: "critic-1".to_string(),
                    effective_route: "review:github_models:gpt-5.4-mini".to_string(),
                    provider_family: Some("github_models".to_string()),
                    context_basis: "workspace".to_string(),
                    prompting_pattern: "critique".to_string(),
                    status: ReasoningParticipantStatus::Completed,
                    result_summary: None,
                }],
                budget: ReasoningBudget {
                    max_participants: 1,
                    max_branches: 1,
                    max_debate_rounds: 1,
                    max_reflexion_revisions: 1,
                    max_calls: 1,
                    max_tokens: 128,
                    max_adjudication_steps: 1,
                },
                posture: None,
                independence: None,
                outcome: None,
                confidence: None,
            })?
        }),
        recorded_at: UPDATED_AT,
    });
    assert_eq!(reasoning_actor.kind, SessionAuditActorKind::ReasoningParticipant);
    assert_eq!(reasoning_actor.id, "critic-1");
    assert_eq!(reasoning_actor.provider.as_deref(), Some("github_models"));
    assert_eq!(reasoning_actor.route_slot.as_deref(), Some("review"));
    assert_eq!(reasoning_actor.model_name.as_deref(), Some("gpt-5.4-mini"));

    let governance_actor = governance_audit_actor(&json!({
        "selected_runtime": "canon",
        "stage_key": "plan:requirements"
    }));
    assert_eq!(governance_actor.kind, SessionAuditActorKind::GovernanceRuntime);
    assert_eq!(governance_actor.route_slot.as_deref(), Some("planning"));
    assert_eq!(governance_actor.runtime_kind.as_deref(), Some("canon"));
    assert_eq!(governance_actor.provider.as_deref(), Some("canon"));
    assert_eq!(governance_route_slot_for_stage_key(" run:implementation "), Some("implementation"));
    assert_eq!(governance_route_slot_for_stage_key("   "), None);

    let mut actor = SessionAuditActor::system("boundline");
    apply_route_text_to_actor(&mut actor, "copilot/gpt-5.4");
    assert_eq!(actor.route_slot, None);
    assert_eq!(actor.runtime_kind.as_deref(), Some("copilot"));
    assert_eq!(actor.provider.as_deref(), Some("copilot"));
    assert_eq!(actor.model_name.as_deref(), Some("gpt-5.4"));

    assert_eq!(
        parse_three_segment_route("review:copilot:gpt-5.4"),
        Some(("review".to_string(), "copilot".to_string(), "gpt-5.4".to_string(),))
    );
    assert_eq!(parse_three_segment_route("review::gpt-5.4"), None);
    assert_eq!(payload_string(Some(&Value::Bool(true))), Some("true".to_string()));
    assert_eq!(
        payload_string(Some(&json!({"key": "value"}))),
        Some("{\"key\":\"value\"}".to_string())
    );
    assert_eq!(payload_string(Some(&Value::Null)), None);
    assert_eq!(trace_event_type_text(TraceEventType::GovernanceBlocked), "governance_blocked");

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_trace_event_message_and_actor_edges()
-> Result<(), Box<dyn Error>> {
    let message_cases = [
        (json!({"message": "operator note"}), "stage failed: operator note"),
        (json!({"headline": "selected a safer path"}), "stage failed: selected a safer path"),
        (
            json!({"selection_summary": "review council selected change"}),
            "stage failed: review council selected change",
        ),
        (json!({"stage_key": IMPLEMENTATION_STAGE_KEY}), "stage failed: stage run:implementation"),
        (json!({"reviewer_id": "reviewer-42"}), "stage failed: reviewer reviewer-42"),
        (json!({"participant_id": "participant-42"}), "stage failed: participant participant-42"),
    ];
    for (payload, expected_message) in message_cases {
        let event = sample_trace_event(TraceEventType::StageFailed, payload);
        assert_eq!(trace_event_audit_message(&event), expected_message);
    }

    for event_type in [
        TraceEventType::ReviewerStarted,
        TraceEventType::ReviewerCompleted,
        TraceEventType::ReviewAdjudicated,
    ] {
        let actor = trace_event_audit_actor(&sample_trace_event(event_type, json!({})));
        assert_eq!(actor.kind, SessionAuditActorKind::Reviewer);
        assert_eq!(actor.id, UNKNOWN_REVIEWER_ID);
        assert_eq!(actor.display_name.as_deref(), Some(UNKNOWN_REVIEWER_ID));
        assert_eq!(actor.role, None);
        assert_eq!(actor.provider, None);
        assert_eq!(actor.route_slot, None);
    }

    for event_type in [
        TraceEventType::ReviewCouncilAssembled,
        TraceEventType::ReviewStopSemanticsRecorded,
        TraceEventType::ReviewVoteResolved,
    ] {
        let actor = trace_event_audit_actor(&sample_trace_event(event_type, json!({})));
        assert_eq!(actor.kind, SessionAuditActorKind::Reviewer);
        assert_eq!(actor.id, REVIEW_COUNCIL_ID);
        assert_eq!(actor.display_name.as_deref(), Some(REVIEW_COUNCIL_DISPLAY_NAME));
        assert_eq!(actor.route_slot.as_deref(), Some(ROUTE_SLOT_REVIEW));
        assert!(actor.participant_routes.is_empty());
        assert!(!actor.mixed_routes);
        assert_eq!(actor.role, None);
    }

    for event_type in
        [TraceEventType::ReasoningParticipantStarted, TraceEventType::ReasoningParticipantCompleted]
    {
        let actor = trace_event_audit_actor(&sample_trace_event(
            event_type,
            json!({"participant_id": UNKNOWN_PARTICIPANT_ID}),
        ));
        assert_eq!(actor.kind, SessionAuditActorKind::ReasoningParticipant);
        assert_eq!(actor.id, UNKNOWN_PARTICIPANT_ID);
        assert_eq!(actor.display_name.as_deref(), Some(UNKNOWN_PARTICIPANT_ID));
        assert_eq!(actor.provider, None);
        assert_eq!(actor.route_slot, None);
    }

    for event_type in [
        TraceEventType::GovernanceSelected,
        TraceEventType::GovernanceStarted,
        TraceEventType::GovernanceDecisionRecorded,
        TraceEventType::GovernanceAwaitingApproval,
        TraceEventType::GovernanceCompleted,
        TraceEventType::GovernanceBlocked,
        TraceEventType::GovernancePacketRejected,
    ] {
        let actor = trace_event_audit_actor(&sample_trace_event(
            event_type,
            json!({"runtime": "canon", "stage_key": IMPLEMENTATION_STAGE_KEY}),
        ));
        assert_eq!(actor.kind, SessionAuditActorKind::GovernanceRuntime);
        assert_eq!(actor.id, "canon");
        assert_eq!(actor.display_name.as_deref(), Some("canon"));
        assert_eq!(actor.route_slot.as_deref(), Some(ROUTE_SLOT_IMPLEMENTATION));
        assert_eq!(actor.role.as_deref(), Some(IMPLEMENTATION_STAGE_KEY));
        assert_eq!(actor.provider.as_deref(), Some("canon"));
    }

    for event_type in [
        TraceEventType::DecisionCreated,
        TraceEventType::DecisionDispatched,
        TraceEventType::DecisionVerified,
        TraceEventType::DecisionFailed,
        TraceEventType::DecisionRecovered,
    ] {
        let actor = trace_event_audit_actor(&sample_trace_event(event_type, json!({})));
        assert_eq!(actor.kind, SessionAuditActorKind::Agent);
        assert_eq!(actor.id, DECISION_LOOP_ID);
        assert_eq!(actor.display_name.as_deref(), Some(DECISION_LOOP_DISPLAY_NAME));
    }

    for event_type in [
        TraceEventType::ReviewStarted,
        TraceEventType::ReviewTriggerIgnored,
        TraceEventType::ReviewTerminalRecorded,
        TraceEventType::VotingDecisionRecorded,
    ] {
        let actor = trace_event_audit_actor(&sample_trace_event(event_type, json!({})));
        assert_eq!(actor.kind, SessionAuditActorKind::Reviewer);
        assert_eq!(actor.id, REVIEW_COUNCIL_ID);
        assert_eq!(actor.display_name.as_deref(), Some(REVIEW_COUNCIL_DISPLAY_NAME));
        assert_eq!(actor.route_slot, None);
    }

    let default_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::FlowSelected, json!({})));
    assert_eq!(default_actor.id, BOUNDLINE_SYSTEM_ID);

    let default_governance_actor = governance_audit_actor(&json!({}));
    assert_eq!(default_governance_actor.id, DEFAULT_GOVERNANCE_RUNTIME);
    assert_eq!(default_governance_actor.display_name.as_deref(), Some(DEFAULT_GOVERNANCE_RUNTIME));
    assert_eq!(default_governance_actor.provider, None);
    assert_eq!(default_governance_actor.route_slot, None);

    let mut three_segment_actor = SessionAuditActor::system(BOUNDLINE_SYSTEM_ID);
    three_segment_actor.provider = Some(DEFAULT_GOVERNANCE_RUNTIME.to_string());
    apply_route_text_to_actor(&mut three_segment_actor, REVIEW_ROUTE_COPILOT);
    assert_eq!(three_segment_actor.route_slot.as_deref(), Some(ROUTE_SLOT_REVIEW));
    assert_eq!(three_segment_actor.runtime_kind.as_deref(), Some("copilot"));
    assert_eq!(three_segment_actor.provider.as_deref(), Some(DEFAULT_GOVERNANCE_RUNTIME));
    assert_eq!(three_segment_actor.model_name.as_deref(), Some("gpt-5.4"));

    let mut empty_runtime_actor = SessionAuditActor::system(BOUNDLINE_SYSTEM_ID);
    apply_route_text_to_actor(&mut empty_runtime_actor, SIMPLE_ROUTE_EMPTY_RUNTIME);
    assert_eq!(empty_runtime_actor.runtime_kind, None);
    assert_eq!(empty_runtime_actor.provider, None);
    assert_eq!(empty_runtime_actor.model_name.as_deref(), Some("gpt-5.4"));

    let mut empty_model_actor = SessionAuditActor::system(BOUNDLINE_SYSTEM_ID);
    apply_route_text_to_actor(&mut empty_model_actor, SIMPLE_ROUTE_EMPTY_MODEL);
    assert_eq!(empty_model_actor.runtime_kind.as_deref(), Some("copilot"));
    assert_eq!(empty_model_actor.provider.as_deref(), Some("copilot"));
    assert_eq!(empty_model_actor.model_name, None);

    let mut two_segment_actor = SessionAuditActor::system(BOUNDLINE_SYSTEM_ID);
    apply_route_text_to_actor(&mut two_segment_actor, SIMPLE_ROUTE_COPILOT);
    assert_eq!(two_segment_actor.runtime_kind.as_deref(), Some("copilot"));
    assert_eq!(two_segment_actor.provider.as_deref(), Some("copilot"));
    assert_eq!(two_segment_actor.model_name.as_deref(), Some("gpt-5.4"));

    assert_eq!(
        parse_three_segment_route(" review : openai : o3 "),
        Some((ROUTE_SLOT_REVIEW.to_string(), "openai".to_string(), REVIEW_MODEL_O3.to_string(),))
    );
    assert_eq!(parse_three_segment_route("review:copilot:"), None);

    assert_eq!(
        payload_string(Some(&Value::String("gpt-5.4".to_string()))),
        Some("gpt-5.4".to_string())
    );
    assert_eq!(payload_string(Some(&json!(7))), Some("7".to_string()));
    assert_eq!(payload_string(None), None);

    let deserialize_source =
        serde_json::from_str::<Value>("{").err().ok_or("expected JSON parse error")?;
    let workspace = temp_workspace("boundline-runtime-support-host-error")?;
    let runtime = SessionRuntime::for_workspace(workspace.as_path());
    let protocol_failure = runtime.framework_adapter_stage_failure_from_host_error(
        &sample_stage_runtime(Vec::new()),
        FrameworkAdapterHostError::DeserializeResponse {
            command: ADAPTER_COMMAND.to_string(),
            request_kind: "execute-stage".to_string(),
            source: deserialize_source,
        },
    );
    assert_eq!(protocol_failure.execution.failure_class, Some(AdapterFailureClass::ProtocolError));
    assert!(protocol_failure.summary.contains("protocol error"));

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_context_and_workspace_helpers()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-runtime-support-utility-helpers")?;
    let artifact_dir = workspace.as_path().join("packet");
    fs::create_dir_all(&artifact_dir)?;
    fs::write(artifact_dir.join("empty.md"), "   ")?;
    fs::write(artifact_dir.join("short.md"), "  short artifact  ")?;
    fs::write(artifact_dir.join("long.md"), "x".repeat(UPSTREAM_EVIDENCE_MAX_CHARS + 10))?;

    assert_eq!(read_upstream_artifact_capped(&artifact_dir, "missing.md"), None);
    assert_eq!(read_upstream_artifact_capped(&artifact_dir, "empty.md"), None);
    assert_eq!(
        read_upstream_artifact_capped(&artifact_dir, "short.md"),
        Some("short artifact".to_string())
    );
    let truncated = read_upstream_artifact_capped(&artifact_dir, "long.md")
        .ok_or("missing truncated artifact")?;
    assert!(truncated.ends_with("[truncated]"));
    assert!(truncated.chars().count() > UPSTREAM_EVIDENCE_MAX_CHARS);

    let context = TaskContext::new(
        SESSION_ID,
        workspace.as_path().to_string_lossy().into_owned(),
        crate::domain::limits::RunLimits::default(),
        serde_json::Map::from_iter([
            (
                super::LATEST_CHANGED_FILES_KEY.to_string(),
                json!(["src/lib.rs", "src/main.rs", "src/lib.rs", " "]),
            ),
            ("changed_files".to_string(), json!(["tests/runtime.rs", "src/main.rs"])),
        ]),
    );
    assert_eq!(
        execution_governance_read_targets(&context, &["fallback.rs".to_string()]),
        vec!["src/lib.rs".to_string(), "src/main.rs".to_string(), "tests/runtime.rs".to_string(),]
    );

    let empty_context = TaskContext::new(
        SESSION_ID,
        workspace.as_path().to_string_lossy().into_owned(),
        crate::domain::limits::RunLimits::default(),
        serde_json::Map::new(),
    );
    assert_eq!(
        execution_governance_read_targets(
            &empty_context,
            &["fallback.rs".to_string(), " ".to_string(), "fallback.rs".to_string()],
        ),
        vec!["fallback.rs".to_string()]
    );

    let malformed_context = TaskContext::new(
        SESSION_ID,
        workspace.as_path().to_string_lossy().into_owned(),
        crate::domain::limits::RunLimits::default(),
        serde_json::Map::from_iter([(
            super::LATEST_CHANGED_FILES_KEY.to_string(),
            json!("not-an-array"),
        )]),
    );
    assert_eq!(
        execution_governance_read_targets(&malformed_context, &["fallback.rs".to_string()]),
        vec!["fallback.rs".to_string()]
    );

    assert_eq!(default_planning_system_context(CanonMode::Discovery), SystemContextBinding::New);
    assert_eq!(
        default_planning_system_context(CanonMode::Implementation),
        SystemContextBinding::Existing
    );
    assert_eq!(parse_planning_system_context("new"), Some(SystemContextBinding::New));
    assert_eq!(parse_planning_system_context("existing"), Some(SystemContextBinding::Existing));
    assert_eq!(parse_planning_system_context("unknown"), None);

    let field_error = missing_planning_governance_field(CanonMode::Architecture, "packet_ref");
    assert!(
        field_error.to_string().contains(
            "planning governance for Canon mode architecture requires field 'packet_ref'"
        )
    );

    assert_eq!(session_status_for_task_status(TaskStatus::Planned), SessionStatus::Planned);
    assert_eq!(session_status_for_task_status(TaskStatus::Running), SessionStatus::Running);
    assert_eq!(session_status_for_task_status(TaskStatus::Succeeded), SessionStatus::Succeeded);
    assert_eq!(session_status_for_task_status(TaskStatus::Failed), SessionStatus::Failed);
    assert_eq!(session_status_for_task_status(TaskStatus::Exhausted), SessionStatus::Exhausted);
    assert_eq!(session_status_for_task_status(TaskStatus::Aborted), SessionStatus::Aborted);
    assert_eq!(cluster_task_status_text(TaskStatus::Running), "running");
    assert_eq!(cluster_task_status_text(TaskStatus::Succeeded), "succeeded");
    assert_eq!(session_status_text(SessionStatus::Initialized), "initialized");
    assert_eq!(session_status_text(SessionStatus::GoalCaptured), "goal_captured");
    assert_eq!(session_status_text(SessionStatus::Planned), "planned");
    assert_eq!(session_status_text(SessionStatus::Blocked), "blocked");
    assert_eq!(session_status_text(SessionStatus::Running), "running");
    assert_eq!(session_status_text(SessionStatus::Succeeded), "succeeded");
    assert_eq!(session_status_text(SessionStatus::Failed), "failed");
    assert_eq!(session_status_text(SessionStatus::Exhausted), "exhausted");
    assert_eq!(session_status_text(SessionStatus::Aborted), "aborted");
    assert_eq!(session_status_text(SessionStatus::Invalid), "invalid");

    let repo_root = workspace.as_path().join("repo-root");
    let nested_workspace = repo_root.join("nested-workspace");
    fs::create_dir_all(repo_root.join(".git"))?;
    fs::create_dir_all(&nested_workspace)?;
    let mismatch_reason = canon_workspace_scope_mismatch_reason(&nested_workspace)
        .ok_or("missing git-root mismatch reason")?;
    assert!(mismatch_reason.contains("nested-workspace"));
    assert!(mismatch_reason.contains("repo-root"));

    let standalone_workspace = workspace.as_path().join("standalone-workspace");
    fs::create_dir_all(standalone_workspace.join(".git"))?;
    assert_eq!(canon_workspace_scope_mismatch_reason(&standalone_workspace), None);

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_runtime_precedence_and_cluster_blocking()
-> Result<(), Box<dyn Error>> {
    let workspace_config =
        RoutingConfig { assistant_runtimes: vec![RuntimeKind::Codex], ..RoutingConfig::default() };
    let cluster_config = RoutingConfig {
        assistant_runtimes: vec![RuntimeKind::Copilot],
        ..RoutingConfig::default()
    };
    let global_config =
        RoutingConfig { assistant_runtimes: vec![RuntimeKind::Claude], ..RoutingConfig::default() };
    assert_eq!(
        effective_assistant_runtimes(
            Some(&workspace_config),
            Some(&cluster_config),
            Some(&global_config),
        ),
        vec![RuntimeKind::Codex]
    );
    assert_eq!(
        effective_assistant_runtimes(None, Some(&cluster_config), Some(&global_config)),
        vec![RuntimeKind::Copilot]
    );
    assert_eq!(
        effective_assistant_runtimes(None, None, Some(&global_config)),
        vec![RuntimeKind::Claude]
    );
    assert!(effective_assistant_runtimes(None, None, None).is_empty());

    assert_eq!(cluster_task_status_text(TaskStatus::Exhausted), "exhausted");
    assert_eq!(cluster_task_status_text(TaskStatus::Aborted), "aborted");
    assert_eq!(cluster_task_status_text(TaskStatus::Planned), "planned");
    assert_eq!(cluster_task_status_text(TaskStatus::Failed), "failed");

    let workspace = temp_workspace("boundline-runtime-support-cluster-blocking")?;
    let source_dir = workspace.as_path().join("src");
    fs::create_dir_all(&source_dir)?;
    write_execution_profile(workspace.as_path())?;

    fs::write(source_dir.join("lib.rs"), "fn sample() { /* before */ }")?;
    assert!(!cluster_workspace_is_blocked(&workspace.as_path().to_string_lossy()));

    fs::write(source_dir.join("lib.rs"), "fn sample() { /* after */ }")?;
    assert!(!cluster_workspace_is_blocked(&workspace.as_path().to_string_lossy()));

    fs::write(source_dir.join("lib.rs"), "fn sample() { /* unrelated */ }")?;
    assert!(cluster_workspace_is_blocked(&workspace.as_path().to_string_lossy()));

    fs::remove_file(source_dir.join("lib.rs"))?;
    assert!(cluster_workspace_is_blocked(&workspace.as_path().to_string_lossy()));

    let missing_manifest_workspace = temp_workspace("boundline-runtime-support-cluster-missing")?;
    assert!(cluster_workspace_is_blocked(&missing_manifest_workspace.as_path().to_string_lossy()));

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_trimmed_planning_context_and_type_text() {
    let governance_actor = governance_audit_actor(&json!({
        "selected_runtime": "canon",
        "stage_key": " plan:requirements "
    }));
    assert_eq!(governance_actor.route_slot.as_deref(), Some("planning"));
    assert_eq!(governance_actor.provider.as_deref(), Some("canon"));
    assert_eq!(trace_event_type_text(TraceEventType::StageFailed), "stage_failed");
    assert_eq!(parse_planning_system_context(" existing "), Some(SystemContextBinding::Existing));
    assert_eq!(parse_planning_system_context(" new "), Some(SystemContextBinding::New));
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_helper_fallback_edges()
-> Result<(), Box<dyn Error>> {
    let invalid_council_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::ReviewVoteResolved,
        json!({
            "vote_resolution": {
                "participants": "invalid-shape"
            }
        }),
    ));
    assert_eq!(invalid_council_actor.kind, SessionAuditActorKind::Reviewer);
    assert_eq!(invalid_council_actor.id, REVIEW_COUNCIL_ID);
    assert_eq!(invalid_council_actor.route_slot.as_deref(), Some(ROUTE_SLOT_REVIEW));
    assert!(invalid_council_actor.participant_routes.is_empty());
    assert!(!invalid_council_actor.mixed_routes);
    assert_eq!(invalid_council_actor.provider, None);

    let single_route_council_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::ReviewVoteResolved,
        json!({
            "vote_resolution": {
                "participants": serde_json::to_value(vec![ReviewerParticipation {
                    reviewer_id: "reviewer-1".to_string(),
                    status: ReviewerParticipationStatus::Completed,
                    reason: None,
                    effective_route: Some(REVIEW_ROUTE_COPILOT.to_string()),
                }])?
            }
        }),
    ));
    assert_eq!(single_route_council_actor.provider.as_deref(), Some("copilot"));
    assert_eq!(single_route_council_actor.runtime_kind.as_deref(), Some("copilot"));
    assert_eq!(single_route_council_actor.model_name.as_deref(), Some("gpt-5.4"));
    assert!(!single_route_council_actor.mixed_routes);
    assert_eq!(single_route_council_actor.role, None);

    let mismatched_reasoning_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::ReasoningParticipantStarted,
        json!({
            "participant_id": "critic-2",
            "role": "critic",
            "reasoning_profile_record": serde_json::to_value(ProfileActivationRecord {
                activation_id: "activation-2".to_string(),
                stage_key: "plan:requirements".to_string(),
                profile_id: ReasoningProfileId::BoundedSelfConsistency,
                trigger: ReasoningActivationTrigger::LocalFixture,
                activation_reason: "exercise participant mismatch".to_string(),
                status: ReasoningActivationStatus::Completed,
                participants: vec![ParticipantAssignment {
                    role_id: "critic".to_string(),
                    participant_id: "critic-1".to_string(),
                    effective_route: REVIEW_ROUTE_COPILOT.to_string(),
                    provider_family: Some("copilot".to_string()),
                    context_basis: "workspace".to_string(),
                    prompting_pattern: "critique".to_string(),
                    status: ReasoningParticipantStatus::Pending,
                    result_summary: None,
                }],
                budget: ReasoningBudget {
                    max_participants: 1,
                    max_branches: 1,
                    max_debate_rounds: 1,
                    max_reflexion_revisions: 1,
                    max_calls: 1,
                    max_tokens: 128,
                    max_adjudication_steps: 1,
                },
                posture: None,
                independence: None,
                outcome: None,
                confidence: None,
            })?
        }),
    ));
    assert_eq!(mismatched_reasoning_actor.id, "critic-2");
    assert_eq!(mismatched_reasoning_actor.provider, None);
    assert_eq!(mismatched_reasoning_actor.route_slot, None);
    assert_eq!(mismatched_reasoning_actor.model_name, None);

    let explicit_runtime_actor = governance_audit_actor(&json!({
        "runtime": "codex",
        "selected_runtime": "canon",
        "stage_key": " run:implementation "
    }));
    assert_eq!(explicit_runtime_actor.id, "codex");
    assert_eq!(explicit_runtime_actor.provider.as_deref(), Some("codex"));
    assert_eq!(explicit_runtime_actor.route_slot.as_deref(), Some(ROUTE_SLOT_IMPLEMENTATION));

    let mut unchanged_actor = SessionAuditActor::system(BOUNDLINE_SYSTEM_ID);
    apply_route_text_to_actor(&mut unchanged_actor, "standalone-label");
    assert_eq!(unchanged_actor.route_slot, None);
    assert_eq!(unchanged_actor.runtime_kind, None);
    assert_eq!(unchanged_actor.provider, None);
    assert_eq!(unchanged_actor.model_name, None);

    let default_message =
        trace_event_audit_message(&sample_trace_event(TraceEventType::FlowSelected, json!({})));
    assert_eq!(default_message, "flow selected");

    Ok(())
}

#[test]
fn run_stage_runtime_support_helper_utilities_pin_match_variant_lines() {
    let goal_captured = session_audit_outcome_for_status(SessionStatus::GoalCaptured);
    assert_eq!(goal_captured.status, SessionAuditOutcomeStatus::Recorded);
    let planned = session_audit_outcome_for_status(SessionStatus::Planned);
    assert_eq!(planned.status, SessionAuditOutcomeStatus::Completed);
    let running = session_audit_outcome_for_status(SessionStatus::Running);
    assert_eq!(running.status, SessionAuditOutcomeStatus::Started);
    let succeeded = session_audit_outcome_for_status(SessionStatus::Succeeded);
    assert_eq!(succeeded.status, SessionAuditOutcomeStatus::Succeeded);
    let failed = session_audit_outcome_for_status(SessionStatus::Failed);
    assert_eq!(failed.status, SessionAuditOutcomeStatus::Failed);
    let aborted = session_audit_outcome_for_status(SessionStatus::Aborted);
    assert_eq!(aborted.status, SessionAuditOutcomeStatus::Failed);
    let invalid = session_audit_outcome_for_status(SessionStatus::Invalid);
    assert_eq!(invalid.status, SessionAuditOutcomeStatus::Failed);

    let decision_dispatched_algorithm =
        trace_event_audit_algorithm(TraceEventType::DecisionDispatched);
    assert_eq!(decision_dispatched_algorithm.family, "decision_loop");
    let reviewer_started_algorithm = trace_event_audit_algorithm(TraceEventType::ReviewerStarted);
    assert_eq!(reviewer_started_algorithm.family, "review_trace");
    let reasoning_adjudication_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReasoningAdjudicationRecorded);
    assert_eq!(reasoning_adjudication_algorithm.family, "reasoning_profile");
    let governance_started_algorithm =
        trace_event_audit_algorithm(TraceEventType::GovernanceStarted);
    assert_eq!(governance_started_algorithm.family, "governance");
    let stage_retry_algorithm = trace_event_audit_algorithm(TraceEventType::StageRetryScheduled);
    assert_eq!(stage_retry_algorithm.family, "recovery");
    let stage_replanned_algorithm = trace_event_audit_algorithm(TraceEventType::StageReplanned);
    assert_eq!(stage_replanned_algorithm.family, "recovery");
    let flow_selected_algorithm = trace_event_audit_algorithm(TraceEventType::FlowSelected);
    assert_eq!(flow_selected_algorithm.family, "session_runtime");
    let step_completed_algorithm = trace_event_audit_algorithm(TraceEventType::StepCompleted);
    assert_eq!(step_completed_algorithm.name, "advance_task");

    let step_started_outcome =
        trace_event_audit_outcome(&sample_trace_event(TraceEventType::StepStarted, json!({})));
    assert_eq!(step_started_outcome.status, SessionAuditOutcomeStatus::Started);
    let voting_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::VotingDecisionRecorded,
        json!({}),
    ));
    assert_eq!(voting_outcome.status, SessionAuditOutcomeStatus::Recorded);
    let reasoning_adjudication_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::ReasoningAdjudicationRecorded,
        json!({"summary": "resolved"}),
    ));
    assert_eq!(reasoning_adjudication_outcome.status, SessionAuditOutcomeStatus::Completed);
    let governance_packet_rejected_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::GovernancePacketRejected,
        json!({"summary": "rejected"}),
    ));
    assert_eq!(governance_packet_rejected_outcome.status, SessionAuditOutcomeStatus::Blocked);
    assert!(governance_packet_rejected_outcome.blocking);
    let decision_failed_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::DecisionFailed,
        json!({"summary": "failed"}),
    ));
    assert_eq!(decision_failed_outcome.status, SessionAuditOutcomeStatus::Failed);
    let stage_retry_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::StageRetryScheduled,
        json!({"summary": "retry"}),
    ));
    assert_eq!(stage_retry_outcome.status, SessionAuditOutcomeStatus::Retried);
    let stage_replanned_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::StageReplanned,
        json!({"summary": "replanned"}),
    ));
    assert_eq!(stage_replanned_outcome.status, SessionAuditOutcomeStatus::Replanned);
    let stage_failed_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::StageFailed,
        json!({"summary": "stage failed"}),
    ));
    assert_eq!(stage_failed_outcome.status, SessionAuditOutcomeStatus::Failed);

    let governance_blocked_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::GovernanceBlocked,
        json!({"runtime": "canon", "stage_key": IMPLEMENTATION_STAGE_KEY}),
    ));
    assert_eq!(governance_blocked_actor.id, "canon");
    let governance_packet_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::GovernancePacketRejected,
        json!({"runtime": "canon", "stage_key": IMPLEMENTATION_STAGE_KEY}),
    ));
    assert_eq!(governance_packet_actor.id, "canon");
    let decision_dispatched_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::DecisionDispatched, json!({})));
    assert_eq!(decision_dispatched_actor.id, DECISION_LOOP_ID);
    let decision_failed_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::DecisionFailed, json!({})));
    assert_eq!(decision_failed_actor.id, DECISION_LOOP_ID);
    let review_trigger_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::ReviewTriggerIgnored,
        json!({}),
    ));
    assert_eq!(review_trigger_actor.id, REVIEW_COUNCIL_ID);
    let review_terminal_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::ReviewTerminalRecorded,
        json!({}),
    ));
    assert_eq!(review_terminal_actor.id, REVIEW_COUNCIL_ID);
    let voting_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::VotingDecisionRecorded,
        json!({}),
    ));
    assert_eq!(voting_actor.id, REVIEW_COUNCIL_ID);
}

#[test]
fn run_stage_runtime_support_helper_utilities_pin_remaining_match_lines() {
    let initialized = session_audit_outcome_for_status(SessionStatus::Initialized);
    assert_eq!(initialized.summary, "session initialized");
    let exhausted = session_audit_outcome_for_status(SessionStatus::Exhausted);
    assert_eq!(exhausted.summary, "session exhausted its execution budget");

    let goal_plan_algorithm = trace_event_audit_algorithm(TraceEventType::GoalPlanCreated);
    assert_eq!(goal_plan_algorithm.name, "build_goal_plan_with_sources");
    let flow_inferred_algorithm = trace_event_audit_algorithm(TraceEventType::FlowInferred);
    assert_eq!(flow_inferred_algorithm.name, "plan_goal_plan");
    let review_council_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReviewCouncilAssembled);
    assert_eq!(review_council_algorithm.family, "review_council");
    let review_stop_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReviewStopSemanticsRecorded);
    assert_eq!(review_stop_algorithm.family, "review_governance");
    let reviewer_completed_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReviewerCompleted);
    assert_eq!(reviewer_completed_algorithm.family, "review_trace");
    let review_trigger_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReviewTriggerIgnored);
    assert_eq!(review_trigger_algorithm.name, "record_review_step_completed");
    let reasoning_confidence_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReasoningConfidenceRecorded);
    assert_eq!(reasoning_confidence_algorithm.family, "reasoning_profile");
    let reasoning_blocked_algorithm =
        trace_event_audit_algorithm(TraceEventType::ReasoningProfileBlocked);
    assert_eq!(reasoning_blocked_algorithm.phase, SessionAuditPhase::Reasoning);
    let governance_awaiting_algorithm =
        trace_event_audit_algorithm(TraceEventType::GovernanceAwaitingApproval);
    assert_eq!(governance_awaiting_algorithm.phase, SessionAuditPhase::Governance);
    let governance_completed_algorithm =
        trace_event_audit_algorithm(TraceEventType::GovernanceCompleted);
    assert_eq!(governance_completed_algorithm.name, "execute_governance_for_step");
    let replanned_algorithm = trace_event_audit_algorithm(TraceEventType::Replanned);
    assert_eq!(replanned_algorithm.family, "recovery");
    let stage_failed_algorithm = trace_event_audit_algorithm(TraceEventType::StageFailed);
    assert_eq!(stage_failed_algorithm.name, "decide_recovery");
    let stage_routed_algorithm = trace_event_audit_algorithm(TraceEventType::StageRouted);
    assert_eq!(stage_routed_algorithm.family, "framework_adapter");
    let checkpoint_algorithm = trace_event_audit_algorithm(TraceEventType::CheckpointCreated);
    assert_eq!(checkpoint_algorithm.name, "prepare_checkpoint_for_mutation");
    let stage_transitioned_algorithm =
        trace_event_audit_algorithm(TraceEventType::StageTransitioned);
    assert_eq!(stage_transitioned_algorithm.family, "session_runtime");

    let decision_created_outcome =
        trace_event_audit_outcome(&sample_trace_event(TraceEventType::DecisionCreated, json!({})));
    assert_eq!(decision_created_outcome.status, SessionAuditOutcomeStatus::Started);
    let stage_routed_outcome =
        trace_event_audit_outcome(&sample_trace_event(TraceEventType::StageRouted, json!({})));
    assert_eq!(stage_routed_outcome.status, SessionAuditOutcomeStatus::Recorded);
    let reasoning_confidence_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::ReasoningConfidenceRecorded,
        json!({"summary": "confidence recorded"}),
    ));
    assert_eq!(reasoning_confidence_outcome.status, SessionAuditOutcomeStatus::Completed);
    let governance_blocked_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::GovernanceBlocked,
        json!({"summary": "blocked"}),
    ));
    assert_eq!(governance_blocked_outcome.status, SessionAuditOutcomeStatus::Blocked);
    assert!(governance_blocked_outcome.blocking);
    let reasoning_blocked_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::ReasoningProfileBlocked,
        json!({"summary": "paused"}),
    ));
    assert_eq!(reasoning_blocked_outcome.status, SessionAuditOutcomeStatus::Blocked);
    assert!(reasoning_blocked_outcome.blocking);
    let retry_scheduled_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::RetryScheduled,
        json!({"summary": "retry scheduled"}),
    ));
    assert_eq!(retry_scheduled_outcome.status, SessionAuditOutcomeStatus::Retried);
    let replanned_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::Replanned,
        json!({"summary": "replanned"}),
    ));
    assert_eq!(replanned_outcome.status, SessionAuditOutcomeStatus::Replanned);

    let reviewer_completed_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::ReviewerCompleted, json!({})));
    assert_eq!(reviewer_completed_actor.kind, SessionAuditActorKind::Reviewer);
    let reasoning_completed_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::ReasoningParticipantCompleted,
        json!({"participant_id": UNKNOWN_PARTICIPANT_ID}),
    ));
    assert_eq!(reasoning_completed_actor.id, UNKNOWN_PARTICIPANT_ID);
    let governance_completed_actor = trace_event_audit_actor(&sample_trace_event(
        TraceEventType::GovernanceCompleted,
        json!({"runtime": "canon", "stage_key": IMPLEMENTATION_STAGE_KEY}),
    ));
    assert_eq!(governance_completed_actor.id, "canon");
    let decision_created_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::DecisionCreated, json!({})));
    assert_eq!(decision_created_actor.id, DECISION_LOOP_ID);
    let decision_verified_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::DecisionVerified, json!({})));
    assert_eq!(decision_verified_actor.id, DECISION_LOOP_ID);
    let decision_recovered_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::DecisionRecovered, json!({})));
    assert_eq!(decision_recovered_actor.id, DECISION_LOOP_ID);
    let review_started_actor =
        trace_event_audit_actor(&sample_trace_event(TraceEventType::ReviewStarted, json!({})));
    assert_eq!(review_started_actor.id, REVIEW_COUNCIL_ID);
}

#[test]
fn run_stage_runtime_support_helper_utilities_cover_additional_projection_variants() {
    for (event_type, expected_phase, expected_family, expected_name) in [
        (
            TraceEventType::ProjectScalePathProposed,
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
        ),
        (
            TraceEventType::ProjectScaleStageTransitioned,
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
        ),
        (
            TraceEventType::ReviewStarted,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
        ),
        (
            TraceEventType::ReviewAdjudicated,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
        ),
        (
            TraceEventType::ReviewTerminalRecorded,
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
        ),
        (
            TraceEventType::ReasoningProfileActivated,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        (
            TraceEventType::ReasoningParticipantStarted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        (
            TraceEventType::ReasoningParticipantCompleted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        (
            TraceEventType::ReasoningDisagreementRecorded,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        (
            TraceEventType::ReasoningDebateRoundCompleted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        (
            TraceEventType::ReasoningReflexionRevisionCompleted,
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        (
            TraceEventType::GovernanceSelected,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
        ),
        (
            TraceEventType::GovernanceDecisionRecorded,
            SessionAuditPhase::Governance,
            "governance",
            "build_autopilot_decision",
        ),
        (
            TraceEventType::GovernancePacketRejected,
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
        ),
        (
            TraceEventType::TerminalRecorded,
            SessionAuditPhase::Run,
            "session_runtime",
            "finalize_task",
        ),
    ] {
        let algorithm = trace_event_audit_algorithm(event_type);
        assert_eq!(algorithm.phase, expected_phase);
        assert_eq!(algorithm.family, expected_family);
        assert_eq!(algorithm.name, expected_name);
    }

    for event_type in [
        TraceEventType::ProjectScalePathProposed,
        TraceEventType::GovernanceSelected,
        TraceEventType::GovernanceStarted,
        TraceEventType::GovernanceDecisionRecorded,
        TraceEventType::ReviewStarted,
        TraceEventType::ReasoningProfileActivated,
        TraceEventType::ReasoningParticipantStarted,
    ] {
        let outcome = trace_event_audit_outcome(&sample_trace_event(event_type, json!({})));
        assert_eq!(outcome.status, SessionAuditOutcomeStatus::Started);
        assert!(!outcome.blocking);
    }

    for event_type in [
        TraceEventType::ProjectScaleStageTransitioned,
        TraceEventType::ReviewAdjudicated,
        TraceEventType::ReviewTerminalRecorded,
        TraceEventType::ReasoningParticipantCompleted,
        TraceEventType::ReasoningDisagreementRecorded,
        TraceEventType::ReasoningDebateRoundCompleted,
        TraceEventType::ReasoningReflexionRevisionCompleted,
        TraceEventType::TerminalRecorded,
    ] {
        let outcome = trace_event_audit_outcome(&sample_trace_event(
            event_type,
            json!({"summary": "completed"}),
        ));
        assert_eq!(outcome.status, SessionAuditOutcomeStatus::Completed);
        assert!(!outcome.blocking);
    }

    for event_type in [
        TraceEventType::ReasoningProfileEscalated,
        TraceEventType::DecisionFailed,
        TraceEventType::StageFailed,
    ] {
        let outcome = trace_event_audit_outcome(&sample_trace_event(
            event_type,
            json!({"summary": "failed"}),
        ));
        assert_eq!(outcome.status, SessionAuditOutcomeStatus::Failed);
        assert!(!outcome.blocking);
    }

    let interrupted_outcome = trace_event_audit_outcome(&sample_trace_event(
        TraceEventType::ReasoningProfileInterrupted,
        json!({"summary": "paused"}),
    ));
    assert_eq!(interrupted_outcome.status, SessionAuditOutcomeStatus::Awaiting);
    assert!(!interrupted_outcome.blocking);
}

fn sample_trace_event(event_type: TraceEventType, payload: Value) -> TraceEvent {
    TraceEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type,
        step_id: None,
        plan_revision: 1,
        payload,
        recorded_at: UPDATED_AT,
    }
}

fn sample_stage_runtime(
    hook_subscriptions: Vec<FrameworkHookKey>,
) -> FrameworkAdapterClaimedStageRuntime {
    FrameworkAdapterClaimedStageRuntime {
        run_id: RUN_ID.to_string(),
        adapter_id: ADAPTER_ID.to_string(),
        hook_subscriptions,
    }
}

fn sample_session(workspace: &Path) -> ActiveSessionRecord {
    ActiveSessionRecord {
        session_id: SESSION_ID.to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: UPDATED_AT,
        updated_at: UPDATED_AT,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
    }
}

fn sample_adapter_selection(command: &str) -> PersistedAdapterConfiguration {
    PersistedAdapterConfiguration {
        selection: AdapterSelectionRecord {
            selection_mode: AdapterSelectionMode::KnownProfile,
            adapter_id: ADAPTER_ID.to_string(),
            display_name: ADAPTER_DISPLAY_NAME.to_string(),
            command: command.to_string(),
            args: Vec::new(),
            registration_source: AdapterRegistrationSource::AdapterAdd,
            discovery_state: AdapterDiscoveryState::ExplicitCommand,
            compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
            updated_at: UPDATED_AT,
        },
        schema_fingerprint: SCHEMA_FINGERPRINT.to_string(),
        completeness_state: AdapterConfigCompletenessState::Complete,
        interactive_resolution: false,
        last_validated_at: Some(UPDATED_AT),
        value_count: 0,
        values: Vec::new(),
    }
}

fn save_local_adapter(
    workspace: &Path,
    selection: PersistedAdapterConfiguration,
) -> Result<(), Box<dyn Error>> {
    FileConfigStore::for_workspace(workspace)
        .save_local(&ConfigFile { adapter: Some(selection), ..ConfigFile::default() })?;
    Ok(())
}

fn goal_plan_has_any_fallback_reason(session: &ActiveSessionRecord) -> bool {
    session.goal_plan.as_ref().is_some_and(|goal_plan| {
        goal_plan
            .planning_rationale
            .as_deref()
            .is_some_and(|rationale| rationale.contains("adapter_fallback_reason:"))
    })
}

fn goal_plan_has_fallback_reason(session: &ActiveSessionRecord, reason: &str) -> bool {
    session.goal_plan.as_ref().is_some_and(|goal_plan| {
        goal_plan
            .planning_rationale
            .as_deref()
            .is_some_and(|rationale| rationale.contains(&goal_plan_fallback_note(reason)))
    })
}

fn goal_plan_fallback_note(reason: &str) -> String {
    format!("adapter_fallback_reason: {reason}")
}

fn blocked_execute_stage_response() -> crate::adapters::FrameworkAdapterExecuteStageResponse {
    let mut response = sample_framework_adapter_execute_stage_success_response();
    response.status = crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked;
    response.summary = "run stage blocked pending operator action".to_string();
    response.workflow_id = Some("speckit-implementation".to_string());
    response.implementation_status =
        Some(crate::adapters::FrameworkAdapterImplementationStatus::Blocked);
    response.validation_refs = vec!["validation/run.md".to_string()];
    response.next_action = Some("resume run stage".to_string());
    response
}

fn sample_checkpoint_projection(
    workspace: &Path,
) -> crate::orchestrator::session_runtime::CheckpointProjectionState {
    crate::orchestrator::session_runtime::CheckpointProjectionState {
        checkpoint_id: CHECKPOINT_ID.to_string(),
        scope: CHECKPOINT_SCOPE_WORKSPACE.to_string(),
        restore_command: CHECKPOINT_RESTORE_COMMAND.to_string(),
        workspace_refs: vec![workspace.to_string_lossy().into_owned()],
    }
}

fn sample_goal_plan_session(workspace: &Path) -> Result<ActiveSessionRecord, Box<dyn Error>> {
    let goal_plan = sample_goal_plan()?;
    let goal = goal_plan.goal_text.clone();
    Ok(ActiveSessionRecord {
        session_id: SESSION_ID.to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some(goal),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(goal_plan),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: UPDATED_AT,
        updated_at: UPDATED_AT,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
    })
}

fn sample_goal_plan() -> Result<GoalPlan, Box<dyn Error>> {
    GoalPlan::new(
        "Drive the framework-adapter run stage",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Repair arithmetic".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: None,
        }],
    )
    .map_err(Into::into)
}

fn write_goal_plan_target_file(workspace: &Path) -> Result<(), Box<dyn Error>> {
    let source_dir = workspace.join("src");
    fs::create_dir_all(&source_dir)?;
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"boundline-runtime-support-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )?;
    fs::write(
        source_dir.join("lib.rs"),
        "#[cfg(test)]\nmod tests {\n    fn compute(left: i32, right: i32) -> i32 {\n        left - right\n    }\n\n    #[test]\n    fn computes_addition() {\n        assert_eq!(compute(2, 3), 5);\n    }\n}\n",
    )?;
    Ok(())
}

fn write_execution_profile(workspace: &Path) -> Result<(), Box<dyn Error>> {
    let execution_path = crate::fixture::execution_manifest_path(workspace);
    if let Some(parent) = execution_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let profile = WorkspaceExecutionProfile {
        name: "runtime-support-profile".to_string(),
        read_targets: vec!["src/lib.rs".to_string()],
        validation_command: ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string()],
        },
        attempts: vec![ExecutionAttemptDefinition {
            attempt_id: "attempt-1".to_string(),
            summary: "apply cluster change".to_string(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "before".to_string(),
                replace: "after".to_string(),
            }],
        }],
        adaptive: None,
        limits: crate::domain::limits::RunLimits::default(),
        governance: None,
        review: None,
        legacy_source: None,
    };
    fs::write(execution_path, serde_json::to_vec_pretty(&profile)?)?;
    Ok(())
}

fn write_framework_adapter_script(
    workspace: &Path,
    describe: &crate::adapters::FrameworkAdapterDescribeResponse,
    preflight_mode: PreflightMode,
    execute_mode: ExecuteMode,
) -> Result<String, Box<dyn Error>> {
    let describe_path = workspace.join(DESCRIBE_RESPONSE_FILE_NAME);
    let preflight_path = workspace.join(PREFLIGHT_RESPONSE_FILE_NAME);
    let execute_path = workspace.join(EXECUTE_RESPONSE_FILE_NAME);
    let emit_hook_path = workspace.join(EMIT_HOOK_RESPONSE_FILE_NAME);
    let script_path = workspace.join(FRAMEWORK_ADAPTER_SCRIPT_FILE_NAME);

    fs::write(
        &describe_path,
        serde_json::to_string(&sample_framework_adapter_success_envelope(describe.clone()))?,
    )?;
    fs::write(
        &emit_hook_path,
        serde_json::to_string(&sample_framework_adapter_success_envelope(
            sample_framework_adapter_hook_emission_response(),
        ))?,
    )?;

    let preflight_block = match preflight_mode {
        PreflightMode::Response(response) => {
            fs::write(
                &preflight_path,
                serde_json::to_string(&sample_framework_adapter_success_envelope(response))?,
            )?;
            format!("cat >/dev/null\n  cat '{}'", preflight_path.to_string_lossy())
        }
        PreflightMode::ProcessFailure => {
            "cat >/dev/null\n  echo 'preflight failed' >&2\n  exit 1".to_string()
        }
    };

    let execute_block = match execute_mode {
        ExecuteMode::Response(response) => {
            fs::write(
                &execute_path,
                serde_json::to_string(&sample_framework_adapter_success_envelope(response))?,
            )?;
            format!("cat >/dev/null\n  cat '{}'", execute_path.to_string_lossy())
        }
        ExecuteMode::ProcessFailure => {
            "cat >/dev/null\n  echo 'transport failed' >&2\n  exit 1".to_string()
        }
    };

    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\ncase \"$1\" in\ndescribe)\n  cat '{}'\n  ;;\npreflight)\n  {}\n  ;;\nexecute-stage)\n  {}\n  ;;\nemit-hook)\n  cat >/dev/null\n  cat '{}'\n  ;;\n*)\n  exit 1\n  ;;\nesac\n",
            describe_path.to_string_lossy(),
            preflight_block,
            execute_block,
            emit_hook_path.to_string_lossy(),
        ),
    )?;

    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    Ok(path_string(&script_path))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
    TestWorkspace::new(prefix)
}

enum PreflightMode {
    Response(crate::adapters::FrameworkAdapterPreflightResponse),
    ProcessFailure,
}

#[allow(clippy::large_enum_variant)]
enum ExecuteMode {
    Response(crate::adapters::FrameworkAdapterExecuteStageResponse),
    ProcessFailure,
}

struct TestWorkspace {
    path: PathBuf,
}

impl TestWorkspace {
    fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn as_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
