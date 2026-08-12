//! Versioned product fingerprints bind proof and publication to exact Git state.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

const FINGERPRINT_SCHEMA_VERSION: &str = "boundline-product-fingerprint-v1";
const DOMAIN_SEPARATOR: &str = "boundline-product-fingerprint\0v1\0";
const GIT_DIRECTORY: &str = ".git";
const BOUNDLINE_DIRECTORY: &str = ".boundline";
const TARGET_DIRECTORY: &str = "target";

/// Explicit policy for product-relevant untracked paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FingerprintPolicy {
    admitted_untracked_paths: BTreeSet<String>,
}

impl FingerprintPolicy {
    /// Creates a policy whose listed untracked paths participate in the digest.
    pub fn new<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self { admitted_untracked_paths: paths.into_iter().map(Into::into).collect() }
    }
}

/// Versioned digest of product-relevant repository state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductFingerprint {
    digest: String,
}

impl ProductFingerprint {
    /// Captures HEAD, index, tracked worktree state, and admitted untracked paths.
    pub fn capture(
        root: impl AsRef<Path>,
        policy: &FingerprintPolicy,
    ) -> Result<Self, FingerprintError> {
        let root = fs::canonicalize(root)?;
        let paths = product_paths(&root, policy)?;
        reject_collisions(&paths)?;
        let record = FingerprintRecord {
            schema_version: FINGERPRINT_SCHEMA_VERSION,
            head: git_output(&root, &["rev-parse", "HEAD"])?,
            base_revision: git_output(&root, &["merge-base", "HEAD", "HEAD"])?,
            index: git_output_bytes(&root, &["ls-files", "--stage", "-z"])?,
            status: git_output_bytes(
                &root,
                &["status", "--porcelain=v2", "-z", "--untracked-files=no"],
            )?,
            entries: paths
                .into_iter()
                .map(|path| capture_entry(&root, path))
                .collect::<Result<Vec<_>, _>>()?,
            exclusions: vec![GIT_DIRECTORY, BOUNDLINE_DIRECTORY, TARGET_DIRECTORY],
        };
        let encoded = serde_json::to_vec(&record)?;
        let mut hasher = Sha256::new();
        hasher.update(DOMAIN_SEPARATOR.as_bytes());
        hasher.update(encoded);
        Ok(Self { digest: format!("sha256:{:x}", hasher.finalize()) })
    }

    /// Returns the frozen fingerprint schema identifier.
    pub const fn schema_version() -> &'static str {
        FINGERPRINT_SCHEMA_VERSION
    }

    /// Returns the cryptographic digest projection.
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Serialize)]
struct FingerprintRecord<'a> {
    schema_version: &'a str,
    head: String,
    base_revision: String,
    index: Vec<u8>,
    status: Vec<u8>,
    entries: Vec<FileEntry>,
    exclusions: Vec<&'a str>,
}

#[derive(Serialize)]
struct FileEntry {
    path: String,
    kind: FileKind,
    mode: u32,
    content: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum FileKind {
    Regular,
    Symlink,
    Missing,
}

/// Fail-closed fingerprint construction errors.
#[derive(Debug, Error)]
pub enum FingerprintError {
    /// Filesystem inspection failed.
    #[error("fingerprint I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Stable typed encoding failed.
    #[error("fingerprint encoding failed: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Git state could not be inspected.
    #[error("fingerprint Git inspection failed: {0}")]
    Git(String),
    /// Two paths collide under the portable normalized comparison policy.
    #[error("product paths collide after normalization: {first} and {second}")]
    PathCollision { first: String, second: String },
    /// An admitted path escaped the repository.
    #[error("product path is not normalized and repository-relative: {0}")]
    InvalidPath(String),
}

fn product_paths(
    root: &Path,
    policy: &FingerprintPolicy,
) -> Result<BTreeSet<String>, FingerprintError> {
    let tracked = git_output_bytes(root, &["ls-files", "-z"])?;
    let mut paths = nul_paths(&tracked)?;
    for path in &policy.admitted_untracked_paths {
        validate_relative_path(path)?;
        paths.insert(path.clone());
    }
    Ok(paths)
}

fn nul_paths(bytes: &[u8]) -> Result<BTreeSet<String>, FingerprintError> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| {
            String::from_utf8(part.to_vec())
                .map_err(|error| FingerprintError::Git(error.to_string()))
        })
        .collect()
}

fn validate_relative_path(value: &str) -> Result<(), FingerprintError> {
    let path = Path::new(value);
    let valid = !path.is_absolute()
        && path.components().all(|component| matches!(component, Component::Normal(_)));
    if valid { Ok(()) } else { Err(FingerprintError::InvalidPath(value.to_owned())) }
}

fn reject_collisions(paths: &BTreeSet<String>) -> Result<(), FingerprintError> {
    let mut normalized = BTreeMap::new();
    for path in paths {
        let key = path.nfc().flat_map(char::to_lowercase).collect::<String>();
        if let Some(first) = normalized.insert(key, path.clone()) {
            return Err(FingerprintError::PathCollision { first, second: path.clone() });
        }
    }
    Ok(())
}

fn capture_entry(root: &Path, path: String) -> Result<FileEntry, FingerprintError> {
    let absolute = root.join(PathBuf::from(&path));
    match fs::symlink_metadata(&absolute) {
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(FileEntry {
            path,
            kind: FileKind::Symlink,
            mode: metadata.permissions().mode(),
            content: fs::read_link(absolute)?.as_os_str().as_encoded_bytes().to_vec(),
        }),
        Ok(metadata) if metadata.is_file() => Ok(FileEntry {
            path,
            kind: FileKind::Regular,
            mode: metadata.permissions().mode(),
            content: fs::read(absolute)?,
        }),
        Ok(_) => Err(FingerprintError::Io(std::io::Error::other("unsupported product file type"))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(FileEntry { path, kind: FileKind::Missing, mode: 0, content: Vec::new() })
        }
        Err(error) => Err(error.into()),
    }
}

fn git_output(root: &Path, arguments: &[&str]) -> Result<String, FingerprintError> {
    let bytes = git_output_bytes(root, arguments)?;
    String::from_utf8(bytes)
        .map(|value| value.trim().to_owned())
        .map_err(|error| FingerprintError::Git(error.to_string()))
}

fn git_output_bytes(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, FingerprintError> {
    let output = Command::new("git").args(arguments).current_dir(root).output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(FingerprintError::Git(String::from_utf8_lossy(&output.stderr).trim().to_owned()))
    }
}
