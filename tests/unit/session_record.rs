use serde_json::json;
use synod::domain::flow::built_in_flow;
use synod::domain::limits::RunLimits;
use synod::domain::plan::Plan;
use synod::domain::session::{
    ActiveSessionRecord, SessionCommand, SessionStatus, SessionStatusView, SessionTransition,
    SessionValidationError,
};
use synod::domain::step::Step;
use synod::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};

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
    let task = build_task("/tmp/synod-session-record");
    let record = ActiveSessionRecord {
        session_id: "session-1".to_string(),
        workspace_ref: "/tmp/synod-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
        active_task: Some(task),
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some("/tmp/synod-session-record/.synod/traces/task-1.json".to_string()),
        created_at: 10,
        updated_at: 20,
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
        active_flow: Some("bug-fix".to_string()),
        current_stage_id: Some("investigate".to_string()),
        current_stage_index: Some(0),
        total_stages: Some(3),
        plan_revision: Some(0),
        current_step_id: Some("analyze".to_string()),
        current_step_index: Some(0),
        latest_status: SessionStatus::Planned,
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_changed_files: None,
        latest_validation_status: None,
        next_command: Some("synod step".to_string()),
        explanation: "the active plan is ready for explicit execution".to_string(),
    };
    view.validate(&record).unwrap();
}

#[test]
fn session_record_validation_rejects_workspace_mismatches_and_external_traces() {
    let task = build_task("/tmp/other-workspace");
    let record = ActiveSessionRecord {
        session_id: "session-2".to_string(),
        workspace_ref: "/tmp/synod-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        active_flow: None,
        active_task: Some(task),
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some("/tmp/synod-session-record/.synod/traces/task-1.json".to_string()),
        created_at: 10,
        updated_at: 20,
    };

    assert_eq!(
        record.validate().unwrap_err(),
        SessionValidationError::TaskWorkspaceMismatch {
            expected: "/tmp/synod-session-record".to_string(),
            actual: "/tmp/other-workspace".to_string(),
        }
    );
}

#[test]
fn terminal_session_requires_terminal_reason_and_consistent_view() {
    let mut task = build_task("/tmp/synod-session-record");
    task.apply_terminal(
        TaskStatus::Succeeded,
        TerminalReason::new(synod::domain::limits::TerminalCondition::GoalSatisfied, "done", None),
    );

    let record = ActiveSessionRecord {
        session_id: "session-terminal".to_string(),
        workspace_ref: "/tmp/synod-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        active_flow: None,
        active_task: Some(task),
        latest_status: SessionStatus::Succeeded,
        latest_terminal_reason: None,
        latest_trace_ref: Some("/tmp/synod-session-record/.synod/traces/task-1.json".to_string()),
        created_at: 10,
        updated_at: 20,
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
        workspace_ref: "/tmp/synod-session-record".to_string(),
        goal: None,
        active_flow: None,
        active_task: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
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
        workspace_ref: "/tmp/synod-session-record".to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        active_flow: Some(synod::domain::flow::SessionFlowState {
            flow_name: "bug-fix".to_string(),
            current_stage_id: "verify".to_string(),
            current_stage_index: 0,
            total_stages: 3,
        }),
        active_task: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
    };

    assert!(matches!(record.validate().unwrap_err(), SessionValidationError::InvalidFlowState(_)));
}
