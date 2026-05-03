#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use uuid::Uuid;

const FIXTURE_CARGO_TOML: &str = concat!(
    "[package]\n",
    "name = \"boundline-fixture\"\n",
    "version = \"0.1.0\"\n",
    "edition = \"2024\"\n",
);

const RED_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left - right\n", "}\n",);

const GREEN_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left + right\n", "}\n",);

const MULTIPLY_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left * right\n", "}\n",);

const GUIDED_ADAPTIVE_LIB_RS: &str = concat!(
    "mod helper;\n\n",
    "pub fn add(left: i32, right: i32) -> i32 {\n",
    "    let _unused = left - right;\n",
    "    helper::add_pair(left, right)\n",
    "}\n",
);

const GUIDED_ADAPTIVE_HELPER_RS: &str =
    concat!("pub fn add_pair(left: i32, right: i32) -> i32 {\n", "    left - right\n", "}\n",);

const ORDERING_BOUNDARY_LIB_RS: &str =
    concat!("pub fn includes_threshold(value: i32) -> bool {\n", "    value > 3\n", "}\n",);

const ORDERING_BOUNDARY_TEST_RS: &str = concat!(
    "use boundline_fixture::includes_threshold;\n\n",
    "#[test]\n",
    "fn threshold_is_inclusive() {\n",
    "    assert!(includes_threshold(3));\n",
    "    assert!(!includes_threshold(2));\n",
    "}\n",
);

const GUIDED_ADAPTIVE_VALIDATE_SH: &str = concat!(
    "#!/bin/sh\n",
    "set +e\n",
    "cargo test --quiet\n",
    "status=$?\n",
    "if [ \"$status\" -ne 0 ]; then\n",
    "  printf 'validation hint: inspect src/helper.rs for the remaining failing arithmetic path\\n' >&2\n",
    "fi\n",
    "exit \"$status\"\n",
);

const MISSING_CANON_COMMAND: &str = "/definitely/missing/canon";

const FIXTURE_TEST_RS: &str = concat!(
    "use boundline_fixture::add;\n\n",
    "#[test]\n",
    "fn boundline_drives_red_to_green() {\n",
    "    assert_eq!(add(2, 2), 4);\n",
    "}\n",
);

const VALID_WORKFLOWS_TOML: &str = concat!(
    "[workflow.default]\n",
    "goal_source = \"session\"\n",
    "entry = \"capture\"\n",
    "phases = [\"capture\", \"plan\", \"run\", \"inspect\"]\n",
    "allow_review = true\n",
    "allow_governance = true\n\n",
    "[workflow.default.output]\n",
    "next_command = true\n",
    "routing_summary = true\n",
    "execution_condition = true\n",
);

const INVALID_WORKFLOWS_TOML: &str = concat!(
    "[workflow.invalid-flow]\n",
    "goal_source = \"session\"\n",
    "entry = \"run\"\n",
    "phases = [\"run\", \"fan-out\", \"inspect\"]\n",
    "allow_review = true\n",
    "allow_governance = true\n",
);

const WORKFLOW_FOLLOW_THROUGH_TOML: &str = concat!(
    "[workflow.governed-delivery]\n",
    "goal_source = \"session\"\n",
    "entry = \"capture\"\n",
    "phases = [\"capture\", \"plan\", \"run\", \"review\", \"govern\", \"inspect\"]\n",
    "allow_review = true\n",
    "allow_governance = true\n\n",
    "[workflow.governed-delivery.when]\n",
    "review = \"review_triggered\"\n",
    "governance = \"governance_required\"\n\n",
    "[workflow.governed-delivery.output]\n",
    "next_command = true\n",
    "routing_summary = true\n",
    "execution_condition = true\n",
);

const DISCOVERY_WORKFLOWS_TOML: &str = concat!(
    "[workflow.governed-delivery]\n",
    "goal_source = \"session\"\n",
    "entry = \"capture\"\n",
    "phases = [\"capture\", \"plan\", \"run\", \"review\", \"govern\", \"inspect\"]\n",
    "allow_review = true\n",
    "allow_governance = true\n",
    "summary = \"bounded delivery path with review and governance before completion\"\n",
    "recommended_when = \"the task needs explicit review and governance evidence\"\n\n",
    "[workflow.governed-delivery.when]\n",
    "review = \"review_triggered\"\n",
    "governance = \"governance_required\"\n\n",
    "[workflow.quick-fix]\n",
    "goal_source = \"session\"\n",
    "entry = \"capture\"\n",
    "phases = [\"capture\", \"plan\", \"run\", \"inspect\"]\n",
    "allow_review = false\n",
    "allow_governance = false\n",
);

const BLOCKED_GOVERN_WORKFLOW_TOML: &str = concat!(
    "[workflow.blocked-delivery]\n",
    "goal_source = \"session\"\n",
    "entry = \"capture\"\n",
    "phases = [\"capture\", \"plan\", \"run\", \"govern\", \"inspect\"]\n",
    "allow_review = false\n",
    "allow_governance = true\n",
);

pub fn temp_fixture_workspace(prefix: &str) -> PathBuf {
    create_fixture_workspace(
        prefix,
        vec![execution_attempt(
            "fix-add",
            "Replace subtraction with addition",
            "terminal",
            "left - right",
            "left + right",
        )],
    )
}

pub fn temp_cluster_workspaces(prefix: &str) -> (PathBuf, PathBuf) {
    (
        temp_fixture_workspace(&format!("{prefix}-primary")),
        temp_fixture_workspace(&format!("{prefix}-secondary")),
    )
}

pub fn temp_broken_fixture_workspace(prefix: &str) -> PathBuf {
    create_fixture_workspace(
        prefix,
        vec![execution_attempt(
            "broken-change",
            "Attempt a missing patch",
            "terminal",
            "left * right",
            "left + right",
        )],
    )
}

pub fn temp_adaptive_fixture_workspace(prefix: &str) -> PathBuf {
    create_adaptive_fixture_workspace(prefix, RED_LIB_RS)
}

pub fn temp_adaptive_replanning_workspace(prefix: &str) -> PathBuf {
    create_adaptive_fixture_workspace(prefix, MULTIPLY_LIB_RS)
}

pub fn temp_adaptive_guided_replanning_workspace(prefix: &str) -> PathBuf {
    create_adaptive_guided_fixture_workspace(prefix)
}

pub fn temp_adaptive_ordering_boundary_workspace(prefix: &str) -> PathBuf {
    create_adaptive_ordering_boundary_workspace(prefix)
}

pub fn temp_optional_governance_workspace(prefix: &str) -> PathBuf {
    create_governance_fixture_workspace(prefix, false)
}

pub fn temp_required_governance_workspace(prefix: &str) -> PathBuf {
    create_governance_fixture_workspace(prefix, true)
}

pub fn temp_canon_governance_workspace(prefix: &str) -> PathBuf {
    create_canon_governance_fixture_workspace(prefix, CanonFixtureScenario::Reusable)
}

pub fn temp_canon_packet_rejection_workspace(prefix: &str) -> PathBuf {
    create_canon_governance_fixture_workspace(prefix, CanonFixtureScenario::RejectedPacket)
}

pub fn temp_canon_approval_workspace(prefix: &str) -> PathBuf {
    create_canon_governance_fixture_workspace(prefix, CanonFixtureScenario::Approval)
}

pub fn temp_canon_autopilot_blocked_workspace(prefix: &str) -> PathBuf {
    create_canon_governance_fixture_workspace(prefix, CanonFixtureScenario::AutopilotBlocked)
}

pub fn temp_canon_security_assessment_workspace(prefix: &str) -> PathBuf {
    create_canon_governance_fixture_workspace(prefix, CanonFixtureScenario::VerifySecurityReusable)
}

pub fn temp_canon_security_approval_workspace(prefix: &str) -> PathBuf {
    create_canon_governance_fixture_workspace(prefix, CanonFixtureScenario::VerifySecurityApproval)
}

pub fn temp_workflow_layer_workspace(prefix: &str) -> PathBuf {
    create_workflow_fixture_workspace(prefix, VALID_WORKFLOWS_TOML, false)
}

pub fn temp_invalid_workflow_layer_workspace(prefix: &str) -> PathBuf {
    create_workflow_fixture_workspace(prefix, INVALID_WORKFLOWS_TOML, false)
}

pub fn temp_workflow_layer_compat_workspace(prefix: &str) -> PathBuf {
    create_workflow_fixture_workspace(prefix, VALID_WORKFLOWS_TOML, true)
}

pub fn temp_workflow_follow_through_workspace(prefix: &str) -> PathBuf {
    let workspace = create_canon_governance_fixture_workspace(
        prefix,
        CanonFixtureScenario::VerifySecurityReusable,
    );
    write_workflow_definitions(&workspace, WORKFLOW_FOLLOW_THROUGH_TOML);
    write_review_profile_into_execution_profile(&workspace);
    workspace
}

pub fn temp_workflow_follow_through_approval_workspace(prefix: &str) -> PathBuf {
    let workspace = create_canon_governance_fixture_workspace(
        prefix,
        CanonFixtureScenario::VerifySecurityApproval,
    );
    write_workflow_definitions(&workspace, WORKFLOW_FOLLOW_THROUGH_TOML);
    write_review_profile_into_execution_profile(&workspace);
    workspace
}

pub fn temp_workflow_discovery_workspace(prefix: &str) -> PathBuf {
    create_workflow_fixture_workspace(prefix, DISCOVERY_WORKFLOWS_TOML, false)
}

pub fn temp_workflow_discovery_compat_workspace(prefix: &str) -> PathBuf {
    create_workflow_fixture_workspace(prefix, DISCOVERY_WORKFLOWS_TOML, true)
}

pub fn temp_workflow_follow_through_blocked_workspace(prefix: &str) -> PathBuf {
    create_workflow_fixture_workspace(prefix, BLOCKED_GOVERN_WORKFLOW_TOML, false)
}

pub fn temp_workflow_governed_stage_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_canon_governance_workspace(prefix);
    write_workflow_definitions(&workspace, VALID_WORKFLOWS_TOML);
    workspace
}

pub fn temp_workflow_governed_stage_approval_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_canon_approval_workspace(prefix);
    write_workflow_definitions(&workspace, VALID_WORKFLOWS_TOML);
    workspace
}

#[allow(dead_code)]
pub fn temp_replanning_execution_workspace(prefix: &str) -> PathBuf {
    create_fixture_workspace(
        prefix,
        vec![
            execution_attempt(
                "bad-fix",
                "Introduce a wrong division fix",
                "replan",
                "left - right",
                "left / right",
            ),
            execution_attempt(
                "good-fix",
                "Replace division with addition",
                "terminal",
                "left / right",
                "left + right",
            ),
        ],
    )
}

pub fn run_boundline(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap()
}

pub fn run_boundline_in(workspace: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

pub fn write_markdown_brief(
    workspace: &Path,
    relative_path: impl AsRef<Path>,
    contents: impl AsRef<str>,
) -> PathBuf {
    let path = workspace.join(relative_path.as_ref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents.as_ref()).unwrap();
    path
}

pub fn write_workflow_definitions(workspace: &Path, contents: impl AsRef<str>) -> PathBuf {
    let path = workspace.join(".boundline/workflows.toml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents.as_ref()).unwrap();
    path
}

pub fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn extract_trace_path(text: &str) -> Option<PathBuf> {
    text.split_whitespace().find_map(|token| {
        let cleaned = token.trim_matches(|ch: char| ch == '"' || ch == ',' || ch == ':');
        if cleaned.ends_with(".json") { Some(PathBuf::from(cleaned)) } else { None }
    })
}

fn create_fixture_workspace(prefix: &str, attempts: Vec<serde_json::Value>) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "red-to-green-execution",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "attempts": attempts,
        }))
        .unwrap(),
    )
    .unwrap();

    debug_assert_ne!(RED_LIB_RS, GREEN_LIB_RS);
    workspace
}

fn create_workflow_fixture_workspace(
    prefix: &str,
    workflow_contents: &str,
    include_execution_profile: bool,
) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    write_workflow_definitions(&workspace, workflow_contents);

    if include_execution_profile {
        write_basic_execution_profile(&workspace, "workflow-layer-compat-execution");
    }

    workspace
}

fn create_adaptive_fixture_workspace(prefix: &str, source_contents: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), source_contents).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "adaptive-red-to-green-execution",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "attempts": [],
            "adaptive": {
                "max_selected_targets": 1,
                "max_generated_attempts": 4,
                "path_preferences": ["src/"],
                "allowed_change_kinds": ["arithmetic_swap"],
            },
        }))
        .unwrap(),
    )
    .unwrap();

    workspace
}

fn create_adaptive_guided_fixture_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), GUIDED_ADAPTIVE_LIB_RS).unwrap();
    fs::write(workspace.join("src/helper.rs"), GUIDED_ADAPTIVE_HELPER_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();

    let validate_script = workspace.join("validate.sh");
    fs::write(&validate_script, GUIDED_ADAPTIVE_VALIDATE_SH).unwrap();
    let mut permissions = fs::metadata(&validate_script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&validate_script, permissions).unwrap();

    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "adaptive-guided-replanning-execution",
            "read_targets": ["src/lib.rs", "src/helper.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "./validate.sh",
                "args": [],
            },
            "attempts": [],
            "adaptive": {
                "max_selected_targets": 1,
                "max_generated_attempts": 4,
                "path_preferences": ["src/lib.rs"],
                "allowed_change_kinds": ["arithmetic_swap"],
            },
        }))
        .unwrap(),
    )
    .unwrap();

    workspace
}

fn create_adaptive_ordering_boundary_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), ORDERING_BOUNDARY_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), ORDERING_BOUNDARY_TEST_RS).unwrap();
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "adaptive-ordering-boundary-execution",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "attempts": [],
            "adaptive": {
                "max_selected_targets": 1,
                "max_generated_attempts": 4,
                "path_preferences": ["src/"],
                "allowed_change_kinds": ["ordering_boundary_flip"],
            },
        }))
        .unwrap(),
    )
    .unwrap();

    workspace
}

fn write_basic_execution_profile(workspace: &Path, profile_name: &str) {
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": profile_name,
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "attempts": [
                execution_attempt(
                    "fix-add",
                    "Replace subtraction with addition",
                    "terminal",
                    "left - right",
                    "left + right",
                )
            ],
        }))
        .unwrap(),
    )
    .unwrap();
}

fn create_governance_fixture_workspace(prefix: &str, required: bool) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": if required {
                "required-governance-execution"
            } else {
                "optional-governance-execution"
            },
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "attempts": [
                execution_attempt(
                    "fix-add",
                    "Replace subtraction with addition",
                    "terminal",
                    "left - right",
                    "left + right",
                )
            ],
            "governance": {
                "default_runtime": "local",
                "canon": {
                    "command": MISSING_CANON_COMMAND,
                    "default_owner": "platform",
                    "default_risk": "medium",
                    "default_zone": "engineering",
                    "default_system_context": "existing"
                },
                "stages": [
                    {
                        "flow_name": "bug-fix",
                        "stage_id": "investigate",
                        "enabled": true,
                        "required": required,
                        "autopilot": false,
                        "runtime": "canon",
                        "canon_mode": "discovery",
                        "system_context": "existing",
                        "risk": "medium",
                        "zone": "engineering",
                        "owner": "platform"
                    }
                ]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    workspace
}

fn write_review_profile_into_execution_profile(workspace: &Path) {
    let path = workspace.join(".boundline/execution.json");
    let mut profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    profile["review"] = serde_json::json!({
        "triggers": ["pr_ready"],
        "reviewers": [
            {
                "reviewer_id": "safety",
                "role": "Safety",
                "source": "gpt",
                "weight": 1
            },
            {
                "reviewer_id": "maintainability",
                "role": "Maintainability",
                "source": "claude",
                "weight": 1
            }
        ],
        "vote_rule": {
            "strategy": "majority"
        },
        "scenarios": [
            {
                "trigger": "pr_ready",
                "findings": [
                    {
                        "reviewer_id": "safety",
                        "disposition": "approve",
                        "summary": "No blockers"
                    },
                    {
                        "reviewer_id": "maintainability",
                        "disposition": "approve",
                        "summary": "Ready to ship"
                    }
                ]
            }
        ]
    });
    fs::write(path, serde_json::to_string_pretty(&profile).unwrap()).unwrap();
}

#[derive(Clone, Copy)]
enum CanonFixtureScenario {
    Reusable,
    RejectedPacket,
    Approval,
    AutopilotBlocked,
    VerifySecurityReusable,
    VerifySecurityApproval,
}

fn create_canon_governance_fixture_workspace(
    prefix: &str,
    scenario: CanonFixtureScenario,
) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::create_dir_all(workspace.join(".canon/runs")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();

    let command = match scenario {
        CanonFixtureScenario::AutopilotBlocked => "canon-missing".to_string(),
        _ => write_canon_stub_script(&workspace, scenario).to_string_lossy().into_owned(),
    };

    write_canon_fixture_documents(&workspace, scenario);

    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": match scenario {
                CanonFixtureScenario::Reusable => "canon-governance-execution",
                CanonFixtureScenario::RejectedPacket => "canon-packet-rejection-execution",
                CanonFixtureScenario::Approval => "canon-approval-execution",
                CanonFixtureScenario::AutopilotBlocked => "canon-autopilot-blocked-execution",
                CanonFixtureScenario::VerifySecurityReusable => "canon-security-assessment-execution",
                CanonFixtureScenario::VerifySecurityApproval => "canon-security-approval-execution",
            },
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "attempts": [
                execution_attempt(
                    "fix-add",
                    "Replace subtraction with addition",
                    "terminal",
                    "left - right",
                    "left + right",
                )
            ],
            "governance": canon_governance_profile(&command, scenario),
        }))
        .unwrap(),
    )
    .unwrap();

    workspace
}

fn canon_governance_profile(command: &str, scenario: CanonFixtureScenario) -> serde_json::Value {
    if matches!(
        scenario,
        CanonFixtureScenario::VerifySecurityReusable | CanonFixtureScenario::VerifySecurityApproval
    ) {
        return serde_json::json!({
            "default_runtime": "local",
            "canon": {
                "command": command,
                "default_owner": "platform",
                "default_risk": "medium",
                "default_zone": "engineering",
                "default_system_context": "existing"
            },
            "stages": [
                {
                    "flow_name": "bug-fix",
                    "stage_id": "verify",
                    "enabled": true,
                    "required": true,
                    "autopilot": true,
                    "runtime": "canon",
                    "system_context": "existing",
                    "risk": "medium",
                    "zone": "engineering",
                    "owner": "platform"
                }
            ]
        });
    }

    let investigate_policy = match scenario {
        CanonFixtureScenario::Approval => serde_json::json!({
            "flow_name": "bug-fix",
            "stage_id": "investigate",
            "enabled": true,
            "required": true,
            "autopilot": true,
            "runtime": "canon",
            "system_context": "existing",
            "risk": "medium",
            "zone": "engineering",
            "owner": "platform"
        }),
        CanonFixtureScenario::AutopilotBlocked => serde_json::json!({
            "flow_name": "bug-fix",
            "stage_id": "investigate",
            "enabled": true,
            "required": true,
            "autopilot": true,
            "runtime": "canon",
            "system_context": "existing",
            "risk": "medium",
            "zone": "engineering",
            "owner": "platform"
        }),
        _ => serde_json::json!({
            "flow_name": "bug-fix",
            "stage_id": "investigate",
            "enabled": true,
            "required": false,
            "autopilot": false,
            "runtime": "canon",
            "canon_mode": "discovery",
            "system_context": "existing",
            "risk": "medium",
            "zone": "engineering",
            "owner": "platform"
        }),
    };

    let mut stages = vec![investigate_policy];
    if matches!(scenario, CanonFixtureScenario::Reusable) {
        stages.push(serde_json::json!({
            "flow_name": "bug-fix",
            "stage_id": "implement",
            "enabled": true,
            "required": false,
            "autopilot": false,
            "runtime": "canon",
            "canon_mode": "implementation",
            "system_context": "existing",
            "risk": "medium",
            "zone": "engineering",
            "owner": "platform"
        }));
    }

    serde_json::json!({
        "default_runtime": "local",
        "canon": {
            "command": command,
            "default_owner": "platform",
            "default_risk": "medium",
            "default_zone": "engineering",
            "default_system_context": "existing"
        },
        "stages": stages,
    })
}

fn write_canon_fixture_documents(workspace: &Path, scenario: CanonFixtureScenario) {
    fs::create_dir_all(workspace.join(".canon/runs/canon-run-investigate")).unwrap();
    fs::write(
        workspace.join(".canon/runs/canon-run-investigate/discovery.md"),
        match scenario {
            CanonFixtureScenario::RejectedPacket => "# Discovery\n\nTODO\n",
            _ => "# Discovery\n\nObserved checkout failure in the parser boundary.\n",
        },
    )
    .unwrap();
    fs::create_dir_all(workspace.join(".canon/runs/canon-run-implement")).unwrap();
    fs::write(
        workspace.join(".canon/runs/canon-run-implement/implementation.md"),
        "# Implementation\n\nPrepared governed implementation guidance.\n",
    )
    .unwrap();
    fs::create_dir_all(workspace.join(".canon/runs/canon-run-approval")).unwrap();
    fs::write(
        workspace.join(".canon/runs/canon-run-approval/discovery.md"),
        "# Discovery\n\nApproval-gated governed investigation.\n",
    )
    .unwrap();
    fs::create_dir_all(workspace.join(".canon/runs/canon-run-security")).unwrap();
    fs::write(
        workspace.join(".canon/runs/canon-run-security/security-assessment.md"),
        "# Security Assessment\n\nValidated the bounded security review for the verify stage.\n",
    )
    .unwrap();
    fs::create_dir_all(workspace.join(".canon/runs/canon-run-security-approval")).unwrap();
    fs::write(
        workspace.join(".canon/runs/canon-run-security-approval/security-assessment.md"),
        "# Security Assessment\n\nApproval-gated governed security review for the verify stage.\n",
    )
    .unwrap();
    if matches!(scenario, CanonFixtureScenario::Approval) {
        fs::write(workspace.join(".canon/approval-state.txt"), "requested\n").unwrap();
    }
    if matches!(scenario, CanonFixtureScenario::VerifySecurityApproval) {
        fs::write(workspace.join(".canon/approval-state.txt"), "requested\n").unwrap();
    }
}

fn write_canon_stub_script(workspace: &Path, scenario: CanonFixtureScenario) -> PathBuf {
    let script_path = workspace.join(".boundline/canon-stub.sh");
    let script = match scenario {
        CanonFixtureScenario::Reusable => {
            r#"#!/bin/sh
request=$(cat)
case "$request" in
  *'"mode":"implementation"'*)
    run_ref="canon-run-implement"
    packet_ref=".canon/runs/canon-run-implement"
    document_ref="$packet_ref/implementation.md"
    headline="implementation packet ready"
    ;;
  *)
    run_ref="canon-run-investigate"
    packet_ref=".canon/runs/canon-run-investigate"
    document_ref="$packet_ref/discovery.md"
    headline="discovery packet ready"
    ;;
esac
printf '{"status":"governed_ready","run_ref":"%s","packet_ref":"%s","expected_document_refs":["%s"],"document_refs":["%s"],"approval_state":"not_needed","packet_readiness":"reusable","missing_sections":[],"headline":"%s","message":"Canon completed the governed stage"}' "$run_ref" "$packet_ref" "$document_ref" "$document_ref" "$headline"
"#
        }
        CanonFixtureScenario::RejectedPacket => {
            r#"#!/bin/sh
cat >/dev/null
printf '{"status":"governed_ready","run_ref":"canon-run-investigate","packet_ref":".canon/runs/canon-run-investigate","expected_document_refs":[".canon/runs/canon-run-investigate/discovery.md"],"document_refs":[".canon/runs/canon-run-investigate/discovery.md"],"approval_state":"not_needed","packet_readiness":"reusable","missing_sections":[],"headline":"discovery packet pending","message":"Canon completed the governed stage"}'
"#
        }
        CanonFixtureScenario::Approval => {
            r#"#!/bin/sh
request=$(cat)
case "$request" in
  *'"request_kind":"refresh"'*)
    state=$(cat .canon/approval-state.txt 2>/dev/null | tr -d '\n')
    if [ "$state" = "granted" ]; then
      printf '{"status":"governed_ready","run_ref":"canon-run-approval","packet_ref":".canon/runs/canon-run-approval","expected_document_refs":[".canon/runs/canon-run-approval/discovery.md"],"document_refs":[".canon/runs/canon-run-approval/discovery.md"],"approval_state":"granted","packet_readiness":"reusable","missing_sections":[],"headline":"approval granted packet ready","message":"Canon approval granted"}'
    else
      printf '{"status":"awaiting_approval","run_ref":"canon-run-approval","packet_ref":".canon/runs/canon-run-approval","expected_document_refs":[".canon/runs/canon-run-approval/discovery.md"],"document_refs":[],"approval_state":"requested","packet_readiness":"pending","missing_sections":[],"headline":"awaiting approval","message":"Canon is waiting for approval"}'
    fi
    ;;
  *)
    printf '{"status":"awaiting_approval","run_ref":"canon-run-approval","packet_ref":".canon/runs/canon-run-approval","expected_document_refs":[".canon/runs/canon-run-approval/discovery.md"],"document_refs":[],"approval_state":"requested","packet_readiness":"pending","missing_sections":[],"headline":"awaiting approval","message":"Canon is waiting for approval"}'
    ;;
esac
"#
        }
        CanonFixtureScenario::AutopilotBlocked => {
            unreachable!("blocked scenario should not create a stub")
        }
        CanonFixtureScenario::VerifySecurityReusable => {
            r#"#!/bin/sh
cat >/dev/null
printf '{"status":"governed_ready","run_ref":"canon-run-security","packet_ref":".canon/runs/canon-run-security","expected_document_refs":[".canon/runs/canon-run-security/security-assessment.md"],"document_refs":[".canon/runs/canon-run-security/security-assessment.md"],"approval_state":"not_needed","packet_readiness":"reusable","missing_sections":[],"headline":"security assessment packet ready","message":"Canon completed the governed security assessment"}'
"#
        }
        CanonFixtureScenario::VerifySecurityApproval => {
            r#"#!/bin/sh
request=$(cat)
case "$request" in
    *'"request_kind":"refresh"'*)
        state=$(cat .canon/approval-state.txt 2>/dev/null | tr -d '\n')
        if [ "$state" = "granted" ]; then
            printf '{"status":"governed_ready","run_ref":"canon-run-security-approval","packet_ref":".canon/runs/canon-run-security-approval","expected_document_refs":[".canon/runs/canon-run-security-approval/security-assessment.md"],"document_refs":[".canon/runs/canon-run-security-approval/security-assessment.md"],"approval_state":"granted","packet_readiness":"reusable","missing_sections":[],"headline":"security assessment approval granted","message":"Canon approval granted for the governed security assessment"}'
        else
            printf '{"status":"awaiting_approval","run_ref":"canon-run-security-approval","packet_ref":".canon/runs/canon-run-security-approval","expected_document_refs":[".canon/runs/canon-run-security-approval/security-assessment.md"],"document_refs":[],"approval_state":"requested","packet_readiness":"pending","missing_sections":[],"headline":"awaiting security approval","message":"Canon is waiting for security approval"}'
        fi
        ;;
    *)
        printf '{"status":"awaiting_approval","run_ref":"canon-run-security-approval","packet_ref":".canon/runs/canon-run-security-approval","expected_document_refs":[".canon/runs/canon-run-security-approval/security-assessment.md"],"document_refs":[],"approval_state":"requested","packet_readiness":"pending","missing_sections":[],"headline":"awaiting security approval","message":"Canon is waiting for security approval"}'
        ;;
esac
"#
        }
    };
    fs::write(&script_path, script).unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    script_path
}

fn execution_attempt(
    attempt_id: &str,
    summary: &str,
    failure_mode: &str,
    find: &str,
    replace: &str,
) -> serde_json::Value {
    serde_json::json!({
        "attempt_id": attempt_id,
        "summary": summary,
        "failure_mode": failure_mode,
        "changes": [
            {
                "path": "src/lib.rs",
                "find": find,
                "replace": replace,
            }
        ]
    })
}
