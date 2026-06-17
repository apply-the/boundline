//! Plan execution orchestration models.
//!
//! Defines the typed domain model for multi-task plan execution:
//! `ExecutionRun` identity, `ExecutionCheckpoint` persistence,
//! `TaskDependencyGraph` validation, execution state, blocked-task
//! records, and terminal outcomes.
//!
//! All models use typed serde structs/enums per Boundline language rules
//! — no ad hoc `serde_json::Map` assembly or raw key strings.

use serde::{Deserialize, Serialize};

/// Schema version for the checkpoint file format.
pub const CHECKPOINT_SCHEMA_VERSION: &str = "1";

/// Lifecycle state of an execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPlanState {
    Ready,
    Running,
    Paused,
    Blocked,
    Completed,
}

impl ExecutionPlanState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
        }
    }
}

/// Outcome of a single task terminal transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskOutcome {
    Completed,
    Blocked,
    Skipped,
    Deferred,
    Failed,
}

impl TaskOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Blocked => "blocked",
            Self::Skipped => "skipped",
            Self::Deferred => "deferred",
            Self::Failed => "failed",
        }
    }
}

/// Reason a checkpoInt was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointReason {
    TaskTerminalOutcome,
    PauseRequested,
    Interrupted,
    Blocked,
}

impl CheckpointReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TaskTerminalOutcome => "task_terminal_outcome",
            Self::PauseRequested => "pause_requested",
            Self::Interrupted => "interrupted",
            Self::Blocked => "blocked",
        }
    }
}

/// A single blocked task record in the checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedTaskRecord {
    pub task_id: String,
    pub reason: String,
    pub evidence_ref: Option<String>,
}

/// The last terminal outcome recorded in a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalOutcome {
    pub task_id: String,
    pub outcome: TaskOutcome,
}

/// Identity and metadata for a single execution run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRun {
    pub run_id: String,
    pub plan_ref: String,
    pub plan_fingerprint: String,
    pub workspace_ref: String,
    pub session_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub checkpoint_sequence: u32,
}

/// Canonical resumable execution state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub schema_version: String,
    pub run_id: String,
    pub checkpoint_sequence: u32,
    pub plan_ref: String,
    pub plan_fingerprint: String,
    pub workspace_ref: String,
    pub execution_state: ExecutionPlanState,
    pub active_task_id: Option<String>,
    pub next_runnable_task_id: Option<String>,
    pub completed_task_ids: Vec<String>,
    pub blocked_tasks: Vec<BlockedTaskRecord>,
    pub skipped_tasks: Vec<String>,
    pub last_terminal_outcome: Option<TerminalOutcome>,
    pub checkpoint_reason: CheckpointReason,
    pub created_at: String,
}

/// A node in the dependency graph, representing one plan task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskNode {
    pub task_id: String,
    pub depends_on: Vec<String>,
    pub runnable: bool,
}

/// Validated dependency graph for plan execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDependencyGraph {
    pub nodes: Vec<TaskNode>,
    pub topological_order: Vec<String>,
    pub cycles: Vec<Vec<String>>,
    pub missing_references: Vec<String>,
    pub self_dependencies: Vec<String>,
    /// Set of task IDs that have been marked completed.
    completed_ids: std::collections::HashSet<String>,
}

/// Errors that can occur during dependency graph construction.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DependencyGraphError {
    /// One or more tasks reference themselves in `depends_on`.
    #[error("self-dependency detected for task(s): {task_ids:?}")]
    SelfDependency { task_ids: Vec<String> },
    /// One or more tasks reference a task ID not present in the plan.
    #[error("missing dependency reference(s): {task_ids:?}")]
    MissingReference { task_ids: Vec<String> },
    /// A cycle was detected among the dependency edges.
    #[error("dependency cycle detected: {cycles:?}")]
    CycleDetected { cycles: Vec<Vec<String>> },
}

impl TaskDependencyGraph {
    /// Build a validated dependency graph from a goal plan's task list.
    ///
    /// Performs duplicate normalization, self-dependency detection,
    /// missing-reference validation, and cycle detection via Kahn's algorithm.
    /// Returns `Ok(graph)` when the plan is valid, or `Err(error)` with
    /// details for the first validation failure encountered.
    pub fn from_plan(
        plan: &crate::domain::goal_plan::GoalPlan,
    ) -> Result<Self, DependencyGraphError> {
        let planned_tasks = &plan.tasks;

        // Collect all task IDs from the plan.
        let plan_task_ids: std::collections::HashSet<&str> =
            planned_tasks.iter().map(|t| t.task_id.as_str()).collect();

        // --- Duplicate normalization ---
        // Deduplicate depends_on entries within each task.
        let normalized_deps: Vec<(String, Vec<String>)> = planned_tasks
            .iter()
            .map(|t| {
                let mut deps: Vec<String> = t.depends_on.as_deref().unwrap_or(&[]).to_vec();
                deps.sort();
                deps.dedup();
                (t.task_id.clone(), deps)
            })
            .collect();

        // --- Self-dependency detection ---
        let self_deps: Vec<String> = normalized_deps
            .iter()
            .filter_map(|(tid, deps)| if deps.contains(tid) { Some(tid.clone()) } else { None })
            .collect();
        if !self_deps.is_empty() {
            return Err(DependencyGraphError::SelfDependency { task_ids: self_deps });
        }

        // --- Missing reference detection ---
        let mut missing_refs: Vec<String> = Vec::new();
        for (_tid, deps) in &normalized_deps {
            for dep in deps {
                if !plan_task_ids.contains(dep.as_str()) && !missing_refs.contains(dep) {
                    missing_refs.push(dep.clone());
                }
            }
        }
        if !missing_refs.is_empty() {
            return Err(DependencyGraphError::MissingReference { task_ids: missing_refs });
        }

        // --- Topological sort via Kahn's algorithm ---
        let topo = kahn_topological_sort(&normalized_deps);

        match topo {
            Ok(order) => {
                // Build nodes in topological order.
                let dep_map: std::collections::HashMap<&str, &[String]> = normalized_deps
                    .iter()
                    .map(|(tid, deps)| (tid.as_str(), deps.as_slice()))
                    .collect();
                let nodes: Vec<TaskNode> = order
                    .iter()
                    .map(|tid| {
                        let deps = dep_map.get(tid.as_str()).copied().unwrap_or(&[]);
                        TaskNode {
                            task_id: tid.clone(),
                            depends_on: deps.to_vec(),
                            runnable: deps.is_empty(),
                        }
                    })
                    .collect();
                Ok(TaskDependencyGraph {
                    nodes,
                    topological_order: order,
                    cycles: Vec::new(),
                    missing_references: Vec::new(),
                    self_dependencies: Vec::new(),
                    completed_ids: std::collections::HashSet::new(),
                })
            }
            Err(cycles) => Err(DependencyGraphError::CycleDetected { cycles }),
        }
    }

    /// Returns the next runnable task ID that respects the topological
    /// order when multiple tasks are simultaneously ready, preferring
    /// the one that appears first in plan order.
    pub fn next_runnable_task_id(&self) -> Option<&str> {
        self.nodes.iter().find(|node| node.runnable).map(|node| node.task_id.as_str())
    }

    /// Marks a task as completed and recomputes runnable status for
    /// tasks that depended on it.
    pub fn mark_completed(&mut self, task_id: &str) {
        self.completed_ids.insert(task_id.to_string());

        // Mark the completed task as not-runnable.
        if let Some(node) = self.nodes.iter_mut().find(|n| n.task_id == task_id) {
            node.runnable = false;
        }

        // Unblock dependents whose prerequisites are all satisfied,
        // skipping already-completed tasks.
        for node in &mut self.nodes {
            if node.runnable
                || node.depends_on.is_empty()
                || self.completed_ids.contains(node.task_id.as_str())
            {
                continue;
            }
            let all_deps_satisfied =
                node.depends_on.iter().all(|dep| self.completed_ids.contains(dep.as_str()));
            if all_deps_satisfied {
                node.runnable = true;
            }
        }
    }
}

/// Run Kahn's algorithm for topological sort with cycle detection.
///
/// Accepts a slice of `(task_id, depends_on)` pairs where `depends_on`
/// entries are deduplicated and sorted. Returns the topological order
/// or detected cycles.
fn kahn_topological_sort(tasks: &[(String, Vec<String>)]) -> Result<Vec<String>, Vec<Vec<String>>> {
    let (adjacency, mut in_degree) = build_dependency_index(tasks);
    let mut queue = build_zero_in_degree_queue(tasks, &in_degree);
    let order = drain_topological_queue(tasks.len(), &adjacency, &mut in_degree, &mut queue);

    if order.len() == tasks.len() {
        return Ok(order);
    }

    Err(detect_cycle_groups(tasks, &in_degree))
}

type TaskAdjacency<'a> = std::collections::HashMap<&'a str, Vec<&'a str>>;
type TaskInDegree<'a> = std::collections::HashMap<&'a str, usize>;
type RemainingTaskIds<'a> = std::collections::HashSet<&'a str>;
type ReadyTaskQueue<'a> = std::collections::VecDeque<&'a str>;

/// Multiplier used to cap degraded cycle walks in malformed graphs.
const CYCLE_SCAN_STEP_MULTIPLIER: usize = 2;

fn build_dependency_index<'a>(
    tasks: &'a [(String, Vec<String>)],
) -> (TaskAdjacency<'a>, TaskInDegree<'a>) {
    let mut adjacency = TaskAdjacency::new();
    let mut in_degree = TaskInDegree::new();

    for (task_id, dependencies) in tasks {
        in_degree.entry(task_id.as_str()).or_insert(0);
        for dependency in dependencies {
            adjacency.entry(dependency.as_str()).or_default().push(task_id.as_str());
            *in_degree.entry(task_id.as_str()).or_insert(0) += 1;
        }
    }

    (adjacency, in_degree)
}

fn build_zero_in_degree_queue<'a>(
    tasks: &'a [(String, Vec<String>)],
    in_degree: &TaskInDegree<'a>,
) -> ReadyTaskQueue<'a> {
    tasks
        .iter()
        .filter(|(task_id, _)| in_degree.get(task_id.as_str()).copied().unwrap_or(0) == 0)
        .map(|(task_id, _)| task_id.as_str())
        .collect()
}

fn drain_topological_queue<'a>(
    task_count: usize,
    adjacency: &TaskAdjacency<'a>,
    in_degree: &mut TaskInDegree<'a>,
    queue: &mut ReadyTaskQueue<'a>,
) -> Vec<String> {
    let mut order = Vec::with_capacity(task_count);

    while let Some(current) = queue.pop_front() {
        order.push(current.to_string());
        release_dependents(current, adjacency, in_degree, queue);
    }

    order
}

fn release_dependents<'a>(
    current: &'a str,
    adjacency: &TaskAdjacency<'a>,
    in_degree: &mut TaskInDegree<'a>,
    queue: &mut ReadyTaskQueue<'a>,
) {
    let Some(dependents) = adjacency.get(current) else {
        return;
    };

    for &dependent in dependents {
        decrement_in_degree(dependent, in_degree, queue);
    }
}

fn decrement_in_degree<'a>(
    dependent: &'a str,
    in_degree: &mut TaskInDegree<'a>,
    queue: &mut ReadyTaskQueue<'a>,
) {
    let Some(degree) = in_degree.get_mut(dependent) else {
        return;
    };

    *degree = degree.saturating_sub(1);
    if *degree == 0 {
        queue.push_back(dependent);
    }
}

fn detect_cycle_groups<'a>(
    tasks: &'a [(String, Vec<String>)],
    in_degree: &TaskInDegree<'a>,
) -> Vec<Vec<String>> {
    let remaining = collect_remaining_tasks(tasks, in_degree);
    let mut visited = RemainingTaskIds::new();
    let mut cycles = Vec::new();

    for &start in &remaining {
        if let Some(path) = follow_cycle_group(tasks, &remaining, &mut visited, start) {
            cycles.push(path);
        }
    }

    cycles
}

fn collect_remaining_tasks<'a>(
    tasks: &'a [(String, Vec<String>)],
    in_degree: &TaskInDegree<'a>,
) -> RemainingTaskIds<'a> {
    tasks
        .iter()
        .filter(|(task_id, _)| in_degree.get(task_id.as_str()).copied().unwrap_or(0) > 0)
        .map(|(task_id, _)| task_id.as_str())
        .collect()
}

fn follow_cycle_group<'a>(
    tasks: &'a [(String, Vec<String>)],
    remaining: &RemainingTaskIds<'a>,
    visited: &mut RemainingTaskIds<'a>,
    start: &'a str,
) -> Option<Vec<String>> {
    if visited.contains(start) {
        return None;
    }

    let mut path = Vec::new();
    let mut current = start;
    let max_steps = remaining.len() * CYCLE_SCAN_STEP_MULTIPLIER;
    let mut steps = 0;

    while remaining.contains(current) && steps < max_steps {
        visited.insert(current);
        path.push(current.to_string());

        let Some(next_dependency) = first_dependency_for(tasks, current) else {
            break;
        };
        current = next_dependency;
        steps += 1;
    }

    (!path.is_empty()).then_some(path)
}

fn first_dependency_for<'a>(tasks: &'a [(String, Vec<String>)], task_id: &str) -> Option<&'a str> {
    tasks
        .iter()
        .find(|(candidate_id, _)| candidate_id.as_str() == task_id)
        .and_then(|(_, dependencies)| dependencies.first())
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_plan_state_as_str() {
        assert_eq!(ExecutionPlanState::Ready.as_str(), "ready");
        assert_eq!(ExecutionPlanState::Running.as_str(), "running");
        assert_eq!(ExecutionPlanState::Paused.as_str(), "paused");
        assert_eq!(ExecutionPlanState::Blocked.as_str(), "blocked");
        assert_eq!(ExecutionPlanState::Completed.as_str(), "completed");
    }

    #[test]
    fn task_outcome_as_str() {
        assert_eq!(TaskOutcome::Completed.as_str(), "completed");
        assert_eq!(TaskOutcome::Blocked.as_str(), "blocked");
        assert_eq!(TaskOutcome::Skipped.as_str(), "skipped");
        assert_eq!(TaskOutcome::Deferred.as_str(), "deferred");
        assert_eq!(TaskOutcome::Failed.as_str(), "failed");
    }

    #[test]
    fn checkpoint_reason_as_str() {
        assert_eq!(CheckpointReason::TaskTerminalOutcome.as_str(), "task_terminal_outcome");
        assert_eq!(CheckpointReason::PauseRequested.as_str(), "pause_requested");
        assert_eq!(CheckpointReason::Interrupted.as_str(), "interrupted");
        assert_eq!(CheckpointReason::Blocked.as_str(), "blocked");
    }

    #[test]
    fn execution_run_serde_roundtrip() {
        let run = ExecutionRun {
            run_id: "ER-20260614-abc123".into(),
            plan_ref: ".boundline/plans/P-123/accepted-plan.json".into(),
            plan_fingerprint: "sha256:abc".into(),
            workspace_ref: "/workspace".into(),
            session_id: "S-123".into(),
            created_at: "2026-06-14T12:00:00Z".into(),
            updated_at: "2026-06-14T12:00:00Z".into(),
            checkpoint_sequence: 0,
        };
        let json = serde_json::to_string(&run).unwrap();
        let parsed: ExecutionRun = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.run_id, run.run_id);
    }

    #[test]
    fn checkpoint_serde_roundtrip() {
        let checkpoint = ExecutionCheckpoint {
            schema_version: CHECKPOINT_SCHEMA_VERSION.into(),
            run_id: "ER-abc".into(),
            checkpoint_sequence: 1,
            plan_ref: ".boundline/plans/P-1/plan.json".into(),
            plan_fingerprint: "sha256:def".into(),
            workspace_ref: "/ws".into(),
            execution_state: ExecutionPlanState::Running,
            active_task_id: Some("T-001".into()),
            next_runnable_task_id: Some("T-002".into()),
            completed_task_ids: vec![],
            blocked_tasks: vec![],
            skipped_tasks: vec![],
            last_terminal_outcome: None,
            checkpoint_reason: CheckpointReason::TaskTerminalOutcome,
            created_at: "2026-06-14T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&checkpoint).unwrap();
        let parsed: ExecutionCheckpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.run_id, checkpoint.run_id);
        assert_eq!(parsed.checkpoint_sequence, 1);
    }

    #[test]
    fn task_node_defaults_to_not_runnable() {
        let node = TaskNode { task_id: "T-001".into(), depends_on: vec![], runnable: false };
        assert!(!node.runnable);
        assert!(node.depends_on.is_empty());
    }
}
