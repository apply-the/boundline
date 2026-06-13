#![allow(dead_code)]

use std::fs;
use std::path::Path;

use super::workspace_fixture::TempGitWorkspace;

const FIXTURE_ROOT: &str = "tests/fixtures/completion_verification_runtime";
const CARGO_TOML_PATH: &str = "Cargo.toml";
const LIB_RS_PATH: &str = "src/lib.rs";
const TEST_RS_PATH: &str = "tests/smoke.rs";
const DOC_PATH: &str = "docs/release-readiness.md";
const GITIGNORE_PATH: &str = ".gitignore";

/// Creates a temporary git workspace seeded with the completion-verification fixture tree.
pub fn completion_verification_workspace(prefix: &str) -> TempGitWorkspace {
    TempGitWorkspace::with_initializer(prefix, |root| {
        copy_fixture_file(root, CARGO_TOML_PATH);
        copy_fixture_file(root, LIB_RS_PATH);
        copy_fixture_file(root, TEST_RS_PATH);
        copy_fixture_file(root, DOC_PATH);
        copy_fixture_file(root, GITIGNORE_PATH);
    })
}

/// Writes or overwrites one workspace-relative file.
pub fn write_workspace_file(
    workspace_root: &Path,
    relative_path: &str,
    contents: &str,
) -> Result<(), String> {
    let target = workspace_root.join(relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(target, contents).map_err(|error| error.to_string())
}

fn copy_fixture_file(workspace_root: &Path, relative_path: &str) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT).join(relative_path);
    let target = workspace_root.join(relative_path);

    if let Some(parent) = target.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(contents) = fs::read(&source) {
        let _ = fs::write(target, contents);
    }
}
