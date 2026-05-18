use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use boundline::cli::session::{execute_capture, execute_plan, execute_start, execute_status};

use crate::workspace_fixture::temp_fixture_workspace;

const SEMANTIC_VECTOR_STATE_OVERRIDE_ENV: &str = "BOUNDLINE_SEMANTIC_VECTOR_STATE_OVERRIDE";
const SEMANTIC_VECTOR_STATE_MISSING_VALUE: &str = "missing";

static SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK: Mutex<()> = Mutex::new(());

struct EnvVarGuard {
    name: &'static str,
    previous: Option<OsString>,
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            unsafe {
                std::env::set_var(self.name, previous);
            }
        } else {
            unsafe {
                std::env::remove_var(self.name);
            }
        }
    }
}

fn set_env_var(name: &'static str, value: &str) -> EnvVarGuard {
    let previous = std::env::var_os(name);
    unsafe {
        std::env::set_var(name, value);
    }
    EnvVarGuard { name, previous }
}

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
    let _guard =
        SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard =
        set_env_var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, SEMANTIC_VECTOR_STATE_MISSING_VALUE);
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

#[test]
fn s7_plan_and_status_surface_hidden_impact_fallback_when_semantic_capability_is_unavailable() {
    let _guard =
        SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard =
        set_env_var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, SEMANTIC_VECTOR_STATE_MISSING_VALUE);
    let workspace =
        write_semantic_fallback_workspace("boundline-context-intelligence-s7-semantic-fallback");

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
        assert!(output.contains("hidden_impact_fallback_disclosure:"), "{output}");
        assert!(output.contains("higher-order impact inference is unavailable"), "{output}");
        assert!(output.contains("challenge_strongest_objection:"), "{output}");
    }
}
