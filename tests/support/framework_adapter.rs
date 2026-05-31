#![allow(dead_code)]

//! Shared fixtures for the framework-adapter feature tests.

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::workspace_fixture::TempGitWorkspace;

const HOST_REPO_NAME: &str = "boundline";
const TEMPLATE_REPO_NAME: &str = "boundline-framework-template";
const SPECKIT_REPO_NAME: &str = "boundline-adapter-speckit";
const CARGO_MANIFEST_FILE: &str = "Cargo.toml";
const README_FILE: &str = "README.md";
const TARGET_DIRECTORY: &str = "target";
const DEBUG_DIRECTORY: &str = "debug";
pub const SPECKIT_ADAPTER_ID: &str = "speckit";
pub const SPECKIT_BINARY_NAME: &str = "boundline-adapter-speckit";

static SPECKIT_BINARY_DIRECTORY: OnceLock<Result<PathBuf, String>> = OnceLock::new();

/// Creates a temporary git-backed workspace fixture for framework-adapter tests.
pub fn temp_framework_adapter_workspace(prefix: &str) -> TempGitWorkspace {
    TempGitWorkspace::new(prefix)
}

/// Returns the host repository root for the active test session.
pub fn host_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Returns the host repository root and validates the expected checkout name.
pub fn validated_host_repo_root() -> Result<PathBuf, Box<dyn Error>> {
    let root = host_repo_root();
    let repo_name = root.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        Box::new(io::Error::other("host repository root has no terminal path segment"))
            as Box<dyn Error>
    })?;
    if repo_name == HOST_REPO_NAME {
        Ok(root)
    } else {
        Err(Box::new(io::Error::other(format!(
            "expected host repository root name {HOST_REPO_NAME}, found {repo_name}"
        ))))
    }
}

/// Returns the parent directory that contains the host and sibling repositories.
pub fn sibling_repo_parent() -> Result<PathBuf, Box<dyn Error>> {
    host_repo_root().parent().map(Path::to_path_buf).ok_or_else(|| {
        Box::new(io::Error::other(
            "host repository has no parent directory for sibling adapter repos",
        )) as Box<dyn Error>
    })
}

/// Returns the reusable template repository root and verifies it exists.
pub fn template_repo_root() -> Result<PathBuf, Box<dyn Error>> {
    required_repo_path(TEMPLATE_REPO_NAME)
}

/// Returns the concrete Speckit adapter repository root and verifies it exists.
pub fn speckit_repo_root() -> Result<PathBuf, Box<dyn Error>> {
    required_repo_path(SPECKIT_REPO_NAME)
}

/// Returns the template repository manifest path and verifies it exists.
pub fn template_manifest_path() -> Result<PathBuf, Box<dyn Error>> {
    required_file_path(TEMPLATE_REPO_NAME, CARGO_MANIFEST_FILE)
}

/// Returns the Speckit repository manifest path and verifies it exists.
pub fn speckit_manifest_path() -> Result<PathBuf, Box<dyn Error>> {
    required_file_path(SPECKIT_REPO_NAME, CARGO_MANIFEST_FILE)
}

/// Returns the template repository README path and verifies it exists.
pub fn template_readme_path() -> Result<PathBuf, Box<dyn Error>> {
    required_file_path(TEMPLATE_REPO_NAME, README_FILE)
}

/// Returns the Speckit repository README path and verifies it exists.
pub fn speckit_readme_path() -> Result<PathBuf, Box<dyn Error>> {
    required_file_path(SPECKIT_REPO_NAME, README_FILE)
}

/// Builds the sibling Speckit binary once and returns its debug output directory.
pub fn built_speckit_binary_dir() -> Result<PathBuf, Box<dyn Error>> {
    cached_binary_directory(&SPECKIT_BINARY_DIRECTORY, SPECKIT_REPO_NAME, SPECKIT_BINARY_NAME)
}

/// Builds the sibling Speckit binary when the sibling repo is present.
pub fn optional_built_speckit_binary_dir() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let repo_root = sibling_repo_parent()?.join(SPECKIT_REPO_NAME);
    if repo_root.is_dir() { built_speckit_binary_dir().map(Some) } else { Ok(None) }
}

/// Returns the explicit operator command for activating the known Speckit profile.
pub fn speckit_registration_command() -> String {
    format!("boundline adapter add {SPECKIT_ADAPTER_ID}")
}

fn required_repo_path(repo_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = sibling_repo_parent()?.join(repo_name);
    if path.is_dir() {
        Ok(path)
    } else {
        Err(Box::new(io::Error::other(format!(
            "expected sibling repository directory at {}",
            path.display()
        ))))
    }
}

fn required_file_path(repo_name: &str, file_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = required_repo_path(repo_name)?.join(file_name);
    if path.is_file() {
        Ok(path)
    } else {
        Err(Box::new(io::Error::other(format!(
            "expected sibling repository file at {}",
            path.display()
        ))))
    }
}

fn cached_binary_directory(
    cache: &'static OnceLock<Result<PathBuf, String>>,
    repo_name: &str,
    binary_name: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let result = cache.get_or_init(|| build_binary_directory(repo_name, binary_name));
    match result {
        Ok(path) => Ok(path.clone()),
        Err(message) => Err(Box::new(io::Error::other(message.clone()))),
    }
}

fn build_binary_directory(repo_name: &str, binary_name: &str) -> Result<PathBuf, String> {
    let repo_root = required_repo_path(repo_name).map_err(|error| error.to_string())?;
    let output = Command::new("cargo")
        .args(["build", "--bin", binary_name])
        .current_dir(&repo_root)
        .output()
        .map_err(|error| {
            format!("failed to run cargo build in {}: {error}", repo_root.display())
        })?;
    if !output.status.success() {
        return Err(format!(
            "cargo build --bin {binary_name} failed in {}\nstdout:\n{}\nstderr:\n{}",
            repo_root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let binary_directory = repo_root.join(TARGET_DIRECTORY).join(DEBUG_DIRECTORY);
    let binary_path = binary_directory.join(binary_name);
    if binary_path.is_file() {
        Ok(binary_directory)
    } else {
        Err(format!("expected built binary at {} after cargo build", binary_path.display()))
    }
}
