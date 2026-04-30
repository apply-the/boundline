# Contract: Workflow Definition Validation

## Purpose

Define the minimum validation behavior for local named workflow definitions in the first workflow-layer slice.

## Requirements

### 1. Definitions stay bounded

The first workflow-definition surface MUST allow only:

- a workflow name
- one entry phase
- one sequential list of supported phases
- bounded conditional activation for supported optional phases
- bounded output preferences

### 2. Unsupported semantics fail before execution

Definitions that attempt to introduce unsupported behavior MUST be rejected before work starts, including:

- loops
- arbitrary branching
- fan-out or concurrency
- hidden background progression
- Canon-owned workflow progression

### 3. Definitions cannot silently override route ownership

Workflow definitions MUST NOT:

- silently replace direct session-native commands
- silently force the compatibility path
- treat Canon as the workflow controller

### 4. Progress is session-owned

Once a definition is accepted, runtime progress MUST be represented through the existing session record and traces rather than through an independent workflow state machine.

## Acceptance Examples

### Accepted definition shape

```text
workflow name: default
entry phase: capture
phases: capture -> clarify -> plan -> run -> inspect
optional condition: review only when review is triggered
```

### Rejected definition shape

```text
workflow name: auto-loop
entry phase: run
phases: run -> inspect -> run
extra rule: repeat until success
result: rejected before execution
```