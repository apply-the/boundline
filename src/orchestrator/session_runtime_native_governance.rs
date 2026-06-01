use std::fs;

use serde_json::json;
use uuid::Uuid;

use crate::adapters::governance_runtime::{
    CanonCliRuntime, GovernanceRequestKind, GovernanceRuntime, GovernanceRuntimeRequest,
    GovernanceRuntimeResponse,
};
use crate::domain::decision::Decision;
use crate::domain::flow::FlowStepMetadata;
use crate::domain::goal_plan::GoalPlan;
use crate::domain::governance::{
    CanonAuthorityZone, CanonIntendedPersona, CanonMode, CanonRiskClass, GovernanceLifecycleState,
    GovernanceRuntimeKind, GovernedStageRecord, PacketReadiness, execution_stage_key_for_mode,
};
use crate::domain::limits::TerminalCondition;
use crate::domain::project_memory::{
    GovernedEvidencePromotionRequest,
    promote_governed_evidence_bundle as promote_project_evidence_bundle,
};
use crate::domain::session::ActiveSessionRecord;
use crate::domain::task::{Task, TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType};
use crate::fixture::{FixtureRuntime, build_fixture_plan_for_goal, build_task_request};
use crate::orchestrator::decision_loop::LoopTerminal;
use crate::orchestrator::governance::{
    append_governed_document_to_lifecycle, compacted_canon_memory_from_response,
    governance_projection_snapshot, governance_state_patch, governed_document_ref_from_response,
    overlay_stage_policy_with_intent, planning_governance_input_documents,
    requested_governance_intent, runtime_command_available, selected_stage_policy,
};
use crate::orchestrator::terminal::build_terminal_reason;

use super::{
    EXECUTION_GOVERNANCE_ROOT, EXECUTION_STAGE_BRIEF_FILE_NAME, GovernanceStepDecision,
    NativeGovernanceProjection, SessionRuntime, SessionRuntimeError,
    canon_workspace_scope_mismatch_reason, execution_governance_read_targets,
    is_governance_trace_event, reasoning_profile_block_message, render_execution_stage_brief,
};

impl SessionRuntime {
    /// Invokes Canon governance with execution-time modes after the decision
    /// loop produces implementation artifacts. Only activates when the session
    /// has an active governance lifecycle backed by the Canon runtime and the
    /// Canon CLI command is available.
    pub(super) fn execute_post_implementation_governance(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &FixtureRuntime,
        goal_plan: &mut GoalPlan,
        decisions: &[Decision],
        native_context: &mut TaskContext,
        trace: &mut ExecutionTrace,
    ) -> Result<(), SessionRuntimeError> {
        let Some(lifecycle) = session.governance_lifecycle.as_ref() else {
            return Ok(());
        };
        if lifecycle.governance_runtime != GovernanceRuntimeKind::Canon {
            return Ok(());
        }
        let Some(governance) = runtime.profile.governance.as_ref() else {
            return Ok(());
        };
        let Some(canon) = governance.canon.as_ref() else {
            return Ok(());
        };
        if !runtime_command_available(&canon.command) {
            return Ok(());
        }
        if canon_workspace_scope_mismatch_reason(&self.workspace_ref).is_some() {
            return Ok(());
        }

        let goal = goal_plan.goal_text.clone();
        let execution_modes: &[CanonMode] = &[CanonMode::Implementation, CanonMode::Verification];

        for &mode in execution_modes {
            let Some(stage_key) = execution_stage_key_for_mode(mode) else {
                continue;
            };
            let stage_brief_ref = self.materialize_execution_stage_brief(
                mode,
                decisions,
                goal_plan,
                native_context,
                &runtime.profile.read_targets,
            )?;
            let governance_attempt_id = Uuid::new_v4().to_string();
            let previous_attempt_id = session.governance_lifecycle.as_ref().and_then(|lifecycle| {
                lifecycle
                    .stage_records
                    .iter()
                    .rev()
                    .find(|record| record.stage_key == stage_key)
                    .map(|record| record.governance_attempt_id.clone())
            });
            let input_documents = planning_governance_input_documents(
                session.authored_brief.as_ref(),
                &stage_brief_ref,
                goal_plan.compacted_canon_memory.as_ref(),
            );
            let read_targets =
                execution_governance_read_targets(native_context, &runtime.profile.read_targets);

            let request = GovernanceRuntimeRequest {
                request_kind: GovernanceRequestKind::Start,
                governance_attempt_id: governance_attempt_id.clone(),
                stage_key: stage_key.to_string(),
                goal: goal.clone(),
                workspace_ref: self.workspace_ref.to_string_lossy().to_string(),
                autopilot: true,
                mode: Some(mode),
                system_context: canon.default_system_context,
                risk: canon.default_risk.clone().map(|risk| {
                    CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
                }),
                zone: canon.default_zone.clone().map(|zone| {
                    CanonAuthorityZone::canonicalize_label(&zone)
                        .map(str::to_string)
                        .unwrap_or(zone)
                }),
                owner: canon.default_owner.clone().map(|owner| {
                    CanonIntendedPersona::canonicalize_label(&owner)
                        .map(str::to_string)
                        .unwrap_or(owner)
                }),
                run_ref: None,
                packet_ref: None,
                bounded_context: crate::adapters::governance_runtime::GovernanceBoundedContext {
                    read_targets: read_targets.clone(),
                    stage_brief_ref: Some(stage_brief_ref.clone()),
                    reused_packets: Vec::new(),
                },
                input_documents,
            };

            trace.record_event(
                TraceEventType::GovernanceStarted,
                None,
                goal_plan.proposal_revision,
                json!({
                    "stage_key": stage_key,
                    "runtime": GovernanceRuntimeKind::Canon,
                    "canon_mode": mode,
                    "phase": "post-implementation",
                    "stage_brief_ref": stage_brief_ref,
                    "read_targets": read_targets,
                }),
            );

            let response = match CanonCliRuntime::new(canon.command.clone())
                .with_working_directory(&self.workspace_ref)
                .execute(&request)
            {
                Ok(response) => response,
                Err(error) => {
                    trace.record_event(
                        TraceEventType::GovernanceCompleted,
                        None,
                        goal_plan.proposal_revision,
                        json!({
                            "stage_key": stage_key,
                            "canon_mode": mode,
                            "status": "error",
                            "message": error.to_string(),
                        }),
                    );
                    break;
                }
            };

            let blocked_reason = matches!(
                response.status,
                GovernanceLifecycleState::AwaitingApproval
                    | GovernanceLifecycleState::Blocked
                    | GovernanceLifecycleState::Failed
            )
            .then(|| response.message.clone());

            let record = GovernedStageRecord {
                stage_key: stage_key.to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                lifecycle_state: response.status,
                required: false,
                autopilot_enabled: true,
                approval_state: response.approval_state,
                canon_run_ref: response.run_ref.clone(),
                governance_attempt_id,
                previous_governance_attempt_id: previous_attempt_id,
                packet_ref: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                decision_ref: None,
                stage_council: None,
                blocked_reason: blocked_reason.clone(),
            };

            let compacted_canon_memory = compacted_canon_memory_from_response(
                stage_key,
                GovernanceRuntimeKind::Canon,
                &response,
            );
            if let Some(memory) = compacted_canon_memory.clone() {
                goal_plan.compacted_canon_memory = Some(memory);
            }
            let projection = governance_projection_snapshot(
                native_context,
                stage_key,
                response.packet.as_ref(),
                response.approval_state,
            )
            .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
            let patch = governance_state_patch(
                &record,
                response.packet.as_ref(),
                None,
                None,
                compacted_canon_memory.as_ref(),
                &projection,
            )
            .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
            native_context.apply_state_patch(&patch);

            trace.record_event(
                TraceEventType::GovernanceCompleted,
                None,
                goal_plan.proposal_revision,
                json!({
                    "stage_key": stage_key,
                    "canon_mode": mode,
                    "packet_ref": response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                    "packet_readiness": response.packet.as_ref().map(|packet| packet.readiness),
                    "document_refs": response.packet.as_ref().map(|packet| packet.document_refs.clone()).unwrap_or_default(),
                    "headline": response.packet.as_ref().map(|packet| packet.headline.clone()).unwrap_or_else(|| response.message.clone()),
                    "status": response.status,
                    "approval_state": response.approval_state,
                    "run_ref": response.run_ref,
                    "latest_governance_runtime_state": projection.runtime_state,
                    "latest_governance_rollout_profile": projection.rollout_profile,
                    "latest_governance_reason": projection.reason,
                    "latest_governance_contract_lines": projection.contract_lines,
                    "latest_governance_approval_provenance": projection.approval_provenance,
                }),
            );

            self.upsert_execution_stage_record(session, record);

            if response.status == GovernanceLifecycleState::GovernedReady
                && response.packet.is_some()
            {
                let doc_ref = governed_document_ref_from_response(stage_key, mode, &response);
                append_governed_document_to_lifecycle(session, doc_ref);
                self.promote_governed_evidence_outputs(stage_key, mode, &response)?;
            }

            if response.status != GovernanceLifecycleState::GovernedReady {
                break;
            }
        }

        Ok(())
    }

    fn upsert_execution_stage_record(
        &self,
        session: &mut ActiveSessionRecord,
        record: GovernedStageRecord,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        if let Some(existing_index) = lifecycle
            .stage_records
            .iter()
            .position(|existing| existing.stage_key == record.stage_key)
        {
            lifecycle.stage_records[existing_index] = record;
        } else {
            lifecycle.stage_records.push(record);
        }
    }

    pub(super) fn promote_governed_evidence_outputs(
        &self,
        stage_key: &str,
        canon_mode: CanonMode,
        response: &GovernanceRuntimeResponse,
    ) -> Result<(), SessionRuntimeError> {
        let Some(packet) = response.packet.as_ref() else {
            return Ok(());
        };
        if packet.readiness != PacketReadiness::Reusable || packet.document_refs.is_empty() {
            return Ok(());
        }
        let Some(run_ref) = response.run_ref.as_deref().filter(|value| !value.trim().is_empty())
        else {
            return Ok(());
        };

        promote_project_evidence_bundle(
            &self.workspace_ref,
            GovernedEvidencePromotionRequest {
                canon_mode,
                stage_key,
                run_ref,
                approval_state: response.approval_state,
                packet_readiness: packet.readiness,
                packet_ref: &packet.packet_ref,
                document_refs: &packet.document_refs,
            },
        )
        .map(|_| ())
        .map_err(|error| SessionRuntimeError::GovernanceRuntime(error.to_string()))
    }

    fn materialize_execution_stage_brief(
        &self,
        mode: CanonMode,
        decisions: &[Decision],
        goal_plan: &GoalPlan,
        native_context: &TaskContext,
        fallback_targets: &[String],
    ) -> Result<String, SessionRuntimeError> {
        let stage_brief_ref = format!(
            "{}/{}/{}",
            EXECUTION_GOVERNANCE_ROOT,
            mode.as_str(),
            EXECUTION_STAGE_BRIEF_FILE_NAME
        );
        let stage_brief_path = self.workspace_ref.join(&stage_brief_ref);
        let Some(parent) = stage_brief_path.parent() else {
            return Err(SessionRuntimeError::ExecutionInvariant(format!(
                "execution stage brief path has no parent for mode {}",
                mode.as_str()
            )));
        };
        fs::create_dir_all(parent).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to create execution stage brief directory for {}: {error}",
                mode.as_str()
            ))
        })?;
        fs::write(
            &stage_brief_path,
            render_execution_stage_brief(
                mode,
                goal_plan,
                decisions,
                native_context,
                fallback_targets,
            ),
        )
        .map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to write execution stage brief for {}: {error}",
                mode.as_str()
            ))
        })?;
        Ok(stage_brief_ref)
    }

    pub(super) fn prepare_native_governance_projection(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &FixtureRuntime,
        goal_plan: &GoalPlan,
    ) -> Result<NativeGovernanceProjection, SessionRuntimeError> {
        let Some(active_flow) = session.active_flow.as_ref() else {
            return Ok(NativeGovernanceProjection::None);
        };
        let Some(governance) = runtime.profile.governance.as_ref() else {
            return Ok(NativeGovernanceProjection::None);
        };
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let request = build_task_request(
            &self.workspace_ref,
            &goal,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let mut task = if let Some(active_task) = session
            .active_task
            .as_ref()
            .filter(|task| task.goal == goal && !task.status.is_terminal())
        {
            active_task.clone()
        } else {
            let plan = build_fixture_plan_for_goal(&self.workspace_ref, Some(active_flow), &goal)
                .map_err(SessionRuntimeError::FixtureRuntime)?;
            Task::new(Uuid::new_v4().to_string(), &request, plan)
                .map_err(SessionRuntimeError::TaskRequest)?
        };
        let mut governance_trace = self.build_goal_plan_trace(&session.session_id, goal_plan);
        let mut saw_governance = false;
        let start_step_index = task.plan.current_step_index;

        for step_index in start_step_index..task.plan.steps.len() {
            task.plan.current_step_index = step_index;
            let step = task.plan.steps[step_index].clone();
            let Some(metadata) = FlowStepMetadata::from_step(&step)
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
            else {
                continue;
            };
            let Some(policy) =
                selected_stage_policy(Some(governance), &metadata.flow_name, &metadata.stage_id)
            else {
                continue;
            };
            let governance_intent = requested_governance_intent(&task.input);
            let policy = overlay_stage_policy_with_intent(&policy, governance_intent.as_ref());
            if !policy.enabled {
                continue;
            }
            saw_governance = true;

            match self.execute_governance_for_step(
                session,
                &mut task,
                &mut governance_trace,
                runtime,
                &step,
                &metadata,
                governance,
                &policy,
                GovernanceRequestKind::Start,
            )? {
                GovernanceStepDecision::Continue => {}
                GovernanceStepDecision::Halt => {
                    let response =
                        self.build_native_governance_halt_response(session, &mut task)?;
                    return Ok(NativeGovernanceProjection::Terminal {
                        response: Box::new(response),
                        task: Box::new(task),
                    });
                }
                GovernanceStepDecision::Terminal(response) => {
                    return Ok(NativeGovernanceProjection::Terminal {
                        response: Box::new(response),
                        task: Box::new(task),
                    });
                }
            }
        }

        if !saw_governance {
            return Ok(NativeGovernanceProjection::None);
        }

        let events = governance_trace
            .events
            .into_iter()
            .filter(|event| is_governance_trace_event(event.event_type))
            .collect();
        Ok(NativeGovernanceProjection::Task { task: Box::new(task), events })
    }

    pub(super) fn finalize_native_projected_task(
        &self,
        mut task: Task,
        terminal_status: TaskStatus,
        terminal_reason: &TerminalReason,
        native_context: &TaskContext,
    ) -> Task {
        task.context.apply_state_patch(&native_context.state);
        task.apply_terminal(terminal_status, terminal_reason.clone());
        task
    }

    pub(super) fn synthesize_native_persisted_task(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
        final_context: &TaskContext,
        terminal_status: TaskStatus,
        terminal_reason: &TerminalReason,
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
        task.apply_terminal(terminal_status, terminal_reason.clone());
        Ok(task)
    }

    fn build_native_governance_halt_response(
        &self,
        session: &ActiveSessionRecord,
        task: &mut Task,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        if matches!(task.status, TaskStatus::Planned) {
            task.mark_running();
        }
        let latest_governance = task
            .context
            .latest_governance_stage()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
            .ok_or(SessionRuntimeError::MissingGovernanceStage)?;
        if let Some(reasoning_profile) = session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
            .filter(|record| {
                record.stage_key == latest_governance.stage_key
                    && record.status.halts_outer_workflow()
            })
        {
            let trace_location = session
                .latest_trace_ref
                .clone()
                .ok_or(SessionRuntimeError::MissingTraceReference)?;

            return Ok(TaskRunResponse {
                task_id: task.id.clone(),
                terminal_status: TaskStatus::Running,
                terminal_reason: build_terminal_reason(
                    TerminalCondition::TaskNotCredible,
                    reasoning_profile_block_message(reasoning_profile),
                    Some(json!({
                        "stage_key": reasoning_profile.stage_key,
                        "profile_id": reasoning_profile.profile_id,
                        "status": reasoning_profile.status,
                    })),
                ),
                final_context: task.context.clone(),
                plan_revision: task.plan.revision,
                trace_location,
            });
        }
        let message = match latest_governance.lifecycle_state {
            GovernanceLifecycleState::AwaitingApproval => {
                format!("governance approval is still pending for {}", latest_governance.stage_key)
            }
            GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed => format!(
                "governance blocked stage {}: {}",
                latest_governance.stage_key,
                latest_governance
                    .blocked_reason
                    .clone()
                    .unwrap_or_else(|| "governance review did not clear the stage".to_string())
            ),
            _ => format!("governance is still in progress for {}", latest_governance.stage_key),
        };
        let trace_location =
            session.latest_trace_ref.clone().ok_or(SessionRuntimeError::MissingTraceReference)?;

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: TaskStatus::Running,
            terminal_reason: build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                message,
                Some(json!({
                    "stage_key": latest_governance.stage_key,
                    "state": latest_governance.lifecycle_state,
                })),
            ),
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    pub(super) fn native_terminal_reason(&self, terminal: &LoopTerminal) -> TerminalReason {
        match terminal {
            LoopTerminal::Success => build_terminal_reason(
                TerminalCondition::GoalSatisfied,
                "goal plan completed through the native decision loop",
                None,
            ),
            LoopTerminal::Failure(message) => {
                build_terminal_reason(TerminalCondition::UnrecoverableError, message, None)
            }
            LoopTerminal::Exhausted { steps_taken, max_steps } => build_terminal_reason(
                TerminalCondition::StepLimitExceeded,
                format!("native goal plan exhausted after {steps_taken} decision step(s)"),
                Some(json!({
                    "steps_taken": steps_taken,
                    "max_steps": max_steps,
                })),
            ),
            LoopTerminal::NoActionableState(message) => {
                build_terminal_reason(TerminalCondition::NoCredibleNextStep, message, None)
            }
        }
    }
}
