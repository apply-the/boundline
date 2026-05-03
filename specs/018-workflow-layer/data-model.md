# Data Model: Session-Native Workflow Layer

**Feature**: 018-workflow-layer  
**Date**: 2026-04-30

## Core Entities

### Workflow Definition

The developer-authored named workflow that binds a small sequence of delivery phases onto the existing session-native runtime.

```text
WorkflowDefinition
├── workflow_name: String
├── entry_phase: WorkflowPhase
├── phases: Vec<WorkflowPhase>
├── allow_review: Boolean
├── allow_governance: Boolean
├── conditional_phases: Vec<ConditionalWorkflowPhase>
└── output_preferences: WorkflowOutputPreferences
```

**Behavioral rules**:
- The first slice supports only bounded built-in phases.
- Phase order must remain sequential.
- Unsupported control-flow constructs are invalid at validation time.

### Workflow Phase

The bounded workflow step that maps directly onto an existing Boundline delivery capability.

```text
WorkflowPhase
├── capture
├── clarify
├── plan
├── run
├── review
├── govern
└── inspect
```

**Behavioral rules**:
- A phase is valid only if Boundline already has a credible runtime path for it.
- The first slice allows one active phase at a time.
- Conditional phases may be skipped only when their bounded condition is not met explicitly.

### Workflow Progress State

The persisted session-owned state that tracks which named workflow is active and how far it has progressed.

```text
WorkflowProgressState
├── workflow_name: String
├── lifecycle_state: idle | active | paused | blocked | completed | failed
├── current_phase: Option<WorkflowPhase>
├── completed_phases: Vec<WorkflowPhase>
├── blocked_reason: Option<String>
├── next_action: Option<String>
└── routing_summary: Option<String>
```

**Behavioral rules**:
- Progress state lives inside the active session record.
- Already completed phases must not be replayed by default on resume.
- `blocked` and `failed` states must preserve the reason and next action.

### Conditional Workflow Phase

The bounded rule that determines whether an optional workflow phase should become active.

```text
ConditionalWorkflowPhase
├── phase: WorkflowPhase
├── condition_kind: missing_authored_input | review_triggered | governance_required
└── enabled: Boolean
```

**Behavioral rules**:
- Conditions remain declarative and bounded.
- The first slice does not allow arbitrary expressions.
- If a condition cannot be evaluated credibly, the workflow pauses explicitly instead of guessing.

### Workflow Execution Condition

The operator-facing summary of whether a named workflow is progressing, paused, blocked, or complete.

```text
WorkflowExecutionCondition
├── kind: ready | running | waiting | blocked | terminal
├── message: String
├── next_command: Option<String>
└── terminal_status: Option<success | failed | exhausted | aborted>
```

**Behavioral rules**:
- The condition must stay aligned with the underlying session runtime state.
- `waiting` is used for bounded pause conditions such as pending clarification, review, or governance.
- `terminal` must preserve the final session outcome rather than inventing workflow-only terminal semantics.

## Relationships

- `WorkflowDefinition` declares the allowed `WorkflowPhase` sequence.
- `WorkflowProgressState` records the active `WorkflowDefinition` inside the existing session record.
- `ConditionalWorkflowPhase` allows a declared phase to activate only when the corresponding bounded runtime condition is present.
- `WorkflowExecutionCondition` is derived from `WorkflowProgressState` plus the active session state and trace evidence.

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
```

### Phase Progression

```text
entry_phase -> next_declared_phase
conditional_phase -> skipped_when_condition_absent
completed_phase -> persisted_as_satisfied
invalid_phase -> blocked_before_execution
```

### Resume Behavior

```text
active + interruption -> paused
paused + unmet_condition -> paused
paused + condition_resolved -> active
paused + invalid_state_detected -> blocked
```

The model is intentionally small: it adds named workflow definitions and persisted workflow progress while keeping the underlying session-native runtime authoritative.