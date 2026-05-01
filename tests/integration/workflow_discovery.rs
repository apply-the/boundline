use crate::workspace_fixture::{
    temp_invalid_workflow_layer_workspace, temp_workflow_discovery_workspace, terminal_text,
};

fn run_synod_in(workspace: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_synod"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

#[test]
fn workflow_list_surfaces_names_metadata_and_invocation_guidance() {
    let workspace = temp_workflow_discovery_workspace("workflow-discovery-list");

    let output = run_synod_in(&workspace, &["workflow", "list", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow registry status: ready"), "{text}");
    assert!(text.contains("workflow: governed-delivery"), "{text}");
    assert!(
        text.contains(
            "summary: bounded delivery path with review and governance before completion"
        ),
        "{text}"
    );
    assert!(
        text.contains("recommended_when: the task needs explicit review and governance evidence"),
        "{text}"
    );
    assert!(
        text.contains("phases: capture -> plan -> run -> review -> govern -> inspect"),
        "{text}"
    );
    assert!(text.contains("workflow: quick-fix"), "{text}");
    assert!(
        text.contains("summary: bounded workflow covering capture -> plan -> run -> inspect"),
        "{text}"
    );
    assert!(text.contains("invoke_with: synod workflow run quick-fix --workspace "), "{text}");
}

#[test]
fn workflow_list_reports_invalid_registry_state_explicitly() {
    let workspace = temp_invalid_workflow_layer_workspace("workflow-discovery-invalid");

    let output = run_synod_in(&workspace, &["workflow", "list", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow registry status: invalid"), "{text}");
    assert!(text.contains("reason: workflow definitions could not be parsed"), "{text}");
    assert!(text.contains("next_command: synod workflow inspect --workspace "), "{text}");
}
