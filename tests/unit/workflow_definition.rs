use boundline::domain::workflow::{
    ConditionalWorkflowPhase, WorkflowAvailabilityState, WorkflowConditionKind, WorkflowDefinition,
    WorkflowDefinitionError, WorkflowGoalSource, WorkflowLifecycleState, WorkflowOutputPreferences,
    WorkflowPhase, WorkflowProgressState, WorkflowRegistry,
};

use crate::workspace_fixture::{
    temp_invalid_workflow_layer_workspace, temp_workflow_discovery_workspace,
    temp_workflow_layer_workspace,
};

#[test]
fn loads_valid_workflow_definition_from_workspace_file() {
    let workspace = temp_workflow_layer_workspace("workflow-definition-valid");

    let registry = WorkflowRegistry::load(&workspace.join(".boundline/workflows.toml")).unwrap();
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

    let error = WorkflowRegistry::load(&workspace.join(".boundline/workflows.toml")).unwrap_err();
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

#[test]
fn loads_optional_discovery_metadata_and_fallback_summary() {
    let workspace = temp_workflow_discovery_workspace("workflow-definition-discovery");

    let registry = WorkflowRegistry::load(&workspace.join(".boundline/workflows.toml")).unwrap();
    let entries = registry.discovery_entries(&workspace);

    let governed = entries.iter().find(|entry| entry.workflow_name == "governed-delivery").unwrap();
    assert_eq!(governed.availability_state, WorkflowAvailabilityState::Ready);
    assert_eq!(
        governed.summary,
        "bounded delivery path with review and governance before completion"
    );
    assert_eq!(
        governed.recommended_when.as_deref(),
        Some("the task needs explicit review and governance evidence")
    );

    let quick_fix = entries.iter().find(|entry| entry.workflow_name == "quick-fix").unwrap();
    assert_eq!(quick_fix.summary, "bounded workflow covering capture -> plan -> run -> inspect");
    assert!(quick_fix.recommended_when.is_none());
    assert!(quick_fix.invocation_command.contains("boundline workflow run quick-fix --workspace "));
}

#[test]
fn workflow_text_helpers_cover_display_and_summary_fallbacks() {
    assert_eq!(WorkflowGoalSource::Session.to_string(), "session");

    for (phase, expected) in [
        (WorkflowPhase::Capture, "capture"),
        (WorkflowPhase::Clarify, "clarify"),
        (WorkflowPhase::Plan, "plan"),
        (WorkflowPhase::Run, "run"),
        (WorkflowPhase::Review, "review"),
        (WorkflowPhase::Govern, "govern"),
        (WorkflowPhase::Inspect, "inspect"),
    ] {
        assert_eq!(phase.to_string(), expected);
    }

    for (condition, expected) in [
        (WorkflowConditionKind::MissingAuthoredInput, "missing_authored_input"),
        (WorkflowConditionKind::ReviewTriggered, "review_triggered"),
        (WorkflowConditionKind::GovernanceRequired, "governance_required"),
    ] {
        assert_eq!(condition.to_string(), expected);
    }

    for (state, expected) in [
        (WorkflowLifecycleState::Idle, "idle"),
        (WorkflowLifecycleState::Active, "active"),
        (WorkflowLifecycleState::Paused, "paused"),
        (WorkflowLifecycleState::Blocked, "blocked"),
        (WorkflowLifecycleState::Completed, "completed"),
        (WorkflowLifecycleState::Failed, "failed"),
    ] {
        assert_eq!(state.to_string(), expected);
    }

    let workflow = WorkflowDefinition {
        workflow_name: "guided".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Capture,
        phases: vec![
            WorkflowPhase::Capture,
            WorkflowPhase::Plan,
            WorkflowPhase::Run,
            WorkflowPhase::Inspect,
        ],
        allow_review: false,
        allow_governance: false,
        conditional_phases: Vec::new(),
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    };

    assert_eq!(
        workflow.discovery_summary(),
        "bounded workflow covering capture -> plan -> run -> inspect"
    );
    assert_eq!(workflow.phase_chain_text(), "capture -> plan -> run -> inspect");
}

#[test]
fn workflow_validation_reports_shape_errors_and_duplicate_progress() {
    let missing_name = WorkflowDefinition {
        workflow_name: " ".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Capture,
        phases: vec![WorkflowPhase::Capture],
        allow_review: false,
        allow_governance: false,
        conditional_phases: Vec::new(),
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(missing_name, WorkflowDefinitionError::MissingWorkflowName));

    let missing_phases = WorkflowDefinition {
        workflow_name: "guided".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Capture,
        phases: Vec::new(),
        allow_review: false,
        allow_governance: false,
        conditional_phases: Vec::new(),
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(missing_phases, WorkflowDefinitionError::MissingPhases { .. }));

    let wrong_entry = WorkflowDefinition {
        workflow_name: "guided".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Plan,
        phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
        allow_review: false,
        allow_governance: false,
        conditional_phases: Vec::new(),
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(wrong_entry, WorkflowDefinitionError::EntryPhaseMustBeFirst { .. }));

    let duplicate_phase = WorkflowDefinition {
        workflow_name: "guided".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Capture,
        phases: vec![WorkflowPhase::Capture, WorkflowPhase::Capture],
        allow_review: false,
        allow_governance: false,
        conditional_phases: Vec::new(),
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(duplicate_phase, WorkflowDefinitionError::DuplicatePhase { .. }));

    let progress_error = WorkflowProgressState {
        workflow_name: "guided".to_string(),
        lifecycle_state: WorkflowLifecycleState::Active,
        current_phase: Some(WorkflowPhase::Run),
        completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Capture],
        blocked_reason: None,
        next_action: Some("boundline step".to_string()),
        routing_summary: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(
        progress_error,
        WorkflowDefinitionError::DuplicateCompletedPhase { phase: WorkflowPhase::Capture, .. }
    ));
}

#[test]
fn workflow_registry_covers_missing_definitions_and_clarify_condition_paths() {
    let empty_error = WorkflowRegistry::from_toml_str("[workflow]\n").unwrap_err();
    assert!(matches!(empty_error, WorkflowDefinitionError::MissingWorkflowDefinitions));

    let registry = WorkflowRegistry::from_toml_str(concat!(
        "[workflow.guided]\n",
        "goal_source = \"session\"\n",
        "entry = \"capture\"\n",
        "phases = [\"capture\", \"clarify\", \"inspect\"]\n",
        "allow_review = false\n",
        "allow_governance = false\n\n",
        "[workflow.guided.when]\n",
        "clarify = \"missing_authored_input\"\n",
    ))
    .unwrap();
    let workflow = registry.workflow("guided").unwrap();
    assert_eq!(workflow.conditional_phases.len(), 1);
    assert_eq!(workflow.conditional_phases[0].phase, WorkflowPhase::Clarify);
    assert_eq!(
        workflow.conditional_phases[0].condition_kind,
        WorkflowConditionKind::MissingAuthoredInput
    );
    assert!(workflow.conditional_phases[0].enabled);

    let unsupported = WorkflowDefinition {
        workflow_name: "guided".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Capture,
        phases: vec![WorkflowPhase::Capture, WorkflowPhase::Inspect],
        allow_review: false,
        allow_governance: false,
        conditional_phases: vec![ConditionalWorkflowPhase {
            phase: WorkflowPhase::Inspect,
            condition_kind: WorkflowConditionKind::ReviewTriggered,
            enabled: true,
        }],
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(unsupported, WorkflowDefinitionError::UnsupportedConditionalPhase { .. }));

    let unexpected = WorkflowDefinition {
        workflow_name: "guided".to_string(),
        goal_source: WorkflowGoalSource::Session,
        entry_phase: WorkflowPhase::Capture,
        phases: vec![WorkflowPhase::Capture, WorkflowPhase::Clarify],
        allow_review: false,
        allow_governance: false,
        conditional_phases: vec![ConditionalWorkflowPhase {
            phase: WorkflowPhase::Clarify,
            condition_kind: WorkflowConditionKind::ReviewTriggered,
            enabled: true,
        }],
        output_preferences: WorkflowOutputPreferences::default(),
        summary: None,
        recommended_when: None,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(unexpected, WorkflowDefinitionError::UnexpectedConditionKind { .. }));
}

#[test]
fn conditional_phase_enabled_defaults_to_true_when_omitted() {
    let registry = WorkflowRegistry::from_toml_str(concat!(
        "[workflow.guided]\n",
        "goal_source = \"session\"\n",
        "entry = \"capture\"\n",
        "phases = [\"capture\", \"clarify\", \"inspect\"]\n",
        "allow_review = false\n",
        "allow_governance = false\n\n",
        "[workflow.guided.when]\n",
        "clarify = \"missing_authored_input\"\n",
    ))
    .unwrap();

    let workflow = registry.workflow("guided").unwrap();
    assert_eq!(workflow.conditional_phases.len(), 1);
    assert!(workflow.conditional_phases[0].enabled);
}
