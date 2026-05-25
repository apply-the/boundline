use crate::workspace_fixture::{
    temp_invalid_workflow_layer_workspace, temp_workflow_discovery_workspace, terminal_text,
};

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

#[test]
fn workflow_list_surfaces_names_metadata_and_invocation_guidance() {
    let workspace = temp_workflow_discovery_workspace("workflow-discovery-list");

    let output = run_boundline_in(&workspace, &["workflow", "list", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow registry status: ready"), "{text}");
    assert!(text.contains("delivery_path_count: 1"), "{text}");
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
    assert!(text.contains("invoke_with: boundline workflow run quick-fix --workspace "), "{text}");
    assert!(text.contains("delivery_path: idea_to_code"), "{text}");
    assert!(
        text.contains(
            "description: Move from idea intake to verified code through bounded stages."
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "stages: discovery -> requirements -> system-shaping -> architecture -> backlog -> implementation -> verification -> pr-review"
        ),
        "{text}"
    );
    assert!(text.contains("adaptive: true"), "{text}");
}

#[test]
fn workflow_list_reports_invalid_registry_state_explicitly() {
    let workspace = temp_invalid_workflow_layer_workspace("workflow-discovery-invalid");

    let output = run_boundline_in(&workspace, &["workflow", "list", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow registry status: invalid"), "{text}");
    assert!(text.contains("reason: workflow definitions could not be parsed"), "{text}");
    assert!(text.contains("next_command: boundline workflow inspect --workspace "), "{text}");
}
