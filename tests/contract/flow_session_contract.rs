use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::json;
use synod::domain::session::ActiveSessionRecord;
use uuid::Uuid;

fn run_limits(max_steps: usize) -> serde_json::Value {
    json!({
        "max_steps": max_steps,
        "max_retries": 0,
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

fn execution_profile(name: &str, max_steps: usize) -> serde_json::Value {
    json!({
        "name": name,
        "read_targets": ["fixture-target.txt"],
        "validation_command": {
            "program": "sh",
            "args": ["-c", "grep -q green fixture-target.txt"]
        },
        "limits": run_limits(max_steps),
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
    let workspace = std::env::temp_dir().join(format!("synod-flow-session-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".synod")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(workspace.join("fixture-target.txt"), "red\n").unwrap();
    fs::write(
        workspace.join(".synod").join("execution.json"),
        serde_json::to_vec_pretty(&execution_profile("flow-session", 8)).unwrap(),
    )
    .unwrap();
    workspace
}

fn run_synod(workspace: &std::path::Path, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_synod"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0), "{text}");
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    serde_json::from_slice(&fs::read(workspace.join(".synod").join("session.json")).unwrap())
        .unwrap()
}

#[test]
fn delivery_flow_plan_persists_stage_tagged_steps_and_active_flow_state() {
    let workspace = temp_workspace();
    run_synod(&workspace, &["start"]);
    run_synod(&workspace, &["capture", "--goal", "Deliver the checkout fix"]);
    run_synod(&workspace, &["flow", "delivery"]);
    run_synod(&workspace, &["plan"]);

    let record = load_session_record(&workspace);
    record.validate().unwrap();

    let active_flow = record.active_flow.expect("delivery flow should be active");
    assert_eq!(active_flow.flow_name, "delivery");
    assert_eq!(active_flow.current_stage_id, "requirements");
    assert_eq!(active_flow.current_stage_index, 0);
    assert_eq!(active_flow.total_stages, 4);

    let task = record.active_task.expect("planned task should be present");
    let stage_ids: Vec<_> = task
        .plan
        .steps
        .iter()
        .map(|step| {
            step.input
                .get("delivery_flow")
                .and_then(|value| value.get("stage_id"))
                .and_then(|value| value.as_str())
                .expect("stage metadata should exist")
                .to_string()
        })
        .collect();
    assert_eq!(
        stage_ids,
        vec![
            "requirements".to_string(),
            "architecture".to_string(),
            "backlog".to_string(),
            "implementation".to_string(),
            "implementation".to_string(),
        ]
    );
}
