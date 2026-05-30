use std::fs;
use std::path::PathBuf;

use boundline::cli::session::{execute_goal, execute_plan, execute_status};

use crate::workspace_fixture::{
    SEMANTIC_VECTOR_STATE_MISSING_VALUE, force_semantic_vector_state_override,
    temp_fixture_workspace,
};

const SEMANTIC_VECTOR_STATE_CORRUPT_VALUE: &str = "corrupt";

fn write_semantic_fallback_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_fixture_workspace(prefix);
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join(".boundline/config.toml"),
        concat!(
            "version = 1\n\n",
            "[routing.advanced_context]\n",
            "retrieval_mode = \"local\"\n",
            "remote_policy = \"local_only\"\n\n",
            "[routing.semantic_acceleration]\n",
            "policy = \"local\"\n",
        ),
    )
    .unwrap();
    workspace
}

#[test]
fn plan_status_and_inspect_surface_explicit_semantic_fallback_when_local_capability_is_unavailable()
{
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_MISSING_VALUE);
    let workspace =
        write_semantic_fallback_workspace("boundline-context-intelligence-semantic-fallback");

    execute_goal(Some(&workspace), Some("fix the failing add path"), &[], None, None, None, None)
        .unwrap();
    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();

    for output in [plan.terminal_output.as_str(), status.terminal_output.as_str()] {
        assert!(output.contains("semantic_policy_state: local"), "{output}");
        assert!(output.contains("semantic_capability_state: missing"), "{output}");
        assert!(output.contains("semantic_engine: baseline_json"), "{output}");
        assert!(output.contains("hybrid_outcome: skipped"), "{output}");
        assert!(output.contains("vector_query_count: 0"), "{output}");
        assert!(output.contains("vector_candidates_returned: 0"), "{output}");
        assert!(output.contains("semantic_fallback_reason:"), "{output}");
        assert!(
            output.contains(
                "semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval"
            ),
            "{output}"
        );
    }
}

#[test]
fn plan_and_status_surface_hidden_impact_fallback_when_semantic_capability_is_unavailable() {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_MISSING_VALUE);
    let workspace =
        write_semantic_fallback_workspace("boundline-context-intelligence-semantic-fallback");

    execute_goal(Some(&workspace), Some("fix the failing add path"), &[], None, None, None, None)
        .unwrap();
    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();

    for output in [plan.terminal_output.as_str(), status.terminal_output.as_str()] {
        assert!(output.contains("hidden_impact_fallback_disclosure:"), "{output}");
        assert!(output.contains("higher-order impact inference is unavailable"), "{output}");
        assert!(output.contains("challenge_strongest_objection:"), "{output}");
    }
}

#[test]
fn plan_and_status_surface_explicit_corrupt_semantic_fallback_state() {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_CORRUPT_VALUE);
    let workspace =
        write_semantic_fallback_workspace("boundline-context-intelligence-semantic-corrupt");

    execute_goal(Some(&workspace), Some("fix the failing add path"), &[], None, None, None, None)
        .unwrap();
    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();

    for output in [plan.terminal_output.as_str(), status.terminal_output.as_str()] {
        assert!(output.contains("semantic_policy_state: local"), "{output}");
        assert!(output.contains("semantic_capability_state: corrupt"), "{output}");
        assert!(output.contains("semantic_engine: baseline_json"), "{output}");
        assert!(output.contains("semantic_fallback_reason:"), "{output}");
        assert!(output.contains("sqlite-vec state is corrupt"), "{output}");
    }
}
