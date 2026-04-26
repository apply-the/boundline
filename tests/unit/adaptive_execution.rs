use serde_json::json;
use synod::fixture::{build_fixture_plan_for_goal, load_workspace_execution_profile};

use crate::workspace_fixture::{
    temp_adaptive_fixture_workspace, temp_adaptive_replanning_workspace,
};

#[test]
fn adaptive_profile_builds_a_goal_aware_initial_plan_without_authored_attempts() {
    let workspace = temp_adaptive_fixture_workspace("synod-adaptive-plan");
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
        json!("selected src/lib.rs for adaptive delivery")
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
    let workspace = temp_adaptive_replanning_workspace("synod-adaptive-replan-plan");
    let plan =
        build_fixture_plan_for_goal(&workspace, None, "Recover after validation fails").unwrap();

    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["find"], json!(" * "));
    assert_eq!(plan.steps[1].input["adaptive_attempt"]["changes"][0]["replace"], json!(" - "));
}
