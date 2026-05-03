use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use boundline::FileTraceStore;
use boundline::adapters::trace_store::TraceStore;
use boundline::domain::session::ActiveSessionRecord;
use boundline::domain::trace::TraceEventType;
use serde_json::json;
use uuid::Uuid;

fn run_limits(max_steps: usize, max_retries: usize) -> serde_json::Value {
    json!({
        "max_steps": max_steps,
        "max_retries": max_retries,
        "max_replans": 0,
        "terminal_precedence": [
            "goal_satisfied",
            "unrecoverable_error",
            "task_not_credible",
            "retry_budget_exhausted",
            "replan_budget_exhausted",
            "step_limit_exceeded",
            "no_credible_next_step"
        ]
    })
}

fn execution_profile(name: &str, max_steps: usize, max_retries: usize) -> serde_json::Value {
    json!({
        "name": name,
        "read_targets": ["fixture-target.txt"],
        "validation_command": {
            "program": "sh",
            "args": ["-c", "grep -q green fixture-target.txt"]
        },
        "limits": run_limits(max_steps, max_retries),
        "attempts": [
            {
                "attempt_id": "fix-target",
                "summary": "Replace red with green",
                "failure_mode": "terminal",
                "changes": [
                    {"path": "fixture-target.txt", "find": "red", "replace": "green"}
                ]
            }
        ]
    })
}

fn temp_workspace() -> PathBuf {
    temp_workspace_with_retries(0)
}

fn temp_workspace_with_retries(max_retries: usize) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("boundline-flow-run-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"boundline-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(workspace.join("fixture-target.txt"), "red\n").unwrap();
    fs::write(
        workspace.join(".boundline").join("execution.json"),
        serde_json::to_vec_pretty(&execution_profile("flow-run", 8, max_retries)).unwrap(),
    )
    .unwrap();
    workspace
}

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    serde_json::from_slice(&fs::read(workspace.join(".boundline").join("session.json")).unwrap())
        .unwrap()
}

fn persist_session_record(workspace: &std::path::Path, record: &ActiveSessionRecord) {
    fs::write(
        workspace.join(".boundline").join("session.json"),
        serde_json::to_vec_pretty(record).unwrap(),
    )
    .unwrap();
}

#[test]
fn bug_fix_flow_run_reports_failed_decisions_and_trace_guidance() {
    let workspace = temp_workspace();
    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let planned_status = terminal_text(&run_boundline_in(&workspace, &["status"]));
    assert!(planned_status.contains("current_stage: investigate"), "{planned_status}");
    assert!(planned_status.contains("stage_progress: 1/3"), "{planned_status}");

    let run_output = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(run_text.contains("terminal_status: failed"), "{run_text}");
    assert!(run_text.contains("next_command: boundline inspect"), "{run_text}");

    let final_status = terminal_text(&run_boundline_in(&workspace, &["status"]));
    assert!(final_status.contains("latest_status: failed"), "{final_status}");
    assert!(final_status.contains("current_stage: investigate"), "{final_status}");
    assert!(final_status.contains("stage_progress: 1/3"), "{final_status}");

    let inspect_output = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect_output);
    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: failed"), "{inspect_text}");
}

#[test]
fn delivery_flow_preserves_stage_projection_when_native_delivery_change_is_unavailable() {
    let workspace = temp_workspace();
    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Deliver the checkout fix end to end"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "delivery"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run_output = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_status: failed"), "{run_text}");
    assert!(run_text.contains("recovery decision for fixture-target.txt failed"), "{run_text}");

    let status_text = terminal_text(&run_boundline_in(&workspace, &["status"]));
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("active_flow: delivery"), "{status_text}");
    assert!(status_text.contains("current_stage: requirements"), "{status_text}");
    assert!(status_text.contains("stage_progress: 1/4"), "{status_text}");

    let inspect_output = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect_output);
    assert_eq!(inspect_output.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: failed"), "{inspect_text}");
}

#[test]
fn bug_fix_recovery_decision_is_recorded_after_initial_fix_failure() {
    let workspace = temp_workspace_with_retries(1);
    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run_output = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("recovery decision for "), "{run_text}");
    assert!(run_text.contains(" failed"), "{run_text}");

    let record = load_session_record(&workspace);
    let trace_path = workspace.join(record.latest_trace_ref.expect("trace reference should exist"));
    let trace = FileTraceStore::for_workspace(&workspace).load(&trace_path).unwrap();
    let failed_decisions = trace
        .events
        .iter()
        .filter(|event| event.event_type == TraceEventType::DecisionFailed)
        .count();
    assert_eq!(
        failed_decisions, 1,
        "expected one failed fix decision before explicit recovery stop"
    );

    let recovery_decision =
        trace
            .events
            .iter()
            .find(|event| {
                event.event_type == TraceEventType::DecisionCreated
                    && event.payload.get("rationale").and_then(|value| value.as_str()).is_some_and(
                        |rationale| rationale.starts_with("Replan after failed decision"),
                    )
            })
            .expect("recovery decision should be recorded");
    assert_eq!(
        recovery_decision.payload.get("decision_type").and_then(|value| value.as_str()),
        Some("fix")
    );
    assert_eq!(
        recovery_decision.payload.get("selector").and_then(|value| value.as_str()),
        Some("replan")
    );
}

#[test]
fn flow_cannot_be_replaced_once_a_plan_exists() {
    let workspace = temp_workspace();
    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let output = run_boundline_in(&workspace, &["flow", "change"]);
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("cannot replace active flow `bug-fix` with `change`"), "{text}");
    assert!(text.contains("next_command: boundline start"), "{text}");
}

#[test]
fn invalid_flow_state_requires_a_new_session_before_stage_execution_resumes() {
    let workspace = temp_workspace();
    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let mut record = load_session_record(&workspace);
    let active_flow = record.active_flow.as_mut().expect("flow should be present");
    active_flow.current_stage_index = active_flow.total_stages;
    active_flow.current_stage_id = "corrupted-stage".to_string();
    persist_session_record(&workspace, &record);

    let output = run_boundline_in(&workspace, &["step"]);
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("active session is invalid: session flow state is invalid"), "{text}");
    assert!(text.contains("invalid stage index"), "{text}");
    assert!(text.contains("next_command: boundline start"), "{text}");
}
