use boundline::{
    ProjectScaleBoundaryRequest, ProjectScaleInput, ProjectScalePathKind, ProjectScaleStageKind,
    evaluate_project_scale_boundary, propose_project_scale_path,
};

#[test]
fn broad_unclear_idea_proposes_idea_to_code_path() {
    let path = propose_project_scale_path(ProjectScaleInput {
        goal: "Build a customer onboarding capability with audit logging".to_string(),
        problem_unclear: true,
        product_scope_unclear: true,
        capability_structure_unclear: true,
        architecture_material: true,
        existing_system_change: false,
        operational_entry: None,
    });

    assert_eq!(path.kind, ProjectScalePathKind::IdeaToCode);
    assert_eq!(
        path.stages.iter().map(|stage| stage.kind).collect::<Vec<_>>(),
        vec![
            ProjectScaleStageKind::Discovery,
            ProjectScaleStageKind::Requirements,
            ProjectScaleStageKind::SystemShaping,
            ProjectScaleStageKind::Architecture,
            ProjectScaleStageKind::Backlog,
            ProjectScaleStageKind::Implementation,
            ProjectScaleStageKind::Verification,
            ProjectScaleStageKind::PrReview,
        ]
    );
    assert!(path.requires_confirmation);
    assert_eq!(path.next_action, "confirm_project_scale_path");
}

#[test]
fn operational_entry_routes_back_to_delivery_work() {
    let path = propose_project_scale_path(ProjectScaleInput {
        goal: "Assess supply-chain risk before dependency migration".to_string(),
        problem_unclear: false,
        product_scope_unclear: false,
        capability_structure_unclear: false,
        architecture_material: false,
        existing_system_change: true,
        operational_entry: Some(ProjectScaleStageKind::SupplyChainAnalysis),
    });

    assert_eq!(path.kind, ProjectScalePathKind::OperationalOrRisk);
    assert_eq!(path.stages[0].kind, ProjectScaleStageKind::SupplyChainAnalysis);
    assert!(path.stages.iter().any(|stage| stage.kind == ProjectScaleStageKind::Change));
    assert!(path.stages.iter().any(|stage| stage.kind == ProjectScaleStageKind::Verification));
    assert!(!path.unbounded_autonomy);
}

#[test]
fn stage_transition_requires_confirmation_when_boundary_changes() {
    let decision = evaluate_project_scale_boundary(ProjectScaleBoundaryRequest {
        active_stage: ProjectScaleStageKind::Architecture,
        requested_stage: ProjectScaleStageKind::Implementation,
        confirmed: false,
    });

    assert!(decision.blocked);
    assert_eq!(decision.next_action, "confirm_stage_transition");
    assert!(decision.reason.contains("exceeds current stage boundary"));

    let confirmed = evaluate_project_scale_boundary(ProjectScaleBoundaryRequest {
        active_stage: ProjectScaleStageKind::Architecture,
        requested_stage: ProjectScaleStageKind::Implementation,
        confirmed: true,
    });
    assert!(!confirmed.blocked);
    assert_eq!(confirmed.next_action, "continue_project_scale_stage");
}

#[test]
fn same_stage_boundary_and_existing_system_path_stay_bounded() {
    let same_stage = evaluate_project_scale_boundary(ProjectScaleBoundaryRequest {
        active_stage: ProjectScaleStageKind::Backlog,
        requested_stage: ProjectScaleStageKind::Backlog,
        confirmed: false,
    });

    assert!(!same_stage.blocked);
    assert_eq!(same_stage.next_action, "continue_project_scale_stage");
    assert!(same_stage.reason.contains("remains inside"));

    let path = propose_project_scale_path(ProjectScaleInput {
        goal: "Modify the existing onboarding auth flow".to_string(),
        problem_unclear: false,
        product_scope_unclear: false,
        capability_structure_unclear: false,
        architecture_material: false,
        existing_system_change: true,
        operational_entry: None,
    });

    assert_eq!(path.kind, ProjectScalePathKind::ExistingSystemChange);
    assert_eq!(
        path.stage_names(),
        "system-assessment -> change -> implementation -> verification -> pr-review"
    );
    assert_eq!(path.stages[0].kind, ProjectScaleStageKind::SystemAssessment);
    assert_eq!(path.stages[1].kind, ProjectScaleStageKind::Change);
}

#[test]
fn operational_entry_dedupes_repeated_change_stage() {
    let path = propose_project_scale_path(ProjectScaleInput {
        goal: "Route operational change back into delivery".to_string(),
        problem_unclear: false,
        product_scope_unclear: false,
        capability_structure_unclear: false,
        architecture_material: false,
        existing_system_change: false,
        operational_entry: Some(ProjectScaleStageKind::Change),
    });

    assert_eq!(path.kind, ProjectScalePathKind::OperationalOrRisk);
    assert_eq!(
        path.stages.iter().filter(|stage| stage.kind == ProjectScaleStageKind::Change).count(),
        1
    );
}

#[test]
fn project_scale_stage_kind_text_covers_late_stage_variants() {
    let late_stage_text = [
        (ProjectScaleStageKind::Refactor, "refactor"),
        (ProjectScaleStageKind::Review, "review"),
        (ProjectScaleStageKind::Verification, "verification"),
        (ProjectScaleStageKind::PrReview, "pr-review"),
        (ProjectScaleStageKind::Incident, "incident"),
        (ProjectScaleStageKind::SecurityAssessment, "security-assessment"),
        (ProjectScaleStageKind::SystemAssessment, "system-assessment"),
        (ProjectScaleStageKind::Migration, "migration"),
        (ProjectScaleStageKind::SupplyChainAnalysis, "supply-chain-analysis"),
    ];

    for (kind, expected) in late_stage_text {
        assert_eq!(kind.as_str(), expected);
    }
}
