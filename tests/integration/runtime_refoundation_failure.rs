use std::fs;
use std::path::Path;

use boundline::adapters::agent::FnAgentAdapter;
use boundline::adapters::tool::FnToolAdapter;
use boundline::adapters::trace_store::FileTraceStore;
use boundline::domain::step::{ErrorInfo, Recoverability, StepExecutionResult};
use boundline::domain::trace::TraceEventType;
use boundline::orchestrator::decision_loop::{DecisionLoop, LoopTerminal};
use boundline::orchestrator::goal_planner::build_goal_plan;
use boundline::registry::agent_registry::AgentRegistry;
use boundline::registry::tool_registry::ToolRegistry;
use serde_json::json;

use crate::runtime_refoundation::{
    temp_runtime_refoundation_failure_workspace, temp_runtime_refoundation_workspace,
};
use crate::workspace_fixture::{run_boundline_in, terminal_text};

fn build_loop(workspace: &Path, max_steps: usize) -> DecisionLoop<FileTraceStore> {
    let mut agents = AgentRegistry::new();
    let analyzer_workspace = workspace.to_path_buf();
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(move |request| {
                let target =
                    request.input.get("target").and_then(|value| value.as_str()).unwrap_or("");
                match fs::read_to_string(analyzer_workspace.join(target)) {
                    Ok(contents) => {
                        StepExecutionResult::success(json!({"stdout": contents, "target": target}))
                    }
                    Err(error) => StepExecutionResult::failure(
                        ErrorInfo::new("file_read_failed", error.to_string()),
                        Recoverability::ReplanRequired,
                    ),
                }
            }),
        )
        .unwrap();
    let coder_workspace = workspace.to_path_buf();
    agents
        .register(
            "coder",
            FnAgentAdapter::new(move |request| {
                let target =
                    request.input.get("target").and_then(|value| value.as_str()).unwrap_or("");
                let path = coder_workspace.join(target);
                let contents = fs::read_to_string(&path).unwrap_or_default();
                let updated = if contents.contains("left - right") {
                    contents.replacen("left - right", "left + right", 1)
                } else {
                    contents
                };
                fs::write(&path, &updated).unwrap();
                StepExecutionResult::success(
                    json!({"stdout": "changed", "diff": "updated", "changed_files": [target]}),
                )
            }),
        )
        .unwrap();

    let mut tools = ToolRegistry::new();
    tools
        .register(
            "tester",
            FnToolAdapter::new(move |_request| {
                StepExecutionResult::success(json!({"stdout": "tests passed", "exit_code": 0}))
            }),
        )
        .unwrap();
    tools
        .register(
            "replanner",
            FnToolAdapter::new(move |_request| {
                StepExecutionResult::success(json!({"stdout": "replanned"}))
            }),
        )
        .unwrap();

    DecisionLoop::new(agents, tools, FileTraceStore::for_workspace(workspace), max_steps)
}

#[test]
fn inspect_preserves_failure_and_recovery_evidence_for_native_no_actionable_runs() {
    let workspace = temp_runtime_refoundation_failure_workspace("runtime-refoundation-failure");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--no-flow"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_status: failed"), "{run_text}");

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("summary: goal_plan_summary="), "{inspect_text}");
    assert!(inspect_text.contains("risk_summary:"), "{inspect_text}");
    assert!(inspect_text.contains("failed to apply the workspace change set"), "{inspect_text}");
    assert!(
        inspect_text.contains("terminal_reason: recovery decision for src/lib.rs failed"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("next_command: /boundline-next"), "{inspect_text}");
}

#[test]
fn native_decision_loop_exhaustion_is_explicit_when_step_budget_is_hit() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-exhausted");
    let goal_plan = build_goal_plan("fix the failing add test", &workspace).unwrap();

    let loop_runner = build_loop(&workspace, 1);
    let (terminal, decisions, trace) = loop_runner
        .run(&goal_plan, None, &workspace.to_string_lossy(), "session-exhausted")
        .unwrap();

    assert!(matches!(terminal, LoopTerminal::Exhausted { steps_taken: 1, max_steps: 1 }));
    assert_eq!(decisions.len(), 1);
    assert!(trace.events.iter().any(|event| {
        event.event_type == TraceEventType::TerminalRecorded
            && event.payload.get("terminal") == Some(&json!("exhausted"))
    }));
}
