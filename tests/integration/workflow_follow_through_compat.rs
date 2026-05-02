use synod::adapters::session_store::{FileSessionStore, SessionStore};

use crate::workspace_fixture::{
    run_synod_in, temp_workflow_discovery_compat_workspace, temp_workflow_discovery_workspace,
    terminal_text,
};

#[test]
fn direct_session_native_commands_remain_available_with_discovery_enabled_workflows() {
    let workspace = temp_workflow_discovery_workspace("workflow-follow-through-compat-native");

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

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert!(session.workflow_progress.is_none());
}

#[test]
fn explicit_compatibility_run_remains_available_with_discovery_enabled_workflows() {
    let workspace =
        temp_workflow_discovery_compat_workspace("workflow-follow-through-compat-explicit");

    let run = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test", "--compatibility"],
    );
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: compatibility"), "{run_text}");
    assert!(run_text.contains("execution_path: fixture_compatibility"), "{run_text}");
    assert!(!run_text.contains("workflow:"), "{run_text}");
}
