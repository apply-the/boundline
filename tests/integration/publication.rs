//! Verified candidate publication uses preconditions and target-ref compare-and-swap.

use std::path::Path;
use std::process::Command;
#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

use boundline_core::publication::transaction::{PublicationError, PublicationTransaction};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}
fn output(root: &Path, arguments: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let result = Command::new("git").args(arguments).current_dir(root).output()?;
    require(result.status.success(), &String::from_utf8_lossy(&result.stderr))?;
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}
fn git(root: &Path, arguments: &[&str]) -> TestResult {
    output(root, arguments).map(|_| ())
}
fn fixture() -> Result<(tempfile::TempDir, std::path::PathBuf), Box<dyn std::error::Error>> {
    let fixture = tempfile::tempdir()?;
    let repository = fixture.path().join("repository");
    std::fs::create_dir(&repository)?;
    git(&repository, &["init", "-b", "main"])?;
    git(&repository, &["config", "user.name", "Boundline Test"])?;
    git(&repository, &["config", "user.email", "boundline@example.invalid"])?;
    git(&repository, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(repository.join("tracked.txt"), "base\n")?;
    git(&repository, &["add", "tracked.txt"])?;
    git(&repository, &["commit", "-m", "base"])?;
    Ok((fixture, repository))
}
fn session(repository: &Path, path: &Path) -> TestResult {
    git(repository, &["worktree", "add", "--detach", path.to_str().ok_or("session path")?, "HEAD"])
}

#[cfg(unix)]
fn install_reference_transaction_hook(repository: &Path, body: &str) -> TestResult {
    let hooks = repository.join(".git/hooks");
    std::fs::create_dir_all(&hooks)?;
    let hook = hooks.join("reference-transaction");
    std::fs::write(&hook, format!("#!/bin/sh\nif [ \"$1\" = committed ]; then\n{body}\nfi\n"))?;
    std::fs::set_permissions(&hook, Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(unix)]
fn install_post_checkout_hook(repository: &Path, body: &str) -> TestResult {
    let hooks = repository.join(".git/hooks");
    std::fs::create_dir_all(&hooks)?;
    let hook = hooks.join("post-checkout");
    std::fs::write(&hook, format!("#!/bin/sh\n{body}\n"))?;
    std::fs::set_permissions(&hook, Permissions::from_mode(0o755))?;
    Ok(())
}

#[test]
fn candidate_commit_publishes_by_cas_and_leaves_authoritative_clean() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "published\n")?;
    std::fs::write(session_path.join("new.txt"), "new\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    require(prepared.affected_paths().len() == 2, "candidate ledger omitted a path")?;
    let result = prepared.publish()?;
    require(
        output(&repository, &["rev-parse", "HEAD"])? == result.published_commit(),
        "HEAD did not advance",
    )?;
    require(
        output(&repository, &["rev-parse", "HEAD^{tree}"])? == result.final_tree_digest(),
        "published tree projection did not match the authoritative commit",
    )?;
    require(
        output(&repository, &["status", "--porcelain"])?.is_empty(),
        "authoritative worktree remained dirty",
    )?;
    require(
        std::fs::read_to_string(repository.join("tracked.txt"))? == "published\n",
        "candidate content not published",
    )
}

#[test]
fn external_path_change_fails_precondition_without_overwrite() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "candidate\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    std::fs::write(repository.join("tracked.txt"), "external\n")?;
    require(
        matches!(prepared.publish(), Err(PublicationError::AuthoritativePreconditionFailed)),
        "external edit overwritten",
    )?;
    require(
        std::fs::read_to_string(repository.join("tracked.txt"))? == "external\n",
        "failed publication changed user content",
    )
}

#[test]
fn second_session_becomes_rebase_required_after_first_publication() -> TestResult {
    let (fixture, repository) = fixture()?;
    let first_path = fixture.path().join("first");
    let second_path = fixture.path().join("second");
    session(&repository, &first_path)?;
    session(&repository, &second_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(first_path.join("tracked.txt"), "first\n")?;
    std::fs::write(second_path.join("tracked.txt"), "second\n")?;
    let first =
        PublicationTransaction::prepare(&repository, &first_path, "refs/heads/main", &base)?;
    let second =
        PublicationTransaction::prepare(&repository, &second_path, "refs/heads/main", &base)?;
    first.publish()?;
    require(
        matches!(second.publish(), Err(PublicationError::PublicationRebaseRequired)),
        "stale candidate auto-merged or published",
    )
}

#[test]
fn publication_replaces_adds_and_deletes_through_the_affected_path_ledger() -> TestResult {
    let (fixture, repository) = fixture()?;
    std::fs::write(repository.join("deleted.txt"), "remove me\n")?;
    git(&repository, &["add", "deleted.txt"])?;
    git(&repository, &["commit", "-m", "add deletion fixture"])?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::remove_file(session_path.join("deleted.txt"))?;
    std::fs::write(session_path.join("tracked.txt"), "replaced\n")?;
    std::fs::create_dir(session_path.join("nested"))?;
    std::fs::write(session_path.join("nested/new.txt"), "added\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    require(prepared.affected_paths().len() == 3, "affected path ledger is incomplete")?;
    prepared.publish()?;
    require(!repository.join("deleted.txt").exists(), "deleted candidate path survived")?;
    require(
        std::fs::read_to_string(repository.join("tracked.txt"))? == "replaced\n",
        "replacement was not published",
    )?;
    require(
        std::fs::read_to_string(repository.join("nested/new.txt"))? == "added\n",
        "addition was not published",
    )
}

#[test]
fn invalid_target_ref_is_rejected_before_candidate_mutation() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "candidate\n")?;

    require(
        matches!(
            PublicationTransaction::prepare(&repository, &session_path, "main", &base),
            Err(PublicationError::InvalidName)
        ),
        "non-canonical target ref entered publication",
    )?;
    require(
        output(&session_path, &["rev-parse", "HEAD"])? == base,
        "invalid target ref created a candidate commit",
    )
}

#[test]
fn mismatched_session_base_is_rejected_before_candidate_commit() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "candidate\n")?;

    require(
        matches!(
            PublicationTransaction::prepare(
                &repository,
                &session_path,
                "refs/heads/main",
                "0000000000000000000000000000000000000000",
            ),
            Err(PublicationError::PublicationRebaseRequired)
        ),
        "session with a mismatched admitted base created a candidate",
    )?;
    require(
        output(&session_path, &["rev-parse", "HEAD"])? == base,
        "base mismatch advanced the session checkout",
    )
}

#[test]
fn repository_publication_lock_rejects_a_second_publisher() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "candidate\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    let common = output(&repository, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    let lock_directory = Path::new(&common).join("boundline/publication");
    std::fs::create_dir_all(&lock_directory)?;
    std::fs::write(lock_directory.join("repository.lock"), "other-publisher\n")?;

    require(
        matches!(prepared.publish(), Err(PublicationError::PublicationLocked)),
        "second publisher bypassed the repository-scoped lock",
    )?;
    require(
        output(&repository, &["rev-parse", "HEAD"])? == base,
        "lock rejection advanced the authoritative branch",
    )
}

#[test]
fn non_git_authoritative_directory_fails_before_publication_state() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let authoritative = fixture.path().join("authoritative");
    let session_path = fixture.path().join("session");
    std::fs::create_dir(&authoritative)?;
    std::fs::create_dir(&session_path)?;

    require(
        matches!(
            PublicationTransaction::prepare(
                &authoritative,
                &session_path,
                "refs/heads/main",
                "base",
            ),
            Err(PublicationError::Git(_))
        ),
        "ordinary directory entered the publication transaction",
    )
}

#[cfg(unix)]
#[test]
fn concurrent_untracked_collision_after_cas_stops_path_replacement() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("new.txt"), "candidate\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    install_reference_transaction_hook(
        &repository,
        "printf 'external\\n' > \"$(git rev-parse --show-toplevel)/new.txt\"",
    )?;

    require(
        matches!(prepared.publish(), Err(PublicationError::AuthoritativePreconditionFailed)),
        "concurrent untracked collision was overwritten after compare-and-swap",
    )?;
    require(
        std::fs::read_to_string(repository.join("new.txt"))? == "external\n",
        "failed publication replaced the concurrent file",
    )
}

#[cfg(unix)]
#[test]
fn unexpected_index_entry_after_cas_fails_final_tree_verification() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "candidate\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    install_reference_transaction_hook(
        &repository,
        "root=$(git rev-parse --show-toplevel)\nprintf 'external\\n' > \"$root/concurrent.txt\"\ngit -C \"$root\" add concurrent.txt",
    )?;

    require(
        matches!(prepared.publish(), Err(PublicationError::FinalFingerprintMismatch)),
        "unexpected post-CAS index entry passed final tree verification",
    )?;
    require(
        output(&repository, &["status", "--porcelain"])?.contains("concurrent.txt"),
        "final-tree failure erased the unexplained concurrent state",
    )
}

#[cfg(unix)]
#[test]
fn unexplained_post_checkout_file_fails_final_cleanliness_verification() -> TestResult {
    let (fixture, repository) = fixture()?;
    let session_path = fixture.path().join("session");
    session(&repository, &session_path)?;
    let base = output(&repository, &["rev-parse", "HEAD"])?;
    std::fs::write(session_path.join("tracked.txt"), "candidate\n")?;
    let prepared =
        PublicationTransaction::prepare(&repository, &session_path, "refs/heads/main", &base)?;
    install_post_checkout_hook(
        &repository,
        "printf 'external\\n' > \"$(git rev-parse --show-toplevel)/unexplained.txt\"",
    )?;

    require(
        matches!(prepared.publish(), Err(PublicationError::FinalFingerprintMismatch)),
        "unexplained post-checkout file passed final cleanliness verification",
    )?;
    require(
        std::fs::read_to_string(repository.join("unexplained.txt"))? == "external\n",
        "final cleanliness failure removed unexplained external state",
    )
}
