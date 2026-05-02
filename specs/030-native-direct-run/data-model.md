# Data Model: Native Direct Run

**Feature**: 030-native-direct-run  
**Date**: 2026-05-02

## Core Entities

### Direct Run Bootstrap Request

The operator-facing request that turns one `run` command into either a native
session bootstrap or an explicitly chosen compatibility execution.

```text
DirectRunBootstrapRequest
├── workspace_ref: String
├── goal_text: String?
├── brief_sources: [String]
├── governance_intent: Object?
├── compatibility_requested: bool
└── active_session_present: bool
```

**Behavioral rules**:
- `goal_text` or equivalent authored input must exist before native bootstrap
  can proceed.
- `compatibility_requested` is the only valid reason to choose the subordinate
  compatibility route.
- `active_session_present` must not be ignored when the existing session has
  meaningful captured, planned, or in-flight work.

### Direct Run Route Choice

The explicit ownership choice Synod makes before execution starts.

```text
DirectRunRouteChoice
├── owner: native_session | compatibility
├── reason: String
├── execution_profile_required: bool
├── session_reset_required: bool
└── terminal_if_blocked: bool
```

**Behavioral rules**:
- `owner` is `native_session` by default for direct `run --goal` unless the
  operator explicitly requests compatibility.
- `execution_profile_required` is true only for the compatibility route.
- `session_reset_required` becomes true when direct run encounters meaningful
  active session state it must not overwrite.

### Bootstrap Session Projection

The persisted native session state that direct run creates before or during
execution so later commands can continue from the same story.

```text
BootstrapSessionProjection
├── session_id: String
├── goal: String
├── negotiation_projection: Object?
├── goal_plan_ready: bool
├── confirmed_flow: String?
├── flow_skipped: bool
├── latest_status: initialized | goal_captured | planned | running | succeeded | failed | exhausted
├── latest_trace_ref: String?
└── decisions_recorded: bool
```

**Behavioral rules**:
- `goal_plan_ready` must be true before the native run proceeds.
- `confirmed_flow` is populated when direct run can credibly confirm an inferred
  built-in flow.
- `flow_skipped` is true when direct run chooses bounded no-flow planning
  instead of blocking on pending flow confirmation.

## Relationships

- One `DirectRunBootstrapRequest` produces exactly one `DirectRunRouteChoice`.
- A `DirectRunRouteChoice` with `owner=native_session` creates or updates one
  `BootstrapSessionProjection`.
- Compatibility route choice does not fabricate a native `BootstrapSessionProjection`.

## State Transitions

### Native Direct Run Lifecycle

```text
request_received -> route_chosen
route_chosen -> session_bootstrapped
session_bootstrapped -> goal_plan_ready
goal_plan_ready -> native_run_started
native_run_started -> terminal_native_state
```

### Blocked Or Explicit Compatibility Lifecycle

```text
request_received -> route_chosen
route_chosen -> blocked_for_active_session
route_chosen -> compatibility_run_started
compatibility_run_started -> terminal_compatibility_state
```

The model stays intentionally small: it changes route ownership and bootstrap
state for direct run without introducing another persistence surface or a second
native execution engine.