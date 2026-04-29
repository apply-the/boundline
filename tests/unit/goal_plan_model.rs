use synod::domain::decision::{DecisionType, EvidenceRef};
use synod::domain::goal_plan::{
    GoalPlan, GoalPlanError, GoalPlanStatus, InferredFlow, PlannedTask, WorkspaceSignals,
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
    let mut plan = GoalPlan::new("Goal", vec![sample_task("t1")]).unwrap();
    assert!(plan.confirm().is_ok());
    assert_eq!(plan.status, GoalPlanStatus::Confirmed);
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
    assert!(plan.supersede().is_ok());
    assert_eq!(plan.status, GoalPlanStatus::Superseded);
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
