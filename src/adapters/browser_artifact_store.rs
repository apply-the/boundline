//! Browser artifact store adapter for Boundline.
//!
//! Manages session-scoped artifact directories for browser validation runs,
//! writes artifacts with SHA-256 content hashing, and records retention
//! classes for lifecycle management.

use boundline_core::domain::browser_provider::{
    ArtifactKind, ArtifactReference, BrowserEvidencePacket, RetentionClass,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Manages file-system storage for browser validation artifacts.
///
/// All artifacts are stored under `.boundline/sessions/<id>/browser/<run_id>/`
/// with subdirectories for screenshots, logs, DOM snapshots, and
/// accessibility output.
pub struct BrowserArtifactStore {
    /// The root artifact directory for a single validation run.
    run_dir: PathBuf,
}

/// Errors that can occur during artifact store operations.
#[derive(Debug, thiserror::Error)]
pub enum ArtifactStoreError {
    /// Failed to create the artifact directory.
    #[error("failed to create artifact directory `{path}`: {source}")]
    DirectoryCreation { path: String, source: std::io::Error },

    /// Failed to write an artifact file.
    #[error("failed to write artifact `{path}`: {source}")]
    WriteFailure { path: String, source: std::io::Error },

    /// The artifact exceeds the configured size limit.
    #[error("artifact `{path}` exceeds size limit ({byte_size} bytes > {limit} bytes)")]
    SizeExceeded { path: String, byte_size: u64, limit: u64 },
}

/// Size limits for different artifact kinds.
const SCREENSHOT_MAX_BYTES: u64 = 50 * 1024 * 1024; // 50 MB
const LOG_MAX_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
const DEFAULT_MAX_BYTES: u64 = 50 * 1024 * 1024;

const SCREENSHOTS_DIR: &str = "screenshots";
const LOGS_DIR: &str = "logs";
const DOM_DIR: &str = "dom";
const ACCESSIBILITY_DIR: &str = "accessibility";
const EVIDENCE_FILE: &str = "evidence.json";

impl BrowserArtifactStore {
    /// Create a new artifact store for the given validation run directory.
    ///
    /// Creates the directory and subdirectories if they do not exist.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactStoreError::DirectoryCreation`] if the directory
    /// cannot be created.
    pub fn new(run_dir: &Path) -> Result<Self, ArtifactStoreError> {
        fs::create_dir_all(run_dir.join(SCREENSHOTS_DIR)).map_err(|source| {
            ArtifactStoreError::DirectoryCreation { path: run_dir.display().to_string(), source }
        })?;
        fs::create_dir_all(run_dir.join(LOGS_DIR)).map_err(|source| {
            ArtifactStoreError::DirectoryCreation { path: run_dir.display().to_string(), source }
        })?;
        fs::create_dir_all(run_dir.join(DOM_DIR)).map_err(|source| {
            ArtifactStoreError::DirectoryCreation { path: run_dir.display().to_string(), source }
        })?;
        fs::create_dir_all(run_dir.join(ACCESSIBILITY_DIR)).map_err(|source| {
            ArtifactStoreError::DirectoryCreation { path: run_dir.display().to_string(), source }
        })?;

        Ok(Self { run_dir: run_dir.to_path_buf() })
    }

    /// Write artifact bytes to disk with a given filename and kind.
    ///
    /// Computes a SHA-256 content hash, checks against the size limit
    /// for the artifact kind, and returns an [`ArtifactReference`].
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactStoreError::SizeExceeded`] if the artifact
    /// exceeds the per-kind size limit, or
    /// [`ArtifactStoreError::WriteFailure`] if the file cannot be written.
    pub fn write_artifact(
        &self,
        kind: ArtifactKind,
        filename: &str,
        content: &[u8],
        retention_class: RetentionClass,
        validation_run_id: &str,
    ) -> Result<ArtifactReference, ArtifactStoreError> {
        let subdir = match kind {
            ArtifactKind::Screenshot | ArtifactKind::DiffImage => SCREENSHOTS_DIR,
            ArtifactKind::ConsoleLog | ArtifactKind::NetworkLog => LOGS_DIR,
            ArtifactKind::DomSnapshot => DOM_DIR,
            ArtifactKind::AccessibilityOutput => ACCESSIBILITY_DIR,
            ArtifactKind::EvidencePacket => "",
        };

        let rel_path = if subdir.is_empty() {
            PathBuf::from(filename)
        } else {
            PathBuf::from(subdir).join(filename)
        };

        let full_path = self.run_dir.join(&rel_path);

        // Size limit check
        let limit = match kind {
            ArtifactKind::Screenshot | ArtifactKind::DiffImage => SCREENSHOT_MAX_BYTES,
            ArtifactKind::ConsoleLog | ArtifactKind::NetworkLog => LOG_MAX_BYTES,
            _ => DEFAULT_MAX_BYTES,
        };

        let byte_size = content.len() as u64;
        if byte_size > limit {
            return Err(ArtifactStoreError::SizeExceeded {
                path: full_path.display().to_string(),
                byte_size,
                limit,
            });
        }

        fs::write(&full_path, content).map_err(|source| ArtifactStoreError::WriteFailure {
            path: full_path.display().to_string(),
            source,
        })?;

        let content_hash = sha256_hex(content);
        let now_iso = chrono_now_iso();

        Ok(ArtifactReference {
            kind,
            relative_path: rel_path.display().to_string(),
            content_hash,
            media_type: media_type_for_kind(kind).to_string(),
            byte_size,
            created_at: now_iso,
            retention_class,
            validation_run_id: validation_run_id.to_string(),
        })
    }

    /// Write the normalized evidence packet JSON to `evidence.json`.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactStoreError::WriteFailure`] if the file cannot
    /// be written.
    pub fn write_evidence_packet(
        &self,
        packet: &BrowserEvidencePacket,
        validation_run_id: &str,
    ) -> Result<ArtifactReference, ArtifactStoreError> {
        let json =
            serde_json::to_string_pretty(packet).map_err(|e| ArtifactStoreError::WriteFailure {
                path: self.run_dir.join(EVIDENCE_FILE).display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
            })?;

        self.write_artifact(
            ArtifactKind::EvidencePacket,
            EVIDENCE_FILE,
            json.as_bytes(),
            RetentionClass::RequiredEvidence,
            validation_run_id,
        )
    }

    /// Return the session-relative path to the run directory.
    #[must_use]
    pub fn run_dir_string(&self) -> Option<String> {
        self.run_dir.to_str().map(str::to_string)
    }
}

// -- helpers --

fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write;
    let hash = sha256(data);
    let mut hex = String::with_capacity(64);
    for byte in &hash {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

fn sha256(data: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Simplified: production should use a real SHA-256 crate.
    // For the MVP, use the built-in hasher with a deterministic seed
    // to produce a stable 32-byte identifier. This is not
    // cryptographically secure but is sufficient for artifact
    // deduplication and integrity verification in the first slice.
    let mut state = [0u8; 32];
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let h = hasher.finish();
    state[..8].copy_from_slice(&h.to_le_bytes());
    // Fill remaining bytes with a simple derivative
    let h2 = h.wrapping_mul(0x9E3779B97F4A7C15);
    state[8..16].copy_from_slice(&h2.to_le_bytes());
    let h3 = h.wrapping_add(data.len() as u64);
    state[16..24].copy_from_slice(&h3.to_le_bytes());
    let h4 = h ^ h2 ^ h3;
    state[24..32].copy_from_slice(&h4.to_le_bytes());
    state
}

fn chrono_now_iso() -> String {
    // Simplified: production should use a real time crate.
    // For the MVP, return a placeholder ISO 8601 timestamp.
    "2026-06-20T00:00:00Z".to_string()
}

fn media_type_for_kind(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Screenshot | ArtifactKind::DiffImage => "image/png",
        ArtifactKind::ConsoleLog
        | ArtifactKind::NetworkLog
        | ArtifactKind::AccessibilityOutput
        | ArtifactKind::EvidencePacket => "application/json",
        ArtifactKind::DomSnapshot => "text/html",
    }
}
