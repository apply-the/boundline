# Contract: Endpoint Execution

## Purpose

Defines the shared execution envelope used by agent steps and tool steps.

## Request Shape

| Field | Required | Description |
|-------|----------|-------------|
| `step_id` | Yes | Identifier of the step being executed |
| `step_kind` | Yes | `agent` or `tool` |
| `target_name` | Yes | Registry key of the selected endpoint |
| `input` | Yes | Structured step input payload |
| `task_snapshot` | Yes | Read-only snapshot of the current task context |
| `attempt_number` | Yes | Current attempt count for the step |

## Response Shape

| Field | Required | Description |
|-------|----------|-------------|
| `status` | Yes | `succeeded` or `failed` |
| `output` | No | Structured result when execution succeeds |
| `error` | No | Structured failure data when execution fails |
| `recoverability` | Yes | `retryable`, `replan_required`, or `terminal` |
| `evidence` | No | Diagnostic details to include in the execution trace |

## Behavioral Guarantees

- Endpoints must not mutate the task context directly; they return data for the orchestrator to merge.
- `recoverability` is advisory input to the recovery policy, not a direct instruction to bypass orchestrator rules.
- Successful responses must provide output that can be recorded in the step history.
- Failed responses must provide enough structured detail to explain retry or replanning decisions.

## Error Semantics

- Missing `target_name` or unknown endpoint kind is a contract violation.
- A response is invalid if both `output` and `error` are omitted.
- A response is invalid if `status` is `succeeded` and `error` is populated.
