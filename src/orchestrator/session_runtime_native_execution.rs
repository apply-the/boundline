use serde_json::{Map, json};

use crate::domain::cluster::ClusteredExecutionKind;
use crate::domain::decision::Decision;
use crate::domain::goal_plan::GoalPlan;
use crate::domain::limits::TerminalCondition;
use crate::domain::session::{ActiveSessionRecord, SessionStatus};
use crate::domain::task::{TaskRunResponse, TaskStatus};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEvent, TraceEventType, current_timestamp_millis};
use crate::fixture::FixtureRuntime;
use crate::orchestrator::guidance_runtime::execute_guardians_for_phase;
use crate::orchestrator::review_trace::record_reasoning_profile_events;
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};

use super::{
    LATEST_CHANGED_FILES_KEY, LATEST_VALIDATION_STATUS_KEY, NativePersistenceInput, SessionRuntime,
    SessionRuntimeError, VALIDATION_STATUS_FAILED, VALIDATION_STATUS_PASSED,
    apply_checkpoint_projection_to_context, checkpoint_event_payload,
    session_status_for_task_status,
};

impl SessionRuntime {
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
        trace.finalize(terminal_status, terminal_reason.clone());
        let trace_location = self.persist_trace(&session.session_id, &mut trace)?;
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use serde_json::{Map, Value, json};
    use uuid::Uuid;

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
