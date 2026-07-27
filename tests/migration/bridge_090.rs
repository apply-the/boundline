//! Transactional bridge tests derived from the immutable Boundline 0.82.0 state schema.

use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    thread,
};

use boundline_core::migration::{
    MigrationAction, MigrationBoundary, MigrationControl, MigrationReasonCode, MigrationSource,
    MigrationStatus, apply_migration, apply_migration_with_control, inspect_legacy_state,
    plan_migration, recover_migration,
};
use uuid::Uuid;

type TestResult = Result<(), Box<dyn Error>>;

const FIXTURE: &str = "tests/fixtures/migration/boundline-0.82.0/state";
const FIXTURE_PROVENANCE: &str = "tests/fixtures/migration/boundline-0.82.0/PROVENANCE.toml";
const SOURCE_VERSION: &str = "0.82.0";
const CHILD_SOURCE_ENV: &str = "BOUNDLINE_M1D_CHILD_SOURCE";
const CHILD_BOUNDARY_ENV: &str = "BOUNDLINE_M1D_CHILD_BOUNDARY";
const FORCED_EXIT_CODE: i32 = 86;
const MIGRATIONS_DIRECTORY: &str = ".boundline-migrations";

struct TempRoot(PathBuf);

impl TempRoot {
    fn fixture(label: &str) -> Result<Self, Box<dyn Error>> {
        let root = std::env::temp_dir().join(format!("boundline-m1d-{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&root)?;
        copy_tree(Path::new(FIXTURE), &root)?;
        Ok(Self(root))
    }

    fn source(&self) -> MigrationSource {
        MigrationSource::new(self.0.join(".boundline"), SOURCE_VERSION)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(source: &Path, target: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&destination)?;
            copy_tree(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn snapshot(root: &Path) -> std::io::Result<BTreeMap<String, Vec<u8>>> {
    fn walk(
        base: &Path,
        current: &Path,
        output: &mut BTreeMap<String, Vec<u8>>,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                walk(base, &path, output)?;
            } else {
                let relative =
                    path.strip_prefix(base).map_err(std::io::Error::other)?.to_string_lossy();
                output.insert(relative.replace('\\', "/"), fs::read(path)?);
            }
        }
        Ok(())
    }
    let mut output = BTreeMap::new();
    walk(root, root, &mut output)?;
    Ok(output)
}

fn ensure(condition: bool, message: impl Into<String>) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message.into()).into()) }
}

fn plan(
    source: &MigrationSource,
) -> Result<boundline_core::migration::MigrationPlan, Box<dyn Error>> {
    Ok(plan_migration(&inspect_legacy_state(source)?)?)
}

fn boundary_from_label(label: &str) -> Option<MigrationBoundary> {
    MigrationBoundary::all().iter().copied().find(|boundary| boundary.as_str() == label)
}

#[test]
fn migration_reason_codes_have_frozen_display_and_wire_values() -> TestResult {
    let cases = [
        (MigrationReasonCode::UnsupportedSourceVersion, "unsupported_source_version"),
        (MigrationReasonCode::UnsupportedFutureSchema, "unsupported_future_schema"),
        (MigrationReasonCode::CorruptLegacyState, "corrupt_legacy_state"),
        (MigrationReasonCode::MixedLegacyState, "mixed_legacy_state"),
        (MigrationReasonCode::SourceChanged, "source_changed"),
        (MigrationReasonCode::MigrationInProgress, "migration_in_progress"),
        (MigrationReasonCode::BackupFailed, "backup_failed"),
        (MigrationReasonCode::BackupVerificationFailed, "backup_verification_failed"),
        (MigrationReasonCode::StagingFailed, "staging_failed"),
        (MigrationReasonCode::StagedVerificationFailed, "staged_verification_failed"),
        (MigrationReasonCode::ReplacementFailed, "replacement_failed"),
        (MigrationReasonCode::RecoveryRequired, "recovery_required"),
        (MigrationReasonCode::ArchiveRequired, "archive_required"),
        (MigrationReasonCode::ArchiveVerificationFailed, "archive_verification_failed"),
        (MigrationReasonCode::InsufficientSpace, "insufficient_space"),
        (MigrationReasonCode::PermissionDenied, "permission_denied"),
        (MigrationReasonCode::UnsafePath, "unsafe_path"),
        (MigrationReasonCode::UnsupportedFilesystem, "unsupported_filesystem"),
        (MigrationReasonCode::IdempotencyConflict, "idempotency_conflict"),
    ];
    for (reason, expected) in cases {
        ensure(reason.to_string() == expected, format!("{reason:?} display value drifted"))?;
        ensure(
            serde_json::to_string(&reason)? == format!("\"{expected}\""),
            format!("{reason:?} wire value drifted"),
        )?;
    }
    Ok(())
}

#[test]
fn inspection_plan_and_declared_source_identity_are_revalidated() -> TestResult {
    let workspace = TempRoot::fixture("identity-revalidation")?;
    let source = workspace.source();
    let mut inspection = inspect_legacy_state(&source)?;
    inspection.source_schema_version = "future-schema".to_string();
    ensure(
        plan_migration(&inspection).err().map(|error| error.reason_code().as_str())
            == Some("unsupported_source_version"),
        "an inspection from another schema was accepted",
    )?;

    let mut migration_plan = plan(&source)?;
    migration_plan.implementation_version = "different-implementation".to_string();
    ensure(
        apply_migration(&migration_plan).err().map(|error| error.reason_code().as_str())
            == Some("idempotency_conflict"),
        "a plan from another implementation was accepted",
    )?;

    let mismatched = MigrationSource::new(source.root.clone(), "0.81.0");
    ensure(
        inspect_legacy_state(&mismatched).err().map(|error| error.reason_code().as_str())
            == Some("idempotency_conflict"),
        "a caller-declared version that disagrees with the marker was accepted",
    )?;

    let marker = source.root.join("scaffold-manifest.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&marker)?)?;
    value["version"] = serde_json::Value::from(2);
    fs::write(marker, serde_json::to_vec_pretty(&value)?)?;
    ensure(
        inspect_legacy_state(&source).err().map(|error| error.reason_code().as_str())
            == Some("mixed_legacy_state"),
        "a non-frozen legacy scaffold schema was accepted",
    )
}

#[test]
fn forced_termination_child() -> TestResult {
    let Some(root) = std::env::var_os(CHILD_SOURCE_ENV) else {
        return Ok(());
    };
    let label = std::env::var(CHILD_BOUNDARY_ENV)?;
    let boundary = boundary_from_label(&label)
        .ok_or_else(|| std::io::Error::other("unknown forced-termination boundary"))?;
    let source = MigrationSource::new(root, SOURCE_VERSION);
    let migration_plan = plan(&source)?;
    let _ = apply_migration_with_control(&migration_plan, MigrationControl::stop_after(boundary))?;
    if boundary != MigrationBoundary::BeforeBackup {
        let lock = source
            .root
            .parent()
            .ok_or_else(|| std::io::Error::other("missing source parent"))?
            .join(MIGRATIONS_DIRECTORY)
            .join(&migration_plan.migration_id)
            .join("owner.lock");
        let stale_owner =
            OpenOptions::new().read(true).write(true).create(true).truncate(false).open(lock)?;
        stale_owner.try_lock().map_err(std::io::Error::from)?;
    }
    std::process::exit(FORCED_EXIT_CODE);
}

#[test]
fn forced_process_termination_at_every_durable_boundary_recovers() -> TestResult {
    let executable = std::env::current_exe()?;
    for boundary in MigrationBoundary::all() {
        let workspace = TempRoot::fixture("forced-termination")?;
        let source = workspace.source();
        let status = Command::new(&executable)
            .arg("--exact")
            .arg("bridge_090::forced_termination_child")
            .arg("--nocapture")
            .env(CHILD_SOURCE_ENV, &source.root)
            .env(CHILD_BOUNDARY_ENV, boundary.as_str())
            .status()?;
        ensure(
            status.code() == Some(FORCED_EXIT_CODE),
            format!("{} child did not terminate at the injected boundary", boundary.as_str()),
        )?;
        let recovered = if *boundary == MigrationBoundary::BeforeBackup {
            let migration_root = workspace.0.join(".boundline-migrations");
            ensure(
                !migration_root.exists() || fs::read_dir(&migration_root)?.next().is_none(),
                "pre-backup termination left recovery state",
            )?;
            apply_migration(&plan(&source)?)?
        } else {
            recover_migration(&source)?
        };
        ensure(
            matches!(
                recovered.status,
                MigrationStatus::Complete | MigrationStatus::AlreadyMigrated
            ),
            format!("{} did not recover after process termination", boundary.as_str()),
        )?;
    }
    Ok(())
}

#[test]
fn inspect_and_plan_are_deterministic_and_non_mutating() -> TestResult {
    let workspace = TempRoot::fixture("inspect")?;
    let before = snapshot(&workspace.0)?;
    let first = inspect_legacy_state(&workspace.source())?;
    let second = inspect_legacy_state(&workspace.source())?;
    let first_plan = plan_migration(&first)?;
    let second_plan = plan_migration(&second)?;

    ensure(first == second, "inspection changed across equivalent reads")?;
    ensure(first_plan == second_plan, "migration plan changed across equivalent reads")?;
    ensure(before == snapshot(&workspace.0)?, "inspection or planning mutated the fixture")?;
    ensure(!workspace.0.join(".boundline-migrations").exists(), "inspect created migration state")
}

#[test]
fn historical_fixture_matches_its_immutable_provenance_digest() -> TestResult {
    let workspace = TempRoot::fixture("fixture-provenance")?;
    let provenance = fs::read_to_string(FIXTURE_PROVENANCE)?.parse::<toml::Table>()?;
    let expected = provenance
        .get("fixture_digest")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| std::io::Error::other("fixture provenance digest missing"))?;
    let inspection = inspect_legacy_state(&workspace.source())?;
    ensure(
        inspection.source_digest == expected,
        "Boundline historical fixture drifted from its provenance record",
    )
}

#[test]
fn supported_inactive_and_terminal_state_migrate_with_backup_and_idempotency() -> TestResult {
    let workspace = TempRoot::fixture("supported")?;
    let source = workspace.source();
    let migration_plan = plan(&source)?;
    ensure(
        migration_plan.action == MigrationAction::Convert,
        "inactive state was not convertible",
    )?;
    let first = apply_migration(&migration_plan)?;
    ensure(first.status == MigrationStatus::Complete, "migration did not complete")?;
    ensure(first.backup_digest.is_some(), "verified backup digest missing")?;
    ensure(
        first
            .backup_root
            .parent()
            .ok_or_else(|| std::io::Error::other("missing migration root"))?
            .join("backup-manifest.json")
            .is_file(),
        "durable backup manifest missing",
    )?;
    ensure(first.report.converted_item_count >= 2, "session inventory was not converted")?;
    ensure(
        first.report.verification_results.iter().all(|result| result.passed),
        "migration report contains a failed verification",
    )?;
    let target = snapshot(&workspace.0.join(".boundline"))?;
    ensure(
        target.keys().any(|path| path == "sessions/session-terminal/traces/task-terminal.json"),
        "terminal trace reference was not preserved",
    )?;

    let rerun = apply_migration(&migration_plan)?;
    ensure(rerun.status == MigrationStatus::AlreadyMigrated, "rerun was not idempotent")?;
    ensure(target == snapshot(&workspace.0.join(".boundline"))?, "rerun changed target semantics")?;
    let mut conflicting = migration_plan;
    conflicting.source_digest = "different-source-digest".to_string();
    ensure(
        apply_migration(&conflicting).err().map(|error| error.reason_code().as_str())
            == Some("idempotency_conflict"),
        "same migration identity accepted a conflicting source digest",
    )
}

#[test]
fn active_and_partially_mutated_sessions_are_archived_not_resumed() -> TestResult {
    let workspace = TempRoot::fixture("archive-active")?;
    let session = workspace.0.join(".boundline/session.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&session)?)?;
    value["latest_status"] = serde_json::Value::String("running".to_string());
    value["active_execution_run_id"] = serde_json::Value::String("legacy-live-run".to_string());
    fs::write(&session, serde_json::to_vec_pretty(&value)?)?;

    let inspection = inspect_legacy_state(&workspace.source())?;
    let migration_plan = plan_migration(&inspection)?;
    ensure(
        migration_plan.action == MigrationAction::ArchiveUnsupported,
        "active state was adopted",
    )?;
    let outcome = apply_migration(&migration_plan)?;
    ensure(outcome.status == MigrationStatus::Complete, "archive migration did not complete")?;
    ensure(outcome.report.archived_item_count > 0, "active state was not archived")?;
    ensure(
        outcome.report.unsupported_states.iter().any(|item| item.requires_new_admitted_session),
        "archive did not require a new admitted session",
    )?;
    ensure(!session.exists(), "archived active projection remained resumable")?;
    let archive_root = workspace.0.join(".boundline/archives/session-inactive");
    ensure(archive_root.join("manifest.json").is_file(), "immutable archive manifest missing")?;
    ensure(
        fs::metadata(archive_root.join("manifest.json"))?.permissions().readonly(),
        "archive manifest is not read-only",
    )?;
    ensure(
        fs::metadata(&archive_root)?.permissions().readonly()
            && fs::metadata(archive_root.join("source"))?.permissions().readonly(),
        "archive directories are not read-only",
    )
}

#[test]
fn future_corrupt_and_mixed_state_fail_closed_before_mutation() -> TestResult {
    for (label, mutate, expected) in [
        ("future", "future", "unsupported_future_schema"),
        ("corrupt", "corrupt", "corrupt_legacy_state"),
        ("mixed", "mixed", "mixed_legacy_state"),
    ] {
        let workspace = TempRoot::fixture(label)?;
        let source = workspace.source();
        let before = snapshot(&workspace.0)?;
        match mutate {
            "future" => {
                let marker = workspace.0.join(".boundline/scaffold-manifest.json");
                let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&marker)?)?;
                value["boundline_version"] = serde_json::Value::String("9.0.0".to_string());
                fs::write(marker, serde_json::to_vec_pretty(&value)?)?;
            }
            "corrupt" => fs::write(workspace.0.join(".boundline/session.json"), b"{")?,
            _ => fs::write(
                workspace.0.join(".boundline/sessions/session-inactive/schema-version"),
                b"0.90",
            )?,
        }
        let result = inspect_legacy_state(&source);
        let code = result.err().map(|error| error.reason_code().as_str().to_string());
        ensure(code.as_deref() == Some(expected), format!("{label} returned {code:?}"))?;
        ensure(before != snapshot(&workspace.0)?, "test mutation was not applied")?;
        ensure(
            !workspace.0.join(".boundline-migrations").exists(),
            "failed inspect mutated state",
        )?;
    }
    Ok(())
}

#[test]
fn changed_source_backup_and_staging_collisions_fail_closed() -> TestResult {
    let workspace = TempRoot::fixture("collisions")?;
    let source = workspace.source();
    let migration_plan = plan(&source)?;
    fs::write(workspace.0.join(".boundline/unexplained.txt"), b"external edit")?;
    let changed = apply_migration(&migration_plan).err().map(|error| error.reason_code());
    ensure(
        changed.as_ref().map(|code| code.as_str()) == Some("source_changed"),
        "source drift accepted",
    )?;

    let fresh = plan(&source)?;
    let interrupted = apply_migration_with_control(
        &fresh,
        MigrationControl::stop_after(MigrationBoundary::BackupVerified),
    )?;
    ensure(
        interrupted.status == MigrationStatus::RecoveryRequired,
        "fault did not require recovery",
    )?;
    fs::write(interrupted.backup_root.join("tampered"), b"tampered")?;
    let backup_error = recover_migration(&source).err().map(|error| error.reason_code());
    ensure(
        backup_error.as_ref().map(|code| code.as_str()) == Some("backup_verification_failed"),
        "tampered backup was reused",
    )?;

    let manifest_workspace = TempRoot::fixture("backup-manifest-collision")?;
    let manifest_source = manifest_workspace.source();
    let interrupted = apply_migration_with_control(
        &plan(&manifest_source)?,
        MigrationControl::stop_after(MigrationBoundary::BackupVerified),
    )?;
    fs::write(
        interrupted
            .backup_root
            .parent()
            .ok_or_else(|| std::io::Error::other("missing migration root"))?
            .join("backup-manifest.json"),
        b"{}",
    )?;
    ensure(
        recover_migration(&manifest_source).err().map(|error| error.reason_code().as_str())
            == Some("backup_verification_failed"),
        "mismatching backup manifest was accepted",
    )?;

    let staging_workspace = TempRoot::fixture("staging-collision")?;
    let staging_source = staging_workspace.source();
    let staged = apply_migration_with_control(
        &plan(&staging_source)?,
        MigrationControl::stop_after(MigrationBoundary::StagedTargetVerified),
    )?;
    let staging_root = staged
        .backup_root
        .parent()
        .ok_or_else(|| std::io::Error::other("missing migration root"))?
        .join("staging");
    fs::write(staging_root.join("tampered"), b"tampered")?;
    ensure(
        recover_migration(&staging_source).err().map(|error| error.reason_code().as_str())
            == Some("staged_verification_failed"),
        "mismatching staged identity was accepted",
    )
}

#[cfg(unix)]
#[test]
fn symlinks_and_paths_outside_the_source_root_are_rejected() -> TestResult {
    use std::os::unix::fs::symlink;

    let workspace = TempRoot::fixture("unsafe-path")?;
    let external = workspace.0.join("external.txt");
    fs::write(&external, b"outside")?;
    symlink(&external, workspace.0.join(".boundline/escape"))?;
    let result = inspect_legacy_state(&workspace.source());
    ensure(
        result.err().map(|error| error.reason_code().as_str()) == Some("unsafe_path"),
        "symlink escape was accepted",
    )?;
    ensure(fs::read(external)? == b"outside", "external symlink target was modified")
}

#[cfg(unix)]
#[test]
fn permission_and_write_failures_preserve_the_authoritative_source() -> TestResult {
    use std::os::unix::fs::PermissionsExt;

    let permission_workspace = TempRoot::fixture("permission")?;
    let permission_source = permission_workspace.source();
    let permission_plan = plan(&permission_source)?;
    let before = snapshot(&permission_workspace.0)?;
    fs::set_permissions(&permission_workspace.0, fs::Permissions::from_mode(0o500))?;
    let result = apply_migration(&permission_plan);
    fs::set_permissions(&permission_workspace.0, fs::Permissions::from_mode(0o700))?;
    ensure(
        result.err().map(|error| error.reason_code().as_str()) == Some("permission_denied"),
        "insufficient permissions did not return permission_denied",
    )?;
    ensure(before == snapshot(&permission_workspace.0)?, "permission failure changed source")?;

    let write_workspace = TempRoot::fixture("write-failure")?;
    let write_source = write_workspace.source();
    let write_plan = plan(&write_source)?;
    let before = snapshot(&write_workspace.0)?;
    fs::write(write_workspace.0.join(".boundline-migrations"), b"not-a-directory")?;
    let result = apply_migration(&write_plan);
    ensure(
        result.err().map(|error| error.reason_code().as_str()) == Some("staging_failed"),
        "write failure did not fail before replacement",
    )?;
    let mut after = snapshot(&write_workspace.0)?;
    after.remove(".boundline-migrations");
    ensure(before == after, "write failure changed authoritative source")
}

#[test]
fn concurrent_migrators_admit_one_owner_and_preserve_cross_product_state() -> TestResult {
    let workspace = TempRoot::fixture("concurrent")?;
    let canon_sentinel = workspace.0.join(".canon/sentinel");
    fs::create_dir_all(canon_sentinel.parent().ok_or("missing sentinel parent")?)?;
    fs::write(&canon_sentinel, b"canon-owned")?;
    let source = Arc::new(workspace.source());
    let first_plan = Arc::new(plan(&source)?);
    let left = {
        let plan = Arc::clone(&first_plan);
        thread::spawn(move || apply_migration(&plan))
    };
    let right = {
        let plan = Arc::clone(&first_plan);
        thread::spawn(move || apply_migration(&plan))
    };
    let left = left.join().map_err(|_| "first migrator panicked")?;
    let right = right.join().map_err(|_| "second migrator panicked")?;
    ensure(left.is_ok() || right.is_ok(), "neither concurrent migrator completed")?;
    ensure(fs::read(canon_sentinel)? == b"canon-owned", "Boundline migration changed Canon state")
}

#[test]
fn every_durable_boundary_recovers_without_silent_partial_success() -> TestResult {
    for boundary in MigrationBoundary::all() {
        let workspace = TempRoot::fixture(boundary.as_str())?;
        let source = workspace.source();
        let migration_plan = plan(&source)?;
        let interrupted =
            apply_migration_with_control(&migration_plan, MigrationControl::stop_after(*boundary))?;
        ensure(
            matches!(
                interrupted.status,
                MigrationStatus::SafeRetry
                    | MigrationStatus::RecoveryRequired
                    | MigrationStatus::Complete
            ),
            format!("{} produced silent partial success", boundary.as_str()),
        )?;
        let recovered = if interrupted.status == MigrationStatus::SafeRetry {
            apply_migration(&migration_plan)?
        } else {
            recover_migration(&source)?
        };
        ensure(
            matches!(
                recovered.status,
                MigrationStatus::Complete | MigrationStatus::AlreadyMigrated
            ),
            format!("{} did not recover", boundary.as_str()),
        )?;
        ensure(recovered.report.backup_identity.is_some(), "recovery lost backup identity")?;
    }
    Ok(())
}

#[test]
fn reports_are_complete_deterministic_and_secret_free_after_normalization() -> TestResult {
    let first_workspace = TempRoot::fixture("report-first")?;
    let second_workspace = TempRoot::fixture("report-second")?;
    let first = apply_migration(&plan(&first_workspace.source())?)?;
    let second = apply_migration(&plan(&second_workspace.source())?)?;
    ensure(
        first.report.normalized() == second.report.normalized(),
        "equivalent reports were not deterministic",
    )?;
    let rendered = serde_json::to_string(&first.report)?;
    ensure(
        !rendered.contains(&first_workspace.0.to_string_lossy().to_string()),
        "report leaked path",
    )?;
    ensure(first.report.terminal_reason_code == "migration_complete", "terminal reason missing")
}
