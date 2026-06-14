use std::process::{Command, Output};

use serde_json::{Map, json};

use crate::domain::cluster::ClusteredExecutionKind;
use crate::domain::completion_verification::{
    CompletionClaim, CompletionRequiredAction, CompletionVerificationFinding,
    CompletionVerificationFindingKind, CompletionVerificationFindingSeverity,
    CompletionVerificationProjection, CompletionVerificationScope, CompletionVerificationState,
    WorkspaceContentFingerprint, capture_workspace_fingerprint, compare_workspace_fingerprints,
    proof_rules_for_validation_command, stale_proof_projection,
};
use crate::domain::decision::Decision;
use crate::domain::goal_plan::GoalPlan;
use crate::domain::limits::TerminalCondition;
use crate::domain::session::{ActiveSessionRecord, SessionStatus};
use crate::domain::task::{Task, TaskRunResponse, TaskStatus};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEvent, TraceEventType, current_timestamp_millis};
use crate::fixture::{FixtureRuntime, build_fixture_plan_for_goal, build_task_request};
use crate::orchestrator::guidance_runtime::execute_guardians_for_phase;
use crate::orchestrator::review_trace::record_reasoning_profile_events;
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};

use super::{
    LATEST_CHANGED_FILES_KEY, LATEST_VALIDATION_STATUS_KEY, NativePersistenceInput, SessionRuntime,
    SessionRuntimeError, VALIDATION_STATUS_FAILED, VALIDATION_STATUS_PASSED,
    apply_checkpoint_projection_to_context, checkpoint_event_payload,
    session_status_for_task_status,
};

struct NativeCompletionGateInput {
    decisions: Vec<Decision>,
    trace: ExecutionTrace,
    final_context: TaskContext,
    task_id: String,
    plan_revision: usize,
    projected_task: Option<Task>,
    completion_validation_command: Option<String>,
}

struct NativeProofResumeRecord {
    claim: CompletionClaim,
    evidence_ref: String,
    proof_ref: String,
}

pub(super) struct NativeProofExecutionRequest {
    pub(super) claim: CompletionClaim,
    pub(super) documentation_relevant: bool,
    pub(super) command_line: String,
    pub(super) command_ref: String,
}

impl SessionRuntime {
    pub(super) fn execute_completion_proof(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        request: NativeProofExecutionRequest,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let NativeProofExecutionRequest {
            claim,
            documentation_relevant,
            command_line,
            command_ref,
        } = request;
        let pre_fingerprint =
            self.capture_completion_fingerprint(task, documentation_relevant, true)?;
        let output = Command::new("/bin/sh")
            .arg("-lc")
            .arg(&command_line)
            .current_dir(&self.workspace_ref)
            .output()
            .map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to execute completion proof command `{command_line}`: {error}"
                ))
            })?;
        let post_fingerprint =
            self.capture_completion_fingerprint(task, documentation_relevant, false)?;
        let proof_ref = format!("proof-{}-{}", current_timestamp_millis(), task.id);
        let evidence_ref = format!("evidence-{}-{}", current_timestamp_millis(), task.id);
        trace.set_completion_verification_refs(
            vec![proof_ref.clone()],
            vec![pre_fingerprint.fingerprint_id.clone(), post_fingerprint.fingerprint_id.clone()],
            vec![evidence_ref.clone()],
        );

        let proof_record = NativeProofResumeRecord { claim, evidence_ref, proof_ref };

        if output.status.success() {
            self.complete_resumed_proof(session, task, trace, proof_record)
        } else {
            let _ = command_ref;
            self.block_failed_proof(session, task, trace, proof_record, output)
        }
    }

    pub fn refresh_completion_verification_state(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        let Some(task) = session.active_task.as_mut() else {
            return Ok(false);
        };
        let Some(projection) = task
            .context
            .completion_verification_projection()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
        else {
            return Ok(false);
        };
        if projection.completion_verification_state != CompletionVerificationState::Ready {
            return Ok(false);
        }

        let Some(selection) = task
            .context
            .completion_proof_selection()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
        else {
            return Ok(false);
        };
        let Some(claim) = task
            .context
            .completion_claim()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
        else {
            return Ok(false);
        };
        let Some(passing_fingerprint) = task
            .context
            .completion_proof_post_fingerprint()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
        else {
            return Ok(false);
        };

        let current_fingerprint =
            capture_workspace_fingerprint(&self.workspace_ref, selection.documentation_relevant)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        let diff = compare_workspace_fingerprints(&passing_fingerprint, &current_fingerprint);
        if diff.changed_paths.is_empty() {
            return Ok(false);
        }

        let stale_projection =
            stale_proof_projection(&claim, &projection.completion_evidence_refs, &diff);
        task.context
            .set_completion_verification_projection(&stale_projection)
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        task.status = TaskStatus::Running;
        task.terminal_reason = None;
        session.latest_status = SessionStatus::Blocked;
        session.latest_terminal_reason = None;
        session.updated_at = current_timestamp_millis();

        Ok(true)
    }

    pub(super) fn persist_native_result(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: GoalPlan,
        decisions: Vec<Decision>,
        mut trace: ExecutionTrace,
        input: NativePersistenceInput,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let mut terminal_reason = input.terminal_reason;
        let mut terminal_status = task_status_for_condition(terminal_reason.condition);
        let mut goal_plan = goal_plan;
        let cluster_story = goal_plan
            .cluster_session_projection
            .as_ref()
            .map(|projection| self.build_cluster_delivery_story(projection, terminal_status));
        goal_plan.cluster_delivery_story = cluster_story.clone();
        if let Some(cluster_story) = cluster_story.as_ref()
            && cluster_story.execution_condition.kind == ClusteredExecutionKind::Failed
            && terminal_status == TaskStatus::Succeeded
        {
            terminal_reason = build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                cluster_story.execution_condition.summary.clone(),
                Some(json!({ "cluster_delivery_story": cluster_story })),
            );
            terminal_status = TaskStatus::Failed;
        }
        if !trace.events.iter().any(|event| event.event_type.is_reasoning_event())
            && let Some(reasoning_profile) = session
                .governance_lifecycle
                .as_ref()
                .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
        {
            record_reasoning_profile_events(
                &mut trace,
                "terminal",
                goal_plan.proposal_revision,
                reasoning_profile,
            );
        }
        if input.record_terminal_event {
            trace.record_event(
                TraceEventType::TerminalRecorded,
                None,
                goal_plan.proposal_revision,
                json!({
                    "cluster_delivery_story": cluster_story,
                    "terminal_status": terminal_status,
                    "terminal_reason": terminal_reason.clone(),
                }),
            );
        } else if let Some(cluster_story) = cluster_story.clone()
            && let Some(event) = trace
                .events
                .iter_mut()
                .rev()
                .find(|event| event.event_type == TraceEventType::TerminalRecorded)
            && let Some(payload) = event.payload.as_object_mut()
        {
            payload.insert("cluster_delivery_story".to_string(), json!(cluster_story));
            payload.insert("terminal_status".to_string(), json!(terminal_status));
            payload.insert("terminal_reason".to_string(), json!(terminal_reason.clone()));
        }
        if let Some(guardian_request) =
            self.native_guardian_request(session, &goal_plan, decisions.as_slice())
        {
            let guardian_outcome =
                execute_guardians_for_phase(&self.workspace_ref, &guardian_request);
            Self::merge_guardian_projection(
                &mut goal_plan.guidance_guardian,
                &guardian_outcome.projection,
            );
            if let Some(event) = trace
                .events
                .iter_mut()
                .rev()
                .find(|event| event.event_type == TraceEventType::TerminalRecorded)
            {
                Self::append_guardian_projection_payload(
                    &mut event.payload,
                    &guardian_outcome.projection,
                );
            }
        }
        if let Some(checkpoint_projection) = input.checkpoint_projection.as_ref() {
            trace.record_event(
                TraceEventType::CheckpointCreated,
                None,
                goal_plan.proposal_revision,
                checkpoint_event_payload(checkpoint_projection),
            );
        }
        let mut final_context = self.build_native_task_context(
            session,
            input.limits,
            &goal_plan,
            &input.native_context,
        )?;
        if let Some(checkpoint_projection) = input.checkpoint_projection.as_ref() {
            apply_checkpoint_projection_to_context(&mut final_context, checkpoint_projection);
        }
        let task_id = goal_plan.plan_id.clone();
        let plan_revision = goal_plan.proposal_revision;

        if terminal_status == TaskStatus::Succeeded {
            return self.block_native_completion_closeout(
                session,
                &goal_plan,
                NativeCompletionGateInput {
                    decisions,
                    trace,
                    final_context,
                    task_id,
                    plan_revision,
                    projected_task: input.projected_task,
                    completion_validation_command: input.completion_validation_command,
                },
            );
        }

        trace.finalize(terminal_status, terminal_reason.clone());
        let trace_location = self.persist_trace(&session.session_id, &mut trace)?;
        let projected_task = match input.projected_task {
            Some(task) => Some(task),
            None if cluster_story.is_some() => Some(self.synthesize_native_persisted_task(
                session,
                &goal_plan,
                &final_context,
                terminal_status,
                &terminal_reason,
            )?),
            None => None,
        };

        session.active_task = projected_task;
        session.goal_plan = Some(goal_plan);
        session.decisions = decisions;
        session.latest_status =
            if session.goal_plan.as_ref().and_then(GoalPlan::delegation_continuity).is_some() {
                SessionStatus::Planned
            } else {
                session_status_for_task_status(terminal_status)
            };
        session.latest_terminal_reason = Some(terminal_reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id,
            terminal_status,
            terminal_reason,
            final_context,
            plan_revision,
            trace_location,
        })
    }

    pub(super) fn maybe_resume_completion_verification(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        let Some(mut task) = session.active_task.take() else {
            return Ok(None);
        };

        let projection = task
            .context
            .completion_verification_projection()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        let selection = task
            .context
            .completion_proof_selection()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        let claim = task
            .context
            .completion_claim()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;

        let should_resume = projection.as_ref().is_some_and(|projection| {
            matches!(
                projection.completion_verification_state,
                CompletionVerificationState::ProofRequired
                    | CompletionVerificationState::Failed
                    | CompletionVerificationState::Blocked
            )
        }) && selection.is_some()
            && claim.is_some();
        if !should_resume {
            session.active_task = Some(task);
            return Ok(None);
        }

        let selection = match selection {
            Some(selection) => selection,
            None => {
                session.active_task = Some(task);
                return Ok(None);
            }
        };
        let claim = match claim {
            Some(claim) => claim,
            None => {
                session.active_task = Some(task);
                return Ok(None);
            }
        };
        let task_snapshot = task.clone();
        let mut trace = self.load_or_create_trace(session, &task_snapshot)?;
        let response = self.execute_completion_proof(
            session,
            &mut task,
            &mut trace,
            NativeProofExecutionRequest {
                claim,
                documentation_relevant: selection.documentation_relevant,
                command_line: selection.command_line,
                command_ref: selection.command_ref,
            },
        )?;

        session.active_task = Some(task);
        Ok(Some(response))
    }

    fn block_native_completion_closeout(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &GoalPlan,
        mut gate: NativeCompletionGateInput,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let existing_projection = gate
            .final_context
            .completion_verification_projection()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        if let Some(existing_projection) = existing_projection.as_ref()
            && existing_projection.scope != CompletionVerificationScope::Task
            && existing_projection.completion_verification_state
                != CompletionVerificationState::Ready
        {
            let trace_location = self.persist_trace(&session.session_id, &mut gate.trace)?;
            session.latest_status = SessionStatus::Blocked;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = Some(trace_location.clone());
            session.updated_at = current_timestamp_millis();

            return Ok(TaskRunResponse {
                task_id: gate.task_id.clone(),
                terminal_status: TaskStatus::Running,
                terminal_reason: build_terminal_reason(
                    TerminalCondition::GoalSatisfied,
                    "completion verification blocked closeout because a parent scope remains unresolved",
                    None,
                ),
                final_context: gate.final_context.clone(),
                plan_revision: gate.plan_revision,
                trace_location,
            });
        }
        // Closeout must prove the completion action rather than the initial
        // analysis step, so use the terminal planned task outcome.
        let expected_outcome = goal_plan
            .tasks
            .last()
            .and_then(|planned_task| planned_task.expected_outcome.as_deref());
        let changed_files = gate
            .final_context
            .state
            .get(LATEST_CHANGED_FILES_KEY)
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let proof_rules = gate
            .completion_validation_command
            .as_deref()
            .map(|command_line| {
                proof_rules_for_validation_command("workspace_validation_command", command_line)
            })
            .unwrap_or_default();
        match super::finalization::evaluate_completion_closeout(
            super::finalization::CompletionCloseoutEvaluationInput {
                task_id: &gate.task_id,
                goal: &goal_plan.goal_text,
                expected_outcome,
                changed_files: &changed_files,
                existing_claim: None,
                validation_command: gate.completion_validation_command.as_deref(),
            },
            &proof_rules,
        )? {
            super::finalization::CompletionCloseoutEvaluation::SkipVerification => {
                let trace_location = self.persist_trace(&session.session_id, &mut gate.trace)?;
                session.goal_plan = Some(goal_plan.clone());
                session.decisions = gate.decisions;
                session.latest_status = SessionStatus::Succeeded;
                session.updated_at = current_timestamp_millis();
                session.latest_trace_ref = Some(trace_location.clone());
                let terminal_reason = build_terminal_reason(
                    TerminalCondition::GoalSatisfied,
                    "completion verification skipped: no proof rules or inferred claim available",
                    None,
                );
                session.latest_terminal_reason = Some(terminal_reason.clone());
                Ok(TaskRunResponse {
                    task_id: gate.task_id,
                    terminal_status: TaskStatus::Succeeded,
                    terminal_reason,
                    final_context: gate.final_context,
                    plan_revision: gate.plan_revision,
                    trace_location,
                })
            }
            super::finalization::CompletionCloseoutEvaluation::ClaimConflict(
                conflict_projection,
            ) => {
                gate.final_context
                    .set_completion_claim(conflict_projection.claim.as_ref().ok_or_else(|| {
                        SessionRuntimeError::TaskContext(
                            "claim-conflict projection requires an active claim".to_string(),
                        )
                    })?)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                gate.final_context
                    .set_completion_verification_projection(&conflict_projection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                self.persist_native_blocked_completion_closeout(
                    session,
                    goal_plan,
                    &mut gate,
                    "completion verification blocked closeout pending claim conflict resolution",
                )
            }
            super::finalization::CompletionCloseoutEvaluation::ConfirmationRequired {
                claim,
                proof_selection,
                projection,
            } => {
                gate.final_context
                    .set_completion_claim(&claim)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                gate.final_context
                    .set_completion_proof_selection(&proof_selection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                gate.final_context
                    .set_completion_verification_projection(&projection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                self.persist_native_blocked_completion_closeout(
                    session,
                    goal_plan,
                    &mut gate,
                    "completion verification blocked closeout pending claim confirmation",
                )
            }
            super::finalization::CompletionCloseoutEvaluation::ExecuteProof {
                claim,
                proof_selection,
            } => {
                gate.final_context
                    .set_completion_claim(&claim)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                gate.final_context
                    .set_completion_proof_selection(&proof_selection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                let mut task = match gate.projected_task {
                    Some(mut task) => {
                        task.context = gate.final_context.clone();
                        task.status = TaskStatus::Running;
                        task.terminal_reason = None;
                        task
                    }
                    None => self.synthesize_native_incomplete_task(
                        session,
                        goal_plan,
                        &gate.final_context,
                    )?,
                };
                task.context = gate.final_context.clone();

                let response = self.execute_completion_proof(
                    session,
                    &mut task,
                    &mut gate.trace,
                    NativeProofExecutionRequest {
                        claim,
                        documentation_relevant: proof_selection.documentation_relevant,
                        command_line: proof_selection.command_line,
                        command_ref: proof_selection.command_ref,
                    },
                )?;
                session.active_task = Some(task);
                session.goal_plan = Some(goal_plan.clone());
                session.decisions = gate.decisions;
                Ok(response)
            }
            super::finalization::CompletionCloseoutEvaluation::MissingProof {
                claim,
                projection,
            } => {
                gate.final_context
                    .set_completion_claim(&claim)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                gate.final_context
                    .set_completion_verification_projection(&projection)
                    .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
                self.persist_native_blocked_completion_closeout(
                    session,
                    goal_plan,
                    &mut gate,
                    "completion verification blocked closeout pending proof",
                )
            }
        }
    }

    fn persist_native_blocked_completion_closeout(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &GoalPlan,
        gate: &mut NativeCompletionGateInput,
        message: &str,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let task = self.materialize_native_closeout_task(session, goal_plan, gate)?;
        let trace_location = self.persist_trace(&session.session_id, &mut gate.trace)?;
        session.active_task = Some(task);
        session.goal_plan = Some(goal_plan.clone());
        session.decisions = std::mem::take(&mut gate.decisions);
        session.latest_status = SessionStatus::Blocked;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: gate.task_id.clone(),
            terminal_status: TaskStatus::Running,
            terminal_reason: build_terminal_reason(TerminalCondition::GoalSatisfied, message, None),
            final_context: gate.final_context.clone(),
            plan_revision: gate.plan_revision,
            trace_location,
        })
    }

    fn materialize_native_closeout_task(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
        gate: &NativeCompletionGateInput,
    ) -> Result<Task, SessionRuntimeError> {
        let mut task = match gate.projected_task.clone() {
            Some(task) => task,
            None => {
                self.synthesize_native_incomplete_task(session, goal_plan, &gate.final_context)?
            }
        };
        task.context = gate.final_context.clone();
        task.status = TaskStatus::Running;
        task.terminal_reason = None;
        Ok(task)
    }

    fn synthesize_native_incomplete_task(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
        final_context: &TaskContext,
    ) -> Result<Task, SessionRuntimeError> {
        let request = build_task_request(
            &self.workspace_ref,
            &goal_plan.goal_text,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan = build_fixture_plan_for_goal(
            &self.workspace_ref,
            session.active_flow.as_ref(),
            &goal_plan.goal_text,
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let mut task = Task::new(goal_plan.plan_id.clone(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;
        task.context = final_context.clone();
        task.mark_running();
        task.terminal_reason = None;
        Ok(task)
    }

    fn capture_completion_fingerprint(
        &self,
        task: &mut Task,
        documentation_relevant: bool,
        is_pre: bool,
    ) -> Result<WorkspaceContentFingerprint, SessionRuntimeError> {
        let fingerprint =
            capture_workspace_fingerprint(&self.workspace_ref, documentation_relevant)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        if is_pre {
            task.context
                .set_completion_proof_pre_fingerprint(&fingerprint)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        } else {
            task.set_completion_proof_post_fingerprint(&fingerprint)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        Ok(fingerprint)
    }

    fn complete_resumed_proof(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        proof_record: NativeProofResumeRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let projection = CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::Ready,
            scope: CompletionVerificationScope::Task,
            claim: Some(proof_record.claim),
            completion_blocked_claims: Vec::new(),
            completion_evidence_refs: vec![proof_record.evidence_ref.clone()],
            completion_verification_findings: Vec::new(),
            child_summary: None,
        };
        task.set_completion_verification_projection(&projection)
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;

        let terminal_reason = build_terminal_reason(
            TerminalCondition::GoalSatisfied,
            format!("goal satisfied after fresh proof {}", proof_record.proof_ref),
            None,
        );
        task.apply_terminal(TaskStatus::Succeeded, terminal_reason.clone());
        trace.record_event(
            TraceEventType::TerminalRecorded,
            None,
            task.plan.revision,
            json!({
                "completion_verification_state": "ready",
                "completion_evidence_refs": [proof_record.evidence_ref],
                "terminal_status": TaskStatus::Succeeded,
                "terminal_reason": terminal_reason.clone(),
            }),
        );
        trace.finalize(TaskStatus::Succeeded, terminal_reason.clone());
        let trace_location = self.persist_trace(&session.session_id, trace)?;
        session.latest_status = SessionStatus::Succeeded;
        session.latest_terminal_reason = Some(terminal_reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason,
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    fn block_failed_proof(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        proof_record: NativeProofResumeRecord,
        output: Output,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let message = summarize_proof_failure(
            &task
                .context
                .completion_proof_selection()
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
                .map(|selection| selection.command_line)
                .unwrap_or_default(),
            &output,
        );
        let projection = CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::Failed,
            scope: CompletionVerificationScope::Task,
            claim: Some(proof_record.claim.clone()),
            completion_blocked_claims: vec![proof_record.claim.kind],
            completion_evidence_refs: vec![proof_record.evidence_ref.clone()],
            completion_verification_findings: vec![CompletionVerificationFinding {
                kind: CompletionVerificationFindingKind::FailedProof,
                severity: CompletionVerificationFindingSeverity::Blocking,
                message,
                proof_ref: Some(proof_record.proof_ref),
                task_id: Some(task.id.clone()),
                changed_paths: Vec::new(),
                required_action: CompletionRequiredAction::RerunProof,
            }],
            child_summary: None,
        };
        task.set_completion_verification_projection(&projection)
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        task.status = TaskStatus::Running;
        task.terminal_reason = None;

        trace.record_event(
            TraceEventType::DecisionFailed,
            None,
            task.plan.revision,
            json!({
                "completion_verification_state": "failed",
                "completion_evidence_refs": [proof_record.evidence_ref],
            }),
        );
        let trace_location = self.persist_trace(&session.session_id, trace)?;
        session.latest_status = SessionStatus::Blocked;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: TaskStatus::Running,
            terminal_reason: build_terminal_reason(
                TerminalCondition::GoalSatisfied,
                "completion verification proof failed",
                None,
            ),
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    pub(super) fn build_native_task_context(
        &self,
        session: &ActiveSessionRecord,
        limits: crate::domain::limits::RunLimits,
        goal_plan: &GoalPlan,
        native_context: &TaskContext,
    ) -> Result<TaskContext, SessionRuntimeError> {
        let mut context = TaskContext::new(
            session.session_id.clone(),
            session.workspace_ref.clone(),
            limits,
            Map::new(),
        );
        if !goal_plan.delegation_packet_history().is_empty() {
            context
                .set_delegation_packet_history(goal_plan.delegation_packet_history())
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(continuity) = goal_plan.delegation_continuity() {
            context
                .set_delegation_continuity_state(continuity)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(memory) = goal_plan.compacted_canon_memory.as_ref() {
            context
                .set_latest_compacted_canon_memory(memory)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        // Carry the advanced-context retrieval story into task state so later
        // status projections remain stable after execution begins.
        if let Some(advanced_context) = goal_plan
            .context_pack
            .as_ref()
            .and_then(|context_pack| context_pack.advanced_context.as_ref())
        {
            context
                .set_latest_advanced_context(advanced_context)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(story) = goal_plan.cluster_delivery_story.as_ref() {
            context
                .set_cluster_delivery_story(story)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        self.merge_native_task_context(&mut context, native_context);
        Ok(context)
    }

    pub(super) fn merge_native_task_context(
        &self,
        context: &mut TaskContext,
        native_context: &TaskContext,
    ) {
        context.apply_state_patch(&native_context.state);
        for history_ref in &native_context.history_refs {
            context.push_history_ref(history_ref.clone());
        }
        if let Some(last_result) = native_context.last_result.clone() {
            context.set_last_result(last_result);
        }
    }

    pub(super) fn backfill_native_execution_state(
        &self,
        runtime: &FixtureRuntime,
        native_context: &mut TaskContext,
        terminal_status: TaskStatus,
    ) {
        if !native_context.state.contains_key(LATEST_CHANGED_FILES_KEY) {
            let changed_files = runtime
                .profile
                .attempts
                .iter()
                .flat_map(|attempt| attempt.changes.iter().map(|change| change.path.clone()))
                .collect::<Vec<_>>();
            if !changed_files.is_empty() {
                native_context
                    .state
                    .insert(LATEST_CHANGED_FILES_KEY.to_string(), json!(changed_files));
            }
        }

        native_context.state.insert(
            LATEST_VALIDATION_STATUS_KEY.to_string(),
            json!(if terminal_status == TaskStatus::Succeeded {
                VALIDATION_STATUS_PASSED
            } else {
                VALIDATION_STATUS_FAILED
            }),
        );
    }

    pub(super) fn insert_trace_events_before_terminal(
        &self,
        trace: &mut ExecutionTrace,
        events: Vec<TraceEvent>,
    ) {
        let insert_at = trace
            .events
            .iter()
            .rposition(|event| event.event_type == TraceEventType::TerminalRecorded)
            .unwrap_or(trace.events.len());
        trace.events.splice(insert_at..insert_at, events);
    }
}

fn summarize_proof_failure(command_line: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let summary = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .or_else(|| stdout.lines().find(|line| !line.trim().is_empty()))
        .unwrap_or("proof command reported a non-zero exit status");
    format!("proof command failed: `{command_line}`: {summary}")
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::os::unix::process::ExitStatusExt;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use serde_json::{Map, Value, json};
    use uuid::Uuid;

    use crate::domain::completion_verification::{
        ClaimInferenceConfidence, CompletionClaim, CompletionClaimKind, CompletionClaimSource,
        CompletionRequiredAction, CompletionVerificationFindingKind, CompletionVerificationState,
        ProofCommandSelection,
    };
    use crate::domain::execution::{
        ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, WorkspaceChange,
        WorkspaceExecutionProfile,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::Plan;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::step::{Step, StepResultSummary};
    use crate::domain::task::TaskStatus;
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{ExecutionTrace, TraceEventType};
    use crate::fixture::FixtureRuntime;
    use crate::orchestrator::planner::StaticPlanner;
    use crate::registry::agent_registry::AgentRegistry;
    use crate::registry::tool_registry::ToolRegistry;

    use super::{
        LATEST_CHANGED_FILES_KEY, LATEST_VALIDATION_STATUS_KEY, SessionRuntime,
        VALIDATION_STATUS_FAILED, VALIDATION_STATUS_PASSED,
    };

    const ADDED_TRACE_STEP_ID: &str = "inserted-step";
    const ATTEMPT_ID: &str = "attempt-1";
    const CHANGE_PATH: &str = "src/lib.rs";
    const HISTORY_REF: &str = "attempt-ref-1";
    const INITIAL_STEP_ID: &str = "step-1";
    const NATIVE_CONTEXT_KEY: &str = "native_key";
    const PROFILE_NAME: &str = "native-execution-profile";
    const SESSION_ID: &str = "session-1";
    const TERMINAL_STEP_ID: &str = "terminal-step";
    const TRACE_GOAL: &str = "persist native execution";
    const UPDATED_AT: u64 = 111;

    #[test]
    fn native_execution_helpers_cover_merge_backfill_and_terminal_insertion()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-native-execution")?;
        let runtime = sample_runtime()?;
        let session = sample_session(workspace.as_path());
        let helper_runtime = SessionRuntime::for_workspace(workspace.as_path());

        let mut merged_context = TaskContext::new(
            SESSION_ID,
            workspace.as_path().to_string_lossy().into_owned(),
            RunLimits::default(),
            Map::new(),
        );
        let native_context = sample_native_context(workspace.as_path())?;
        helper_runtime.merge_native_task_context(&mut merged_context, &native_context);
        assert_eq!(
            merged_context.state.get(NATIVE_CONTEXT_KEY),
            Some(&Value::String("merged".to_string()))
        );
        assert_eq!(merged_context.history_refs, vec![HISTORY_REF.to_string()]);
        assert_eq!(merged_context.last_result, native_context.last_result);

        let mut backfilled_context = TaskContext::new(
            SESSION_ID,
            workspace.as_path().to_string_lossy().into_owned(),
            RunLimits::default(),
            Map::new(),
        );
        helper_runtime.backfill_native_execution_state(
            &runtime,
            &mut backfilled_context,
            TaskStatus::Succeeded,
        );
        assert_eq!(
            backfilled_context.state.get(LATEST_CHANGED_FILES_KEY),
            Some(&json!([CHANGE_PATH]))
        );
        assert_eq!(
            backfilled_context.state.get(LATEST_VALIDATION_STATUS_KEY),
            Some(&json!(VALIDATION_STATUS_PASSED))
        );

        let mut preserved_changed_files_context = TaskContext::new(
            SESSION_ID,
            workspace.as_path().to_string_lossy().into_owned(),
            RunLimits::default(),
            Map::from_iter([(LATEST_CHANGED_FILES_KEY.to_string(), json!(["existing.rs"]))]),
        );
        helper_runtime.backfill_native_execution_state(
            &runtime,
            &mut preserved_changed_files_context,
            TaskStatus::Failed,
        );
        assert_eq!(
            preserved_changed_files_context.state.get(LATEST_CHANGED_FILES_KEY),
            Some(&json!(["existing.rs"]))
        );
        assert_eq!(
            preserved_changed_files_context.state.get(LATEST_VALIDATION_STATUS_KEY),
            Some(&json!(VALIDATION_STATUS_FAILED))
        );

        let mut trace = ExecutionTrace::new("task-1", SESSION_ID, TRACE_GOAL);
        trace.record_event(
            TraceEventType::StepStarted,
            Some(INITIAL_STEP_ID.to_string()),
            1,
            json!({"kind": "initial"}),
        );
        trace.record_event(
            TraceEventType::TerminalRecorded,
            Some(TERMINAL_STEP_ID.to_string()),
            1,
            json!({"kind": "terminal"}),
        );
        let inserted_event =
            sample_trace_event(TraceEventType::GovernanceBlocked, ADDED_TRACE_STEP_ID);
        helper_runtime.insert_trace_events_before_terminal(&mut trace, vec![inserted_event]);
        assert_eq!(trace.events.len(), 3);
        assert_eq!(trace.events[1].event_type, TraceEventType::GovernanceBlocked);
        assert_eq!(trace.events[2].event_type, TraceEventType::TerminalRecorded);

        let mut trace_without_terminal = ExecutionTrace::new("task-2", SESSION_ID, TRACE_GOAL);
        trace_without_terminal.record_event(
            TraceEventType::StepStarted,
            Some(INITIAL_STEP_ID.to_string()),
            1,
            json!({"kind": "initial"}),
        );
        helper_runtime.insert_trace_events_before_terminal(
            &mut trace_without_terminal,
            vec![sample_trace_event(TraceEventType::RetryScheduled, "retry-step")],
        );
        assert_eq!(trace_without_terminal.events.len(), 2);
        assert_eq!(trace_without_terminal.events[1].event_type, TraceEventType::RetryScheduled);

        let _ = session;
        Ok(())
    }

    #[test]
    fn completion_proof_execution_and_refresh_cover_success_failure_and_staleness()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-native-completion-proof")?;
        fs::create_dir_all(workspace.as_path().join("src"))?;
        fs::write(
            workspace.as_path().join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )?;

        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());
        let mut success_task = completion_task(workspace.as_path(), "native-proof-success")?;
        let mut success_trace =
            ExecutionTrace::new("native-proof-success", SESSION_ID, "prove the bug fix");

        let success_response = runtime.execute_completion_proof(
            &mut session,
            &mut success_task,
            &mut success_trace,
            super::NativeProofExecutionRequest {
                claim: bug_fix_claim(),
                documentation_relevant: false,
                command_line: "true".to_string(),
                command_ref: "proof.true".to_string(),
            },
        )?;
        assert_eq!(success_response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(session.latest_status, SessionStatus::Succeeded);
        success_task.context.set_completion_claim(&bug_fix_claim())?;
        success_task.context.set_completion_proof_selection(&ProofCommandSelection {
            claim_id: "claim-bug-fix".to_string(),
            command_ref: "proof.true".to_string(),
            command_line: "true".to_string(),
            selection_reason: "test proof".to_string(),
            coverage_note: None,
            documentation_relevant: false,
        })?;
        assert!(success_task.context.completion_verification_projection()?.is_some_and(
            |projection| {
                projection.completion_verification_state == CompletionVerificationState::Ready
            }
        ));

        fs::write(
            workspace.as_path().join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right + 1 }\n",
        )?;
        let mut stale_session = sample_session(workspace.as_path());
        stale_session.active_task = Some(success_task.clone());
        stale_session.latest_status = SessionStatus::Succeeded;
        assert!(runtime.refresh_completion_verification_state(&mut stale_session)?);
        assert_eq!(stale_session.latest_status, SessionStatus::Blocked);
        assert!(
            stale_session
                .active_task
                .as_ref()
                .and_then(|task| task.context.completion_verification_projection().ok().flatten())
                .is_some_and(|projection| {
                    projection.completion_verification_state
                        == CompletionVerificationState::ProofRequired
                        && projection.completion_verification_findings.iter().any(|finding| {
                            finding.kind == CompletionVerificationFindingKind::StaleProof
                        })
                })
        );

        let mut failed_session = sample_session(workspace.as_path());
        let mut failed_task = completion_task(workspace.as_path(), "native-proof-failure")?;
        let mut failed_trace =
            ExecutionTrace::new("native-proof-failure", SESSION_ID, "prove the bug fix");
        let failed_response = runtime.execute_completion_proof(
            &mut failed_session,
            &mut failed_task,
            &mut failed_trace,
            super::NativeProofExecutionRequest {
                claim: bug_fix_claim(),
                documentation_relevant: false,
                command_line: "printf 'proof exploded\\n' >&2; false".to_string(),
                command_ref: "proof.false".to_string(),
            },
        )?;
        assert_eq!(failed_response.terminal_status, TaskStatus::Running);
        assert_eq!(failed_session.latest_status, SessionStatus::Blocked);
        assert!(failed_task.context.completion_verification_projection()?.is_some_and(
            |projection| {
                projection.completion_verification_state == CompletionVerificationState::Failed
                    && projection.completion_verification_findings.iter().any(|finding| {
                        finding.required_action == CompletionRequiredAction::RerunProof
                    })
            }
        ));

        Ok(())
    }

    #[test]
    fn helper_failure_summary_covers_stderr_fallback() -> Result<(), Box<dyn Error>> {
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(256),
            stdout: Vec::new(),
            stderr: b"proof exploded\n".to_vec(),
        };
        let summary = super::summarize_proof_failure("cargo test --quiet", &output);
        assert!(summary.contains("cargo test --quiet"));
        assert!(summary.contains("proof exploded"));

        Ok(())
    }

    fn sample_runtime() -> Result<FixtureRuntime, Box<dyn Error>> {
        let planner = Arc::new(StaticPlanner::new(Plan::new(vec![Step::agent(
            INITIAL_STEP_ID,
            "planner",
            json!({"goal": TRACE_GOAL}),
        )?])?));
        Ok(FixtureRuntime {
            profile: WorkspaceExecutionProfile {
                name: PROFILE_NAME.to_string(),
                read_targets: vec![CHANGE_PATH.to_string()],
                validation_command: ExecutionCommand {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string()],
                },
                attempts: vec![ExecutionAttemptDefinition {
                    attempt_id: ATTEMPT_ID.to_string(),
                    summary: "apply change".to_string(),
                    failure_mode: ExecutionFailureMode::Retry,
                    changes: vec![WorkspaceChange {
                        path: CHANGE_PATH.to_string(),
                        find: "before".to_string(),
                        replace: "after".to_string(),
                    }],
                }],
                adaptive: None,
                limits: RunLimits::default(),
                governance: None,
                review: None,
                legacy_source: None,
            },
            planner,
            agents: AgentRegistry::new(),
            tools: ToolRegistry::new(),
        })
    }

    fn sample_session(workspace: &Path) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: SESSION_ID.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some(TRACE_GOAL.to_string()),
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

    fn sample_native_context(workspace: &Path) -> Result<TaskContext, Box<dyn Error>> {
        let mut context = TaskContext::new(
            SESSION_ID,
            workspace.to_string_lossy().into_owned(),
            RunLimits::default(),
            Map::from_iter([(NATIVE_CONTEXT_KEY.to_string(), Value::String("merged".to_string()))]),
        );
        context.push_history_ref(HISTORY_REF);

        let mut succeeded_step =
            Step::agent(INITIAL_STEP_ID, "planner", json!({"goal": TRACE_GOAL}))?;
        succeeded_step.mark_succeeded(json!({"result": "ok"}));
        context.set_last_result(StepResultSummary::from_step(&succeeded_step));
        Ok(context)
    }

    fn sample_trace_event(
        event_type: TraceEventType,
        step_id: &str,
    ) -> crate::domain::trace::TraceEvent {
        crate::domain::trace::TraceEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            step_id: Some(step_id.to_string()),
            plan_revision: 1,
            payload: json!({"inserted": true}),
            recorded_at: UPDATED_AT,
        }
    }

    fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
        TestWorkspace::new(prefix)
    }

    fn completion_task(
        workspace: &Path,
        task_id: &str,
    ) -> Result<crate::domain::task::Task, Box<dyn Error>> {
        let request = crate::domain::task::TaskRunRequest {
            goal: "prove the bug fix".to_string(),
            input: json!({}),
            session_id: SESSION_ID.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let plan = Plan::new(vec![Step::agent(
            INITIAL_STEP_ID,
            "planner",
            json!({"goal": "prove the bug fix"}),
        )?])?;
        Ok(crate::domain::task::Task::new(task_id, &request, plan)?)
    }

    fn bug_fix_claim() -> CompletionClaim {
        CompletionClaim {
            claim_id: "claim-bug-fix".to_string(),
            kind: CompletionClaimKind::BugFixed,
            scope: crate::domain::completion_verification::CompletionVerificationScope::Task,
            source: CompletionClaimSource::RuntimeInference,
            confidence: Some(ClaimInferenceConfidence::High),
            summary: "bug fix is complete".to_string(),
            supporting_signals: vec!["goal_text".to_string(), "changed_files".to_string()],
        }
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
