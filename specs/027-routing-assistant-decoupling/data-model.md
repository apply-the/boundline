# Data Model: Inspectable Routing And Assistant Decoupling

**Feature**: 027-routing-assistant-decoupling  
**Date**: 2026-05-01

## Core Entities

### Routing Decision Record

The explicit summary of which bounded delivery slot is active, which
provider/model route currently owns it, where that decision came from, and why
it is authoritative for the current execution or follow-up story.

```text
RoutingDecisionRecord
├── slot: planning | implementation | verification | review | adjudication | reviewer_role
├── runtime: String
├── model: String
├── source: cli | workspace | cluster | global | built_in
├── reason: String
├── route_owner: native | compatibility
└── visible_in_follow_up: bool
```

**Behavioral rules**:
- A visible routing decision must always identify both the chosen backend and
  the authority source.
- `route_owner` must not change merely because the assistant/backend family
  changes.
- `visible_in_follow_up` is true whenever `run`, `status`, `next`, or `inspect`
  should surface the record.

### Assistant Binding

The inspectable association between a routing decision and the assistant or
command-pack family used to express or execute the bounded slot.

```text
AssistantBinding
├── slot: same as RoutingDecisionRecord.slot
├── runtime: String
├── command_pack_family: claude | codex | copilot | gemini | none
├── binding_reason: String
└── preserves_cli_surface: bool
```

**Behavioral rules**:
- A binding must follow the active routing decision instead of overriding it.
- `preserves_cli_surface` must remain true for valid bindings because the CLI
  surface is not supposed to change with backend choice.
- A missing or unsupported binding must fail explicitly rather than silently
  reverting to a hard-wired backend.

### Routing Projection

The session or trace-facing view that carries routing and assistant-binding
state into operator follow-up surfaces.

```text
RoutingProjection
├── active_decisions: Vec<RoutingDecisionRecord>
├── active_bindings: Vec<AssistantBinding>
├── compatibility_authority_explicit: bool
└── cluster_authority_explicit: bool
```

**Behavioral rules**:
- The projection must be compact enough for existing CLI summary surfaces.
- When compatibility follow-up is authoritative, the projection must say so
  without implying a resumable native session.
- When clustered delivery is authoritative, the projection must preserve the
  primary workspace as the owner of the routing story.

## Relationships

- One `RoutingDecisionRecord` can produce at most one authoritative
  `AssistantBinding` for the same slot at a time.
- A `RoutingProjection` owns zero or more routing decisions and assistant
  bindings, depending on what the current run or follow-up state can surface.
- `RoutingDecisionRecord` explains which configured route is authoritative;
  `AssistantBinding` explains how that route maps to assistant-facing behavior.

## State Transitions

### Routing Decision Lifecycle

```text
resolved_from_config -> projected_to_session
resolved_from_config -> projected_to_trace
projected_to_session -> projected_to_follow_up
projected_to_trace -> projected_to_follow_up
projected_to_follow_up -> superseded_by_new_resolution
```

### Assistant Binding Lifecycle

```text
unbound -> bound_from_routing_decision
bound_from_routing_decision -> projected_to_follow_up
bound_from_routing_decision -> binding_failed_explicitly
projected_to_follow_up -> superseded_by_new_resolution
```

The model stays intentionally narrow: it adds explicit routing and binding state
to the existing Boundline delivery story without introducing a new runtime,
provider-gateway layer, or assistant-owned orchestration surface.