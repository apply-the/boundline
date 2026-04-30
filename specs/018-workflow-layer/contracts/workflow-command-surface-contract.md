# Contract: Workflow Command Surface

## Purpose

Define the minimum operator-facing behavior for the first `synod workflow` command family.

## Requirements

### 1. Workflow identity stays explicit

When a named workflow is active, the operator-facing workflow surfaces MUST expose:

- the workflow name
- the current workflow phase
- the route being used

### 2. Execution condition stays aligned with the session runtime

`workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` MUST expose a consistent execution condition that tells the operator whether the workflow is:

- running
- waiting
- blocked
- terminal

### 3. Resume guidance stays actionable

When the workflow is paused or blocked, the surface MUST expose:

- the reason it stopped
- the next command to run
- whether the condition is missing input, pending governance, pending review, or a terminal failure

### 4. Workflow commands do not hide the underlying route

Workflow commands MUST preserve the same explicit route story as the existing session-native surfaces and MUST NOT imply that Canon or a compatibility profile owns the workflow by default.

## Acceptance Examples

### Valid named workflow

```text
workflow: default
workflow_phase: plan
routing: native (goal_plan) - goal plan is ready for native execution
execution_condition: waiting - planning still needs confirmed input before execution can continue
next_command: synod workflow resume --workspace .
```

### Invalid workflow definition

```text
workflow: invalid-flow
workflow_phase: blocked
routing: blocked (session_state) - workflow definition is not valid for session-native execution
execution_condition: blocked - unsupported workflow phase ordering
next_command: synod workflow inspect --workspace .
```