# Data Model: Chat-First Host-Integrated Runtime

## Overview

This slice does not introduce a new persistence surface. It reuses the existing
workspace-owned session and trace state, then adds a bounded host-facing output
envelope around the projections that already describe that state.

## Entities

### HostCommandEnvelope

- Purpose: Wrap the result of one Boundline command invocation in a host-friendly
  payload that preserves both machine-readable state and the original rendered
  output.
- Fields:
  - `command_name`: The invoked Boundline command surface.
  - `exit_status`: The bounded command outcome category.
  - `rendered_output`: The human-readable text that Boundline would otherwise
    print by default.
  - `trace_location`: Optional trace file reference when the command produces or
    resolves one.
  - `session_status`: Optional `SessionStatusView` payload for lifecycle
    commands whose primary product is session state.
  - `trace_summary`: Optional `TraceSummaryView` payload for run and inspect
    paths whose primary product is trace-derived outcome data.
- Lifecycle:
  - Created at command dispatch time.
  - Never persisted independently.
  - Serialized only when the structured host-output mode is requested.

### SessionStatusView

- Purpose: Describe the current workspace-owned delivery session, including the
  goal, route ownership, continuity authority, follow-up guidance, blocked
  states, and next recommended command.
- Existing Source: `src/domain/session.rs`
- Role in this slice:
  - Becomes the canonical structured payload for `start`, `goal`, `flow`,
    `plan`, `step`, `status`, and `next`.
  - Continues to back the human-readable `render_session_status` output.
- Constraints:
  - Must remain consistent with the persisted `ActiveSessionRecord`.
  - Must surface non-success states such as clarification required, blocked
    governance, compatibility continuity, and non-credible context.

### TraceSummaryView

- Purpose: Describe the terminal or intermediate outcome of a run/inspect path,
  including routing, executed steps, recovery events, terminal status, and
  inspection guidance.
- Existing Source: `src/domain/trace.rs`
- Role in this slice:
  - Becomes the canonical structured payload for `run` and `inspect`.
  - Continues to back the human-readable trace rendering used by direct CLI
    operators and chat-only fallback.
- Constraints:
  - Must preserve trace references and terminal reasoning exactly.
  - Must remain derivable from the persisted trace rather than a separate host
    cache.

## Relationships

- `HostCommandEnvelope.session_status` references one `SessionStatusView` when a
  lifecycle command returns session state.
- `HostCommandEnvelope.trace_summary` references one `TraceSummaryView` when a
  run or inspect command returns trace-derived state.
- `SessionStatusView` and `TraceSummaryView` remain derived from the existing
  persisted Boundline records rather than new host-specific storage.

## Validation Rules

- Exactly one primary structured payload should be present per successful host
  command invocation: session-oriented commands expose `session_status`, while
  trace-oriented commands expose `trace_summary`.
- `rendered_output` must remain present even when structured output is enabled,
  so hosts can fall back to the text contract without rerunning the command.
- `trace_location` must match the resolved or produced trace when one is
  available.
- The structured payload must not suppress failure, blocked, or compatibility
  continuity information already available on the text surface.