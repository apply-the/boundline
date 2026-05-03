# Contract: Flow-Aware Status and Next Output

## Status Surface

When an active session has a selected flow, `boundline status` must expose flow-specific progress in addition to the existing session fields.

These flow-specific fields appear in the rendered output of `boundline status` and in the shared session-status view consumed by `boundline next`. When no flow is active, these fields are omitted rather than rendered as empty placeholders.

### Required Fields

- `active_flow`: selected flow identifier
- `current_stage`: active stage identifier or display label
- `stage_progress`: one-based current stage position and total stage count
- `current_step`: current step identifier when a plan is active
- `latest_status`: current session status
- `next_command`: next valid command when one exists

### Example

```text
session_id: 8df6f0cf-d6bd-4a3c-b2bf-11e07f8df555
workspace_ref: /workspace/project
goal: Fix failing checkout tests
active_flow: bug-fix
current_stage: investigate
stage_progress: 1/3
current_step: investigate-analyze
latest_status: planned
next_command: boundline step
explanation: current active session state for the workspace
```

## Next Surface

When an active session has a selected flow, `boundline next` must:

- Consider the current stage before recommending the next command.
- Keep guidance within the active stage after retryable or replannable failure.
- Change the recommended command only when the stage has actually advanced or the session has reached a terminal outcome.

## No-Flow Compatibility

- When no flow is selected, `status` and `next` continue to work without flow-specific fields.
- Flow-specific output must be omitted cleanly rather than rendered as empty placeholders.

## Error Cases

- If the session contains invalid flow state, `status` and `next` fail with explicit recovery guidance.
- If no active session exists, both commands fail with the existing missing-session behavior.