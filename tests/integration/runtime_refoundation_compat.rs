use synod::adapters::session_store::{FileSessionStore, SessionStore};

use crate::runtime_refoundation::temp_runtime_refoundation_compat_workspace;
use crate::workspace_fixture::{run_synod_in, terminal_text};

#[test]
fn explicit_compatibility_run_is_visible_when_execution_profile_is_chosen_deliberately() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-refoundation-compat-run");

    let run = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "Fix the failing add test"],
    );
    let run_text = terminal_text(&run);

    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: compatibility"), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("execution_path: fixture_compatibility"), "{run_text}");
    assert!(!run_text.contains("decision "), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("execution_condition: terminal -"), "{inspect_text}");

    let next = run_synod_in(&workspace, &["next", "--workspace", "."]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("continuity_authority: compatibility_trace"), "{next_text}");
    assert!(next_text.contains("routing: compatibility (execution_profile)"), "{next_text}");
    assert!(next_text.contains("execution_condition: terminal -"), "{next_text}");
}

#[test]
fn native_session_run_wins_over_execution_profile_when_goal_plan_is_ready() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("runtime-refoundation-compat-native");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("routing: native (goal_plan)"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("flow_state: confirmed (bug-fix)"), "{status_text}");

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert!(session.goal_plan.is_some());
}
