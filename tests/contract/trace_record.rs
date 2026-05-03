use std::fs;

use boundline::adapters::trace_store::{FileTraceStore, TraceStore};
use boundline::domain::limits::TerminalCondition;
use boundline::domain::task::{TaskStatus, TerminalReason};
use boundline::domain::trace::{ExecutionTrace, TraceEventType};
use serde_json::json;
use uuid::Uuid;

#[test]
fn persists_trace_metadata_and_event_fields_in_a_contract_shape() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-trace-contract-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let store = FileTraceStore::for_workspace(&workspace);
    let mut trace = ExecutionTrace::new("task-1", "session-1", "Inspect trace contract");
    trace.record_event(
        TraceEventType::StepStarted,
        Some("step-1".to_string()),
        0,
        json!({"input": {"ticket": "BUG-7"}}),
    );
    trace.finalize(
        TaskStatus::Succeeded,
        TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
    );

    let path = store.persist(&trace).unwrap();
    trace.set_trace_location(path.to_string_lossy().into_owned());
    store.persist(&trace).unwrap();

    let document: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(document["task_id"], json!("task-1"));
    assert_eq!(document["session_id"], json!("session-1"));
    assert_eq!(document["goal"], json!("Inspect trace contract"));
    assert_eq!(document["terminal_status"], json!("succeeded"));
    assert!(document["trace_location"].is_string());

    let event = &document["events"][0];
    assert!(event["event_id"].is_string());
    assert_eq!(event["event_type"], json!("step_started"));
    assert_eq!(event["step_id"], json!("step-1"));
    assert_eq!(event["plan_revision"], json!(0));
    assert!(event["recorded_at"].is_number());
}
