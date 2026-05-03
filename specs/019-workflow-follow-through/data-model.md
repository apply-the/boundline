# Data Model: Workflow Follow-Through

**Feature**: 019-workflow-follow-through  
**Date**: 2026-05-01

## Core Entities

### Workflow Definition

The workspace-authored named workflow that binds a bounded phase sequence onto the existing session-native runtime and now supports execution through review and govern.

```text
WorkflowDefinition
├── workflow_name: String
├── entry_phase: WorkflowPhase
├── phases: Vec<WorkflowPhase>
├── allow_review: Boolean
├── allow_governance: Boolean
├── conditional_phases: Vec<ConditionalWorkflowPhase>
├── output_preferences: WorkflowOutputPreferences
├── summary: Option<String>
└── recommended_when: Option<String>
```

**Behavioral rules**:
- Supported phases remain bounded to Boundline's existing delivery phases.
- Phase order remains sequential.
- Discovery metadata is optional and must not change runtime ownership or phase semantics.

### Workflow Progress State

The persisted session-owned record of workflow progression across executable, blocked, paused, and terminal phases.

```text
WorkflowProgressState
├── workflow_name: String
├── lifecycle_state: idle | active | paused | blocked | completed | failed
├── current_phase: Option<WorkflowPhase>
├── completed_phases: Vec<WorkflowPhase>
├── blocked_reason: Option<String>
├── next_action: Option<String>
├── routing_summary: Option<String>
└── phase_outcome_summary: Option<String>
```

**Behavioral rules**:
- Progress remains authoritative in the active session record.
- Review and govern phases may now complete, pause, block, or fail instead of always terminating as declaration-only blockers.
- Already completed phases must not replay by default on resume.

### Workflow Discovery Entry

The operator-facing view of one named workflow in the current workspace.

```text
WorkflowDiscoveryEntry
├── workflow_name: String
├── summary: String
├── phases: Vec<WorkflowPhase>
├── recommended_when: Option<String>
├── invocation_command: String
└── availability_state: ready | invalid | unsupported
```

**Behavioral rules**:
- Discovery is derived from the local workflow registry and current validation state.
- Invalid or unsupported workflows must still be visible as such when discovery is requested.
- Discovery must not silently activate a workflow.

### Workflow Phase Readiness

The bounded decision state that explains whether a workflow can continue through its next phase.

```text
WorkflowPhaseReadiness
├── phase: WorkflowPhase
├── readiness: ready | waiting | blocked | terminal
├── reason: String
└── next_action: Option<String>
```

**Behavioral rules**:
- Review and govern must use the same explicit readiness model as earlier workflow phases.
- Missing approvals, reviewer outcomes, or prerequisite state must surface as waiting or blocked instead of hidden retries.

### Workflow Registry Guidance Example

The documented example workflow definition and explanation used to teach maintainers how to author supported registry files.

```text
WorkflowRegistryGuidanceExample
├── example_name: String
├── declared_phases: Vec<WorkflowPhase>
├── supported_conditions: Vec<WorkflowConditionKind>
├── example_summary: String
└── bounded_non_goals: Vec<String>
```

**Behavioral rules**:
- Guidance examples are descriptive and must stay within supported workflow semantics.
- Examples must make the direct session-native path and explicit compatibility path relationship clear.

## Relationships

- `WorkflowDefinition` declares the allowed `WorkflowPhase` sequence and optional discovery metadata.
- `WorkflowProgressState` records the active `WorkflowDefinition` inside the existing session record.
- `WorkflowDiscoveryEntry` is derived from validated `WorkflowDefinition` values plus workspace validation state.
- `WorkflowPhaseReadiness` is derived from `WorkflowProgressState`, session state, and bounded review or governance conditions.
- `WorkflowRegistryGuidanceExample` documents a supported authored shape for maintainers and assistants.

## State Transitions

### Workflow Lifecycle

```text
idle -> active
active -> paused
active -> blocked
active -> completed
active -> failed
paused -> active
paused -> blocked
blocked -> active
blocked -> failed
```

### Phase Progression

```text
run -> review -> govern -> inspect
run -> review -> inspect
run -> govern -> inspect
conditional_phase -> skipped_when_condition_absent
review_or_govern -> paused_when_prerequisites_are_missing
review_or_govern -> blocked_when_progress_is_not_credible
```

### Discovery Behavior

```text
valid_registry -> discovery_entries_ready
invalid_registry -> discovery_entries_invalid
unsupported_shape -> discovery_entries_unsupported
```

The model remains intentionally small: it extends the existing workflow definition and progress primitives just enough to support follow-through execution, workflow discovery, and explicit authoring guidance.