//! Execution leases serialize session executors and fence superseded owners.

use std::sync::{Arc, Barrier};

use boundline_core::execution::lease::{ExecutionLeaseError, ExecutionLeaseStore};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

#[test]
fn concurrent_admission_allows_exactly_one_executor() -> TestResult {
    let root = tempfile::tempdir()?;
    let store = Arc::new(ExecutionLeaseStore::open(root.path())?);
    store.set_revision("session-1", 4)?;
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for owner in ["executor-a", "executor-b"] {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            store.acquire("session-1", owner, 4)
        }));
    }
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| std::io::Error::other("lease thread panicked"))?;
    require(
        results.iter().filter(|result| result.is_ok()).count() == 1,
        "more than one executor admitted",
    )?;
    require(
        results
            .iter()
            .filter(|result| matches!(result, Err(ExecutionLeaseError::AlreadyHeld)))
            .count()
            == 1,
        "loser did not fail closed",
    )
}

#[test]
fn reassignment_increments_token_and_rejects_superseded_writes() -> TestResult {
    let root = tempfile::tempdir()?;
    let store = ExecutionLeaseStore::open(root.path())?;
    store.set_revision("session-1", 4)?;
    let first = store.acquire("session-1", "executor-a", 4)?;
    store.release(&first)?;
    let second = store.acquire("session-1", "executor-b", 4)?;
    require(second.fencing_token() > first.fencing_token(), "fencing token did not increase")?;
    require(
        matches!(store.validate_write(&first), Err(ExecutionLeaseError::StaleFencingToken)),
        "superseded owner wrote state",
    )?;
    store.validate_write(&second)?;
    Ok(())
}

#[test]
fn expected_revision_is_checked_before_executor_admission() -> TestResult {
    let root = tempfile::tempdir()?;
    let store = ExecutionLeaseStore::open(root.path())?;
    require(
        matches!(
            store.acquire("session-1", "executor", 2),
            Err(ExecutionLeaseError::StaleRevision { .. })
        ),
        "stale revision admitted",
    )?;
    store.set_revision("session-1", 2)?;
    let lease = store.acquire("session-1", "executor", 2)?;
    require(lease.expected_revision() == 2, "lease lost expected revision")
}

#[test]
fn active_executor_blocks_revision_rewrite_and_invalid_session_paths() -> TestResult {
    let root = tempfile::tempdir()?;
    let store = ExecutionLeaseStore::open(root.path())?;
    store.set_revision("session-1", 4)?;
    let lease = store.acquire("session-1", "executor", 4)?;

    require(
        matches!(store.set_revision("session-1", 5), Err(ExecutionLeaseError::AlreadyHeld)),
        "active executor allowed the admitted revision to change",
    )?;
    require(
        matches!(
            store.acquire("../escape", "executor", 4),
            Err(ExecutionLeaseError::InvalidSessionId)
        ),
        "session identifier escaped the lease root",
    )?;
    store.validate_write(&lease)?;
    Ok(())
}
