# Contract: Governed Stage Command Surface

## Purpose

Define the minimum operator-facing behavior for governing `bug-fix:investigate` ahead of the existing governed verify path on the direct session-native route.

## Requirements

### 1. Earlier governed stage identity stays explicit

When a session is halted or continued at governed `bug-fix:investigate`, the operator-facing surfaces MUST expose:

- the governed stage key
- the governance runtime
- the selected Canon mode when applicable
- the current governance condition

### 2. Governed progression remains session-owned

When `bug-fix:investigate` is governed, `run`, `status`, `next`, and `inspect` MUST preserve the same session-native routing story instead of implying a separate Canon-owned workflow.

### 3. Blocked and waiting states stay actionable

When governed `investigate` cannot continue, the command surface MUST expose:

- whether work is waiting, blocked, failed, or terminal
- the reason it stopped
- the next command or remediation needed to continue

## Acceptance Examples

### Governed investigate waiting on approval

```text
routing: native (goal_plan) - goal plan is ready for native execution
latest_governance_stage: bug-fix:investigate
latest_governance_state: awaiting_approval
latest_governance_mode: discovery
next_command: synod run --workspace .
```

### Governed investigate blocked explicitly

```text
routing: native (goal_plan) - goal plan is ready for native execution
latest_governance_stage: bug-fix:investigate
latest_governance_state: blocked
execution_condition: blocked - governance blocked stage bug-fix:investigate
next_command: synod inspect --workspace .
```