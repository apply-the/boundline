use boundline::domain::decision::{DecisionType, EvidenceRef};
use boundline::domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan, GoalPlanError,
    GoalPlanFlowMode, GoalPlanFlowState, GoalPlanStatus, InferredFlow, PlannedTask,
    WorkspaceSignals,
};

fn sample_task(id: &str) -> PlannedTask {
    PlannedTask {
        task_id: id.to_string(),
        description: format!("Implement {id}"),
        target: format!("src/{id}.rs"),
        expected_outcome: Some("compiles".to_string()),
        decision_type_hint: Some(DecisionType::Code),
    }
}

#[test]
fn new_goal_plan_is_draft_with_generated_id() {
    let plan = GoalPlan::new("Fix the login bug", vec![sample_task("t1")]).unwrap();
    assert!(!plan.plan_id.is_empty());
    assert_eq!(plan.status, GoalPlanStatus::Draft);
    assert_eq!(plan.proposal_revision, 1);
    assert!(plan.requires_confirmation());
    assert_eq!(plan.proposal_state_text(), "proposed");
    assert_eq!(plan.goal_text, "Fix the login bug");
    assert_eq!(plan.tasks.len(), 1);
}

#[test]
fn validation_rejects_empty_goal_text() {
    let err = GoalPlan::new("", vec![sample_task("t1")]).unwrap_err();
    assert!(matches!(err, GoalPlanError::MissingGoalText));
}

#[test]
fn validation_rejects_no_tasks() {
    let err = GoalPlan::new("Fix something", vec![]).unwrap_err();
    assert!(matches!(err, GoalPlanError::NoTasks));
}

#[test]
fn validation_rejects_task_with_empty_id() {
    let err = GoalPlan::new(
        "Fix something",
        vec![PlannedTask {
            task_id: String::new(),
            description: "d".to_string(),
            target: "t".to_string(),
            expected_outcome: None,
            decision_type_hint: None,
        }],
    )
    .unwrap_err();
    assert!(matches!(err, GoalPlanError::MissingTaskId));
}

#[test]
fn validation_rejects_task_with_empty_description() {
    let err = GoalPlan::new(
        "Fix something",
        vec![PlannedTask {
            task_id: "t1".to_string(),
            description: String::new(),
            target: "t".to_string(),
            expected_outcome: None,
            decision_type_hint: None,
        }],
    )
    .unwrap_err();
    assert!(matches!(err, GoalPlanError::MissingTaskDescription { .. }));
}

#[test]
fn validation_rejects_task_with_empty_target() {
    let err = GoalPlan::new(
        "Fix something",
        vec![PlannedTask {
            task_id: "t1".to_string(),
            description: "d".to_string(),
            target: String::new(),
            expected_outcome: None,
            decision_type_hint: None,
        }],
    )
    .unwrap_err();
    assert!(matches!(err, GoalPlanError::MissingTaskTarget { .. }));
}

#[test]
fn confirm_transitions_draft_to_confirmed() {
    let mut plan =
        GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap().with_flow(InferredFlow {
            flow_name: "bug-fix".to_string(),
            confidence_reason: "selected from evidence".to_string(),
            confirmed: false,
        });
    assert!(plan.confirm().is_ok());
    assert_eq!(plan.status, GoalPlanStatus::Confirmed);
    assert_eq!(plan.proposal_state_text(), "confirmed");
    assert!(!plan.requires_confirmation());
    assert!(plan.confirmed_at.is_some());
    assert_eq!(plan.flow.as_ref().map(|flow| flow.confirmed), Some(true));
}

#[test]
fn confirm_rejects_non_draft() {
    let mut plan = GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap();
    plan.confirm().unwrap();
    let err = plan.confirm().unwrap_err();
    assert!(matches!(
        err,
        GoalPlanError::InvalidTransition { from: GoalPlanStatus::Confirmed, .. }
    ));
}

#[test]
fn supersede_transitions_confirmed_to_superseded() {
    let mut plan = GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap();
    plan.confirm().unwrap();
    assert!(plan.supersede_with(2, "new evidence changed the target family").is_ok());
    assert_eq!(plan.status, GoalPlanStatus::Superseded);
    assert_eq!(plan.proposal_state_text(), "superseded");
    assert_eq!(plan.superseded_by_revision, Some(2));
    assert_eq!(plan.superseded_reason.as_deref(), Some("new evidence changed the target family"));
}

#[test]
fn supersede_rejects_draft() {
    let mut plan = GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap();
    let err = plan.supersede().unwrap_err();
    assert!(matches!(err, GoalPlanError::InvalidTransition { from: GoalPlanStatus::Draft, .. }));
}

#[test]
fn with_signals_sets_workspace_signals() {
    let signals = WorkspaceSignals {
        language: Some("rust".to_string()),
        file_count: 42,
        has_config: true,
        has_canon: false,
        has_tests: true,
    };
    let plan =
        GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap().with_signals(signals.clone());
    assert_eq!(plan.workspace_signals, signals);
}

#[test]
fn with_flow_sets_inferred_flow() {
    let flow = InferredFlow {
        flow_name: "bug-fix".to_string(),
        confidence_reason: "keyword 'fix'".to_string(),
        confirmed: false,
    };
    let plan = GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap().with_flow(flow.clone());
    assert_eq!(plan.flow, Some(flow));
}

#[test]
fn planning_rationale_and_verification_strategy_helpers_set_fields() {
    let plan = GoalPlan::new("Goal", vec![sample_task("t1")])
        .unwrap()
        .with_planning_rationale("selected src/lib.rs because context and trace evidence agree")
        .with_verification_strategy("run targeted arithmetic tests for src/lib.rs");

    assert_eq!(
        plan.planning_rationale.as_deref(),
        Some("selected src/lib.rs because context and trace evidence agree")
    );
    assert_eq!(
        plan.verification_strategy.as_deref(),
        Some("run targeted arithmetic tests for src/lib.rs")
    );
    assert_eq!(plan.next_revision(), 2);
}

#[test]
fn with_evidence_sets_source_evidence() {
    let evidence = vec![EvidenceRef::file("src/lib.rs"), EvidenceRef::canon(".canon/a")];
    let plan =
        GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap().with_evidence(evidence.clone());
    assert_eq!(plan.source_evidence, evidence);
}

#[test]
fn goal_plan_round_trips_through_json() {
    let plan = GoalPlan::new("Fix the bug", vec![sample_task("t1"), sample_task("t2")]).unwrap();
    let json = serde_json::to_string(&plan).unwrap();
    let parsed: GoalPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(plan.plan_id, parsed.plan_id);
    assert_eq!(plan.tasks.len(), parsed.tasks.len());
    assert_eq!(plan.goal_text, parsed.goal_text);
}

#[test]
fn workspace_signals_default_is_empty() {
    let signals = WorkspaceSignals::default();
    assert!(signals.language.is_none());
    assert_eq!(signals.file_count, 0);
    assert!(!signals.has_config);
    assert!(!signals.has_canon);
    assert!(!signals.has_tests);
}

#[test]
fn with_context_pack_sets_summary_and_primary_inputs() {
    let context_pack = ContextPack {
        pack_id: "cp-1".to_string(),
        summary: "bounded planning context".to_string(),
        credibility: ContextPackCredibility::Credible,
        inputs: vec![ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "matches the goal keywords".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec!["src/lib.rs".to_string()],
        staleness_reason: None,
    };

    let plan =
        GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap().with_context_pack(context_pack);

    assert_eq!(plan.context_summary().as_deref(), Some("bounded planning context"));
    assert_eq!(plan.context_credibility().as_deref(), Some("credible"));
    assert_eq!(plan.context_primary_inputs(), vec!["src/lib.rs".to_string()]);
    assert_eq!(
        plan.context_provenance_lines(),
        vec![
            "workspace_file: src/lib.rs (matches the goal keywords) [source=workspace_scan]"
                .to_string()
        ]
    );
}

#[test]
fn goal_plan_validation_rejects_credible_context_without_primary_inputs() {
    let err = GoalPlan::new("Goal", vec![sample_task("t1")])
        .unwrap()
        .with_context_pack(ContextPack {
            pack_id: "cp-2".to_string(),
            summary: "missing primaries".to_string(),
            credibility: ContextPackCredibility::Credible,
            inputs: Vec::new(),
            selected_targets: Vec::new(),
            staleness_reason: None,
        })
        .validate()
        .unwrap_err();

    assert!(matches!(err, GoalPlanError::MissingCredibleContextPrimaryInput));
}

#[test]
fn goal_plan_validation_rejects_stale_context_without_reason() {
    let err = GoalPlan::new("Goal", vec![sample_task("t1")])
        .unwrap()
        .with_context_pack(ContextPack {
            pack_id: "cp-3".to_string(),
            summary: "stale context".to_string(),
            credibility: ContextPackCredibility::Stale,
            inputs: vec![ContextInput {
                kind: ContextInputKind::RecentTrace,
                reference: ".boundline/traces/old.json".to_string(),
                rationale: "was the last authoritative trace".to_string(),
                source: "latest_trace".to_string(),
                primary: false,
            }],
            selected_targets: Vec::new(),
            staleness_reason: None,
        })
        .validate()
        .unwrap_err();

    assert!(matches!(err, GoalPlanError::MissingContextStalenessReason));
}

#[test]
fn context_input_and_flow_state_helpers_cover_remaining_goal_plan_branches() {
    assert_eq!(ContextPackCredibility::Stale.as_str(), "stale");
    assert_eq!(ContextInputKind::AuthoredBrief.as_str(), "authored_brief");
    assert_eq!(ContextInputKind::Negotiation.as_str(), "negotiation");
    assert_eq!(ContextInputKind::CanonArtifact.as_str(), "canon_artifact");

    let missing_reference = ContextInput {
        kind: ContextInputKind::WorkspaceFile,
        reference: " ".to_string(),
        rationale: "matches the requested goal".to_string(),
        source: "workspace_scan".to_string(),
        primary: true,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(missing_reference, GoalPlanError::MissingContextInputReference));

    let missing_rationale = ContextInput {
        kind: ContextInputKind::WorkspaceFile,
        reference: "src/lib.rs".to_string(),
        rationale: " ".to_string(),
        source: "workspace_scan".to_string(),
        primary: true,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(
        missing_rationale,
        GoalPlanError::MissingContextInputRationale { reference } if reference == "src/lib.rs"
    ));

    let missing_source = ContextInput {
        kind: ContextInputKind::SymbolHint,
        reference: "src/lib.rs::add".to_string(),
        rationale: "matches the failing test evidence".to_string(),
        source: " ".to_string(),
        primary: false,
    }
    .validate()
    .unwrap_err();
    assert!(matches!(
        missing_source,
        GoalPlanError::MissingContextInputSource { reference } if reference == "src/lib.rs::add"
    ));

    let input = ContextInput {
        kind: ContextInputKind::SymbolHint,
        reference: "src/lib.rs::add".to_string(),
        rationale: "matches the failing test evidence".to_string(),
        source: "workspace_scan".to_string(),
        primary: false,
    };
    assert_eq!(
        input.provenance_line(),
        "symbol_hint: src/lib.rs::add (matches the failing test evidence) [source=workspace_scan]"
    );

    let context_pack = ContextPack {
        pack_id: "cp-4".to_string(),
        summary: "selected bounded targets".to_string(),
        credibility: ContextPackCredibility::Credible,
        inputs: vec![input],
        selected_targets: vec!["src/lib.rs".to_string()],
        staleness_reason: None,
    };
    assert!(context_pack.validate().is_ok());
    assert_eq!(context_pack.primary_references(), vec!["src/lib.rs".to_string()]);

    let proposed = GoalPlanFlowState {
        mode: GoalPlanFlowMode::Proposed,
        flow_name: Some("bug-fix".to_string()),
        confidence_reason: None,
    };
    assert_eq!(proposed.summary_text(), "proposed (bug-fix)");

    let absent = GoalPlanFlowState {
        mode: GoalPlanFlowMode::Absent,
        flow_name: None,
        confidence_reason: None,
    };
    assert_eq!(absent.summary_text(), "absent");
}
