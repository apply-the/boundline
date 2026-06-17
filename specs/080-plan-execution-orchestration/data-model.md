# Data Model: Plan Execution Orchestration

## Core Entities

### ExecutionRun

A distinct execution run linked to a session for the duration of plan execution.

```rust
struct ExecutionRun {
    run_id: String,              // "ER-{YYYYMMDD}-{random6}"
    plan_ref: String,            // Path to accepted plan JSON
    plan_fingerprint: String,    // SHA-256 of plan content
    workspace_ref: String,       // Absolute workspace path
    session_id: String,          // Owning session
    created_at: String,          // ISO 8601
    updated_at: String,          // ISO 8601
    checkpoint_sequence: u32,    // Monotonically increasing
}
```

### ExecutionCheckpoint

The canonical resumable execution state at `.boundline/execution/checkpoints/<run-id>.json`.

```json
{
  "schema_version": "1",
  "run_id": "ER-20260614-abc123",
  "checkpoint_sequence": 7,
  "plan_ref": ".boundline/plans/P-123/accepted-plan.json",
  "plan_fingerprint": "sha256:a1b2c3...",
  "workspace_ref": "/workspace/project",
  "execution_state": "blocked",
  "active_task_id": null,
  "next_runnable_task_id": "T-008",
  "completed_task_ids": ["T-001", "T-002"],
  "blocked_tasks": [
    {
      "task_id": "T-007",
      "reason": "completion_proof_failed",
      "evidence_ref": ".boundline/traces/proof-T-007.json"
    }
  ],
  "skipped_tasks": [],
  "last_terminal_outcome": {
    "task_id": "T-007",
    "outcome": "blocked"
  },
  "checkpoint_reason": "task_terminal_outcome",
  "created_at": "2026-06-14T12:00:00Z"
}
```

### ExecutionPlanState

Lifecycle states for an execution run.

| State | Meaning |
|-------|---------|
| `ready` | Plan loaded, validated, not yet started |
| `running` | Actively dispatching tasks |
| `paused` | Interrupted, resumable from checkpoint |
| `blocked` | Halted on a blocking finding |
| `completed` | All tasks reached terminal state |

### TaskDependencyGraph

Validated dependency graph derived from `depends_on` declarations.

```rust
struct TaskDependencyGraph {
    nodes: Vec<TaskNode>,          // All tasks in topological order
    root_tasks: Vec<String>,       // Tasks with no prerequisites
    cycles: Vec<Vec<String>>,      // Detected cycles (blocks execution)
    missing_references: Vec<String>, // Depends on IDs not in plan
    self_dependencies: Vec<String>, // Tasks depending on themselves
}
```

### PlannedTask Extension

Existing `PlannedTask` struct extended with:

```rust
struct PlannedTask {
    task_id: String,
    description: String,
    target: String,
    expected_outcome: Option<String>,
    decision_type_hint: Option<DecisionType>,
    depends_on: Option<Vec<String>>,  // NEW: direct prerequisite task IDs
}
```

### Session Extension

`ActiveSessionRecord` extended with:

```rust
struct ActiveSessionRecord {
    // ... existing fields ...
    active_execution_run_id: Option<String>,  // NEW
}
```

### SessionStatusView Extension

`SessionStatusView` extended with additive execution projection fields:

```rust
struct SessionStatusView {
    // ... existing fields ...
    execution_run_id: Option<String>,           // NEW
    execution_plan_state: Option<String>,        // NEW: ready|running|paused|blocked|completed
    execution_current_task_id: Option<String>,   // NEW
    execution_next_task_id: Option<String>,      // NEW
    execution_completed_task_count: Option<u32>, // NEW
    execution_blocked_task_ids: Option<Vec<String>>, // NEW
    execution_checkpoint_ref: Option<String>,    // NEW
    execution_resume_command: Option<String>,    // NEW
}
```

All new fields use `#[serde(default, skip_serializing_if = "Option::is_none")]`.
