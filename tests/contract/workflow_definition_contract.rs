use crate::workspace_fixture::{
    run_boundline_in, temp_workflow_layer_workspace, terminal_text, write_workflow_definitions,
};

#[test]
fn workflow_run_rejects_review_phase_when_review_is_not_allowed() {
    let workspace = temp_workflow_layer_workspace("workflow-definition-contract-review");
    write_workflow_definitions(
        &workspace,
        concat!(
            "[workflow.default]\n",
            "goal_source = \"session\"\n",
            "entry = \"capture\"\n",
            "phases = [\"capture\", \"review\", \"inspect\"]\n",
            "allow_review = false\n",
            "allow_governance = false\n",
        ),
    );

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow: default"), "{text}");
    assert!(text.contains("workflow_phase: blocked"), "{text}");
    assert!(
		text.contains(
			"execution_condition: blocked - workflow `default` includes `review` but allow_review is false"
		),
		"{text}"
	);
}

#[test]
fn workflow_run_rejects_govern_phase_when_governance_is_not_allowed() {
    let workspace = temp_workflow_layer_workspace("workflow-definition-contract-govern");
    write_workflow_definitions(
        &workspace,
        concat!(
            "[workflow.default]\n",
            "goal_source = \"session\"\n",
            "entry = \"capture\"\n",
            "phases = [\"capture\", \"govern\", \"inspect\"]\n",
            "allow_review = false\n",
            "allow_governance = false\n",
        ),
    );

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow: default"), "{text}");
    assert!(text.contains("workflow_phase: blocked"), "{text}");
    assert!(
		text.contains(
			"execution_condition: blocked - workflow `default` includes `govern` but allow_governance is false"
		),
		"{text}"
	);
}
