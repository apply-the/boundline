use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{execute_goal, execute_plan, execute_run};
use boundline::domain::goal_plan::GoalPlanFlowMode;

use crate::runtime_refoundation::{
    temp_runtime_refoundation_no_action_workspace, temp_runtime_refoundation_workspace,
};

#[test]
fn proposed_flow_contract_run_implicitly_confirms() {
    let workspace = temp_runtime_refoundation_workspace("flow-policy-contract-proposed");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), None, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert_eq!(session.goal_plan.as_ref().unwrap().flow_state().mode, GoalPlanFlowMode::Proposed);

    let run_result = execute_run(Some(&workspace));
    assert!(run_result.is_ok());
}

#[test]
fn confirmed_flow_contract_persists_explicit_override() {
    let workspace = temp_runtime_refoundation_workspace("flow-policy-contract-confirmed");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("change"), false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let flow_state = session.goal_plan.as_ref().unwrap().flow_state();

    assert_eq!(flow_state.mode, GoalPlanFlowMode::Confirmed);
    assert_eq!(flow_state.flow_name.as_deref(), Some("change"));
    let run_result = execute_run(Some(&workspace));
    assert!(run_result.is_ok());
}

#[test]
fn skipped_flow_contract_persists_operator_skip_without_active_policy() {
    let workspace = temp_runtime_refoundation_no_action_workspace("flow-policy-contract-skipped");

    execute_goal(
        Some(&workspace),
        Some("implement workspace summary output"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), None, true).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let flow_state = session.goal_plan.as_ref().unwrap().flow_state();

    assert_eq!(flow_state.mode, GoalPlanFlowMode::Skipped);
    assert!(session.active_flow.is_none());
    assert!(session.active_flow_policy.is_none());
    assert!(execute_run(Some(&workspace)).is_err());
}
