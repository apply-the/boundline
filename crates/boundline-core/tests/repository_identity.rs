//! Clone-local repository identity and worktree-role contract tests.

use std::fmt::Debug;
use std::path::Path;
use std::process::Command;

use boundline_core::identity::repository::{
    RepositoryIdentityError, RepositoryIdentityStore, WorktreeRole,
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

fn git(path: &Path, arguments: &[&str]) -> TestResult {
    let output = Command::new("git").args(arguments).current_dir(path).output()?;
    require(
        output.status.success(),
        &format!("git failed: {}", String::from_utf8_lossy(&output.stderr)),
    )
}

fn fixture_repository(root: &Path) -> TestResult {
    git(root, &["init", "-b", "main"])?;
    git(root, &["config", "user.name", "Boundline Test"])?;
    git(root, &["config", "user.email", "boundline@example.invalid"])?;
    git(root, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(root.join("tracked.txt"), "baseline\n")?;
    git(root, &["add", "tracked.txt"])?;
    git(root, &["commit", "-m", "baseline"])
}

#[test]
fn linked_worktrees_share_clone_identity_but_not_worktree_identity() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let repository = fixture.path().join("repository");
    let linked = fixture.path().join("linked");
    std::fs::create_dir(&repository)?;
    fixture_repository(&repository)?;

    let authoritative = RepositoryIdentityStore::open(&repository, WorktreeRole::Authoritative)?;
    git(
        &repository,
        &["worktree", "add", linked.to_str().ok_or("linked path is not UTF-8")?, "-b", "session"],
    )?;
    let session = RepositoryIdentityStore::open(&linked, WorktreeRole::SessionManaged)?;

    require_eq(
        session.repository_id().clone(),
        authoritative.repository_id().clone(),
        "linked worktree repository identity",
    )?;
    require(
        session.worktree_id() != authoritative.worktree_id(),
        "linked worktree reused authoritative worktree identity",
    )?;
    require_eq(session.role(), WorktreeRole::SessionManaged, "session role")?;
    require_eq(authoritative.role(), WorktreeRole::Authoritative, "authoritative role")
}

#[test]
fn separate_clones_of_the_same_origin_have_distinct_local_identities() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let origin = fixture.path().join("origin");
    let clone_one = fixture.path().join("clone-one");
    let clone_two = fixture.path().join("clone-two");
    std::fs::create_dir(&origin)?;
    fixture_repository(&origin)?;
    git(fixture.path(), &["clone", origin.to_str().ok_or("origin path")?, "clone-one"])?;
    git(fixture.path(), &["clone", origin.to_str().ok_or("origin path")?, "clone-two"])?;

    let first = RepositoryIdentityStore::open(&clone_one, WorktreeRole::Authoritative)?;
    let second = RepositoryIdentityStore::open(&clone_two, WorktreeRole::Authoritative)?;

    require(first.repository_id() != second.repository_id(), "separate clones shared a lock domain")
}

#[test]
fn role_markers_are_stable_and_prevent_authoritative_role_confusion() -> TestResult {
    let fixture = tempfile::tempdir()?;
    fixture_repository(fixture.path())?;
    let first = RepositoryIdentityStore::open(fixture.path(), WorktreeRole::SessionManaged)?;
    let reopened = RepositoryIdentityStore::open(fixture.path(), WorktreeRole::SessionManaged)?;
    require_eq(reopened, first, "stable role marker")?;

    let confused = RepositoryIdentityStore::open(fixture.path(), WorktreeRole::Authoritative);
    require(
        matches!(confused, Err(RepositoryIdentityError::WorktreeRoleMismatch { .. })),
        "managed worktree opened as authoritative",
    )?;
    let marker = std::fs::read_to_string(fixture.path().join(".boundline/worktree-role.json"))?;
    require(!marker.contains(fixture.path().to_string_lossy().as_ref()), "marker leaked host path")
}
