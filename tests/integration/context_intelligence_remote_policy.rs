use std::fs;
use std::path::PathBuf;

use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};

use crate::workspace_fixture::temp_fixture_workspace;

// Seed a workspace with the baseline local retrieval policy explicitly disabled
// so the integration flow can prove no remote or degraded fallback retrieval
// occurs silently.
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
fn disabled_policy_surfaces_insufficient_state_without_remote_fallback() {
    let workspace = write_disabled_policy_workspace("boundline-context-intelligence-policy");

    execute_goal(Some(&workspace), Some("fix the failing add path"), &[], None, None, None, None)
        .unwrap();
    let plan = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    let status = execute_status(Some(&workspace)).unwrap();
    execute_run(Some(&workspace)).unwrap();
    let inspect = execute_inspect(None, Some(&workspace), None, false).unwrap();

    for output in [plan.terminal_output.as_str(), status.terminal_output.as_str()] {
        assert!(output.contains("retrieval_mode: disabled"), "{output}");
        assert!(output.contains("retrieval_state: insufficient"), "{output}");
        assert!(output.contains("disabled by configuration"), "{output}");
        assert!(!output.contains("retrieval_mode: remote"), "{output}");
    }
    assert!(
        inspect.terminal_output.contains("retrieval_mode: disabled"),
        "{}",
        inspect.terminal_output
    );
}
