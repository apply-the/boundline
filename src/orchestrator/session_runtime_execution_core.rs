use serde_json::{Value, json};

use crate::adapters::provider_runtime::route_is_available;
use crate::domain::governance::GovernanceLifecycleState;
use crate::domain::limits::TerminalCondition;
use crate::domain::session::{ActiveSessionRecord, SessionStatus};
use crate::domain::step::{ExecutionStatus, StepAttempt, StepResultSummary, StepStatus};
use crate::domain::task::{Task, TaskRunResponse, TaskStatus};
use crate::domain::trace::{ExecutionTrace, TraceEventType, current_timestamp_millis};
use crate::fixture::{
    FixtureRuntime, FixtureRuntimeError, build_fixture_runtime_for_flow,
    build_fixture_runtime_for_goal_plan,
};
use crate::orchestrator::guidance_runtime::execute_guardians_for_phase;
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::review_trace::{record_review_step_completed, record_review_step_started};
use crate::orchestrator::terminal::build_terminal_reason;

use super::{
    GovernanceStepDecision, SessionRuntime, SessionRuntimeError, checkpoint_event_payload,
    checkpoint_projection_from_context, planning_canon_mode_for_stage_key,
    session_status_for_task_status,
};

const CHECKPOINT_ID_PAYLOAD_KEY: &str = "checkpoint_id";
const GOAL_SATISFIED_STATE_KEY: &str = "goal_satisfied";

impl SessionRuntime {
    pub(super) fn build_runtime(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<FixtureRuntime, SessionRuntimeError> {
        if let Some(active_flow) = &session.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

        let goal = session
            .goal
            .as_deref()
            .or_else(|| session.active_task.as_ref().map(|task| task.goal.as_str()))
            .unwrap_or_default()
            .trim()
            .to_string();

        if goal.is_empty() {
            return Err(SessionRuntimeError::MissingGoal);
        }

        let goal_plan = session.goal_plan.as_ref();

        if let Some(goal_plan) = goal_plan
            && session.active_flow_policy.is_none()
            && session.active_workflow_progress().is_none()
        {
            return build_fixture_runtime_for_goal_plan(&self.workspace_ref, goal_plan)
                .map_err(SessionRuntimeError::FixtureRuntime);
        }

        match build_fixture_runtime_for_flow(&self.workspace_ref, session.active_flow.as_ref()) {
            Ok(runtime) => Ok(runtime),
            Err(error @ FixtureRuntimeError::MissingExecutionProfile(_)) => {
                if let Some(goal_plan) = goal_plan {
                    build_fixture_runtime_for_goal_plan(&self.workspace_ref, goal_plan)
                        .map_err(SessionRuntimeError::FixtureRuntime)
                } else {
                    Err(SessionRuntimeError::FixtureRuntime(error))
                }
            }
            Err(error) => Err(SessionRuntimeError::FixtureRuntime(error)),
        }
    }

    pub(super) fn execute_single_step(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &FixtureRuntime,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        let mut task = session.active_task.take().ok_or(SessionRuntimeError::MissingActiveTask)?;

        if task.status.is_terminal() {
            let response = self.existing_terminal_response(session, &task)?;
            session.latest_status = session_status_for_task_status(task.status);
            session.latest_terminal_reason = task.terminal_reason.clone();
            session.updated_at = current_timestamp_millis();
            session.active_task = Some(task);
            return Ok(Some(response));
        }

        if matches!(task.status, TaskStatus::Planned) {
            task.mark_running();
        }

        session.latest_status = SessionStatus::Running;
        session.latest_terminal_reason = None;

        let mut trace = self.load_or_create_trace(session, &task)?;
        if let Some(checkpoint_projection) = checkpoint_projection_from_context(&task.context)
            && !trace.events.iter().any(|event| {
                event.event_type == TraceEventType::CheckpointCreated
                    && event.payload.get(CHECKPOINT_ID_PAYLOAD_KEY).and_then(Value::as_str)
                        == Some(checkpoint_projection.checkpoint_id.as_str())
            })
        {
            trace.record_event(
                TraceEventType::CheckpointCreated,
                None,
                task.plan.revision,
                checkpoint_event_payload(&checkpoint_projection),
            );
        }
        let response = self.advance_task(session, &mut task, &mut trace, runtime)?;
        session.active_task = Some(task);

        Ok(response)
    }

    fn advance_task(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        runtime: &FixtureRuntime,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        if task.total_step_attempts >= task.limits.max_steps {
            let reason = build_terminal_reason(
                TerminalCondition::StepLimitExceeded,
                "maximum step attempts reached",
                Some(json!({
                    "attempts": task.total_step_attempts,
                    "max_steps": task.limits.max_steps,
                })),
            );
            return self.finalize_task(session, task, trace, reason).map(Some);
        }

        if task.plan.current_step().is_none() {
            let reason = build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                "no executable next step remains in the current plan",
                Some(json!({
                    "plan_revision": task.plan.revision,
                })),
            );
            return self.finalize_task(session, task, trace, reason).map(Some);
        }

        match self.ensure_stage_governance(session, task, trace, runtime)? {
            GovernanceStepDecision::Continue => {}
            GovernanceStepDecision::Halt => return Ok(None),
            GovernanceStepDecision::Terminal(response) => return Ok(Some(response)),
        }

        let step_index = task.plan.current_step_index;
        let step_snapshot = {
            let Some(step) = task.plan.current_step_mut() else {
                return Err(SessionRuntimeError::ExecutionInvariant(
                    "current step disappeared after scheduler validation".to_string(),
                ));
            };
            step.mark_running();
            step.clone()
        };
        task.total_step_attempts += 1;

        let started_at = current_timestamp_millis();
        let mut attempt =
            StepAttempt::new(step_snapshot.id.clone(), step_snapshot.input.clone(), started_at);
        trace.record_event(
            TraceEventType::StepStarted,
            Some(step_snapshot.id.clone()),
            task.plan.revision,
            json!({
                "attempt_number": step_snapshot.attempt_count,
                "input": step_snapshot.input,
                "step_kind": step_snapshot.kind,
            }),
        );
        record_review_step_started(
            trace,
            &step_snapshot.id,
            &step_snapshot.input,
            &task.context.state,
            task.plan.revision,
        );
        let trace_location = self.persist_trace(&session.session_id, trace)?;
        session.latest_trace_ref = Some(trace_location);

        let result = self.execute_step(runtime, &step_snapshot, &task.context);
        let result = self.normalize_result(result, &step_snapshot);
        attempt.complete(&result, current_timestamp_millis());
        task.context.push_history_ref(attempt.attempt_id.clone());

        match result.status {
            ExecutionStatus::Succeeded => {
                let Some(output) = result.output.clone() else {
                    return Err(SessionRuntimeError::ExecutionInvariant(format!(
                        "step {} reported success without output after normalization",
                        step_snapshot.id
                    )));
                };
                task.plan.steps[step_index].mark_succeeded(output.clone());
                task.context.apply_success_output(
                    &step_snapshot.id,
                    &output,
                    result.state_patch.as_ref(),
                );
                task.context
                    .set_last_result(StepResultSummary::from_step(&task.plan.steps[step_index]));
                let guardian_phase = Self::guardian_phase_for_step(session, step_index);
                let guardian_request = self.guardian_request_for_step(
                    session,
                    task,
                    &step_snapshot,
                    guardian_phase,
                    &result,
                );
                let guardian_outcome =
                    execute_guardians_for_phase(&self.workspace_ref, &guardian_request);
                if let Some(goal_plan) = session.goal_plan.as_mut() {
                    Self::merge_guardian_projection(
                        &mut goal_plan.guidance_guardian,
                        &guardian_outcome.projection,
                    );
                }
                let mut step_payload = json!({
                    "attempt_id": attempt.attempt_id,
                    "status": "succeeded",
                    "output": output,
                    "evidence": result.evidence,
                });
                Self::append_guardian_projection_payload(
                    &mut step_payload,
                    &guardian_outcome.projection,
                );
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step_snapshot.id.clone()),
                    task.plan.revision,
                    step_payload,
                );
                record_review_step_completed(
                    trace,
                    &step_snapshot.id,
                    &step_snapshot.input,
                    &result,
                    &task.context.state,
                    task.plan.revision,
                );

                let goal_satisfied = task
                    .context
                    .state
                    .get(GOAL_SATISFIED_STATE_KEY)
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    || task.plan.current_step_index + 1 >= task.plan.steps.len();

                if goal_satisfied {
                    task.plan.advance();
                    let reason = build_terminal_reason(
                        TerminalCondition::GoalSatisfied,
                        format!("goal satisfied after step {}", step_snapshot.id),
                        Some(json!({
                            "step_id": step_snapshot.id,
                        })),
                    );
                    return self.finalize_task(session, task, trace, reason).map(Some);
                }

                if let Some((from_stage, to_stage)) = self
                    .advance_session_flow(session, task, step_index)
                    .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
                {
                    trace.record_event(
                        TraceEventType::StageTransitioned,
                        Some(step_snapshot.id.clone()),
                        task.plan.revision,
                        json!({
                            "flow_name": from_stage.flow_name,
                            "from_stage_id": from_stage.stage_id,
                            "to_stage_id": to_stage.stage_id,
                            "from_stage_index": from_stage.stage_index,
                            "to_stage_index": to_stage.stage_index,
                        }),
                    );
                }

                task.plan.advance();
                let trace_location = self.persist_trace(&session.session_id, trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();
                Ok(None)
            }
            ExecutionStatus::Failed => {
                let Some(error) = result.error.clone() else {
                    return Err(SessionRuntimeError::ExecutionInvariant(format!(
                        "step {} reported failure without error details after normalization",
                        step_snapshot.id
                    )));
                };
                task.plan.steps[step_index].mark_failed(error.clone(), result.recoverability);
                task.context.apply_failure_error(&step_snapshot.id, &error);
                if let Some(state_patch) = result.state_patch.as_ref() {
                    task.context.apply_state_patch(state_patch);
                }
                task.context
                    .set_last_result(StepResultSummary::from_step(&task.plan.steps[step_index]));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step_snapshot.id.clone()),
                    task.plan.revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "failed",
                        "error": error,
                        "recoverability": result.recoverability,
                        "evidence": result.evidence,
                    }),
                );
                record_review_step_completed(
                    trace,
                    &step_snapshot.id,
                    &step_snapshot.input,
                    &result,
                    &task.context.state,
                    task.plan.revision,
                );

                match decide_recovery(task, &task.plan.steps[step_index], &result) {
                    RecoveryDecision::Continue => {
                        let trace_location = self.persist_trace(&session.session_id, trace)?;
                        session.latest_status = SessionStatus::Running;
                        session.latest_terminal_reason = None;
                        session.latest_trace_ref = Some(trace_location);
                        session.updated_at = current_timestamp_millis();
                        Ok(None)
                    }
                    RecoveryDecision::Retry { reason } => {
                        task.retry_count += 1;
                        let step = &mut task.plan.steps[step_index];
                        step.status = StepStatus::Pending;
                        let flow_payload =
                            self.flow_payload_for_step(&step_snapshot).map_err(|error| {
                                SessionRuntimeError::InvalidFlowState(error.to_string())
                            })?;
                        let mut payload = json!({
                            "reason": reason,
                            "retry_count": task.retry_count,
                        });
                        if let Some(flow_payload) = flow_payload.clone()
                            && let Some(object) = payload.as_object_mut()
                        {
                            object.insert("flow".to_string(), flow_payload);
                        }
                        trace.record_event(
                            if flow_payload.is_some() {
                                TraceEventType::StageRetryScheduled
                            } else {
                                TraceEventType::RetryScheduled
                            },
                            Some(step_snapshot.id.clone()),
                            task.plan.revision,
                            payload,
                        );
                        let trace_location = self.persist_trace(&session.session_id, trace)?;
                        session.latest_status = SessionStatus::Running;
                        session.latest_terminal_reason = None;
                        session.latest_trace_ref = Some(trace_location);
                        session.updated_at = current_timestamp_millis();
                        Ok(None)
                    }
                    RecoveryDecision::Replan { reason } => {
                        let replacements = match runtime.planner.replan(
                            task,
                            &task.plan.steps[step_index],
                            &result,
                        ) {
                            Ok(replacements) => replacements,
                            Err(error) => {
                                let reason = build_terminal_reason(
                                    TerminalCondition::TaskNotCredible,
                                    "planner could not produce a credible replacement plan",
                                    Some(json!({"error": error.to_string()})),
                                );
                                return self.finalize_task(session, task, trace, reason).map(Some);
                            }
                        };

                        task.replan_count += 1;
                        let revision = match task.plan.replace_remaining_steps(replacements) {
                            Ok(revision) => revision,
                            Err(error) => {
                                let reason = build_terminal_reason(
                                    TerminalCondition::TaskNotCredible,
                                    "replacement plan did not provide a credible next step",
                                    Some(json!({"error": error.to_string()})),
                                );
                                return self.finalize_task(session, task, trace, reason).map(Some);
                            }
                        };

                        let flow_payload =
                            self.flow_payload_for_step(&step_snapshot).map_err(|error| {
                                SessionRuntimeError::InvalidFlowState(error.to_string())
                            })?;
                        let mut payload = json!({
                            "reason": reason,
                            "replan_count": task.replan_count,
                            "from_revision": revision.from_revision,
                            "to_revision": revision.to_revision,
                            "replaced_step_ids": revision.replaced_step_ids,
                            "added_step_ids": revision.added_step_ids,
                        });
                        if let Some(flow_payload) = flow_payload.clone()
                            && let Some(object) = payload.as_object_mut()
                        {
                            object.insert("flow".to_string(), flow_payload);
                        }
                        trace.record_event(
                            if flow_payload.is_some() {
                                TraceEventType::StageReplanned
                            } else {
                                TraceEventType::Replanned
                            },
                            Some(step_snapshot.id.clone()),
                            revision.to_revision,
                            payload,
                        );
                        let trace_location = self.persist_trace(&session.session_id, trace)?;
                        session.latest_status = SessionStatus::Running;
                        session.latest_terminal_reason = None;
                        session.latest_trace_ref = Some(trace_location);
                        session.updated_at = current_timestamp_millis();
                        Ok(None)
                    }
                    RecoveryDecision::Terminate(reason) => {
                        self.finalize_task(session, task, trace, reason).map(Some)
                    }
                }
            }
        }
    }

    pub(super) fn attempt_auto_clear_provider_block(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> bool {
        let dominated_by_provider = |reason: &str| -> bool {
            let lowered = reason.to_ascii_lowercase();
            lowered.contains("provider")
                || lowered.contains("reviewer")
                || lowered.contains("token")
                || lowered.contains("credential")
                || lowered.contains("request failed")
        };

        let blocked_stage_key = {
            let Some(lifecycle) = session.governance_lifecycle.as_ref() else {
                return false;
            };
            lifecycle
                .stage_records
                .iter()
                .rev()
                .find(|record| {
                    planning_canon_mode_for_stage_key(&record.stage_key).is_some()
                        && matches!(
                            record.lifecycle_state,
                            GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed
                        )
                        && record.blocked_reason.as_deref().is_some_and(dominated_by_provider)
                })
                .map(|record| record.stage_key.clone())
        };

        let Some(stage_key) = blocked_stage_key else {
            return false;
        };

        let routing = self.planning_council_effective_routing();
        if !route_is_available(&routing.planning.route) {
            return false;
        }

        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return false;
        };
        lifecycle.stage_records.retain(|record| record.stage_key != stage_key);
        lifecycle.terminal_reason = None;
        true
    }

    fn existing_terminal_response(
        &self,
        session: &ActiveSessionRecord,
        task: &Task,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let trace_location =
            session.latest_trace_ref.clone().ok_or(SessionRuntimeError::MissingTraceReference)?;
        let terminal_reason =
            task.terminal_reason.clone().ok_or(SessionRuntimeError::MissingTerminalReason)?;

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: task.status,
            terminal_reason,
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use crate::adapters::config_store::FileConfigStore;
    use crate::domain::configuration::{ConfigFile, ModelRoute, RuntimeKind};
    use crate::domain::governance::{
        ApprovalState, CanonMode, CanonModeSelectionPreference, GovernanceLifecycleState,
        GovernanceRuntimeKind, GovernedSessionLifecycle, GovernedStageRecord,
    };
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::task::TerminalReason;

    use super::SessionRuntime;

    const AVAILABLE_PLANNING_MODEL: &str = "ollama/test-model";
    const BLOCKED_TERMINAL_REASON: &str = "governance blocked";
    const GENERIC_UNAVAILABLE_REASON: &str =
        "planning governance requires Canon, but command 'canon' is unavailable";
    const NON_PLANNING_STAGE_KEY: &str = "run:implementation";
    const OTHER_PLANNING_STAGE_KEY: &str = "plan:architecture";
    const PLANNING_STAGE_KEY: &str = "plan:requirements";
    const PROVIDER_BLOCKED_REASON: &str =
        "provider request failed because token credentials are missing";
    const STAGE_ATTEMPT_ID: &str = "attempt-1";
    const UNAVAILABLE_PLANNING_MODEL: &str = "unsupported/test-model";
    const UPDATED_AT: u64 = 17;

    #[test]
    fn auto_clear_provider_block_clears_provider_dominated_planning_stage_when_route_available()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-execution-core-auto-clear")?;
        save_planning_route(workspace.as_path(), AVAILABLE_PLANNING_MODEL)?;

        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(
            workspace.as_path(),
            vec![
                sample_stage_record(
                    OTHER_PLANNING_STAGE_KEY,
                    GovernanceLifecycleState::Completed,
                    None,
                ),
                sample_stage_record(
                    PLANNING_STAGE_KEY,
                    GovernanceLifecycleState::Blocked,
                    Some(PROVIDER_BLOCKED_REASON.to_string()),
                ),
            ],
        );

        assert!(runtime.attempt_auto_clear_provider_block(&mut session));

        let lifecycle = session.governance_lifecycle.as_ref().ok_or("missing lifecycle")?;
        assert_eq!(lifecycle.terminal_reason, None);
        assert_eq!(lifecycle.stage_records.len(), 1);
        assert_eq!(lifecycle.stage_records[0].stage_key, OTHER_PLANNING_STAGE_KEY);

        Ok(())
    }

    #[test]
    fn auto_clear_provider_block_preserves_unmatched_or_unavailable_cases()
    -> Result<(), Box<dyn Error>> {
        let available_workspace = temp_workspace("boundline-execution-core-generic-block")?;
        save_planning_route(available_workspace.as_path(), AVAILABLE_PLANNING_MODEL)?;
        let available_runtime = SessionRuntime::for_workspace(available_workspace.as_path());

        let mut generic_unavailable_session = sample_session(
            available_workspace.as_path(),
            vec![sample_stage_record(
                PLANNING_STAGE_KEY,
                GovernanceLifecycleState::Blocked,
                Some(GENERIC_UNAVAILABLE_REASON.to_string()),
            )],
        );
        assert!(
            !available_runtime.attempt_auto_clear_provider_block(&mut generic_unavailable_session)
        );
        assert_eq!(
            generic_unavailable_session
                .governance_lifecycle
                .as_ref()
                .and_then(|lifecycle| lifecycle.terminal_reason.as_deref()),
            Some(BLOCKED_TERMINAL_REASON)
        );

        let unavailable_route_workspace =
            temp_workspace("boundline-execution-core-unavailable-route")?;
        save_planning_route(unavailable_route_workspace.as_path(), UNAVAILABLE_PLANNING_MODEL)?;
        let unavailable_runtime =
            SessionRuntime::for_workspace(unavailable_route_workspace.as_path());

        let mut unavailable_route_session = sample_session(
            unavailable_route_workspace.as_path(),
            vec![sample_stage_record(
                PLANNING_STAGE_KEY,
                GovernanceLifecycleState::Failed,
                Some(PROVIDER_BLOCKED_REASON.to_string()),
            )],
        );
        assert!(
            !unavailable_runtime.attempt_auto_clear_provider_block(&mut unavailable_route_session)
        );
        assert_eq!(
            unavailable_route_session
                .governance_lifecycle
                .as_ref()
                .map(|lifecycle| lifecycle.stage_records.len()),
            Some(1)
        );

        let mut non_planning_stage_session = sample_session(
            available_workspace.as_path(),
            vec![sample_stage_record(
                NON_PLANNING_STAGE_KEY,
                GovernanceLifecycleState::Blocked,
                Some(PROVIDER_BLOCKED_REASON.to_string()),
            )],
        );
        assert!(
            !available_runtime.attempt_auto_clear_provider_block(&mut non_planning_stage_session)
        );
        assert_eq!(
            non_planning_stage_session
                .governance_lifecycle
                .as_ref()
                .map(|lifecycle| lifecycle.stage_records.len()),
            Some(1)
        );

        Ok(())
    }

    fn sample_session(
        workspace: &Path,
        stage_records: Vec<GovernedStageRecord>,
    ) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: format!("session-{}", Uuid::new_v4()),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Clear provider-dominated blocks".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Blocked,
            latest_terminal_reason: Some(TerminalReason::new(
                TerminalCondition::NoCredibleNextStep,
                BLOCKED_TERMINAL_REASON,
                None,
            )),
            latest_trace_ref: None,
            created_at: UPDATED_AT,
            updated_at: UPDATED_AT,
            governance_lifecycle: Some(GovernedSessionLifecycle {
                governance_runtime: GovernanceRuntimeKind::Canon,
                explicit_opt_out: false,
                mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
                selected_mode: Some(CanonMode::Requirements),
                selected_mode_sequence: vec![CanonMode::Requirements],
                latest_reasoning_profile: None,
                current_stage_index: 0,
                stage_records,
                accumulated_context: Vec::new(),
                terminal_reason: Some(BLOCKED_TERMINAL_REASON.to_string()),
                planning_input_fingerprint: None,
            }),
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        }
    }

    fn sample_stage_record(
        stage_key: &str,
        lifecycle_state: GovernanceLifecycleState,
        blocked_reason: Option<String>,
    ) -> GovernedStageRecord {
        GovernedStageRecord {
            stage_key: stage_key.to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state,
            required: true,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: STAGE_ATTEMPT_ID.to_string(),
            previous_governance_attempt_id: None,
            packet_ref: None,
            decision_ref: None,
            stage_council: None,
            blocked_reason,
        }
    }

    fn save_planning_route(workspace: &Path, model: &str) -> Result<(), Box<dyn Error>> {
        FileConfigStore::for_workspace(workspace).save_local(&ConfigFile {
            routing: crate::domain::configuration::RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: model.to_string(),
                }),
                ..crate::domain::configuration::RoutingConfig::default()
            },
            ..ConfigFile::default()
        })?;
        Ok(())
    }

    fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
        TestWorkspace::new(prefix)
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
}
