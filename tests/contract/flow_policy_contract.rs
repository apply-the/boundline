use synod::adapters::session_store::{FileSessionStore, SessionStore};
use synod::cli::session::{
    SessionCommandError, execute_capture, execute_plan, execute_run, execute_start,
};
use synod::domain::goal_plan::GoalPlanFlowMode;

use crate::runtime_refoundation::temp_runtime_refoundation_workspace;

#[test]
fn proposed_flow_contract_blocks_run_until_operator_confirms_or_skips() {
    let workspace = temp_runtime_refoundation_workspace("flow-policy-contract-proposed");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), None, false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert_eq!(session.goal_plan.as_ref().unwrap().flow_state().mode, GoalPlanFlowMode::Proposed);
    assert!(matches!(
        execute_run(Some(&workspace)).unwrap_err(),
        SessionCommandError::PlanConfirmationRequired { flow_name }
            if flow_name.as_deref() == Some("bug-fix")
    ));
}

#[test]
fn confirmed_flow_contract_persists_explicit_override() {
    let workspace = temp_runtime_refoundation_workspace("flow-policy-contract-confirmed");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("change"), false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let flow_state = session.goal_plan.as_ref().unwrap().flow_state();

    assert_eq!(flow_state.mode, GoalPlanFlowMode::Confirmed);
    assert_eq!(flow_state.flow_name.as_deref(), Some("change"));
    assert!(execute_run(Some(&workspace)).is_ok());
}

#[test]
fn skipped_flow_contract_persists_operator_skip_without_active_policy() {
    let workspace = temp_runtime_refoundation_workspace("flow-policy-contract-skipped");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("implement workspace summary output"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), None, true, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let flow_state = session.goal_plan.as_ref().unwrap().flow_state();

    assert_eq!(flow_state.mode, GoalPlanFlowMode::Skipped);
    assert!(session.active_flow.is_none());
    assert!(session.active_flow_policy.is_none());
    assert!(execute_run(Some(&workspace)).is_ok());
}
