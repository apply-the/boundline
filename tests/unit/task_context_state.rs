use serde_json::{Map, Value, json};
use synod::domain::limits::RunLimits;
use synod::domain::step::{Recoverability, Step, StepResultSummary};
use synod::domain::task_context::TaskContext;

#[test]
fn task_context_merges_state_patches_and_repairs_nested_buckets() {
    let mut initial_state = Map::new();
    initial_state.insert("step_outputs".to_string(), Value::String("broken".to_string()));

    let mut context = TaskContext::new(
        "session-context",
        "/tmp/synod-context",
        RunLimits::default(),
        initial_state,
    );

    let mut patch = Map::new();
    patch.insert("goal_satisfied".to_string(), json!(true));
    patch.insert("verified".to_string(), json!(true));

    context.apply_success_output("analyze", &json!({"analysis": "complete"}), Some(&patch));

    assert_eq!(context.state["analysis"], json!("complete"));
    assert_eq!(context.state["goal_satisfied"], json!(true));
    assert_eq!(context.state["verified"], json!(true));
    assert_eq!(context.state["step_outputs"]["analyze"]["analysis"], json!("complete"));

    let mut failed_step = Step::decision("verify", json!({})).unwrap();
    failed_step.mark_failed(
        synod::domain::step::ErrorInfo::new("retryable", "try again"),
        Recoverability::Retryable,
    );
    context.set_last_result(StepResultSummary::from_step(&failed_step));
    context.push_history_ref("attempt-1");

    assert_eq!(context.history_refs, vec!["attempt-1".to_string()]);
    assert_eq!(context.last_result.as_ref().unwrap().step_id, "verify");
}
