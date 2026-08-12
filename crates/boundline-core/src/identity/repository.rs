//! Clone-local identity uses Git's common directory as the shared lock domain.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const REPOSITORY_SCHEMA_VERSION: &str = "boundline-repository-identity-v1";
const REPOSITORY_STATE_DIRECTORY: &str = "boundline";
const REPOSITORY_RECORD_FILE: &str = "repository-identity.json";
const WORKTREE_METADATA_DIRECTORY: &str = ".boundline";
const WORKTREE_ROLE_FILE: &str = "worktree-role.json";
const COMMON_DIRECTORY_DIGEST_DOMAIN: &str = "boundline-git-common-directory-v1";

/// Opaque identity shared only by linked worktrees of one local clone.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepositoryId(String);

impl RepositoryId {
    /// Returns the portable opaque value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque identity of one checkout or linked worktree.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorktreeId(String);

impl WorktreeId {
    /// Returns the portable opaque value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Declared role of a Boundline-visible Git worktree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeRole {
    /// Clean checkout that may receive a verified publication.
    Authoritative,
    /// Persistent external checkout owned by one governed session.
    SessionManaged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RepositoryIdentityRecord {
    schema_version: String,
    repository_id: RepositoryId,
    git_common_directory_fingerprint: String,
    authoritative_worktree_id: Option<WorktreeId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WorktreeRoleRecord {
    schema_version: String,
    repository_id: RepositoryId,
    worktree_id: WorktreeId,
    role: WorktreeRole,
}

/// Validated clone and worktree identity projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryIdentityStore {
    repository_id: RepositoryId,
    worktree_id: WorktreeId,
    role: WorktreeRole,
}

impl RepositoryIdentityStore {
    /// Opens or creates identity records and rejects role or clone confusion.
    pub fn open(
        worktree: impl AsRef<Path>,
        role: WorktreeRole,
    ) -> Result<Self, RepositoryIdentityError> {
        let worktree = canonical_directory(worktree.as_ref())?;
        let common_directory = git_common_directory(&worktree)?;
        let repository_path =
            common_directory.join(REPOSITORY_STATE_DIRECTORY).join(REPOSITORY_RECORD_FILE);
        let mut repository = load_or_create_repository(&repository_path)?;
        let role_path = worktree.join(WORKTREE_METADATA_DIRECTORY).join(WORKTREE_ROLE_FILE);
        let marker = load_or_create_worktree_marker(&role_path, &repository.repository_id, role)?;
        validate_marker(&marker, &repository.repository_id, role)?;
        bind_authoritative_worktree(&repository_path, &mut repository, &marker)?;
        Ok(Self {
            repository_id: repository.repository_id,
            worktree_id: marker.worktree_id,
            role: marker.role,
        })
    }

    /// Returns the clone-local lock-domain identity.
    pub fn repository_id(&self) -> &RepositoryId {
        &self.repository_id
    }

    /// Returns the identity of this exact checkout.
    pub fn worktree_id(&self) -> &WorktreeId {
        &self.worktree_id
    }

    /// Returns the validated checkout role.
    pub const fn role(&self) -> WorktreeRole {
        self.role
    }
}

/// Fail-closed repository identity errors.
#[derive(Debug, Error)]
pub enum RepositoryIdentityError {
    /// Filesystem persistence or canonicalization failed.
    #[error("repository identity I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// A persisted typed record could not be decoded.
    #[error("repository identity record is invalid: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The supplied path is not a usable Git worktree.
    #[error("Git common directory discovery failed: {0}")]
    GitDiscovery(String),
    /// A worktree marker belongs to another local clone.
    #[error("worktree marker repository identity does not match the Git common directory")]
    RepositoryMismatch,
    /// A worktree was reopened under a different authority role.
    #[error("worktree role mismatch: expected {expected:?}, recorded {recorded:?}")]
    WorktreeRoleMismatch {
        /// Requested role.
        expected: WorktreeRole,
        /// Durable marker role.
        recorded: WorktreeRole,
    },
    /// Another checkout is already bound as the authoritative worktree.
    #[error("a different authoritative worktree is already bound to this repository")]
    AuthoritativeWorktreeMismatch,
    /// A record uses an unsupported schema.
    #[error("repository identity schema is unsupported")]
    UnsupportedSchema,
}

fn canonical_directory(path: &Path) -> Result<PathBuf, RepositoryIdentityError> {
    let canonical = fs::canonicalize(path)?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(RepositoryIdentityError::Io(std::io::Error::other("worktree is not a directory")))
    }
}

fn git_common_directory(worktree: &Path) -> Result<PathBuf, RepositoryIdentityError> {
    let output = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(worktree)
        .output()?;
    if !output.status.success() {
        return Err(RepositoryIdentityError::GitDiscovery(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let value = String::from_utf8(output.stdout)
        .map_err(|error| RepositoryIdentityError::GitDiscovery(error.to_string()))?;
    canonical_directory(Path::new(value.trim()))
}

fn load_or_create_repository(
    path: &Path,
) -> Result<RepositoryIdentityRecord, RepositoryIdentityError> {
    if path.is_file() {
        let record: RepositoryIdentityRecord = serde_json::from_slice(&fs::read(path)?)?;
        validate_schema(&record.schema_version)?;
        return Ok(record);
    }
    let repository_id = RepositoryId(uuid::Uuid::new_v4().to_string());
    let fingerprint = fingerprint_common_directory(&repository_id);
    let record = RepositoryIdentityRecord {
        schema_version: REPOSITORY_SCHEMA_VERSION.to_owned(),
        repository_id,
        git_common_directory_fingerprint: fingerprint,
        authoritative_worktree_id: None,
    };
    write_typed_record(path, &record)?;
    Ok(record)
}

fn load_or_create_worktree_marker(
    path: &Path,
    repository_id: &RepositoryId,
    role: WorktreeRole,
) -> Result<WorktreeRoleRecord, RepositoryIdentityError> {
    if path.is_file() {
        let record: WorktreeRoleRecord = serde_json::from_slice(&fs::read(path)?)?;
        validate_schema(&record.schema_version)?;
        return Ok(record);
    }
    let record = WorktreeRoleRecord {
        schema_version: REPOSITORY_SCHEMA_VERSION.to_owned(),
        repository_id: repository_id.clone(),
        worktree_id: WorktreeId(uuid::Uuid::new_v4().to_string()),
        role,
    };
    write_typed_record(path, &record)?;
    Ok(record)
}

fn validate_marker(
    marker: &WorktreeRoleRecord,
    repository_id: &RepositoryId,
    role: WorktreeRole,
) -> Result<(), RepositoryIdentityError> {
    if marker.repository_id != *repository_id {
        return Err(RepositoryIdentityError::RepositoryMismatch);
    }
    if marker.role != role {
        return Err(RepositoryIdentityError::WorktreeRoleMismatch {
            expected: role,
            recorded: marker.role,
        });
    }
    Ok(())
}

fn bind_authoritative_worktree(
    path: &Path,
    repository: &mut RepositoryIdentityRecord,
    marker: &WorktreeRoleRecord,
) -> Result<(), RepositoryIdentityError> {
    if marker.role != WorktreeRole::Authoritative {
        return Ok(());
    }
    match repository.authoritative_worktree_id.as_ref() {
        Some(identity) if identity != &marker.worktree_id => {
            Err(RepositoryIdentityError::AuthoritativeWorktreeMismatch)
        }
        Some(_) => Ok(()),
        None => {
            repository.authoritative_worktree_id = Some(marker.worktree_id.clone());
            write_typed_record(path, repository)
        }
    }
}

fn fingerprint_common_directory(repository_id: &RepositoryId) -> String {
    let mut digest = Sha256::new();
    digest.update(COMMON_DIRECTORY_DIGEST_DOMAIN.as_bytes());
    digest.update([0]);
    digest.update(repository_id.as_str().as_bytes());
    format!("sha256:{:x}", digest.finalize())
}

fn validate_schema(schema: &str) -> Result<(), RepositoryIdentityError> {
    if schema == REPOSITORY_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(RepositoryIdentityError::UnsupportedSchema)
    }
}

fn write_typed_record<T: Serialize>(
    path: &Path,
    record: &T,
) -> Result<(), RepositoryIdentityError> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("identity parent missing"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let mut file = File::create(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(record)?)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
