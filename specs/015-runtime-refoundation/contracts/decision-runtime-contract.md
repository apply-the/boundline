# Decision Runtime Contract

**Feature**: 015-runtime-refoundation  
**Date**: 2026-04-29

## Overview

This contract defines the minimum persisted and operator-visible shape of the session-native runtime path once a bounded task draft has been created.

## Decision Record

Each runtime decision MUST expose the following fields through session state and trace output:

| Field | Required | Contract |
|-------|----------|----------|
| `id` | Yes | Stable identifier for the bounded action |
| `decision_type` | Yes | One of the allowed decision families for the active runtime state |
| `target` | Yes | File, test, subsystem, or workspace slice acted upon |
| `rationale` | Yes | Short inspectable explanation of why the decision was chosen |
| `expected_outcome` | Yes | Verifiable claim that will be checked after execution |
| `evidence_inputs` | Yes | Ordered references to prior evidence used to choose the decision |
| `action_result` | No before dispatch, Yes after dispatch | Structured result of the tool or agent action |
| `status` | Yes | `pending`, `dispatched`, `verified`, `failed`, or `recovered` |
| `created_at` | Yes | Creation timestamp |
| `completed_at` | No before completion, Yes after terminal decision status | Completion timestamp |

## Runtime Guarantees

1. Every session-native run that begins work MUST create at least one persisted decision.
2. Decisions MUST be appended in execution order to both session state and trace output.
3. A failed decision MUST remain inspectable even if a later recovery decision succeeds.
4. A recovery or replan decision MUST reference the evidence from the failed decision that triggered it.
5. Terminal states MUST remain explainable from persisted decision history plus explicit terminal reasoning.

## GoalPlan Handoff Contract

Before the first decision is dispatched:

1. A confirmed or flow-skipped `GoalPlan` MUST already exist.
2. The runtime MUST expose a summary of the bounded task draft to the operator.
3. If flow is only proposed, the runtime MUST block policy-bound execution until confirmation or skip is explicit.

## Operator Surface Contract

### `status` and `next`

The operator must be able to determine:

- whether the current route is session-native or compatibility
- whether the flow is proposed, confirmed, skipped, or absent
- the latest persisted decision status
- the next recommended command when the runtime is blocked or terminal

### `inspect`

`inspect` MUST surface, in order:

1. route choice
2. bounded task draft summary
3. decision timeline with rationale and expected outcome
4. failure or recovery evidence when present
5. explicit terminal reason

## Failure Contract

- Missing bounded task draft MUST block run with remediation instead of falling back silently.
- Adapter or tool unavailability MUST be persisted as decision failure evidence.
- Exhausted and no-actionable-state outcomes MUST be terminal and inspectable, not retried invisibly.