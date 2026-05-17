use std::fs;
use std::path::PathBuf;

use boundline::cli::session::{execute_capture, execute_plan, execute_start, execute_status};

use crate::workspace_fixture::temp_fixture_workspace;

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
    let workspace =
        write_semantic_fallback_workspace("boundline-context-intelligence-semantic-fallback");

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

    for output in [plan.terminal_output.as_str(), status.terminal_output.as_str()] {
        assert!(output.contains("semantic_policy_state: local"), "{output}");
        assert!(output.contains("semantic_capability_state: unavailable"), "{output}");
        assert!(output.contains("hybrid_outcome: skipped"), "{output}");
        assert!(
            output.contains(
                "semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval"
            ),
            "{output}"
        );
    }
}
