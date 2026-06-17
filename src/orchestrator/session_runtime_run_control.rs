use uuid::Uuid;

use super::runtime_support::FrameworkAdapterClaimedStageRuntime;
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
            return self.run_goal_plan_to_terminal(session, checkpoint_projection);
        }

        let runtime = self.build_runtime(session)?;

        loop {
            if let Some(response) = self.execute_single_step(session, &runtime)? {
                self.refresh_checkpoint_projection_if_present(
                    session,
                    checkpoint_projection.as_ref(),
                )?;
                return Ok(response);
            }
        }
    }

    fn run_goal_plan_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<super::CheckpointProjectionState>,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        if let Some(response) = self.maybe_resume_completion_verification(session)? {
            self.refresh_checkpoint_projection_if_present(session, checkpoint_projection.as_ref())?;
            return Ok(response);
        }

        let claimed_stage_outcome =
            self.maybe_execute_framework_adapter_run_stage(session, checkpoint_projection.clone())?;
        let (response, claimed_stage_runtime, not_claimed_routing) = self
            .resolve_goal_plan_run_outcome(session, checkpoint_projection, claimed_stage_outcome)?;

        self.record_not_claimed_run_stage_if_present(session, &response, not_claimed_routing)?;
        self.emit_run_stage_hook_if_present(session, &response, claimed_stage_runtime.as_ref())?;
        Ok(response)
    }

    fn resolve_goal_plan_run_outcome(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<super::CheckpointProjectionState>,
        claimed_stage_outcome: super::FrameworkAdapterRunStageOutcome,
    ) -> Result<
        (
            TaskRunResponse,
            Option<FrameworkAdapterClaimedStageRuntime>,
            Option<crate::domain::execution::StageRoutingDecisionRecord>,
        ),
        SessionRuntimeError,
    > {
        let outcome = match claimed_stage_outcome {
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
        self.refresh_checkpoint_projection_if_present(session, checkpoint_projection.as_ref())?;
        Ok(outcome)
    }

    fn record_not_claimed_run_stage_if_present(
        &self,
        session: &mut ActiveSessionRecord,
        response: &TaskRunResponse,
        routing_record: Option<crate::domain::execution::StageRoutingDecisionRecord>,
    ) -> Result<(), SessionRuntimeError> {
        if let Some(routing_record) = routing_record {
            self.record_framework_adapter_run_stage_not_claimed_routing(
                session,
                &response.trace_location,
                response.plan_revision,
                routing_record,
            )?;
        }
        Ok(())
    }

    fn emit_run_stage_hook_if_present(
        &self,
        session: &ActiveSessionRecord,
        response: &TaskRunResponse,
        stage_runtime: Option<&FrameworkAdapterClaimedStageRuntime>,
    ) -> Result<(), SessionRuntimeError> {
        if let Some(stage_runtime) = stage_runtime {
            self.emit_framework_adapter_run_stage_hook(
                session,
                stage_runtime,
                response.terminal_status,
                &response.trace_location,
            )?;
        }
        Ok(())
    }

    fn refresh_checkpoint_projection_if_present(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<&super::CheckpointProjectionState>,
    ) -> Result<(), SessionRuntimeError> {
        if let Some(projection) = checkpoint_projection {
            self.refresh_checkpoint_projection(session, projection)?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    fn write_source_target(workspace: &Path) -> Result<(), Box<dyn Error>> {
        let source_dir = workspace.join("src");
        fs::create_dir_all(&source_dir)?;
        fs::write(source_dir.join("lib.rs"), "pub fn sample_target() -> bool { true }\n")?;
        Ok(())
    }
    use crate::domain::execution::{
        ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, WorkspaceChange,
        WorkspaceExecutionProfile,
    };
    use crate::domain::flow::built_in_flow;
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::governance::{
        ApprovalState, GovernanceLifecycleState, GovernanceProfile, GovernanceRuntimeKind,
        GovernedStageRecord,
    };
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::task::TerminalReason;

    use super::{ActiveSessionRecord, SessionRuntime, SessionRuntimeError, SessionStatus};

    const GOAL_TEXT: &str = "Drive the session runtime";
    const SESSION_ID: &str = "session-run-control";
    const UPDATED_AT: u64 = 42;

    #[test]
    fn run_control_execute_next_step_rejects_terminal_session_without_task()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-run-control-terminal")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session =
            sample_session(workspace.as_path(), Some(sample_goal_plan()?), SessionStatus::Failed);

        let error = runtime.execute_next_step(&mut session).unwrap_err();
        assert!(matches!(error, SessionRuntimeError::MissingActiveTask));

        Ok(())
    }

    #[test]
    fn run_control_flow_selected_compatibility_step_handles_missing_and_invalid_profiles()
    -> Result<(), Box<dyn Error>> {
        let missing_workspace = temp_workspace("boundline-run-control-missing-profile")?;
        let missing_runtime = SessionRuntime::for_workspace(missing_workspace.as_path());
        let missing_session = sample_flow_selected_session(missing_workspace.as_path())?;
        assert!(
            !missing_runtime.flow_selected_goal_plan_uses_compatibility_step(&missing_session)?
        );

        let invalid_workspace = temp_workspace("boundline-run-control-invalid-profile")?;
        fs::create_dir_all(invalid_workspace.as_path().join(".boundline"))?;
        fs::write(
            invalid_workspace.as_path().join(".boundline").join("execution.json"),
            "{not-json",
        )?;
        let invalid_runtime = SessionRuntime::for_workspace(invalid_workspace.as_path());
        let invalid_session = sample_flow_selected_session(invalid_workspace.as_path())?;

        let error = invalid_runtime
            .flow_selected_goal_plan_uses_compatibility_step(&invalid_session)
            .unwrap_err();
        assert!(matches!(error, SessionRuntimeError::FixtureRuntime(_)));

        Ok(())
    }

    #[test]
    fn run_control_compatibility_task_covers_valid_missing_goal_and_noop_paths()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-run-control-compatibility")?;
        write_execution_profile(workspace.as_path(), None)?;
        write_source_target(workspace.as_path())?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());

        let mut session = sample_flow_selected_session(workspace.as_path())?;
        assert!(runtime.flow_selected_goal_plan_uses_compatibility_step(&session)?);
        runtime.ensure_flow_selected_compatibility_task(&mut session)?;
        let task_id = session
            .active_task
            .as_ref()
            .map(|task| task.id.clone())
            .ok_or("missing compatibility task")?;
        assert_eq!(session.latest_status, SessionStatus::Planned);
        assert!(session.latest_terminal_reason.is_none());
        assert!(session.latest_trace_ref.is_none());

        session.latest_terminal_reason = Some(TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "preserve the existing task",
            None,
        ));
        runtime.ensure_flow_selected_compatibility_task(&mut session)?;
        assert_eq!(
            session.active_task.as_ref().map(|task| task.id.as_str()),
            Some(task_id.as_str())
        );
        assert_eq!(
            session.latest_terminal_reason.as_ref().map(|reason| reason.message.as_str()),
            Some("preserve the existing task")
        );

        let mut missing_goal_session = sample_flow_selected_session(workspace.as_path())?;
        missing_goal_session.goal = None;
        let error =
            runtime.ensure_flow_selected_compatibility_task(&mut missing_goal_session).unwrap_err();
        assert!(matches!(error, SessionRuntimeError::MissingGoal));

        Ok(())
    }

    #[test]
    fn run_control_refresh_governance_state_covers_non_refreshable_paths()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-run-control-refresh")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());

        let mut no_task_session = sample_session(workspace.as_path(), None, SessionStatus::Planned);
        assert!(!runtime.refresh_governance_state(&mut no_task_session)?);

        write_execution_profile(workspace.as_path(), None)?;
        write_source_target(workspace.as_path())?;
        let mut no_stage_session = sample_flow_selected_session(workspace.as_path())?;
        runtime.ensure_flow_selected_compatibility_task(&mut no_stage_session)?;
        assert!(!runtime.refresh_governance_state(&mut no_stage_session)?);

        let mut no_governance_session = sample_flow_selected_session(workspace.as_path())?;
        runtime.ensure_flow_selected_compatibility_task(&mut no_governance_session)?;
        no_governance_session
            .active_task
            .as_mut()
            .ok_or("missing task for no-governance refresh test")?
            .context
            .set_latest_governance_stage(&sample_governed_stage_record(
                GovernanceLifecycleState::AwaitingApproval,
            ))?;
        assert!(!runtime.refresh_governance_state(&mut no_governance_session)?);

        let policyless_workspace = temp_workspace("boundline-run-control-refresh-policyless")?;
        write_execution_profile(
            policyless_workspace.as_path(),
            Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Local,
                canon: None,
                stages: Vec::new(),
            }),
        )?;
        write_source_target(policyless_workspace.as_path())?;
        let policyless_runtime = SessionRuntime::for_workspace(policyless_workspace.as_path());
        let mut policyless_session = sample_flow_selected_session(policyless_workspace.as_path())?;
        policyless_runtime.ensure_flow_selected_compatibility_task(&mut policyless_session)?;
        policyless_session
            .active_task
            .as_mut()
            .ok_or("missing task for policyless refresh test")?
            .context
            .set_latest_governance_stage(&sample_governed_stage_record(
                GovernanceLifecycleState::AwaitingApproval,
            ))?;
        assert!(!policyless_runtime.refresh_governance_state(&mut policyless_session)?);

        Ok(())
    }

    fn sample_flow_selected_session(
        workspace: &Path,
    ) -> Result<ActiveSessionRecord, Box<dyn Error>> {
        let active_flow = built_in_flow("bug-fix").ok_or("missing bug-fix flow")?.initial_state();
        Ok(ActiveSessionRecord {
            active_flow: Some(active_flow),
            active_execution_run_id: None,
            ..sample_session(workspace, Some(sample_goal_plan()?), SessionStatus::Planned)
        })
    }

    fn sample_session(
        workspace: &Path,
        goal_plan: Option<GoalPlan>,
        latest_status: SessionStatus,
    ) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: SESSION_ID.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some(GOAL_TEXT.to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: UPDATED_AT,
            updated_at: UPDATED_AT,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        }
    }

    fn sample_goal_plan() -> Result<GoalPlan, Box<dyn Error>> {
        GoalPlan::new(
            GOAL_TEXT,
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Repair arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
                depends_on: None,
            }],
        )
        .map_err(Into::into)
    }

    fn sample_governed_stage_record(
        lifecycle_state: GovernanceLifecycleState,
    ) -> GovernedStageRecord {
        GovernedStageRecord {
            stage_key: "run:implementation".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state,
            required: false,
            autopilot_enabled: true,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: "attempt-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: None,
            decision_ref: None,
            stage_council: None,
            blocked_reason: None,
        }
    }

    fn write_execution_profile(
        workspace: &Path,
        governance: Option<GovernanceProfile>,
    ) -> Result<(), Box<dyn Error>> {
        let execution_path = crate::fixture::execution_manifest_path(workspace);
        if let Some(parent) = execution_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let profile = WorkspaceExecutionProfile {
            name: "run-control-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "attempt-1".to_string(),
                summary: "apply change".to_string(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "before".to_string(),
                    replace: "after".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance,
            review: None,
            legacy_source: None,
        };
        fs::write(execution_path, serde_json::to_vec_pretty(&profile)?)?;
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
