# Contract: Host Command JSON Surface

## Purpose

Define the structured host-output contract for the existing Boundline lifecycle
and inspection commands.

## Command Coverage

The first slice covers these commands when structured output is explicitly
requested:

- `boundline start`
- `boundline capture`
- `boundline flow`
- `boundline plan`
- `boundline step`
- `boundline run`
- `boundline status`
- `boundline next`
- `boundline inspect`

## Response Envelope

Structured output MUST serialize one envelope object per command invocation.

```json
{
  "command_name": "status",
  "exit_status": "succeeded",
  "rendered_output": "session_id: ...\nworkspace_ref: ...",
  "trace_location": null,
  "session_status": {
    "session_id": "...",
    "workspace_ref": "...",
    "latest_status": "planned",
    "next_command": "boundline run",
    "explanation": "current active session state for the workspace"
  },
  "trace_summary": null
}
```

## Rules

### Shared Rules

- `command_name` MUST reflect the invoked surface.
- `exit_status` MUST match the command's bounded outcome category.
- `rendered_output` MUST contain the same human-readable text the command would
  emit without structured output.
- `trace_location` MUST be present when the command creates or resolves a trace
  path.
- Structured output MUST preserve the existing process exit code semantics.

### Session-Oriented Commands

`start`, `capture`, `flow`, `plan`, `step`, `status`, and `next` MUST provide:

- `session_status`: populated with the serialized `SessionStatusView`
- `trace_summary`: omitted or `null`

The payload MUST preserve at least these fields when present on the session
view:

- `latest_status`
- `continuity_authority`
- `compatibility_follow_up`
- `execution_path`
- `current_step_id`
- `latest_trace_ref`
- `context_summary`
- `context_credibility`
- `clarification_prompt`
- `governance_next_action`
- `next_command`

### Trace-Oriented Commands

`run` and `inspect` MUST provide:

- `trace_summary`: populated with the serialized `TraceSummaryView`
- `session_status`: omitted or `null`

The payload MUST preserve at least these fields when present on the trace
summary:

- `trace_ref`
- `routing_summary`
- `routing_projection`
- `context_summary`
- `context_credibility`
- `delegation`
- `governance_next_action`
- `terminal_status`
- `terminal_reason`
- `duration`

## Failure Semantics

- Validation failures and trace-read failures MUST still produce the existing
  non-success exit codes.
- When structured payload data cannot be constructed because the command fails
  before session or trace resolution, the envelope MAY omit both
  `session_status` and `trace_summary`, but it MUST still include
  `command_name`, `exit_status`, and `rendered_output`.

## Host Guidance Alignment

Assistant command packs for shell-enabled paths SHOULD prefer the structured
output mode for the commands above, while chat-only fallback MUST continue to
document the plain-text command path and pasted-output interpretation.