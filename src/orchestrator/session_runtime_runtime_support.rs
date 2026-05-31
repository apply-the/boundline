use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::agent::{
    FrameworkAdapterHost, FrameworkAdapterHostError, SubprocessFrameworkAdapterHost,
};
use crate::adapters::audit_store::{FileSessionAuditStore, FrameworkAdapterHookAuditStore};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::adapters::trace_store::TraceStore;
use crate::domain::configuration::PersistedAdapterConfiguration;
use crate::domain::execution::StageRoutingDecisionRecord;
use crate::domain::framework_adapter::{
    AdapterExecutionSource, AdapterFailureClass, AdapterHookKey, AdapterLifecycleStageKey,
    HookDispatchStatus, LifecycleStageExecutionStatus, StageClaimState, StageRoutingDecisionReason,
};
use crate::domain::review::{ReviewerParticipation, ReviewerParticipationStatus};
use crate::domain::session::{FrameworkAdapterStageFailureDetails, LifecycleStageExecutionRecord};
use crate::domain::trace::HookEventDispatchRecord;

use super::{
    ActiveSessionRecord, CanonMode, LATEST_CHANGED_FILES_KEY, ProfileActivationRecord,
    RoutingConfig, RuntimeKind, SYSTEM_CONTEXT_EXISTING_TEXT, SYSTEM_CONTEXT_NEW_TEXT,
    SessionAuditActor, SessionAuditActorKind, SessionAuditAlgorithm, SessionAuditOutcome,
    SessionAuditOutcomeStatus, SessionAuditPhase, SessionRuntime, SessionRuntimeError,
    SessionStatus, SystemContextBinding, TaskContext, TaskRunResponse, TaskStatus,
    TerminalCondition, TraceEvent, TraceEventType, UPSTREAM_EVIDENCE_MAX_CHARS,
    build_terminal_reason, load_workspace_execution_profile,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FrameworkAdapterRuntimeBinding {
    pub selection: PersistedAdapterConfiguration,
    pub host: SubprocessFrameworkAdapterHost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FrameworkAdapterClaimedStageRuntime {
    pub run_id: String,
    pub adapter_id: String,
    pub hook_subscriptions: Vec<crate::orchestrator::FrameworkHookKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum FrameworkAdapterRunStageOutcome {
    NotClaimed {
        routing_record: Option<StageRoutingDecisionRecord>,
    },
    Completed {
        stage_runtime: FrameworkAdapterClaimedStageRuntime,
        response: crate::adapters::FrameworkAdapterExecuteStageResponse,
    },
    Blocked(FrameworkAdapterStageFailureDetails),
    Terminal {
        stage_runtime: FrameworkAdapterClaimedStageRuntime,
        response: Box<TaskRunResponse>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct FrameworkAdapterStageFailedTracePayload {
    pub(super) stage_id: String,
    pub(super) stage_key: AdapterLifecycleStageKey,
    pub(super) reason: String,
    pub(super) summary: String,
    pub(super) framework_adapter_stage_failure: FrameworkAdapterStageFailureDetails,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub(super) enum FrameworkAdapterRuntimeSupportError {
    #[error("framework-adapter config store operation failed: {0}")]
    ConfigStore(#[from] ConfigStoreError),
    #[error("framework-adapter host setup failed: {0}")]
    Host(#[from] FrameworkAdapterHostError),
}

#[allow(dead_code)]
pub(super) fn configured_framework_adapter_binding(
    workspace: &Path,
) -> Result<Option<FrameworkAdapterRuntimeBinding>, FrameworkAdapterRuntimeSupportError> {
    let Some(selection) = FileConfigStore::for_workspace(workspace).local_adapter()? else {
        return Ok(None);
    };

    let host = framework_adapter_host_from_selection(workspace, &selection)?;
    Ok(Some(FrameworkAdapterRuntimeBinding { selection, host }))
}

impl SessionRuntime {
    pub(super) fn maybe_execute_framework_adapter_run_stage(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<super::CheckpointProjectionState>,
    ) -> Result<FrameworkAdapterRunStageOutcome, SessionRuntimeError> {
        let binding =
            configured_framework_adapter_binding(&self.workspace_ref).map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to load framework-adapter runtime binding: {error}"
                ))
            })?;
        let Some(binding) = binding else {
            return Ok(FrameworkAdapterRunStageOutcome::NotClaimed { routing_record: None });
        };
        let adapter_id = binding.selection.selection.adapter_id.clone();

        let describe = match binding.host.describe() {
            Ok(describe) => describe,
            Err(_) => {
                append_run_stage_adapter_fallback_reason(session, "unavailable_binary");
                return Ok(FrameworkAdapterRunStageOutcome::NotClaimed {
                    routing_record: Some(framework_adapter_run_stage_not_claimed_record(
                        session,
                        Some(adapter_id.clone()),
                        StageRoutingDecisionReason::CompatibilityBlocked,
                    )),
                });
            }
        };

        if !describe.declared_stage_overrides.contains(&crate::orchestrator::FrameworkStageKey::Run)
        {
            return Ok(FrameworkAdapterRunStageOutcome::NotClaimed {
                routing_record: Some(framework_adapter_run_stage_not_claimed_record(
                    session,
                    Some(adapter_id.clone()),
                    StageRoutingDecisionReason::UndeclaredStage,
                )),
            });
        }

        if !crate::adapters::framework_adapter_supports_v1_transport(&describe.supported_transports)
        {
            append_run_stage_adapter_fallback_reason(session, "unsupported_transport");
            return Ok(FrameworkAdapterRunStageOutcome::NotClaimed {
                routing_record: Some(framework_adapter_run_stage_not_claimed_record(
                    session,
                    Some(adapter_id.clone()),
                    StageRoutingDecisionReason::CompatibilityBlocked,
                )),
            });
        }

        let config_values = runtime_framework_adapter_config_values(&binding.selection);
        let preflight =
            match binding.host.preflight(&crate::adapters::FrameworkAdapterPreflightRequest {
                boundline_version: env!("CARGO_PKG_VERSION").to_string(),
                workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
                non_interactive: true,
                config_values: config_values.clone(),
            }) {
                Ok(preflight) => preflight,
                Err(_) => {
                    append_run_stage_adapter_fallback_reason(session, "unavailable_binary");
                    return Ok(FrameworkAdapterRunStageOutcome::NotClaimed {
                        routing_record: Some(framework_adapter_run_stage_not_claimed_record(
                            session,
                            Some(adapter_id.clone()),
                            StageRoutingDecisionReason::CompatibilityBlocked,
                        )),
                    });
                }
            };

        if preflight.status == crate::adapters::FrameworkAdapterPreflightStatus::Blocked {
            append_run_stage_adapter_fallback_reason(session, "preflight_blocked");
            return Ok(FrameworkAdapterRunStageOutcome::NotClaimed {
                routing_record: Some(framework_adapter_run_stage_not_claimed_record(
                    session,
                    Some(adapter_id.clone()),
                    StageRoutingDecisionReason::PreflightBlocked,
                )),
            });
        }

        let runtime_config_values = if preflight.normalized_config_values.is_empty() {
            config_values
        } else {
            preflight.normalized_config_values.clone()
        };

        let run_id = Uuid::new_v4();
        let stage_runtime = FrameworkAdapterClaimedStageRuntime {
            run_id: run_id.to_string(),
            adapter_id: binding.selection.selection.adapter_id.clone(),
            hook_subscriptions: describe.declared_hook_subscriptions.clone(),
        };
        let request = crate::adapters::FrameworkAdapterExecuteStageRequest {
            run_id,
            stage_key: crate::orchestrator::FrameworkStageKey::Run,
            stage_attempt: 1,
            workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
            adapter_id: binding.selection.selection.adapter_id.clone(),
            config_values: runtime_config_values,
            context_artifacts: Vec::new(),
        };

        match binding.host.execute_stage(&request) {
            Ok(response)
                if response.status
                    == crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded =>
            {
                Ok(FrameworkAdapterRunStageOutcome::Completed { stage_runtime, response })
            }
            Ok(response)
                if response.status
                    == crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked =>
            {
                Ok(FrameworkAdapterRunStageOutcome::Blocked(
                    self.framework_adapter_stage_blocked_from_execute_response(
                        &stage_runtime,
                        response,
                    ),
                ))
            }
            Ok(response) => {
                let failure = self.framework_adapter_stage_failure_from_execute_response(
                    &stage_runtime,
                    response,
                );
                let response = self.persist_framework_adapter_run_stage_failure(
                    session,
                    checkpoint_projection,
                    failure,
                )?;
                Ok(FrameworkAdapterRunStageOutcome::Terminal {
                    stage_runtime,
                    response: Box::new(response),
                })
            }
            Err(error) => {
                let failure =
                    self.framework_adapter_stage_failure_from_host_error(&stage_runtime, error);
                let response = self.persist_framework_adapter_run_stage_failure(
                    session,
                    checkpoint_projection,
                    failure,
                )?;
                Ok(FrameworkAdapterRunStageOutcome::Terminal {
                    stage_runtime,
                    response: Box::new(response),
                })
            }
        }
    }

    pub(super) fn emit_framework_adapter_run_stage_hook(
        &self,
        session: &ActiveSessionRecord,
        stage_runtime: &FrameworkAdapterClaimedStageRuntime,
        terminal_status: TaskStatus,
        trace_location: &str,
    ) -> Result<(), SessionRuntimeError> {
        let Some((framework_hook_key, adapter_hook_key)) =
            run_stage_hook_keys_for_status(terminal_status)
        else {
            return Ok(());
        };
        if !stage_runtime.hook_subscriptions.contains(&framework_hook_key) {
            return Ok(());
        }

        let binding =
            configured_framework_adapter_binding(&self.workspace_ref).map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to reload framework-adapter runtime binding for hook dispatch: {error}"
                ))
            })?;
        let Some(binding) = binding else {
            return Ok(());
        };

        let hook_result = binding.host.emit_hook(&crate::adapters::HookEmissionRequest {
            run_id: Uuid::new_v4(),
            hook_key: framework_hook_key,
            stage_key: crate::orchestrator::FrameworkStageKey::Run,
            stage_claimed: true,
            workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
            payload_ref: trace_location.to_string(),
        });

        let (dispatch_status, summary) = match hook_result {
            Ok(response) => (hook_dispatch_status_from_response(response.status), response.summary),
            Err(error) => (
                HookDispatchStatus::Failed,
                format!("framework-adapter hook delivery failed: {error}"),
            ),
        };

        let record = HookEventDispatchRecord {
            run_id: session.session_id.clone(),
            hook_key: adapter_hook_key,
            stage_key: AdapterLifecycleStageKey::Run,
            adapter_id: stage_runtime.adapter_id.clone(),
            stage_claimed: true,
            payload_ref: trace_location.to_string(),
            dispatch_status,
            summary,
            recorded_at: crate::domain::trace::current_timestamp_millis(),
        };

        FileSessionAuditStore::for_session(&self.workspace_ref, &session.session_id)
            .append_hook_dispatch(&record)
            .map(|_| ())
            .map_err(SessionRuntimeError::SessionAuditStore)
    }

    pub(super) fn record_framework_adapter_run_stage_not_claimed_routing(
        &self,
        session: &ActiveSessionRecord,
        trace_location: &str,
        plan_revision: usize,
        routing_record: StageRoutingDecisionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let mut trace = self
            .trace_store
            .load(Path::new(trace_location))
            .map_err(SessionRuntimeError::TraceStore)?;
        trace.record_event(
            TraceEventType::StageRouted,
            None,
            plan_revision,
            framework_adapter_stage_routing_value(framework_adapter_stage_routing_trace_payload(
                routing_record,
            ))?,
        );
        self.persist_trace(&session.session_id, &mut trace).map(|_| ())
    }

    fn framework_adapter_stage_failure_from_execute_response(
        &self,
        stage_runtime: &FrameworkAdapterClaimedStageRuntime,
        response: crate::adapters::FrameworkAdapterExecuteStageResponse,
    ) -> FrameworkAdapterStageFailureDetails {
        let finished_at = crate::domain::trace::current_timestamp_millis();
        let failure_class = response
            .failure_class
            .map(map_framework_adapter_failure_class)
            .or(Some(AdapterFailureClass::AdapterRuntime));
        let status = match response.status {
            crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded => {
                LifecycleStageExecutionStatus::Succeeded
            }
            crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked => {
                LifecycleStageExecutionStatus::Blocked
            }
            crate::adapters::FrameworkAdapterStageExecutionStatus::Failed => {
                LifecycleStageExecutionStatus::Failed
            }
        };
        FrameworkAdapterStageFailureDetails {
            execution: LifecycleStageExecutionRecord {
                run_id: stage_runtime.run_id.clone(),
                stage_key: AdapterLifecycleStageKey::Run,
                execution_source: AdapterExecutionSource::Adapter,
                adapter_id: Some(stage_runtime.adapter_id.clone()),
                status,
                intervention_required: true,
                failure_class,
                produced_artifacts: response.produced_artifacts,
                started_at: Some(finished_at),
                finished_at: Some(finished_at),
            },
            claim_state: StageClaimState::FailedAfterClaim,
            summary: response.summary,
            detail: None,
            protocol_error_code: None,
        }
    }

    fn framework_adapter_stage_blocked_from_execute_response(
        &self,
        stage_runtime: &FrameworkAdapterClaimedStageRuntime,
        response: crate::adapters::FrameworkAdapterExecuteStageResponse,
    ) -> FrameworkAdapterStageFailureDetails {
        let finished_at = crate::domain::trace::current_timestamp_millis();

        FrameworkAdapterStageFailureDetails {
            execution: LifecycleStageExecutionRecord {
                run_id: stage_runtime.run_id.clone(),
                stage_key: AdapterLifecycleStageKey::Run,
                execution_source: AdapterExecutionSource::Adapter,
                adapter_id: Some(stage_runtime.adapter_id.clone()),
                status: LifecycleStageExecutionStatus::Blocked,
                intervention_required: true,
                failure_class: None,
                produced_artifacts: response.produced_artifacts,
                started_at: Some(finished_at),
                finished_at: Some(finished_at),
            },
            claim_state: StageClaimState::Claimed,
            summary: response.summary,
            detail: response.next_action,
            protocol_error_code: None,
        }
    }

    fn framework_adapter_stage_failure_from_host_error(
        &self,
        stage_runtime: &FrameworkAdapterClaimedStageRuntime,
        error: FrameworkAdapterHostError,
    ) -> FrameworkAdapterStageFailureDetails {
        let (failure_class, protocol_error_code) = match &error {
            FrameworkAdapterHostError::DeserializeResponse { .. }
            | FrameworkAdapterHostError::InvalidEnvelope { .. }
            | FrameworkAdapterHostError::ProtocolError { .. } => {
                (AdapterFailureClass::ProtocolError, protocol_error_code_from_host_error(&error))
            }
            FrameworkAdapterHostError::EmptyCommand
            | FrameworkAdapterHostError::SerializeRequest { .. }
            | FrameworkAdapterHostError::Spawn { .. }
            | FrameworkAdapterHostError::WriteRequest { .. }
            | FrameworkAdapterHostError::Wait { .. }
            | FrameworkAdapterHostError::ProcessFailed { .. } => {
                (AdapterFailureClass::TransportFailure, None)
            }
        };
        let summary = match failure_class {
            AdapterFailureClass::ProtocolError => {
                let code_suffix = protocol_error_code
                    .as_deref()
                    .map(|code| format!(" code={code}"))
                    .unwrap_or_default();
                format!(
                    "framework-adapter returned a protocol error after claiming run stage{code_suffix}"
                )
            }
            AdapterFailureClass::TransportFailure => {
                "framework-adapter transport failed after claiming run stage".to_string()
            }
            _ => "framework-adapter run stage failed after claim".to_string(),
        };
        let finished_at = crate::domain::trace::current_timestamp_millis();
        FrameworkAdapterStageFailureDetails {
            execution: LifecycleStageExecutionRecord {
                run_id: stage_runtime.run_id.clone(),
                stage_key: AdapterLifecycleStageKey::Run,
                execution_source: AdapterExecutionSource::Adapter,
                adapter_id: Some(stage_runtime.adapter_id.clone()),
                status: LifecycleStageExecutionStatus::Failed,
                intervention_required: true,
                failure_class: Some(failure_class),
                produced_artifacts: Vec::new(),
                started_at: Some(finished_at),
                finished_at: Some(finished_at),
            },
            claim_state: StageClaimState::FailedAfterClaim,
            summary,
            detail: Some(error.to_string()),
            protocol_error_code,
        }
    }

    fn persist_framework_adapter_run_stage_failure(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<super::CheckpointProjectionState>,
        failure: FrameworkAdapterStageFailureDetails,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let Some(goal_plan) = session.goal_plan.clone() else {
            return Err(SessionRuntimeError::MissingGoalPlan);
        };

        let terminal_reason = build_terminal_reason(
            framework_adapter_stage_failure_terminal_condition(&failure),
            failure.summary.clone(),
            Some(serde_json::to_value(&failure).map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to serialize framework-adapter run-stage failure details: {error}"
                ))
            })?),
        );
        let mut trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
        trace.record_event(
            TraceEventType::StageRouted,
            None,
            goal_plan.proposal_revision,
            serde_json::to_value(framework_adapter_run_stage_routing_record_from_failure(&failure))
                .map_err(|error| {
                    SessionRuntimeError::ExecutionInvariant(format!(
                        "failed to serialize framework-adapter stage-routing trace payload: {error}"
                    ))
                })?,
        );
        trace.record_event(
            TraceEventType::StageFailed,
            None,
            goal_plan.proposal_revision,
            serde_json::to_value(&FrameworkAdapterStageFailedTracePayload {
                stage_id: AdapterLifecycleStageKey::Run.as_str().to_string(),
                stage_key: AdapterLifecycleStageKey::Run,
                reason: failure.summary.clone(),
                summary: failure.summary.clone(),
                framework_adapter_stage_failure: failure.clone(),
            })
            .map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to serialize framework-adapter stage-failed trace payload: {error}"
                ))
            })?,
        );

        self.persist_native_result(
            session,
            goal_plan,
            Vec::new(),
            trace,
            super::NativePersistenceInput {
                checkpoint_projection,
                terminal_reason,
                limits: crate::domain::limits::RunLimits::default(),
                native_context: TaskContext::new(
                    session.session_id.clone(),
                    session.workspace_ref.clone(),
                    crate::domain::limits::RunLimits::default(),
                    Default::default(),
                ),
                record_terminal_event: true,
                projected_task: None,
            },
        )
    }

    pub(super) fn persist_framework_adapter_run_stage_success(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<super::CheckpointProjectionState>,
        stage_runtime: &FrameworkAdapterClaimedStageRuntime,
        response: crate::adapters::FrameworkAdapterExecuteStageResponse,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let Some(goal_plan) = session.goal_plan.clone() else {
            return Err(SessionRuntimeError::MissingGoalPlan);
        };

        let terminal_reason =
            build_terminal_reason(TerminalCondition::GoalSatisfied, response.summary, None);
        let mut trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
        trace.record_event(
            TraceEventType::StageRouted,
            None,
            goal_plan.proposal_revision,
            framework_adapter_stage_routing_value(framework_adapter_stage_routing_trace_payload(
                framework_adapter_run_stage_routing_record(
                    stage_runtime,
                    StageClaimState::Completed,
                    Some(LifecycleStageExecutionStatus::Succeeded),
                    response.produced_artifacts,
                ),
            ))?,
        );

        self.persist_native_result(
            session,
            goal_plan,
            Vec::new(),
            trace,
            super::NativePersistenceInput {
                checkpoint_projection,
                terminal_reason,
                limits: crate::domain::limits::RunLimits::default(),
                native_context: TaskContext::new(
                    session.session_id.clone(),
                    session.workspace_ref.clone(),
                    crate::domain::limits::RunLimits::default(),
                    Default::default(),
                ),
                record_terminal_event: true,
                projected_task: None,
            },
        )
    }

    pub(super) fn persist_framework_adapter_run_stage_blocked(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<super::CheckpointProjectionState>,
        blocked: FrameworkAdapterStageFailureDetails,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let Some(goal_plan) = session.goal_plan.clone() else {
            return Err(SessionRuntimeError::MissingGoalPlan);
        };

        let terminal_reason = build_terminal_reason(
            TerminalCondition::NoCredibleNextStep,
            blocked.summary.clone(),
            Some(serde_json::to_value(&blocked).map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to serialize framework-adapter run-stage blocked details: {error}"
                ))
            })?),
        );
        let mut trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
        trace.record_event(
            TraceEventType::StageRouted,
            None,
            goal_plan.proposal_revision,
            framework_adapter_stage_routing_value(framework_adapter_stage_routing_trace_payload(
                framework_adapter_run_stage_routing_record_from_blocked(&blocked),
            ))?,
        );
        if let Some(checkpoint_projection) = checkpoint_projection.as_ref() {
            trace.record_event(
                TraceEventType::CheckpointCreated,
                None,
                goal_plan.proposal_revision,
                super::checkpoint_event_payload(checkpoint_projection),
            );
        }
        let trace_location = self.persist_trace(&session.session_id, &mut trace)?;
        let mut final_context = TaskContext::new(
            session.session_id.clone(),
            session.workspace_ref.clone(),
            crate::domain::limits::RunLimits::default(),
            Default::default(),
        );
        if let Some(checkpoint_projection) = checkpoint_projection.as_ref() {
            super::apply_checkpoint_projection_to_context(
                &mut final_context,
                checkpoint_projection,
            );
        }

        session.active_task = None;
        session.goal_plan = Some(goal_plan.clone());
        session.decisions.clear();
        session.latest_status = SessionStatus::Blocked;
        session.latest_terminal_reason = Some(terminal_reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = crate::domain::trace::current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: goal_plan.plan_id.clone(),
            terminal_status: TaskStatus::Failed,
            terminal_reason,
            final_context,
            plan_revision: goal_plan.proposal_revision,
            trace_location,
        })
    }
}

fn framework_adapter_run_stage_routing_record(
    stage_runtime: &FrameworkAdapterClaimedStageRuntime,
    claim_state: StageClaimState,
    stage_status: Option<LifecycleStageExecutionStatus>,
    produced_artifacts: Vec<String>,
) -> StageRoutingDecisionRecord {
    StageRoutingDecisionRecord {
        run_id: stage_runtime.run_id.clone(),
        stage_key: AdapterLifecycleStageKey::Run,
        execution_source: AdapterExecutionSource::Adapter,
        decision_reason: StageRoutingDecisionReason::DeclaredOverride,
        claim_state,
        adapter_id: Some(stage_runtime.adapter_id.clone()),
        stage_status,
        produced_artifacts,
        recorded_at: crate::domain::trace::current_timestamp_millis(),
    }
}

fn framework_adapter_run_stage_routing_record_from_failure(
    failure: &FrameworkAdapterStageFailureDetails,
) -> FrameworkAdapterStageRoutingTracePayload {
    framework_adapter_stage_routing_trace_payload(StageRoutingDecisionRecord {
        run_id: failure.execution.run_id.clone(),
        stage_key: failure.execution.stage_key,
        execution_source: failure.execution.execution_source,
        decision_reason: StageRoutingDecisionReason::DeclaredOverride,
        claim_state: failure.claim_state,
        adapter_id: failure.execution.adapter_id.clone(),
        stage_status: Some(failure.execution.status),
        produced_artifacts: failure.execution.produced_artifacts.clone(),
        recorded_at: crate::domain::trace::current_timestamp_millis(),
    })
}

fn framework_adapter_run_stage_routing_record_from_blocked(
    blocked: &FrameworkAdapterStageFailureDetails,
) -> StageRoutingDecisionRecord {
    StageRoutingDecisionRecord {
        run_id: blocked.execution.run_id.clone(),
        stage_key: blocked.execution.stage_key,
        execution_source: blocked.execution.execution_source,
        decision_reason: StageRoutingDecisionReason::DeclaredOverride,
        claim_state: blocked.claim_state,
        adapter_id: blocked.execution.adapter_id.clone(),
        stage_status: Some(blocked.execution.status),
        produced_artifacts: blocked.execution.produced_artifacts.clone(),
        recorded_at: crate::domain::trace::current_timestamp_millis(),
    }
}

fn framework_adapter_run_stage_not_claimed_record(
    session: &ActiveSessionRecord,
    adapter_id: Option<String>,
    decision_reason: StageRoutingDecisionReason,
) -> StageRoutingDecisionRecord {
    StageRoutingDecisionRecord {
        run_id: session.session_id.clone(),
        stage_key: AdapterLifecycleStageKey::Run,
        execution_source: AdapterExecutionSource::BuiltIn,
        decision_reason,
        claim_state: StageClaimState::NotClaimed,
        adapter_id,
        stage_status: None,
        produced_artifacts: Vec::new(),
        recorded_at: crate::domain::trace::current_timestamp_millis(),
    }
}

pub(super) fn framework_adapter_stage_routing_trace_payload(
    routing_record: StageRoutingDecisionRecord,
) -> FrameworkAdapterStageRoutingTracePayload {
    FrameworkAdapterStageRoutingTracePayload {
        summary: format!(
            "framework-adapter routed {} stage as {} ({})",
            routing_record.stage_key.as_str(),
            routing_record.claim_state.as_str(),
            routing_record.decision_reason.as_str()
        ),
        framework_adapter_stage_routing: routing_record,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct FrameworkAdapterStageRoutingTracePayload {
    pub(super) summary: String,
    pub(super) framework_adapter_stage_routing: StageRoutingDecisionRecord,
}

pub(super) fn framework_adapter_stage_routing_value(
    payload: FrameworkAdapterStageRoutingTracePayload,
) -> Result<Value, SessionRuntimeError> {
    serde_json::to_value(payload).map_err(|error| {
        SessionRuntimeError::ExecutionInvariant(format!(
            "failed to serialize framework-adapter stage-routing record: {error}"
        ))
    })
}

fn framework_adapter_host_from_selection(
    workspace: &Path,
    selection: &PersistedAdapterConfiguration,
) -> Result<SubprocessFrameworkAdapterHost, FrameworkAdapterHostError> {
    let mut host = SubprocessFrameworkAdapterHost::new(selection.selection.command.clone())?
        .with_args(selection.selection.args.clone());
    if workspace.is_dir() {
        host = host.with_working_directory(workspace.to_path_buf());
    }
    Ok(host)
}

fn append_run_stage_adapter_fallback_reason(session: &mut ActiveSessionRecord, reason: &str) {
    let Some(goal_plan) = session.goal_plan.as_mut() else {
        return;
    };
    let note = format!("adapter_fallback_reason: {reason}");
    goal_plan.planning_rationale = Some(match goal_plan.planning_rationale.take() {
        Some(existing) if existing.contains(&note) => existing,
        Some(existing) => format!("{existing}; {note}"),
        None => note,
    });
}

fn runtime_framework_adapter_config_values(
    selection: &PersistedAdapterConfiguration,
) -> Vec<crate::adapters::FrameworkAdapterConfigValue> {
    selection
        .values
        .iter()
        .map(|value| crate::adapters::FrameworkAdapterConfigValue {
            field_key: value.field_key.clone(),
            value_kind: match value.value_kind {
                crate::domain::framework_adapter::AdapterValueKind::String => {
                    crate::adapters::FrameworkAdapterFieldValueKind::String
                }
                crate::domain::framework_adapter::AdapterValueKind::Path => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Path
                }
                crate::domain::framework_adapter::AdapterValueKind::Boolean => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Boolean
                }
                crate::domain::framework_adapter::AdapterValueKind::Integer => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Integer
                }
                crate::domain::framework_adapter::AdapterValueKind::Enum => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Enum
                }
            },
            string_value: value.string_value.clone(),
            path_value: value.path_value.clone(),
            bool_value: value.bool_value,
            int_value: value.int_value,
        })
        .collect()
}

pub(super) fn map_framework_adapter_failure_class(
    failure_class: crate::adapters::FrameworkAdapterFailureClass,
) -> AdapterFailureClass {
    match failure_class {
        crate::adapters::FrameworkAdapterFailureClass::Preflight => AdapterFailureClass::Preflight,
        crate::adapters::FrameworkAdapterFailureClass::Manifest => AdapterFailureClass::Manifest,
        crate::adapters::FrameworkAdapterFailureClass::MissingConfig => {
            AdapterFailureClass::MissingConfig
        }
        crate::adapters::FrameworkAdapterFailureClass::AdapterRuntime => {
            AdapterFailureClass::AdapterRuntime
        }
        crate::adapters::FrameworkAdapterFailureClass::Compatibility => {
            AdapterFailureClass::Compatibility
        }
    }
}

pub(super) fn protocol_error_code_from_host_error(
    error: &FrameworkAdapterHostError,
) -> Option<String> {
    match error {
        FrameworkAdapterHostError::ProtocolError { code, .. } => Some(code.clone()),
        _ => None,
    }
}

pub(super) fn framework_adapter_stage_failure_terminal_condition(
    failure: &FrameworkAdapterStageFailureDetails,
) -> TerminalCondition {
    match failure.execution.status {
        LifecycleStageExecutionStatus::Blocked => TerminalCondition::NoCredibleNextStep,
        LifecycleStageExecutionStatus::Failed
            if matches!(
                failure.execution.failure_class,
                Some(AdapterFailureClass::ProtocolError | AdapterFailureClass::TransportFailure)
            ) =>
        {
            TerminalCondition::UnrecoverableError
        }
        LifecycleStageExecutionStatus::Succeeded | LifecycleStageExecutionStatus::Skipped => {
            TerminalCondition::TaskNotCredible
        }
        LifecycleStageExecutionStatus::Failed => TerminalCondition::TaskNotCredible,
    }
}

fn run_stage_hook_keys_for_status(
    terminal_status: TaskStatus,
) -> Option<(crate::orchestrator::FrameworkHookKey, AdapterHookKey)> {
    match terminal_status {
        TaskStatus::Succeeded => Some((
            crate::orchestrator::FrameworkHookKey::StageCompleted,
            AdapterHookKey::StageCompleted,
        )),
        TaskStatus::Failed | TaskStatus::Exhausted | TaskStatus::Aborted => {
            Some((crate::orchestrator::FrameworkHookKey::StageFailed, AdapterHookKey::StageFailed))
        }
        TaskStatus::Planned | TaskStatus::Running => None,
    }
}

fn hook_dispatch_status_from_response(
    status: crate::adapters::FrameworkAdapterHookDeliveryStatus,
) -> HookDispatchStatus {
    match status {
        crate::adapters::FrameworkAdapterHookDeliveryStatus::Delivered => {
            HookDispatchStatus::Delivered
        }
        crate::adapters::FrameworkAdapterHookDeliveryStatus::Ignored => HookDispatchStatus::Ignored,
        crate::adapters::FrameworkAdapterHookDeliveryStatus::Warning => HookDispatchStatus::Warning,
        crate::adapters::FrameworkAdapterHookDeliveryStatus::Failed => HookDispatchStatus::Failed,
    }
}

pub(super) fn effective_assistant_runtimes(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> Vec<RuntimeKind> {
    workspace
        .filter(|config| !config.assistant_runtimes.is_empty())
        .map(|config| config.assistant_runtimes.clone())
        .or_else(|| {
            cluster
                .filter(|config| !config.assistant_runtimes.is_empty())
                .map(|config| config.assistant_runtimes.clone())
        })
        .or_else(|| {
            global
                .filter(|config| !config.assistant_runtimes.is_empty())
                .map(|config| config.assistant_runtimes.clone())
        })
        .unwrap_or_default()
}

pub(super) fn cluster_task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Exhausted => "exhausted",
        TaskStatus::Aborted => "aborted",
    }
}

pub(super) fn cluster_workspace_is_blocked(workspace_ref: &str) -> bool {
    let workspace = Path::new(workspace_ref);
    let Ok(profile) = load_workspace_execution_profile(workspace) else {
        return true;
    };

    !profile.attempts.iter().any(|attempt| {
        attempt.changes.iter().all(|change| {
            let Ok(contents) = std::fs::read_to_string(workspace.join(&change.path)) else {
                return false;
            };
            contents.contains(&change.find) || contents.contains(&change.replace)
        })
    })
}

pub(super) fn canon_workspace_scope_mismatch_reason(workspace: &Path) -> Option<String> {
    let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());
    let git_root = nearest_git_root(&workspace)?;
    if git_root == workspace {
        return None;
    }

    Some(format!(
        "planning governance requires a Canon workspace root, but Canon would target git root {} instead of workspace {}; use the repository root as the Boundline workspace or initialize a dedicated nested repository first",
        git_root.display(),
        workspace.display()
    ))
}

fn nearest_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub(super) fn git_config_value(workspace: &Path, key: &str) -> Option<String> {
    let output =
        Command::new("git").current_dir(workspace).args(["config", "--get", key]).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

pub(super) fn session_audit_outcome_for_status(status: SessionStatus) -> SessionAuditOutcome {
    match status {
        SessionStatus::Initialized => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Recorded, "session initialized")
        }
        SessionStatus::GoalCaptured => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Recorded,
            "goal captured for active session",
        ),
        SessionStatus::Planned => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "session planned")
        }
        SessionStatus::Blocked => {
            let mut outcome =
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Blocked, "session blocked");
            outcome.blocking = true;
            outcome
        }
        SessionStatus::Running => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session running")
        }
        SessionStatus::Succeeded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Succeeded, "session succeeded")
        }
        SessionStatus::Failed => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session failed")
        }
        SessionStatus::Exhausted => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Failed,
            "session exhausted its execution budget",
        ),
        SessionStatus::Aborted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session aborted")
        }
        SessionStatus::Invalid => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session invalid")
        }
    }
}

pub(super) fn trace_event_audit_algorithm(event_type: TraceEventType) -> SessionAuditAlgorithm {
    match event_type {
        TraceEventType::GoalPlanCreated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Plan,
            "goal_planner",
            "build_goal_plan_with_sources",
        ),
        TraceEventType::FlowInferred => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Plan, "session_runtime", "plan_goal_plan")
        }
        TraceEventType::ProjectScalePathProposed
        | TraceEventType::ProjectScaleStageTransitioned => SessionAuditAlgorithm::new(
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
        ),
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
        ),
        TraceEventType::ReviewCouncilAssembled => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_council",
            "resolve_council_assembly",
        ),
        TraceEventType::ReviewStopSemanticsRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_governance",
            "active_review_stop_semantics",
        ),
        TraceEventType::ReviewVoteResolved | TraceEventType::VotingDecisionRecorded => {
            SessionAuditAlgorithm::new(
                SessionAuditPhase::Review,
                "review_vote",
                "VoteRuleDefinition::resolve",
            )
        }
        TraceEventType::ReviewStarted
        | TraceEventType::ReviewerStarted
        | TraceEventType::ReviewerCompleted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewAdjudicated
        | TraceEventType::ReviewTerminalRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
        ),
        TraceEventType::ReasoningProfileActivated
        | TraceEventType::ReasoningParticipantStarted
        | TraceEventType::ReasoningParticipantCompleted
        | TraceEventType::ReasoningDisagreementRecorded
        | TraceEventType::ReasoningDebateRoundCompleted
        | TraceEventType::ReasoningReflexionRevisionCompleted
        | TraceEventType::ReasoningAdjudicationRecorded
        | TraceEventType::ReasoningConfidenceRecorded
        | TraceEventType::ReasoningProfileBlocked
        | TraceEventType::ReasoningProfileInterrupted
        | TraceEventType::ReasoningProfileEscalated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        TraceEventType::GovernanceDecisionRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Governance,
            "governance",
            "build_autopilot_decision",
        ),
        TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::GovernanceCompleted
        | TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected => SessionAuditAlgorithm::new(
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
        ),
        TraceEventType::RetryScheduled
        | TraceEventType::StageRetryScheduled
        | TraceEventType::Replanned
        | TraceEventType::StageReplanned
        | TraceEventType::StageFailed => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Recovery, "recovery", "decide_recovery")
        }
        TraceEventType::StageRouted => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "framework_adapter",
            "record_framework_adapter_run_stage_routing",
        ),
        TraceEventType::CheckpointCreated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "checkpoint",
            "prepare_checkpoint_for_mutation",
        ),
        TraceEventType::TerminalRecorded => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Run, "session_runtime", "finalize_task")
        }
        TraceEventType::TaskStarted
        | TraceEventType::FlowSelected
        | TraceEventType::StageTransitioned
        | TraceEventType::StepStarted
        | TraceEventType::StepCompleted => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Run, "session_runtime", "advance_task")
        }
    }
}

pub(super) fn trace_event_audit_outcome(event: &TraceEvent) -> SessionAuditOutcome {
    match event.event_type {
        TraceEventType::TaskStarted
        | TraceEventType::FlowSelected
        | TraceEventType::GoalPlanCreated
        | TraceEventType::FlowInferred
        | TraceEventType::ProjectScalePathProposed
        | TraceEventType::StageTransitioned
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceDecisionRecorded
        | TraceEventType::ReviewStarted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewerStarted
        | TraceEventType::ReviewStopSemanticsRecorded
        | TraceEventType::StepStarted
        | TraceEventType::DecisionCreated
        | TraceEventType::ReasoningProfileActivated
        | TraceEventType::ReasoningParticipantStarted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "activity started")
        }
        TraceEventType::DecisionDispatched
        | TraceEventType::StageRouted
        | TraceEventType::CheckpointCreated
        | TraceEventType::VotingDecisionRecorded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Recorded, "activity recorded")
        }
        TraceEventType::DecisionVerified | TraceEventType::GovernanceCompleted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Succeeded, "activity succeeded")
        }
        TraceEventType::StepCompleted
        | TraceEventType::ReviewerCompleted
        | TraceEventType::ReviewCouncilAssembled
        | TraceEventType::ReviewVoteResolved
        | TraceEventType::ReviewAdjudicated
        | TraceEventType::ReviewTerminalRecorded
        | TraceEventType::DecisionRecovered
        | TraceEventType::ReasoningParticipantCompleted
        | TraceEventType::ReasoningDisagreementRecorded
        | TraceEventType::ReasoningDebateRoundCompleted
        | TraceEventType::ReasoningReflexionRevisionCompleted
        | TraceEventType::ReasoningAdjudicationRecorded
        | TraceEventType::ReasoningConfidenceRecorded
        | TraceEventType::ProjectScaleStageTransitioned
        | TraceEventType::TerminalRecorded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "activity completed")
        }
        TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::ReasoningProfileInterrupted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Awaiting, "awaiting follow-up")
        }
        TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected
        | TraceEventType::ReasoningProfileBlocked => {
            let mut outcome = SessionAuditOutcome::new(
                SessionAuditOutcomeStatus::Blocked,
                trace_event_summary(event),
            );
            outcome.blocking = true;
            outcome
        }
        TraceEventType::DecisionFailed | TraceEventType::ReasoningProfileEscalated => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, trace_event_summary(event))
        }
        TraceEventType::RetryScheduled | TraceEventType::StageRetryScheduled => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Retried, trace_event_summary(event))
        }
        TraceEventType::Replanned | TraceEventType::StageReplanned => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Replanned,
            trace_event_summary(event),
        ),
        TraceEventType::StageFailed => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, trace_event_summary(event))
        }
    }
}

pub(super) fn trace_event_audit_message(event: &TraceEvent) -> String {
    let event_label = trace_event_type_text(event.event_type).replace('_', " ");
    let summary = trace_event_summary(event);
    if summary == event_label { event_label } else { format!("{event_label}: {summary}") }
}

fn trace_event_summary(event: &TraceEvent) -> String {
    payload_string(event.payload.get("summary"))
        .or_else(|| payload_string(event.payload.get("reason")))
        .or_else(|| payload_string(event.payload.get("message")))
        .or_else(|| payload_string(event.payload.get("headline")))
        .or_else(|| payload_string(event.payload.get("selection_summary")))
        .or_else(|| {
            payload_string(event.payload.get("stop_semantics"))
                .map(|stop_semantics| format!("stop semantics {stop_semantics}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("target")).map(|target| format!("target {target}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("stage_key"))
                .map(|stage_key| format!("stage {stage_key}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("reviewer_id"))
                .map(|reviewer_id| format!("reviewer {reviewer_id}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("participant_id"))
                .map(|participant_id| format!("participant {participant_id}"))
        })
        .unwrap_or_else(|| trace_event_type_text(event.event_type).replace('_', " "))
}

pub(super) fn trace_event_audit_actor(event: &TraceEvent) -> SessionAuditActor {
    match event.event_type {
        TraceEventType::ReviewerStarted | TraceEventType::ReviewerCompleted => {
            reviewer_audit_actor(&event.payload)
        }
        TraceEventType::ReviewAdjudicated => reviewer_audit_actor(&event.payload),
        TraceEventType::ReviewCouncilAssembled
        | TraceEventType::ReviewStopSemanticsRecorded
        | TraceEventType::ReviewVoteResolved => review_council_audit_actor(&event.payload),
        TraceEventType::ReasoningParticipantStarted
        | TraceEventType::ReasoningParticipantCompleted => {
            reasoning_participant_audit_actor(&event.payload)
        }
        TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceDecisionRecorded
        | TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::GovernanceCompleted
        | TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected => governance_audit_actor(&event.payload),
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => SessionAuditActor {
            kind: SessionAuditActorKind::Agent,
            id: "boundline-decision-loop".to_string(),
            display_name: Some("Boundline Decision Loop".to_string()),
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        },
        TraceEventType::ReviewStarted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewTerminalRecorded
        | TraceEventType::VotingDecisionRecorded => SessionAuditActor {
            kind: SessionAuditActorKind::Reviewer,
            id: "review-council".to_string(),
            display_name: Some("Review Council".to_string()),
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        },
        _ => SessionAuditActor::system("boundline"),
    }
}

fn reviewer_audit_actor(payload: &Value) -> SessionAuditActor {
    let reviewer_id = payload_string(payload.get("reviewer_id"))
        .unwrap_or_else(|| "unknown-reviewer".to_string());
    let reviewer_role = payload_string(payload.get("reviewer_role"));
    let reviewer_source = payload_string(payload.get("reviewer_source"));
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::Reviewer,
        id: reviewer_id.clone(),
        display_name: Some(reviewer_id),
        role: reviewer_role,
        runtime_kind: None,
        provider: None,
        route_slot: None,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };
    if let Some(source) = reviewer_source.as_deref() {
        apply_route_text_to_actor(&mut actor, source);
    }
    actor
}

fn review_council_audit_actor(payload: &Value) -> SessionAuditActor {
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::Reviewer,
        id: "review-council".to_string(),
        display_name: Some("Review Council".to_string()),
        role: None,
        runtime_kind: None,
        provider: None,
        route_slot: Some("review".to_string()),
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };

    let participants = payload
        .get("vote_resolution")
        .and_then(|value| value.get("participants"))
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerParticipation>>(value).ok())
        .unwrap_or_default();

    let completed_routes = participants
        .iter()
        .filter(|participant| participant.status == ReviewerParticipationStatus::Completed)
        .filter_map(|participant| participant.effective_route.as_deref())
        .map(str::trim)
        .filter(|route| !route.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    actor.participant_routes = completed_routes.clone();
    actor.mixed_routes = completed_routes.len() > 1;

    if let Some(route) = completed_routes.first() {
        apply_route_text_to_actor(&mut actor, route);
    }

    if actor.mixed_routes {
        actor.role = Some("multi-reviewer".to_string());
    }

    actor
}

fn reasoning_participant_audit_actor(payload: &Value) -> SessionAuditActor {
    let participant_id = payload_string(payload.get("participant_id"))
        .unwrap_or_else(|| "unknown-participant".to_string());
    let role = payload_string(payload.get("role"));
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::ReasoningParticipant,
        id: participant_id.clone(),
        display_name: Some(participant_id.clone()),
        role,
        runtime_kind: None,
        provider: None,
        route_slot: None,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };

    if let Some(record) = payload
        .get("reasoning_profile_record")
        .cloned()
        .and_then(|value| serde_json::from_value::<ProfileActivationRecord>(value).ok())
        && let Some(participant) = record
            .participants
            .iter()
            .find(|participant| participant.participant_id == participant_id)
    {
        actor.provider = participant.provider_family.clone();
        apply_route_text_to_actor(&mut actor, &participant.effective_route);
    }

    actor
}

pub(super) fn governance_audit_actor(payload: &Value) -> SessionAuditActor {
    let runtime = payload_string(payload.get("runtime"))
        .or_else(|| payload_string(payload.get("selected_runtime")))
        .unwrap_or_else(|| "governance".to_string());
    let route_slot = payload_string(payload.get("stage_key"))
        .as_deref()
        .and_then(governance_route_slot_for_stage_key)
        .map(str::to_string);
    SessionAuditActor {
        kind: SessionAuditActorKind::GovernanceRuntime,
        id: runtime.clone(),
        display_name: Some(runtime.clone()),
        role: payload_string(payload.get("stage_key")),
        runtime_kind: Some(runtime),
        provider: payload_string(payload.get("runtime"))
            .or_else(|| payload_string(payload.get("selected_runtime"))),
        route_slot,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    }
}

pub(super) fn governance_route_slot_for_stage_key(stage_key: &str) -> Option<&'static str> {
    let stage_key = stage_key.trim();
    if stage_key.is_empty() {
        return None;
    }

    if stage_key.starts_with("plan:") {
        return Some("planning");
    }

    Some("implementation")
}

pub(super) fn apply_route_text_to_actor(actor: &mut SessionAuditActor, route_text: &str) {
    if let Some((route_slot, runtime, model)) = parse_three_segment_route(route_text) {
        actor.route_slot = Some(route_slot);
        actor.runtime_kind = Some(runtime.clone());
        actor.provider.get_or_insert(runtime);
        actor.model_name = Some(model);
        return;
    }

    if let Some((runtime, model)) = route_text.split_once('/') {
        let runtime = runtime.trim();
        let model = model.trim();
        if !runtime.is_empty() {
            actor.runtime_kind = Some(runtime.to_string());
            actor.provider.get_or_insert(runtime.to_string());
        }
        if !model.is_empty() {
            actor.model_name = Some(model.to_string());
        }
    }
}

pub(super) fn parse_three_segment_route(route_text: &str) -> Option<(String, String, String)> {
    let mut parts = route_text.splitn(3, ':');
    let route_slot = parts.next()?.trim();
    let runtime = parts.next()?.trim();
    let model = parts.next()?.trim();
    if route_slot.is_empty() || runtime.is_empty() || model.is_empty() {
        return None;
    }
    Some((route_slot.to_string(), runtime.to_string(), model.to_string()))
}

pub(super) fn payload_string(value: Option<&Value>) -> Option<String> {
    let value = value?;
    match value {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => serde_json::to_string(value).ok(),
    }
}

pub(super) fn trace_event_type_text(event_type: TraceEventType) -> String {
    serde_json::to_value(event_type)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

pub(super) fn default_planning_system_context(mode: CanonMode) -> SystemContextBinding {
    if mode.requires_existing_context() {
        SystemContextBinding::Existing
    } else {
        SystemContextBinding::New
    }
}

pub(super) fn parse_planning_system_context(raw: &str) -> Option<SystemContextBinding> {
    match raw.trim() {
        SYSTEM_CONTEXT_NEW_TEXT => Some(SystemContextBinding::New),
        SYSTEM_CONTEXT_EXISTING_TEXT => Some(SystemContextBinding::Existing),
        _ => None,
    }
}

pub(super) fn read_upstream_artifact_capped(packet_dir: &Path, file_name: &str) -> Option<String> {
    let path = packet_dir.join(file_name);
    let content = fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() <= UPSTREAM_EVIDENCE_MAX_CHARS {
        return Some(trimmed.to_string());
    }
    Some(truncate_with_ellipsis_marker(trimmed, UPSTREAM_EVIDENCE_MAX_CHARS))
}

fn truncate_with_ellipsis_marker(text: &str, max_chars: usize) -> String {
    let Some((end_index, _)) = text.char_indices().nth(max_chars) else {
        return text.to_string();
    };
    let mut truncated = text[..end_index].to_string();
    truncated.push_str("\n\n[truncated]");
    truncated
}

pub(super) fn execution_governance_read_targets(
    native_context: &TaskContext,
    fallback_targets: &[String],
) -> Vec<String> {
    const CHANGED_FILES_EVIDENCE_KEY: &str = "changed_files";

    let mut targets = BTreeSet::new();
    for state_key in [LATEST_CHANGED_FILES_KEY, CHANGED_FILES_EVIDENCE_KEY] {
        if let Some(changed_files) = native_context.state.get(state_key).and_then(Value::as_array) {
            for changed_file in changed_files.iter().filter_map(Value::as_str) {
                if !changed_file.trim().is_empty() {
                    targets.insert(changed_file.to_string());
                }
            }
        }
    }

    if targets.is_empty() {
        for target in fallback_targets {
            if !target.trim().is_empty() {
                targets.insert(target.clone());
            }
        }
    }

    targets.into_iter().collect()
}

pub(super) fn missing_planning_governance_field(
    mode: CanonMode,
    field: &'static str,
) -> SessionRuntimeError {
    SessionRuntimeError::GoalPlan(format!(
        "planning governance for Canon mode {} requires field '{field}'",
        mode.as_str()
    ))
}

pub(super) fn session_status_for_task_status(status: TaskStatus) -> SessionStatus {
    match status {
        TaskStatus::Planned => SessionStatus::Planned,
        TaskStatus::Running => SessionStatus::Running,
        TaskStatus::Succeeded => SessionStatus::Succeeded,
        TaskStatus::Failed => SessionStatus::Failed,
        TaskStatus::Exhausted => SessionStatus::Exhausted,
        TaskStatus::Aborted => SessionStatus::Aborted,
    }
}
