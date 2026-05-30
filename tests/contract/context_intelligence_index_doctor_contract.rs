use std::error::Error;
use std::fs;
use std::process::Command;

use crate::workspace_fixture::{run_boundline_in, stdout_json, temp_git_workspace, terminal_text};

type TestResult = Result<(), Box<dyn Error>>;

const DOCTOR_COMMAND: &str = "boundline index doctor";
const REQUIRED_CHECK_FIELDS: [&str; 4] = ["check_name", "result", "detail", "suggested_fix"];
const REQUIRED_DOCTOR_FIELDS: [&str; 7] = [
    "status",
    "checks",
    "tracked_index_files",
    "missing_ignore_rules",
    "wal_sidecars_present",
    "manifest_consistency",
    "vector_schema_consistency",
];
const INDEX_DIRECTORY_RELATIVE: &str = ".boundline/context-intelligence";
const INDEX_DATABASE_RELATIVE: &str = ".boundline/context-intelligence/retrieval-index.sqlite3";
const INDEX_MANIFEST_RELATIVE: &str = ".boundline/context-intelligence/manifest.json";
const INDEX_WAL_RELATIVE: &str = ".boundline/context-intelligence/retrieval-index.sqlite3-wal";

#[test]
fn index_doctor_contract_reports_tracked_and_corrupt_artifacts() -> TestResult {
    if DOCTOR_COMMAND.trim().is_empty() {
        return Err("expected doctor command entry to be non-empty".into());
    }

    let workspace = temp_git_workspace("boundline-index-doctor-contract");
    let index_directory = workspace.join(INDEX_DIRECTORY_RELATIVE);
    fs::create_dir_all(&index_directory)?;
    fs::write(workspace.join(INDEX_DATABASE_RELATIVE), b"not-a-sqlite-db")?;
    fs::write(workspace.join(INDEX_MANIFEST_RELATIVE), b"{not-json}")?;
    fs::write(workspace.join(INDEX_WAL_RELATIVE), b"wal-sidecar")?;
    fs::write(
        workspace.join(".gitignore"),
        b"# intentionally incomplete ignore rules for doctor contract\n",
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

    let doctor = run_boundline_in(
        &workspace,
        &["index", "doctor", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let doctor_text = terminal_text(&doctor);
    if doctor.status.code() != Some(0) {
        return Err(format!("expected doctor success, got output: {doctor_text}").into());
    }

    let report: serde_json::Value = stdout_json(&doctor);
    let report_object = report.as_object().ok_or("expected doctor report JSON object")?;
    for field in REQUIRED_DOCTOR_FIELDS {
        if !report_object.contains_key(field) {
            return Err(format!("missing `{field}` in {report}").into());
        }
    }
    let checks = report["checks"].as_array().ok_or("expected doctor checks array")?;
    if checks.is_empty() {
        return Err(format!("expected doctor checks to be non-empty: {report}").into());
    }
    for check in checks {
        let check_object = check.as_object().ok_or("expected doctor check object")?;
        for field in REQUIRED_CHECK_FIELDS {
            if !check_object.contains_key(field) {
                return Err(format!("missing `{field}` in doctor check {check}").into());
            }
        }
    }
    if report["status"] != "failed" {
        return Err(format!(
            "expected failed doctor status for tracked/corrupt artifacts: {report}"
        )
        .into());
    }
    if !report["tracked_index_files"]
        .as_array()
        .is_some_and(|entries| entries.iter().any(|entry| entry == INDEX_DATABASE_RELATIVE))
    {
        return Err(format!("expected tracked index database to be reported: {report}").into());
    }
    if report["manifest_consistency"] != "corrupt" {
        return Err(format!("expected corrupt manifest consistency: {report}").into());
    }
    if report["vector_schema_consistency"] != "corrupt" {
        return Err(format!("expected corrupt vector schema consistency: {report}").into());
    }
    if report["wal_sidecars_present"] != true {
        return Err(format!("expected WAL sidecar presence to be reported: {report}").into());
    }

    Ok(())
}
