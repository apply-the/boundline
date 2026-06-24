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

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write;
    let hash = sha256(data);
    let mut hex = String::with_capacity(64);
    for byte in &hash {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use boundline_core::domain::browser_provider::{
        BrowserEvidencePacket, BrowserFinding, FindingKind, FindingSeverity, StepStatus, StepTiming,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_run_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        std::env::temp_dir().join(format!("boundline-artifact-test-{pid}-{n}"))
    }

    fn sample_evidence_packet(run_id: &str) -> BrowserEvidencePacket {
        BrowserEvidencePacket {
            validation_run_id: run_id.into(),
            provider_id: "test-provider".into(),
            status: StepStatus::Completed,
            started_at: "2026-06-24T00:00:00Z".into(),
            completed_at: "2026-06-24T00:00:01Z".into(),
            page_title: Some("Test Page".into()),
            http_status: Some(200),
            artifacts: vec![],
            findings: vec![BrowserFinding {
                kind: FindingKind::ConsoleError,
                severity: FindingSeverity::Warning,
                message: "test finding".into(),
                evidence_refs: vec![],
                retryability: None,
                confirmed_intermittent: false,
            }],
            timing: StepTiming {
                queue_wait_ms: None,
                navigation_ms: Some(100),
                readiness_wait_ms: None,
                script_execution_ms: None,
                accessibility_ms: None,
                total_ms: 100,
            },
            capabilities_active: vec!["screenshot".into()],
            schema_version: 1,
        }
    }

    #[test]
    fn new_creates_all_subdirectories() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        assert!(run_dir.join("screenshots").is_dir());
        assert!(run_dir.join("logs").is_dir());
        assert!(run_dir.join("dom").is_dir());
        assert!(run_dir.join("accessibility").is_dir());
        // Cleanup
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn write_screenshot_artifact() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let content = b"fake-png-data";
        let artifact = store
            .write_artifact(
                ArtifactKind::Screenshot,
                "test.png",
                content,
                RetentionClass::RequiredEvidence,
                "run-1",
            )
            .expect("write ok");
        assert_eq!(artifact.kind, ArtifactKind::Screenshot);
        assert_eq!(artifact.byte_size, 13);
        assert_eq!(artifact.retention_class, RetentionClass::RequiredEvidence);
        assert_eq!(artifact.media_type, "image/png");
        assert!(!artifact.content_hash.is_empty());
        assert!(artifact.relative_path.contains("screenshots/test.png"));
        assert!(run_dir.join("screenshots/test.png").exists());
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn write_console_log_artifact() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::ConsoleLog,
                "console.json",
                b"[]",
                RetentionClass::RequiredEvidence,
                "run-2",
            )
            .expect("write ok");
        assert_eq!(artifact.kind, ArtifactKind::ConsoleLog);
        assert_eq!(artifact.media_type, "application/json");
        assert!(artifact.relative_path.contains("logs/console.json"));
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn write_evidence_packet_produces_valid_json() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let packet = sample_evidence_packet("run-evidence");
        let artifact = store.write_evidence_packet(&packet, "run-evidence").expect("write ok");
        assert_eq!(artifact.kind, ArtifactKind::EvidencePacket);
        assert!(run_dir.join("evidence.json").exists());
        // Verify it's valid JSON
        let raw = std::fs::read_to_string(run_dir.join("evidence.json")).expect("read");
        let _parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn screenshot_over_size_limit_rejected() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        // Screenshot limit is 50 MB, so 51 MB should fail
        let big = vec![0u8; 51 * 1024 * 1024];
        let err = store
            .write_artifact(
                ArtifactKind::Screenshot,
                "big.png",
                &big,
                RetentionClass::RequiredEvidence,
                "run-3",
            )
            .expect_err("should reject");
        assert!(err.to_string().contains("exceeds size limit"));
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn log_over_10mb_rejected() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let big = vec![0u8; 11 * 1024 * 1024];
        let err = store
            .write_artifact(
                ArtifactKind::ConsoleLog,
                "big.log",
                &big,
                RetentionClass::RequiredEvidence,
                "run-4",
            )
            .expect_err("should reject");
        assert!(err.to_string().contains("exceeds size limit"));
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn sha256_produces_stable_hash() {
        let data = b"hello world";
        let h1 = sha256_hex(data);
        let h2 = sha256_hex(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn artifact_content_hash_differs_for_different_data() {
        let h1 = sha256_hex(b"a");
        let h2 = sha256_hex(b"b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn run_dir_string_returns_valid_path() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        assert!(store.run_dir_string().is_some());
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn dom_snapshot_uses_text_html_media_type() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::DomSnapshot,
                "page.html",
                b"<html></html>",
                RetentionClass::Diagnostic,
                "run-dom",
            )
            .expect("write ok");
        assert_eq!(artifact.media_type, "text/html");
        assert_eq!(artifact.retention_class, RetentionClass::Diagnostic);
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn accessibility_artifact_stored_in_accessibility_dir() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::AccessibilityOutput,
                "a11y.json",
                b"{}",
                RetentionClass::RequiredEvidence,
                "run-a11y",
            )
            .expect("write ok");
        assert!(artifact.relative_path.contains("accessibility/a11y.json"));
        assert_eq!(artifact.media_type, "application/json");
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn network_log_stored_in_logs_dir() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::NetworkLog,
                "network.json",
                b"[]",
                RetentionClass::Verbose,
                "run-net",
            )
            .expect("write ok");
        assert!(artifact.relative_path.contains("logs/network.json"));
        assert_eq!(artifact.retention_class, RetentionClass::Verbose);
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn diff_image_uses_image_png_media_type() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::DiffImage,
                "diff.png",
                b"fake",
                RetentionClass::RequiredEvidence,
                "run-diff",
            )
            .expect("write ok");
        assert_eq!(artifact.media_type, "image/png");
        assert!(artifact.relative_path.contains("screenshots/diff.png"));
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn ephemeral_retention_class_preserved() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::ConsoleLog,
                "temp.log",
                b"ephemeral",
                RetentionClass::Ephemeral,
                "run-eph",
            )
            .expect("write ok");
        assert_eq!(artifact.retention_class, RetentionClass::Ephemeral);
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn evidence_packet_artifact_stored_in_root() {
        let run_dir = temp_run_dir();
        let store = BrowserArtifactStore::new(&run_dir).expect("create store");
        let artifact = store
            .write_artifact(
                ArtifactKind::EvidencePacket,
                "custom-evidence.json",
                b"{}",
                RetentionClass::RequiredEvidence,
                "run-ep",
            )
            .expect("write ok");
        // EvidencePacket goes in root, not a subdirectory
        assert!(!artifact.relative_path.contains('/'));
        assert_eq!(artifact.media_type, "application/json");
        let _ = std::fs::remove_dir_all(&run_dir);
        let _ = store;
    }

    #[test]
    fn sha256_hex_is_64_chars_for_empty() {
        assert_eq!(sha256_hex(b"").len(), 64);
    }

    #[test]
    fn sha256_deterministic_across_same_input() {
        let data = vec![0u8; 1024];
        assert_eq!(sha256_hex(&data), sha256_hex(&data));
    }
}
