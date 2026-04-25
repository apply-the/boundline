use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use uuid::Uuid;

const FIXTURE_CARGO_TOML: &str = concat!(
    "[package]\n",
    "name = \"synod-fixture\"\n",
    "version = \"0.1.0\"\n",
    "edition = \"2024\"\n",
);

const RED_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left - right\n", "}\n",);

const GREEN_LIB_RS: &str =
    concat!("pub fn add(left: i32, right: i32) -> i32 {\n", "    left + right\n", "}\n",);

const FIXTURE_TEST_RS: &str = concat!(
    "use synod_fixture::add;\n\n",
    "#[test]\n",
    "fn synod_drives_red_to_green() {\n",
    "    assert_eq!(add(2, 2), 4);\n",
    "}\n",
);

pub fn temp_fixture_workspace(prefix: &str) -> PathBuf {
    create_fixture_workspace(prefix, "left - right", "left + right")
}

pub fn temp_broken_fixture_workspace(prefix: &str) -> PathBuf {
    create_fixture_workspace(prefix, "left * right", "left + right")
}

pub fn run_synod(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap()
}

pub fn run_synod_in(workspace: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod")).args(args).current_dir(workspace).output().unwrap()
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

fn create_fixture_workspace(prefix: &str, find: &str, replace: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".synod")).unwrap();

    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    fs::write(
        workspace.join(".synod/fixture.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "red-to-green",
            "test_command": {
                "program": "cargo",
                "args": ["test", "--quiet"],
            },
            "file_patches": [
                {
                    "path": "src/lib.rs",
                    "find": find,
                    "replace": replace,
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    debug_assert_ne!(RED_LIB_RS, GREEN_LIB_RS);
    workspace
}
