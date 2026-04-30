use synod::adapters::session_store::{FileSessionStore, SessionStore};

use crate::workspace_fixture::{
    run_synod_in, temp_workflow_layer_compat_workspace, temp_workflow_layer_workspace,
    terminal_text,
};

#[test]
fn direct_session_native_commands_remain_available_without_workflow_invocation() {
    let workspace = temp_workflow_layer_workspace("workflow-layer-compat-native");

    assert_eq!(run_synod_in(&workspace, &["start", "--workspace", "."]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--workspace", ".", "--goal", "Fix the failing add test"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        run_synod_in(&workspace, &["plan", "--workspace", ".", "--flow", "bug-fix"]).status.code(),
        Some(0)
    );

    let run = run_synod_in(&workspace, &["run", "--workspace", "."]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(!run_text.contains("workflow:"), "{run_text}");

    let status = run_synod_in(&workspace, &["status", "--workspace", "."]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("routing: native (goal_plan)"), "{status_text}");
    assert!(!status_text.contains("workflow:"), "{status_text}");

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert!(session.workflow_progress.is_none());
}

#[test]
fn explicit_compatibility_run_remains_available_without_workflow_invocation() {
    let workspace = temp_workflow_layer_compat_workspace("workflow-layer-compat-explicit");

    let run = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test"],
    );
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: compatibility"), "{run_text}");
    assert!(run_text.contains("execution_path: fixture_compatibility"), "{run_text}");
    assert!(!run_text.contains("workflow:"), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
		inspect_text.contains(
			"routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
		),
		"{inspect_text}"
	);
}
