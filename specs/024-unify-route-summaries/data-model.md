# Data Model: Unify Route Summaries And Config Projection

**Feature**: 024-unify-route-summaries  
**Date**: 2026-05-01

## Core Entities

### Unified Route Summary

The bounded operator-facing projection that describes the current follow-up story using one shared vocabulary across route families.

```text
UnifiedRouteSummary
├── route_owner: native | workflow | review | governance | compatibility
├── continuity_authority: active_session | latest_trace | workflow_pause | inspect_only
├── execution_condition: String
├── next_action: String
├── state_headline: String
├── state_detail: Option<String>
└── route_evidence: Vec<String>
```

**Behavioral rules**:
- Every rendered follow-up surface must expose route owner and continuity authority.
- `execution_condition` names may align across routes, but route ownership must still remain explicit.
- Missing route evidence must omit that field rather than invent explanatory text.

### Config Projection

The subset of routing and configuration data that materially explains why the current route owns follow-up.

```text
ConfigProjection
├── explicit_route_override: Option<String>
├── workspace_default_route: Option<String>
├── global_default_route: Option<String>
├── workflow_name: Option<String>
├── workflow_recommended_when: Option<String>
├── governance_mode: Option<String>
└── projection_reason: Vec<String>
```

**Behavioral rules**:
- Only config that materially affects the current follow-up interpretation should be projected.
- Explicit command choices override workspace and global defaults in the projection story.
- Stale or irrelevant config must be excluded from route summaries.

### Follow-Up Authority State

The persisted authority that determines whether the operator should resume, inspect, wait, or stop.

```text
FollowUpAuthorityState
├── source: session | workflow | trace
├── resumable: Boolean
├── inspect_only: Boolean
├── terminal: Boolean
├── recommended_command: Option<String>
└── authority_reason: String
```

**Behavioral rules**:
- Only one authority source is active for the current follow-up story.
- Compatibility follow-up without an active session must remain inspect-only.
- Terminal states must still preserve recommended follow-up guidance when guidance exists.

### Route Ownership Projection

The explicit statement of which route owns the current state and which route-specific boundaries still apply.

```text
RouteOwnershipProjection
├── owner: native | workflow | review | governance | compatibility
├── owner_reason: String
├── inherited_from: Option<String>
└── incompatible_assumptions: Vec<String>
```

**Behavioral rules**:
- Ownership projection must reject hidden promotion between routes.
- Workflow, review, and governance ownership may inherit native session context but must still name the current bounded owner.
- Compatibility ownership must remain explicit even when wording converges with native summaries.

## Relationships

- `UnifiedRouteSummary` renders the combined follow-up story using `FollowUpAuthorityState`, `RouteOwnershipProjection`, and `ConfigProjection`.
- `ConfigProjection` explains why the active `RouteOwnershipProjection` is authoritative for the current summary.
- `FollowUpAuthorityState` controls whether the operator should resume, inspect, wait, or treat the route as terminal.
- `RouteOwnershipProjection` bounds how aligned summary vocabulary can be interpreted.

## State Transitions

### Follow-Up Authority Lifecycle

```text
active_session -> resumable_follow_up
active_session -> terminal_follow_up
latest_trace -> inspect_only_follow_up
workflow_pause -> wait_or_resume_follow_up
workflow_pause -> terminal_follow_up
```

### Route Summary Lifecycle

```text
state_loaded -> ownership_projected
ownership_projected -> config_projected
config_projected -> summary_rendered
summary_rendered -> updated_after_next_state_change
```

The model stays intentionally read-side only: it deepens the projection of existing route and config state without introducing a new orchestration authority.
