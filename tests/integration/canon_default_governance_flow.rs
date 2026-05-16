use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use boundline::{AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE, SUPPORTED_CANON_VERSION};

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

/// Create a workspace with `[canon]` config preferences and a mock Canon CLI.
fn temp_canon_default_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = temp_fixture_workspace(&format!("{prefix}-canon-default"));
    let boundline_dir = workspace.join(".boundline");

    // Write a config with [canon] section
    fs::write(
        boundline_dir.join("config.toml"),
        r#"[canon]
	mode_selection = "auto-confirm"
	default_risk = "medium"
	default_zone = "engineering"
	default_owner = "platform"
	"#,
    )
    .unwrap();

    workspace
}

/// Create a workspace without `[canon]` config (backward compatibility).
fn temp_no_canon_config_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = temp_fixture_workspace(&format!("{prefix}-no-canon-config"));
    let boundline_dir = workspace.join(".boundline");

    // Config without [canon] section
    fs::write(boundline_dir.join("config.toml"), "").unwrap();

    workspace
}

fn governed_ready_response(
    run_ref: &str,
    packet_ref: &str,
    document_ref: &str,
    headline: &str,
    message: &str,
) -> String {
    serde_json::json!({
        "status": "governed_ready",
        "approval_state": "granted",
        "run_ref": run_ref,
        "packet_ref": packet_ref,
        "expected_document_refs": [document_ref],
        "document_refs": [document_ref],
        "packet_readiness": "reusable",
        "missing_sections": [],
        "headline": headline,
        "authority_governance": {
            "contract_line": AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE,
            "authority_zone": "green",
            "change_class": "low-impact",
            "intended_persona": "delivery-engineer",
            "approval_state": "granted",
            "packet_readiness": "reusable",
            "risk": "low-impact",
            "persona_anti_behaviors": [],
            "artifact_order": [],
            "promotion_refs": [],
            "stage_role_hints": []
        },
        "message": message
    })
    .to_string()
}

#[cfg(unix)]
fn write_ready_canon_on_path(prefix: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin_dir = std::env::temp_dir().join(format!("{prefix}-canon-bin"));
    let _ = fs::remove_dir_all(&bin_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    let canon = bin_dir.join("canon");
    let capabilities = format!(
        r#"{{"canon_version":"{SUPPORTED_CANON_VERSION}","supported_schema_versions":["2026-02-01"],"operations":["start","refresh","capabilities"],"supported_modes":["requirements","discovery","system-shaping","architecture","backlog","change","implementation","refactor","review","verification","pr-review","incident","security-assessment","system-assessment","migration","supply-chain-analysis"],"status_values":["governed_ready","awaiting_approval","blocked"],"approval_state_values":["not_needed","requested","granted"],"packet_readiness_values":["reusable","pending","incomplete"],"compatibility_notes":["stable-json"]}}"#
    );
    fs::write(
        &canon,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'canon version {SUPPORTED_CANON_VERSION}'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{}'\n  exit 0\nfi\nexit 1\n",
            capabilities
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&canon).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&canon, permissions).unwrap();
    bin_dir
}

#[cfg(unix)]
fn write_capturing_canon_on_path(prefix: &str, workspace: &Path, response_json: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin_dir = std::env::temp_dir().join(format!("{prefix}-canon-bin"));
    let _ = fs::remove_dir_all(&bin_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    let canon = bin_dir.join("canon");
    let capture_path = workspace.join(".boundline/canon-request.json");
    let capabilities = format!(
        r#"{{"canon_version":"{SUPPORTED_CANON_VERSION}","supported_schema_versions":["2026-02-01"],"operations":["start","refresh","capabilities"],"supported_modes":["requirements","discovery","system-shaping","architecture","backlog","change","implementation","refactor","review","verification","pr-review","incident","security-assessment","system-assessment","migration","supply-chain-analysis"],"status_values":["governed_ready","awaiting_approval","blocked","pending_selection","incomplete"],"approval_state_values":["not_needed","requested","granted"],"packet_readiness_values":["reusable","pending","incomplete"],"compatibility_notes":["stable-json"]}}"#
    );
    fs::write(
        &canon,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'canon version {SUPPORTED_CANON_VERSION}'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{}'\n  exit 0\nfi\ncat > '{}'\nprintf '%s' '{}'\n",
            capabilities,
            capture_path.display(),
            response_json
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&canon).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&canon, permissions).unwrap();
    bin_dir
}

#[cfg(unix)]
fn write_multi_stage_capturing_canon_on_path(
    prefix: &str,
    workspace: &Path,
    first_response_json: &str,
    second_response_json: &str,
) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin_dir = std::env::temp_dir().join(format!("{prefix}-canon-bin"));
    let _ = fs::remove_dir_all(&bin_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    let canon = bin_dir.join("canon");
    let capture_prefix = workspace.join(".boundline/canon-request");
    let count_path = workspace.join(".boundline/canon-request-count");
    let capabilities = format!(
        r#"{{"canon_version":"{SUPPORTED_CANON_VERSION}","supported_schema_versions":["2026-02-01"],"operations":["start","refresh","capabilities"],"supported_modes":["requirements","discovery","system-shaping","architecture","backlog","change","implementation","refactor","review","verification","pr-review","incident","security-assessment","system-assessment","migration","supply-chain-analysis"],"status_values":["governed_ready","awaiting_approval","blocked","pending_selection","incomplete"],"approval_state_values":["not_needed","requested","granted"],"packet_readiness_values":["reusable","pending","incomplete"],"compatibility_notes":["stable-json"]}}"#
    );
    fs::write(
        &canon,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'canon version {SUPPORTED_CANON_VERSION}'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{}'\n  exit 0\nfi\ncount=0\nif [ -f '{}' ]; then\n  count=$(cat '{}')\nfi\ncount=$((count + 1))\necho \"$count\" > '{}'\ncapture='{}-'$count'.json'\ncat > \"$capture\"\nif [ \"$count\" = \"1\" ]; then\n  printf '%s' '{}'\nelse\n  printf '%s' '{}'\nfi\n",
            capabilities,
            count_path.display(),
            count_path.display(),
            count_path.display(),
            capture_prefix.display(),
            first_response_json,
            second_response_json
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&canon).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&canon, permissions).unwrap();
    bin_dir
}

fn write_canon_execution_profile(workspace: &Path, canon_command: &Path) {
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "canon-default-input-assembly",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"]
            },
            "attempts": [{
                "attempt_id": "fix-add",
                "summary": "Replace subtraction with addition",
                "failure_mode": "terminal",
                "changes": [{
                    "path": "src/lib.rs",
                    "find": "left - right",
                    "replace": "left + right"
                }]
            }],
            "governance": {
                "default_runtime": "canon",
                "canon": {
                    "command": canon_command.to_string_lossy(),
                    "default_owner": "platform",
                    "default_risk": "medium",
                    "default_zone": "engineering",
                    "default_system_context": "existing"
                },
                "stages": [{
                    "flow_name": "bug-fix",
                    "stage_id": "investigate",
                    "enabled": true,
                    "required": true,
                    "autopilot": false,
                    "runtime": "canon",
                    "canon_mode": "discovery",
                    "system_context": "existing",
                    "risk": "medium",
                    "zone": "engineering",
                    "owner": "platform"
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_two_stage_canon_execution_profile(workspace: &Path, canon_command: &Path) {
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "canon-default-multi-stage",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"]
            },
            "attempts": [{
                "attempt_id": "fix-add",
                "summary": "Replace subtraction with addition",
                "failure_mode": "terminal",
                "changes": [{
                    "path": "src/lib.rs",
                    "find": "left - right",
                    "replace": "left + right"
                }]
            }],
            "governance": {
                "default_runtime": "canon",
                "canon": {
                    "command": canon_command.to_string_lossy(),
                    "default_owner": "platform",
                    "default_risk": "medium",
                    "default_zone": "engineering",
                    "default_system_context": "existing"
                },
                "stages": [{
                    "flow_name": "bug-fix",
                    "stage_id": "investigate",
                    "enabled": true,
                    "required": true,
                    "autopilot": false,
                    "runtime": "canon",
                    "canon_mode": "discovery",
                    "system_context": "existing",
                    "risk": "medium",
                    "zone": "engineering",
                    "owner": "platform"
                }, {
                    "flow_name": "bug-fix",
                    "stage_id": "implement",
                    "enabled": true,
                    "required": true,
                    "autopilot": false,
                    "runtime": "canon",
                    "canon_mode": "implementation",
                    "system_context": "existing",
                    "risk": "medium",
                    "zone": "engineering",
                    "owner": "platform"
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn run_boundline_in_with_path(workspace: &Path, args: &[&str], path_prefix: &Path) -> Output {
    let existing_path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![path_prefix.to_path_buf()];
    paths.extend(std::env::split_paths(&existing_path));
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .env("PATH", std::env::join_paths(paths).unwrap())
        .output()
        .unwrap()
}

fn run_boundline_in_with_exact_path(workspace: &Path, args: &[&str], path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .env("PATH", path)
        .output()
        .unwrap()
}

#[test]
#[cfg(unix)]
fn run_with_canon_config_defaults_to_canon_governance() {
    let workspace = temp_canon_default_workspace("canon-default-gov");
    let canon_path = write_ready_canon_on_path("canon-default-gov");

    let output = run_boundline_in_with_path(
        &workspace,
        &["run", "--goal", "Add user authentication"],
        &canon_path,
    );
    let text = terminal_text(&output);

    assert!(output.status.success(), "{text}");
    assert!(
        text.contains("canon") || text.contains("Canon"),
        "expected Canon governance reference in output: {text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn run_with_no_canon_falls_back_to_local_governance() {
    let workspace = temp_canon_default_workspace("no-canon-fallback");

    let output = run_boundline_in(&workspace, &["run", "--no-canon", "--goal", "Fix login bug"]);
    let text = terminal_text(&output);

    assert!(output.status.success(), "{text}");
    assert!(!text.contains("canon governance"), "expected local governance, not Canon: {text}");

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn run_without_canon_config_uses_local_governance() {
    let workspace = temp_no_canon_config_workspace("no-canon-compat");

    let output = run_boundline_in(&workspace, &["run", "--goal", "Improve performance"]);
    let text = terminal_text(&output);

    assert!(output.status.success(), "{text}");
    assert!(
        !text.contains("canon governance"),
        "expected local governance without [canon] config: {text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
#[cfg(unix)]
fn run_with_mode_defaults_to_canon_without_workspace_canon_config() {
    let workspace = temp_no_canon_config_workspace("mode-implies-canon");
    let canon_path = write_ready_canon_on_path("mode-implies-canon");

    let output = run_boundline_in_with_path(
        &workspace,
        &["run", "--mode", "requirements", "--goal", "Shape onboarding requirements"],
        &canon_path,
    );
    let text = terminal_text(&output);

    assert!(output.status.success(), "{text}");
    assert!(text.contains("canon"), "expected Canon governance from --mode: {text}");
    assert!(
        text.contains("selected_mode") || text.contains("requirements"),
        "expected selected mode in output: {text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn run_with_incomplete_canon_surface_stops_with_repair_guidance() {
    let workspace = temp_canon_default_workspace("incomplete-surface");
    let empty_path = std::env::temp_dir().join("boundline-empty-canon-path");
    let _ = fs::remove_dir_all(&empty_path);
    fs::create_dir_all(&empty_path).unwrap();

    let output = run_boundline_in_with_exact_path(
        &workspace,
        &["run", "--goal", "Deploy service"],
        &empty_path,
    );
    let text = terminal_text(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{text}\n{stderr}");

    assert!(!output.status.success(), "{combined}");
    assert!(
        combined.contains("repair") || combined.contains("doctor") || combined.contains("install"),
        "expected repair guidance in output: {combined}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

/// T038: Integration test verifying `boundline run --goal "<goal>" --brief docs/prd.md --brief docs/arch.md`
/// assembles a Canon governance start request with the correct `input_documents` array
/// and `bounded_context` fields (mock Canon CLI).
#[test]
#[cfg(unix)]
fn run_with_briefs_assembles_canon_governance_start_request() {
    let workspace = temp_canon_default_workspace("brief-assembly");

    // Create brief files in workspace
    let docs_dir = workspace.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("prd.md"), "# Product Brief\n\nBuild a task management API").unwrap();
    fs::write(docs_dir.join("arch.md"), "# Architecture\n\nMicroservices with REST endpoints")
        .unwrap();
    let response = governed_ready_response(
        "run-inputs-001",
        ".canon/runs/run-inputs-001",
        ".canon/runs/run-inputs-001/discovery.md",
        "discovery packet ready",
        "Canon completed discovery",
    );
    fs::create_dir_all(workspace.join(".canon/runs/run-inputs-001")).unwrap();
    fs::write(workspace.join(".canon/runs/run-inputs-001/discovery.md"), "# Discovery\n\nReady\n")
        .unwrap();
    let canon_path = write_capturing_canon_on_path("brief-assembly", &workspace, &response);
    write_canon_execution_profile(&workspace, &canon_path.join("canon"));

    let output = run_boundline_in_with_path(
        &workspace,
        &[
            "run",
            "--goal",
            "Build a task management API",
            "--brief",
            "docs/prd.md",
            "--brief",
            "docs/arch.md",
        ],
        &canon_path,
    );
    let text = terminal_text(&output);

    // The run should succeed with Canon governance
    assert!(output.status.success(), "run should succeed: {text}");
    assert!(
        text.contains("canon") || text.contains("Canon"),
        "expected Canon governance in output: {text}"
    );
    let captured =
        fs::read_to_string(workspace.join(".boundline/canon-request.json")).unwrap_or_default();
    assert!(captured.contains(r#""request_kind":"start""#), "{captured}");
    assert!(captured.contains(r#""input_documents""#), "{captured}");
    assert!(captured.contains(r#""kind":"stage-brief""#), "{captured}");
    assert!(captured.contains(r#""path":"docs/prd.md""#), "{captured}");
    assert!(captured.contains(r#""kind":"authored-brief""#), "{captured}");
    assert!(captured.contains(r#""path":"docs/arch.md""#), "{captured}");
    let session = fs::read_to_string(workspace.join(".boundline/session.json")).unwrap_or_default();
    assert!(session.contains(r#""accumulated_context""#), "{session}");
    assert!(session.contains(r#".canon/runs/run-inputs-001"#), "{session}");

    let _ = fs::remove_dir_all(&workspace);
}

/// T082: Integration test verifying multi-stage governed forwarding: the first
/// Canon governed document is accumulated and the second Canon stage receives it
/// as bounded-context packet reuse.
#[test]
#[cfg(unix)]
fn multi_stage_canon_run_reuses_prior_governed_packet() {
    let workspace = temp_canon_default_workspace("multi-stage-reuse");
    let first_response = governed_ready_response(
        "run-stage-001",
        ".canon/runs/run-stage-001",
        ".canon/runs/run-stage-001/discovery.md",
        "discovery packet ready",
        "Canon completed discovery",
    );
    let second_response = governed_ready_response(
        "run-stage-002",
        ".canon/runs/run-stage-002",
        ".canon/runs/run-stage-002/implementation.md",
        "implementation packet ready",
        "Canon completed implementation",
    );
    fs::create_dir_all(workspace.join(".canon/runs/run-stage-001")).unwrap();
    fs::create_dir_all(workspace.join(".canon/runs/run-stage-002")).unwrap();
    fs::write(
        workspace.join(".canon/runs/run-stage-001/discovery.md"),
        "# Discovery\n\nPrior governed discovery context.\n",
    )
    .unwrap();
    fs::write(
        workspace.join(".canon/runs/run-stage-002/implementation.md"),
        "# Implementation\n\nGoverned implementation context.\n",
    )
    .unwrap();
    let canon_path = write_multi_stage_capturing_canon_on_path(
        "multi-stage-reuse",
        &workspace,
        &first_response,
        &second_response,
    );
    write_two_stage_canon_execution_profile(&workspace, &canon_path.join("canon"));

    let output = run_boundline_in_with_path(
        &workspace,
        &["run", "--goal", "Fix arithmetic behavior with governed stages"],
        &canon_path,
    );
    let text = terminal_text(&output);

    assert!(output.status.success(), "{text}");
    let second_request =
        fs::read_to_string(workspace.join(".boundline/canon-request-2.json")).unwrap_or_default();
    assert!(second_request.contains(r#""reused_packets""#), "{second_request}");
    assert!(second_request.contains(r#".canon/runs/run-stage-001"#), "{second_request}");
    assert!(second_request.contains("discovery packet ready"), "{second_request}");
    let session = fs::read_to_string(workspace.join(".boundline/session.json")).unwrap_or_default();
    assert!(session.contains(r#""accumulated_context""#), "{session}");
    assert!(session.contains("discovery.md"), "{session}");

    let _ = fs::remove_dir_all(&workspace);
    let _ = fs::remove_dir_all(&canon_path);
}

/// T039: Integration test verifying that when Canon returns `incomplete` with `missing_sections`,
/// Boundline surfaces a clarification prompt rather than failing silently.
#[test]
#[cfg(unix)]
fn run_with_incomplete_canon_response_surfaces_clarification() {
    let workspace = temp_canon_default_workspace("incomplete-response");
    let docs_dir = workspace.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("prd.md"), "# PRD\n\nShape onboarding requirements").unwrap();
    let incomplete_response = r#"{"status":"incomplete","approval_state":"not_needed","run_ref":"run-incomplete-001","packet_ref":"pkt-001","expected_document_refs":[".canon/runs/run-incomplete-001/requirements.md"],"document_refs":[],"packet_readiness":"incomplete","missing_sections":["Stakeholders","Non-Functional Requirements"],"headline":"Requirements document incomplete","reason_code":"missing_sections","message":"Document is missing required sections: Stakeholders, Non-Functional Requirements"}"#;
    let bin_dir =
        write_capturing_canon_on_path("incomplete-response", &workspace, incomplete_response);
    write_canon_execution_profile(&workspace, &bin_dir.join("canon"));

    let output = run_boundline_in_with_path(
        &workspace,
        &["run", "--goal", "Shape requirements for onboarding", "--brief", "docs/prd.md"],
        &bin_dir,
    );
    let text = terminal_text(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{text}\n{stderr}");

    assert!(!output.status.success(), "{combined}");
    assert!(
        combined.contains("incomplete")
            || combined.contains("clarification")
            || combined.contains("missing"),
        "expected clarification/incomplete indication in output: {combined}"
    );
    assert!(
        combined.contains("Stakeholders") && combined.contains("Non-Functional Requirements"),
        "expected targeted missing sections in output: {combined}"
    );

    let _ = fs::remove_dir_all(&workspace);
    let _ = fs::remove_dir_all(&bin_dir);
}
