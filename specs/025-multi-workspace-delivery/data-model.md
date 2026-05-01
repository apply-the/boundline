# Data Model: Expand Multi-Workspace Delivery

**Feature**: 025-multi-workspace-delivery  
**Date**: 2026-05-01

## Core Entities

### Cluster Delivery Story

The bounded multi-workspace execution context that ties one delivery goal to
one authoritative owner and one known set of participating member workspaces.

```text
ClusterDeliveryStory
├── cluster_id: String
├── primary_workspace_ref: String
├── authoritative_workspace_ref: String
├── route_owner: native | workflow | review | governance | compatibility
├── member_workspace_refs: Vec<String>
├── participating_workspace_refs: Vec<String>
├── started_from_command: String
├── execution_condition: String
└── updated_at: u64
```

**Behavioral rules**:
- The story must name exactly one authoritative workspace context at a time.
- Participating workspaces must be a subset of the registered cluster members.
- A clustered story must remain inspectable even when it stops in a blocked,
  failed, exhausted, or inspect-only state.

### Workspace Participation Record

The inspectable statement of how one member workspace contributed to the
clustered delivery story.

```text
WorkspaceParticipationRecord
├── workspace_ref: String
├── participation_kind: entry | read_only | mutated | blocked | skipped
├── order: usize
├── latest_trace_ref: Option<String>
├── latest_status: Option<String>
├── headline: String
└── terminal_reason: Option<String>
```

**Behavioral rules**:
- Each participating workspace appears at most once in the current story.
- `blocked` participation must carry enough reason text for follow-up guidance.
- `mutated` and `read_only` participation remain distinct so the operator can
  identify which repositories changed.

### Cluster Follow-Up Authority

The explicit statement of which route and workspace context currently own the
next action after a clustered run.

```text
ClusterFollowUpAuthority
├── authority_kind: active_session | compatibility_trace | inspect_only
├── route_owner: native | workflow | review | governance | compatibility
├── authoritative_workspace_ref: String
├── continuity_reason: String
└── next_command: String
```

**Behavioral rules**:
- There must be one and only one current follow-up authority.
- Inspect-only authority must never imply resumable execution.
- The authoritative workspace ref must remain visible even when the route owner
  is shared across multiple workspaces.

### Clustered Execution Condition

The bounded state of the clustered delivery story at the current follow-up
point.

```text
ClusteredExecutionCondition
├── kind: success | paused | blocked | failed | exhausted | inspect_only
├── active_workspace_ref: Option<String>
├── blocking_workspace_ref: Option<String>
├── summary: String
└── recovery_allowed: bool
```

**Behavioral rules**:
- `success` and `inspect_only` are terminal for direct execution, but only
  `inspect_only` explicitly routes the operator to trace inspection.
- `blocked` and `failed` conditions must name the member workspace responsible
  when one exists.
- Recovery guidance must stay bounded to the registered cluster members.

## Relationships

- `ClusterDeliveryStory` owns the current `ClusterFollowUpAuthority` and a set
  of `WorkspaceParticipationRecord` values.
- `ClusteredExecutionCondition` explains the current state of the
  `ClusterDeliveryStory` and constrains what the `ClusterFollowUpAuthority`
  can recommend next.
- `WorkspaceParticipationRecord` references per-workspace traces and statuses
  that remain persisted in the existing local workspace state.

## State Transitions

### Cluster Delivery Lifecycle

```text
cluster_registered -> story_initialized
story_initialized -> workspace_selected
workspace_selected -> workspace_executing
workspace_executing -> workspace_handoff
workspace_handoff -> workspace_selected
workspace_executing -> terminal_success
workspace_executing -> terminal_blocked
workspace_executing -> terminal_failed
workspace_executing -> terminal_exhausted
workspace_executing -> inspect_only_follow_up
```

### Follow-Up Authority Lifecycle

```text
active_session -> active_session
active_session -> inspect_only
active_session -> compatibility_trace
compatibility_trace -> inspect_only
```

The model stays intentionally local and sequential: it expands the current
cluster/session read-write surfaces without adding distributed background state
or multiple simultaneous owners.