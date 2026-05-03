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
fn workflow_list_contract_exposes_available_workflows_and_invocation_guidance() {
    let workspace = temp_workflow_discovery_workspace("workflow-discovery-contract-ready");

    let output = run_boundline_in(&workspace, &["workflow", "list", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow registry status: ready"), "{text}");
    assert!(text.contains("workflow: governed-delivery"), "{text}");
    assert!(text.contains("summary:"), "{text}");
    assert!(
        text.contains("phases: capture -> plan -> run -> review -> govern -> inspect"),
        "{text}"
    );
    assert!(
        text.contains("invoke_with: boundline workflow run governed-delivery --workspace "),
        "{text}"
    );
    assert!(text.contains("primary Boundline workflow surface"), "{text}");
}

#[test]
fn workflow_list_contract_reports_invalid_registry_state_without_silence() {
    let workspace = temp_invalid_workflow_layer_workspace("workflow-discovery-contract-invalid");

    let output = run_boundline_in(&workspace, &["workflow", "list", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow registry status: invalid"), "{text}");
    assert!(text.contains("reason: workflow definitions could not be parsed"), "{text}");
}
