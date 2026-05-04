#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

const RUNTIME_CARGO_TOML: &str = concat!(
    "[package]\n",
    "name = \"runtime_refoundation_fixture\"\n",
    "version = \"0.1.0\"\n",
    "edition = \"2024\"\n",
);

const RED_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left - right\n", "}\n");

const MULTIPLY_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left * right\n", "}\n");

const NO_ACTIONABLE_LIB_RS: &str =
    concat!("pub fn workspace_summary_output() -> &'static str {\n", "    \"todo\"\n", "}\n",);

const FIXTURE_TEST_RS: &str = concat!(
    "use runtime_refoundation_fixture::add;\n\n",
    "#[test]\n",
    "fn runtime_refoundation_drives_red_to_green() {\n",
    "    assert_eq!(add(2, 2), 4);\n",
    "}\n",
);

const NO_ACTIONABLE_TEST_RS: &str = concat!(
    "use runtime_refoundation_fixture::workspace_summary_output;\n\n",
    "#[test]\n",
    "fn workspace_summary_output_is_available() {\n",
    "    assert_eq!(workspace_summary_output(), \"todo\");\n",
    "}\n",
);

pub fn temp_runtime_refoundation_workspace(prefix: &str) -> PathBuf {
    create_runtime_workspace(prefix, RED_LIB_RS)
}

pub fn temp_runtime_refoundation_failure_workspace(prefix: &str) -> PathBuf {
    create_runtime_workspace(prefix, MULTIPLY_LIB_RS)
}

pub fn temp_runtime_refoundation_no_action_workspace(prefix: &str) -> PathBuf {
    create_runtime_workspace(prefix, NO_ACTIONABLE_LIB_RS)
}

pub fn temp_runtime_refoundation_compat_workspace(prefix: &str) -> PathBuf {
    let workspace = create_runtime_workspace(prefix, RED_LIB_RS);
    write_execution_profile(&workspace);
    workspace
}

pub fn temp_runtime_refoundation_governed_workspace(prefix: &str) -> PathBuf {
    let workspace = create_runtime_workspace(prefix, RED_LIB_RS);
    write_canon_artifact(
        &workspace,
        Path::new("requirements.md"),
        "# Governed Requirements\n\n- keep the change bounded\n- preserve auditability\n",
    );
    workspace
}

pub fn write_canon_artifact(workspace: &Path, relative_path: &Path, contents: &str) -> PathBuf {
    let path = workspace.join(".canon").join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents).unwrap();
    path
}

fn create_runtime_workspace(prefix: &str, source_contents: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();

    fs::write(workspace.join("Cargo.toml"), RUNTIME_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), source_contents).unwrap();
    if source_contents.contains("pub fn add") {
        fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    } else if source_contents.contains("workspace_summary_output") {
        fs::write(workspace.join("tests/workspace_summary_output.rs"), NO_ACTIONABLE_TEST_RS)
            .unwrap();
    }

    workspace
}

fn write_execution_profile(workspace: &Path) {
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "runtime-refoundation-compat-profile",
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
            ]
        }))
        .unwrap(),
    )
    .unwrap();
}
