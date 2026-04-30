# Contract: Governed Analysis Surface

## Purpose

Define the minimum operator-facing information that Synod must expose when a bounded stage routes through Canon `security-assessment`.

## Requirements

### 1. Selected Canon mode stays explicit

When a stage routes through governed security analysis, the operator-facing surfaces MUST expose:

- the active Canon mode
- the stage key bound to that mode
- the reason the mode was selected or required

### 2. Governance condition stays explicit

`run`, `status`, `next`, and `inspect` MUST expose a consistent governance condition that tells the operator whether the governed analysis path is:

- running
- waiting for approval
- blocked
- terminal

### 3. Packet provenance remains bounded

Surfaces MUST expose only:

- run reference
- packet reference
- packet readiness
- packet headline
- packet binding reason when reuse exists

Surfaces MUST NOT dump the full `.canon/` artifact tree.

### 4. Next-step guidance is aligned

When the governed analysis path is waiting or blocked, every operator-facing surface MUST expose the same next action or corrective guidance.

## Acceptance Examples

### Governed security analysis in progress

```text
routing: native (goal_plan) - goal plan is ready for native execution
selected_canon_mode: security-assessment
execution_condition: waiting - governance approval is still pending before execution can continue
latest_governance_run_ref: canon-run-security-100
latest_governance_packet_ref: .canon/runs/canon-run-security-100
next_command: synod status
```

### Unsupported mode blocked

```text
routing: blocked (session_state) - session cannot continue with the requested governed analysis
selected_canon_mode: security-assessment
execution_condition: blocked - the requested Canon mode is not supported for this stage
next_command: synod inspect --workspace .
```