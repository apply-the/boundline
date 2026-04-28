use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::json;
use synod::FileTraceStore;
use synod::adapters::trace_store::TraceStore;
use synod::domain::session::ActiveSessionRecord;
use synod::domain::trace::TraceEventType;
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
    let workspace = std::env::temp_dir().join(format!("synod-flow-run-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".synod")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(workspace.join("fixture-target.txt"), "red\n").unwrap();
    fs::write(
        workspace.join(".synod").join("execution.json"),
        serde_json::to_vec_pretty(&execution_profile("flow-run", 8, max_retries)).unwrap(),
    )
    .unwrap();
    workspace
}

fn run_synod_in(workspace: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod")).args(args).current_dir(workspace).output().unwrap()
}

fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    serde_json::from_slice(&fs::read(workspace.join(".synod").join("session.json")).unwrap())
        .unwrap()
}

fn persist_session_record(workspace: &std::path::Path, record: &ActiveSessionRecord) {
    fs::write(
        workspace.join(".synod").join("session.json"),
        serde_json::to_vec_pretty(record).unwrap(),
    )
    .unwrap();
}

#[test]
fn bug_fix_flow_progresses_stages_and_inspect_reports_transitions() {
    let workspace = temp_workspace();
    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let planned_status = terminal_text(&run_synod_in(&workspace, &["status"]));
    assert!(planned_status.contains("current_stage: investigate"), "{planned_status}");
    assert!(planned_status.contains("stage_progress: 1/3"), "{planned_status}");

    let step_output = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step_output);
    assert_eq!(step_output.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("current_stage: implement"), "{step_text}");
    assert!(step_text.contains("stage_progress: 2/3"), "{step_text}");

    let run_output = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(0), "{run_text}");

    let final_status = terminal_text(&run_synod_in(&workspace, &["status"]));
    assert!(final_status.contains("latest_status: succeeded"), "{final_status}");
    assert!(final_status.contains("current_stage: verify"), "{final_status}");
    assert!(final_status.contains("stage_progress: 3/3"), "{final_status}");

    let inspect_output = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect_output);
    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("flow: bug-fix @ investigate"), "{inspect_text}");
    assert!(inspect_text.contains("stage: investigate -> implement"), "{inspect_text}");
    assert!(inspect_text.contains("stage: implement -> verify"), "{inspect_text}");
}

#[test]
fn delivery_flow_runs_all_four_stages_to_completion() {
    let workspace = temp_workspace();
    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Deliver the checkout fix end to end"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "delivery"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run_output = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(0), "{run_text}");

    let status_text = terminal_text(&run_synod_in(&workspace, &["status"]));
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("active_flow: delivery"), "{status_text}");
    assert!(status_text.contains("current_stage: implementation"), "{status_text}");
    assert!(status_text.contains("stage_progress: 4/4"), "{status_text}");

    let inspect_output = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect_output);
    assert_eq!(inspect_output.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("stage: requirements -> architecture"), "{inspect_text}");
    assert!(inspect_text.contains("stage: architecture -> backlog"), "{inspect_text}");
    assert!(inspect_text.contains("stage: backlog -> implementation"), "{inspect_text}");
}

#[test]
fn bug_fix_retry_is_recorded_against_the_implement_stage() {
    let workspace = temp_workspace_with_retries(1);
    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run_output = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run_output);
    assert_eq!(run_output.status.code(), Some(0), "{run_text}");

    let record = load_session_record(&workspace);
    let trace_path = workspace.join(record.latest_trace_ref.expect("trace reference should exist"));
    let trace = FileTraceStore::for_workspace(&workspace).load(&trace_path).unwrap();
    let retry_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::StageRetryScheduled)
        .expect("stage retry should be recorded");

    assert_eq!(retry_event.step_id.as_deref(), Some("implement"));
    assert_eq!(
        retry_event
            .payload
            .get("flow")
            .and_then(|value| value.get("stage_id"))
            .and_then(|value| value.as_str()),
        Some("implement")
    );
    assert!(run_text.contains("stage retry for implement"), "{run_text}");
}

#[test]
fn flow_cannot_be_replaced_once_a_plan_exists() {
    let workspace = temp_workspace();
    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let output = run_synod_in(&workspace, &["flow", "change"]);
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("cannot replace active flow `bug-fix` with `change`"), "{text}");
    assert!(text.contains("next_command: synod start"), "{text}");
}

#[test]
fn invalid_flow_state_requires_a_new_session_before_stage_execution_resumes() {
    let workspace = temp_workspace();
    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let mut record = load_session_record(&workspace);
    let active_flow = record.active_flow.as_mut().expect("flow should be present");
    active_flow.current_stage_index = active_flow.total_stages;
    active_flow.current_stage_id = "corrupted-stage".to_string();
    persist_session_record(&workspace, &record);

    let output = run_synod_in(&workspace, &["step"]);
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("active session is invalid: session flow state is invalid"), "{text}");
    assert!(text.contains("invalid stage index"), "{text}");
    assert!(text.contains("next_command: synod start"), "{text}");
}
