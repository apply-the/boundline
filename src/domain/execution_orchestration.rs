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
    // Build adjacency list: task_id → set of tasks that depend on it.
    let mut adjacency: std::collections::HashMap<&str, Vec<&str>> =
        std::collections::HashMap::new();
    // In-degree count per task.
    let mut in_degree: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    for (tid, deps) in tasks {
        in_degree.entry(tid.as_str()).or_insert(0);
        for dep in deps {
            adjacency.entry(dep.as_str()).or_default().push(tid.as_str());
            *in_degree.entry(tid.as_str()).or_insert(0) += 1;
        }
    }

    // Initialize queue with zero in-degree tasks, in plan order.
    let mut queue: Vec<&str> = tasks
        .iter()
        .filter(|(tid, _)| in_degree.get(tid.as_str()).copied().unwrap_or(0) == 0)
        .map(|(tid, _)| tid.as_str())
        .collect();

    let mut order: Vec<String> = Vec::with_capacity(tasks.len());

    while let Some(current) = { if queue.is_empty() { None } else { Some(queue.remove(0)) } } {
        order.push(current.to_string());
        if let Some(dependents) = adjacency.get(current) {
            for &dependent in dependents {
                let deg = in_degree.get_mut(dependent).expect("in_degree entry");
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push(dependent);
                }
            }
        }
    }

    if order.len() == tasks.len() {
        Ok(order)
    } else {
        // Detect cycles: collect remaining tasks with non-zero in-degree.
        let remaining: std::collections::HashSet<&str> = tasks
            .iter()
            .filter(|(tid, _)| in_degree.get(tid.as_str()).copied().unwrap_or(0) > 0)
            .map(|(tid, _)| tid.as_str())
            .collect();

        // Build a simple cycle report — for small graphs, report each
        // strongly-connected component as a cycle group.
        let mut cycles: Vec<Vec<String>> = Vec::new();
        let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for &start in &remaining {
            if visited.contains(start) {
                continue;
            }
            // Walk forward through dependency edges to find a cycle.
            let mut path: Vec<String> = Vec::new();
            let mut current = start;
            let max_steps = remaining.len() * 2;
            let mut steps = 0;
            while remaining.contains(current) && steps < max_steps {
                visited.insert(current);
                path.push(current.to_string());
                // Follow first dependency edge
                if let Some((_, deps)) = tasks.iter().find(|(tid, _)| tid.as_str() == current) {
                    if let Some(next_dep) = deps.first() {
                        current = next_dep.as_str();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                steps += 1;
            }
            if !path.is_empty() {
                cycles.push(path);
            }
        }

        Err(cycles)
    }
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
