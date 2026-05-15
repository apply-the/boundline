use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{execute_capture, execute_plan, execute_start};

use crate::workspace_fixture::temp_fixture_workspace;

#[test]
fn plan_persists_guidance_resolution_with_workspace_precedence_and_local_only_disclosure() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-resolution");
    fs::create_dir_all(workspace.join(".boundline/guidance")).unwrap();
    fs::write(
        workspace.join(".boundline/guidance/clean-code.md"),
        "# Workspace Clean Code\nPrefer the local bounded rule set.\n",
    )
    .unwrap();

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test with the local clean code rule"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let plan_report = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should be persisted");

    assert!(
        plan.guidance_guardian
            .loaded_guidance_sources
            .iter()
            .any(|source| source == ".boundline/guidance/clean-code.md")
    );
    assert!(
        plan.guidance_guardian
            .loaded_guidance_sources
            .iter()
            .any(|source| source.starts_with("assistant/packs/"))
    );
    assert!(plan.guidance_guardian.skipped_guidance_sources.iter().any(|source| {
        source.contains("assistant/packs/engineering-foundations.toml")
            && source.contains("shadowed")
    }));
    assert!(
        plan.guidance_guardian
            .loaded_packs
            .iter()
            .any(|pack| { pack.contains("assistant/packs/guidance-catalog") })
    );
    assert!(plan.guidance_guardian.catalog_validation_findings.is_empty());
    assert!(
        plan.guidance_guardian
            .loaded_guidance_sources
            .iter()
            .any(|source| { source == "assistant/packs/guidance-catalog" })
    );
    assert!(
        plan.guidance_guardian
            .skipped_guidance_sources
            .iter()
            .any(|source| source.contains(".canon/boundline/guidance"))
    );
    assert!(
        plan_report.terminal_output.contains("guidance_resolution_summary: resolved"),
        "{}",
        plan_report.terminal_output
    );
    assert!(
        plan_report.terminal_output.contains("loaded_packs: assistant/packs/guidance-catalog"),
        "{}",
        plan_report.terminal_output
    );
    assert!(
        plan.guidance_guardian
            .capability_resolution_summary
            .as_deref()
            .is_some_and(|summary| summary.starts_with("resolved "))
    );
}

#[test]
fn plan_reports_invalid_workspace_guardian_override_as_skipped_source() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-invalid-guardian");
    fs::create_dir_all(workspace.join(".boundline/guardians")).unwrap();
    fs::write(
        workspace.join(".boundline/guardians/invalid.toml"),
        "[guardians.invalid\nkind = \"deterministic\"\n",
    )
    .unwrap();

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test with explicit guardian provenance"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let plan_report = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should be persisted");

    assert!(plan.guidance_guardian.skipped_guardian_sources.iter().any(|source| {
        source.contains(".boundline/guardians/invalid.toml")
            && source.contains("failed to parse workspace guardian override")
    }));
    assert!(
        plan_report.terminal_output.contains("skipped_guardian_sources:"),
        "{}",
        plan_report.terminal_output
    );
    assert!(
        plan_report.terminal_output.contains(".boundline/guardians/invalid.toml"),
        "{}",
        plan_report.terminal_output
    );
}
