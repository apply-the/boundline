# Execution Orchestration Projection Contract

## CLI Contract: `boundline run` Extensions

### Synopsis

```
boundline run --plan <plan-ref>       # Start plan execution
boundline run --accepted-plan         # Start from session-attached plan
boundline run --resume <run-id>       # Resume from checkpoint
boundline run                         # Preserve existing single-task behavior
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tasks completed successfully |
| 1 | Execution blocked or failed |
| 2 | Invalid plan (cycle, missing deps, unaccepted) |
| 3 | Checkpoint read/write failure |
| 4 | Completion verification unavailable |

### Session Projection Contract

When an execution run is active, `SessionStatusView` extends with:

| Field | Type | When Present |
|-------|------|-------------|
| `execution_run_id` | string | Always when execution run is linked |
| `execution_plan_state` | string | `ready`, `running`, `paused`, `blocked`, `completed` |
| `execution_current_task_id` | string | When a task is locked for execution |
| `execution_next_task_id` | string | When a runnable task is queued |
| `execution_completed_task_count` | u32 | After at least one task completes |
| `execution_blocked_task_ids` | [string] | When tasks are blocked |
| `execution_checkpoint_ref` | string | After first checkpoint is written |
| `execution_resume_command` | string | When paused or blocked |

All fields are additive and must not redefine existing session states.

### Checkpoint Schema

```json
{
  "schema_version": "1",
  "run_id": "string",
  "checkpoint_sequence": "u32",
  "plan_ref": "string",
  "plan_fingerprint": "string",
  "workspace_ref": "string",
  "execution_state": "ready|running|paused|blocked|completed",
  "active_task_id": "string|null",
  "next_runnable_task_id": "string|null",
  "completed_task_ids": ["string"],
  "blocked_tasks": [
    {
      "task_id": "string",
      "reason": "string",
      "evidence_ref": "string|null"
    }
  ],
  "skipped_tasks": ["string"],
  "last_terminal_outcome": {
    "task_id": "string",
    "outcome": "completed|blocked|skipped|deferred|failed"
  },
  "checkpoint_reason": "task_terminal_outcome|pause_requested|interrupted|blocked",
  "created_at": "ISO8601"
}
```
