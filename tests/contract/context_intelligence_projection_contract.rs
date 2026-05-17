use std::fs;
use std::path::PathBuf;

use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{
    execute_capture, execute_plan, execute_run, execute_start, execute_status,
};

use crate::workspace_fixture::temp_fixture_workspace;

// Build a bounded Rust workspace that emits selected evidence and relationship
// lines through the session-facing status and inspect surfaces.
fn write_projection_workspace(prefix: &str) -> PathBuf {
    temp_fixture_workspace(prefix)
}

// Build a workspace with the advanced-context policy explicitly disabled so the
// contract can assert the degraded terminal reason across command surfaces.
fn write_disabled_policy_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_fixture_workspace(prefix);
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join(".boundline/config.toml"),
        concat!(
            "version = 1\n\n",
            "[routing.advanced_context]\n",
            "retrieval_mode = \"disabled\"\n",
            "remote_policy = \"blocked\"\n",
        ),
    )
    .unwrap();
    workspace
}

#[test]
fn advanced_context_projection_contract_surfaces_local_projection_lines() {
    let workspace = write_projection_workspace("boundline-context-projection-contract");

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
        assert!(output.contains("semantic_policy_state: disabled"), "{output}");
        assert!(output.contains("semantic_capability_state: unsupported"), "{output}");
        assert!(output.contains("hybrid_outcome: baseline_only"), "{output}");
        assert!(output.contains("selected_evidence_count:"), "{output}");
        assert!(output.contains("semantic_selected_count: 0"), "{output}");
        assert!(output.contains("semantic_rejected_count: 0"), "{output}");
        assert!(output.contains("origin=fts"), "{output}");
        assert!(output.contains("relationship: src/lib.rs [requires_evidence]"), "{output}");
        assert!(output.contains("impact_finding: tests/lib.rs [missing_test]"), "{output}");
    }
    assert!(
        inspect.terminal_output.contains("advanced_context=mode=local, remote_policy=local_only"),
        "{}",
        inspect.terminal_output
    );
}

#[test]
fn advanced_context_projection_contract_surfaces_disabled_policy_reason() {
    let workspace = write_disabled_policy_workspace("boundline-context-projection-disabled");

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
        assert!(output.contains("retrieval_mode: disabled"), "{output}");
        assert!(output.contains("retrieval_state: insufficient"), "{output}");
        assert!(output.contains("disabled by configuration"), "{output}");
    }
    assert!(
        inspect.terminal_output.contains("advanced_context=mode=disabled, remote_policy=blocked"),
        "{}",
        inspect.terminal_output
    );
}
