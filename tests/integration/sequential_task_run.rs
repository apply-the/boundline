use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use synod::adapters::agent::FnAgentAdapter;
use synod::adapters::tool::FnToolAdapter;
use synod::adapters::trace_store::FileTraceStore;
use synod::domain::limits::RunLimits;
use synod::domain::plan::Plan;
use synod::domain::step::{Step, StepExecutionRequest, StepExecutionResult};
use synod::domain::task::TaskRunRequest;
use synod::orchestrator::engine::Orchestrator;
use synod::orchestrator::planner::StaticPlanner;
use synod::registry::agent_registry::AgentRegistry;
use synod::registry::tool_registry::ToolRegistry;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let path = std::env::temp_dir().join(format!("synod-integration-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn build_three_step_plan() -> Plan {
    Plan::new(vec![
        Step::agent("analyze", "analyzer", json!({"phase": "analyze"})).unwrap(),
        Step::agent("code", "coder", json!({"phase": "code"})).unwrap(),
        Step::tool("verify", "tester", json!({"phase": "verify"})).unwrap(),
    ])
    .unwrap()
}

fn build_success_orchestrator(workspace: &Path) -> Orchestrator<StaticPlanner, FileTraceStore> {
    let planner = StaticPlanner::new(build_three_step_plan());
    let mut agents = AgentRegistry::new();
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(|request: StepExecutionRequest| {
                StepExecutionResult::success(json!({
                    "analysis": request.input,
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
                    "patch_applied": true,
                    "source_step": request.step_id,
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
                    "tests_passed": true,
                    "verified_input": request.input,
                }))
            }),
        )
        .unwrap();

    Orchestrator::new(planner, agents, tools, FileTraceStore::for_workspace(workspace))
}

#[test]
fn completes_a_three_step_task_and_preserves_context_between_steps() {
    let workspace = temp_workspace();
    let orchestrator = build_success_orchestrator(&workspace);
    let request = TaskRunRequest {
        goal: "Complete a bounded multi-step task".to_string(),
        input: json!({"ticket": "BUG-3"}),
        session_id: "session-us1".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: RunLimits::default(),
        initial_context: None,
    };

    let response = orchestrator.run(request).unwrap();
    assert_eq!(response.final_context.history_refs.len(), 3);
    assert_eq!(response.final_context.state.get("ready_for_code"), Some(&json!(true)));
    assert_eq!(response.final_context.state.get("patch_applied"), Some(&json!(true)));
    assert_eq!(response.final_context.state.get("tests_passed"), Some(&json!(true)));
}
