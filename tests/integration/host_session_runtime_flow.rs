use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use serde_json::Value;

use crate::workspace_fixture::{
    initialize_nested_git_repository, run_boundline_in, stdout_json, temp_fixture_workspace,
    terminal_text,
};

fn stdout_json_lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).unwrap_or_else(|error| {
                panic!("failed to parse orchestrate JSON line `{line}`: {error}")
            })
        })
        .collect()
}

fn governed_planning_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_fixture_workspace(prefix);
    initialize_nested_git_repository(&workspace);
    fs::write(
        workspace.join("brief.md"),
        concat!(
            "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n\n",
            "Authoritative persistence store: workspace-local .boundline/session.json.\n",
            "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
            "In-scope API operations: goal, plan, and orchestrate for the first slice.\n",
            "Domain entities in scope: session, plan brief, run brief, and planning stage brief.\n",
            "Success criteria: governed planning emits one reusable stage handoff at a time with resume metadata.\n",
            "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
        ),
    )
    .unwrap();

    let canon_command = write_pending_planning_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "governed-planning-execution",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"]
            },
            "attempts": [
                {
                    "attempt_id": "fix-add",
                    "summary": "Replace subtraction with addition",
                    "failure_mode": "terminal",
                    "changes": [
                        {
                            "path": "src/lib.rs",
                            "find": "left - right",
                            "replace": "left + right"
                        }
                    ]
                }
            ],
            "governance": {
                "default_runtime": "canon",
                "canon": {
                    "command": canon_command.to_string_lossy(),
                    "default_owner": "platform",
                    "default_risk": "medium",
                    "default_zone": "engineering",
                    "default_system_context": "existing"
                },
                "stages": []
            }
        }))
        .unwrap(),
    )
    .unwrap();

    workspace
}

fn write_pending_planning_canon_command(workspace: &Path) -> PathBuf {
    let packet_dir = workspace.join(".canon/planning-packet");
    fs::create_dir_all(&packet_dir).unwrap();
    fs::write(packet_dir.join("brief.md"), "# Planning Packet\n\nAwaiting operator approval.\n")
        .unwrap();

    let response = serde_json::json!({
        "status": "awaiting_approval",
        "approval_state": "requested",
        "run_ref": "canon-run-plan",
        "packet_ref": ".canon/planning-packet",
        "expected_document_refs": [".canon/planning-packet/brief.md"],
        "document_refs": [],
        "packet_readiness": "pending",
        "missing_sections": [],
        "headline": "awaiting planning approval",
        "message": "Canon is waiting for planning approval"
    });
    let command_path = workspace.join(".boundline/canon-plan-stub.sh");
    fs::write(
        &command_path,
        format!("#!/bin/sh\ncat >/dev/null\ncat <<'EOF'\n{}\nEOF\n", response),
    )
    .unwrap();
    let mut permissions = fs::metadata(&command_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&command_path, permissions).unwrap();
    command_path
}

fn write_reusable_backlog_packet(workspace: &Path) {
    let backlog_dir = workspace.join(".boundline/governance/planning/backlog");
    fs::create_dir_all(&backlog_dir).unwrap();
    fs::write(
        backlog_dir.join("backlog.md"),
        concat!(
            "# Backlog\n\n",
            "MVP: hand off one governed backlog packet into terminal execution.\n",
            "Dependencies: T001 -> T002 -> T003.\n\n",
            "- [ ] T001 Record host planning-stage completion in the governed lifecycle.\n",
            "- [ ] T002 Promote reusable backlog artifacts for execution gating.\n",
            "- [ ] T003 Resume execution after the backlog handoff is complete.\n",
        ),
    )
    .unwrap();
}

#[test]
fn structured_session_output_preserves_continuation_state_across_goal_plan_status_and_next() {
    let workspace = temp_fixture_workspace("boundline-host-session-runtime");

    let goal =
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing add test", "--json"]);
    let goal_text = terminal_text(&goal);
    assert_eq!(goal.status.code(), Some(0), "{goal_text}");
    let goal_json: Value = stdout_json(&goal);
    assert_eq!(goal_json["command_name"], "goal", "{goal_text}");
    assert_eq!(goal_json["session_status"]["goal"], "Fix the failing add test", "{goal_text}");

    let plan = run_boundline_in(&workspace, &["plan", "--flow", "bug-fix", "--json"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    let plan_json: Value = stdout_json(&plan);
    assert_eq!(plan_json["session_status"]["latest_status"], "planned", "{plan_text}");
    assert_eq!(plan_json["session_status"]["goal_plan_state"], "confirmed", "{plan_text}");
    assert!(
        plan_json["session_status"]["flow_state"].as_str().unwrap_or_default().contains("bug-fix"),
        "{plan_text}"
    );

    let status = run_boundline_in(&workspace, &["status", "--json"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    let status_json: Value = stdout_json(&status);
    assert_eq!(status_json["session_status"]["latest_status"], "planned", "{status_text}");
    assert!(status_json["session_status"]["next_command"].is_string(), "{status_text}");
    assert!(
        status_json["rendered_output"].as_str().unwrap_or_default().contains("next_command:"),
        "{status_text}"
    );

    let next = run_boundline_in(&workspace, &["next", "--json"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    let next_json: Value = stdout_json(&next);
    assert_eq!(
        next_json["session_status"]["next_command"], status_json["session_status"]["next_command"],
        "{next_text}"
    );
}

#[test]
fn orchestrate_stream_can_run_to_terminal_from_goal_input() {
    let workspace = temp_fixture_workspace("boundline-host-session-runtime-orchestrate");

    let orchestrate = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Fix the failing add test",
            "--intent",
            "continue-until-terminal",
            "--json-stream",
        ],
    );
    let orchestrate_text = terminal_text(&orchestrate);
    assert_eq!(orchestrate.status.code(), Some(0), "{orchestrate_text}");

    let frames = stdout_json_lines(&orchestrate);
    let terminal = frames
        .iter()
        .find(|frame| frame["event_kind"] == "terminal")
        .unwrap_or_else(|| panic!("missing terminal frame: {orchestrate_text}"));
    assert_eq!(terminal["phase_kind"], "execution", "{orchestrate_text}");
    assert_eq!(terminal["stage_key"], "run", "{orchestrate_text}");
    assert_eq!(terminal["trace_summary"]["terminal_status"], "succeeded", "{orchestrate_text}");
    assert!(
        frames.iter().any(|frame| frame["event_kind"] == "artifact_recorded"),
        "{orchestrate_text}"
    );
    assert!(
        !frames.iter().any(|frame| frame["event_kind"] == "phase_request"),
        "{orchestrate_text}"
    );
}

#[test]
fn orchestrate_staged_planning_resume_can_handoff_backlog_into_terminal_execution() {
    let workspace = governed_planning_workspace("boundline-host-session-runtime-governed-terminal");

    let first = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Deliver a governed feature",
            "--brief",
            "brief.md",
            "--flow",
            "delivery",
            "--governance",
            "canon",
            "--risk",
            "medium",
            "--zone",
            "engineering",
            "--owner",
            "platform",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let first_text = terminal_text(&first);
    assert_eq!(first.status.code(), Some(0), "{first_text}");

    let first_frames = stdout_json_lines(&first);
    assert!(
        first_frames.iter().any(|frame| {
            frame["event_kind"] == "phase_request" && frame["stage_key"] == "plan:requirements"
        }),
        "{first_text}"
    );

    let second = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--planning-stage-complete",
            "plan:requirements",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let second_text = terminal_text(&second);
    assert_eq!(second.status.code(), Some(0), "{second_text}");

    let second_frames = stdout_json_lines(&second);
    assert!(
        second_frames.iter().any(|frame| {
            frame["event_kind"] == "phase_request" && frame["stage_key"] == "plan:architecture"
        }),
        "{second_text}"
    );

    let third = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--planning-stage-complete",
            "plan:architecture",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let third_text = terminal_text(&third);
    assert_eq!(third.status.code(), Some(0), "{third_text}");

    let third_frames = stdout_json_lines(&third);
    assert!(
        third_frames.iter().any(|frame| {
            frame["event_kind"] == "phase_request" && frame["stage_key"] == "plan:backlog"
        }),
        "{third_text}"
    );

    write_reusable_backlog_packet(&workspace);

    let fourth = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--planning-stage-complete",
            "plan:backlog",
            "--intent",
            "continue-until-terminal",
            "--json-stream",
        ],
    );
    let fourth_text = terminal_text(&fourth);
    assert_eq!(fourth.status.code(), Some(0), "{fourth_text}");

    let fourth_frames = stdout_json_lines(&fourth);
    assert!(
        fourth_frames.iter().any(|frame| {
            frame["event_kind"] == "execution_update"
                && frame["stage_key"] == "plan:backlog"
                && frame["message"] == "recorded host completion for planning stage plan:backlog"
        }),
        "{fourth_text}"
    );
    let terminal = fourth_frames
        .iter()
        .find(|frame| frame["event_kind"] == "terminal")
        .unwrap_or_else(|| panic!("missing terminal frame: {fourth_text}"));
    assert_eq!(terminal["phase_kind"], "execution", "{fourth_text}");
    assert_eq!(terminal["stage_key"], "run", "{fourth_text}");
    assert_eq!(terminal["trace_summary"]["terminal_status"], "succeeded", "{fourth_text}");
    assert!(
        !fourth_frames.iter().any(|frame| frame["event_kind"] == "phase_request"),
        "{fourth_text}"
    );

    let session: ActiveSessionRecord =
        serde_json::from_slice(&fs::read(workspace.join(".boundline/session.json")).unwrap())
            .unwrap();
    session.validate().unwrap();
    assert_eq!(session.latest_status, SessionStatus::Succeeded, "{session:?}");
    assert!(session.latest_trace_ref.is_some(), "{session:?}");
    assert_eq!(
        session.governance_lifecycle.as_ref().map(|lifecycle| lifecycle.current_stage_index),
        Some(3),
        "{session:?}"
    );
}
