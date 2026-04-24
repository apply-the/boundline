use serde_json::{Map, json};
use synod::domain::limits::{RunLimits, TerminalCondition};
use synod::domain::plan::{Plan, PlanError, PlanStatus};
use synod::domain::step::{Recoverability, Step, StepError};
use synod::domain::task::{Task, TaskRequestError, TaskRunRequest, TaskStatus, TerminalReason};

fn build_request() -> TaskRunRequest {
    TaskRunRequest {
        goal: "Ship a bounded change".to_string(),
        input: json!({"ticket": "BUG-9"}),
        session_id: "session-task".to_string(),
        workspace_ref: "/tmp/synod-task".to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    }
}

#[test]
fn task_status_marks_only_terminal_states_as_terminal() {
    assert!(!TaskStatus::Planned.is_terminal());
    assert!(!TaskStatus::Running.is_terminal());
    assert!(TaskStatus::Succeeded.is_terminal());
    assert!(TaskStatus::Failed.is_terminal());
    assert!(TaskStatus::Exhausted.is_terminal());
    assert!(TaskStatus::Aborted.is_terminal());
}

#[test]
fn task_request_validation_rejects_missing_required_fields() {
    let mut request = build_request();
    request.goal = "   ".to_string();
    assert_eq!(request.validate().unwrap_err(), TaskRequestError::EmptyGoal);

    let mut request = build_request();
    request.session_id = " ".to_string();
    assert_eq!(request.validate().unwrap_err(), TaskRequestError::MissingSessionId);

    let mut request = build_request();
    request.workspace_ref = " ".to_string();
    assert_eq!(request.validate().unwrap_err(), TaskRequestError::MissingWorkspaceRef);

    let mut request = build_request();
    request.limits.max_steps = 0;
    match request.validate().unwrap_err() {
        TaskRequestError::InvalidRunLimits(message) => {
            assert!(message.contains("max_steps"));
        }
        other => panic!("expected invalid run limits, got {other:?}"),
    }
}

#[test]
fn task_new_preserves_initial_context_and_updates_terminal_state() {
    let mut request = build_request();
    let mut initial_context = Map::new();
    initial_context.insert("ticket".to_string(), json!("BUG-9"));
    request.initial_context = Some(initial_context);

    let plan =
        Plan::new(vec![Step::decision("evaluate", json!({"phase": "check"})).unwrap()]).unwrap();
    let mut task = Task::new("task-9", &request, plan).unwrap();

    assert_eq!(task.context.state["ticket"], json!("BUG-9"));
    assert_eq!(task.status, TaskStatus::Planned);

    task.mark_running();
    assert_eq!(task.status, TaskStatus::Running);

    let reason = TerminalReason::new(
        TerminalCondition::GoalSatisfied,
        "done",
        Some(json!({"source": "unit-test"})),
    );
    task.apply_terminal(TaskStatus::Succeeded, reason.clone());
    assert_eq!(task.status, TaskStatus::Succeeded);
    assert_eq!(task.terminal_reason, Some(reason));
}

#[test]
fn task_new_rejects_invalid_plan_and_converts_step_errors() {
    let request = build_request();
    let invalid_plan = Plan {
        revision: 0,
        steps: vec![Step::decision("evaluate", json!({})).unwrap()],
        current_step_index: 2,
        status: PlanStatus::Active,
    };

    match Task::new("task-invalid", &request, invalid_plan).unwrap_err() {
        TaskRequestError::InvalidPlan(message) => {
            assert!(message.contains("current_step_index"));
        }
        other => panic!("expected invalid plan, got {other:?}"),
    }

    assert_eq!(Step::agent("", "analyzer", json!({})).unwrap_err(), StepError::MissingId);
}

#[test]
fn plan_advance_and_replace_remaining_steps_update_revision_and_cursor() {
    let mut plan = Plan::new(vec![
        Step::decision("analyze", json!({})).unwrap(),
        Step::decision("verify", json!({})).unwrap(),
        Step::decision("finalize", json!({})).unwrap(),
    ])
    .unwrap();

    assert_eq!(plan.current_step().unwrap().id, "analyze");
    plan.current_step_mut().unwrap().mark_running();
    assert_eq!(plan.current_step().unwrap().attempt_count, 1);

    plan.advance();
    assert_eq!(plan.current_step().unwrap().id, "verify");

    let revision = plan
        .replace_remaining_steps(vec![
            Step::decision("reworked-verify", json!({})).unwrap(),
            Step::decision("reworked-finalize", json!({})).unwrap(),
        ])
        .unwrap();

    assert_eq!(revision.from_revision, 0);
    assert_eq!(revision.to_revision, 1);
    assert_eq!(revision.replaced_step_ids, vec!["finalize".to_string()]);
    assert_eq!(revision.added_step_ids.len(), 2);
    assert_eq!(plan.current_step().unwrap().id, "reworked-verify");
    assert_eq!(plan.status, PlanStatus::Active);

    plan.advance();
    plan.advance();
    assert_eq!(plan.status, PlanStatus::Completed);
}

#[test]
fn plan_validate_and_replace_remaining_steps_reject_invalid_shapes() {
    let plan = Plan {
        revision: 0,
        steps: vec![Step::decision("only-step", json!({})).unwrap()],
        current_step_index: 4,
        status: PlanStatus::Active,
    };
    assert_eq!(
        plan.validate().unwrap_err(),
        PlanError::InvalidCurrentStepIndex { index: 4, len: 1 }
    );

    let mut completed_plan = Plan::new(vec![Step::decision("done", json!({})).unwrap()]).unwrap();
    completed_plan.advance();
    assert_eq!(
        completed_plan.replace_remaining_steps(Vec::new()).unwrap_err(),
        PlanError::NoExecutableSteps
    );
}

#[test]
fn terminal_reason_keeps_condition_message_and_details() {
    let details = json!({"attempts": 2, "recoverability": Recoverability::Retryable});
    let reason =
        TerminalReason::new(TerminalCondition::StepLimitExceeded, "stopped", Some(details.clone()));

    assert_eq!(reason.condition, TerminalCondition::StepLimitExceeded);
    assert_eq!(reason.message, "stopped");
    assert_eq!(reason.details, Some(details));
}
