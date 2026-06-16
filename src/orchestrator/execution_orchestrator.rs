//! Execution orchestrator: drives plan execution with dependency-ordered
//! task scheduling, atomic checkpoints, blocked-state handling, and resume.
//!
//! The orchestrator consumes a [`GoalPlan`] with task-level `depends_on`
//! edges, builds a topological execution order via Kahn's algorithm, and
//! writes durable [`ExecutionCheckpoint`] records under
//! `.boundline/execution/checkpoints/<run-id>.json`.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::domain::execution_orchestration::{
    BlockedTaskRecord, CheckpointReason, ExecutionCheckpoint, ExecutionPlanState, ExecutionRun,
    TaskDependencyGraph, TaskOutcome, TerminalOutcome,
};
use crate::domain::goal_plan::GoalPlan;
use crate::domain::session::SessionStatusView;

/// Error type for execution orchestrator operations.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionOrchestratorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Dependency graph error: {0:?}")]
    DependencyGraph(#[from] crate::domain::execution_orchestration::DependencyGraphError),
    #[error(
        "Incompatible checkpoint: plan fingerprint mismatch (expected {expected}, found {found})"
    )]
    IncompatibleCheckpoint { expected: String, found: String },
    #[error("Checkpoint not found: {path}")]
    CheckpointNotFound { path: String },
    #[error("Execution run already active: {run_id}")]
    RunAlreadyActive { run_id: String },
    #[error("Degraded checkpoint: {path} — {reason}")]
    DegradedCheckpoint { path: String, reason: String },
}

/// Subdirectory under `.boundline/execution/` for checkpoint storage.
const CHECKPOINT_DIR: &str = "checkpoints";

/// Standard blocked-task stop reasons used across the execution orchestrator.
pub const BLOCKED_REASON_COMPLETION_PROOF_FAILED: &str = "completion_proof_failed";
pub const BLOCKED_REASON_VERIFICATION_UNAVAILABLE: &str = "verification_unavailable";
pub const BLOCKED_REASON_GOVERNANCE_BLOCKED: &str = "governance_blocked";
pub const BLOCKED_REASON_TASK_FAILED: &str = "task_failed";

/// Drives plan execution for a single session.
///
/// Lifecycle:
/// 1. Accept a goal plan with task dependency edges.
/// 2. Build a task dependency graph and derive a ready queue.
/// 3. Execute tasks in dependency order, writing checkpoints after each
///    task completion.
/// 4. Detect blocked states and emit blocked-task records.
/// 5. On termination, produce a terminal outcome.
#[derive(Debug)]
pub struct ExecutionOrchestrator {
    /// The current execution run identity.
    run: Option<ExecutionRun>,
    /// The validated dependency graph for the plan.
    graph: Option<TaskDependencyGraph>,
    /// The currently executing task, if any.
    active_task_id: Option<String>,
    /// Task IDs that have reached a terminal outcome.
    completed_task_ids: Vec<String>,
    /// Blocked tasks with stop reasons.
    blocked_tasks: Vec<BlockedTaskRecord>,
    /// Skipped task IDs.
    skipped_tasks: Vec<String>,
    /// The last terminal outcome recorded.
    last_terminal_outcome: Option<TerminalOutcome>,
}

/// Parameters for writing an execution checkpoint.
#[derive(Debug, Clone)]
pub struct CheckpointWriteParams<'a> {
    pub state: ExecutionPlanState,
    pub active_task_id: Option<&'a str>,
    pub next_runnable: Option<&'a str>,
    pub completed_ids: &'a [String],
    pub blocked: &'a [crate::domain::execution_orchestration::BlockedTaskRecord],
    pub skipped: &'a [String],
    pub last_outcome: Option<&'a TerminalOutcome>,
    pub reason: CheckpointReason,
}

impl ExecutionOrchestrator {
    /// Create a new orchestrator instance.
    pub fn new() -> Self {
        Self {
            run: None,
            graph: None,
            active_task_id: None,
            completed_task_ids: Vec::new(),
            blocked_tasks: Vec::new(),
            skipped_tasks: Vec::new(),
            last_terminal_outcome: None,
        }
    }

    /// Start a new execution run from a validated goal plan.
    ///
    /// Builds the dependency graph, validates it, and writes an initial
    /// checkpoint in the `Ready` state.
    pub fn start(
        &mut self,
        plan: &GoalPlan,
        workspace: &Path,
        session_id: &str,
    ) -> Result<&ExecutionRun, ExecutionOrchestratorError> {
        // Reject duplicate start — return existing run instead of overwriting.
        if let Some(ref existing) = self.run {
            return Err(ExecutionOrchestratorError::RunAlreadyActive {
                run_id: existing.run_id.clone(),
            });
        }

        let graph = TaskDependencyGraph::from_plan(plan)?;
        let plan_fingerprint = compute_plan_fingerprint(plan);
        let run_id = generate_run_id();
        let now = current_timestamp_iso();

        self.run = Some(ExecutionRun {
            run_id: run_id.clone(),
            plan_ref: plan.plan_id.clone(),
            plan_fingerprint,
            workspace_ref: workspace.to_string_lossy().into_owned(),
            session_id: session_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
            checkpoint_sequence: 0,
        });
        self.graph = Some(graph);
        self.active_task_id = None;
        self.completed_task_ids.clear();
        self.blocked_tasks.clear();
        self.skipped_tasks.clear();
        self.last_terminal_outcome = None;

        // Write initial checkpoint so resume can find it.
        let params = CheckpointWriteParams {
            state: ExecutionPlanState::Ready,
            active_task_id: None,
            next_runnable: self.graph.as_ref().and_then(|g| g.next_runnable_task_id()),
            completed_ids: &self.completed_task_ids,
            blocked: &self.blocked_tasks,
            skipped: &self.skipped_tasks,
            last_outcome: None,
            reason: CheckpointReason::PauseRequested,
        };
        self.write_checkpoint(&params)?;

        Ok(self.run.as_ref().expect("just set"))
    }

    /// Advance execution by selecting the next runnable task.
    ///
    /// Returns `Some(task_id)` when a task is ready to dispatch.
    /// Returns `None` when all tasks have completed or execution is blocked.
    /// The caller is responsible for dispatching the task and then calling
    /// [`record_outcome`] with the result.
    pub fn advance(&mut self) -> Option<String> {
        let graph = self.graph.as_ref()?;

        // Check if blocked — don't advance if there are unresolved blocks.
        if !self.blocked_tasks.is_empty() {
            return None;
        }

        let next = graph.next_runnable_task_id()?;
        let task_id = next.to_string();

        // Mark as active but not yet completed.
        self.active_task_id = Some(task_id.clone());

        // Update the run timestamp on first advance.
        if let Some(ref mut run) = self.run {
            run.updated_at = current_timestamp_iso();
        }

        Some(task_id)
    }

    /// Record the terminal outcome of a dispatched task and persist a
    /// checkpoint. After recording, the dependency graph is updated so
    /// that dependents may become runnable.
    pub fn record_outcome(
        &mut self,
        task_id: &str,
        outcome: TaskOutcome,
        blocked_reason: Option<&str>,
        evidence_ref: Option<&str>,
    ) -> Result<(), ExecutionOrchestratorError> {
        // Clear the active task.
        if self.active_task_id.as_deref() == Some(task_id) {
            self.active_task_id = None;
        }

        let terminal = TerminalOutcome { task_id: task_id.to_string(), outcome };
        self.last_terminal_outcome = Some(terminal.clone());

        // Scope the graph mutation to drop the mutable borrow before
        // calling write_checkpoint (which needs &self).
        {
            let graph = self.graph.as_mut().expect("graph must exist");

            match outcome {
                TaskOutcome::Completed => {
                    self.completed_task_ids.push(task_id.to_string());
                    graph.mark_completed(task_id);
                }
                TaskOutcome::Blocked => {
                    self.blocked_tasks.push(BlockedTaskRecord {
                        task_id: task_id.to_string(),
                        reason: blocked_reason.unwrap_or("unknown").to_string(),
                        evidence_ref: evidence_ref.map(String::from),
                    });
                    // Do NOT mark as completed — dependents stay blocked.
                }
                TaskOutcome::Skipped => {
                    self.skipped_tasks.push(task_id.to_string());
                    graph.mark_completed(task_id);
                }
                TaskOutcome::Failed | TaskOutcome::Deferred => {
                    self.blocked_tasks.push(BlockedTaskRecord {
                        task_id: task_id.to_string(),
                        reason: blocked_reason.unwrap_or("task_failed").to_string(),
                        evidence_ref: evidence_ref.map(String::from),
                    });
                }
            }
        } // graph mutable borrow dropped here

        // Determine the new execution state and next runnable from self.
        let next_runnable = self.graph.as_ref().and_then(|g| g.next_runnable_task_id());

        let state = if !self.blocked_tasks.is_empty() {
            ExecutionPlanState::Blocked
        } else if next_runnable.is_none() {
            ExecutionPlanState::Completed
        } else {
            ExecutionPlanState::Running
        };

        // Increment checkpoint sequence.
        if let Some(ref mut run) = self.run {
            run.checkpoint_sequence += 1;
            run.updated_at = current_timestamp_iso();
        }

        let params = CheckpointWriteParams {
            state,
            active_task_id: self.active_task_id.as_deref(),
            next_runnable,
            completed_ids: &self.completed_task_ids,
            blocked: &self.blocked_tasks,
            skipped: &self.skipped_tasks,
            last_outcome: Some(&terminal),
            reason: CheckpointReason::TaskTerminalOutcome,
        };
        self.write_checkpoint(&params)?;

        Ok(())
    }

    /// Resume an execution run from a previously written checkpoint.
    ///
    /// Reloads the checkpoint, validates plan identity, rebuilds the
    /// dependency graph from the checkpoint's completed/blocked/skipped
    /// lists, and makes the orchestrator ready to continue advancing.
    pub fn resume(
        &mut self,
        run_id: &str,
        workspace: &Path,
        plan: &GoalPlan,
    ) -> Result<&ExecutionRun, ExecutionOrchestratorError> {
        let plan_fingerprint = compute_plan_fingerprint(plan);

        // Read and validate the checkpoint.
        let checkpoint = self.read_checkpoint(workspace, run_id, Some(&plan_fingerprint))?;

        // Rebuild the dependency graph from the plan.
        let mut graph = TaskDependencyGraph::from_plan(plan)?;

        // Replay completed tasks into the graph.
        for tid in &checkpoint.completed_task_ids {
            graph.mark_completed(tid);
        }
        // Skipped tasks also unblock dependents.
        for tid in &checkpoint.skipped_tasks {
            graph.mark_completed(tid);
        }

        let now = current_timestamp_iso();
        self.run = Some(ExecutionRun {
            run_id: run_id.to_string(),
            plan_ref: plan.plan_id.clone(),
            plan_fingerprint,
            workspace_ref: workspace.to_string_lossy().into_owned(),
            session_id: String::new(), // will be updated by the CLI
            created_at: checkpoint.created_at.clone(),
            updated_at: now,
            checkpoint_sequence: checkpoint.checkpoint_sequence,
        });
        self.graph = Some(graph);
        self.active_task_id = checkpoint.active_task_id.clone();
        self.completed_task_ids = checkpoint.completed_task_ids.clone();
        self.blocked_tasks = checkpoint.blocked_tasks.clone();
        self.skipped_tasks = checkpoint.skipped_tasks.clone();
        self.last_terminal_outcome = checkpoint.last_terminal_outcome.clone();

        Ok(self.run.as_ref().expect("just set"))
    }

    /// Check whether completion-verification is available (spec 079).
    ///
    /// Returns `false` when the verification runtime is not initialized,
    /// which should cause the orchestrator to pause with a
    /// `verification_unavailable` block.
    pub fn completion_verification_available(&self) -> bool {
        // Stub: spec 079 integration not yet wired.
        // When wired, check the session's completion-verification state.
        false
    }

    /// Derive a human-readable stop reason from the current blocked state.
    ///
    /// When one or more tasks are blocked, returns a summary string
    /// suitable for status output and resume guidance.
    pub fn pause_reason(&self) -> Option<String> {
        if self.blocked_tasks.is_empty() {
            return None;
        }
        let reasons: Vec<&str> = self.blocked_tasks.iter().map(|b| b.reason.as_str()).collect();
        let task_ids: Vec<&str> = self.blocked_tasks.iter().map(|b| b.task_id.as_str()).collect();
        Some(format!(
            "execution blocked on task(s) [{}] with reason(s): [{}]",
            task_ids.join(", "),
            reasons.join(", ")
        ))
    }

    /// Halt downstream dependents of a blocked task.
    ///
    /// Marks all tasks that transitively depend on `blocked_task_id` as
    /// not-runnable in the dependency graph, preventing `advance` from
    /// selecting them until the block is resolved.
    pub fn downstream_halt(&mut self, blocked_task_id: &str) {
        let graph = match self.graph.as_mut() {
            Some(g) => g,
            None => return,
        };

        // Collect all task IDs reachable from blocked_task_id.
        let mut halted: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut queue: Vec<String> = vec![blocked_task_id.to_string()];
        halted.insert(blocked_task_id.to_string());

        while let Some(current) = queue.pop() {
            for node in &graph.nodes {
                if node.depends_on.contains(&current) && !halted.contains(&node.task_id) {
                    halted.insert(node.task_id.clone());
                    queue.push(node.task_id.clone());
                }
            }
        }

        // Mark all halted tasks as not-runnable (skip the blocked task itself
        // which is already handled by record_outcome).
        for node in &mut graph.nodes {
            if halted.contains(&node.task_id) && node.task_id != blocked_task_id {
                node.runnable = false;
            }
        }
    }

    /// Resolve a blocked task and unblock its dependents.
    ///
    /// Removes the blocked-task record, marks the task as completed in
    /// the graph, and re-evaluates dependent runnability. Should be
    /// called after the operator resolves the blocking condition
    /// (e.g., governance approval, verification rerun).
    pub fn resolve_blocked_task(&mut self, task_id: &str) {
        // Remove the blocked record.
        self.blocked_tasks.retain(|b| b.task_id != task_id);

        // Mark the task as completed so dependents can unblock.
        self.completed_task_ids.push(task_id.to_string());
        if let Some(graph) = self.graph.as_mut() {
            graph.mark_completed(task_id);
        }
    }

    /// Populate execution projection fields on a [`SessionStatusView`].
    ///
    /// Called before status output to surface the current execution
    /// state. All eight execution fields are populated additively;
    /// when no run is active, all fields remain `None`.
    pub fn project_status(&self, view: &mut SessionStatusView) {
        let run = match self.run.as_ref() {
            Some(r) => r,
            None => return,
        };
        let graph = match self.graph.as_ref() {
            Some(g) => g,
            None => return,
        };

        view.execution_run_id = Some(run.run_id.clone());

        let state = if !self.blocked_tasks.is_empty() {
            ExecutionPlanState::Blocked
        } else if graph.next_runnable_task_id().is_none() {
            ExecutionPlanState::Completed
        } else if self.active_task_id.is_some() {
            ExecutionPlanState::Running
        } else {
            ExecutionPlanState::Ready
        };
        view.execution_plan_state = Some(state.as_str().to_string());

        view.execution_current_task_id = self.active_task_id.clone();
        view.execution_next_task_id = graph.next_runnable_task_id().map(String::from);
        view.execution_completed_task_count = Some(self.completed_task_ids.len());

        if !self.blocked_tasks.is_empty() {
            view.execution_blocked_task_ids =
                Some(self.blocked_tasks.iter().map(|b| b.task_id.clone()).collect());
        }

        view.execution_checkpoint_ref =
            Some(format!(".boundline/execution/checkpoints/{}.json", run.run_id));
        view.execution_resume_command = Some(format!("boundline run --resume {}", run.run_id));
    }

    /// Return a reference to the current execution run, if started.
    pub fn current_run(&self) -> Option<&ExecutionRun> {
        self.run.as_ref()
    }

    /// Return a reference to the current blocked tasks.
    pub fn blocked_tasks(&self) -> &[BlockedTaskRecord] {
        &self.blocked_tasks
    }

    /// Return a reference to the dependency graph, if built.
    pub fn current_graph(&self) -> Option<&TaskDependencyGraph> {
        self.graph.as_ref()
    }

    /// Persist the current execution state as an atomic checkpoint.
    ///
    /// Writes to a temp file, flushes, syncs, then renames atomically
    /// over the canonical checkpoint path. On failure, the previous
    /// checkpoint (if any) is preserved.
    pub fn write_checkpoint(
        &self,
        params: &CheckpointWriteParams<'_>,
    ) -> Result<(), ExecutionOrchestratorError> {
        let run = self.run.as_ref().expect("run must be started before checkpoint");
        let workspace = Path::new(&run.workspace_ref);

        let checkpoint = ExecutionCheckpoint {
            schema_version: crate::domain::execution_orchestration::CHECKPOINT_SCHEMA_VERSION
                .to_string(),
            run_id: run.run_id.clone(),
            checkpoint_sequence: run.checkpoint_sequence,
            plan_ref: run.plan_ref.clone(),
            plan_fingerprint: run.plan_fingerprint.clone(),
            workspace_ref: run.workspace_ref.clone(),
            execution_state: params.state,
            active_task_id: params.active_task_id.map(String::from),
            next_runnable_task_id: params.next_runnable.map(String::from),
            completed_task_ids: params.completed_ids.to_vec(),
            blocked_tasks: params.blocked.to_vec(),
            skipped_tasks: params.skipped.to_vec(),
            last_terminal_outcome: params.last_outcome.cloned(),
            checkpoint_reason: params.reason,
            created_at: current_timestamp_iso(),
        };

        let checkpoint_dir = workspace.join(".boundline").join("execution").join(CHECKPOINT_DIR);
        fs::create_dir_all(&checkpoint_dir)?;

        let canonical_path = checkpoint_dir.join(format!("{}.json", run.run_id));
        let tmp_path = checkpoint_dir.join(format!("{}.json.tmp", run.run_id));

        let json = serde_json::to_string_pretty(&checkpoint)?;

        // Write to temp file.
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(json.as_bytes())?;
            f.flush()?;
            f.sync_all()?;
        }

        // Atomic rename.
        fs::rename(&tmp_path, &canonical_path)?;

        Ok(())
    }

    /// Read and validate a checkpoint from disk.
    ///
    /// Returns the deserialized checkpoint if the schema is compatible.
    /// Rejects checkpoints with an unknown or unsupported schema version.
    pub fn read_checkpoint(
        &self,
        workspace: &Path,
        run_id: &str,
        expected_fingerprint: Option<&str>,
    ) -> Result<ExecutionCheckpoint, ExecutionOrchestratorError> {
        let checkpoint_path = workspace
            .join(".boundline")
            .join("execution")
            .join(CHECKPOINT_DIR)
            .join(format!("{run_id}.json"));

        if !checkpoint_path.exists() {
            return Err(ExecutionOrchestratorError::CheckpointNotFound {
                path: checkpoint_path.to_string_lossy().into_owned(),
            });
        }

        let raw = fs::read_to_string(&checkpoint_path)?;
        let checkpoint: ExecutionCheckpoint = serde_json::from_str(&raw)?;

        // Validate schema version.
        if checkpoint.schema_version
            != crate::domain::execution_orchestration::CHECKPOINT_SCHEMA_VERSION
        {
            return Err(ExecutionOrchestratorError::IncompatibleCheckpoint {
                expected: crate::domain::execution_orchestration::CHECKPOINT_SCHEMA_VERSION
                    .to_string(),
                found: checkpoint.schema_version,
            });
        }

        // Optionally validate plan fingerprint.
        if let Some(expected) = expected_fingerprint
            && checkpoint.plan_fingerprint != expected
        {
            return Err(ExecutionOrchestratorError::IncompatibleCheckpoint {
                expected: expected.to_string(),
                found: checkpoint.plan_fingerprint,
            });
        }

        Ok(checkpoint)
    }
}

/// Populate execution projection fields on a [`SessionStatusView`] from a
/// checkpoint on disk. This is a lightweight convenience for status surfaces
/// that need execution visibility without holding an active orchestrator.
pub fn populate_execution_projection_from_checkpoint(
    workspace: &Path,
    run_id: &str,
    view: &mut SessionStatusView,
) {
    let checkpoint_path = workspace
        .join(".boundline")
        .join("execution")
        .join(CHECKPOINT_DIR)
        .join(format!("{run_id}.json"));

    let checkpoint: ExecutionCheckpoint = match std::fs::read_to_string(&checkpoint_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
    {
        Some(cp) => cp,
        None => {
            // Degraded projection: checkpoint unreadable or missing.
            // The session linkage (active_execution_run_id) is preserved;
            // only the projection fields remain empty.
            tracing::warn!(
                checkpoint_path = %checkpoint_path.display(),
                "execution checkpoint unreadable; producing degraded projection"
            );
            return;
        }
    };

    view.execution_run_id = Some(run_id.to_string());
    view.execution_plan_state = Some(checkpoint.execution_state.as_str().to_string());
    view.execution_current_task_id = checkpoint.active_task_id.clone();
    view.execution_next_task_id = checkpoint.next_runnable_task_id.clone();
    view.execution_completed_task_count = Some(checkpoint.completed_task_ids.len());

    if !checkpoint.blocked_tasks.is_empty() {
        view.execution_blocked_task_ids =
            Some(checkpoint.blocked_tasks.iter().map(|b| b.task_id.clone()).collect());
    }

    view.execution_checkpoint_ref = Some(checkpoint_path.to_string_lossy().into_owned());
    view.execution_resume_command = Some(format!("boundline run --resume {run_id}"));
}

impl Default for ExecutionOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a stable fingerprint for the plan content.
fn compute_plan_fingerprint(plan: &GoalPlan) -> String {
    use std::hash::{Hash, Hasher};
    let json = serde_json::to_string(&plan.tasks).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    json.hash(&mut hasher);
    format!("sha256:{:016x}", hasher.finish())
}

/// Generate a run ID in the format `ER-{YYYYMMDD}-{random6}`.
fn generate_run_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    // Derive a simple date from epoch seconds.
    let days = secs / 86400;
    // Days since epoch to YYYYMMDD (approximate, but good enough for IDs).
    let (y, m, d) = epoch_days_to_ymd(days);
    let random = (secs % 1_000_000) as u32;
    format!("ER-{y:04}{m:02}{d:02}-{random:06x}")
}

/// Convert days since Unix epoch to (year, month, day).
fn epoch_days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Simplified Gregorian calendar conversion.
    let mut d = days;
    let mut y = 1970u64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let month_days: [u64; 12] =
        [31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }
    (y, m, d + 1)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

/// Return an ISO 8601 timestamp for the current moment.
fn current_timestamp_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let (y, m, d) = epoch_days_to_ymd(secs / 86400);
    let remaining = secs % 86400;
    let h = remaining / 3600;
    let min = (remaining % 3600) / 60;
    let s = remaining % 60;
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}
