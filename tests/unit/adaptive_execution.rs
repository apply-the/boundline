use boundline::domain::execution::ValidationRecord;
use boundline::domain::step::{ErrorInfo, Recoverability, StepExecutionResult};
use boundline::domain::task::Task;
use boundline::fixture::{
    build_fixture_plan_for_goal, build_fixture_runtime, build_task_request,
    load_workspace_execution_profile,
};
use boundline::orchestrator::planner::Planner;
use serde_json::json;

use crate::workspace_fixture::{
    temp_adaptive_fixture_workspace, temp_adaptive_guided_replanning_workspace,
    temp_adaptive_ordering_boundary_workspace, temp_adaptive_replanning_workspace,
};

#[test]
fn adaptive_profile_builds_a_goal_aware_initial_plan_without_authored_attempts() {
    let workspace = temp_adaptive_fixture_workspace("boundline-adaptive-plan");
    let profile = load_workspace_execution_profile(&workspace).unwrap();
    let plan = build_fixture_plan_for_goal(&workspace, None, "Fix the failing add test").unwrap();

    assert!(profile.attempts.is_empty());
    assert!(profile.adaptive.is_some());
    assert_eq!(plan.steps[0].id, "analyze");
    assert_eq!(plan.steps[1].id, "code-adaptive-attempt-1");
    assert_eq!(plan.steps[2].id, "verify-adaptive-attempt-1");
    assert_eq!(plan.steps[1].input["attempt_id"], json!("adaptive-attempt-1"));
    assert_eq!(
        plan.steps[0].input["selection_headline"],
        json!("selected src/lib.rs via arithmetic_swap for adaptive delivery")
    );
    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["path"], json!("src/lib.rs"));
    assert!(
        plan.steps[1].input["candidate_signature"]
            .as_str()
            .is_some_and(|signature| signature.contains("src/lib.rs"))
    );
}

#[test]
fn adaptive_profile_generates_a_deterministic_first_candidate_for_replanning() {
    let workspace = temp_adaptive_replanning_workspace("boundline-adaptive-replan-plan");
    let plan =
        build_fixture_plan_for_goal(&workspace, None, "Recover after validation fails").unwrap();

    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["find"], json!(" * "));
    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["replace"], json!(" - "));
}

#[test]
fn adaptive_replan_uses_latest_validation_record_to_shift_selected_target() {
    let workspace =
        temp_adaptive_guided_replanning_workspace("boundline-adaptive-guided-replan-plan");
    let runtime = build_fixture_runtime(&workspace).unwrap();
    let request = build_task_request(
        &workspace,
        "Recover after validation points to helper.rs",
        "session-adaptive-guided",
        None,
        None,
    )
    .unwrap();
    let plan = build_fixture_plan_for_goal(
        &workspace,
        None,
        "Recover after validation points to helper.rs",
    )
    .unwrap();
    let mut task = Task::new("task-adaptive-guided", &request, plan.clone()).unwrap();
    let initial_signature = plan.steps[1].input["candidate_signature"].as_str().unwrap();
    task.context.state.insert(
        "latest_validation_record".to_string(),
        json!(ValidationRecord {
            command: "./validate.sh".to_string(),
            exit_code: 101,
            stdout: String::new(),
            stderr:
                "validation hint: inspect src/helper.rs for the remaining failing arithmetic path"
                    .to_string(),
            succeeded: false,
        }),
    );
    task.context
        .state
        .insert("latest_attempt_id".to_string(), plan.steps[1].input["attempt_id"].clone());
    task.context
        .state
        .insert("adaptive_candidate_signatures".to_string(), json!([initial_signature]));

    let failure = StepExecutionResult::failure(
        ErrorInfo::new(
            "execution_validation_failed",
            "workspace execution profile still fails validation after attempt adaptive-attempt-1",
        )
        .with_details(json!({
            "stderr": "validation hint: inspect src/helper.rs for the remaining failing arithmetic path"
        })),
        Recoverability::ReplanRequired,
    );

    let replanned = runtime.planner.replan(&task, &plan.steps[2], &failure).unwrap();

    assert_eq!(
        replanned[0].input["adaptive_attempt"]["changes"][0]["path"],
        json!("src/helper.rs")
    );
    assert_eq!(replanned[0].input["workspace_slice"]["selected_targets"], json!(["src/helper.rs"]));
    assert!(
        replanned[0].input["selection_evidence"]["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("validation"))
    );
}

#[test]
fn adaptive_profile_can_select_the_ordering_boundary_family() {
    let workspace =
        temp_adaptive_ordering_boundary_workspace("boundline-adaptive-ordering-boundary");
    let plan =
        build_fixture_plan_for_goal(&workspace, None, "Fix the inclusive threshold boundary")
            .unwrap();

    assert_eq!(
        plan.steps[1].input["selection_evidence"]["candidate_family"],
        json!("ordering_boundary_flip")
    );
    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["find"], json!(" > "));
    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["replace"], json!(" >= "));
}
