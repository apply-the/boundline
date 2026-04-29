# Contract: Operator Surface Summary

## Purpose

Define the minimum operator-facing information that `run`, `status`, `next`, and `inspect` must expose after session-native surface unification.

## Requirements

### 1. Shared route explanation

Every operator-facing surface MUST expose:

- active route mode: `native`, `compatibility`, or `blocked`
- route source
- a human-readable reason explaining why that route is active

### 2. Shared execution condition

Every operator-facing surface MUST expose a normalized execution condition that tells the operator whether the session is:

- running
- waiting
- blocked
- terminal

When applicable, the condition MUST also include:

- a reason
- the recommended next action

### 3. Shared latest decision summary

If a decision has already been created or dispatched, every operator-facing surface MUST expose:

- the latest decision status
- the latest decision target

If no decision exists yet, surfaces MUST omit the fields rather than fill them with placeholders.

### 4. Optional mode projections remain attached, not dominant

If review, adaptive execution, or governance state exists, the operator-facing surface MUST expose it using stable labels that extend the same summary model.

These projections MUST NOT replace or obscure:

- route explanation
- execution condition
- next-action guidance

### 5. `inspect` remains richer, not semantically different

`inspect` MUST preserve trace-specific detail, including:

- decision timeline
- failure evidence
- recovery history

`inspect` MUST still use the same route and execution-condition semantics as `status` and `next`.

## Acceptance Examples

### Native path example

```text
routing: native (goal-plan) - session-native plan is ready
execution_condition: running - executing the next bounded decision
latest_decision_status: dispatched
latest_decision_target: src/lib.rs
next_command: synod inspect --workspace .
```

### Governed waiting example

```text
routing: native (goal-plan) - session-native plan is ready
execution_condition: waiting - governance approval is required before the next bounded action
governance_next_action: synod run --workspace . --approve-governance
next_command: synod run --workspace . --approve-governance
```

### Compatibility example

```text
routing: compatibility (execution-profile) - explicit compatibility path selected
execution_condition: terminal - compatibility run completed with explicit manifest-backed routing
next_command: synod inspect --workspace .
```