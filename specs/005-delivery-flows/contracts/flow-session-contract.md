# Contract: Persisted Session Flow State

## Session Record Extension

The workspace session record may contain an optional `active_flow` object.

```json
{
  "session_id": "8df6f0cf-d6bd-4a3c-b2bf-11e07f8df555",
  "workspace_ref": "/workspace/project",
  "goal": "Fix failing checkout tests",
  "latest_status": "planned",
  "active_flow": {
    "flow_name": "bug-fix",
    "current_stage_id": "investigate",
    "current_stage_index": 0,
    "total_stages": 3
  }
}
```

## Compatibility Rules

- `active_flow` is optional so pre-existing sessions remain valid.
- Sessions without `active_flow` continue to support non-flow planning and execution.
- Sessions with `active_flow` must still satisfy all existing session validation rules.

## Validation Rules

- `flow_name` must match a built-in flow definition.
- `current_stage_index` must be within `0..total_stages`.
- `current_stage_id` must match the stage at `current_stage_index` in the selected flow.
- `total_stages` must equal the stage count of the selected flow.

## Transition Rules

- On flow selection, `active_flow` is created and points to the first stage.
- On successful completion of the active stage, `current_stage_id` and `current_stage_index` advance together.
- On retry or replan within a stage, `active_flow` remains unchanged.
- On terminal success or failure, `active_flow` remains persisted for inspection of the last stage state.

## Invalid State Handling

- If `active_flow` references an unknown flow, the session is invalid for flow-aware execution.
- If stage identity and stage index do not match, the session must fail closed and require explicit recovery.
- If `active_flow` is present while the session is otherwise missing required goal or task state for the current command, the command must surface the underlying session error instead of inferring missing data.