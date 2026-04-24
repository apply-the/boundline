use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;
use synod::adapters::agent::FnAgentAdapter;
use synod::adapters::tool::FnToolAdapter;
use synod::adapters::trace_store::FileTraceStore;
use synod::domain::limits::RunLimits;
use synod::domain::plan::Plan;
use synod::domain::step::{
    ErrorInfo, Recoverability, Step, StepExecutionRequest, StepExecutionResult,
};
use synod::domain::task::{TaskRunRequest, TaskStatus};
use synod::orchestrator::engine::Orchestrator;
use synod::orchestrator::planner::StaticPlanner;
use synod::registry::agent_registry::AgentRegistry;
use synod::registry::tool_registry::ToolRegistry;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let path = std::env::temp_dir().join(format!("synod-recovery-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn build_request(workspace: &Path, limits: RunLimits) -> TaskRunRequest {
    TaskRunRequest {
        goal: "Recover a failing execution".to_string(),
        input: json!({"ticket": "BUG-6"}),
        session_id: format!("session-{}", Uuid::new_v4()),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits,
        initial_context: None,
    }
}

#[test]
fn retries_a_failed_step_and_then_completes_successfully() {
    let workspace = temp_workspace();
    let planner = StaticPlanner::new(
        Plan::new(vec![
            Step::agent("analyze", "analyzer", json!({"phase": "analyze"})).unwrap(),
            Step::agent("code", "coder", json!({"phase": "code"})).unwrap(),
            Step::tool("verify", "tester", json!({"phase": "verify"})).unwrap(),
        ])
        .unwrap(),
    );

    let mut agents = AgentRegistry::new();
    let retry_counter = Arc::new(Mutex::new(0usize));
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(|_| StepExecutionResult::success(json!({"ready_for_code": true}))),
        )
        .unwrap();
    agents
        .register("coder", {
            let retry_counter = retry_counter.clone();
            FnAgentAdapter::new(move |_request: StepExecutionRequest| {
                let mut counter = retry_counter.lock().unwrap();
                *counter += 1;
                if *counter == 1 {
                    StepExecutionResult::failure(
                        ErrorInfo::new("transient", "temporary coding failure"),
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
    let response = orchestrator.run(build_request(&workspace, RunLimits::default())).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(*retry_counter.lock().unwrap(), 2);
    assert_eq!(response.final_context.state.get("patch_applied"), Some(&json!(true)));
}

#[test]
fn replans_after_a_failed_step_and_continues_with_the_revised_plan() {
    let workspace = temp_workspace();
    let planner = StaticPlanner::with_replans(
        Plan::new(vec![
            Step::agent("analyze", "analyzer", json!({"phase": "analyze"})).unwrap(),
            Step::agent("code", "coder", json!({"phase": "code"})).unwrap(),
        ])
        .unwrap(),
        vec![vec![
            Step::agent("recode", "coder", json!({"phase": "recode"})).unwrap(),
            Step::tool("verify", "tester", json!({"phase": "verify"})).unwrap(),
        ]],
    );

    let mut agents = AgentRegistry::new();
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(|_| {
                StepExecutionResult::success(json!({"analysis_complete": true}))
            }),
        )
        .unwrap();
    agents
        .register(
            "coder",
            FnAgentAdapter::new(|request: StepExecutionRequest| {
                if request.step_id == "code" {
                    StepExecutionResult::failure(
                        ErrorInfo::new("invalid_plan", "the current plan no longer applies"),
                        Recoverability::ReplanRequired,
                    )
                } else {
                    StepExecutionResult::success(json!({"replanned_patch": true}))
                }
            }),
        )
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
    let response = orchestrator.run(build_request(&workspace, RunLimits::default())).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(response.plan_revision, 1);
    assert_eq!(response.final_context.state.get("replanned_patch"), Some(&json!(true)));
}

#[test]
fn returns_exhausted_when_retry_budget_is_zero() {
    let workspace = temp_workspace();
    let planner = StaticPlanner::new(
        Plan::new(vec![Step::tool("verify", "tester", json!({"phase": "verify"})).unwrap()])
            .unwrap(),
    );

    let agents = AgentRegistry::new();
    let mut tools = ToolRegistry::new();
    tools
        .register(
            "tester",
            FnToolAdapter::new(|_| {
                StepExecutionResult::failure(
                    ErrorInfo::new("flaky_tool", "tool execution failed"),
                    Recoverability::Retryable,
                )
            }),
        )
        .unwrap();

    let orchestrator =
        Orchestrator::new(planner, agents, tools, FileTraceStore::for_workspace(&workspace));
    let response = orchestrator
        .run(build_request(&workspace, RunLimits { max_retries: 0, ..RunLimits::default() }))
        .unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Exhausted);
}
