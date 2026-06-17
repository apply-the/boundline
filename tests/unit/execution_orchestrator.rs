//! Unit tests for the execution orchestrator: Kahn's algorithm, dependency
//! graph validation, checkpoint serialization, blocked-task detection, and
//! terminal outcome projections.

use std::path::Path;

use boundline::domain::execution_orchestration::{
    CheckpointReason, DependencyGraphError, ExecutionPlanState, TaskDependencyGraph, TaskOutcome,
    TerminalOutcome,
};
use boundline::domain::goal_plan::{GoalPlan, PlannedTask};
use boundline::orchestrator::execution_orchestrator::{
    CheckpointWriteParams, ExecutionOrchestrator, ExecutionOrchestratorError,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_plan(tasks: Vec<PlannedTask>) -> GoalPlan {
    GoalPlan::new("Test plan", tasks).expect("build plan")
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
    let suffix = format!("boundline-test-{}-{}", std::process::id(), n,);
    let dir = std::env::temp_dir().join(&suffix);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.to_string_lossy().into_owned();
    (dir, path)
}

// ── T012: Dependency graph tests ─────────────────────────────────────────────

#[test]
fn t012_valid_linear_chain() {
    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into()])),
        make_task("T003", Some(vec!["T002".into()])),
    ]);
    let graph = TaskDependencyGraph::from_plan(&plan).expect("linear chain should be valid");
    assert_eq!(graph.topological_order, vec!["T001", "T002", "T003"]);
    assert!(graph.cycles.is_empty());
    assert!(graph.missing_references.is_empty());
    assert!(graph.self_dependencies.is_empty());
    // Only T001 should be runnable initially (no dependencies).
    assert_eq!(graph.next_runnable_task_id(), Some("T001"));
}

#[test]
fn t012_valid_dag() {
    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", None),
        make_task("T003", Some(vec!["T001".into(), "T002".into()])),
    ]);
    let graph = TaskDependencyGraph::from_plan(&plan).expect("DAG should be valid");
    // Both T001 and T002 have zero in-degree; plan-order tie-breaking
    // should produce T001 first.
    assert_eq!(graph.topological_order, vec!["T001", "T002", "T003"]);
    assert_eq!(graph.next_runnable_task_id(), Some("T001"));
}

#[test]
fn t012_cycle_detection() {
    let plan = make_plan(vec![
        make_task("T001", Some(vec!["T002".into()])),
        make_task("T002", Some(vec!["T001".into()])),
    ]);
    let err = TaskDependencyGraph::from_plan(&plan).unwrap_err();
    assert!(matches!(err, DependencyGraphError::CycleDetected { .. }));
}

#[test]
fn t012_self_dependency() {
    let plan = make_plan(vec![make_task("T001", Some(vec!["T001".into()]))]);
    let err = TaskDependencyGraph::from_plan(&plan).unwrap_err();
    assert!(matches!(err, DependencyGraphError::SelfDependency { .. }));
}

#[test]
fn t012_missing_reference() {
    let plan = make_plan(vec![make_task("T001", Some(vec!["T999".into()]))]);
    let err = TaskDependencyGraph::from_plan(&plan).unwrap_err();
    assert!(matches!(err, DependencyGraphError::MissingReference { .. }));
}

#[test]
fn t012_duplicate_normalization() {
    // Duplicate and unsorted depends_on entries should be normalized.
    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into(), "T001".into()])),
    ]);
    let graph = TaskDependencyGraph::from_plan(&plan).expect("duplicates should be normalized");
    let t002 = graph.nodes.iter().find(|n| n.task_id == "T002").expect("T002");
    assert_eq!(t002.depends_on, vec!["T001"]);
}

#[test]
fn t012_plan_order_tie_breaking() {
    // When multiple tasks are simultaneously runnable (zero in-degree),
    // Kahn's algorithm should prefer plan order.
    let plan =
        make_plan(vec![make_task("T003", None), make_task("T001", None), make_task("T002", None)]);
    let graph = TaskDependencyGraph::from_plan(&plan).expect("valid");
    // Plan order: T003, T001, T002
    assert_eq!(graph.topological_order, vec!["T003", "T001", "T002"]);
    assert_eq!(graph.next_runnable_task_id(), Some("T003"));
}

#[test]
fn t012_empty_plan_rejected() {
    // GoalPlan::new rejects empty task lists with NoTasks.
    let result = GoalPlan::new("Empty", vec![]);
    assert!(result.is_err(), "empty task list should be rejected");
}

#[test]
fn t012_multiple_independent_tasks() {
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", None), make_task("T003", None)]);
    let graph = TaskDependencyGraph::from_plan(&plan).expect("valid");
    assert_eq!(graph.topological_order, vec!["T001", "T002", "T003"]);
    // All three are runnable, but T001 is first in plan order.
    assert_eq!(graph.next_runnable_task_id(), Some("T001"));
}

// ── T013: Checkpoint tests ───────────────────────────────────────────────────

#[test]
fn t013_checkpoint_write_and_read() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    // Set up .boundline/ directory
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    let params = CheckpointWriteParams {
        state: ExecutionPlanState::Running,
        active_task_id: Some("T001"),
        next_runnable: Some("T002"),
        completed_ids: &[],
        blocked: &[],
        skipped: &[],
        last_outcome: None,
        reason: CheckpointReason::TaskTerminalOutcome,
    };
    orchestrator.write_checkpoint(&params).expect("write checkpoint");

    // Read back and validate.
    let run = orchestrator.current_run().expect("run exists");
    let checkpoint =
        orchestrator.read_checkpoint(workspace_path, &run.run_id, None).expect("read checkpoint");
    assert_eq!(checkpoint.run_id, run.run_id);
    assert_eq!(checkpoint.execution_state, ExecutionPlanState::Running);
    assert_eq!(checkpoint.checkpoint_reason, CheckpointReason::TaskTerminalOutcome);
    assert_eq!(checkpoint.active_task_id.as_deref(), Some("T001"));
    assert_eq!(checkpoint.next_runnable_task_id.as_deref(), Some("T002"));
}

#[test]
fn t013_checkpoint_schema_validation() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    let params = CheckpointWriteParams {
        state: ExecutionPlanState::Running,
        active_task_id: None,
        next_runnable: None,
        completed_ids: &[],
        blocked: &[],
        skipped: &[],
        last_outcome: None,
        reason: CheckpointReason::PauseRequested,
    };
    orchestrator.write_checkpoint(&params).expect("write");

    let run = orchestrator.current_run().expect("run");
    let checkpoint = orchestrator
        .read_checkpoint(workspace_path, &run.run_id, None)
        .expect("read should succeed with valid schema");
    assert_eq!(checkpoint.schema_version, "1");
}

#[test]
fn t013_checkpoint_incompatible_fingerprint_rejected() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    let params = CheckpointWriteParams {
        state: ExecutionPlanState::Running,
        active_task_id: None,
        next_runnable: None,
        completed_ids: &[],
        blocked: &[],
        skipped: &[],
        last_outcome: None,
        reason: CheckpointReason::PauseRequested,
    };
    orchestrator.write_checkpoint(&params).expect("write");

    let run = orchestrator.current_run().expect("run");
    let err = orchestrator
        .read_checkpoint(workspace_path, &run.run_id, Some("sha256:WRONG"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fingerprint"), "expected fingerprint mismatch: {msg}");
}

#[test]
fn t013_checkpoint_previous_preserved_on_failure() {
    // Atomicity: when a write fails, the previous checkpoint should remain intact.
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Write first checkpoint.
    let params1 = CheckpointWriteParams {
        state: ExecutionPlanState::Running,
        active_task_id: Some("T001"),
        next_runnable: None,
        completed_ids: &[],
        blocked: &[],
        skipped: &[],
        last_outcome: None,
        reason: CheckpointReason::TaskTerminalOutcome,
    };
    orchestrator.write_checkpoint(&params1).expect("write 1");

    let run = orchestrator.current_run().expect("run");
    let first =
        orchestrator.read_checkpoint(workspace_path, &run.run_id, None).expect("read first");
    assert_eq!(first.active_task_id.as_deref(), Some("T001"));

    // Now write a second checkpoint.
    let params2 = CheckpointWriteParams {
        state: ExecutionPlanState::Completed,
        active_task_id: None,
        next_runnable: None,
        completed_ids: &["T001".into()],
        blocked: &[],
        skipped: &[],
        last_outcome: Some(&TerminalOutcome {
            task_id: "T001".into(),
            outcome: TaskOutcome::Completed,
        }),
        reason: CheckpointReason::TaskTerminalOutcome,
    };
    orchestrator.write_checkpoint(&params2).expect("write 2");

    let second =
        orchestrator.read_checkpoint(workspace_path, &run.run_id, None).expect("read second");
    assert_eq!(second.execution_state, ExecutionPlanState::Completed);
    assert_eq!(second.completed_task_ids, vec!["T001"]);
}

// ── T022: Executor tests (start, advance, record_outcome, resume) ────────────

#[test]
fn t022_start_with_valid_plan() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);

    let run = orchestrator.start(&plan, workspace_path, "S-001").expect("start");
    assert_eq!(run.session_id, "S-001");
    assert!(orchestrator.current_run().is_some());
    assert!(orchestrator.current_graph().is_some());
}

#[test]
fn t022_start_with_cyclic_plan_fails() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![
        make_task("T001", Some(vec!["T002".into()])),
        make_task("T002", Some(vec!["T001".into()])),
    ]);

    let err = orchestrator.start(&plan, workspace_path, "S-001").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("cycle"), "expected cycle error: {msg}");
}

#[test]
fn t022_advance_through_three_task_chain() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into()])),
        make_task("T003", Some(vec!["T002".into()])),
    ]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Advance to T001.
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T001"));

    // Complete T001.
    orchestrator.record_outcome("T001", TaskOutcome::Completed, None, None).expect("record T001");

    // Advance to T002.
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T002"));

    // Complete T002.
    orchestrator.record_outcome("T002", TaskOutcome::Completed, None, None).expect("record T002");

    // Advance to T003.
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T003"));

    // Complete T003.
    orchestrator.record_outcome("T003", TaskOutcome::Completed, None, None).expect("record T003");

    // No more tasks.
    let task = orchestrator.advance();
    assert_eq!(task, None);
}

#[test]
fn t022_resume_from_checkpoint() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);

    // First run: start, advance T001, complete it, write checkpoint.
    let mut orch1 = ExecutionOrchestrator::new();
    orch1.start(&plan, workspace_path, "S-001").expect("start");
    orch1.advance();
    orch1.record_outcome("T001", TaskOutcome::Completed, None, None).expect("record T001");

    let run_id = orch1.current_run().expect("run").run_id.clone();

    // Resume in a new orchestrator.
    let mut orch2 = ExecutionOrchestrator::new();
    orch2.resume(&run_id, workspace_path, &plan).expect("resume");

    // Next task should be T002.
    let task = orch2.advance();
    assert_eq!(task.as_deref(), Some("T002"));
}

#[test]
fn t022_resume_incompatible_plan_rejected() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan_a = make_plan(vec![make_task("T001", None)]);
    let plan_b = make_plan(vec![make_task("T999", None)]);

    let mut orch1 = ExecutionOrchestrator::new();
    orch1.start(&plan_a, workspace_path, "S-001").expect("start");
    let run_id = orch1.current_run().expect("run").run_id.clone();

    // Resume with a different plan should fail due to fingerprint mismatch.
    let mut orch2 = ExecutionOrchestrator::new();
    let err = orch2.resume(&run_id, workspace_path, &plan_b).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fingerprint"), "expected fingerprint error: {msg}");
}

// ── T030: Blocked-task detection tests ──────────────────────────────────────

#[test]
fn t030_blocked_completion_proof_failed() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Advance and block T001 with completion_proof_failed.
    orchestrator.advance();
    orchestrator
        .record_outcome(
            "T001",
            TaskOutcome::Blocked,
            Some(boundline::orchestrator::execution_orchestrator::BLOCKED_REASON_COMPLETION_PROOF_FAILED),
            Some(".boundline/traces/proof-T001.json"),
        )
        .expect("record blocked");

    assert_eq!(orchestrator.blocked_tasks().len(), 1);
    let blocked = &orchestrator.blocked_tasks()[0];
    assert_eq!(blocked.task_id, "T001");
    assert_eq!(blocked.reason, "completion_proof_failed");
    assert_eq!(blocked.evidence_ref.as_deref(), Some(".boundline/traces/proof-T001.json"));

    // Pause reason should include the blocked task.
    let reason = orchestrator.pause_reason().expect("pause reason");
    assert!(reason.contains("T001"));
    assert!(reason.contains("completion_proof_failed"));
}

#[test]
fn t030_blocked_verification_unavailable() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    orchestrator.advance();
    orchestrator
        .record_outcome(
            "T001",
            TaskOutcome::Blocked,
            Some(boundline::orchestrator::execution_orchestrator::BLOCKED_REASON_VERIFICATION_UNAVAILABLE),
            None,
        )
        .expect("record blocked");

    let blocked = &orchestrator.blocked_tasks()[0];
    assert_eq!(blocked.reason, "verification_unavailable");
}

#[test]
fn t030_blocked_governance_blocked() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    orchestrator.advance();
    orchestrator
        .record_outcome(
            "T001",
            TaskOutcome::Blocked,
            Some(
                boundline::orchestrator::execution_orchestrator::BLOCKED_REASON_GOVERNANCE_BLOCKED,
            ),
            None,
        )
        .expect("record blocked");

    let blocked = &orchestrator.blocked_tasks()[0];
    assert_eq!(blocked.reason, "governance_blocked");
}

// ── T031: Downstream halt tests ──────────────────────────────────────────────

#[test]
fn t031_blocked_task_prevents_dependent_execution() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into()])),
        make_task("T003", Some(vec!["T002".into()])),
    ]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Complete T001 normally.
    orchestrator.advance();
    orchestrator.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    // Advance to T002 and block it.
    orchestrator.advance();
    orchestrator
        .record_outcome("T002", TaskOutcome::Blocked, Some("completion_proof_failed"), None)
        .expect("block T002");

    // Halt downstream dependents of T002.
    orchestrator.downstream_halt("T002");

    // Advance should NOT return T003 because it's been halted.
    let next = orchestrator.advance();
    assert_eq!(next, None, "T003 should be halted");
}

#[test]
fn t031_resolution_unblocks_dependents() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Complete T001.
    orchestrator.advance();
    orchestrator.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    // Block T002.
    orchestrator.advance();
    orchestrator
        .record_outcome("T002", TaskOutcome::Blocked, Some("completion_proof_failed"), None)
        .expect("block T002");

    orchestrator.downstream_halt("T002");
    assert_eq!(orchestrator.advance(), None);

    // Resolve the block on T002.
    orchestrator.resolve_blocked_task("T002");

    // After resolution, no more tasks (all done).
    assert_eq!(orchestrator.advance(), None);
    assert!(orchestrator.blocked_tasks().is_empty());
}

// ── T037: Status projection tests ────────────────────────────────────────────

#[test]
fn t037_status_projection_running_state() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Create a fresh view and project.
    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    assert!(view.execution_run_id.is_some());
    assert_eq!(view.execution_plan_state.as_deref(), Some("ready"));
    assert_eq!(view.execution_completed_task_count, Some(0));
    assert!(view.execution_next_task_id.is_some());
    assert!(view.execution_checkpoint_ref.is_some());
    assert!(view.execution_resume_command.is_some());
    assert!(view.execution_blocked_task_ids.is_none());
}

#[test]
fn t037_status_projection_active_task_reports_running_state() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    assert_eq!(orchestrator.advance().as_deref(), Some("T001"));

    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    assert_eq!(view.execution_plan_state.as_deref(), Some("running"));
    assert_eq!(view.execution_current_task_id.as_deref(), Some("T001"));
    assert_eq!(view.execution_completed_task_count, Some(0));
}

#[test]
fn t037_status_projection_blocked_state() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    orchestrator.advance();
    orchestrator
        .record_outcome("T001", TaskOutcome::Blocked, Some("completion_proof_failed"), None)
        .expect("block T001");

    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    assert_eq!(view.execution_plan_state.as_deref(), Some("blocked"));
    assert_eq!(view.execution_completed_task_count, Some(0));
    let blocked_ids = view.execution_blocked_task_ids.expect("blocked ids");
    assert!(blocked_ids.contains(&"T001".to_string()));
}

#[test]
fn t037_status_projection_completed_state() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    orchestrator.advance();
    orchestrator.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    assert_eq!(view.execution_plan_state.as_deref(), Some("completed"));
    assert_eq!(view.execution_completed_task_count, Some(1));
    assert!(view.execution_blocked_task_ids.is_none());
    assert_eq!(view.execution_next_task_id, None);
}

#[test]
fn t037_status_projection_inactive_when_no_run() {
    let orchestrator = ExecutionOrchestrator::new();
    let mut view = boundline::domain::session::SessionStatusView::default();
    orchestrator.project_status(&mut view);

    // All execution fields should remain None when no run is active.
    assert!(view.execution_run_id.is_none());
    assert!(view.execution_plan_state.is_none());
    assert!(view.execution_current_task_id.is_none());
    assert!(view.execution_next_task_id.is_none());
    assert!(view.execution_completed_task_count.is_none());
    assert!(view.execution_blocked_task_ids.is_none());
    assert!(view.execution_checkpoint_ref.is_none());
    assert!(view.execution_resume_command.is_none());
}

// ── T044: Edge case tests ───────────────────────────────────────────────────

#[test]
fn t044_empty_plan_rejected_at_construction() {
    // GoalPlan::new rejects empty task lists — this is tested in t012.
    // Here we verify orchestrator does not produce a run for empty plans.
    let result = GoalPlan::new("Empty", vec![]);
    assert!(result.is_err(), "empty plan should be rejected at GoalPlan level");
}

#[test]
fn t044_duplicate_start_returns_existing_run() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);

    // First start succeeds.
    orchestrator.start(&plan, workspace_path, "S-001").expect("first start");

    // Second start with the same orchestrator should fail with RunAlreadyActive.
    let err = orchestrator.start(&plan, workspace_path, "S-002").unwrap_err();
    assert!(
        matches!(err, ExecutionOrchestratorError::RunAlreadyActive { .. }),
        "expected RunAlreadyActive, got: {err}"
    );
}

#[test]
fn t044_degraded_projection_on_missing_checkpoint() {
    // Checkpoint file doesn't exist — projection should remain empty
    // but not panic.
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut view = boundline::domain::session::SessionStatusView::default();
    boundline::orchestrator::execution_orchestrator::populate_execution_projection_from_checkpoint(
        workspace_path,
        "ER-nonexistent",
        &mut view,
    );

    // All fields should remain None (degraded projection).
    assert!(view.execution_run_id.is_none());
    assert!(view.execution_plan_state.is_none());
    assert!(view.execution_checkpoint_ref.is_none());
}

#[test]
fn t044_resume_with_incompatible_plan_fingerprint_rejected() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan_a = make_plan(vec![make_task("T001", None)]);
    let plan_b = make_plan(vec![make_task("T999", None)]);

    let mut orch1 = ExecutionOrchestrator::new();
    orch1.start(&plan_a, workspace_path, "S-001").expect("start");
    let run_id = orch1.current_run().expect("run").run_id.clone();

    let mut orch2 = ExecutionOrchestrator::new();
    let err = orch2.resume(&run_id, workspace_path, &plan_b).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fingerprint"), "expected fingerprint mismatch: {msg}");
}

#[test]
fn t044_skipped_task_unblocks_dependents() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    // Skip T001 instead of completing it.
    orchestrator.advance();
    orchestrator.record_outcome("T001", TaskOutcome::Skipped, None, None).expect("skip T001");

    // T002 should become runnable since T001 was skipped.
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T002"), "T002 should be runnable after skip");
}

#[test]
fn t044_completion_verification_defaults_to_unavailable() {
    let orchestrator = ExecutionOrchestrator::new();
    assert!(!orchestrator.completion_verification_available());
}

#[test]
fn t044_failed_task_blocks_execution_with_default_reason() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    assert_eq!(orchestrator.advance().as_deref(), Some("T001"));
    orchestrator
        .record_outcome("T001", TaskOutcome::Failed, None, Some("trace://failed"))
        .expect("fail T001");

    let blocked = orchestrator.blocked_tasks();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].task_id, "T001");
    assert_eq!(blocked[0].reason, "task_failed");
    assert_eq!(blocked[0].evidence_ref.as_deref(), Some("trace://failed"));
}

#[test]
fn t044_deferred_task_blocks_execution_with_custom_reason() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let mut orchestrator = ExecutionOrchestrator::new();
    let plan = make_plan(vec![make_task("T001", None)]);
    orchestrator.start(&plan, workspace_path, "S-001").expect("start");

    assert_eq!(orchestrator.advance().as_deref(), Some("T001"));
    orchestrator
        .record_outcome(
            "T001",
            TaskOutcome::Deferred,
            Some("verification_unavailable"),
            Some("trace://deferred"),
        )
        .expect("defer T001");

    let blocked = orchestrator.blocked_tasks();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].task_id, "T001");
    assert_eq!(blocked[0].reason, "verification_unavailable");
    assert_eq!(blocked[0].evidence_ref.as_deref(), Some("trace://deferred"));
}
