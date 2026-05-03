# Contract: Session Record Surface

## Purpose

Defines the persisted shape and behavioral guarantees of the workspace-scoped session record stored for active Boundline work.

## Storage Location

- The active session record MUST live at `<workspace>/.boundline/session.json`.
- The file MUST be human-readable JSON.
- The session record MUST remain local to the workspace and MUST NOT require an external service.

## Required Fields

| Field | Requirement |
|-------|-------------|
| `session_id` | MUST identify the active session uniquely |
| `workspace_ref` | MUST identify the workspace the session belongs to |
| `goal` | MUST be present after goal capture and before planning or execution |
| `active_task` | MUST be present whenever stepwise or resumable execution is possible |
| `latest_status` | MUST represent the current lifecycle state of the session |
| `latest_trace_ref` | MUST be present after any execution that emits a persisted trace |
| `updated_at` | MUST change whenever the session record is persisted after a meaningful transition |

## Behavioral Guarantees

- The session record MUST be the authoritative persisted interaction state for the current workspace.
- The session record MUST keep enough state to continue execution without reconstructing task state from raw traces.
- The session record MUST remain consistent with the latest persisted trace reference when one is present.
- The session record MUST remain readable even after non-success execution outcomes.
- The session record MUST reject or clearly surface malformed, stale, or workspace-mismatched content rather than silently repairing it.

## Update Guarantees

- `start` MUST initialize a fresh session record.
- `capture` MUST update goal-related fields without dropping unrelated valid session state.
- `plan` MUST write a new active task snapshot and reset execution position.
- `step` and `run` MUST persist updated task, status, and trace fields before returning control to the user.
- Terminal execution MUST preserve the latest task outcome until the user explicitly starts fresh or replaces the active goal.