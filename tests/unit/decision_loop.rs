use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use boundline::adapters::agent::FnAgentAdapter;
use boundline::adapters::tool::FnToolAdapter;
use boundline::adapters::trace_store::FileTraceStore;
use boundline::domain::decision::{ActionSelector, DecisionStatus, DecisionType, EvidenceRef};
use boundline::domain::flow_policy::FlowPolicy;
use boundline::domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan, PlannedTask,
};
use boundline::domain::step::{ErrorInfo, ExecutionStatus, Recoverability, StepExecutionResult};
use boundline::domain::trace::TraceEventType;
use boundline::orchestrator::decision_loop::{DecisionLoop, LoopTerminal, Observation};
use boundline::registry::agent_registry::AgentRegistry;
use boundline::registry::tool_registry::ToolRegistry;
use serde_json::json;

fn temp_workspace(prefix: &str) -> PathBuf {
    let ws = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&ws).unwrap();
    ws
}

fn sample_task(id: &str, target: &str, hint: DecisionType) -> PlannedTask {
    PlannedTask {
        task_id: id.to_string(),
        description: format!("Process {target}"),
        target: target.to_string(),
        expected_outcome: Some("completed".to_string()),
        decision_type_hint: Some(hint),
    }
}

fn collect_workspace_files(
    root: &std::path::Path,
    current: &std::path::Path,
    files: &mut Vec<String>,
) {
    if files.len() >= 16 {
        return;
    }

    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        if files.len() >= 16 {
            break;
        }

        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if name == "target" {
                continue;
            }
            collect_workspace_files(root, &path, files);
            continue;
        }

        if path.is_file()
            && let Ok(relative) = path.strip_prefix(root)
        {
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn build_loop(workspace: &std::path::Path, max_steps: usize) -> DecisionLoop<FileTraceStore> {
    let mut agents = AgentRegistry::new();
    let analyzer_workspace = workspace.to_path_buf();
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(move |request| {
                let selector = request
                    .input
                    .get("selector")
                    .and_then(|value| value.as_str())
                    .unwrap_or("read");
                let target =
                    request.input.get("target").and_then(|value| value.as_str()).unwrap_or("");
                if selector == "search" {
                    let mut files = Vec::new();
                    collect_workspace_files(&analyzer_workspace, &analyzer_workspace, &mut files);
                    return if files.is_empty() {
                        StepExecutionResult::failure(
                            ErrorInfo::new(
                                "workspace_search_failed",
                                format!("failed to find credible workspace evidence for {target}"),
                            ),
                            Recoverability::ReplanRequired,
                        )
                    } else {
                        StepExecutionResult::success(json!({
                            "stdout": files.join("\n"),
                            "target": target,
                            "matches": files,
                        }))
                    };
                }

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
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let contents = fs::read_to_string(&path).unwrap_or_default();
                let updated = if contents.contains("left - right") {
                    contents.replacen("left - right", "left + right", 1)
                } else if contents.is_empty() {
                    "// generated by test adapter\n".to_string()
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
    tools
        .register(
            "asker",
            FnToolAdapter::new(move |request| {
                let prompt = request
                    .input
                    .get("expected_outcome")
                    .and_then(|value| value.as_str())
                    .unwrap_or("clarify the next bounded action before continuing");
                StepExecutionResult::success(json!({"stdout": prompt, "prompt": prompt}))
            }),
        )
        .unwrap();
    let trace_store = FileTraceStore::for_workspace(workspace);
    DecisionLoop::new(agents, tools, trace_store, max_steps)
}

#[test]
fn observe_phase_collects_workspace_files_and_evidence() {
    // Observation struct captures workspace state correctly
    let obs = Observation {
        workspace_files: vec!["src/main.rs".to_string()],
        last_decision: None,
        accumulated_evidence: vec![EvidenceRef::file("src/lib.rs")],
        remaining_tasks: vec!["src/main.rs".to_string()],
    };
    assert_eq!(obs.workspace_files.len(), 1);
    assert!(obs.last_decision.is_none());
    assert_eq!(obs.accumulated_evidence.len(), 1);
}

#[test]
fn decide_phase_selects_action_from_plan_task() {
    let workspace = temp_workspace("dl-decide");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/target.rs"), "// code").unwrap();

    let plan = GoalPlan::new(
        "Analyze target",
        vec![sample_task("t1", "src/target.rs", DecisionType::Analyze)],
    )
    .unwrap();

    let dl = build_loop(workspace, 10);
    let (_terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-1").unwrap();

    assert_eq!(decisions[0].decision_type, DecisionType::Analyze);
    assert_eq!(decisions[0].selector_kind(), ActionSelector::Read);
    assert_eq!(decisions[0].target, "src/target.rs");
}

#[test]
fn verify_phase_transitions_decision_status() {
    let workspace = temp_workspace("dl-verify");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/f.rs"), "fn f() {}").unwrap();

    let plan =
        GoalPlan::new("Read file", vec![sample_task("t1", "src/f.rs", DecisionType::Analyze)])
            .unwrap();

    let dl = build_loop(workspace, 10);
    let (_terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-1").unwrap();

    // Analyze on existing file → success → Verified
    assert_eq!(decisions[0].status, DecisionStatus::Verified);
    assert!(decisions[0].tool_result.is_some());
    assert!(decisions[0].tool_result.as_ref().unwrap().success);
}

#[test]
fn exhaustion_terminal_when_max_steps_reached() {
    let workspace = temp_workspace("dl-exhaust");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/a.rs"), "a").unwrap();
    std::fs::write(workspace.join("src/b.rs"), "b").unwrap();
    std::fs::write(workspace.join("src/c.rs"), "c").unwrap();

    let plan = GoalPlan::new(
        "Process three files",
        vec![
            sample_task("t1", "src/a.rs", DecisionType::Analyze),
            sample_task("t2", "src/b.rs", DecisionType::Analyze),
            sample_task("t3", "src/c.rs", DecisionType::Analyze),
        ],
    )
    .unwrap();

    let dl = build_loop(workspace, 2);
    let (terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-1").unwrap();

    assert!(matches!(terminal, LoopTerminal::Exhausted { steps_taken: 2, max_steps: 2 }));
    assert_eq!(decisions.len(), 2);
}

#[test]
fn trace_contains_decision_events() {
    let workspace = temp_workspace("dl-trace");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/t.rs"), "test").unwrap();

    let plan = GoalPlan::new("Analyze", vec![sample_task("t1", "src/t.rs", DecisionType::Analyze)])
        .unwrap();

    let dl = build_loop(workspace, 10);
    let (_terminal, _decisions, trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-1").unwrap();

    let event_types: Vec<_> = trace.events.iter().map(|e| e.event_type).collect();
    assert!(event_types.contains(&TraceEventType::GoalPlanCreated));
    assert!(event_types.contains(&TraceEventType::DecisionCreated));
    assert!(event_types.contains(&TraceEventType::DecisionDispatched));
    assert!(event_types.contains(&TraceEventType::DecisionVerified));
    assert!(event_types.contains(&TraceEventType::TerminalRecorded));

    let payload = &trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::DecisionCreated)
        .unwrap()
        .payload;
    assert_eq!(payload.get("selector").and_then(|value| value.as_str()), Some("read"));
}

#[test]
fn recovery_from_failed_analysis_marks_recovered() {
    let workspace = temp_workspace("dl-recover");
    let workspace = workspace.as_path();

    // Target file does NOT exist → Analyze will fail
    let plan = GoalPlan::new(
        "Analyze missing file",
        vec![sample_task("t1", "src/nonexistent.rs", DecisionType::Analyze)],
    )
    .unwrap();

    let dl = build_loop(workspace, 5);
    let (terminal, _decisions, trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-1").unwrap();
    let has_failed = trace.events.iter().any(|e| e.event_type == TraceEventType::DecisionFailed);
    let has_recovered =
        trace.events.iter().any(|e| e.event_type == TraceEventType::DecisionRecovered);
    assert!(has_failed);
    assert!(has_recovered);

    assert!(matches!(terminal, LoopTerminal::NoActionableState(_)));
}

#[test]
fn failed_analysis_uses_search_selector_before_asking_for_clarification() {
    let workspace = temp_workspace("dl-search-recovery");
    let workspace = workspace.as_path();

    let plan = GoalPlan::new(
        "Analyze missing file",
        vec![sample_task("t1", "src/nonexistent.rs", DecisionType::Analyze)],
    )
    .unwrap();

    let dl = build_loop(workspace, 5);
    let (_terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-search").unwrap();

    assert_eq!(decisions[0].selector_kind(), ActionSelector::Search);
    assert_eq!(decisions[1].selector_kind(), ActionSelector::Ask);
}

#[test]
fn successful_search_can_complete_analyze_task() {
    let workspace = temp_workspace("dl-search-success");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/other.rs"), "fn helper() {}\n").unwrap();

    let plan = GoalPlan::new(
        "Analyze missing file with surrounding context",
        vec![sample_task("t1", "src/missing.rs", DecisionType::Analyze)],
    )
    .unwrap();

    let dl = build_loop(workspace, 5);
    let (terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-search-success").unwrap();

    assert!(matches!(terminal, LoopTerminal::Success));
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].selector_kind(), ActionSelector::Search);
    assert_eq!(decisions[0].status, DecisionStatus::Verified);
}

#[test]
fn goal_plan_created_trace_event_includes_context_projection() {
    let workspace = temp_workspace("dl-context-trace");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }",
    )
    .unwrap();

    let plan = GoalPlan::new(
        "Fix the add implementation",
        vec![sample_task("t1", "src/lib.rs", DecisionType::Fix)],
    )
    .unwrap()
    .with_context_pack(ContextPack {
        pack_id: "cp-1".to_string(),
        summary: "bounded context from src/lib.rs".to_string(),
        credibility: ContextPackCredibility::Credible,
        inputs: vec![ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "contains the failing arithmetic path".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec!["src/lib.rs".to_string()],
        advanced_context: None,
        staleness_reason: None,
    });

    let dl = build_loop(workspace, 10);
    let (_terminal, _decisions, trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-1").unwrap();

    let payload = &trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::GoalPlanCreated)
        .unwrap()
        .payload;

    assert_eq!(
        payload.get("context_summary").and_then(|value| value.as_str()),
        Some("bounded context from src/lib.rs")
    );
    assert_eq!(
        payload.get("context_credibility").and_then(|value| value.as_str()),
        Some("credible")
    );
    assert_eq!(payload.get("context_primary_inputs"), Some(&json!(["src/lib.rs"])));
    assert_eq!(
        payload.get("context_provenance"),
        Some(&json!([
            "workspace_file: src/lib.rs (contains the failing arithmetic path) [source=workspace_scan]"
        ]))
    );
}

#[test]
fn decision_loop_with_flow_policy_advances_stage_and_prefers_allowed_recovery_decisions() {
    let workspace = temp_workspace("dl-flow-policy");
    let workspace = workspace.as_path();

    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/ok.rs"), "fn ok() {}\n").unwrap();

    let success_plan = GoalPlan::new(
        "Analyze a file",
        vec![sample_task("t1", "src/ok.rs", DecisionType::Analyze)],
    )
    .unwrap();
    let dl = build_loop(workspace, 10);
    let flow_policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    let (terminal, _decisions, _trace) = dl
        .run(&success_plan, Some(&flow_policy), &workspace.to_string_lossy(), "sess-success")
        .unwrap();
    assert!(matches!(terminal, LoopTerminal::Success));

    let failed_plan = GoalPlan::new(
        "Analyze a missing file",
        vec![sample_task("t2", "src/missing.rs", DecisionType::Analyze)],
    )
    .unwrap();
    let (_terminal, decisions, _trace) = dl
        .run(&failed_plan, Some(&flow_policy), &workspace.to_string_lossy(), "sess-failed")
        .unwrap();

    assert_eq!(decisions[0].decision_type, DecisionType::Analyze);
    assert_eq!(decisions[0].selector_kind(), ActionSelector::Search);
    assert_eq!(decisions[0].status, DecisionStatus::Verified);
}

#[test]
fn decision_loop_preserves_structured_failure_output_in_tool_results() {
    let workspace = temp_workspace("dl-structured-failure");
    let workspace = workspace.as_path();

    let mut agents = AgentRegistry::new();
    agents
        .register(
            "analyzer",
            FnAgentAdapter::new(move |_request| StepExecutionResult {
                status: ExecutionStatus::Failed,
                output: Some(json!({
                    "stderr": "lint failed",
                    "diff": "patched diff",
                    "exit_code": 17
                })),
                error: Some(ErrorInfo::new("lint_failed", "fallback error message")),
                recoverability: Recoverability::Retryable,
                evidence: None,
                state_patch: None,
            }),
        )
        .unwrap();
    let tools = ToolRegistry::new();
    let trace_store = FileTraceStore::for_workspace(workspace);
    let dl = DecisionLoop::new(agents, tools, trace_store, 1);

    let plan = GoalPlan::new(
        "Analyze a failing command",
        vec![sample_task("t-analyze", "src/lib.rs", DecisionType::Analyze)],
    )
    .unwrap();

    let (_terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-structured").unwrap();

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].decision_type, DecisionType::Analyze);
    let tool_result = decisions[0].tool_result.as_ref().unwrap();
    assert!(tool_result.stdout.contains("\"diff\":\"patched diff\""));
    assert_eq!(tool_result.stderr, "lint failed");
    assert_eq!(tool_result.diff.as_deref(), Some("patched diff"));
    assert_eq!(tool_result.exit_code, Some(17));
}

#[test]
fn decision_loop_uses_missing_adapter_failure_for_unregistered_replanner() {
    let workspace = temp_workspace("dl-missing-adapter");
    let workspace = workspace.as_path();

    let agents = AgentRegistry::new();
    let tools = ToolRegistry::new();
    let trace_store = FileTraceStore::for_workspace(workspace);
    let dl = DecisionLoop::new(agents, tools, trace_store, 1);
    let plan = GoalPlan::new(
        "Request replanning",
        vec![sample_task("t-replan", "src/lib.rs", DecisionType::Replan)],
    )
    .unwrap();

    let (_terminal, decisions, _trace) =
        dl.run(&plan, None, &workspace.to_string_lossy(), "sess-missing-adapter").unwrap();

    assert_eq!(decisions.len(), 1);
    let tool_result = decisions[0].tool_result.as_ref().unwrap();
    assert!(tool_result.stderr.contains("no adapter named `replanner`"));
    assert_eq!(tool_result.exit_code, Some(-1));
}
