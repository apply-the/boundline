use std::path::Path;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::adapters::trace_store::{FileTraceStore, TraceStore};
use boundline::cli::session::{execute_goal, execute_plan, execute_run};
use boundline::domain::goal_plan::GoalPlanStatus;
use boundline::domain::session::SessionStatus;
use boundline::domain::trace::TraceEventType;

use crate::runtime_refoundation::{
    temp_runtime_refoundation_governed_workspace, temp_runtime_refoundation_workspace,
};

#[test]
fn bounded_goal_plan_handoff_is_persisted_before_native_run() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-contract-handoff");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let goal_plan = session.goal_plan.as_ref().expect("goal plan should persist after planning");

    assert_eq!(session.latest_status, SessionStatus::Planned);
    assert!(session.active_task.is_none());
    assert!(session.decisions.is_empty());
    assert_eq!(goal_plan.status, GoalPlanStatus::Confirmed);
    assert!(!goal_plan.tasks.is_empty());
}

#[test]
fn native_run_persists_decision_contract_fields_into_trace_output() {
    let workspace =
        temp_runtime_refoundation_governed_workspace("runtime-refoundation-contract-trace");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), None, true).unwrap();
    execute_run(Some(&workspace)).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let trace_ref = session.latest_trace_ref.as_ref().expect("trace ref should persist");
    let trace = FileTraceStore::for_workspace(&workspace).load(Path::new(trace_ref)).unwrap();
    let decision = session.decisions.first().expect("at least one decision should persist");

    assert!(!decision.expected_outcome.is_empty());
    assert!(
        !decision.evidence_inputs.is_empty(),
        "expected initial decision evidence to include bounded task draft inputs"
    );

    let created = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::DecisionCreated)
        .expect("decision created event should be recorded");
    assert_eq!(
        created.payload.get("rationale").and_then(|value| value.as_str()),
        Some(decision.rationale.as_str())
    );
    assert_eq!(
        created.payload.get("expected_outcome").and_then(|value| value.as_str()),
        Some(decision.expected_outcome.as_str())
    );
    assert_eq!(created.payload.get("status").and_then(|value| value.as_str()), Some("pending"));
    assert!(
        created
            .payload
            .get("evidence_inputs")
            .and_then(|value| value.as_array())
            .is_some_and(|items| !items.is_empty()),
        "trace payload should expose decision evidence inputs"
    );

    let verified = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::DecisionVerified)
        .expect("decision verified event should be recorded");
    assert_eq!(verified.payload.get("status").and_then(|value| value.as_str()), Some("verified"));
    assert!(
        verified.payload.get("completed_at").is_some(),
        "verified decision event should expose completion time"
    );
}
