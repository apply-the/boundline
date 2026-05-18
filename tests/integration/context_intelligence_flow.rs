use std::path::PathBuf;

use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{
    execute_capture, execute_plan, execute_run, execute_start, execute_status,
};

use crate::workspace_fixture::temp_fixture_workspace;

// Seed a bounded Rust workspace so the local advanced-context flow can surface
// selected evidence consistently across plan, status, and inspect.
fn write_flow_workspace(prefix: &str) -> PathBuf {
    temp_fixture_workspace(prefix)
}

#[test]
fn plan_status_and_inspect_surface_selected_local_evidence() {
    let workspace = write_flow_workspace("boundline-context-intelligence-flow");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add path"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();
    execute_run(Some(&workspace)).unwrap();
    let inspect = execute_inspect(None, Some(&workspace)).unwrap();

    for output in [plan.terminal_output.as_str(), status.terminal_output.as_str()] {
        assert!(output.contains("retrieval_mode: local"), "{output}");
        assert!(output.contains("selected_evidence_count:"), "{output}");
        assert!(output.contains("src/lib.rs"), "{output}");
        assert!(output.contains("tests/red_to_green.rs"), "{output}");
    }
    assert!(
        inspect.terminal_output.contains("advanced_context=mode=local, remote_policy=local_only"),
        "{}",
        inspect.terminal_output
    );
}

#[test]
fn s7_plan_status_and_inspect_surface_us2_cognitive_lenses_when_advanced_context_is_available() {
    let workspace = write_flow_workspace("boundline-context-intelligence-s7-us2");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add path"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();
    execute_run(Some(&workspace)).unwrap();
    let inspect = execute_inspect(None, Some(&workspace)).unwrap();

    for output in [
        plan.terminal_output.as_str(),
        status.terminal_output.as_str(),
        inspect.terminal_output.as_str(),
    ] {
        assert!(output.contains("assumptions_summary:"), "{output}");
        assert!(output.contains("hidden_impact_summary:"), "{output}");
        assert!(output.contains("challenge_strongest_objection:"), "{output}");
        assert!(output.contains("explain_plan_summary:"), "{output}");
    }
}
