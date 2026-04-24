# Contract: Diagnostics Report

## Purpose

Defines the readiness report emitted by `synod doctor`.

## Report Shape

| Field | Required | Description |
|-------|----------|-------------|
| `workspace_ref` | Yes | Workspace that was checked |
| `ready` | Yes | Whether the command surface can proceed without a blocking setup issue |
| `checks` | Yes | Ordered readiness checks with name, status, and actionable message |
| `missing_prerequisites` | No | Blocking prerequisites detected during validation |
| `suggested_actions` | No | Human-readable follow-up guidance |

## Check Shape

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Stable readiness-check identifier |
| `status` | Yes | `passed` or `failed` |
| `message` | Yes | Human-readable result and next action if needed |

## Behavioral Guarantees

- The report covers every local prerequisite required for `demo`, `run`, and `inspect`.
- Every failed check includes an actionable message.
- `ready = true` means the developer command surface can proceed without a known blocking setup problem.
- The report is safe to run repeatedly and does not mutate task state or traces.