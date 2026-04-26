use serde_json::json;
use synod::domain::limits::RunLimits;
use synod::domain::plan::Plan;
use synod::domain::step::{ErrorInfo, Recoverability, Step, StepExecutionResult};
use synod::domain::task::{Task, TaskRunRequest};
use synod::orchestrator::planner::{CallbackPlanner, Planner, PlanningError, StaticPlanner};

fn build_task() -> Task {
    let request = TaskRunRequest {
        goal: "Replan a bounded task".to_string(),
        input: json!({"ticket": "BUG-10"}),
        session_id: "session-planner".to_string(),
        workspace_ref: "/tmp/synod-planner".to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    };
    let plan = Plan::new(vec![Step::tool("verify", "tester", json!({})).unwrap()]).unwrap();
    Task::new("task-planner", &request, plan).unwrap()
}

#[test]
fn static_planner_returns_initial_plan_and_queued_replans_in_order() {
    let initial_plan = Plan::new(vec![Step::decision("analyze", json!({})).unwrap()]).unwrap();
    let queued_replans = vec![
        vec![Step::decision("retry", json!({"phase": 1})).unwrap()],
        vec![Step::decision("finish", json!({"phase": 2})).unwrap()],
    ];
    let planner = StaticPlanner::with_replans(initial_plan.clone(), queued_replans);

    let task = build_task();
    let failure = StepExecutionResult::failure(
        ErrorInfo::new("bad_output", "need a new plan"),
        Recoverability::ReplanRequired,
    );
    let current_step = task.plan.current_step().unwrap().clone();

    let created_plan = planner
        .create_initial_plan(
            &TaskRunRequest {
                goal: "goal".to_string(),
                input: json!({}),
                session_id: "session".to_string(),
                workspace_ref: "/tmp/workspace".to_string(),
                limits: RunLimits::default(),
                initial_context: None,
            },
            &task.context,
        )
        .unwrap();
    assert_eq!(created_plan, initial_plan);

    let first_replan = planner.replan(&task, &current_step, &failure).unwrap();
    let second_replan = planner.replan(&task, &current_step, &failure).unwrap();
    assert_eq!(first_replan[0].id, "retry");
    assert_eq!(second_replan[0].id, "finish");
}

#[test]
fn static_planner_reports_when_no_replan_is_available() {
    let planner =
        StaticPlanner::new(Plan::new(vec![Step::decision("analyze", json!({})).unwrap()]).unwrap());
    let task = build_task();
    let failure = StepExecutionResult::failure(
        ErrorInfo::new("bad_output", "need a new plan"),
        Recoverability::ReplanRequired,
    );
    let current_step = task.plan.current_step().unwrap().clone();

    match planner.replan(&task, &current_step, &failure).unwrap_err() {
        PlanningError::ReplanUnavailable(message) => {
            assert!(message.contains("no replacement plan"));
        }
        other => panic!("expected missing replan error, got {other:?}"),
    }

    let invalid = PlanningError::InvalidPlan("missing steps".to_string());
    assert_eq!(invalid.to_string(), "planner returned an invalid plan: missing steps");
}

#[test]
fn callback_planner_supports_dynamic_initial_plans_and_replans() {
    let initial_plan =
        Plan::new(vec![Step::decision("dynamic-analyze", json!({})).unwrap()]).unwrap();
    let planner = CallbackPlanner::new(
        {
            let initial_plan = initial_plan.clone();
            move |request, _context| {
                assert_eq!(request.goal, "goal");
                Ok(initial_plan.clone())
            }
        },
        move |_task, failed_step, failure| {
            assert_eq!(failed_step.id, "verify");
            assert_eq!(failure.recoverability, Recoverability::ReplanRequired);
            Ok(vec![Step::decision("dynamic-replan", json!({"phase": 2})).unwrap()])
        },
    );

    let task = build_task();
    let failure = StepExecutionResult::failure(
        ErrorInfo::new("bad_output", "need a new plan"),
        Recoverability::ReplanRequired,
    );
    let current_step = task.plan.current_step().unwrap().clone();

    let created_plan = planner
        .create_initial_plan(
            &TaskRunRequest {
                goal: "goal".to_string(),
                input: json!({}),
                session_id: "session".to_string(),
                workspace_ref: "/tmp/workspace".to_string(),
                limits: RunLimits::default(),
                initial_context: None,
            },
            &task.context,
        )
        .unwrap();
    let replanned = planner.replan(&task, &current_step, &failure).unwrap();

    assert_eq!(created_plan, initial_plan);
    assert_eq!(replanned[0].id, "dynamic-replan");
}
