use std::fs;
use std::path::PathBuf;

use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};

use crate::workspace_fixture::{
    SEMANTIC_VECTOR_STATE_READY_VALUE, force_semantic_vector_state_override, temp_empty_workspace,
};

fn write_semantic_inspect_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_empty_workspace(prefix);
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"semantic_inspect_fixture\"\n",
            "version = \"0.1.0\"\n",
            "edition = \"2024\"\n",
        ),
    )
    .unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        concat!("pub fn planner() -> bool {\n", "    true\n", "}\n",),
    )
    .unwrap();
    fs::write(
        workspace.join("src/semantic.rs"),
        "pub fn reconcileConfigState() -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("src/alternate.rs"),
        "pub fn reconcilePlanningConfiguration() -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("tests/basic.rs"),
        concat!(
            "use semantic_inspect_fixture::planner;\n\n",
            "#[test]\n",
            "fn planner_is_true() {\n",
            "    assert!(planner());\n",
            "}\n",
        ),
    )
    .unwrap();
    fs::write(
        workspace.join(".boundline/config.toml"),
        concat!(
            "version = 1\n\n",
            "[routing.advanced_context]\n",
            "retrieval_mode = \"local\"\n",
            "remote_policy = \"local_only\"\n\n",
            "[routing.advanced_context.budgets]\n",
            "refinement_budget = 2\n",
            "refresh_budget = 1\n",
            "depth_limit = 12\n",
            "expansion_limit = 4\n",
            "traversal_limit = 8\n",
            "evidence_limit = 2\n\n",
            "[routing.semantic_acceleration]\n",
            "policy = \"local\"\n",
        ),
    )
    .unwrap();
    workspace
}

#[test]
fn status_and_inspect_surface_semantic_explanation_lines() {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace =
        write_semantic_inspect_workspace("boundline-context-intelligence-semantic-inspect");

    execute_goal(
        Some(&workspace),
        Some("planner reconcile configuration state"),
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
    let inspect = execute_inspect(None, Some(&workspace), None).unwrap();

    for output in [status.terminal_output.as_str(), inspect.terminal_output.as_str()] {
        assert!(output.contains("semantic_policy_state: local"), "{output}");
        assert!(output.contains("semantic_capability_state: ready"), "{output}");
        assert!(output.contains("retrieval_mode: local"), "{output}");
        assert!(output.contains("hybrid_outcome:"), "{output}");
        assert!(output.contains("selected_evidence_count:"), "{output}");
        assert!(output.contains("semantic_rejected_count:"), "{output}");
        assert!(output.contains("origin=semantic_expand"), "{output}");
        assert!(output.contains("rejected_candidate:"), "{output}");
    }
}
