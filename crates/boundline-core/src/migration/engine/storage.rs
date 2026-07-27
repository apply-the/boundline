//! Crash-consistent filesystem primitives for the Boundline bridge.

use super::*;

pub(super) struct MigrationPaths {
    pub(super) root: PathBuf,
    pub(super) lock: PathBuf,
    pub(super) journal: PathBuf,
    pub(super) outcome: PathBuf,
    pub(super) backup: PathBuf,
    pub(super) backup_manifest: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) retained: PathBuf,
}

impl MigrationPaths {
    pub(super) fn new(plan: &MigrationPlan) -> Self {
        let parent = plan.source.root.parent().unwrap_or_else(|| Path::new("."));
        Self::from_root(parent.join(MIGRATIONS_NAME).join(&plan.migration_id))
    }

    pub(super) fn from_root(root: PathBuf) -> Self {
        Self {
            lock: root.join(LOCK_NAME),
            journal: root.join(JOURNAL_NAME),
            outcome: root.join(OUTCOME_NAME),
            backup: root.join(BACKUP_NAME),
            backup_manifest: root.join(BACKUP_MANIFEST_NAME),
            staging: root.join(STAGING_NAME),
            retained: root.join(RETAINED_NAME),
            root,
        }
    }
}

pub(super) struct Ownership {
    file: File,
    path: PathBuf,
}

impl Drop for Ownership {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) fn acquire_ownership(paths: &MigrationPaths) -> Result<Ownership, MigrationError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&paths.lock)
        .map_err(|error| io_error(error, MigrationReasonCode::PermissionDenied))?;
    file.try_lock().map_err(|error| match error {
        std::fs::TryLockError::WouldBlock => MigrationError::new(
            MigrationReasonCode::MigrationInProgress,
            "another live migrator owns the source",
        ),
        std::fs::TryLockError::Error(error) => {
            io_error(error, MigrationReasonCode::PermissionDenied)
        }
    })?;
    file.set_len(0)
        .and_then(|()| file.write_all(b"boundline-bridge-090\n"))
        .and_then(|()| file.sync_all())
        .map_err(|error| io_error(error, MigrationReasonCode::PermissionDenied))?;
    Ok(Ownership { file, path: paths.lock.clone() })
}

pub(super) fn acquire_recovery_ownership(
    paths: &MigrationPaths,
) -> Result<Ownership, MigrationError> {
    acquire_ownership(paths)
}

pub(super) fn release_ownership(paths: &MigrationPaths) -> Result<(), MigrationError> {
    fs::remove_file(&paths.lock)
        .map_err(|error| io_error(error, MigrationReasonCode::RecoveryRequired))
}

pub(super) fn release_ownership_if_present(paths: &MigrationPaths) -> Result<(), MigrationError> {
    if paths.lock.exists() { release_ownership(paths) } else { Ok(()) }
}

pub(super) fn find_recovery_paths(
    source: &MigrationSource,
) -> Result<MigrationPaths, MigrationError> {
    let parent = source.root.parent().unwrap_or_else(|| Path::new("."));
    let root = parent.join(MIGRATIONS_NAME);
    let mut candidates = Vec::new();
    let entries = fs::read_dir(&root)
        .map_err(|error| io_error(error, MigrationReasonCode::RecoveryRequired))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| io_error(error, MigrationReasonCode::RecoveryRequired))?;
        let candidate = MigrationPaths::from_root(entry.path());
        if candidate.journal.is_file() {
            candidates.push(candidate);
        }
    }
    if candidates.len() != 1 {
        return Err(MigrationError::new(
            MigrationReasonCode::RecoveryRequired,
            "recovery requires exactly one explanatory journal",
        ));
    }
    candidates.pop().ok_or_else(|| {
        MigrationError::new(MigrationReasonCode::RecoveryRequired, "migration journal missing")
    })
}

pub(super) fn write_journal(
    paths: &MigrationPaths,
    journal: &MigrationJournal,
) -> Result<(), MigrationError> {
    write_json(&paths.journal, journal, MigrationReasonCode::StagingFailed)?;
    tracing::info!(
        migration_id = %journal.migration_id,
        phase = ?journal.phase,
        "durable migration phase committed"
    );
    Ok(())
}

pub(super) fn write_report(path: &Path, report: &ConversionReport) -> Result<(), MigrationError> {
    write_json(path, report, MigrationReasonCode::StagingFailed)
}

pub(super) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, MigrationError> {
    let bytes =
        fs::read(path).map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
    serde_json::from_slice(&bytes).map_err(|_| {
        MigrationError::new(
            MigrationReasonCode::CorruptLegacyState,
            "legacy JSON is malformed or truncated",
        )
    })
}

pub(super) fn write_json<T: Serialize>(
    path: &Path,
    value: &T,
    reason: MigrationReasonCode,
) -> Result<(), MigrationError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|_| MigrationError::new(reason, "typed migration record serialization failed"))?;
    durable_write(path, &bytes).map_err(|error| io_error(error, reason))
}

pub(super) fn durable_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("missing parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.tmp",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("migration")
    ));
    let mut file = File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    sync_parent(path)
}

pub(super) fn sync_parent(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("missing parent"))?;
    File::open(parent)?.sync_all()
}

pub(super) fn copy_tree(
    source: &Path,
    target: &Path,
    reason: MigrationReasonCode,
) -> Result<(), MigrationError> {
    fs::create_dir_all(target).map_err(|error| io_error(error, reason))?;
    for entry in fs::read_dir(source).map_err(|error| io_error(error, reason))? {
        let entry = entry.map_err(|error| io_error(error, reason))?;
        let metadata =
            fs::symlink_metadata(entry.path()).map_err(|error| io_error(error, reason))?;
        if metadata.file_type().is_symlink() {
            return Err(MigrationError::new(
                MigrationReasonCode::UnsafePath,
                "symlinks are not admitted by the M1D bridge",
            ));
        }
        let destination = target.join(entry.file_name());
        if metadata.is_dir() {
            copy_tree(&entry.path(), &destination, reason)?;
        } else if metadata.is_file() {
            let mut input = File::open(entry.path()).map_err(|error| io_error(error, reason))?;
            let mut output = File::create(&destination).map_err(|error| io_error(error, reason))?;
            std::io::copy(&mut input, &mut output).map_err(|error| io_error(error, reason))?;
            output.sync_all().map_err(|error| io_error(error, reason))?;
            fs::set_permissions(&destination, metadata.permissions())
                .map_err(|error| io_error(error, reason))?;
        } else {
            return Err(MigrationError::new(
                MigrationReasonCode::UnsafePath,
                "special files are not admitted by the M1D bridge",
            ));
        }
    }
    File::open(target)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| io_error(error, reason))
}

pub(super) fn ensure_absent_or_empty(
    path: &Path,
    reason: MigrationReasonCode,
) -> Result<(), MigrationError> {
    if !path.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(path).map_err(|error| io_error(error, reason))?;
    if entries.next().is_some() {
        return Err(MigrationError::new(reason, "existing migration artifact identity mismatches"));
    }
    Ok(())
}

pub(super) fn collect_paths(root: &Path) -> Result<Vec<PathBuf>, MigrationError> {
    fn walk(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), MigrationError> {
        for entry in fs::read_dir(root)
            .map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?
        {
            let entry =
                entry.map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
            if metadata.file_type().is_symlink() {
                return Err(MigrationError::new(
                    MigrationReasonCode::UnsafePath,
                    "legacy state contains a symlink",
                ));
            }
            if metadata.is_dir() {
                walk(&entry.path(), output)?;
            } else if metadata.is_file() {
                output.push(entry.path());
            } else {
                return Err(MigrationError::new(
                    MigrationReasonCode::UnsafePath,
                    "legacy state contains a special file",
                ));
            }
        }
        Ok(())
    }
    let mut output = Vec::new();
    walk(root, &mut output)?;
    output.sort();
    Ok(output)
}

pub(super) fn digest_tree(root: &Path) -> Result<String, MigrationError> {
    let mut hasher = Sha256::new();
    for path in collect_paths(root)? {
        let relative = path.strip_prefix(root).map_err(|_| {
            MigrationError::new(MigrationReasonCode::UnsafePath, "path escaped the source root")
        })?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        hasher.update((normalized.len() as u64).to_be_bytes());
        hasher.update(normalized.as_bytes());
        let mut file = File::open(&path)
            .map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
        let metadata = file
            .metadata()
            .map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
        hasher.update(metadata.len().to_be_bytes());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            hasher.update(metadata.permissions().mode().to_be_bytes());
        }
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| io_error(error, MigrationReasonCode::CorruptLegacyState))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn verify_digest(
    root: &Path,
    expected: &str,
    reason: MigrationReasonCode,
) -> Result<(), MigrationError> {
    if digest_tree(root)? == expected {
        Ok(())
    } else {
        Err(MigrationError::new(reason, "tree digest does not match its durable identity"))
    }
}

pub(super) fn set_read_only_recursive(root: &Path) -> Result<(), MigrationError> {
    fn protect(path: &Path) -> Result<(), MigrationError> {
        let mut permissions = fs::metadata(path)
            .map_err(|error| io_error(error, MigrationReasonCode::ArchiveVerificationFailed))?
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions)
            .map_err(|error| io_error(error, MigrationReasonCode::ArchiveVerificationFailed))
    }

    fn walk(directory: &Path) -> Result<(), MigrationError> {
        for entry in fs::read_dir(directory)
            .map_err(|error| io_error(error, MigrationReasonCode::ArchiveVerificationFailed))?
        {
            let entry = entry
                .map_err(|error| io_error(error, MigrationReasonCode::ArchiveVerificationFailed))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| io_error(error, MigrationReasonCode::ArchiveVerificationFailed))?;
            if metadata.file_type().is_symlink() {
                return Err(MigrationError::new(
                    MigrationReasonCode::UnsafePath,
                    "archive contains a symlink",
                ));
            }
            if metadata.is_dir() {
                walk(&path)?;
            } else if !metadata.is_file() {
                return Err(MigrationError::new(
                    MigrationReasonCode::UnsafePath,
                    "archive contains a special file",
                ));
            }
            protect(&path)?;
        }
        Ok(())
    }

    walk(root)?;
    protect(root)
}

pub(super) fn now_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_millis() as u64)
}

pub(super) fn io_error(error: std::io::Error, default: MigrationReasonCode) -> MigrationError {
    let reason = match error.kind() {
        std::io::ErrorKind::PermissionDenied => MigrationReasonCode::PermissionDenied,
        std::io::ErrorKind::StorageFull => MigrationReasonCode::InsufficientSpace,
        _ => default,
    };
    MigrationError::new(reason, error.to_string())
}
