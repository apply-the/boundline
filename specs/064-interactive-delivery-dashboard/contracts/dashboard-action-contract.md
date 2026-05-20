# Contract: Dashboard Action

## Purpose

Define how an operator action from the dashboard is validated, applied, refused, and surfaced back to the operator.

## Supported Action Kinds

| Action | Mutates State | Requires Revision | Requires Reason | Expected Outcome |
|--------|---------------|-------------------|-----------------|------------------|
| `confirm` | yes | yes | no | Plan or proposed flow becomes confirmed |
| `reject` | yes | yes | yes | Replan requested or session stops with recorded reason |
| `replan` | yes | yes | sometimes | New bounded plan path requested |
| `recover` | yes | yes | sometimes | Recovery path selected or refused |
| `launch` | yes | no | no | New session path starts when workspace is valid |
| `continue` | yes | yes | no | Runtime continues or reaches a terminal state |
| `inspect-only` | no | no | no | Dashboard changes focus only |

## Request Shape

```json
{
  "request_id": "request-uuid",
  "workspace_ref": "/workspace",
  "action_kind": "confirm",
  "target_session_id": "session-uuid",
  "target_session_revision": 3,
  "operator_reason": null,
  "requested_at": "2026-05-19T00:00:00Z"
}
```

## Result Shape

```json
{
  "request_id": "request-uuid",
  "outcome": "applied",
  "state_transition": "planned_to_confirmed",
  "next_snapshot_ref": "snapshot-uuid",
  "next_command": "boundline run",
  "trace_refs": ["trace:path"],
  "refusal_reason": null,
  "operator_message": "Plan confirmed. Continue bounded execution."
}
```

## Refusal Shape

```json
{
  "request_id": "request-uuid",
  "outcome": "refused",
  "state_transition": null,
  "next_snapshot_ref": "snapshot-uuid",
  "next_command": "boundline status",
  "trace_refs": [],
  "refusal_reason": "stale_session_revision",
  "operator_message": "The session changed after this dashboard view was rendered. Refresh and choose from the current actions."
}
```

## Validation Rules

- Mutating requests must fail closed when the target session revision no longer matches current state.
- Requests must fail closed when a higher-priority stop rule, approval wait, invalid workspace, missing context, or invalid session state forbids the action.
- Requests must preserve the same resulting state and trace evidence as the normal command path.
- Requests must not mutate governed artifacts or write dashboard-only state.
- Rejection and selected replanning requests must preserve the operator reason in Boundline-owned state or trace evidence.

## Required Non-Success Outcomes

- `stale_session_revision`
- `invalid_workspace`
- `missing_active_session`
- `missing_required_context`
- `blocked_by_stop_rule`
- `approval_waiting`
- `unsupported_action`
- `dashboard_degraded`
- `runtime_command_unavailable`
