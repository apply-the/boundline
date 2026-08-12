//! Verified Git commits publish through repository locks and target-ref CAS.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const PUBLICATION_SCHEMA_VERSION: &str = "boundline-publication-plan-v1";
const STATE_DIRECTORY: &str = "boundline/publication";
const LOCK_FILE: &str = "repository.lock";
const CANDIDATE_MESSAGE: &str = "Boundline verified candidate";

/// Durable publication intent prepared before authoritative mutation.
#[derive(Clone, Debug)]
pub struct PublicationTransaction {
    authoritative: PathBuf,
    session: PathBuf,
    common_directory: PathBuf,
    manifest: PublicationManifest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PublicationManifest {
    schema_version: String,
    publication_id: String,
    expected_base_revision: String,
    target_ref: String,
    candidate_commit: String,
    target_tree_digest: String,
    affected_paths: Vec<String>,
    path_preconditions: Vec<PathPrecondition>,
    backup_commit: String,
    restore_plan: RestorePlan,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RestorePlan {
    restore_ref_to: String,
    restore_checkout_to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PathPrecondition {
    path: String,
    expected_base_entry: String,
    candidate_entry: String,
}

/// Successful publication projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationResult {
    published_commit: String,
    final_tree_digest: String,
}

impl PublicationResult {
    /// Returns the commit installed by compare-and-swap.
    pub fn published_commit(&self) -> &str {
        &self.published_commit
    }
    /// Returns the verified final Git tree digest.
    pub fn final_tree_digest(&self) -> &str {
        &self.final_tree_digest
    }
}

impl PublicationTransaction {
    /// Creates and durably records a candidate commit without touching the authoritative checkout.
    pub fn prepare(
        authoritative: impl AsRef<Path>,
        session: impl AsRef<Path>,
        target_ref: &str,
        expected_base: &str,
    ) -> Result<Self, PublicationError> {
        validate_ref(target_ref)?;
        let authoritative = fs::canonicalize(authoritative)?;
        let session = fs::canonicalize(session)?;
        let common_directory =
            git_path(&authoritative, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
        let actual_session_base = git(&session, &["rev-parse", "HEAD"])?;
        if actual_session_base != expected_base {
            return Err(PublicationError::PublicationRebaseRequired);
        }
        git_status(&session, &["cat-file", "-e", &format!("{expected_base}^{{commit}}")])?;
        git_status(&session, &["add", "--all"])?;
        git_status(&session, &["-c", "commit.gpgsign=false", "commit", "-m", CANDIDATE_MESSAGE])?;
        let candidate_commit = git(&session, &["rev-parse", "HEAD"])?;
        let target_tree_digest = git(&session, &["rev-parse", "HEAD^{tree}"])?;
        let affected_paths = changed_paths(&session, expected_base, &candidate_commit)?;
        if affected_paths.is_empty() {
            return Err(PublicationError::EmptyCandidate);
        }
        let path_preconditions = affected_paths
            .iter()
            .map(|path| {
                Ok(PathPrecondition {
                    path: path.clone(),
                    expected_base_entry: tree_entry(&session, expected_base, path)?,
                    candidate_entry: tree_entry(&session, &candidate_commit, path)?,
                })
            })
            .collect::<Result<Vec<_>, PublicationError>>()?;
        let publication_id = uuid::Uuid::new_v4().to_string();
        let manifest = PublicationManifest {
            schema_version: PUBLICATION_SCHEMA_VERSION.to_owned(),
            publication_id,
            expected_base_revision: expected_base.to_owned(),
            target_ref: target_ref.to_owned(),
            candidate_commit,
            target_tree_digest,
            affected_paths,
            path_preconditions,
            backup_commit: expected_base.to_owned(),
            restore_plan: RestorePlan {
                restore_ref_to: expected_base.to_owned(),
                restore_checkout_to: expected_base.to_owned(),
            },
        };
        persist_manifest(&common_directory, &manifest)?;
        validate_persisted_manifest(&common_directory, &manifest)?;
        Ok(Self { authoritative, session, common_directory, manifest })
    }

    /// Returns the complete affected-path ledger persisted before publication.
    pub fn affected_paths(&self) -> &[String] {
        &self.manifest.affected_paths
    }

    /// Publishes under an exclusive repository lock and verifies the clean final checkout.
    pub fn publish(self) -> Result<PublicationResult, PublicationError> {
        let _lock = PublicationLock::acquire(&self.common_directory)?;
        self.validate_preconditions()?;
        git_status(
            &self.authoritative,
            &[
                "update-ref",
                &self.manifest.target_ref,
                &self.manifest.candidate_commit,
                &self.manifest.expected_base_revision,
            ],
        )
        .map_err(|_| PublicationError::PublicationRebaseRequired)?;
        self.replace_affected_paths()?;
        let published_index = git(&self.authoritative, &["write-tree"])?;
        if published_index != self.manifest.target_tree_digest {
            return Err(PublicationError::FinalFingerprintMismatch);
        }
        git_status(&self.authoritative, &["reset", "--hard", &self.manifest.candidate_commit])?;
        let final_commit = git(&self.authoritative, &["rev-parse", "HEAD"])?;
        let final_tree = git(&self.authoritative, &["rev-parse", "HEAD^{tree}"])?;
        let status = git(&self.authoritative, &["status", "--porcelain", "--untracked-files=all"])?;
        if final_commit != self.manifest.candidate_commit
            || final_tree != self.manifest.target_tree_digest
            || !status.is_empty()
        {
            return Err(PublicationError::FinalFingerprintMismatch);
        }
        let _session_commit = git(&self.session, &["rev-parse", "HEAD"])?;
        Ok(PublicationResult { published_commit: final_commit, final_tree_digest: final_tree })
    }

    fn replace_affected_paths(&self) -> Result<(), PublicationError> {
        for precondition in &self.manifest.path_preconditions {
            validate_path(&precondition.path)?;
            reject_symlink_ancestor(&self.authoritative, &precondition.path)?;
            let unchanged = git_quiet(
                &self.authoritative,
                &[
                    "diff",
                    "--quiet",
                    &self.manifest.expected_base_revision,
                    "--",
                    &precondition.path,
                ],
            )?;
            let untracked = git(
                &self.authoritative,
                &["ls-files", "--others", "--exclude-standard", "--", &precondition.path],
            )?;
            if !unchanged || !untracked.is_empty() {
                return Err(PublicationError::AuthoritativePreconditionFailed);
            }
            if precondition.candidate_entry.is_empty() {
                remove_candidate_path(&self.authoritative, &precondition.path)?;
            } else {
                git_status(
                    &self.authoritative,
                    &["checkout", &self.manifest.candidate_commit, "--", &precondition.path],
                )?;
            }
        }
        Ok(())
    }

    fn validate_preconditions(&self) -> Result<(), PublicationError> {
        let head = git(&self.authoritative, &["rev-parse", "HEAD"])?;
        let target = git(&self.authoritative, &["rev-parse", &self.manifest.target_ref])?;
        if head != self.manifest.expected_base_revision
            || target != self.manifest.expected_base_revision
        {
            return Err(PublicationError::PublicationRebaseRequired);
        }
        let status = git(&self.authoritative, &["status", "--porcelain", "--untracked-files=all"])?;
        if !status.is_empty() {
            return Err(PublicationError::AuthoritativePreconditionFailed);
        }
        for precondition in &self.manifest.path_preconditions {
            validate_path(&precondition.path)?;
            let unchanged = git_quiet(
                &self.authoritative,
                &[
                    "diff",
                    "--quiet",
                    &self.manifest.expected_base_revision,
                    "--",
                    &precondition.path,
                ],
            )?;
            if !unchanged {
                return Err(PublicationError::AuthoritativePreconditionFailed);
            }
        }
        Ok(())
    }
}

struct PublicationLock {
    path: PathBuf,
    _file: File,
}
impl PublicationLock {
    fn acquire(common: &Path) -> Result<Self, PublicationError> {
        let directory = common.join(STATE_DIRECTORY);
        fs::create_dir_all(&directory)?;
        let path = directory.join(LOCK_FILE);
        let mut file =
            OpenOptions::new().create_new(true).write(true).open(&path).map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    PublicationError::PublicationLocked
                } else {
                    error.into()
                }
            })?;
        file.write_all(b"publication-in-flight\n")?;
        file.sync_all()?;
        Ok(Self { path, _file: file })
    }
}
impl Drop for PublicationLock {
    fn drop(&mut self) {
        let _ignored = fs::remove_file(&self.path);
    }
}

/// Fail-closed publication errors.
#[derive(Debug, Error)]
pub enum PublicationError {
    #[error("publication I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("publication manifest encoding failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Git publication operation failed: {0}")]
    Git(String),
    #[error("publication_rebase_required")]
    PublicationRebaseRequired,
    #[error("authoritative path or index precondition failed")]
    AuthoritativePreconditionFailed,
    #[error("another publication owns the repository lock")]
    PublicationLocked,
    #[error("candidate has no product changes")]
    EmptyCandidate,
    #[error("publication path or ref is invalid")]
    InvalidName,
    #[error("final authoritative fingerprint does not match the candidate")]
    FinalFingerprintMismatch,
    #[error("persisted publication backup or restore plan failed validation")]
    ManifestValidationFailed,
}

fn persist_manifest(common: &Path, manifest: &PublicationManifest) -> Result<(), PublicationError> {
    let directory = common.join(STATE_DIRECTORY).join("plans");
    fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{}.json", manifest.publication_id));
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(&serde_json::to_vec_pretty(manifest)?)?;
    file.sync_all()?;
    File::open(directory)?.sync_all()?;
    Ok(())
}

fn validate_persisted_manifest(
    common: &Path,
    expected: &PublicationManifest,
) -> Result<(), PublicationError> {
    let path = common
        .join(STATE_DIRECTORY)
        .join("plans")
        .join(format!("{}.json", expected.publication_id));
    let persisted: PublicationManifest = serde_json::from_slice(&fs::read(path)?)?;
    if persisted.schema_version == PUBLICATION_SCHEMA_VERSION
        && persisted.expected_base_revision == persisted.backup_commit
        && persisted.expected_base_revision == persisted.restore_plan.restore_ref_to
        && persisted.expected_base_revision == persisted.restore_plan.restore_checkout_to
        && persisted.candidate_commit == expected.candidate_commit
        && persisted.target_tree_digest == expected.target_tree_digest
        && persisted.path_preconditions.len() == persisted.affected_paths.len()
    {
        Ok(())
    } else {
        Err(PublicationError::ManifestValidationFailed)
    }
}
fn changed_paths(
    root: &Path,
    base: &str,
    candidate: &str,
) -> Result<Vec<String>, PublicationError> {
    let range = format!("{base}..{candidate}");
    let output = git_bytes(root, &["diff", "--name-only", "-z", &range])?;
    output
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| {
            String::from_utf8(part.to_vec())
                .map_err(|error| PublicationError::Git(error.to_string()))
        })
        .collect()
}
fn tree_entry(root: &Path, revision: &str, path: &str) -> Result<String, PublicationError> {
    git(root, &["ls-tree", revision, "--", path])
}
fn validate_ref(value: &str) -> Result<(), PublicationError> {
    if value.starts_with("refs/heads/") && !value.contains("..") {
        Ok(())
    } else {
        Err(PublicationError::InvalidName)
    }
}
fn validate_path(value: &str) -> Result<(), PublicationError> {
    let path = Path::new(value);
    if !path.is_absolute()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(())
    } else {
        Err(PublicationError::InvalidName)
    }
}
fn reject_symlink_ancestor(root: &Path, value: &str) -> Result<(), PublicationError> {
    let components = Path::new(value).components().collect::<Vec<_>>();
    let mut current = root.to_path_buf();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        let Component::Normal(segment) = component else {
            return Err(PublicationError::InvalidName);
        };
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(PublicationError::AuthoritativePreconditionFailed);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
fn remove_candidate_path(root: &Path, value: &str) -> Result<(), PublicationError> {
    let target = root.join(value);
    match fs::symlink_metadata(&target) {
        Ok(_) => fs::remove_file(target)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    git_status(root, &["add", "-u", "--", value])
}
fn git_path(root: &Path, arguments: &[&str]) -> Result<PathBuf, PublicationError> {
    fs::canonicalize(PathBuf::from(git(root, arguments)?)).map_err(Into::into)
}
fn git(root: &Path, arguments: &[&str]) -> Result<String, PublicationError> {
    String::from_utf8(git_bytes(root, arguments)?)
        .map(|value| value.trim().to_owned())
        .map_err(|error| PublicationError::Git(error.to_string()))
}
fn git_status(root: &Path, arguments: &[&str]) -> Result<(), PublicationError> {
    git_bytes(root, arguments).map(|_| ())
}
fn git_quiet(root: &Path, arguments: &[&str]) -> Result<bool, PublicationError> {
    let status = Command::new("git").args(arguments).current_dir(root).status()?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(PublicationError::Git("Git precondition inspection failed".to_owned())),
    }
}
fn git_bytes(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, PublicationError> {
    let output = Command::new("git").args(arguments).current_dir(root).output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(PublicationError::Git(String::from_utf8_lossy(&output.stderr).trim().to_owned()))
    }
}

#[cfg(test)]
mod tests {
    //! Boundary tests exercise fail-closed helpers that typed public plans cannot forge.

    use std::path::Path;
    use std::process::Command;

    use super::{
        PUBLICATION_SCHEMA_VERSION, PathPrecondition, PublicationError, PublicationManifest,
        RestorePlan, git_quiet, persist_manifest, reject_symlink_ancestor, remove_candidate_path,
        validate_path, validate_persisted_manifest, validate_ref,
    };

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn require(condition: bool, message: &str) -> TestResult {
        if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
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
        std::fs::write(root.path().join("tracked.txt"), "base\n")?;
        git(root.path(), &["add", "tracked.txt"])?;
        git(root.path(), &["commit", "-m", "base"])?;
        Ok(root)
    }

    #[test]
    fn path_and_ref_validators_reject_escape_forms() -> TestResult {
        require(validate_ref("refs/heads/main").is_ok(), "canonical branch ref was rejected")?;
        require(
            matches!(validate_ref("refs/heads/../escape"), Err(PublicationError::InvalidName)),
            "ref traversal was admitted",
        )?;
        require(validate_path("nested/file.txt").is_ok(), "normalized product path was rejected")?;
        require(
            matches!(validate_path("../escape"), Err(PublicationError::InvalidName)),
            "path traversal was admitted",
        )
    }

    #[test]
    fn symlink_ancestor_and_non_normal_component_fail_closed() -> TestResult {
        let root = tempfile::tempdir()?;
        std::fs::create_dir(root.path().join("outside"))?;
        std::os::unix::fs::symlink("outside", root.path().join("link"))?;

        require(
            matches!(
                reject_symlink_ancestor(root.path(), "link/file.txt"),
                Err(PublicationError::AuthoritativePreconditionFailed)
            ),
            "publication followed a symlink ancestor",
        )?;
        require(
            matches!(
                reject_symlink_ancestor(root.path(), "../escape"),
                Err(PublicationError::InvalidName)
            ),
            "publication helper accepted a non-normal component",
        )?;
        reject_symlink_ancestor(root.path(), "missing/file.txt")?;
        Ok(())
    }

    #[test]
    fn absent_candidate_deletion_is_idempotent_and_preserves_index() -> TestResult {
        let root = repository()?;
        std::fs::remove_file(root.path().join("tracked.txt"))?;
        remove_candidate_path(root.path(), "tracked.txt")?;
        require(
            !git_quiet(root.path(), &["diff", "--quiet", "--cached"])?,
            "already-absent tracked deletion was not staged",
        )
    }

    #[test]
    fn tampered_durable_manifest_is_rejected() -> TestResult {
        let common = tempfile::tempdir()?;
        let manifest = PublicationManifest {
            schema_version: PUBLICATION_SCHEMA_VERSION.to_owned(),
            publication_id: "publication-1".to_owned(),
            expected_base_revision: "base".to_owned(),
            target_ref: "refs/heads/main".to_owned(),
            candidate_commit: "candidate".to_owned(),
            target_tree_digest: "tree".to_owned(),
            affected_paths: vec!["tracked.txt".to_owned()],
            path_preconditions: vec![PathPrecondition {
                path: "tracked.txt".to_owned(),
                expected_base_entry: "base-entry".to_owned(),
                candidate_entry: "candidate-entry".to_owned(),
            }],
            backup_commit: "base".to_owned(),
            restore_plan: RestorePlan {
                restore_ref_to: "base".to_owned(),
                restore_checkout_to: "base".to_owned(),
            },
        };
        persist_manifest(common.path(), &manifest)?;
        let path = common.path().join("boundline/publication/plans/publication-1.json");
        let mut tampered = manifest.clone();
        tampered.backup_commit = "different-base".to_owned();
        std::fs::write(path, serde_json::to_vec_pretty(&tampered)?)?;

        require(
            matches!(
                validate_persisted_manifest(common.path(), &manifest),
                Err(PublicationError::ManifestValidationFailed)
            ),
            "tampered backup manifest remained publishable",
        )
    }

    #[test]
    fn git_quiet_distinguishes_changes_from_inspection_failure() -> TestResult {
        let root = repository()?;
        std::fs::write(root.path().join("tracked.txt"), "changed\n")?;
        require(
            !git_quiet(root.path(), &["diff", "--quiet"])?,
            "dirty tracked content was reported unchanged",
        )?;
        require(
            matches!(
                git_quiet(root.path(), &["diff", "--definitely-invalid-option"]),
                Err(PublicationError::Git(_))
            ),
            "Git inspection failure was interpreted as a clean precondition",
        )
    }
}
