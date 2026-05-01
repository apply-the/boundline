# Contract: Workflow Follow-Through Command Surface

## Purpose

Define the minimum operator-facing behavior for executing bounded review and govern phases from the `synod workflow` command surface.

## Requirements

### 1. Workflow identity stays explicit through follow-through phases

When a named workflow is active during review or govern, the operator-facing workflow surfaces MUST expose:

- the workflow name
- the current workflow phase
- the route being used

### 2. Review and govern are executable, not declaration-only blockers

When bounded prerequisites are satisfied, `workflow run` or `workflow resume` MUST allow review and govern to execute as real workflow phases instead of always stopping at those phases as static blockers.

### 3. Blocked and non-success states stay actionable

When review or govern cannot continue, the workflow surface MUST expose:

- whether the workflow is waiting, blocked, failed, or terminal
- the reason it stopped
- the next command or remediation needed to continue

### 4. Workflow follow-through does not hide route ownership

Workflow follow-through MUST preserve the same explicit route story as the existing session-native surfaces and MUST NOT imply that Canon or a compatibility profile owns the workflow by default.

## Acceptance Examples

### Review completed and govern is next

```text
workflow: governed-delivery
workflow_phase: govern
routing: native (goal_plan) - goal plan is ready for native execution
execution_condition: waiting - governance approval is still required before workflow progression can continue
next_command: synod workflow resume --workspace .
```

### Governance blocked explicitly

```text
workflow: governed-delivery
workflow_phase: govern
routing: native (goal_plan) - goal plan is ready for native execution
execution_condition: blocked - governance cannot continue until the required approval state is resolved
next_command: synod workflow inspect --workspace .
```