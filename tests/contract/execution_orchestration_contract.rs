//! Contract tests for the execution orchestrator covering checkpoint
//! serialization round-trips, dependency graph invariants, CLI flag
//! semantics (--plan, --resume), and the execution-orchestration
//! projection contract.

use std::path::Path;

use boundline::domain::execution_orchestration::{
    DependencyGraphError, ExecutionPlanState, TaskOutcome,
};
use boundline::domain::goal_plan::{GoalPlan, PlannedTask};
use boundline::orchestrator::execution_orchestrator::ExecutionOrchestrator;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_plan(tasks: Vec<PlannedTask>) -> GoalPlan {
    GoalPlan::new("Contract test plan", tasks).expect("build plan")
}

fn make_task(id: &str, depends_on: Option<Vec<String>>) -> PlannedTask {
    PlannedTask {
        task_id: id.to_string(),
        description: format!("Task {id}"),
        target: format!("src/{id}.rs"),
        expected_outcome: Some(format!("{id} done")),
        decision_type_hint: None,
        depends_on,
    }
}

fn make_temp_workspace() -> (std::path::PathBuf, String) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let suffix = format!("boundline-contract-{}-{}", std::process::id(), n);
    let dir = std::env::temp_dir().join(&suffix);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.to_string_lossy().into_owned();
    (dir, path)
}

// ── Serde round-trips ────────────────────────────────────────────────────────

#[test]
fn execution_plan_state_serde_round_trip() {
    let original = ExecutionPlanState::Ready;
    let json = serde_json::to_string(&original).expect("serialize");
    let round_tripped: ExecutionPlanState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, round_tripped);

    let original = ExecutionPlanState::Running;
    let json = serde_json::to_string(&original).expect("serialize");
    let round_tripped: ExecutionPlanState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, round_tripped);
}

// ── T023: CLI flag semantics contract tests ───────────────────────────────────

#[test]
fn t023_start_with_valid_plan_succeeds() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);

    let run_id = {
        let run =
            orchestrator.start(&plan, workspace_path, "S-001").expect("start with valid plan");
        assert!(run.run_id.starts_with("ER-"));
        run.run_id.clone()
    };
    assert!(orchestrator.current_run().is_some());

    // Verify initial checkpoint exists.
    let checkpoint =
        orchestrator.read_checkpoint(workspace_path, &run_id, None).expect("initial checkpoint");
    assert_eq!(checkpoint.execution_state, ExecutionPlanState::Ready);
}

#[test]
fn t023_start_with_cyclic_plan_is_blocked_at_plan_load() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![
        make_task("T001", Some(vec!["T002".into()])),
        make_task("T002", Some(vec!["T001".into()])),
    ]);

    let err = orchestrator.start(&plan, workspace_path, "S-001").unwrap_err();
    assert!(
        matches!(err, boundline::orchestrator::execution_orchestrator::ExecutionOrchestratorError::DependencyGraph(
            DependencyGraphError::CycleDetected { .. }
        )),
        "expected cycle error, got: {err}"
    );
}

#[test]
fn t023_start_with_missing_plan_reference_errors() {
    // Simulating `--plan <missing-ref>` by passing a non-existent plan path
    // would normally happen at the CLI layer. Here we test that a plan with
    // missing dependency references is rejected at graph-build time.
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", Some(vec!["T999".into()]))]);

    let err = orchestrator.start(&plan, workspace_path, "S-001").unwrap_err();
    assert!(
        matches!(err, boundline::orchestrator::execution_orchestrator::ExecutionOrchestratorError::DependencyGraph(
            DependencyGraphError::MissingReference { .. }
        )),
        "expected missing reference error, got: {err}"
    );
}

#[test]
fn t023_resume_after_interruption_continues_from_checkpoint() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into()])),
        make_task("T003", Some(vec!["T002".into()])),
    ]);

    // First orchestrator: start, advance T001, complete it.
    let mut orch1 = ExecutionOrchestrator::new();
    orch1.start(&plan, workspace_path, "S-001").expect("start");
    let run_id = orch1.current_run().expect("run").run_id.clone();

    let task = orch1.advance();
    assert_eq!(task.as_deref(), Some("T001"));
    orch1.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    // Simulate interruption — resume in a new orchestrator.
    let mut orch2 = ExecutionOrchestrator::new();
    orch2.resume(&run_id, workspace_path, &plan).expect("resume");

    // Next task should be T002, not restarting from scratch.
    let task = orch2.advance();
    assert_eq!(task.as_deref(), Some("T002"));

    // Complete remaining tasks.
    orch2.record_outcome("T002", TaskOutcome::Completed, None, None).expect("complete T002");
    orch2.record_outcome("T003", TaskOutcome::Completed, None, None).expect("complete T003");

    // All done.
    assert_eq!(orch2.advance(), None);
}

// ── T038: Status projection contract tests ──────────────────────────────────

#[test]
fn t038_status_projection_fields_present_when_execution_active() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    // All eight execution fields should be populated.
    assert!(view.execution_run_id.is_some(), "execution_run_id");
    assert!(view.execution_plan_state.is_some(), "execution_plan_state");
    // current_task_id may be None if no task is active
    assert!(view.execution_next_task_id.is_some(), "execution_next_task_id");
    assert_eq!(view.execution_completed_task_count, Some(0));
    assert!(view.execution_checkpoint_ref.is_some(), "execution_checkpoint_ref");
    assert!(view.execution_resume_command.is_some(), "execution_resume_command");
    // blocked_task_ids absent when not blocked
    assert!(view.execution_blocked_task_ids.is_none());
}

#[test]
fn t038_status_projection_fields_absent_when_inactive() {
    let orchestrator = ExecutionOrchestrator::new();
    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    // All eight fields should remain None.
    assert!(view.execution_run_id.is_none());
    assert!(view.execution_plan_state.is_none());
    assert!(view.execution_current_task_id.is_none());
    assert!(view.execution_next_task_id.is_none());
    assert!(view.execution_completed_task_count.is_none());
    assert!(view.execution_blocked_task_ids.is_none());
    assert!(view.execution_checkpoint_ref.is_none());
    assert!(view.execution_resume_command.is_none());
}
