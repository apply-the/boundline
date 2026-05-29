use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;

use boundline::{
    AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE, ConfigFile, FileConfigStore, ModelRoute, RoutingConfig,
    RuntimeKind, SUPPORTED_CANON_VERSION,
};

use crate::workspace_fixture::{
    initialize_nested_git_repository, run_boundline_in, temp_fixture_workspace, terminal_text,
};

const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
const OPENAI_BASE_URL_ENV: &str = "OPENAI_BASE_URL";

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvRestore<'a> {
    saved: BTreeMap<&'static str, Option<OsString>>,
    _lock: MutexGuard<'a, ()>,
}

impl Drop for EnvRestore<'_> {
    fn drop(&mut self) {
        unsafe {
            for (key, value) in &self.saved {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

/// Create a workspace with `[canon]` config preferences and a mock Canon CLI.
fn temp_canon_default_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = temp_fixture_workspace(&format!("{prefix}-canon-default"));
    initialize_nested_git_repository(&workspace);
    let boundline_dir = workspace.join(".boundline");

    // Write a config with [canon] section
    fs::write(
        boundline_dir.join("config.toml"),
        r#"[canon]
	mode_selection = "auto-confirm"
    default_risk = "low-impact"
    default_zone = "green"
    default_owner = "delivery-engineer"
	"#,
    )
    .unwrap();

    workspace
}

/// Create a workspace without `[canon]` config (backward compatibility).
fn temp_no_canon_config_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = temp_fixture_workspace(&format!("{prefix}-no-canon-config"));
    initialize_nested_git_repository(&workspace);
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

fn request_headers_complete(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n").map(|index| index + 4)
}

fn request_content_length(buffer: &[u8]) -> Option<usize> {
    let headers_end = request_headers_complete(buffer)?;
    let headers = String::from_utf8_lossy(&buffer[..headers_end]);
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if !name.trim().eq_ignore_ascii_case("content-length") {
            return None;
        }
        value.trim().parse::<usize>().ok()
    })
}

fn request_complete(buffer: &[u8]) -> bool {
    match (request_headers_complete(buffer), request_content_length(buffer)) {
        (Some(headers_end), Some(content_length)) => buffer.len() >= headers_end + content_length,
        (Some(_), None) => true,
        _ => false,
    }
}

fn with_env_test<T>(tracked_keys: &[&'static str], action: impl FnOnce() -> T) -> T {
    let lock = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let saved =
        tracked_keys.iter().map(|key| (*key, std::env::var_os(key))).collect::<BTreeMap<_, _>>();
    let restore = EnvRestore { saved, _lock: lock };
    let result = action();
    drop(restore);
    result
}

fn spawn_scripted_response_server(
    response_bodies: Vec<String>,
) -> Result<(String, mpsc::Receiver<String>, thread::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        for response_body in response_bodies {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };

            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];
            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(read) => {
                        buffer.extend_from_slice(&chunk[..read]);
                        if request_complete(&buffer) {
                            break;
                        }
                    }
                    Err(_) => return,
                }
            }

            let request_text = String::from_utf8_lossy(&buffer).to_string();
            let _ = sender.send(request_text);
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    Ok((format!("http://{address}"), receiver, handle))
}

fn openai_completion_response(payload: serde_json::Value) -> String {
    serde_json::json!({
        "choices": [
            {
                "message": {
                    "content": payload.to_string()
                }
            }
        ]
    })
    .to_string()
}

fn with_scripted_openai_reviews<T>(review_responses: usize, action: impl FnOnce() -> T) -> T {
    with_env_test(&[OPENAI_BASE_URL_ENV, OPENAI_API_KEY_ENV], || {
        let review_response = openai_completion_response(serde_json::json!({
            "disposition": "approve",
            "summary": "Bounded planning artifact is acceptable.",
            "details": "The governed planning artifact is credible and can proceed.",
            "required_action": null,
            "evidence_refs": [".boundline/governance/planning/discovery/brief.md"]
        }));
        let (base_url, _receiver, _handle) =
            spawn_scripted_response_server(vec![review_response; review_responses]).unwrap();
        unsafe {
            std::env::set_var(OPENAI_BASE_URL_ENV, &base_url);
            std::env::set_var(OPENAI_API_KEY_ENV, "token");
        }

        action()
    })
}

fn seed_planning_reviewer_routes(workspace: &Path) {
    let store = FileConfigStore::for_workspace(workspace);
    let mut config = store.load_local().ok().flatten().unwrap_or_default();
    let mut routing = RoutingConfig {
        planning: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "openai/gpt-5.4".to_string(),
        }),
        implementation: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "openai/gpt-5.4".to_string(),
        }),
        review: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "openai/gpt-4o-mini".to_string(),
        }),
        adjudication: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "openai/gpt-4o-mini".to_string(),
        }),
        ..RoutingConfig::default()
    };
    routing.reviewer_roles.insert(
        "reviewer_primary".to_string(),
        ModelRoute { runtime: RuntimeKind::Codex, model: "openai/gpt-5.4".to_string() },
    );
    routing.reviewer_roles.insert(
        "reviewer_secondary".to_string(),
        ModelRoute { runtime: RuntimeKind::Codex, model: "openai/gpt-4o-mini".to_string() },
    );
    config.routing = routing;
    store.save_local(&ConfigFile { version: 1, ..config }).unwrap();
}

fn planning_ready_brief(
    goal_summary: &str,
    intended_outcome: &str,
    domain_entities: &str,
    api_operations: &str,
    validation_target: &str,
) -> String {
    format!(
        concat!(
            "{goal_summary}\n\n",
            "Intended outcome: {intended_outcome}.\n",
            "Authoritative persistence store: workspace-local .boundline/session.json.\n",
            "Authentication boundary: OAuth2 token validation stops at the edge; service authorization begins in Boundline route selection.\n",
            "In-scope API operations: {api_operations}.\n",
            "Domain entities in scope: {domain_entities}.\n",
            "Success criteria: {intended_outcome}.\n",
            "Validation target: {validation_target}.\n"
        ),
        goal_summary = goal_summary,
        intended_outcome = intended_outcome,
        api_operations = api_operations,
        domain_entities = domain_entities,
        validation_target = validation_target,
    )
}

#[cfg(unix)]
fn write_ready_canon_on_path(prefix: &str, workspace: &Path, mode: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin_dir = std::env::temp_dir().join(format!("{prefix}-canon-bin"));
    let _ = fs::remove_dir_all(&bin_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    let canon = bin_dir.join("canon");
    let run_ref = format!("{prefix}-ready-001");
    let packet_ref = format!(".canon/runs/{run_ref}");
    let document_ref = format!("{packet_ref}/{mode}.md");
    fs::create_dir_all(workspace.join(&packet_ref)).unwrap();
    fs::write(
        workspace.join(&document_ref),
        format!("# {}\n\nCanon governed context ready.\n", mode),
    )
    .unwrap();
    let response_json = governed_ready_response(
        &run_ref,
        &packet_ref,
        &document_ref,
        &format!("{mode} packet ready"),
        &format!("Canon completed {mode}"),
    );
    let capabilities = format!(
        r#"{{"canon_version":"{SUPPORTED_CANON_VERSION}","supported_schema_versions":["2026-02-01"],"operations":["start","refresh","capabilities"],"supported_modes":["requirements","discovery","system-shaping","architecture","backlog","change","implementation","refactor","review","verification","pr-review","incident","security-assessment","system-assessment","migration","supply-chain-analysis"],"status_values":["governed_ready","awaiting_approval","blocked"],"approval_state_values":["not_needed","requested","granted"],"packet_readiness_values":["reusable","pending","incomplete"],"compatibility_notes":["stable-json"]}}"#
    );
    fs::write(
        &canon,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'canon version {SUPPORTED_CANON_VERSION}'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{}'\n  exit 0\nfi\ncat >/dev/null\nprintf '%s' '{}'\n",
            capabilities,
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
                    "default_owner": "delivery-engineer",
                    "default_risk": "low-impact",
                    "default_zone": "green",
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
                    "risk": "low-impact",
                    "zone": "green",
                    "owner": "delivery-engineer"
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
                    "default_owner": "delivery-engineer",
                    "default_risk": "low-impact",
                    "default_zone": "green",
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
                    "risk": "low-impact",
                    "zone": "green",
                    "owner": "delivery-engineer"
                }, {
                    "flow_name": "bug-fix",
                    "stage_id": "implement",
                    "enabled": true,
                    "required": true,
                    "autopilot": false,
                    "runtime": "canon",
                    "canon_mode": "implementation",
                    "system_context": "existing",
                    "risk": "low-impact",
                    "zone": "green",
                    "owner": "delivery-engineer"
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

#[cfg(unix)]
fn write_git_only_path(prefix: &str) -> PathBuf {
    let bin_dir = std::env::temp_dir().join(format!("{prefix}-git-only-bin"));
    let _ = fs::remove_dir_all(&bin_dir);
    fs::create_dir_all(&bin_dir).unwrap();
    std::os::unix::fs::symlink("/usr/bin/git", bin_dir.join("git")).unwrap();
    bin_dir
}

#[test]
#[cfg(unix)]
fn run_with_canon_config_defaults_to_canon_governance() {
    let workspace = temp_canon_default_workspace("canon-default-gov");
    let canon_path = write_ready_canon_on_path("canon-default-gov", &workspace, "discovery");

    let output = run_boundline_in_with_path(
        &workspace,
        &["run", "--goal", "Add user authentication"],
        &canon_path,
    );
    let text = terminal_text(&output);
    let status = run_boundline_in_with_path(&workspace, &["status"], &canon_path);
    let status_text = terminal_text(&status);

    assert!(!output.status.success(), "run should pause for Canon planning governance: {text}");
    assert!(
        text.contains("planning governance") || text.contains("plan:discovery"),
        "expected planning-governance pause in output: {text}"
    );
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(
        status_text.contains("governance_lifecycle_runtime: canon"),
        "expected Canon runtime in persisted session status: {status_text}"
    );
    assert!(
        status_text.contains("governance_lifecycle_selected_mode: discovery"),
        "expected discovery mode in persisted session status: {status_text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn run_with_no_canon_falls_back_to_local_governance() {
    let workspace = temp_canon_default_workspace("no-canon-fallback");

    let output = run_boundline_in(&workspace, &["run", "--no-canon", "--goal", "Fix login bug"]);
    let text = terminal_text(&output);

    assert!(!text.contains("canon governance"), "expected local governance, not Canon: {text}");
    assert!(
        !text.contains("governance_selected:") && !text.contains("governance_started:"),
        "expected local governance path without Canon stage selection: {text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn run_without_canon_config_uses_local_governance() {
    let workspace = temp_no_canon_config_workspace("no-canon-compat");

    let output = run_boundline_in(&workspace, &["run", "--goal", "Improve performance"]);
    let text = terminal_text(&output);

    assert!(
        !text.contains("canon governance"),
        "expected local governance without [canon] config: {text}"
    );
    assert!(
        !text.contains("governance_selected:") && !text.contains("governance_started:"),
        "expected local governance path without Canon stage selection: {text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
#[cfg(unix)]
fn run_with_mode_defaults_to_canon_without_workspace_canon_config() {
    let workspace = temp_no_canon_config_workspace("mode-implies-canon");
    let canon_path = write_ready_canon_on_path("mode-implies-canon", &workspace, "requirements");

    let output = run_boundline_in_with_path(
        &workspace,
        &["run", "--mode", "requirements", "--goal", "Shape onboarding requirements"],
        &canon_path,
    );
    let text = terminal_text(&output);
    let status = run_boundline_in_with_path(&workspace, &["status"], &canon_path);
    let status_text = terminal_text(&status);

    assert!(!output.status.success(), "run should pause for Canon planning governance: {text}");
    assert!(
        text.contains("planning governance") || text.contains("plan:requirements"),
        "expected planning-governance pause in output: {text}"
    );
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(
        status_text.contains("governance_lifecycle_runtime: canon"),
        "expected Canon runtime from --mode in persisted session status: {status_text}"
    );
    assert!(
        status_text.contains("governance_lifecycle_selected_mode: requirements"),
        "expected requirements mode in persisted session status: {status_text}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn run_with_incomplete_canon_surface_stops_with_repair_guidance() {
    let workspace = temp_canon_default_workspace("incomplete-surface");
    let git_only_path = write_git_only_path("incomplete-surface");
    let docs_dir = workspace.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        docs_dir.join("deploy-brief.md"),
        planning_ready_brief(
            "Deliver the first bounded service deployment workflow.",
            "deliver a governed deployment path that packages, approves, and releases a service build",
            "service_release, deployment_environment, approval_ticket, deployment_run",
            "create deployment plan, approve release, execute deployment, inspect deployment status",
            "cargo test --quiet",
        ),
    )
    .unwrap();

    let output = run_boundline_in_with_exact_path(
        &workspace,
        &["run", "--goal", "Deploy service", "--brief", "docs/deploy-brief.md"],
        &git_only_path,
    );
    let text = terminal_text(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{text}\n{stderr}");

    assert!(!output.status.success(), "{combined}");
    assert!(
        combined.contains("repair")
            || combined.contains("doctor")
            || combined.contains("install")
            || combined.contains("Canon initialization")
            || combined.contains("governance.canon"),
        "expected repair guidance in output: {combined}"
    );

    let _ = fs::remove_dir_all(&workspace);
    let _ = fs::remove_dir_all(&git_only_path);
}

/// T038: Integration test verifying `boundline run --goal "<goal>" --brief docs/prd.md --brief docs/arch.md`
/// assembles a Canon governance start request with the correct `input_documents` array
/// and `bounded_context` fields (mock Canon CLI).
#[test]
#[cfg(unix)]
fn run_with_briefs_assembles_canon_governance_start_request() {
    with_scripted_openai_reviews(6, || {
        let workspace = temp_canon_default_workspace("brief-assembly");
        seed_planning_reviewer_routes(&workspace);

        // Create brief files in workspace
        let docs_dir = workspace.join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(
            docs_dir.join("prd.md"),
            format!(
                "# Product Brief\n\n{}",
                planning_ready_brief(
                    "Build a task management API for planning, assignment, and delivery tracking.",
                    "deliver an API that can create tasks, assign owners, and track task delivery status",
                    "task, assignee, project, comment, delivery_status",
                    "create task, list task, assign task, update task status",
                    "cargo test --quiet",
                )
            ),
        )
        .unwrap();
        fs::write(
            docs_dir.join("arch.md"),
            "# Architecture\n\nMicroservices with REST endpoints and governed delivery handoffs.\n",
        )
        .unwrap();
        let response = governed_ready_response(
            "run-inputs-001",
            ".canon/runs/run-inputs-001",
            ".canon/runs/run-inputs-001/discovery.md",
            "discovery packet ready",
            "Canon completed discovery",
        );
        fs::create_dir_all(workspace.join(".canon/runs/run-inputs-001")).unwrap();
        fs::write(
            workspace.join(".canon/runs/run-inputs-001/discovery.md"),
            "# Discovery\n\nReady\n",
        )
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

        assert!(
            text.contains("governance_completed: discovery packet ready"),
            "expected governed discovery completion before downstream clarification: {text}"
        );
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
        let session =
            fs::read_to_string(workspace.join(".boundline/session.json")).unwrap_or_default();
        assert!(session.contains(r#""accumulated_context""#), "{session}");
        assert!(session.contains(r#".canon/runs/run-inputs-001"#), "{session}");

        let _ = fs::remove_dir_all(&workspace);
    });
}

/// T082: Integration test verifying multi-stage governed forwarding: the first
/// Canon governed document is accumulated and the second Canon stage receives it
/// as bounded-context packet reuse.
#[test]
#[cfg(unix)]
fn multi_stage_canon_run_reuses_prior_governed_packet() {
    with_scripted_openai_reviews(6, || {
        let workspace = temp_canon_default_workspace("multi-stage-reuse");
        seed_planning_reviewer_routes(&workspace);
        let docs_dir = workspace.join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(
            docs_dir.join("context.md"),
            planning_ready_brief(
                "Fix arithmetic behavior with governed discovery and implementation stages.",
                "deliver a governed repair that restores addition behavior and validates the fix",
                "arithmetic_operation, regression_case, patch_candidate, governed_packet",
                "evaluate regression, apply bounded patch, validate arithmetic behavior",
                "cargo test --quiet",
            ),
        )
        .unwrap();
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
            &[
                "run",
                "--goal",
                "Fix arithmetic behavior with governed stages",
                "--brief",
                "docs/context.md",
            ],
            &canon_path,
        );
        let text = terminal_text(&output);

        assert!(
            text.contains("governance_started: bug-fix:implement (implementation) from bug-fix:investigate (upstream_stage_context)"),
            "expected implementation stage to reuse upstream governed context: {text}"
        );
        assert!(
            text.contains("governance_completed: implementation packet ready"),
            "expected implementation governance completion before downstream clarification: {text}"
        );
        let second_request = fs::read_to_string(workspace.join(".boundline/canon-request-2.json"))
            .unwrap_or_default();
        assert!(second_request.contains(r#""reused_packets""#), "{second_request}");
        assert!(second_request.contains(r#".canon/runs/run-stage-001"#), "{second_request}");
        assert!(second_request.contains(r#""stage_key":"plan:discovery""#), "{second_request}");
        let session =
            fs::read_to_string(workspace.join(".boundline/session.json")).unwrap_or_default();
        assert!(session.contains(r#""accumulated_context""#), "{session}");
        assert!(session.contains("discovery.md"), "{session}");

        let _ = fs::remove_dir_all(&workspace);
        let _ = fs::remove_dir_all(&canon_path);
    });
}

/// T039: Integration test verifying that when Canon returns `incomplete` with `missing_sections`,
/// Boundline surfaces a clarification prompt rather than failing silently.
#[test]
#[cfg(unix)]
fn run_with_incomplete_canon_response_surfaces_clarification() {
    with_scripted_openai_reviews(6, || {
        let workspace = temp_canon_default_workspace("incomplete-response");
        seed_planning_reviewer_routes(&workspace);
        let docs_dir = workspace.join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(
            docs_dir.join("prd.md"),
            format!(
                "# PRD\n\n{}",
                planning_ready_brief(
                    "Shape onboarding requirements for the first delivery slice.",
                    "deliver a governed requirements packet for the onboarding experience",
                    "onboarding_flow, stakeholder, policy_requirement, non_functional_requirement",
                    "capture onboarding requirements, list stakeholders, record acceptance constraints",
                    "cargo test --quiet",
                )
            ),
        )
        .unwrap();
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
    });
}

/// Blocked planning stage is retried with `refresh` on next `plan` invocation and
/// progresses to subsequent stages after Canon returns `governed_ready`.
#[test]
#[cfg(unix)]
fn blocked_planning_stage_retries_with_refresh_and_progresses() {
    with_scripted_openai_reviews(6, || {
        let workspace = temp_canon_default_workspace("blocked-retry-refresh");
        seed_planning_reviewer_routes(&workspace);
        let docs_dir = workspace.join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(
            docs_dir.join("prd.md"),
            format!(
                "# PRD\n\n{}",
                planning_ready_brief(
                    "Deliver a governed planning flow that retries after a block.",
                    "deliver a governed requirements packet that passes on retry",
                    "requirements_packet, planning_stage, retry_outcome",
                    "capture requirements, approve planning artifact, record retry outcome",
                    "cargo test --quiet",
                )
            ),
        )
        .unwrap();

        // First Canon response: incomplete (blocked) — includes a run_ref.
        let incomplete_response = r#"{"status":"incomplete","approval_state":"not_needed","run_ref":"run-blocked-001","packet_ref":".canon/runs/run-blocked-001","expected_document_refs":[".canon/runs/run-blocked-001/requirements.md"],"document_refs":[],"packet_readiness":"incomplete","missing_sections":["Domain Model"],"headline":"Requirements incomplete","message":"Missing Domain Model section"}"#;
        let bin_dir =
            write_capturing_canon_on_path("blocked-retry-refresh", &workspace, incomplete_response);
        write_canon_execution_profile(&workspace, &bin_dir.join("canon"));

        // First run: Canon blocks at the first planning stage with incomplete.
        let output = run_boundline_in_with_path(
            &workspace,
            &["run", "--goal", "Deliver governed planning that retries", "--brief", "docs/prd.md"],
            &bin_dir,
        );
        let text = terminal_text(&output);
        assert!(!output.status.success(), "first run should block: {text}");

        // Verify session shows blocked stage with a canon_run_ref.
        let session_raw =
            fs::read_to_string(workspace.join(".boundline/session.json")).unwrap_or_default();
        assert!(
            session_raw.contains("\"canon_run_ref\": \"run-blocked-001\"")
                || session_raw.contains(r#""canon_run_ref":"run-blocked-001""#),
            "expected canon_run_ref from first incomplete response: {session_raw}"
        );

        // Now swap Canon to return governed_ready on the retry.
        let packet_dir = workspace.join(".canon/runs/run-blocked-001");
        fs::create_dir_all(&packet_dir).unwrap();
        fs::write(
            packet_dir.join("requirements.md"),
            "# Requirements\n\nGoverned requirements context ready on retry.\n",
        )
        .unwrap();
        let ready_response = governed_ready_response(
            "run-blocked-001",
            ".canon/runs/run-blocked-001",
            ".canon/runs/run-blocked-001/requirements.md",
            "requirements ready on retry",
            "Canon completed requirements after refresh",
        );
        // Overwrite fake Canon with one that returns success and captures the request.
        let _ = fs::remove_dir_all(&bin_dir);
        let bin_dir =
            write_capturing_canon_on_path("blocked-retry-refresh", &workspace, &ready_response);

        // Second run: plan again — retry should use refresh (existing run_ref) and succeed.
        let output2 = run_boundline_in_with_path(&workspace, &["plan"], &bin_dir);
        let text2 = terminal_text(&output2);

        // The retry should have completed the previously-blocked stage.
        assert!(
            text2.contains("requirements ready on retry") || text2.contains("governance_completed"),
            "expected governance completion after retry: {text2}"
        );

        // Verify the retry Canon request used refresh with the existing run_ref.
        let captured =
            fs::read_to_string(workspace.join(".boundline/canon-request.json")).unwrap_or_default();
        assert!(
            captured.contains(r#""request_kind":"refresh""#),
            "retry should use refresh request kind: {captured}"
        );
        assert!(
            captured.contains(r#""run_ref":"run-blocked-001""#),
            "retry should carry the previous canon_run_ref: {captured}"
        );

        // Session should now show the stage as governed_ready, not blocked.
        let session_after =
            fs::read_to_string(workspace.join(".boundline/session.json")).unwrap_or_default();
        assert!(
            session_after.contains("\"lifecycle_state\": \"governed_ready\"")
                || session_after.contains(r#""lifecycle_state":"governed_ready""#),
            "stage should be governed_ready after successful retry: {session_after}"
        );

        let _ = fs::remove_dir_all(&workspace);
        let _ = fs::remove_dir_all(&bin_dir);
    });
}
