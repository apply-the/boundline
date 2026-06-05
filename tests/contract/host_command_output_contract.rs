use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::thread;
use std::time::Duration;

use boundline::FileConfigStore;
use boundline::domain::configuration::{ConfigFile, RoutingConfig};
use boundline::domain::domain_templates::{
    DomainFamily, DomainTemplateSettings, ExternalContextBinding, ExternalContextKind,
};
use boundline::domain::session::SessionStatus;
use serde_json::{Value, json};

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
            "Success criteria: governed planning emits one stage request at a time with resume metadata.\n",
            "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
        ),
    )
    .unwrap();

    let canon_command = write_pending_planning_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&json!({
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

fn stale_plan_quality_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_fixture_workspace(prefix);
    fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#).unwrap();
    fs::create_dir_all(workspace.join("src/components")).unwrap();
    fs::create_dir_all(workspace.join("design")).unwrap();
    fs::write(workspace.join("design/reference.md"), "button guidance\n").unwrap();
    thread::sleep(Duration::from_millis(20));
    fs::write(
        workspace.join("src/components/App.tsx"),
        "export function App() { return <button>Save</button>; }\n",
    )
    .unwrap();
    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            version: 1,
            routing: RoutingConfig {
                domain_templates: std::collections::BTreeMap::from([(
                    DomainFamily::React,
                    DomainTemplateSettings {
                        enabled: Some(true),
                        standards: Some("workspace react standards".to_string()),
                        external_context_bindings: vec![ExternalContextBinding {
                            kind: ExternalContextKind::DesignReference,
                            reference: "design/reference.md".to_string(),
                            required: true,
                            notes: None,
                        }],
                    },
                )]),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        })
        .unwrap();
    workspace
}

fn write_pending_planning_canon_command(workspace: &Path) -> PathBuf {
    let packet_dir = workspace.join(".canon/planning-packet");
    fs::create_dir_all(&packet_dir).unwrap();
    fs::write(packet_dir.join("brief.md"), "# Planning Packet\n\nAwaiting operator approval.\n")
        .unwrap();

    let response = json!({
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

fn workspace_session_json(workspace: &Path) -> Value {
    serde_json::from_slice(&fs::read(workspace.join(".boundline/session.json")).unwrap()).unwrap()
}

fn planning_stage_record<'a>(session_json: &'a Value, stage_key: &str) -> &'a Value {
    session_json["governance_lifecycle"]["stage_records"]
        .as_array()
        .unwrap_or_else(|| panic!("missing governance stage records: {session_json}"))
        .iter()
        .find(|record| record["stage_key"] == stage_key)
        .unwrap_or_else(|| panic!("missing planning stage record for {stage_key}: {session_json}"))
}

fn assert_planning_brief_exists(workspace: &Path, mode: &str) {
    let brief_path = workspace.join(format!(".boundline/governance/planning/{mode}/brief.md"));
    assert!(brief_path.exists(), "missing planning brief: {}", brief_path.display());
}

#[test]
fn session_lifecycle_commands_can_emit_structured_host_output() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-session");

    let goal =
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing add test", "--json"]);
    let goal_text = terminal_text(&goal);
    assert_eq!(goal.status.code(), Some(0), "{goal_text}");
    let goal_json: Value = stdout_json(&goal);
    assert_eq!(goal_json["command_name"], "goal", "{goal_text}");
    assert_eq!(goal_json["exit_status"], "succeeded", "{goal_text}");
    assert_eq!(goal_json["session_status"]["goal"], "Fix the failing add test", "{goal_text}");

    let plan = run_boundline_in(&workspace, &["plan", "--json"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    let plan_json: Value = stdout_json(&plan);
    assert_eq!(plan_json["command_name"], "plan", "{plan_text}");
    assert_eq!(plan_json["session_status"]["latest_status"], "planned", "{plan_text}");
    assert_eq!(plan_json["session_status"]["plan_quality_state"], "ready", "{plan_text}");
    assert!(plan_json["session_status"]["planning_analysis_state"].is_string(), "{plan_text}");
    assert!(plan_json["session_status"]["planning_analysis_coverage"].is_object(), "{plan_text}");

    let status = run_boundline_in(&workspace, &["status", "--json"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    let status_json: Value = stdout_json(&status);
    assert_eq!(status_json["command_name"], "status", "{status_text}");
    assert!(status_json["session_status"]["next_command"].is_string(), "{status_text}");
    assert_eq!(status_json["session_status"]["plan_quality_state"], "ready", "{status_text}");
    assert!(status_json["session_status"]["planning_analysis_state"].is_string(), "{status_text}");
    assert!(
        status_json["session_status"]["planning_analysis_coverage"].is_object(),
        "{status_text}"
    );

    let next = run_boundline_in(&workspace, &["next", "--json"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    let next_json: Value = stdout_json(&next);
    assert_eq!(next_json["command_name"], "next", "{next_text}");
    assert!(next_json["session_status"]["next_command"].is_string(), "{next_text}");
}

#[test]
fn run_and_inspect_can_emit_structured_trace_output() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-trace");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run", "--json"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    let run_json: Value = stdout_json(&run);
    assert_eq!(run_json["command_name"], "run", "{run_text}");
    assert_eq!(run_json["exit_status"], "succeeded", "{run_text}");
    assert_eq!(run_json["trace_summary"]["terminal_status"], "succeeded", "{run_text}");
    assert!(run_json["trace_location"].is_string(), "{run_text}");

    let inspect = run_boundline_in(&workspace, &["inspect", "--json"]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    let inspect_json: Value = stdout_json(&inspect);
    assert_eq!(inspect_json["command_name"], "inspect", "{inspect_text}");
    assert_eq!(inspect_json["trace_summary"]["terminal_status"], "succeeded", "{inspect_text}");
    assert!(inspect_json["trace_summary"]["trace_ref"].is_string(), "{inspect_text}");
}

#[test]
fn invalid_invocations_can_emit_structured_host_output() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-invalid");

    let goal = run_boundline_in(&workspace, &["goal", "--goal", "   ", "--json"]);
    let goal_text = terminal_text(&goal);
    assert_eq!(goal.status.code(), Some(2), "{goal_text}");

    let goal_json: Value = stdout_json(&goal);
    assert_eq!(goal_json["command_name"], "goal", "{goal_text}");
    assert_eq!(goal_json["exit_status"], "invalid_invocation", "{goal_text}");
    assert_eq!(goal_json["rendered_output"], "goal requires a non-empty --goal", "{goal_text}");
    assert!(goal_json["trace_location"].is_null(), "{goal_text}");
    assert!(goal_json["session_status"].is_null(), "{goal_text}");
    assert!(goal_json["trace_summary"].is_null(), "{goal_text}");
}

#[test]
fn goal_captured_status_output_surfaces_clarification_guidance() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-partial");

    let goal = run_boundline_in(
        &workspace,
        &["goal", "--goal", "Explain why this delivery is safe", "--json"],
    );
    let goal_text = terminal_text(&goal);
    assert_eq!(goal.status.code(), Some(0), "{goal_text}");

    let status = run_boundline_in(&workspace, &["status", "--json"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");

    let status_json: Value = stdout_json(&status);
    assert_eq!(status_json["session_status"]["latest_status"], "goal_captured", "{status_text}");
    assert_eq!(
        status_json["session_status"]["goal_quality_state"], "clarification_required",
        "{status_text}"
    );
    assert!(
        status_json["session_status"]["goal_quality_findings"]
            .as_array()
            .is_some_and(|findings| !findings.is_empty()),
        "{status_text}"
    );
    let rendered = status_json["rendered_output"].as_str().unwrap_or_default();
    assert!(rendered.contains("goal_quality_state: clarification_required"), "{status_text}");
    assert!(rendered.contains("goal_quality_findings:"), "{status_text}");
    assert!(rendered.contains("clarification_headline:"), "{status_text}");
    assert!(rendered.contains("clarification_questions:"), "{status_text}");
    assert!(
        rendered.contains("next_command: boundline goal --goal <narrower goal>"),
        "{status_text}"
    );
}

#[test]
fn inspect_output_surfaces_runtime_source_attribution() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-inspect");
    fs::write(
        workspace.join("brief.md"),
        concat!(
            "Explain the runtime source attribution for src/lib.rs.\n\n",
            "Authoritative persistence store: workspace-local .boundline/session.json.\n",
            "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
            "In-scope API operations: goal, plan, run, status, and inspect for the first slice.\n",
            "Domain entities in scope: session, plan brief, run brief, and trace summary.\n",
            "Success criteria: inspect output shows runtime source attribution and actionable fallback disclosure.\n",
            "Validation target: cargo test --test contract host_command_output_contract -- --test-threads=1.\n",
        ),
    )
    .unwrap();

    assert_eq!(
        run_boundline_in(
            &workspace,
            &["goal", "--goal", "Fix the failing add test", "--brief", "brief.md"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect = run_boundline_in(&workspace, &["inspect", "--json"]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");

    let inspect_json: Value = stdout_json(&inspect);
    let rendered = inspect_json["rendered_output"].as_str().unwrap_or_default();
    assert!(rendered.contains("source_attribution: runtime="), "{inspect_text}");
    assert!(
        rendered.contains(
            "fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only"
        ),
        "{inspect_text}"
    );
    assert!(rendered.contains("next_best_action:"), "{inspect_text}");
}

#[test]
fn orchestrate_continues_through_plan_to_execution_without_phase_request() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-orchestrate");

    let orchestrate = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Prepare an architecture brief for the failing add test",
            "--json-stream",
        ],
    );
    let orchestrate_text = terminal_text(&orchestrate);
    assert_eq!(orchestrate.status.code(), Some(0), "{orchestrate_text}");

    let frames = stdout_json_lines(&orchestrate);
    assert!(!frames.is_empty(), "{orchestrate_text}");
    assert_eq!(frames[0]["event_kind"], "session_opened", "{orchestrate_text}");
    assert!(frames.iter().any(|frame| frame["event_kind"] == "artifact_recorded"));
    assert!(
        frames
            .iter()
            .any(|frame| frame["event_kind"] == "phase_started"
                && frame["phase_kind"] == "execution"),
        "orchestrate should proceed to execution without stopping at plan: {orchestrate_text}"
    );
}

#[test]
fn orchestrate_json_host_output_uses_human_report_rendering() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-orchestrate-json");

    let orchestrate = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Prepare an architecture brief for the failing add test",
            "--json",
        ],
    );
    let orchestrate_text = terminal_text(&orchestrate);
    assert_eq!(orchestrate.status.code(), Some(0), "{orchestrate_text}");

    let orchestrate_json: Value = stdout_json(&orchestrate);
    let rendered = orchestrate_json["rendered_output"].as_str().unwrap_or_default();
    assert!(rendered.contains("Session: "), "{orchestrate_text}");
    assert!(rendered.contains("Status: Succeeded"), "{orchestrate_text}");
    assert!(!rendered.contains("legacy terminal output"), "{orchestrate_text}");
}

#[test]
fn orchestrate_resume_stream_uses_session_resumed_event() {
    let workspace = temp_fixture_workspace("boundline-host-command-contract-orchestrate-resume");

    let first = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Prepare an architecture brief for the failing add test",
            "--json-stream",
        ],
    );
    let first_text = terminal_text(&first);
    assert_eq!(first.status.code(), Some(0), "{first_text}");

    let resume = run_boundline_in(
        &workspace,
        &["orchestrate", "--intent", "continue-until-phase-request", "--json-stream"],
    );
    let resume_text = terminal_text(&resume);

    let resume_frames = stdout_json_lines(&resume);
    assert!(!resume_frames.is_empty(), "{resume_text}");
    assert_eq!(resume_frames[0]["event_kind"], "session_resumed", "{resume_text}");
}

#[test]
fn orchestrate_plan_quality_block_emits_one_phase_request_and_withholds_execution() {
    let workspace = stale_plan_quality_workspace("boundline-host-command-contract-plan-quality");

    let orchestrate = run_boundline_in(
        &workspace,
        &[
            "orchestrate",
            "--goal",
            "Refresh src/components/App.tsx against the latest design guidance",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ],
    );
    let orchestrate_text = terminal_text(&orchestrate);
    assert_eq!(orchestrate.status.code(), Some(0), "{orchestrate_text}");

    let frames = stdout_json_lines(&orchestrate);
    let phase_requests =
        frames.iter().filter(|frame| frame["event_kind"] == "phase_request").collect::<Vec<_>>();
    assert_eq!(phase_requests.len(), 1, "{orchestrate_text}");
    assert_eq!(phase_requests[0]["stage_key"], "plan", "{orchestrate_text}");
    assert_eq!(
        phase_requests[0]["session_status"]["latest_status"],
        serde_json::to_value(SessionStatus::Blocked).unwrap(),
        "{orchestrate_text}"
    );
    assert_eq!(
        phase_requests[0]["session_status"]["plan_quality_state"], "blocked",
        "{orchestrate_text}"
    );
    assert!(
        !frames
            .iter()
            .any(|frame| frame["event_kind"] == "phase_started"
                && frame["phase_kind"] == "execution"),
        "{orchestrate_text}"
    );
}

#[test]
fn orchestrate_can_advance_ndjson_planning_stage_phase_requests_one_stage_at_a_time() {
    let workspace =
        governed_planning_workspace("boundline-host-command-contract-governed-orchestrate");

    let orchestrate = run_boundline_in(
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
    let orchestrate_text = terminal_text(&orchestrate);
    assert_eq!(orchestrate.status.code(), Some(0), "{orchestrate_text}");

    let frames = stdout_json_lines(&orchestrate);
    let phase_requests =
        frames.iter().filter(|frame| frame["event_kind"] == "phase_request").collect::<Vec<_>>();
    assert_eq!(phase_requests.len(), 1, "{orchestrate_text}");
    assert_eq!(phase_requests[0]["stage_key"], "plan:requirements", "{orchestrate_text}");
    assert_eq!(phase_requests[0]["route_slot"], "planning", "{orchestrate_text}");
    assert_eq!(phase_requests[0]["governance_mode"], "requirements", "{orchestrate_text}");
    assert_eq!(
        phase_requests[0]["artifact"]["artifact_kind"], "canon_packet",
        "{orchestrate_text}"
    );
    assert!(
        phase_requests[0]["artifact"]["artifact_ref"]
            .as_str()
            .unwrap_or_default()
            .contains(".canon/planning-packet"),
        "{orchestrate_text}"
    );
    assert!(
        phase_requests[0]["resume_command"]
            .as_str()
            .unwrap_or_default()
            .contains("--planning-stage-complete plan:requirements --until phase-request"),
        "{orchestrate_text}"
    );
    let first_request_id =
        phase_requests[0]["phase_request"]["request_id"].as_str().unwrap_or_default();
    assert!(!first_request_id.is_empty(), "{orchestrate_text}");
    assert_eq!(
        phase_requests[0]["phase_request"]["expected_answer"]["type"], "confirmation",
        "{orchestrate_text}"
    );
    assert!(
        phase_requests[0]["resume_command"].as_str().unwrap_or_default().contains(first_request_id),
        "{orchestrate_text}"
    );

    assert_planning_brief_exists(&workspace, "requirements");
    assert_planning_brief_exists(&workspace, "system-shaping");
    assert_planning_brief_exists(&workspace, "architecture");
    assert_planning_brief_exists(&workspace, "backlog");

    let first_session = workspace_session_json(&workspace);
    assert_eq!(first_session["governance_lifecycle"]["current_stage_index"], 0, "{first_session}");
    assert!(
        first_session["governance_lifecycle"]["selected_mode_sequence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|mode| mode == "system-shaping"),
        "{first_session}"
    );
    let first_stage_record = planning_stage_record(&first_session, "plan:requirements");
    assert_eq!(first_stage_record["lifecycle_state"], "awaiting_approval", "{first_session}");
    assert_eq!(first_stage_record["approval_state"], "requested", "{first_session}");
    assert_eq!(first_stage_record["canon_run_ref"], "canon-run-plan", "{first_session}");
    assert_eq!(first_stage_record["packet_ref"], ".canon/planning-packet", "{first_session}");

    let resume = run_boundline_in(
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
    let resume_text = terminal_text(&resume);
    assert_eq!(resume.status.code(), Some(0), "{resume_text}");

    let resume_frames = stdout_json_lines(&resume);
    let resume_phase_requests = resume_frames
        .iter()
        .filter(|frame| frame["event_kind"] == "phase_request")
        .collect::<Vec<_>>();
    assert_eq!(resume_phase_requests.len(), 1, "{resume_text}");
    assert_eq!(resume_phase_requests[0]["stage_key"], "plan:architecture", "{resume_text}");
    assert!(
        resume_frames.iter().any(|frame| {
            frame["event_kind"] == "execution_update"
                && frame["stage_key"] == "plan:requirements"
                && frame["message"]
                    == "recorded host completion for planning stage plan:requirements"
        }),
        "{resume_text}"
    );

    assert!(
        resume_phase_requests[0]["resume_command"]
            .as_str()
            .unwrap_or_default()
            .contains("--planning-stage-complete plan:architecture --until phase-request"),
        "{resume_text}"
    );
    let second_request_id =
        resume_phase_requests[0]["phase_request"]["request_id"].as_str().unwrap_or_default();
    assert!(!second_request_id.is_empty(), "{resume_text}");
    assert!(
        resume_phase_requests[0]["resume_command"]
            .as_str()
            .unwrap_or_default()
            .contains(second_request_id),
        "{resume_text}"
    );

    let second_session = workspace_session_json(&workspace);
    assert_eq!(
        second_session["governance_lifecycle"]["current_stage_index"], 1,
        "{second_session}"
    );
    let second_stage_record = planning_stage_record(&second_session, "plan:requirements");
    assert_eq!(second_stage_record["lifecycle_state"], "completed", "{second_session}");
    assert_eq!(second_stage_record["approval_state"], "not_needed", "{second_session}");
    assert_eq!(second_stage_record["canon_run_ref"], "canon-run-plan", "{second_session}");
    assert_eq!(second_stage_record["packet_ref"], ".canon/planning-packet", "{second_session}");
    assert_eq!(
        second_session["governance_lifecycle"]["stage_records"].as_array().map(Vec::len),
        Some(1),
        "{second_session}"
    );

    let resume_architecture = run_boundline_in(
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
    let resume_architecture_text = terminal_text(&resume_architecture);
    assert_eq!(resume_architecture.status.code(), Some(0), "{resume_architecture_text}");

    let resume_architecture_frames = stdout_json_lines(&resume_architecture);
    let resume_architecture_phase_requests = resume_architecture_frames
        .iter()
        .filter(|frame| frame["event_kind"] == "phase_request")
        .collect::<Vec<_>>();
    assert_eq!(resume_architecture_phase_requests.len(), 1, "{resume_architecture_text}");
    assert_eq!(
        resume_architecture_phase_requests[0]["stage_key"], "plan:backlog",
        "{resume_architecture_text}"
    );
    assert_eq!(
        resume_architecture_phase_requests[0]["session_status"]["backlog_quality_state"],
        "clarification_required",
        "{resume_architecture_text}"
    );
    assert!(
        resume_architecture_phase_requests[0]["session_status"]["backlog_quality_findings"]
            .as_array()
            .is_some_and(|findings| findings
                .iter()
                .any(|finding| finding == "backlog_packet_pending")),
        "{resume_architecture_text}"
    );
    assert!(
        resume_architecture_phase_requests[0]["resume_command"]
            .as_str()
            .unwrap_or_default()
            .contains("--planning-stage-complete plan:backlog --until terminal"),
        "{resume_architecture_text}"
    );
    let third_request_id = resume_architecture_phase_requests[0]["phase_request"]["request_id"]
        .as_str()
        .unwrap_or_default();
    assert!(!third_request_id.is_empty(), "{resume_architecture_text}");
    assert!(
        resume_architecture_phase_requests[0]["resume_command"]
            .as_str()
            .unwrap_or_default()
            .contains(third_request_id),
        "{resume_architecture_text}"
    );
    assert!(
        resume_architecture_frames.iter().any(|frame| {
            frame["event_kind"] == "execution_update"
                && frame["stage_key"] == "plan:architecture"
                && frame["message"]
                    == "recorded host completion for planning stage plan:architecture"
        }),
        "{resume_architecture_text}"
    );

    let third_session = workspace_session_json(&workspace);
    assert_eq!(third_session["governance_lifecycle"]["current_stage_index"], 2, "{third_session}");
    let third_stage_record = planning_stage_record(&third_session, "plan:requirements");
    assert_eq!(third_stage_record["lifecycle_state"], "completed", "{third_session}");
    assert_eq!(third_stage_record["approval_state"], "not_needed", "{third_session}");
    assert_eq!(third_stage_record["canon_run_ref"], "canon-run-plan", "{third_session}");
    assert_eq!(third_stage_record["packet_ref"], ".canon/planning-packet", "{third_session}");
    let architecture_stage_record = planning_stage_record(&third_session, "plan:architecture");
    assert_eq!(architecture_stage_record["lifecycle_state"], "completed", "{third_session}");
    assert_eq!(architecture_stage_record["approval_state"], "not_needed", "{third_session}");
    assert_eq!(
        architecture_stage_record["packet_ref"], ".boundline/governance/planning/architecture",
        "{third_session}"
    );
    assert_eq!(
        third_session["governance_lifecycle"]["stage_records"].as_array().map(Vec::len),
        Some(2),
        "{third_session}"
    );
}
