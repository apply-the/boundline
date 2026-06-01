use uuid::Uuid;

use super::{
    ActiveSessionRecord, FixtureRuntimeError, FlowStepMetadata, GovernanceLifecycleState,
    GovernanceRequestKind, GovernanceStepDecision, SessionCommand, SessionRuntime,
    SessionRuntimeError, SessionStatus, Task, TaskRunResponse, build_fixture_plan_for_goal,
    build_task_request, current_timestamp_millis, load_workspace_execution_profile,
    overlay_stage_policy_with_intent, requested_governance_intent, selected_stage_policy,
};

impl SessionRuntime {
    /// Advances the active session by exactly one bounded step.
    /// Flow-selected goal plans are bridged into compatibility tasks when a
    /// fixture execution profile remains authoritative.
    pub fn execute_next_step(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        if session.active_task.is_none() && session.latest_status.is_terminal() {
            return Err(SessionRuntimeError::MissingActiveTask);
        }
        if session.active_task.is_none()
            && self.flow_selected_goal_plan_uses_compatibility_step(session)?
        {
            self.ensure_flow_selected_compatibility_task(session)?;
        }
        let checkpoint_projection =
            self.prepare_checkpoint_for_mutation(session, SessionCommand::Step)?;
        let runtime = self.build_runtime(session)?;
        let _ = self.execute_single_step(session, &runtime)?;
        if let Some(projection) = checkpoint_projection.as_ref() {
            self.refresh_checkpoint_projection(session, projection)?;
        }
        Ok(())
    }

    pub(super) fn flow_selected_goal_plan_uses_compatibility_step(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        if session.goal_plan.is_none() || session.active_flow.is_none() {
            return Ok(false);
        }

        match load_workspace_execution_profile(&self.workspace_ref) {
            Ok(_) => Ok(true),
            Err(FixtureRuntimeError::MissingExecutionProfile(_)) => Ok(false),
            Err(error) => Err(SessionRuntimeError::FixtureRuntime(error)),
        }
    }

    pub(super) fn ensure_flow_selected_compatibility_task(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        if session.active_task.is_some() {
            return Ok(());
        }

        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        if let Some(active_flow) = &session.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

        let request = build_task_request(
            &self.workspace_ref,
            &goal,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan =
            build_fixture_plan_for_goal(&self.workspace_ref, session.active_flow.as_ref(), &goal)
                .map_err(SessionRuntimeError::FixtureRuntime)?;
        let task = Task::new(Uuid::new_v4().to_string(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;

        session.active_task = Some(task);
        session.decisions.clear();
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    /// Continues the active session until it reaches a terminal response.
    /// Native goal-plan sessions use the native path; compatibility sessions
    /// continue one fixture step at a time until terminal.
    pub fn run_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let checkpoint_projection =
            self.prepare_checkpoint_for_mutation(session, SessionCommand::Run)?;
        if session.goal_plan.is_some() {
            let claimed_stage_outcome = self.maybe_execute_framework_adapter_run_stage(
                session,
                checkpoint_projection.clone(),
            )?;
            let (response, claimed_stage_runtime, not_claimed_routing) = match claimed_stage_outcome
            {
                super::FrameworkAdapterRunStageOutcome::NotClaimed { routing_record } => (
                    self.run_native_goal_plan(session, checkpoint_projection.clone())?,
                    None,
                    routing_record,
                ),
                super::FrameworkAdapterRunStageOutcome::Completed { stage_runtime, response } => (
                    self.persist_framework_adapter_run_stage_success(
                        session,
                        checkpoint_projection.clone(),
                        &stage_runtime,
                        response,
                    )?,
                    Some(stage_runtime),
                    None,
                ),
                super::FrameworkAdapterRunStageOutcome::Blocked(blocked) => (
                    self.persist_framework_adapter_run_stage_blocked(
                        session,
                        checkpoint_projection.clone(),
                        blocked,
                    )?,
                    None,
                    None,
                ),
                super::FrameworkAdapterRunStageOutcome::Terminal { stage_runtime, response } => {
                    (*response, Some(stage_runtime), None)
                }
            };
            if let Some(routing_record) = not_claimed_routing {
                self.record_framework_adapter_run_stage_not_claimed_routing(
                    session,
                    &response.trace_location,
                    response.plan_revision,
                    routing_record,
                )?;
            }
            if let Some(stage_runtime) = claimed_stage_runtime.as_ref() {
                self.emit_framework_adapter_run_stage_hook(
                    session,
                    stage_runtime,
                    response.terminal_status,
                    &response.trace_location,
                )?;
            }
            if let Some(projection) = checkpoint_projection.as_ref() {
                self.refresh_checkpoint_projection(session, projection)?;
            }
            return Ok(response);
        }

        let runtime = self.build_runtime(session)?;

        loop {
            if let Some(response) = self.execute_single_step(session, &runtime)? {
                if let Some(projection) = checkpoint_projection.as_ref() {
                    self.refresh_checkpoint_projection(session, projection)?;
                }
                return Ok(response);
            }
        }
    }

    /// Refreshes governance state when a run is paused awaiting approval and a
    /// governance runtime can provide a newer answer.
    pub fn refresh_governance_state(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        let Some(mut task) = session.active_task.take() else {
            return Ok(false);
        };
        let result = (|| {
            let Some(record) = task
                .context
                .latest_governance_stage()
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
            else {
                return Ok(false);
            };
            if record.lifecycle_state != GovernanceLifecycleState::AwaitingApproval {
                return Ok(false);
            }

            let runtime = self.build_runtime(session)?;
            let mut trace = self.load_or_create_trace(session, &task)?;
            let step =
                task.plan.current_step().cloned().ok_or(SessionRuntimeError::MissingActiveTask)?;
            let metadata = FlowStepMetadata::from_step(&step)
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
                .ok_or_else(|| {
                    SessionRuntimeError::InvalidFlowState(
                        "governance refresh requires flow metadata".to_string(),
                    )
                })?;
            let Some(governance) = runtime.profile.governance.as_ref() else {
                return Ok(false);
            };
            let Some(policy) =
                selected_stage_policy(Some(governance), &metadata.flow_name, &metadata.stage_id)
            else {
                return Ok(false);
            };
            let governance_intent = requested_governance_intent(&task.input);
            let policy = overlay_stage_policy_with_intent(&policy, governance_intent.as_ref());

            let decision = self.execute_governance_for_step(
                session,
                &mut task,
                &mut trace,
                &runtime,
                &step,
                &metadata,
                governance,
                &policy,
                GovernanceRequestKind::Refresh,
            )?;

            Ok(!matches!(decision, GovernanceStepDecision::Continue))
        })();
        session.active_task = Some(task);
        result
    }
}
