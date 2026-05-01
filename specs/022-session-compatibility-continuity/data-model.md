# Data Model: Session And Compatibility Continuity

**Feature**: 022-session-compatibility-continuity  
**Date**: 2026-05-01

## Core Entities

### Continuity Authority

The bounded decision about which persisted state later commands should trust after a compatibility run.

```text
ContinuityAuthority
├── route: native_session | compatibility_trace | no_follow_up_state
├── source_ref: Option<String>
├── workspace_ref: String
├── explanation: String
└── resumable: Boolean
```

**Behavioral rules**:
- Authority must be derived only from existing persisted session and trace state.
- Authority must remain explicit in CLI output; it must never be implied silently.
- `resumable = false` must remain visible instead of being converted into a guessed next step.

### Compatibility Follow-Up State

The bounded summary of what a latest compatibility trace means for later commands.

```text
CompatibilityFollowUpState
├── trace_ref: String
├── terminal_status: succeeded | failed | exhausted | aborted | running
├── execution_path: String
├── routing_summary: String
├── next_action: String
└── continuity_mode: inspect_only | resumable | superseded
```

**Behavioral rules**:
- Follow-up state must be derivable from persisted trace evidence plus any active session state.
- `superseded` must remain explicit when a newer authoritative route state exists.
- Follow-up state must not imply that compatibility work joined the native session unless that relationship is explicitly persisted and visible.

### Shared Route Summary Surface

The aligned operator-facing vocabulary reused across native and compatibility outputs.

```text
SharedRouteSummarySurface
├── routing: String
├── execution_condition: String
├── terminal_reason: Option<String>
├── adaptive_summary: Option<String>
├── review_summary: Option<String>
├── governance_summary: Option<String>
└── next_command: Option<String>
```

**Behavioral rules**:
- Shared summary fields may align wording across routes, but routing attribution must remain explicit.
- Missing summaries must be omitted rather than synthesized from unrelated state.
- The same concept must not have materially different wording depending only on route.

## Relationships

- `ContinuityAuthority` determines whether later commands should use active session state, latest compatibility trace state, or explicit no-follow-up handling.
- `CompatibilityFollowUpState` is one possible realization of `ContinuityAuthority` when the latest authoritative state comes from a compatibility trace.
- `SharedRouteSummarySurface` is populated from either native session state or compatibility follow-up state, but keeps route attribution explicit.

## State Transitions

### Compatibility Follow-Up Resolution

```text
compatibility_run_completed -> latest_workspace_trace_available
latest_workspace_trace_available -> compatibility_trace_authority
compatibility_trace_authority -> inspect_only
compatibility_trace_authority -> resumable
compatibility_trace_authority -> superseded
```

### Mixed Route Workspace

```text
active_native_session + newer_compatibility_trace -> explicit_dual_route_summary
active_native_session + no_newer_compatibility_trace -> native_session_authority
no_active_session + latest_compatibility_trace -> compatibility_trace_authority
no_active_session + no_trace -> no_follow_up_state
```

The model stays intentionally narrow: it clarifies authoritative follow-up state and shared summary wording without changing the underlying execution engines.