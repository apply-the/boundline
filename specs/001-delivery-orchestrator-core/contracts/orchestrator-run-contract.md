# Contract: Orchestrator Run

## Purpose

Defines the library-facing contract for starting and completing one bounded orchestrator task run.

## Consumers

- Future Synod delivery flows
- CLI or editor commands that submit bounded engineering tasks
- Integration tests using fake endpoints

## Request Shape

| Field | Required | Description |
|-------|----------|-------------|
| `goal` | Yes | Human-readable task objective |
| `input` | Yes | Structured task input payload |
| `session_id` | Yes | Identifier for the task session |
| `workspace_ref` | Yes | Workspace path or equivalent workspace reference |
| `limits` | Yes | Maximum steps, retries, replans, and terminal precedence |
| `initial_context` | No | Preloaded state made available before planning begins |

## Response Shape

| Field | Required | Description |
|-------|----------|-------------|
| `task_id` | Yes | Stable identifier for the run |
| `terminal_status` | Yes | `succeeded`, `failed`, `exhausted`, or `aborted` |
| `terminal_reason` | Yes | Structured explanation of the final outcome |
| `final_context` | Yes | Task context after the last executed step |
| `plan_revision` | Yes | Final active plan revision number |
| `trace_location` | Yes | Inspectable persisted trace location |

## Behavioral Guarantees

- The orchestrator executes at most one step at a time.
- Every run ends in one explicit terminal status.
- Every retry and replanning decision is recorded in the trace before control returns to the caller.
- Completed step history remains visible in the final context and trace even if the task fails.

## Failure Contract

- Invalid task requests fail before execution begins and do not create a partial run.
- Missing endpoint registrations fail the affected step and must surface a structured terminal or recovery decision.
- Exhausted step, retry, or replanning budgets must return `exhausted` with the budget breach recorded in the terminal reason.
