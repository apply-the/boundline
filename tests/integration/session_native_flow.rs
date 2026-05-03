use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use serde_json::json;
use synod::FileConfigStore;
use synod::adapters::agent::FnAgentAdapter;
use synod::adapters::session_store::{FileSessionStore, SessionStore};
use synod::adapters::tool::FnToolAdapter;
use synod::adapters::trace_store::FileTraceStore;
use synod::cli::inspect::execute_inspect;
use synod::cli::session::{
    execute_capture, execute_plan, execute_run, execute_start, execute_status,
};
use synod::domain::configuration::{
    CapabilityState, ConfigFile, EffortFallbackPolicy, EffortLevel, ModelRoute, RouteSlot,
    RoutingConfig, RuntimeCapabilityProfile, RuntimeKind, SlotEffortPolicy,
};
use synod::domain::decision::{ActionSelector, DecisionType};
use synod::domain::flow_policy::FlowPolicy;
use synod::domain::goal_plan::{GoalPlan, PlannedTask};
use synod::domain::session::SessionStatus;
use synod::domain::step::{ErrorInfo, Recoverability, StepExecutionResult};
use synod::domain::trace::TraceEventType;
use synod::orchestrator::decision_loop::{DecisionLoop, LoopTerminal};
use synod::orchestrator::flow_inference::infer_flow;
use synod::orchestrator::goal_planner::build_goal_plan;
use synod::registry::agent_registry::AgentRegistry;
use synod::registry::tool_registry::ToolRegistry;

use crate::workspace_fixture::temp_fixture_workspace;

fn temp_workspace(prefix: &str) -> PathBuf {
    let ws = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&ws).unwrap();
    ws
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
                    "// generated by integration adapter\n".to_string()
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
    let tester_workspace = workspace.to_path_buf();
    tools
        .register(
            "tester",
            FnToolAdapter::new(move |_request| {
                if tester_workspace.join("Cargo.toml").exists() {
                    StepExecutionResult::success(json!({"stdout": "tests passed", "exit_code": 0}))
                } else {
                    StepExecutionResult::success(
                        json!({"stdout": "validation skipped", "exit_code": 0}),
                    )
                }
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

    DecisionLoop::new(agents, tools, FileTraceStore::for_workspace(workspace), max_steps)
}

/// Full session-native flow: goal → plan → infer → run → inspect
#[test]
fn session_native_full_flow_produces_decisions_and_trace() {
    let ws = temp_workspace("snf-full");

    // Set up a minimal Rust workspace
    fs::write(ws.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    fs::create_dir(ws.join("tests")).unwrap();
    fs::write(ws.join("tests/basic.rs"), "#[test] fn it_works() { assert_eq!(2 + 2, 4); }")
        .unwrap();

    // Step 1: Build goal plan from workspace
    let goal = "fix the broken add function";
    let plan = build_goal_plan(goal, &ws).unwrap();
    assert!(!plan.tasks.is_empty());
    assert!(plan.workspace_signals.language.is_some());

    // Step 2: Infer flow from goal text
    let flow = infer_flow(goal);
    assert!(flow.is_some());
    assert_eq!(flow.as_ref().unwrap().flow_name, "bug-fix");

    // Step 3: Build flow policy
    let policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert!(policy.validate().is_ok());

    // Step 4: Run decision loop
    let dl = build_loop(&ws, 20);
    let (_terminal, decisions, trace) =
        dl.run(&plan, Some(&policy), &ws.to_string_lossy(), "session-snf").unwrap();

    // Step 5: Verify outcomes
    assert!(!decisions.is_empty());
    assert!(!trace.events.is_empty());

    // Trace must contain goal plan creation and decision events
    let event_types: Vec<_> = trace.events.iter().map(|e| e.event_type).collect();
    assert!(event_types.contains(&TraceEventType::GoalPlanCreated));
    assert!(event_types.contains(&TraceEventType::DecisionCreated));
    assert!(event_types.contains(&TraceEventType::TerminalRecorded));

    // All decisions should be in a terminal status
    assert!(decisions.iter().all(|d| d.status.is_terminal()));
}

#[test]
fn blocked_native_run_surfaces_delegation_across_status_and_inspect() {
    let ws = temp_fixture_workspace("snf-delegated-native");
    let mut config = ConfigFile {
        version: 1,
        routing: RoutingConfig {
            implementation: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "sonnet-4".to_string(),
            }),
            assistant_runtimes: vec![RuntimeKind::Codex],
            ..RoutingConfig::default()
        },
    };
    config.routing.slot_effort_policies.insert(
        RouteSlot::Implementation,
        SlotEffortPolicy {
            level: EffortLevel::High,
            fallback: EffortFallbackPolicy::Preserve,
            rationale: Some("keep implementation on the highest-effort bounded path".to_string()),
        },
    );
    config.routing.runtime_capabilities.insert(
        RuntimeKind::Claude,
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Unsupported,
            resume: CapabilityState::Unsupported,
            validation: CapabilityState::Unsupported,
            handoff_target: CapabilityState::Unsupported,
            escalation_context: CapabilityState::Supported,
            notes: Some("requires a handoff for bounded continuation".to_string()),
        },
    );
    config.routing.runtime_capabilities.insert(
        RuntimeKind::Codex,
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: None,
        },
    );
    FileConfigStore::for_workspace(&ws).save_local(&config).unwrap();

    execute_start(Some(&ws)).unwrap();
    execute_capture(Some(&ws), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    let plan = execute_plan(Some(&ws), Some("bug-fix"), false, false).unwrap();
    assert!(plan.terminal_output.contains("runtime_capabilities:"), "{}", plan.terminal_output);
    assert!(plan.terminal_output.contains("slot_effort_policies:"), "{}", plan.terminal_output);
    assert!(plan.terminal_output.contains("routing policy:"), "{}", plan.terminal_output);

    let run = execute_run(Some(&ws)).unwrap();
    assert!(
        run.terminal_output.contains("delegation_mode: handoff_required"),
        "{}",
        run.terminal_output
    );
    assert!(
        run.terminal_output.contains("delegation_packet_kind: handoff"),
        "{}",
        run.terminal_output
    );
    assert!(
        run.terminal_output.contains("delegation_target_owner: codex"),
        "{}",
        run.terminal_output
    );

    let status = execute_status(Some(&ws)).unwrap();
    assert!(
        status.terminal_output.contains("delegation_mode: handoff_required"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("next_command: synod status"),
        "{}",
        status.terminal_output
    );

    let inspect = execute_inspect(None, Some(&ws)).unwrap();
    assert!(
        inspect.terminal_output.contains("delegation_mode: handoff_required"),
        "{}",
        inspect.terminal_output
    );
    assert!(
        inspect.terminal_output.contains("runtime_capabilities:"),
        "{}",
        inspect.terminal_output
    );
    assert!(
        inspect.terminal_output.contains("slot_effort_policies:"),
        "{}",
        inspect.terminal_output
    );

    let record = FileSessionStore::for_workspace(&ws).load().unwrap().unwrap();
    let goal_plan = record.goal_plan.as_ref().expect("delegated goal plan should persist");
    let continuity =
        goal_plan.delegation_continuity().expect("delegation continuity should persist");
    assert_eq!(continuity.mode.as_str(), "handoff_required");
    assert_eq!(continuity.next_command, "synod status");
    assert_eq!(record.latest_status, SessionStatus::Planned);
}

/// Decision loop with flow inference, policy constraints, and stage transitions
#[test]
fn decision_loop_with_flow_inference_and_policy() {
    let ws = temp_workspace("snf-policy");

    fs::create_dir_all(ws.join("src")).unwrap();
    fs::write(ws.join("src/main.rs"), "fn main() {}").unwrap();

    let goal = "implement a dashboard feature";
    let flow = infer_flow(goal).unwrap();
    assert_eq!(flow.flow_name, "change");

    let plan = GoalPlan::new(
        goal,
        vec![PlannedTask {
            task_id: "t1".to_string(),
            description: "Analyze requirements".to_string(),
            target: "src/main.rs".to_string(),
            expected_outcome: Some("requirements understood".to_string()),
            decision_type_hint: Some(DecisionType::Analyze),
        }],
    )
    .unwrap();

    let policy = FlowPolicy::from_builtin(&flow.flow_name).unwrap();
    let dl = build_loop(&ws, 10);
    let (_terminal, decisions, _trace) =
        dl.run(&plan, Some(&policy), &ws.to_string_lossy(), "session-policy").unwrap();

    assert!(!decisions.is_empty());
}

/// Recovery path: verification failure triggers fix decision
#[test]
fn decision_loop_recovery_on_failure() {
    let ws = temp_workspace("snf-recovery");

    // Nonexistent file → Analyze will fail → recovery
    let plan = GoalPlan::new(
        "analyze missing code",
        vec![PlannedTask {
            task_id: "t1".to_string(),
            description: "Analyze missing file".to_string(),
            target: "src/missing.rs".to_string(),
            expected_outcome: Some("contents read".to_string()),
            decision_type_hint: Some(DecisionType::Analyze),
        }],
    )
    .unwrap();

    let dl = build_loop(&ws, 5);
    let (terminal, decisions, trace) =
        dl.run(&plan, None, &ws.to_string_lossy(), "session-recover").unwrap();

    assert!(matches!(terminal, LoopTerminal::NoActionableState(_)));
    assert_eq!(decisions[0].selector_kind(), ActionSelector::Search);
    assert_eq!(decisions[1].selector_kind(), ActionSelector::Ask);

    let event_types: Vec<_> = trace.events.iter().map(|e| e.event_type).collect();
    assert!(event_types.contains(&TraceEventType::DecisionFailed));
    assert!(event_types.contains(&TraceEventType::DecisionRecovered));
}

/// Exhaustion terminal at step limit
#[test]
fn decision_loop_exhaustion_at_step_limit() {
    let ws = temp_workspace("snf-exhaust");

    fs::create_dir_all(ws.join("src")).unwrap();
    fs::write(ws.join("src/a.rs"), "a").unwrap();
    fs::write(ws.join("src/b.rs"), "b").unwrap();

    let plan = GoalPlan::new(
        "process files",
        vec![
            PlannedTask {
                task_id: "t1".to_string(),
                description: "Analyze a".to_string(),
                target: "src/a.rs".to_string(),
                expected_outcome: Some("done".to_string()),
                decision_type_hint: Some(DecisionType::Analyze),
            },
            PlannedTask {
                task_id: "t2".to_string(),
                description: "Analyze b".to_string(),
                target: "src/b.rs".to_string(),
                expected_outcome: Some("done".to_string()),
                decision_type_hint: Some(DecisionType::Analyze),
            },
        ],
    )
    .unwrap();

    let dl = build_loop(&ws, 1);
    let (terminal, decisions, _trace) =
        dl.run(&plan, None, &ws.to_string_lossy(), "session-exhaust").unwrap();

    assert!(matches!(terminal, LoopTerminal::Exhausted { steps_taken: 1, max_steps: 1 }));
    assert_eq!(decisions.len(), 1);
}

#[test]
fn cli_plan_persists_goal_plan_and_proposed_flow_before_confirmation() {
    let ws = temp_workspace("snf-cli-plan-proposed");

    fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"snf_cli_plan_proposed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::write(
        ws.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
    )
    .unwrap();

    execute_start(Some(&ws)).unwrap();
    execute_capture(Some(&ws), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    let planned = execute_plan(Some(&ws), None, false, false).unwrap();
    assert!(
        planned
            .terminal_output
            .contains("proposed `bug-fix` flow is persisted and awaiting plan confirmation"),
        "{}",
        planned.terminal_output
    );
    assert!(
        planned
            .terminal_output
            .contains("execution_path: native_goal_plan_pending_plan_confirmation"),
        "{}",
        planned.terminal_output
    );
    assert!(
        planned.terminal_output.contains(
            "execution_condition: blocked - plan confirmation is still pending before native execution"
        ),
        "{}",
        planned.terminal_output
    );

    let status = execute_status(Some(&ws)).unwrap();
    assert!(
        status
            .terminal_output
            .contains("execution_path: native_goal_plan_pending_plan_confirmation"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains(
            "execution_condition: blocked - plan confirmation is still pending before native execution"
        ),
        "{}",
        status.terminal_output
    );

    let record = FileSessionStore::for_workspace(&ws).load().unwrap().unwrap();
    assert!(record.goal_plan.is_some());
    assert!(record.active_task.is_none());
    let flow = record.goal_plan.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(flow.flow_name, "bug-fix");
    assert!(!flow.confirmed);
}

#[test]
fn cli_plan_blocks_when_context_pack_is_not_credible() {
    let ws = temp_workspace("snf-cli-plan-blocked-context");

    execute_start(Some(&ws)).unwrap();
    let mut record = FileSessionStore::for_workspace(&ws).load().unwrap().unwrap();
    record.goal = Some("investigate a thing".to_string());
    record.latest_status = SessionStatus::GoalCaptured;
    FileSessionStore::for_workspace(&ws).persist(&record).unwrap();

    let error = execute_plan(Some(&ws), None, false, false).unwrap_err();
    let error_text = error.to_string();
    assert!(error_text.contains("bounded context required before planning"), "{error_text}");

    let status = execute_status(Some(&ws)).unwrap();
    assert!(status.terminal_output.contains("context_credibility: insufficient"));
    assert!(status.terminal_output.contains("context_summary: no credible bounded context"));
    assert!(
        status.terminal_output.contains("next_command: synod capture --goal <narrower goal>"),
        "{}",
        status.terminal_output
    );
}

#[test]
fn session_native_cli_surfaces_context_projection_on_status_run_and_inspect() {
    let ws = temp_workspace("snf-cli-context-projection");

    fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"snf_cli_context_projection\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::create_dir_all(ws.join("tests")).unwrap();
    fs::write(
        ws.join("src/context_router.rs"),
        "pub fn build_context_router() -> &'static str { \"ok\" }\n",
    )
    .unwrap();
    fs::write(
        ws.join("src/lib.rs"),
        "pub mod context_router;\npub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        ws.join("tests/basic.rs"),
        "#[test]\nfn it_works() { assert_eq!(snf_cli_context_projection::add(2, 2), 4); }\n",
    )
    .unwrap();

    execute_start(Some(&ws)).unwrap();
    execute_capture(Some(&ws), Some("build a context router"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&ws), None, false, false).unwrap();
    execute_plan(Some(&ws), None, false, true).unwrap();

    let status = execute_status(Some(&ws)).unwrap();
    assert!(status.terminal_output.contains("context_summary:"), "{}", status.terminal_output);
    assert!(
        status.terminal_output.contains("context_credibility: credible"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("context_primary_inputs:"),
        "{}",
        status.terminal_output
    );

    let run = execute_run(Some(&ws)).unwrap();
    assert!(run.terminal_output.contains("context_summary:"), "{}", run.terminal_output);

    let inspect = execute_inspect(None, Some(&ws)).unwrap();
    assert!(inspect.terminal_output.contains("context_summary:"), "{}", inspect.terminal_output);
    assert!(
        inspect.terminal_output.contains("context_credibility: credible"),
        "{}",
        inspect.terminal_output
    );
}

#[test]
fn cli_plan_supports_explicit_no_flow_for_native_session() {
    let ws = temp_workspace("snf-cli-plan-no-flow");

    fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"snf_cli_plan_no_flow\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn summarize() -> &'static str {\n    \"todo\"\n}\n")
        .unwrap();

    execute_start(Some(&ws)).unwrap();
    execute_capture(
        Some(&ws),
        Some("implement workspace summary output"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let planned = execute_plan(Some(&ws), None, true, false).unwrap();
    assert!(
        planned.terminal_output.contains("operator-skipped flow constraints"),
        "{}",
        planned.terminal_output
    );
    assert!(
        planned.terminal_output.contains("execution_path: native_goal_plan"),
        "{}",
        planned.terminal_output
    );
    assert!(
        planned.terminal_output.contains(
            "execution_condition: waiting - planning is complete and execution can begin"
        ),
        "{}",
        planned.terminal_output
    );

    let status = execute_status(Some(&ws)).unwrap();
    assert!(
        status.terminal_output.contains("execution_path: native_goal_plan"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains(
            "execution_condition: waiting - planning is complete and execution can begin"
        ),
        "{}",
        status.terminal_output
    );
    assert!(status.terminal_output.contains("flow_state: skipped"), "{}", status.terminal_output);

    let record = FileSessionStore::for_workspace(&ws).load().unwrap().unwrap();
    assert!(record.goal_plan.is_some());
    assert!(record.goal_plan.as_ref().unwrap().flow.is_none());
    assert!(record.goal_plan.as_ref().unwrap().flow_skipped);
    assert!(record.active_flow.is_none());
    assert!(record.active_flow_policy.is_none());
}

#[test]
fn cli_session_native_run_persists_decisions_and_applies_real_changes() {
    let ws = temp_workspace("snf-cli-e2e");

    fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"snf_cli_e2e\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::create_dir_all(ws.join("tests")).unwrap();
    fs::write(
        ws.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
    )
    .unwrap();
    fs::write(
        ws.join("tests/addition.rs"),
        "#[test]\nfn red_to_green_addition() {\n    assert_eq!(snf_cli_e2e::add(2, 2), 4);\n}\n",
    )
    .unwrap();

    execute_start(Some(&ws)).unwrap();
    execute_capture(Some(&ws), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&ws), Some("bug-fix"), false, false).unwrap();

    let run = execute_run(Some(&ws)).unwrap();
    assert!(run.terminal_output.contains("terminal_status: succeeded"), "{}", run.terminal_output);
    assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);

    let status = execute_status(Some(&ws)).unwrap();
    assert!(
        status.terminal_output.contains("latest_status: succeeded"),
        "{}",
        status.terminal_output
    );

    let record = FileSessionStore::for_workspace(&ws).load().unwrap().unwrap();
    assert!(!record.decisions.is_empty());
    assert!(record.decisions.iter().all(|decision| decision.tool_result.is_some()));
    assert!(fs::read_to_string(ws.join("src/lib.rs")).unwrap().contains("left + right"));
}
