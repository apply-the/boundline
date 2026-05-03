# Data Model: Product Unification And Surface Closure

**Feature**: 032-workflow-surface-closure  
**Date**: 2026-05-02

## Core Entities

### Workflow Assistant Surface

The assistant-facing command or guidance surface that lets an operator discover,
start, continue, and inspect a named workflow without leaving the primary Boundline
product story.

```text
WorkflowAssistantSurface
├── assistant_family: claude | codex | copilot | gemini
├── workflow_action: list | run | status | resume | inspect
├── required_context: workspace_ref | workflow_name | goal | trace_ref
├── shell_enabled_command: String
├── chat_only_fallback: String
├── primary_path: bool
└── allowed_follow_ups: Vec<String>
```

**Behavioral rules**:
- Every shipped assistant family must expose the same bounded workflow actions,
  even if the artifact format differs.
- `primary_path` remains true for workflow actions because workflows are part of
  the same primary Boundline product surface as direct session-native execution.
- Chat-only fallback must preserve the exact workflow command rather than
  paraphrasing it into provider-specific prose.

### Workflow Route Projection

The compact view that explains how a workflow is currently executing and why
that route is authoritative.

```text
WorkflowRouteProjection
├── workflow_name: String
├── workflow_phase: capture | clarify | plan | run | review | govern | inspect
├── routing: String
├── route_owner: native | compatibility
├── route_config_projection: Vec<String>
├── execution_path: String
├── continuity_authority: native_session | compatibility_trace | none
├── assistant_bindings_visible: bool
└── next_command: String
```

**Behavioral rules**:
- Workflow projection must reuse the same routing and follow-through vocabulary
  as direct session-native surfaces.
- `route_owner` must stay `native` for named workflow execution unless the
  operator explicitly moved into compatibility follow-up.
- `assistant_bindings_visible` must be true whenever route-config projection is
  available for the active workflow state.

### Product Path Cue

The explicit indicator that tells the operator whether the current follow-up is
on a primary Boundline surface or on the subordinate compatibility path.

```text
ProductPathCue
├── surface_kind: workflow | session_native | compatibility_follow_up
├── primary_surface: bool
├── authority_reason: String
├── subordinate_route_explicit: bool
└── inspectable_in_output: bool
```

**Behavioral rules**:
- Workflow and direct session-native surfaces must always set
  `primary_surface=true`.
- Compatibility follow-up must always set `subordinate_route_explicit=true`.
- `authority_reason` must explain ownership without requiring the operator to
  read config files or source code.

## Relationships

- One `WorkflowAssistantSurface` maps one assistant family to one workflow
  action at a time.
- One `WorkflowRouteProjection` can be surfaced by multiple assistant families,
  but the underlying route and authority cues must remain identical.
- One `ProductPathCue` accompanies each workflow or follow-up view and explains
  whether the current path is primary or subordinate.

## State Transitions

### Workflow Assistant Surface Lifecycle

```text
declared -> available_to_operator
available_to_operator -> invoked
invoked -> resumed_with_next_command
invoked -> blocked_with_explicit_reason
```

### Workflow Route Projection Lifecycle

```text
derived_from_session -> rendered_to_workflow_output
derived_from_session -> rendered_to_assistant_guidance
rendered_to_workflow_output -> superseded_by_new_phase
rendered_to_assistant_guidance -> superseded_by_new_phase
```

### Product Path Cue Lifecycle

```text
primary_native -> primary_workflow
primary_workflow -> compatibility_follow_up_explicit
compatibility_follow_up_explicit -> inspect_only_subordinate
```

The model stays narrow on purpose: it closes the operator-facing product story
without adding a new control plane, runtime, or persistence surface.