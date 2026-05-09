use serde_json::Value;

use crate::workspace_fixture::{
    run_boundline_in, stdout_json, temp_fixture_workspace, terminal_text,
};

#[test]
fn structured_session_output_preserves_continuation_state_across_start_capture_plan_status_and_next()
 {
    let workspace = temp_fixture_workspace("boundline-host-session-runtime");

    let start = run_boundline_in(&workspace, &["start", "--json"]);
    let start_text = terminal_text(&start);
    assert_eq!(start.status.code(), Some(0), "{start_text}");
    let start_json: Value = stdout_json(&start);
    assert_eq!(start_json["session_status"]["latest_status"], "initialized", "{start_text}");

    let capture =
        run_boundline_in(&workspace, &["capture", "--goal", "Fix the failing add test", "--json"]);
    let capture_text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{capture_text}");
    let capture_json: Value = stdout_json(&capture);
    assert_eq!(
        capture_json["session_status"]["goal"], "Fix the failing add test",
        "{capture_text}"
    );

    let plan = run_boundline_in(&workspace, &["plan", "--flow", "bug-fix", "--json"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    let plan_json: Value = stdout_json(&plan);
    assert_eq!(plan_json["session_status"]["latest_status"], "planned", "{plan_text}");
    assert_eq!(plan_json["session_status"]["goal_plan_state"], "confirmed", "{plan_text}");
    assert!(
        plan_json["session_status"]["flow_state"].as_str().unwrap_or_default().contains("bug-fix"),
        "{plan_text}"
    );

    let status = run_boundline_in(&workspace, &["status", "--json"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    let status_json: Value = stdout_json(&status);
    assert_eq!(status_json["session_status"]["latest_status"], "planned", "{status_text}");
    assert!(status_json["session_status"]["next_command"].is_string(), "{status_text}");
    assert!(
        status_json["rendered_output"].as_str().unwrap_or_default().contains("next_command:"),
        "{status_text}"
    );

    let next = run_boundline_in(&workspace, &["next", "--json"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    let next_json: Value = stdout_json(&next);
    assert_eq!(
        next_json["session_status"]["next_command"], status_json["session_status"]["next_command"],
        "{next_text}"
    );
}
