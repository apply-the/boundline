use serde_json::json;
use synod::domain::limits::{RunLimits, TerminalCondition};
use synod::domain::plan::Plan;
use synod::domain::step::{ErrorInfo, Recoverability, Step, StepExecutionResult};
use synod::domain::task::{Task, TaskRunRequest};
use synod::orchestrator::recovery::{RecoveryDecision, decide_recovery};

fn build_task(limits: RunLimits) -> Task {
    let request = TaskRunRequest {
        goal: "Recover a failing step".to_string(),
        input: json!({"ticket": "BUG-5"}),
        session_id: "session-recovery".to_string(),
        workspace_ref: "/tmp/synod-recovery".to_string(),
        limits,
        initial_context: None,
    };
    let plan = Plan::new(vec![Step::tool("verify", "tester", json!({})).unwrap()]).unwrap();
    Task::new("task-recovery", &request, plan).unwrap()
}

#[test]
fn retries_when_retry_budget_remains() {
    let task = build_task(RunLimits::default());
    let step = task.plan.current_step().unwrap().clone();
    let result = StepExecutionResult::failure(
        ErrorInfo::new("retryable", "temporary failure"),
        Recoverability::Retryable,
    );

    let decision = decide_recovery(&task, &step, &result);
    assert!(matches!(decision, RecoveryDecision::Retry { .. }));
}

#[test]
fn exhausts_when_retry_budget_is_consumed() {
    let mut task = build_task(RunLimits { max_retries: 0, ..RunLimits::default() });
    task.retry_count = 0;
    let step = task.plan.current_step().unwrap().clone();
    let result = StepExecutionResult::failure(
        ErrorInfo::new("retryable", "temporary failure"),
        Recoverability::Retryable,
    );

    let decision = decide_recovery(&task, &step, &result);
    match decision {
        RecoveryDecision::Terminate(reason) => {
            assert_eq!(reason.condition, TerminalCondition::RetryBudgetExhausted);
        }
        other => panic!("expected retry exhaustion, got {other:?}"),
    }
}

#[test]
fn requests_replanning_when_policy_marks_failure_as_replan_required() {
    let task = build_task(RunLimits::default());
    let step = task.plan.current_step().unwrap().clone();
    let result = StepExecutionResult::failure(
        ErrorInfo::new("needs_replan", "current plan is invalid"),
        Recoverability::ReplanRequired,
    );

    let decision = decide_recovery(&task, &step, &result);
    assert!(matches!(decision, RecoveryDecision::Replan { .. }));
}

#[test]
fn continues_when_step_execution_succeeds() {
    let task = build_task(RunLimits::default());
    let step = task.plan.current_step().unwrap().clone();
    let result = StepExecutionResult::success(json!({"tests_passed": true}));

    let decision = decide_recovery(&task, &step, &result);
    assert_eq!(decision, RecoveryDecision::Continue);
}

#[test]
fn exhausts_when_replan_budget_is_consumed() {
    let task = build_task(RunLimits { max_replans: 0, ..RunLimits::default() });
    let step = task.plan.current_step().unwrap().clone();
    let result = StepExecutionResult::failure(
        ErrorInfo::new("needs_replan", "current plan is invalid"),
        Recoverability::ReplanRequired,
    );

    let decision = decide_recovery(&task, &step, &result);
    match decision {
        RecoveryDecision::Terminate(reason) => {
            assert_eq!(reason.condition, TerminalCondition::ReplanBudgetExhausted);
            assert_eq!(reason.details.unwrap()["step_id"], json!(step.id));
        }
        other => panic!("expected replan exhaustion, got {other:?}"),
    }
}

#[test]
fn terminates_unrecoverable_failures_with_error_details() {
    let task = build_task(RunLimits::default());
    let step = task.plan.current_step().unwrap().clone();
    let result = StepExecutionResult::failure(
        ErrorInfo::new("fatal", "cannot continue").with_details(json!({"exit_code": 2})),
        Recoverability::Terminal,
    );

    let decision = decide_recovery(&task, &step, &result);
    match decision {
        RecoveryDecision::Terminate(reason) => {
            assert_eq!(reason.condition, TerminalCondition::UnrecoverableError);
            assert_eq!(reason.details.unwrap()["code"], json!("fatal"));
        }
        other => panic!("expected terminal failure, got {other:?}"),
    }
}
