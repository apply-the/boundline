//! Filesystem transaction engine for the Boundline 0.82.0 bridge.

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod storage;

use storage::*;

use super::model::{
    CanonicalVerificationResult, ConversionReport, LegacyStateItem,
    MIGRATION_IMPLEMENTATION_VERSION, MigrationAction, MigrationBoundary, MigrationControl,
    MigrationError, MigrationInspection, MigrationOutcome, MigrationPlan, MigrationReasonCode,
    MigrationSource, MigrationStatus, SOURCE_SCHEMA_VERSION, SUPPORTED_SOURCE_VERSION,
    SemanticLossRecord, TARGET_SCHEMA_VERSION, UnsupportedStateRecord,
};

const PRODUCT_STATE_NAME: &str = ".boundline";
const MIGRATIONS_NAME: &str = ".boundline-migrations";
const TARGET_VERSION_LINE: &str = "0.90";
const MARKER_NAME: &str = "scaffold-manifest.json";
const TARGET_MARKER_NAME: &str = "schema-version.json";
const JOURNAL_NAME: &str = "journal.json";
const LOCK_NAME: &str = "owner.lock";
const OUTCOME_NAME: &str = "outcome.json";
const BACKUP_NAME: &str = "backup";
const BACKUP_MANIFEST_NAME: &str = "backup-manifest.json";
const STAGING_NAME: &str = "staging";
const RETAINED_NAME: &str = "retained-source";
const REPORT_NAME: &str = "migration-report.json";

#[derive(Debug, Deserialize)]
struct LegacyScaffoldMarker {
    version: u32,
    boundline_version: String,
}

#[derive(Debug, Deserialize)]
struct LegacySessionHeader {
    session_id: String,
    latest_status: String,
    #[serde(default)]
    active_execution_run_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TargetSchemaMarker<'a> {
    source_product: &'a str,
    source_version: &'a str,
    source_schema_version: &'a str,
    target_version_line: &'a str,
    target_schema_version: &'a str,
    migration_id: &'a str,
    source_digest: &'a str,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct BackupManifest {
    migration_id: String,
    source_product: String,
    source_version: String,
    source_schema_version: String,
    source_digest: String,
    backup_digest: String,
}

#[derive(Debug, Serialize)]
struct ArchiveManifest<'a> {
    migration_id: &'a str,
    source_version: &'a str,
    source_schema_version: &'a str,
    source_digest: &'a str,
    archive_digest: &'a str,
    item_identity: &'a str,
    reason: &'a str,
    requires_new_admitted_session: bool,
    creation_timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum JournalPhase {
    Owned,
    BackupCreated,
    BackupVerified,
    Staging,
    StagedVerified,
    ReplacementReady,
    Replaced,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
struct MigrationJournal {
    migration_id: String,
    source_digest: String,
    source_schema_version: String,
    action: MigrationAction,
    items: Vec<LegacyStateItem>,
    warnings: Vec<String>,
    phase: JournalPhase,
    backup_digest: Option<String>,
    staged_digest: Option<String>,
    report: ConversionReport,
}

/// Inspects the exact historical state without creating migration artifacts.
pub fn inspect_legacy_state(
    source: &MigrationSource,
) -> Result<MigrationInspection, MigrationError> {
    validate_source_root(source)?;
    let source_digest = digest_tree(&source.root)?;
    let marker = read_json::<LegacyScaffoldMarker>(&source.root.join(MARKER_NAME))?;
    if marker.version != 1 {
        return Err(MigrationError::new(
            MigrationReasonCode::MixedLegacyState,
            "legacy scaffold schema is not the frozen version",
        ));
    }
    validate_source_version(source, &marker.boundline_version)?;
    reject_target_markers(&source.root)?;
    let mut items = inspect_sessions(&source.root)?;
    items.sort_by(|left, right| left.identity.cmp(&right.identity));
    let action = if items.iter().any(|item| item.archive_required) {
        MigrationAction::ArchiveUnsupported
    } else {
        MigrationAction::Convert
    };
    let warnings = if action == MigrationAction::ArchiveUnsupported {
        vec!["unsupported active legacy state requires read-only archival".to_string()]
    } else {
        Vec::new()
    };
    Ok(MigrationInspection {
        source: source.clone(),
        source_digest,
        source_version: SUPPORTED_SOURCE_VERSION.to_string(),
        source_schema_version: SOURCE_SCHEMA_VERSION.to_string(),
        items,
        action,
        warnings,
    })
}

/// Creates an immutable deterministic plan bound to the inspected bytes.
pub fn plan_migration(inspection: &MigrationInspection) -> Result<MigrationPlan, MigrationError> {
    if inspection.source_schema_version != SOURCE_SCHEMA_VERSION {
        return Err(MigrationError::new(
            MigrationReasonCode::UnsupportedSourceVersion,
            "inspection schema is outside the frozen bridge",
        ));
    }
    let migration_id = format!("boundline-090-{}", &inspection.source_digest[..24]);
    Ok(MigrationPlan {
        migration_id,
        source: inspection.source.clone(),
        source_digest: inspection.source_digest.clone(),
        source_schema_version: inspection.source_schema_version.clone(),
        target_schema_version: TARGET_SCHEMA_VERSION.to_string(),
        implementation_version: MIGRATION_IMPLEMENTATION_VERSION.to_string(),
        action: inspection.action,
        items: inspection.items.clone(),
        warnings: inspection.warnings.clone(),
    })
}

/// Applies a plan with the production no-fault control.
pub fn apply_migration(plan: &MigrationPlan) -> Result<MigrationOutcome, MigrationError> {
    apply_migration_with_control(plan, MigrationControl::default())
}

/// Applies a plan and optionally stops after a declared durable boundary.
pub fn apply_migration_with_control(
    plan: &MigrationPlan,
    control: MigrationControl,
) -> Result<MigrationOutcome, MigrationError> {
    validate_plan(plan)?;
    let paths = MigrationPaths::new(plan);
    if paths.outcome.is_file() {
        return completed_outcome(plan, &paths, MigrationStatus::AlreadyMigrated);
    }
    fs::create_dir_all(&paths.root)
        .map_err(|error| io_error(error, MigrationReasonCode::StagingFailed))?;
    let _ownership = acquire_ownership(&paths)?;
    if paths.outcome.is_file() {
        return completed_outcome(plan, &paths, MigrationStatus::AlreadyMigrated);
    }
    revalidate_source(plan)?;

    let mut journal = MigrationJournal {
        migration_id: plan.migration_id.clone(),
        source_digest: plan.source_digest.clone(),
        source_schema_version: plan.source_schema_version.clone(),
        action: plan.action,
        items: plan.items.clone(),
        warnings: plan.warnings.clone(),
        phase: JournalPhase::Owned,
        backup_digest: None,
        staged_digest: None,
        report: initial_report(plan),
    };
    write_journal(&paths, &journal)?;
    if stopped(control, MigrationBoundary::BeforeBackup) {
        fs::remove_file(&paths.journal)
            .map_err(|error| io_error(error, MigrationReasonCode::RecoveryRequired))?;
        fs::remove_file(&paths.lock)
            .map_err(|error| io_error(error, MigrationReasonCode::RecoveryRequired))?;
        fs::remove_dir(&paths.root)
            .map_err(|error| io_error(error, MigrationReasonCode::RecoveryRequired))?;
        return Ok(outcome(&paths, journal.report, MigrationStatus::SafeRetry, None));
    }

    ensure_absent_or_empty(&paths.backup, MigrationReasonCode::BackupFailed)?;
    copy_tree(&plan.source.root, &paths.backup, MigrationReasonCode::BackupFailed)?;
    let backup_digest = digest_tree(&paths.backup)?;
    write_backup_manifest(plan, &paths, &backup_digest)?;
    journal.backup_digest = Some(backup_digest.clone());
    journal.report.backup_identity = Some(format!("backup:{}", plan.migration_id));
    journal.report.backup_digest = Some(backup_digest.clone());
    journal.phase = JournalPhase::BackupCreated;
    write_journal(&paths, &journal)?;
    if stopped(control, MigrationBoundary::BackupCreated) {
        return recovery_outcome(&paths, journal);
    }

    verify_digest(
        &paths.backup,
        &plan.source_digest,
        MigrationReasonCode::BackupVerificationFailed,
    )?;
    verify_backup_manifest(plan, &paths, &backup_digest)?;
    journal.phase = JournalPhase::BackupVerified;
    journal.report.verification_results.push(verification("backup_digest", true));
    write_journal(&paths, &journal)?;
    if stopped(control, MigrationBoundary::BackupVerified) {
        return recovery_outcome(&paths, journal);
    }

    ensure_absent_or_empty(&paths.staging, MigrationReasonCode::StagingFailed)?;
    copy_tree(&plan.source.root, &paths.staging, MigrationReasonCode::StagingFailed)?;
    journal.phase = JournalPhase::Staging;
    write_journal(&paths, &journal)?;
    write_target_marker(plan, &paths.staging)?;
    if stopped(control, MigrationBoundary::DuringTargetStaging) {
        return recovery_outcome(&paths, journal);
    }
    transform_staging(plan, &paths.staging, &mut journal.report)?;
    let staged_digest = digest_tree(&paths.staging)?;
    journal.staged_digest = Some(staged_digest.clone());
    journal.report.staged_output_identity = Some(format!("staging:{}", plan.migration_id));
    journal.report.staged_output_digest = Some(staged_digest.clone());
    journal.report.verification_results.extend([
        verification("source_identity_revalidated", true),
        verification("unsupported_state_not_resumable", true),
        verification("staged_target_reopened", true),
    ]);
    write_report(&paths.staging.join(REPORT_NAME), &journal.report)?;
    journal.phase = JournalPhase::StagedVerified;
    write_journal(&paths, &journal)?;
    if stopped(control, MigrationBoundary::StagedTargetVerified) {
        return recovery_outcome(&paths, journal);
    }

    journal.phase = JournalPhase::ReplacementReady;
    write_journal(&paths, &journal)?;
    if stopped(control, MigrationBoundary::BeforeReplacement) {
        return recovery_outcome(&paths, journal);
    }
    commit_replacement(&paths, &plan.source.root)?;
    journal.phase = JournalPhase::Replaced;
    write_journal(&paths, &journal)?;
    if stopped(control, MigrationBoundary::AfterReplacement) {
        return recovery_outcome(&paths, journal);
    }
    complete(&paths, &plan.source.root, &mut journal)?;
    if stopped(control, MigrationBoundary::AfterCompletion) {
        return Ok(outcome(&paths, journal.report, MigrationStatus::Complete, Some(backup_digest)));
    }
    release_ownership(&paths)?;
    Ok(outcome(&paths, journal.report, MigrationStatus::Complete, Some(backup_digest)))
}

/// Reconciles the one durable migration associated with the supplied source.
pub fn recover_migration(source: &MigrationSource) -> Result<MigrationOutcome, MigrationError> {
    let paths = find_recovery_paths(source)?;
    let _ownership = acquire_recovery_ownership(&paths)?;
    let mut journal = read_json::<MigrationJournal>(&paths.journal)?;
    let expected_backup = journal.backup_digest.clone().ok_or_else(|| {
        MigrationError::new(
            MigrationReasonCode::BackupVerificationFailed,
            "journal has no verified backup identity",
        )
    })?;
    let plan = plan_from_journal(source, &journal);
    verify_digest(&paths.backup, &expected_backup, MigrationReasonCode::BackupVerificationFailed)?;
    verify_backup_manifest(&plan, &paths, &expected_backup)?;
    if journal.phase == JournalPhase::Completed {
        validate_target(source)?;
        write_report(&paths.outcome, &journal.report)?;
        write_report(&source.root.join(REPORT_NAME), &journal.report)?;
        release_ownership_if_present(&paths)?;
        return Ok(outcome(
            &paths,
            journal.report,
            MigrationStatus::AlreadyMigrated,
            Some(expected_backup),
        ));
    }
    if matches!(
        journal.phase,
        JournalPhase::BackupCreated | JournalPhase::BackupVerified | JournalPhase::Staging
    ) {
        if paths.staging.exists() {
            fs::remove_dir_all(&paths.staging)
                .map_err(|error| io_error(error, MigrationReasonCode::StagingFailed))?;
        }
        copy_tree(&source.root, &paths.staging, MigrationReasonCode::StagingFailed)?;
        write_target_marker(&plan, &paths.staging)?;
        transform_staging(&plan, &paths.staging, &mut journal.report)?;
        let digest = digest_tree(&paths.staging)?;
        journal.staged_digest = Some(digest.clone());
        journal.report.staged_output_digest = Some(digest);
        write_report(&paths.staging.join(REPORT_NAME), &journal.report)?;
        journal.phase = JournalPhase::StagedVerified;
        write_journal(&paths, &journal)?;
    } else if journal.phase == JournalPhase::StagedVerified
        || journal.phase == JournalPhase::ReplacementReady
    {
        let expected = journal.staged_digest.as_deref().ok_or_else(|| {
            MigrationError::new(
                MigrationReasonCode::StagedVerificationFailed,
                "journal has no staged digest",
            )
        })?;
        verify_staged_digest(&paths.staging, expected)?;
    }
    if journal.phase != JournalPhase::Replaced {
        commit_replacement(&paths, &source.root)?;
        journal.phase = JournalPhase::Replaced;
        write_journal(&paths, &journal)?;
    }
    complete(&paths, &source.root, &mut journal)?;
    release_ownership_if_present(&paths)?;
    Ok(outcome(&paths, journal.report, MigrationStatus::Complete, Some(expected_backup)))
}

fn validate_source_root(source: &MigrationSource) -> Result<(), MigrationError> {
    let metadata = fs::symlink_metadata(&source.root)
        .map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(MigrationError::new(
            MigrationReasonCode::UnsafePath,
            "source root must be a real product-owned directory",
        ));
    }
    if source.root.file_name().and_then(|name| name.to_str()) != Some(PRODUCT_STATE_NAME) {
        return Err(MigrationError::new(
            MigrationReasonCode::UnsafePath,
            "source root does not have the admitted product identity",
        ));
    }
    Ok(())
}

fn validate_source_version(
    source: &MigrationSource,
    marker_version: &str,
) -> Result<(), MigrationError> {
    if marker_version != SUPPORTED_SOURCE_VERSION {
        let reason = if marker_version > SUPPORTED_SOURCE_VERSION {
            MigrationReasonCode::UnsupportedFutureSchema
        } else {
            MigrationReasonCode::UnsupportedSourceVersion
        };
        return Err(MigrationError::new(reason, "source release is outside the frozen bridge"));
    }
    if source.declared_version != marker_version {
        return Err(MigrationError::new(
            MigrationReasonCode::IdempotencyConflict,
            "declared source version conflicts with the persisted marker",
        ));
    }
    Ok(())
}

fn reject_target_markers(root: &Path) -> Result<(), MigrationError> {
    if root.join(TARGET_MARKER_NAME).exists() {
        return Err(MigrationError::new(
            MigrationReasonCode::UnsupportedFutureSchema,
            "target or future schema marker is already present",
        ));
    }
    for path in collect_paths(root)? {
        if path.file_name().and_then(|name| name.to_str()) == Some("schema-version") {
            return Err(MigrationError::new(
                MigrationReasonCode::MixedLegacyState,
                "mixed schema generations were detected",
            ));
        }
    }
    Ok(())
}

fn inspect_sessions(root: &Path) -> Result<Vec<LegacyStateItem>, MigrationError> {
    let mut by_identity = BTreeMap::<String, LegacyStateItem>::new();
    for path in collect_paths(root)? {
        if path.file_name().and_then(|name| name.to_str()) != Some("session.json") {
            continue;
        }
        let header = read_json::<LegacySessionHeader>(&path)?;
        if header.session_id.trim().is_empty() {
            return Err(MigrationError::new(
                MigrationReasonCode::CorruptLegacyState,
                "legacy session identity is empty",
            ));
        }
        let archive_required = is_active_status(&header.latest_status)
            || header.active_execution_run_id.as_deref().is_some_and(|value| !value.is_empty());
        let item = LegacyStateItem {
            identity: format!("session:{}", header.session_id),
            kind: "session".to_string(),
            state: header.latest_status,
            archive_required,
            archive_reason: archive_required
                .then(|| "legacy_active_state_has_no_090_transaction_ownership".to_string()),
        };
        by_identity
            .entry(item.identity.clone())
            .and_modify(|existing| {
                if item.archive_required {
                    *existing = item.clone();
                }
            })
            .or_insert(item);
    }
    if by_identity.is_empty() {
        return Err(MigrationError::new(
            MigrationReasonCode::CorruptLegacyState,
            "no recognizable legacy session records were found",
        ));
    }
    Ok(by_identity.into_values().collect())
}

fn is_active_status(status: &str) -> bool {
    matches!(status, "goal_captured" | "planned" | "blocked" | "running" | "invalid")
}

fn validate_plan(plan: &MigrationPlan) -> Result<(), MigrationError> {
    if plan.implementation_version != MIGRATION_IMPLEMENTATION_VERSION
        || plan.source_schema_version != SOURCE_SCHEMA_VERSION
        || plan.target_schema_version != TARGET_SCHEMA_VERSION
    {
        return Err(MigrationError::new(
            MigrationReasonCode::IdempotencyConflict,
            "plan identity does not match this migration implementation",
        ));
    }
    Ok(())
}

fn revalidate_source(plan: &MigrationPlan) -> Result<(), MigrationError> {
    validate_source_root(&plan.source)?;
    verify_digest(&plan.source.root, &plan.source_digest, MigrationReasonCode::SourceChanged)
}

fn transform_staging(
    plan: &MigrationPlan,
    staging: &Path,
    report: &mut ConversionReport,
) -> Result<(), MigrationError> {
    let mut converted = 0;
    let mut archived = 0;
    for item in &plan.items {
        if item.archive_required {
            archive_session(plan, staging, item, report)?;
            archived += 1;
        } else {
            converted += 1;
        }
    }
    report.converted_item_count = converted;
    report.archived_item_count = archived;
    report.semantic_losses.sort_by(|left, right| left.item_identity.cmp(&right.item_identity));
    report.unsupported_states.sort_by(|left, right| left.item_identity.cmp(&right.item_identity));
    Ok(())
}

fn archive_session(
    plan: &MigrationPlan,
    staging: &Path,
    item: &LegacyStateItem,
    report: &mut ConversionReport,
) -> Result<(), MigrationError> {
    let session_id = item.identity.strip_prefix("session:").ok_or_else(|| {
        MigrationError::new(MigrationReasonCode::ArchiveRequired, "invalid session identity")
    })?;
    let archive_root = staging.join("archives").join(session_id);
    let archive_source = archive_root.join("source");
    fs::create_dir_all(&archive_source)
        .map_err(|error| io_error(error, MigrationReasonCode::ArchiveRequired))?;
    let session_dir = staging.join("sessions").join(session_id);
    if session_dir.exists() {
        fs::rename(&session_dir, archive_source.join("session"))
            .map_err(|error| io_error(error, MigrationReasonCode::ArchiveRequired))?;
    }
    let active_projection = staging.join("session.json");
    if active_projection.is_file() {
        let active = read_json::<LegacySessionHeader>(&active_projection)?;
        if active.session_id == session_id {
            fs::rename(&active_projection, archive_source.join("active-session.json"))
                .map_err(|error| io_error(error, MigrationReasonCode::ArchiveRequired))?;
            let pointer = staging.join("active-session");
            if pointer.exists() {
                fs::remove_file(pointer)
                    .map_err(|error| io_error(error, MigrationReasonCode::ArchiveRequired))?;
            }
        }
    }
    let archive_digest = digest_tree(&archive_source)?;
    let reason = item.archive_reason.as_deref().unwrap_or("archive_required");
    write_json(
        &archive_root.join("manifest.json"),
        &ArchiveManifest {
            migration_id: &plan.migration_id,
            source_version: SUPPORTED_SOURCE_VERSION,
            source_schema_version: SOURCE_SCHEMA_VERSION,
            source_digest: &plan.source_digest,
            archive_digest: &archive_digest,
            item_identity: &item.identity,
            reason,
            requires_new_admitted_session: true,
            creation_timestamp_ms: now_millis(),
        },
        MigrationReasonCode::ArchiveVerificationFailed,
    )?;
    set_read_only_recursive(&archive_root)?;
    report.semantic_losses.push(SemanticLossRecord {
        code: "legacy_active_state_not_resumable".to_string(),
        item_identity: item.identity.clone(),
        detail:
            "0.82.0 state lacks exact 0.90 ownership, revision, fingerprint, and idempotency data"
                .to_string(),
    });
    report.unsupported_states.push(UnsupportedStateRecord {
        item_identity: item.identity.clone(),
        reason_code: reason.to_string(),
        requires_new_admitted_session: true,
    });
    Ok(())
}

fn write_target_marker(plan: &MigrationPlan, staging: &Path) -> Result<(), MigrationError> {
    write_json(
        &staging.join(TARGET_MARKER_NAME),
        &TargetSchemaMarker {
            source_product: "boundline",
            source_version: SUPPORTED_SOURCE_VERSION,
            source_schema_version: SOURCE_SCHEMA_VERSION,
            target_version_line: TARGET_VERSION_LINE,
            target_schema_version: TARGET_SCHEMA_VERSION,
            migration_id: &plan.migration_id,
            source_digest: &plan.source_digest,
        },
        MigrationReasonCode::StagingFailed,
    )
}

fn backup_manifest(plan: &MigrationPlan, backup_digest: &str) -> BackupManifest {
    BackupManifest {
        migration_id: plan.migration_id.clone(),
        source_product: "boundline".to_string(),
        source_version: SUPPORTED_SOURCE_VERSION.to_string(),
        source_schema_version: SOURCE_SCHEMA_VERSION.to_string(),
        source_digest: plan.source_digest.clone(),
        backup_digest: backup_digest.to_string(),
    }
}

fn write_backup_manifest(
    plan: &MigrationPlan,
    paths: &MigrationPaths,
    backup_digest: &str,
) -> Result<(), MigrationError> {
    write_json(
        &paths.backup_manifest,
        &backup_manifest(plan, backup_digest),
        MigrationReasonCode::BackupFailed,
    )
}

fn verify_backup_manifest(
    plan: &MigrationPlan,
    paths: &MigrationPaths,
    backup_digest: &str,
) -> Result<(), MigrationError> {
    let bytes = fs::read(&paths.backup_manifest)
        .map_err(|error| io_error(error, MigrationReasonCode::BackupVerificationFailed))?;
    let persisted = serde_json::from_slice::<BackupManifest>(&bytes).map_err(|_| {
        MigrationError::new(
            MigrationReasonCode::BackupVerificationFailed,
            "backup manifest is malformed or truncated",
        )
    })?;
    if persisted == backup_manifest(plan, backup_digest) {
        Ok(())
    } else {
        Err(MigrationError::new(
            MigrationReasonCode::BackupVerificationFailed,
            "backup manifest does not match the admitted source identity",
        ))
    }
}

fn initial_report(plan: &MigrationPlan) -> ConversionReport {
    let now = now_millis();
    ConversionReport {
        migration_id: plan.migration_id.clone(),
        source_product: "boundline".to_string(),
        source_version: SUPPORTED_SOURCE_VERSION.to_string(),
        source_schema_version: SOURCE_SCHEMA_VERSION.to_string(),
        target_version_line: TARGET_VERSION_LINE.to_string(),
        target_schema_version: TARGET_SCHEMA_VERSION.to_string(),
        source_digest: plan.source_digest.clone(),
        inspection_timestamp_ms: now,
        migration_start_timestamp_ms: now,
        migration_completion_timestamp_ms: None,
        backup_identity: None,
        backup_digest: None,
        staged_output_identity: None,
        staged_output_digest: None,
        converted_item_count: 0,
        archived_item_count: 0,
        skipped_item_count: 0,
        warnings: plan.warnings.clone(),
        semantic_losses: Vec::new(),
        unsupported_states: Vec::new(),
        verification_results: Vec::new(),
        terminal_status: MigrationStatus::RecoveryRequired,
        terminal_reason_code: MigrationReasonCode::RecoveryRequired.as_str().to_string(),
        recovery_instructions: Some(
            "re-run the repository-local migration recovery service".to_string(),
        ),
    }
}

fn complete(
    paths: &MigrationPaths,
    source_root: &Path,
    journal: &mut MigrationJournal,
) -> Result<(), MigrationError> {
    validate_target_root(source_root)?;
    journal.report.migration_completion_timestamp_ms = Some(now_millis());
    journal.report.terminal_status = MigrationStatus::Complete;
    journal.report.terminal_reason_code = "migration_complete".to_string();
    journal.report.recovery_instructions = None;
    journal.phase = JournalPhase::Completed;
    write_journal(paths, journal)?;
    write_report(&paths.outcome, &journal.report)?;
    write_report(&source_root.join(REPORT_NAME), &journal.report)?;
    Ok(())
}

fn commit_replacement(paths: &MigrationPaths, source_root: &Path) -> Result<(), MigrationError> {
    if source_root.join(TARGET_MARKER_NAME).is_file() {
        return Ok(());
    }
    if paths.retained.exists() {
        return Err(MigrationError::new(
            MigrationReasonCode::ReplacementFailed,
            "retained source collision requires operator inspection",
        ));
    }
    fs::rename(source_root, &paths.retained)
        .map_err(|error| io_error(error, MigrationReasonCode::ReplacementFailed))?;
    if let Err(error) = fs::rename(&paths.staging, source_root) {
        return Err(io_error(error, MigrationReasonCode::RecoveryRequired));
    }
    sync_parent(source_root)
        .map_err(|error| io_error(error, MigrationReasonCode::ReplacementFailed))
}

fn validate_target(source: &MigrationSource) -> Result<(), MigrationError> {
    validate_target_root(&source.root)
}

fn validate_target_root(root: &Path) -> Result<(), MigrationError> {
    let value = read_json::<serde_json::Value>(&root.join(TARGET_MARKER_NAME))?;
    if value.get("target_schema_version").and_then(serde_json::Value::as_str)
        != Some(TARGET_SCHEMA_VERSION)
    {
        return Err(MigrationError::new(
            MigrationReasonCode::StagedVerificationFailed,
            "committed target schema marker is invalid",
        ));
    }
    Ok(())
}

fn verify_staged_digest(staging: &Path, expected: &str) -> Result<(), MigrationError> {
    let report = staging.join(REPORT_NAME);
    let report_bytes = fs::read(&report)
        .map_err(|error| io_error(error, MigrationReasonCode::StagedVerificationFailed))?;
    fs::remove_file(&report)
        .map_err(|error| io_error(error, MigrationReasonCode::StagedVerificationFailed))?;
    let result = verify_digest(staging, expected, MigrationReasonCode::StagedVerificationFailed);
    fs::write(&report, report_bytes)
        .map_err(|error| io_error(error, MigrationReasonCode::StagedVerificationFailed))?;
    result
}

fn verification(check: &str, passed: bool) -> CanonicalVerificationResult {
    CanonicalVerificationResult {
        check: check.to_string(),
        passed,
        detail: if passed { "verified".to_string() } else { "failed".to_string() },
    }
}

fn stopped(control: MigrationControl, boundary: MigrationBoundary) -> bool {
    control.stop_after == Some(boundary)
}

fn recovery_outcome(
    paths: &MigrationPaths,
    mut journal: MigrationJournal,
) -> Result<MigrationOutcome, MigrationError> {
    journal.report.terminal_status = MigrationStatus::RecoveryRequired;
    journal.report.terminal_reason_code =
        MigrationReasonCode::RecoveryRequired.as_str().to_string();
    write_journal(paths, &journal)?;
    Ok(outcome(paths, journal.report, MigrationStatus::RecoveryRequired, journal.backup_digest))
}

fn outcome(
    paths: &MigrationPaths,
    mut report: ConversionReport,
    status: MigrationStatus,
    backup_digest: Option<String>,
) -> MigrationOutcome {
    report.terminal_status = status;
    MigrationOutcome { status, report, backup_root: paths.backup.clone(), backup_digest }
}

fn completed_outcome(
    plan: &MigrationPlan,
    paths: &MigrationPaths,
    status: MigrationStatus,
) -> Result<MigrationOutcome, MigrationError> {
    let mut report = read_json::<ConversionReport>(&paths.outcome)?;
    if report.migration_id != plan.migration_id
        || report.source_digest != plan.source_digest
        || report.source_schema_version != plan.source_schema_version
        || report.target_schema_version != plan.target_schema_version
    {
        return Err(MigrationError::new(
            MigrationReasonCode::IdempotencyConflict,
            "recorded migration outcome conflicts with the supplied plan",
        ));
    }
    report.terminal_status = status;
    Ok(MigrationOutcome {
        status,
        backup_root: paths.backup.clone(),
        backup_digest: report.backup_digest.clone(),
        report,
    })
}

fn plan_from_journal(source: &MigrationSource, journal: &MigrationJournal) -> MigrationPlan {
    MigrationPlan {
        migration_id: journal.migration_id.clone(),
        source: source.clone(),
        source_digest: journal.source_digest.clone(),
        source_schema_version: journal.source_schema_version.clone(),
        target_schema_version: TARGET_SCHEMA_VERSION.to_string(),
        implementation_version: MIGRATION_IMPLEMENTATION_VERSION.to_string(),
        action: journal.action,
        items: journal.items.clone(),
        warnings: journal.warnings.clone(),
    }
}
