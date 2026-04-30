use synod::domain::workflow::{WorkflowDefinitionError, WorkflowPhase, WorkflowRegistry};

use crate::workspace_fixture::{
    temp_invalid_workflow_layer_workspace, temp_workflow_layer_workspace,
};

#[test]
fn loads_valid_workflow_definition_from_workspace_file() {
    let workspace = temp_workflow_layer_workspace("workflow-definition-valid");

    let registry = WorkflowRegistry::load(&workspace.join(".synod/workflows.toml")).unwrap();
    assert_eq!(registry.workflow_names(), vec!["default"]);

    let workflow = registry.workflow("default").unwrap();
    assert_eq!(workflow.workflow_name, "default");
    assert_eq!(workflow.entry_phase, WorkflowPhase::Capture);
    assert_eq!(
        workflow.phases,
        vec![
            WorkflowPhase::Capture,
            WorkflowPhase::Plan,
            WorkflowPhase::Run,
            WorkflowPhase::Inspect,
        ]
    );
    assert!(workflow.output_preferences.next_command);
    assert!(workflow.output_preferences.routing_summary);
    assert!(workflow.output_preferences.execution_condition);
}

#[test]
fn rejects_unknown_phase_while_parsing_workflow_file() {
    let workspace = temp_invalid_workflow_layer_workspace("workflow-definition-invalid-phase");

    let error = WorkflowRegistry::load(&workspace.join(".synod/workflows.toml")).unwrap_err();
    assert!(matches!(error, WorkflowDefinitionError::ParseWorkflowDefinitions(_)));
}

#[test]
fn rejects_conditional_phase_that_is_not_declared() {
    let error = WorkflowRegistry::from_toml_str(concat!(
        "[workflow.default]\n",
        "goal_source = \"session\"\n",
        "entry = \"capture\"\n",
        "phases = [\"capture\", \"plan\", \"run\", \"inspect\"]\n",
        "allow_review = true\n",
        "allow_governance = false\n\n",
        "[workflow.default.when]\n",
        "review = \"review_triggered\"\n",
    ))
    .unwrap_err();

    assert!(matches!(
        error,
        WorkflowDefinitionError::ConditionalPhaseMissing { phase: WorkflowPhase::Review, .. }
    ));
}

#[test]
fn rejects_review_phase_when_review_is_not_allowed() {
    let error = WorkflowRegistry::from_toml_str(concat!(
        "[workflow.default]\n",
        "goal_source = \"session\"\n",
        "entry = \"capture\"\n",
        "phases = [\"capture\", \"review\", \"inspect\"]\n",
        "allow_review = false\n",
        "allow_governance = false\n",
    ))
    .unwrap_err();

    assert!(matches!(error, WorkflowDefinitionError::ReviewPhaseNotAllowed { .. }));
}

#[test]
fn rejects_govern_phase_when_governance_is_not_allowed() {
    let error = WorkflowRegistry::from_toml_str(concat!(
        "[workflow.default]\n",
        "goal_source = \"session\"\n",
        "entry = \"capture\"\n",
        "phases = [\"capture\", \"govern\", \"inspect\"]\n",
        "allow_review = false\n",
        "allow_governance = false\n",
    ))
    .unwrap_err();

    assert!(matches!(error, WorkflowDefinitionError::GovernancePhaseNotAllowed { .. }));
}
