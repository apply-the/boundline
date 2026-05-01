use synod::domain::goal_plan::{GoalPlan, PlannedTask};
use synod::domain::session::{
    ActiveSessionRecord, SessionStatus, SessionStatusView, SessionValidationError,
    execution_path_text,
};
use synod::domain::workflow::{WorkflowLifecycleState, WorkflowPhase, WorkflowProgressState};

fn build_goal_plan() -> GoalPlan {
    GoalPlan::new(
        "Deliver a named workflow",
        vec![PlannedTask {
            task_id: "workflow-task-1".to_string(),
            description: "Compile the named workflow onto the session runtime".to_string(),
            target: "src/orchestrator/session_runtime.rs".to_string(),
            expected_outcome: Some("workflow progress is persisted".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap()
    .with_workflow_progress(WorkflowProgressState {
        workflow_name: "default".to_string(),
        lifecycle_state: WorkflowLifecycleState::Active,
        current_phase: Some(WorkflowPhase::Plan),
        completed_phases: vec![WorkflowPhase::Capture],
        blocked_reason: None,
        next_action: Some("synod workflow resume default".to_string()),
        routing_summary: Some("native session plan is active".to_string()),
    })
}

fn build_record() -> ActiveSessionRecord {
    let goal_plan = build_goal_plan();
    let workflow_progress = goal_plan.workflow_progress.clone();

    ActiveSessionRecord {
        session_id: "session-workflow-projection".to_string(),
        workspace_ref: "/tmp/synod-workflow-projection".to_string(),
        goal: Some("Deliver a named workflow".to_string()),
        authored_brief: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(goal_plan),
        workflow_progress,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
    }
}

fn build_view(record: &ActiveSessionRecord) -> SessionStatusView {
    SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        goal: record.goal.clone(),
        authored_input_summary: None,
        authored_input_sources: None,
        authored_input_deduplicated_sources: None,
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: None,
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: record
            .goal_plan
            .as_ref()
            .map(|goal_plan| goal_plan.flow_state().summary_text()),
        active_workflow: record.active_workflow_name(),
        workflow_phase: record.active_workflow_phase_text(),
        workflow_next_action: record.active_workflow_next_action(),
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: None,
        current_step_id: None,
        current_step_index: None,
        latest_status: record.latest_status,
        execution_path: execution_path_text(record),
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
        latest_review_trigger: None,
        latest_review_vote: None,
        latest_review_outcome: None,
        latest_review_headline: None,
        latest_governance_stage: None,
        latest_governance_runtime: None,
        latest_governance_mode: None,
        latest_governance_run_ref: None,
        latest_governance_state: None,
        latest_governance_blocked_reason: None,
        latest_governance_packet_ref: None,
        latest_governance_packet_source_stage: None,
        latest_governance_packet_binding_reason: None,
        latest_governance_approval: None,
        latest_governance_decision: None,
        latest_governance_candidates: None,
        governance_next_action: None,
        next_command: Some("synod workflow resume default".to_string()),
        explanation: "workflow projection is consistent".to_string(),
    }
}

#[test]
fn goal_plan_surfaces_workflow_identity_phase_and_next_action() {
    let goal_plan = build_goal_plan();

    assert_eq!(goal_plan.workflow_name().as_deref(), Some("default"));
    assert_eq!(goal_plan.workflow_phase_text().as_deref(), Some("plan"));
    assert_eq!(goal_plan.workflow_next_action().as_deref(), Some("synod workflow resume default"));
}

#[test]
fn session_status_view_accepts_matching_workflow_projection() {
    let record = build_record();
    record.validate().unwrap();

    let view = build_view(&record);
    view.validate(&record).unwrap();
}

#[test]
fn session_status_view_rejects_workflow_phase_mismatch() {
    let record = build_record();
    let mut view = build_view(&record);
    view.workflow_phase = Some("run".to_string());

    assert!(matches!(
        view.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewWorkflowPhaseMismatch { .. }
    ));
}

#[test]
fn session_status_view_accepts_session_owned_workflow_progress_without_goal_plan() {
    let record = ActiveSessionRecord {
        session_id: "session-workflow-session-owned".to_string(),
        workspace_ref: "/tmp/synod-workflow-session-owned".to_string(),
        goal: None,
        authored_brief: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Paused,
            current_phase: Some(WorkflowPhase::Capture),
            completed_phases: Vec::new(),
            blocked_reason: Some(
                "workflow is waiting for a captured goal before it can continue".to_string(),
            ),
            next_action: Some(
                "synod capture --workspace /tmp/synod-workflow-session-owned --goal <goal>"
                    .to_string(),
            ),
            routing_summary: Some("routing: blocked (session_state) - session has no goal plan or compatibility task to route".to_string()),
        }),
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
    };

    let view = build_view(&record);
    view.validate(&record).unwrap();
    assert_eq!(view.active_workflow.as_deref(), Some("default"));
    assert_eq!(view.workflow_phase.as_deref(), Some("capture"));
    assert!(
        view.workflow_next_action
            .as_deref()
            .unwrap()
            .contains("synod capture --workspace /tmp/synod-workflow-session-owned --goal <goal>")
    );
}
