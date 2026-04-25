use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Static seeded contents for the demo workspace.
///
/// `BUG_MARKER` is a sentinel string that lives inside [`BUGGY_SOURCE`] and gets
/// removed when [`FIXED_SOURCE`] replaces it. The tester adapter reads the file
/// and reports failure while the marker is present and success once it is gone.
pub const BUG_MARKER: &str = "// TODO-BUG: returns 1 instead of 0";

/// Initial buggy source seeded into the demo workspace.
pub const BUGGY_SOURCE: &str = "//! Seeded buggy module for the synod test-fix loop demo.\n\n// TODO-BUG: returns 1 instead of 0\npub fn answer() -> i32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn answer_should_be_zero() {\n        assert_eq!(answer(), 0);\n    }\n}\n";

/// Fixed source that the coder agent writes during the demo run.
pub const FIXED_SOURCE: &str = "//! Seeded fixed module for the synod test-fix loop demo.\n\npub fn answer() -> i32 {\n    0\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn answer_should_be_zero() {\n        assert_eq!(answer(), 0);\n    }\n}\n";

/// Partial fix the coder writes on its first successful attempt: still contains
/// the bug marker, so the tester triggers a replan.
pub const PARTIAL_FIX_SOURCE: &str = "//! Seeded partially-fixed module for the synod test-fix loop demo.\n\n// TODO-BUG: returns 1 instead of 0\npub fn answer() -> i32 {\n    // attempted partial fix; bug marker still present so tester will replan\n    1\n}\n";

/// Initial failing test definition file (informational; the actual verification is
/// performed in-process by the tester adapter reading the source file).
pub const FAILING_TEST_DEFINITION: &str = "// Failing test seed for the synod test-fix loop demo.\n// The tester adapter verifies that `answer()` returns 0 by reading\n// `src/buggy.rs` and looking for the absence of the bug marker.\n";

/// Resolved demo workspace layout on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoWorkspace {
    pub root: PathBuf,
    pub target_file: PathBuf,
    pub test_file: PathBuf,
    pub bug_marker: &'static str,
    pub fixed_content: &'static str,
}

impl DemoWorkspace {
    fn from_root(root: PathBuf) -> Self {
        let target_file = root.join("src").join("buggy.rs");
        let test_file = root.join("tests").join("buggy_test.rs");
        Self { root, target_file, test_file, bug_marker: BUG_MARKER, fixed_content: FIXED_SOURCE }
    }
}

/// Errors produced by the demo workspace helper.
#[derive(Debug, Error)]
pub enum DemoWorkspaceError {
    #[error("demo workspace root must be absolute: {0}")]
    RelativeRoot(PathBuf),
    #[error("demo workspace root must end in `.synod/demo-workspace`, got: {0}")]
    UnsafeRoot(PathBuf),
    #[error("failed to create demo workspace directory {0}: {1}")]
    CreateDirectory(PathBuf, io::Error),
    #[error("failed to remove existing demo workspace {0}: {1}")]
    Remove(PathBuf, io::Error),
    #[error("failed to write demo workspace file {0}: {1}")]
    WriteFile(PathBuf, io::Error),
}

fn validate_root(root: &Path) -> Result<(), DemoWorkspaceError> {
    if !root.is_absolute() {
        return Err(DemoWorkspaceError::RelativeRoot(root.to_path_buf()));
    }
    let last = root.file_name().and_then(|s| s.to_str()).unwrap_or_default();
    let parent_last =
        root.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or_default();
    if last != "demo-workspace" || parent_last != ".synod" {
        return Err(DemoWorkspaceError::UnsafeRoot(root.to_path_buf()));
    }
    Ok(())
}

/// Create the demo workspace at `root` if it does not exist, writing the seeded
/// buggy source file and the failing test definition.
pub fn seed_demo_workspace(root: &Path) -> Result<DemoWorkspace, DemoWorkspaceError> {
    validate_root(root)?;
    let workspace = DemoWorkspace::from_root(root.to_path_buf());

    fs::create_dir_all(&workspace.root)
        .map_err(|e| DemoWorkspaceError::CreateDirectory(workspace.root.clone(), e))?;
    if let Some(parent) = workspace.target_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DemoWorkspaceError::CreateDirectory(parent.to_path_buf(), e))?;
    }
    if let Some(parent) = workspace.test_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DemoWorkspaceError::CreateDirectory(parent.to_path_buf(), e))?;
    }

    fs::write(&workspace.target_file, BUGGY_SOURCE)
        .map_err(|e| DemoWorkspaceError::WriteFile(workspace.target_file.clone(), e))?;
    fs::write(&workspace.test_file, FAILING_TEST_DEFINITION)
        .map_err(|e| DemoWorkspaceError::WriteFile(workspace.test_file.clone(), e))?;

    Ok(workspace)
}

/// Reset the demo workspace at `root`: remove any existing contents and re-seed.
pub fn reset_demo_workspace(root: &Path) -> Result<DemoWorkspace, DemoWorkspaceError> {
    validate_root(root)?;
    if root.exists() {
        fs::remove_dir_all(root).map_err(|e| DemoWorkspaceError::Remove(root.to_path_buf(), e))?;
    }
    seed_demo_workspace(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_root(suffix: &str) -> PathBuf {
        let mut base = env::temp_dir();
        base.push(format!(
            "synod-demo-{}-{}",
            suffix,
            crate::domain::trace::current_timestamp_millis()
        ));
        base.push(".synod");
        base.push("demo-workspace");
        base
    }

    #[test]
    fn seed_creates_buggy_source_and_failing_test_files() {
        let root = temp_root("seed");
        let ws = seed_demo_workspace(&root).expect("seed succeeds");
        let src = fs::read_to_string(&ws.target_file).unwrap();
        let tst = fs::read_to_string(&ws.test_file).unwrap();
        assert!(src.contains(BUG_MARKER), "buggy source must contain bug marker");
        assert!(tst.contains("synod test-fix loop"));
        let _ = fs::remove_dir_all(root.parent().unwrap());
    }

    #[test]
    fn reset_removes_prior_state_and_reseeds() {
        let root = temp_root("reset");
        let ws = seed_demo_workspace(&root).expect("seed");
        // mutate the file as if a prior run had fixed it
        fs::write(&ws.target_file, FIXED_SOURCE).unwrap();
        assert!(!fs::read_to_string(&ws.target_file).unwrap().contains(BUG_MARKER));
        let ws2 = reset_demo_workspace(&root).expect("reset");
        assert_eq!(ws.target_file, ws2.target_file);
        assert!(fs::read_to_string(&ws2.target_file).unwrap().contains(BUG_MARKER));
        let _ = fs::remove_dir_all(root.parent().unwrap());
    }

    #[test]
    fn rejects_root_not_under_dot_synod_demo_workspace() {
        let mut bad = env::temp_dir();
        bad.push("synod-bad-root");
        bad.push("not-demo-workspace");
        assert!(matches!(seed_demo_workspace(&bad), Err(DemoWorkspaceError::UnsafeRoot(_))));
        assert!(matches!(reset_demo_workspace(&bad), Err(DemoWorkspaceError::UnsafeRoot(_))));
    }

    #[test]
    fn rejects_relative_root() {
        let bad = PathBuf::from(".synod/demo-workspace");
        assert!(matches!(seed_demo_workspace(&bad), Err(DemoWorkspaceError::RelativeRoot(_))));
    }
}
