//! Product fingerprint fixtures cover Git state and explicit exclusion policy.

use std::fmt::Debug;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use boundline_core::transaction::fingerprint::{
    FingerprintError, FingerprintPolicy, ProductFingerprint,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

fn require_eq<T: Debug + PartialEq>(actual: T, expected: T, message: &str) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("{message}: {actual:?} != {expected:?}")).into())
    }
}

fn git(root: &Path, arguments: &[&str]) -> TestResult {
    let output = Command::new("git").args(arguments).current_dir(root).output()?;
    require(output.status.success(), &String::from_utf8_lossy(&output.stderr))
}

fn repository() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    git(root.path(), &["init", "-b", "main"])?;
    git(root.path(), &["config", "user.name", "Boundline Test"])?;
    git(root.path(), &["config", "user.email", "boundline@example.invalid"])?;
    git(root.path(), &["config", "commit.gpgsign", "false"])?;
    std::fs::write(root.path().join("tracked.txt"), "one\n")?;
    git(root.path(), &["add", "tracked.txt"])?;
    git(root.path(), &["commit", "-m", "baseline"])?;
    Ok(root)
}

#[test]
fn fingerprint_changes_for_index_worktree_untracked_rename_delete_mode_symlink_and_binary()
-> TestResult {
    let root = repository()?;
    let policy = FingerprintPolicy::new(["admitted.bin"]);
    let baseline = ProductFingerprint::capture(root.path(), &policy)?;
    std::fs::write(root.path().join("tracked.txt"), "two\n")?;
    let worktree = ProductFingerprint::capture(root.path(), &policy)?;
    require(baseline != worktree, "worktree content omitted")?;
    git(root.path(), &["add", "tracked.txt"])?;
    let index = ProductFingerprint::capture(root.path(), &policy)?;
    require(worktree != index, "index state omitted")?;
    std::fs::write(root.path().join("admitted.bin"), [0_u8, 1, 255])?;
    let binary = ProductFingerprint::capture(root.path(), &policy)?;
    require(index != binary, "admitted binary omitted")?;
    git(root.path(), &["mv", "tracked.txt", "renamed.txt"])?;
    let renamed = ProductFingerprint::capture(root.path(), &policy)?;
    require(binary != renamed, "rename omitted")?;
    let mut permissions = std::fs::metadata(root.path().join("renamed.txt"))?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(root.path().join("renamed.txt"), permissions)?;
    let executable = ProductFingerprint::capture(root.path(), &policy)?;
    require(renamed != executable, "executable bit omitted")?;
    std::os::unix::fs::symlink("renamed.txt", root.path().join("link"))?;
    git(root.path(), &["add", "link"])?;
    let symlink = ProductFingerprint::capture(root.path(), &policy)?;
    require(executable != symlink, "symlink omitted")?;
    std::fs::remove_file(root.path().join("renamed.txt"))?;
    let deleted = ProductFingerprint::capture(root.path(), &policy)?;
    require(symlink != deleted, "delete omitted")
}

#[test]
fn ignored_cache_and_boundline_metadata_are_deliberately_excluded() -> TestResult {
    let root = repository()?;
    std::fs::write(root.path().join(".gitignore"), "target/\n.boundline/\n*.cache\n")?;
    git(root.path(), &["add", ".gitignore"])?;
    git(root.path(), &["commit", "-m", "ignore policy"])?;
    let policy = FingerprintPolicy::new(std::iter::empty::<&str>());
    let baseline = ProductFingerprint::capture(root.path(), &policy)?;
    std::fs::create_dir_all(root.path().join("target"))?;
    std::fs::create_dir_all(root.path().join(".boundline"))?;
    std::fs::write(root.path().join("target/cache"), "derived")?;
    std::fs::write(root.path().join(".boundline/lease.json"), "metadata")?;
    std::fs::write(root.path().join("tool.cache"), "derived")?;
    require_eq(
        ProductFingerprint::capture(root.path(), &policy)?,
        baseline,
        "excluded state changed product fingerprint",
    )
}

#[test]
fn schema_is_versioned_and_case_folded_unicode_collisions_fail_closed() -> TestResult {
    let root = repository()?;
    std::fs::write(root.path().join("é.txt"), "one")?;
    let policy = FingerprintPolicy::new(["é.txt", "e\u{301}.txt"]);
    require(
        matches!(
            ProductFingerprint::capture(root.path(), &policy),
            Err(FingerprintError::PathCollision { .. })
        ),
        "case-folded collision admitted",
    )?;
    require_eq(
        ProductFingerprint::schema_version(),
        "boundline-product-fingerprint-v1",
        "schema version",
    )
}
