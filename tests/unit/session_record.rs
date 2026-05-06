use boundline::domain::brief::normalize_inputs;
use boundline::domain::cluster::ClusterSessionProjection;
use boundline::domain::flow::built_in_flow;
use boundline::domain::goal_plan::{GoalPlan, PlannedTask};
use boundline::domain::limits::RunLimits;
use boundline::domain::plan::Plan;
use boundline::domain::session::{
    ActiveSessionRecord, SessionCommand, SessionStatus, SessionStatusView, SessionTransition,
    SessionValidationError,
};
use boundline::domain::step::Step;
use boundline::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};
use serde_json::json;
use std::fs;

fn build_goal_plan() -> GoalPlan {
    let mut plan = GoalPlan::new(
        "Deliver a session-backed CLI",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Analyze the current CLI path".to_string(),
            target: "/tmp/boundline-session-record/src/lib.rs".to_string(),
            expected_outcome: Some("routing gaps understood".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap();
    plan.confirm().unwrap();
    plan
}

fn build_task(workspace_ref: &str) -> Task {
    let request = TaskRunRequest {
        goal: "Deliver a session-backed CLI".to_string(),
        input: json!({"ticket": "SESSION-1"}),
        session_id: "session-1".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    };

    let plan =
        Plan::new(vec![Step::decision("analyze", json!({"phase": "bootstrap"})).unwrap()]).unwrap();

    Task::new("task-1", &request, plan).unwrap()
}

#[test]
fn session_record_round_trips_and_status_values_serialize() {
    let task = build_task("/tmp/boundline-session-record");
    let record = ActiveSessionRecord {
        session_id: "session-1".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
        active_task: Some(task),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(
            "/tmp/boundline-session-record/.boundline/traces/task-1.json".to_string(),
        ),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    record.validate().unwrap();

    let encoded = serde_json::to_value(&record).unwrap();
    assert_eq!(encoded["latest_status"], json!("planned"));

    let decoded: ActiveSessionRecord = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, record);

    let transition = SessionTransition {
        trigger_command: SessionCommand::Plan,
        from_status: Some(SessionStatus::GoalCaptured),
        to_status: SessionStatus::Planned,
        trace_ref: record.latest_trace_ref.clone(),
        reason: "planned from the captured goal".to_string(),
    };
    transition.validate(&record).unwrap();

    let view = SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        goal: record.goal.clone(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
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
        active_flow: Some("bug-fix".to_string()),
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: Some("investigate".to_string()),
        current_stage_index: Some(0),
        total_stages: Some(3),
        plan_revision: Some(0),
        current_step_id: Some("analyze".to_string()),
        current_step_index: Some(0),
        latest_status: SessionStatus::Planned,
        execution_path: boundline::domain::session::execution_path_text(&record),
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
        latest_exhaustion_reason: None,
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
        next_command: Some("boundline step".to_string()),
        explanation: "the active plan is ready for explicit execution".to_string(),
        ..Default::default()
    };
    view.validate(&record).unwrap();
}

#[test]
fn session_record_validation_rejects_workspace_mismatches_and_external_traces() {
    let task = build_task("/tmp/other-workspace");
    let record = ActiveSessionRecord {
        session_id: "session-2".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(task),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(
            "/tmp/boundline-session-record/.boundline/traces/task-1.json".to_string(),
        ),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    assert_eq!(
        record.validate().unwrap_err(),
        SessionValidationError::TaskWorkspaceMismatch {
            expected: "/tmp/boundline-session-record".to_string(),
            actual: "/tmp/other-workspace".to_string(),
        }
    );
}

#[test]
fn session_record_validation_allows_cluster_member_tasks_when_projection_is_present() {
    let mut task = build_task("/tmp/cluster-member");
    task.context
        .set_cluster_session_projection(&ClusterSessionProjection {
            cluster_id: "cluster-1".to_string(),
            primary_workspace_ref: "/tmp/boundline-session-record".to_string(),
            member_workspace_refs: vec![
                "/tmp/boundline-session-record".to_string(),
                "/tmp/cluster-member".to_string(),
            ],
            started_from_command: "run".to_string(),
            updated_at: 20,
        })
        .unwrap();
    let record = ActiveSessionRecord {
        session_id: "session-cluster".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(task),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(
            "/tmp/boundline-session-record/.boundline/traces/task-1.json".to_string(),
        ),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    record.validate().unwrap();
}

#[test]
fn terminal_session_requires_terminal_reason_and_consistent_view() {
    let mut task = build_task("/tmp/boundline-session-record");
    task.apply_terminal(
        TaskStatus::Succeeded,
        TerminalReason::new(
            boundline::domain::limits::TerminalCondition::GoalSatisfied,
            "done",
            None,
        ),
    );

    let record = ActiveSessionRecord {
        session_id: "session-terminal".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(task),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Succeeded,
        latest_terminal_reason: None,
        latest_trace_ref: Some(
            "/tmp/boundline-session-record/.boundline/traces/task-1.json".to_string(),
        ),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    assert_eq!(
        record.validate().unwrap_err(),
        SessionValidationError::MissingTerminalReason(SessionStatus::Succeeded)
    );
}

#[test]
fn goal_captured_sessions_require_a_goal_but_invalid_sessions_can_clear_context() {
    let missing_goal = ActiveSessionRecord {
        session_id: "session-goal-captured".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: None,
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    assert_eq!(
        missing_goal.validate().unwrap_err(),
        SessionValidationError::MissingGoal(SessionStatus::GoalCaptured)
    );

    let invalid = ActiveSessionRecord { latest_status: SessionStatus::Invalid, ..missing_goal };

    invalid.validate().unwrap();
}

#[test]
fn invalid_flow_state_is_rejected_by_session_validation() {
    let record = ActiveSessionRecord {
        session_id: "session-flow".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: Some(boundline::domain::flow::SessionFlowState {
            flow_name: "bug-fix".to_string(),
            current_stage_id: "verify".to_string(),
            current_stage_index: 0,
            total_stages: 3,
        }),
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    assert!(matches!(record.validate().unwrap_err(), SessionValidationError::InvalidFlowState(_)));
}

#[test]
fn goal_captured_status_view_can_project_clarification_fields_from_authored_brief() {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-session-record-clarification-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    let bundle = normalize_inputs(
        &workspace,
        Some("Improve the platform docs and fix whatever tests are broken"),
        &[],
    )
    .unwrap();

    let record = ActiveSessionRecord {
        session_id: "session-clarification".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some(bundle.render_goal_text()),
        authored_brief: Some(bundle.clone()),
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    let view = SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        goal: record.goal.clone(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        authored_input_summary: Some(bundle.summary_text()),
        authored_input_sources: Some(bundle.ordered_source_labels()),
        authored_input_deduplicated_sources: None,
        clarification_headline: bundle.clarification_headline(),
        clarification_prompt: bundle.clarification_prompt(),
        clarification_missing_fields: bundle.clarification_missing_fields(),
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: None,
        current_step_id: None,
        current_step_index: None,
        latest_status: SessionStatus::GoalCaptured,
        execution_path: boundline::domain::session::execution_path_text(&record),
        latest_trace_ref: None,
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
        latest_exhaustion_reason: None,
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
        next_command: Some("boundline capture --goal <narrower goal>".to_string()),
        explanation: "clarification is required before planning can continue".to_string(),
        ..Default::default()
    };

    view.validate(&record).unwrap();

    record.validate().unwrap();
}

#[test]
fn planned_session_with_goal_plan_and_no_active_task_is_valid() {
    let record = ActiveSessionRecord {
        session_id: "session-native-plan".to_string(),
        workspace_ref: "/tmp/boundline-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(build_goal_plan()),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
    };

    record.validate().unwrap();
}
