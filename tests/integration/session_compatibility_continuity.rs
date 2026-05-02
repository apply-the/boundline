use synod::adapters::session_store::{FileSessionStore, SessionStore};

use crate::runtime_refoundation::temp_runtime_refoundation_compat_workspace;
use crate::workspace_fixture::{run_synod_in, terminal_text};

#[test]
fn status_surfaces_native_session_and_compatibility_follow_up_without_replacing_session_state() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("session-compatibility-continuity-mixed-route");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));

    let run = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test", "--compatibility"],
    );
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_synod_in(&workspace, &["status", "--workspace", "."]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("routing: native (goal_plan)"), "{status_text}");
    assert!(status_text.contains("continuity_authority: native_session"), "{status_text}");
    assert!(status_text.contains("compatibility_follow_up: inspect_only"), "{status_text}");
    assert!(
        status_text.contains("compatibility_routing: compatibility (execution_profile)"),
        "{status_text}"
    );
    assert!(
        status_text.contains("compatibility_follow_up_command: synod inspect --workspace "),
        "{status_text}"
    );

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert!(session.goal_plan.is_some());
    assert!(session.active_task.is_none());
}

#[test]
fn next_without_active_session_recommends_workspace_inspect_for_latest_compatibility_trace() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("session-compatibility-continuity-no-session");

    let run = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test", "--compatibility"],
    );
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let next = run_synod_in(&workspace, &["next", "--workspace", "."]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("continuity_authority: compatibility_trace"), "{next_text}");
    assert!(next_text.contains("compatibility_follow_up: inspect_only"), "{next_text}");
    assert!(next_text.contains("next_command: synod inspect --workspace "), "{next_text}");
}
