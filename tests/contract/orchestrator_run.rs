use std::fs;
use std::path::{Path, PathBuf};

use boundline::adapters::agent::FnAgentAdapter;
use boundline::adapters::tool::FnToolAdapter;
use boundline::adapters::trace_store::FileTraceStore;
use boundline::domain::limits::RunLimits;
use boundline::domain::plan::Plan;
use boundline::domain::step::{Step, StepExecutionRequest, StepExecutionResult};
use boundline::domain::task::{TaskRunRequest, TaskStatus};
use boundline::orchestrator::engine::Orchestrator;
use boundline::orchestrator::planner::StaticPlanner;
use boundline::registry::agent_registry::AgentRegistry;
use boundline::registry::tool_registry::ToolRegistry;
use serde_json::json;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let path = std::env::temp_dir().join(format!("boundline-contract-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn build_success_orchestrator(workspace: &Path) -> Orchestrator<StaticPlanner, FileTraceStore> {
    let plan = Plan::new(vec![
        Step::agent("analyze", "analyzer", json!({"task": "analyze"})).unwrap(),
        Step::agent("code", "coder", json!({"task": "code"})).unwrap(),
        Step::tool("verify", "tester", json!({"task": "verify"})).unwrap(),
    ])
    .unwrap();

    let planner = StaticPlanner::new(plan);
    let mut agents = AgentRegistry::new();
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(|request: StepExecutionRequest| {
                StepExecutionResult::success(json!({
                    "analysis": format!("analyzed {}", request.step_id),
                    "ready_for_code": true,
                }))
            }),
        )
        .unwrap();
    agents
        .register(
            "coder",
            FnAgentAdapter::new(|request: StepExecutionRequest| {
                StepExecutionResult::success(json!({
                    "patch": format!("patched {}", request.step_id),
                    "updated_files": ["src/lib.rs"],
                }))
            }),
        )
        .unwrap();

    let mut tools = ToolRegistry::new();
    tools
        .register(
            "tester",
            FnToolAdapter::new(|request: StepExecutionRequest| {
                StepExecutionResult::success(json!({
                    "verified_step": request.step_id,
                    "tests_passed": true,
                }))
            }),
        )
        .unwrap();

    Orchestrator::new(planner, agents, tools, FileTraceStore::for_workspace(workspace))
}

#[test]
fn rejects_invalid_task_requests_before_execution_starts() {
    let workspace = temp_workspace();
    let orchestrator = build_success_orchestrator(&workspace);
    let request = TaskRunRequest {
        goal: String::new(),
        input: json!({"ticket": "BUG-1"}),
        session_id: "session-1".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: RunLimits::default(),
        initial_context: None,
    };

    let error = orchestrator.run(request).unwrap_err();
    assert!(error.to_string().contains("goal"));
}

#[test]
fn returns_terminal_response_with_context_and_trace_location() {
    let workspace = temp_workspace();
    let orchestrator = build_success_orchestrator(&workspace);
    let request = TaskRunRequest {
        goal: "Fix a failing test".to_string(),
        input: json!({"ticket": "BUG-2"}),
        session_id: "session-2".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: RunLimits::default(),
        initial_context: None,
    };

    let response = orchestrator.run(request).unwrap();
    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(response.plan_revision, 0);
    assert!(Path::new(&response.trace_location).exists());
    assert_eq!(response.final_context.state.get("tests_passed"), Some(&json!(true)));
    assert_eq!(response.final_context.history_refs.len(), 3);
}
