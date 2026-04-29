# Decision Object Contract

**Feature**: 013-session-native-orchestrator  
**Date**: 2026-04-29

## Overview

The Decision object is the primary interface contract for the session-native
orchestrator. Every iteration of the execution loop produces exactly one
Decision. External consumers (CLI output, trace inspection, session persistence)
depend on this shape.

## JSON Representation

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "decision_type": "code",
  "target": "src/auth.rs",
  "rationale": "auth_test failure indicates missing validation in authenticate()",
  "expected_outcome": "auth_test passes after patching authenticate() to validate token expiry",
  "evidence_inputs": [
    { "kind": "tool_output", "reference": "decision-prev-uuid" },
    { "kind": "file", "reference": "src/auth.rs" },
    { "kind": "trace", "reference": "event-uuid-step-failed" }
  ],
  "status": "pending",
  "tool_result": null,
  "created_at": 1746000000000,
  "completed_at": null
}
```

## Field Contracts

| Field              | Type                | Required | Constraints                                                      |
| ------------------ | ------------------- | -------- | ---------------------------------------------------------------- |
| `id`               | String (UUID v4)    | Yes      | Non-empty, unique within session                                 |
| `decision_type`    | Enum                | Yes      | One of: `analyze`, `code`, `test`, `fix`, `replan`               |
| `target`           | String              | Yes      | Non-empty, identifies file, test, or subsystem                   |
| `rationale`        | String              | Yes      | Non-empty, human-readable                                        |
| `expected_outcome` | String              | Yes      | Non-empty, verifiable claim                                      |
| `evidence_inputs`  | Array<EvidenceRef>  | Yes      | May be empty for first decision only                             |
| `status`           | Enum                | Yes      | One of: `pending`, `dispatched`, `verified`, `failed`, `recovered` |
| `tool_result`      | Object or null      | Yes      | Null until act phase completes                                   |
| `created_at`       | u64                 | Yes      | Unix timestamp in milliseconds                                   |
| `completed_at`     | u64 or null         | Yes      | Null until decision reaches terminal status                      |

## EvidenceRef Contract

| Field       | Type   | Required | Constraints                                          |
| ----------- | ------ | -------- | ---------------------------------------------------- |
| `kind`      | Enum   | Yes      | One of: `trace`, `file`, `canon`, `tool_output`      |
| `reference` | String | Yes      | Non-empty, format depends on kind                    |

## ToolResult Contract

| Field         | Type          | Required | Constraints                          |
| ------------- | ------------- | -------- | ------------------------------------ |
| `tool_id`     | String        | Yes      | Non-empty                            |
| `invocation`  | String        | Yes      | Non-empty, command or operation      |
| `exit_code`   | i32 or null   | No       | Null for non-process tools           |
| `stdout`      | String        | Yes      | May be empty                         |
| `stderr`      | String        | Yes      | May be empty                         |
| `diff`        | String or null | No      | Null when no file diff applicable    |
| `duration_ms` | u64           | Yes      | Elapsed time in milliseconds         |
| `success`     | bool          | Yes      | True if tool operation succeeded     |

## GoalPlan Contract

| Field               | Type              | Required | Constraints                           |
| ------------------- | ----------------- | -------- | ------------------------------------- |
| `plan_id`           | String (UUID v4)  | Yes      | Non-empty, unique                     |
| `goal_text`         | String            | Yes      | Non-empty                             |
| `tasks`             | Array<PlannedTask> | Yes     | Non-empty, at least one task          |
| `source_evidence`   | Array<EvidenceRef> | Yes     | May be empty                          |
| `workspace_signals` | Object            | Yes      | See WorkspaceSignals shape            |
| `flow`              | Object or null    | No       | Inferred flow if applicable           |
| `created_at`        | u64               | Yes      | Unix timestamp in milliseconds        |
| `status`            | Enum              | Yes      | One of: `draft`, `confirmed`, `superseded` |

## Invariants

1. A Decision in `dispatched` status MUST eventually transition to `verified` or `failed`.
2. A Decision in `failed` status MAY transition to `recovered` if a recovery decision is created.
3. The `evidence_inputs` of a Decision MUST reference only evidence that exists in the current session.
4. The `tool_result` field MUST be null while `status` is `pending`, and non-null after `dispatched`.
5. A session MUST NOT have more than `max_steps` decisions with status `dispatched` or later.
6. FlowPolicy stage constraints MUST be enforced before a Decision transitions from `pending` to `dispatched`.

## CLI Output Contract

`synod inspect` MUST display each decision with:

```text
Decision [id_short] (type: code, status: verified)
  Target: src/auth.rs
  Rationale: auth_test failure indicates missing validation
  Expected: auth_test passes after patch
  Evidence: 2 inputs
  Tool: cargo test (exit: 0, 1.2s)
```
