use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use boundline::adapters::agent::FnAgentAdapter;
use boundline::adapters::tool::FnToolAdapter;
use boundline::adapters::trace_store::FileTraceStore;
use boundline::domain::limits::RunLimits;
use boundline::domain::plan::Plan;
use boundline::domain::step::{
    ErrorInfo, Recoverability, Step, StepExecutionRequest, StepExecutionResult,
};
use boundline::domain::task::{TaskRunRequest, TaskStatus};
use boundline::orchestrator::engine::Orchestrator;
use boundline::orchestrator::planner::StaticPlanner;
use boundline::registry::agent_registry::AgentRegistry;
use boundline::registry::tool_registry::ToolRegistry;
use serde_json::json;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let path = std::env::temp_dir().join(format!("boundline-trace-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn build_request(workspace: &Path) -> TaskRunRequest {
    TaskRunRequest {
        goal: "Capture an execution trace".to_string(),
        input: json!({"ticket": "BUG-8"}),
        session_id: format!("session-{}", Uuid::new_v4()),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: RunLimits::default(),
        initial_context: None,
    }
}

#[test]
fn persists_retry_and_success_events_in_the_trace_file() {
    let workspace = temp_workspace();
    let planner = StaticPlanner::new(
        Plan::new(vec![
            Step::agent("code", "coder", json!({"phase": "code"})).unwrap(),
            Step::tool("verify", "tester", json!({"phase": "verify"})).unwrap(),
        ])
        .unwrap(),
    );

    let retry_counter = Arc::new(Mutex::new(0usize));
    let mut agents = AgentRegistry::new();
    agents
        .register("coder", {
            let retry_counter = retry_counter.clone();
            FnAgentAdapter::new(move |_request: StepExecutionRequest| {
                let mut counter = retry_counter.lock().unwrap();
                *counter += 1;
                if *counter == 1 {
                    StepExecutionResult::failure(
                        ErrorInfo::new("transient", "first attempt failed"),
                        Recoverability::Retryable,
                    )
                } else {
                    StepExecutionResult::success(json!({"patch_applied": true}))
                }
            })
        })
        .unwrap();

    let mut tools = ToolRegistry::new();
    tools
        .register(
            "tester",
            FnToolAdapter::new(|_| StepExecutionResult::success(json!({"tests_passed": true}))),
        )
        .unwrap();

    let orchestrator =
        Orchestrator::new(planner, agents, tools, FileTraceStore::for_workspace(&workspace));
    let response = orchestrator.run(build_request(&workspace)).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Succeeded);

    let trace: serde_json::Value =
        serde_json::from_slice(&fs::read(&response.trace_location).unwrap()).unwrap();
    let event_types = trace["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["event_type"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(event_types.first(), Some(&"task_started"));
    assert!(event_types.contains(&"retry_scheduled"));
    assert_eq!(event_types.last(), Some(&"terminal_recorded"));
    assert_eq!(trace["terminal_status"], json!("succeeded"));
}

#[test]
fn persists_failed_terminal_reason_without_live_process_state() {
    let workspace = temp_workspace();
    let planner = StaticPlanner::new(
        Plan::new(vec![Step::tool("verify", "tester", json!({"phase": "verify"})).unwrap()])
            .unwrap(),
    );

    let agents = AgentRegistry::new();
    let tools = ToolRegistry::new();

    let orchestrator =
        Orchestrator::new(planner, agents, tools, FileTraceStore::for_workspace(&workspace));
    let response = orchestrator.run(build_request(&workspace)).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Failed);

    let trace: serde_json::Value =
        serde_json::from_slice(&fs::read(&response.trace_location).unwrap()).unwrap();
    assert_eq!(trace["terminal_status"], json!("failed"));
    assert_eq!(trace["terminal_reason"]["condition"], json!("unrecoverable_error"));
    assert!(trace["events"].as_array().unwrap().len() >= 3);
}
