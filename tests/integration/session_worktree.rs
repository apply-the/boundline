//! Persistent session worktrees survive process restart and obey retention state.

use std::path::Path;
use std::process::Command;

use boundline_core::execution::worktree::{
    ManagedSessionState, SessionWorktreeError, SessionWorktreeManager,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}
fn git(root: &Path, arguments: &[&str]) -> TestResult {
    let output = Command::new("git").args(arguments).current_dir(root).output()?;
    require(output.status.success(), &String::from_utf8_lossy(&output.stderr))
}
fn repository(root: &Path) -> TestResult {
    git(root, &["init", "-b", "main"])?;
    git(root, &["config", "user.name", "Boundline Test"])?;
    git(root, &["config", "user.email", "boundline@example.invalid"])?;
    git(root, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(root.join(".gitignore"), ".boundline/\n")?;
    std::fs::write(root.join("tracked.txt"), "baseline\n")?;
    git(root, &["add", ".gitignore", "tracked.txt"])?;
    git(root, &["commit", "-m", "baseline"])
}

#[test]
fn managed_worktree_is_external_persistent_and_resumable() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let authoritative = fixture.path().join("authoritative");
    let state_root = fixture.path().join("state-root");
    std::fs::create_dir(&authoritative)?;
    repository(&authoritative)?;
    let manager = SessionWorktreeManager::open(&authoritative, &state_root)?;
    let created = manager.create("session-1")?;
    let state_root = std::fs::canonicalize(state_root)?;
    let authoritative = std::fs::canonicalize(authoritative)?;
    require(
        created.path().starts_with(&state_root),
        "managed worktree is outside the managed state root",
    )?;
    require(
        !created.path().starts_with(&authoritative),
        "managed worktree entered authoritative checkout",
    )?;
    std::fs::write(created.path().join("tracked.txt"), "session mutation\n")?;
    drop(manager);
    let reopened = SessionWorktreeManager::open(&authoritative, &state_root)?;
    let resumed = reopened.resume("session-1")?;
    require(
        std::fs::read_to_string(resumed.path().join("tracked.txt"))? == "session mutation\n",
        "restart lost dirty session state",
    )?;
    require(
        std::fs::read_to_string(authoritative.join("tracked.txt"))? == "baseline\n",
        "authoritative worktree changed before publication",
    )
}

#[test]
fn active_and_nonterminal_states_cannot_be_cleaned() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let authoritative = fixture.path().join("authoritative");
    let state_root = fixture.path().join("state-root");
    std::fs::create_dir(&authoritative)?;
    repository(&authoritative)?;
    let manager = SessionWorktreeManager::open(&authoritative, &state_root)?;
    manager.create("session-1")?;
    for state in ManagedSessionState::retained_states() {
        manager.set_state("session-1", state)?;
        require(
            matches!(manager.cleanup("session-1"), Err(SessionWorktreeError::RetentionRequired(_))),
            "retained state was deleted",
        )?;
    }
    manager.set_state("session-1", ManagedSessionState::Terminal)?;
    manager.cleanup("session-1")?;
    require(!manager.session_path("session-1").exists(), "terminal worktree survived cleanup")
}

#[test]
fn explicit_abort_is_durable_before_cleanup() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let authoritative = fixture.path().join("authoritative");
    let state_root = fixture.path().join("state-root");
    std::fs::create_dir(&authoritative)?;
    repository(&authoritative)?;
    let manager = SessionWorktreeManager::open(&authoritative, &state_root)?;
    manager.create("session-1")?;
    manager.abort("session-1")?;
    require(
        manager.resume("session-1")?.state() == ManagedSessionState::Aborted,
        "abort was not durable",
    )?;
    manager.cleanup("session-1")?;
    Ok(())
}

#[test]
fn state_root_inside_authoritative_worktree_fails_closed() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let authoritative = fixture.path().join("authoritative");
    std::fs::create_dir(&authoritative)?;
    repository(&authoritative)?;

    require(
        matches!(
            SessionWorktreeManager::open(&authoritative, authoritative.join("state")),
            Err(SessionWorktreeError::NestedStateRoot)
        ),
        "managed state root was admitted inside the authoritative checkout",
    )
}

#[test]
fn duplicate_session_creation_preserves_the_existing_worktree() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let authoritative = fixture.path().join("authoritative");
    let state_root = fixture.path().join("state-root");
    std::fs::create_dir(&authoritative)?;
    repository(&authoritative)?;
    let manager = SessionWorktreeManager::open(&authoritative, &state_root)?;
    let created = manager.create("session-1")?;

    require(
        matches!(manager.create("session-1"), Err(SessionWorktreeError::AlreadyExists)),
        "duplicate session replaced its managed checkout",
    )?;
    require(created.path().is_dir(), "duplicate admission removed the existing checkout")
}
