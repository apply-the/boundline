# Quickstart: Plan Execution Orchestration

## Overview

`boundline run --plan` executes an accepted multi-task plan one task at a time in dependency order, checkpointing after each terminal outcome. Blocked tasks halt downstream execution until resolved. Resume picks up from the last checkpoint.

## Prerequisites

- Spec 079 (completion-verification runtime) must be active.
- An accepted plan with explicit `depends_on` declarations per task.

## Basic Usage

```bash
# Start executing an accepted plan
boundline run --plan .boundline/plans/P-123/accepted-plan.json

# Use the session-attached accepted plan
boundline run --accepted-plan

# Resume an interrupted run
boundline run --resume ER-20260614-abc123

# Normal single-task run (unchanged)
boundline run --goal "Fix the failing test"
```

## Accepted Plan Format

Each task must declare its direct dependencies:

```json
{
  "tasks": [
    {
      "task_id": "T-001",
      "title": "Analyze the codebase",
      "depends_on": []
    },
    {
      "task_id": "T-002",
      "title": "Refactor the data layer",
      "depends_on": ["T-001"]
    },
    {
      "task_id": "T-003",
      "title": "Add integration tests",
      "depends_on": ["T-001", "T-002"]
    }
  ]
}
```

## Inspecting Execution State

```bash
boundline status
# Output includes:
#   execution_run_id: ER-20260614-abc123
#   execution_plan_state: blocked
#   execution_current_task_id: T-003
#   execution_blocked_task_ids: [T-002]
#   execution_resume_command: boundline run --resume ER-20260614-abc123

boundline inspect
# Shows dependency graph, completed tasks, blocked tasks with evidence refs
```

## Checkpoint Location

```
.boundline/execution/checkpoints/ER-20260614-abc123.json
```
