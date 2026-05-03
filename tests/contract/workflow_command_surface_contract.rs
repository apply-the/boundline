use crate::workspace_fixture::{
    temp_invalid_workflow_layer_workspace, temp_workflow_layer_workspace, terminal_text,
};

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

#[test]
fn workflow_run_surfaces_named_workflow_and_native_route() {
    let workspace = temp_workflow_layer_workspace("workflow-command-surface");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: default"), "{text}");
    assert!(text.contains("workflow_phase: inspect"), "{text}");
    assert!(text.contains("route_owner: workflow"), "{text}");
    assert!(text.contains("route_config_projection:"), "{text}");
    assert!(
        text.contains("routing: native (goal_plan) - goal plan is ready for native execution"),
        "{text}"
    );
    assert!(text.contains("execution_condition: terminal - work completed successfully"), "{text}");
    assert!(text.contains("next_command: boundline workflow inspect"), "{text}");
}

#[test]
fn workflow_run_rejects_invalid_definitions_before_execution_starts() {
    let workspace = temp_invalid_workflow_layer_workspace("workflow-command-invalid");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "invalid-flow", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow: invalid-flow"), "{text}");
    assert!(text.contains("workflow_phase: blocked"), "{text}");
    assert!(text.contains("route_owner: workflow"), "{text}");
    assert!(
		text.contains(
			"routing: blocked (session_state) - workflow definition is not valid for session-native execution"
		),
		"{text}"
	);
    assert!(
        text.contains("execution_condition: blocked - workflow definitions could not be parsed"),
        "{text}"
    );
    assert!(text.contains("next_command: boundline workflow inspect"), "{text}");
}
