use std::error::Error;
use std::fs;
use std::process::Command;

use boundline::cli::index::execute_doctor;
use boundline::domain::context_intelligence::{
    DerivedIndexManifest, IndexDoctorConsistencyState, IndexDoctorStatus, IndexRefreshReason,
    ManifestFtsState, RetrievalIndexState, SemanticEngine, VectorExtensionState,
};
use rusqlite::Connection;

use crate::workspace_fixture::temp_git_workspace;

type TestResult = Result<(), Box<dyn Error>>;

const INDEX_DIRECTORY_RELATIVE: &str = ".boundline/context-intelligence";
const INDEX_DATABASE_RELATIVE: &str = ".boundline/context-intelligence/retrieval-index.sqlite3";
const INDEX_MANIFEST_RELATIVE: &str = ".boundline/context-intelligence/manifest.json";
const INDEX_DATABASE_WAL_RELATIVE: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3-wal";
const INDEX_DATABASE_SHM_RELATIVE: &str =
    ".boundline/context-intelligence/retrieval-index.sqlite3-shm";

#[test]
fn index_doctor_reports_tracked_files_and_corrupt_manifest_state() -> TestResult {
    let workspace = temp_git_workspace("boundline-context-index-doctor");
    fs::create_dir_all(workspace.join(INDEX_DIRECTORY_RELATIVE))?;
    fs::write(workspace.join(INDEX_DATABASE_RELATIVE), b"not-a-sqlite-db")?;
    fs::write(workspace.join(INDEX_MANIFEST_RELATIVE), b"{not-json}")?;
    fs::write(
        workspace.join(".gitignore"),
        b"# doctor integration intentionally missing derived index rules\n",
    )?;

    let tracked_output = Command::new("/usr/bin/git")
        .args(["add", "--", INDEX_DATABASE_RELATIVE])
        .current_dir(workspace.path())
        .output()?;
    if !tracked_output.status.success() {
        return Err(format!(
            "failed to stage tracked doctor fixture: {}{}",
            String::from_utf8_lossy(&tracked_output.stdout),
            String::from_utf8_lossy(&tracked_output.stderr)
        )
        .into());
    }

    let report = execute_doctor(Some(workspace.path())).map_err(|error| error.to_string())?;
    if report.status != IndexDoctorStatus::Failed {
        return Err(format!("expected failed doctor status, got {:?}", report.status).into());
    }
    if !report.tracked_index_files.iter().any(|entry| entry == INDEX_DATABASE_RELATIVE) {
        return Err(
            format!("expected tracked database artifact in doctor report: {report:?}").into()
        );
    }
    if report.manifest_consistency != IndexDoctorConsistencyState::Corrupt {
        return Err(format!(
            "expected corrupt manifest consistency, got {:?}",
            report.manifest_consistency
        )
        .into());
    }
    if report.vector_schema_consistency != IndexDoctorConsistencyState::Corrupt {
        return Err(format!(
            "expected corrupt vector consistency, got {:?}",
            report.vector_schema_consistency
        )
        .into());
    }
    if report.missing_ignore_rules.is_empty() {
        return Err(format!("expected missing ignore rules in doctor report: {report:?}").into());
    }

    Ok(())
}

#[test]
fn index_doctor_reports_missing_index_when_ignore_rules_are_managed() -> TestResult {
    let workspace = temp_git_workspace("boundline-context-index-doctor-missing");
    fs::write(workspace.join(".gitignore"), managed_ignore_rules_fixture())?;

    let report = execute_doctor(Some(workspace.path())).map_err(|error| error.to_string())?;
    if report.status != IndexDoctorStatus::Advisory {
        return Err(format!("expected advisory doctor status, got {:?}", report.status).into());
    }
    if !report.tracked_index_files.is_empty() {
        return Err(format!("expected no tracked index files, got {report:?}").into());
    }
    if !report.missing_ignore_rules.is_empty() {
        return Err(format!("expected no missing ignore rules, got {report:?}").into());
    }
    if report.wal_sidecars_present {
        return Err(format!("expected wal sidecars to be absent, got {report:?}").into());
    }
    if report.manifest_consistency != IndexDoctorConsistencyState::Missing {
        return Err(format!(
            "expected missing manifest consistency, got {:?}",
            report.manifest_consistency
        )
        .into());
    }
    if report.vector_schema_consistency != IndexDoctorConsistencyState::Missing {
        return Err(format!(
            "expected missing vector schema consistency, got {:?}",
            report.vector_schema_consistency
        )
        .into());
    }

    Ok(())
}

#[test]
fn index_doctor_reports_invalid_manifest_and_schema_states() -> TestResult {
    let workspace = temp_git_workspace("boundline-context-index-doctor-invalid");
    fs::create_dir_all(workspace.join(INDEX_DIRECTORY_RELATIVE))?;
    fs::write(workspace.join(".gitignore"), managed_ignore_rules_fixture())?;

    let connection = Connection::open(workspace.join(INDEX_DATABASE_RELATIVE))?;
    connection.execute("CREATE TABLE placeholder (id INTEGER PRIMARY KEY)", [])?;
    drop(connection);

    let manifest = invalid_manifest_fixture(&workspace);
    fs::write(workspace.join(INDEX_MANIFEST_RELATIVE), serde_json::to_string_pretty(&manifest)?)?;

    let report = execute_doctor(Some(workspace.path())).map_err(|error| error.to_string())?;
    if report.status != IndexDoctorStatus::Failed {
        return Err(format!("expected failed doctor status, got {:?}", report.status).into());
    }
    if report.manifest_consistency != IndexDoctorConsistencyState::Invalid {
        return Err(format!(
            "expected invalid manifest consistency, got {:?}",
            report.manifest_consistency
        )
        .into());
    }
    if report.vector_schema_consistency != IndexDoctorConsistencyState::Invalid {
        return Err(format!(
            "expected invalid vector schema consistency, got {:?}",
            report.vector_schema_consistency
        )
        .into());
    }

    Ok(())
}

fn managed_ignore_rules_fixture() -> String {
    format!(
        "{INDEX_DATABASE_RELATIVE}\n{INDEX_MANIFEST_RELATIVE}\n{INDEX_DATABASE_WAL_RELATIVE}\n{INDEX_DATABASE_SHM_RELATIVE}\n"
    )
}

fn invalid_manifest_fixture(workspace: &std::path::Path) -> DerivedIndexManifest {
    DerivedIndexManifest {
        schema_version: "retrieval-index-v3".to_string(),
        workspace_root: format!("{}/other-workspace", workspace.display()),
        git_branch: Some("main".to_string()),
        git_head: Some("abc123".to_string()),
        last_seen_head: Some("abc123".to_string()),
        index_status: RetrievalIndexState::Ready,
        last_refresh_at: Some("2026-05-30 12:00:00".to_string()),
        last_refresh_reason: Some(IndexRefreshReason::ManualRefresh),
        stale_reason: None,
        file_count: 1,
        chunk_count: 1,
        fts5_state: ManifestFtsState::Ready,
        sqlite_vec_state: VectorExtensionState::Ready,
        semantic_engine: SemanticEngine::SqliteVec,
        workspace_fingerprint: "fnv64:0000000000000001".to_string(),
        config_fingerprint: "fnv64:0000000000000002".to_string(),
        chunker_fingerprint: "fnv64:0000000000000003".to_string(),
        embedding_model_fingerprint: "fnv64:0000000000000004".to_string(),
    }
}
