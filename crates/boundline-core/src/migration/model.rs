//! Typed state, reports, and stable failures for the Boundline bridge.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Exact legacy product version accepted by this bridge.
pub const SUPPORTED_SOURCE_VERSION: &str = "0.82.0";
/// Schema identity extracted from the immutable 0.82.0 source tag.
pub const SOURCE_SCHEMA_VERSION: &str = "boundline-workspace-state-0.82.0";
/// Target bridge schema written by M1D.
pub const TARGET_SCHEMA_VERSION: &str = "boundline-workspace-state-0.90";
/// Migration implementation identity used by idempotency.
pub const MIGRATION_IMPLEMENTATION_VERSION: &str = "boundline-bridge-090-v1";

/// A legacy state root and its explicitly admitted product version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationSource {
    /// Product-owned `.boundline` state root.
    pub root: PathBuf,
    /// Claimed source release, verified against the scaffold manifest.
    pub declared_version: String,
}

impl MigrationSource {
    /// Creates a source descriptor without touching the filesystem.
    pub fn new(root: impl Into<PathBuf>, declared_version: impl Into<String>) -> Self {
        Self { root: root.into(), declared_version: declared_version.into() }
    }
}

/// Stable bridge action selected by inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationAction {
    /// Every recognized record can remain in the active historical projection.
    Convert,
    /// At least one record must be removed from active loading and archived.
    ArchiveUnsupported,
}

/// One deterministic legacy inventory entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyStateItem {
    /// Logical identity with no host-local absolute path.
    pub identity: String,
    /// Historical record kind.
    pub kind: String,
    /// Historical lifecycle label.
    pub state: String,
    /// Whether ordinary 0.90 resume must exclude this item.
    pub archive_required: bool,
    /// Stable reason for archival, when required.
    pub archive_reason: Option<String>,
}

/// Non-mutating result of inspecting one exact source state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationInspection {
    /// Admitted source descriptor.
    pub source: MigrationSource,
    /// Cryptographic digest of the complete source tree.
    pub source_digest: String,
    /// Verified product version.
    pub source_version: String,
    /// Historical schema identity.
    pub source_schema_version: String,
    /// Deterministically ordered state inventory.
    pub items: Vec<LegacyStateItem>,
    /// Required bridge action.
    pub action: MigrationAction,
    /// Deterministic warnings discovered during inspection.
    pub warnings: Vec<String>,
}

/// Immutable plan bound to the exact bytes observed during inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    /// Deterministic migration identifier.
    pub migration_id: String,
    /// Source descriptor.
    pub source: MigrationSource,
    /// Source tree digest that apply must revalidate.
    pub source_digest: String,
    /// Source schema identity.
    pub source_schema_version: String,
    /// Target schema identity.
    pub target_schema_version: String,
    /// Migration implementation identity.
    pub implementation_version: String,
    /// Planned state action.
    pub action: MigrationAction,
    /// Planned deterministic inventory.
    pub items: Vec<LegacyStateItem>,
    /// Inspection warnings.
    pub warnings: Vec<String>,
}

/// Durable lifecycle boundaries exposed to fault-injection tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationBoundary {
    /// Ownership exists but backup effects have not begun.
    BeforeBackup,
    /// Backup bytes have been persisted.
    BackupCreated,
    /// Backup digest has been read back and verified.
    BackupVerified,
    /// Target staging has begun.
    DuringTargetStaging,
    /// Staged target has been validated.
    StagedTargetVerified,
    /// No source replacement has occurred yet.
    BeforeReplacement,
    /// Target replaced the source but completion is not durable.
    AfterReplacement,
    /// Completion is durable; lock release may remain.
    AfterCompletion,
}

impl MigrationBoundary {
    /// Returns the complete declared fault order.
    pub const fn all() -> &'static [Self] {
        &[
            Self::BeforeBackup,
            Self::BackupCreated,
            Self::BackupVerified,
            Self::DuringTargetStaging,
            Self::StagedTargetVerified,
            Self::BeforeReplacement,
            Self::AfterReplacement,
            Self::AfterCompletion,
        ]
    }

    /// Returns the stable journal label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeBackup => "before_backup",
            Self::BackupCreated => "after_backup_creation",
            Self::BackupVerified => "after_backup_verification",
            Self::DuringTargetStaging => "during_target_staging",
            Self::StagedTargetVerified => "after_staged_target_verification",
            Self::BeforeReplacement => "before_replacement",
            Self::AfterReplacement => "after_replacement_before_completion",
            Self::AfterCompletion => "after_completion_record",
        }
    }
}

/// Deterministic fault control used by recovery qualification.
#[derive(Debug, Clone, Copy, Default)]
pub struct MigrationControl {
    /// Boundary after which execution stops.
    pub stop_after: Option<MigrationBoundary>,
}

impl MigrationControl {
    /// Stops after the selected durable boundary.
    pub const fn stop_after(boundary: MigrationBoundary) -> Self {
        Self { stop_after: Some(boundary) }
    }
}

/// Stable terminal migration status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    /// Source is unchanged and a fresh retry is safe.
    SafeRetry,
    /// Migration committed and reopened successfully.
    Complete,
    /// The exact migration already completed.
    AlreadyMigrated,
    /// Durable state requires reconciliation.
    RecoveryRequired,
}

/// One deterministic verification result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalVerificationResult {
    /// Stable verification identifier.
    pub check: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Deterministic detail with no absolute path.
    pub detail: String,
}

/// One preserved semantic limitation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticLossRecord {
    /// Stable loss code.
    pub code: String,
    /// Logical record identity.
    pub item_identity: String,
    /// Human-readable bounded description.
    pub detail: String,
}

/// One state record excluded from ordinary 0.90 execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedStateRecord {
    /// Logical record identity.
    pub item_identity: String,
    /// Stable reason.
    pub reason_code: String,
    /// Whether the operator must start a newly admitted session.
    pub requires_new_admitted_session: bool,
}

/// Portable, typed conversion report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionReport {
    /// Deterministic migration identity.
    pub migration_id: String,
    /// Source product.
    pub source_product: String,
    /// Source release.
    pub source_version: String,
    /// Source schema.
    pub source_schema_version: String,
    /// Target release line.
    pub target_version_line: String,
    /// Target schema.
    pub target_schema_version: String,
    /// Cryptographic source identity.
    pub source_digest: String,
    /// Volatile inspection timestamp in milliseconds.
    pub inspection_timestamp_ms: u64,
    /// Volatile start timestamp in milliseconds.
    pub migration_start_timestamp_ms: u64,
    /// Volatile completion timestamp in milliseconds.
    pub migration_completion_timestamp_ms: Option<u64>,
    /// Logical backup identity.
    pub backup_identity: Option<String>,
    /// Verified backup digest.
    pub backup_digest: Option<String>,
    /// Logical staged-output identity.
    pub staged_output_identity: Option<String>,
    /// Verified staged-output digest.
    pub staged_output_digest: Option<String>,
    /// Converted item count.
    pub converted_item_count: usize,
    /// Archived item count.
    pub archived_item_count: usize,
    /// Skipped item count.
    pub skipped_item_count: usize,
    /// Deterministic warnings.
    pub warnings: Vec<String>,
    /// Deterministic semantic-loss records.
    pub semantic_losses: Vec<SemanticLossRecord>,
    /// Deterministic unsupported-state records.
    pub unsupported_states: Vec<UnsupportedStateRecord>,
    /// Deterministic verification results.
    pub verification_results: Vec<CanonicalVerificationResult>,
    /// Terminal lifecycle status.
    pub terminal_status: MigrationStatus,
    /// Stable terminal reason code.
    pub terminal_reason_code: String,
    /// Recovery instructions when reconciliation remains.
    pub recovery_instructions: Option<String>,
}

impl ConversionReport {
    /// Removes explicitly volatile values for semantic comparison.
    pub fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        normalized.inspection_timestamp_ms = 0;
        normalized.migration_start_timestamp_ms = 0;
        normalized.migration_completion_timestamp_ms =
            normalized.migration_completion_timestamp_ms.map(|_| 0);
        if normalized.archived_item_count > 0 {
            normalized.staged_output_digest =
                normalized.staged_output_digest.map(|_| "volatile-archive-digest".to_string());
        }
        normalized
    }
}

/// Result of apply or recovery.
#[derive(Debug, Clone)]
pub struct MigrationOutcome {
    /// Terminal or recovery status.
    pub status: MigrationStatus,
    /// Portable conversion report.
    pub report: ConversionReport,
    /// Internal backup location.
    pub backup_root: PathBuf,
    /// Verified backup digest.
    pub backup_digest: Option<String>,
}

/// Stable reason codes for migration failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationReasonCode {
    UnsupportedSourceVersion,
    UnsupportedFutureSchema,
    CorruptLegacyState,
    MixedLegacyState,
    SourceChanged,
    MigrationInProgress,
    BackupFailed,
    BackupVerificationFailed,
    StagingFailed,
    StagedVerificationFailed,
    ReplacementFailed,
    RecoveryRequired,
    ArchiveRequired,
    ArchiveVerificationFailed,
    InsufficientSpace,
    PermissionDenied,
    UnsafePath,
    UnsupportedFilesystem,
    IdempotencyConflict,
}

impl MigrationReasonCode {
    /// Returns the frozen wire value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSourceVersion => "unsupported_source_version",
            Self::UnsupportedFutureSchema => "unsupported_future_schema",
            Self::CorruptLegacyState => "corrupt_legacy_state",
            Self::MixedLegacyState => "mixed_legacy_state",
            Self::SourceChanged => "source_changed",
            Self::MigrationInProgress => "migration_in_progress",
            Self::BackupFailed => "backup_failed",
            Self::BackupVerificationFailed => "backup_verification_failed",
            Self::StagingFailed => "staging_failed",
            Self::StagedVerificationFailed => "staged_verification_failed",
            Self::ReplacementFailed => "replacement_failed",
            Self::RecoveryRequired => "recovery_required",
            Self::ArchiveRequired => "archive_required",
            Self::ArchiveVerificationFailed => "archive_verification_failed",
            Self::InsufficientSpace => "insufficient_space",
            Self::PermissionDenied => "permission_denied",
            Self::UnsafePath => "unsafe_path",
            Self::UnsupportedFilesystem => "unsupported_filesystem",
            Self::IdempotencyConflict => "idempotency_conflict",
        }
    }
}

impl std::fmt::Display for MigrationReasonCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Typed migration failure.
#[derive(Debug, Error)]
#[error("{reason}: {detail}")]
pub struct MigrationError {
    reason: MigrationReasonCode,
    detail: String,
}

impl MigrationError {
    /// Creates a typed failure without exposing host-local paths.
    pub(crate) fn new(reason: MigrationReasonCode, detail: impl Into<String>) -> Self {
        Self { reason, detail: detail.into() }
    }

    /// Returns the stable reason code.
    pub const fn reason_code(&self) -> MigrationReasonCode {
        self.reason
    }
}
