use std::fs;
use std::path::PathBuf;

use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};

use crate::workspace_fixture::temp_empty_workspace;

const FIXTURE_CARGO_TOML: &str = concat!(
    "[package]\n",
    "name = \"boundline-context-intelligence-impact\"\n",
    "version = \"0.1.0\"\n",
    "edition = \"2024\"\n",
);

// Seed a workspace with an implementation file but no matching test so the
// impact projection can surface the evidence gap end to end.
fn write_impact_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_empty_workspace(prefix);
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/engine.rs"), "pub fn reconcile_plan() -> bool { true }\n")
        .unwrap();
    workspace
}

#[test]
fn status_and_inspect_surface_missing_test_impact_findings() {
    let workspace = write_impact_workspace("boundline-context-intelligence-impact");

    execute_goal(
        Some(&workspace),
        Some("reconcile src/engine.rs and add the missing focused test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();
    execute_run(Some(&workspace)).unwrap();
    let inspect = execute_inspect(None, Some(&workspace), None, false).unwrap();

    let status_output = status.terminal_output.as_str();
    assert!(status_output.contains("impact_finding_count: 1"), "{status_output}");
    assert!(
        status_output.contains("relationship: src/engine.rs [requires_evidence]"),
        "{status_output}"
    );
    assert!(
        status_output.contains("impact_finding: tests/engine.rs [missing_test]"),
        "{status_output}"
    );
    assert!(
        inspect.terminal_output.contains("advanced_context=mode=local, remote_policy=local_only"),
        "{}",
        inspect.terminal_output
    );
}
