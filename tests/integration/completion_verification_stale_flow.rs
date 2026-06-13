use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::CommandExitStatus;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};
use boundline::domain::completion_verification::{
    CompletionVerificationFinding, CompletionVerificationFindingKind,
    CompletionVerificationProjection, CompletionVerificationState,
};
use boundline::domain::session::SessionStatus;

use crate::workspace_fixture::temp_fixture_workspace;

const DOC_PATH: &str = "docs/release-readiness.md";
const LIB_PATH: &str = "src/lib.rs";
const TRUNCATION_MARKER: &str = "[truncated]";

#[test]
fn status_marks_a_passing_proof_stale_after_source_changes() -> Result<(), String> {
    let workspace = temp_fixture_workspace("completion-verification-stale-source");

    prove_add_fix(&workspace)?;
    fs::write(
        workspace.join(LIB_PATH),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right + 1\n}\n",
    )
    .map_err(|error| error.to_string())?;

    let status_report = execute_status(Some(&workspace)).map_err(|error| error.to_string())?;
    let session = load_session(&workspace)?;
    let projection = load_projection(&session)?;
    let finding = stale_finding(&projection)?;

    if session.latest_status != SessionStatus::Blocked {
        return Err(format!(
            "expected blocked session after stale proof, got {:?}",
            session.latest_status
        ));
    }
    if projection.completion_verification_state != CompletionVerificationState::ProofRequired {
        return Err(format!(
            "expected proof_required stale projection, got {:?}",
            projection.completion_verification_state
        ));
    }
    if !finding.changed_paths.iter().any(|path| path == LIB_PATH) {
        return Err(format!(
            "expected changed paths to include {LIB_PATH}, got {:?}",
            finding.changed_paths
        ));
    }
    if !status_report
        .terminal_output
        .contains("completion_verification_finding: stale_proof | blocking |")
    {
        return Err(status_report.terminal_output);
    }

    Ok(())
}

#[test]
fn documentation_changes_are_ignored_when_the_selected_proof_excludes_docs() -> Result<(), String> {
    let workspace = temp_fixture_workspace("completion-verification-docs-ignored");

    prove_add_fix(&workspace)?;
    write_doc_file(
        &workspace,
        "# Release readiness\n\nUpdated note without changing runtime behavior.\n",
    )?;

    let status_report = execute_status(Some(&workspace)).map_err(|error| error.to_string())?;
    let session = load_session(&workspace)?;
    let projection = load_projection(&session)?;
    if session.latest_status != SessionStatus::Succeeded {
        return Err(format!(
            "expected succeeded session when docs are not claim-relevant, got {:?}",
            session.latest_status
        ));
    }
    if projection.completion_verification_state != CompletionVerificationState::Ready {
        return Err(format!(
            "expected ready projection when docs are not claim-relevant, got {:?}",
            projection.completion_verification_state
        ));
    }
    if status_report.terminal_output.contains("stale_proof") {
        return Err(status_report.terminal_output);
    }

    Ok(())
}

#[test]
fn documentation_changes_invalidate_when_the_selected_proof_marks_docs_relevant()
-> Result<(), String> {
    let workspace = temp_fixture_workspace("completion-verification-docs-relevant");

    prove_add_fix(&workspace)?;
    mark_docs_relevant(&workspace)?;
    write_doc_file(
        &workspace,
        "# Release readiness\n\nUpdated note that should invalidate the proof.\n",
    )?;

    let status_report = execute_status(Some(&workspace)).map_err(|error| error.to_string())?;
    let session = load_session(&workspace)?;
    let projection = load_projection(&session)?;
    let finding = stale_finding(&projection)?;

    if session.latest_status != SessionStatus::Blocked {
        return Err(format!(
            "expected blocked session when docs are claim-relevant, got {:?}",
            session.latest_status
        ));
    }
    if !finding.changed_paths.iter().any(|path| path == DOC_PATH) {
        return Err(format!(
            "expected changed paths to include {DOC_PATH}, got {:?}",
            finding.changed_paths
        ));
    }
    if !status_report
        .terminal_output
        .contains("completion_verification_changed_paths: docs/release-readiness.md")
    {
        return Err(status_report.terminal_output);
    }

    Ok(())
}

#[test]
fn stale_changed_paths_are_capped_and_marked_truncated() -> Result<(), String> {
    let workspace = temp_fixture_workspace("completion-verification-stale-truncated");

    prove_add_fix(&workspace)?;
    for index in 0..12 {
        let relative_path = format!("src/generated_{index}.rs");
        fs::write(
            workspace.join(&relative_path),
            format!("pub const VALUE_{index}: i32 = {index};\n"),
        )
        .map_err(|error| error.to_string())?;
    }

    let _status_report = execute_status(Some(&workspace)).map_err(|error| error.to_string())?;
    let session = load_session(&workspace)?;
    let projection = load_projection(&session)?;
    let finding = stale_finding(&projection)?;

    if finding.changed_paths.len() != 11 {
        return Err(format!(
            "expected 10 changed paths plus truncation marker, got {:?}",
            finding.changed_paths
        ));
    }
    if finding.changed_paths.last().map(String::as_str) != Some(TRUNCATION_MARKER) {
        return Err(format!(
            "expected truncation marker at the end of changed paths, got {:?}",
            finding.changed_paths
        ));
    }

    Ok(())
}

fn prove_add_fix(workspace: &std::path::Path) -> Result<(), String> {
    execute_goal(Some(workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .map_err(|error| error.to_string())?;
    execute_plan(Some(workspace), Some("bug-fix"), false).map_err(|error| error.to_string())?;

    let run = execute_run(Some(workspace)).map_err(|error| error.to_string())?;
    if run.exit_status != CommandExitStatus::Succeeded {
        return Err(format!(
            "expected run to succeed after executing fresh proof, got {:?}",
            run.exit_status
        ));
    }

    Ok(())
}

fn mark_docs_relevant(workspace: &std::path::Path) -> Result<(), String> {
    let store = FileSessionStore::for_workspace(workspace);
    let mut session = load_session(workspace)?;
    let task = session.active_task.as_mut().ok_or_else(|| "missing active task".to_string())?;
    let mut selection = task
        .context
        .completion_proof_selection()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "missing proof selection".to_string())?;
    selection.documentation_relevant = true;
    task.context.set_completion_proof_selection(&selection).map_err(|error| error.to_string())?;
    store.persist(&session).map(|_| ()).map_err(|error| error.to_string())
}

fn write_doc_file(workspace: &std::path::Path, contents: &str) -> Result<(), String> {
    let target = workspace.join(DOC_PATH);
    let parent = target.parent().ok_or_else(|| "missing docs parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    fs::write(target, contents).map_err(|error| error.to_string())
}

fn load_session(
    workspace: &std::path::Path,
) -> Result<boundline::domain::session::ActiveSessionRecord, String> {
    FileSessionStore::for_workspace(workspace)
        .load()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "missing active session".to_string())
}

fn load_projection(
    session: &boundline::domain::session::ActiveSessionRecord,
) -> Result<CompletionVerificationProjection, String> {
    session
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_verification_projection().ok().flatten())
        .ok_or_else(|| "missing completion verification projection".to_string())
}

fn stale_finding(
    projection: &CompletionVerificationProjection,
) -> Result<&CompletionVerificationFinding, String> {
    projection
        .completion_verification_findings
        .iter()
        .find(|finding| finding.kind == CompletionVerificationFindingKind::StaleProof)
        .ok_or_else(|| "missing stale proof finding".to_string())
}
