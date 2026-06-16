//! Integration tests for the execution orchestrator covering end-to-end
//! checkpointing, resume, and the full plan-execution lifecycle.

use std::path::Path;

use boundline::domain::execution_orchestration::{ExecutionPlanState, TaskOutcome};
use boundline::domain::goal_plan::{GoalPlan, PlannedTask};
use boundline::orchestrator::execution_orchestrator::ExecutionOrchestrator;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_plan(tasks: Vec<PlannedTask>) -> GoalPlan {
    GoalPlan::new("Integration test plan", tasks).expect("build plan")
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
    let suffix = format!("boundline-integ-{}-{}", std::process::id(), n);
    let dir = std::env::temp_dir().join(&suffix);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.to_string_lossy().into_owned();
    (dir, path)
}

// ── T024: Full three-task plan execution ─────────────────────────────────────

#[test]
fn t024_full_three_task_execution_with_dependency_order() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into()])),
        make_task("T003", Some(vec!["T001".into(), "T002".into()])),
    ]);

    let mut orchestrator = ExecutionOrchestrator::new();

    // Start execution.
    let run = orchestrator.start(&plan, workspace_path, "S-INT-001").expect("start");
    let run_id = run.run_id.clone();
    println!("Started run: {run_id}");

    // Task 1: T001 (no dependencies, should be first).
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T001"), "T001 should be first");
    orchestrator.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    // Verify checkpoint after T001.
    let cp1 =
        orchestrator.read_checkpoint(workspace_path, &run_id, None).expect("checkpoint after T001");
    assert_eq!(cp1.execution_state, ExecutionPlanState::Running);
    assert_eq!(cp1.completed_task_ids, vec!["T001"]);
    assert!(cp1.checkpoint_sequence >= 1);

    // Task 2: T002 (depends on T001, should be next).
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T002"), "T002 should be second");
    orchestrator.record_outcome("T002", TaskOutcome::Completed, None, None).expect("complete T002");

    // Verify checkpoint after T002.
    let cp2 =
        orchestrator.read_checkpoint(workspace_path, &run_id, None).expect("checkpoint after T002");
    assert_eq!(cp2.completed_task_ids, vec!["T001", "T002"]);
    assert!(cp2.checkpoint_sequence >= 2);

    // Task 3: T003 (depends on T001 and T002, should be last).
    let task = orchestrator.advance();
    assert_eq!(task.as_deref(), Some("T003"), "T003 should be third");
    orchestrator.record_outcome("T003", TaskOutcome::Completed, None, None).expect("complete T003");

    // Verify terminal checkpoint.
    let cp3 =
        orchestrator.read_checkpoint(workspace_path, &run_id, None).expect("terminal checkpoint");
    assert_eq!(cp3.execution_state, ExecutionPlanState::Completed);
    assert_eq!(cp3.completed_task_ids, vec!["T001", "T002", "T003"]);
    assert!(cp3.last_terminal_outcome.is_some());

    // No more tasks.
    assert_eq!(orchestrator.advance(), None);
}

#[test]
fn t024_resume_and_complete_remaining_tasks() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan =
        make_plan(vec![make_task("T001", None), make_task("T002", Some(vec!["T001".into()]))]);

    // First run: start and complete T001 only.
    let mut orch1 = ExecutionOrchestrator::new();
    orch1.start(&plan, workspace_path, "S-INT-002").expect("start");
    let run_id = orch1.current_run().expect("run").run_id.clone();

    orch1.advance();
    orch1.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    // Resume and complete T002.
    let mut orch2 = ExecutionOrchestrator::new();
    orch2.resume(&run_id, workspace_path, &plan).expect("resume");

    let task = orch2.advance();
    assert_eq!(task.as_deref(), Some("T002"));
    orch2.record_outcome("T002", TaskOutcome::Completed, None, None).expect("complete T002");

    // Verify final state.
    let cp = orch2.read_checkpoint(workspace_path, &run_id, None).expect("final checkpoint");
    assert_eq!(cp.execution_state, ExecutionPlanState::Completed);
    assert_eq!(cp.completed_task_ids, vec!["T001", "T002"]);
}

// ── T032: Blocked-task integration test ──────────────────────────────────────

#[test]
fn t032_blocked_middle_task_halt_and_resume_after_resolution() {
    let (_dir, workspace) = make_temp_workspace();
    let workspace_path = Path::new(&workspace);
    std::fs::create_dir_all(workspace_path.join(".boundline")).expect("create boundline dir");

    let plan = make_plan(vec![
        make_task("T001", None),
        make_task("T002", Some(vec!["T001".into()])),
        make_task("T003", Some(vec!["T002".into()])),
    ]);

    // Start and complete T001.
    let mut orch = ExecutionOrchestrator::new();
    orch.start(&plan, workspace_path, "S-INT-003").expect("start");
    let run_id = orch.current_run().expect("run").run_id.clone();

    orch.advance();
    orch.record_outcome("T001", TaskOutcome::Completed, None, None).expect("complete T001");

    // Advance to T002 and block it.
    orch.advance();
    orch.record_outcome(
        "T002",
        TaskOutcome::Blocked,
        Some("completion_proof_failed"),
        Some(".boundline/traces/proof-T002.json"),
    )
    .expect("block T002");

    // Halt downstream: T003 should not be runnable.
    orch.downstream_halt("T002");
    assert_eq!(orch.advance(), None, "T003 halted");

    // Verify blocked state in checkpoint.
    let cp_blocked = orch.read_checkpoint(workspace_path, &run_id, None).expect("blocked cp");
    assert_eq!(cp_blocked.execution_state, ExecutionPlanState::Blocked);
    assert_eq!(cp_blocked.blocked_tasks.len(), 1);
    assert_eq!(cp_blocked.blocked_tasks[0].task_id, "T002");
    assert_eq!(cp_blocked.completed_task_ids, vec!["T001"]);

    // Verify pause reason.
    let reason = orch.pause_reason().expect("pause reason");
    assert!(reason.contains("T002"), "pause reason: {reason}");
    assert!(reason.contains("completion_proof_failed"), "pause reason: {reason}");

    // Resolve the block on T002.
    orch.resolve_blocked_task("T002");

    // After resolution, T003 should become runnable.
    let task = orch.advance();
    assert_eq!(task.as_deref(), Some("T003"), "T003 should be runnable after resolution");

    // Complete T003.
    orch.record_outcome("T003", TaskOutcome::Completed, None, None).expect("complete T003");

    // Verify final state.
    let cp_final = orch.read_checkpoint(workspace_path, &run_id, None).expect("final cp");
    assert_eq!(cp_final.execution_state, ExecutionPlanState::Completed);
    assert_eq!(cp_final.completed_task_ids, vec!["T001", "T002", "T003"]);
    assert!(cp_final.blocked_tasks.is_empty());
}
