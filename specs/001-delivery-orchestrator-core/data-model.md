# Data Model: Delivery Orchestrator Core

## Task

Represents one bounded delivery objective managed by the orchestrator.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `id` | UUID-like identifier | Yes | Stable identifier across the full task run |
| `goal` | Non-empty string | Yes | Human-readable objective for the task |
| `input` | Structured payload | Yes | Initial request data for planning and execution |
| `context` | `TaskContext` | Yes | Mutable session-scoped state |
| `plan` | `Plan` | Yes | Current ordered execution path |
| `status` | Enum | Yes | `planned`, `running`, `succeeded`, `failed`, `exhausted`, or `aborted` |
| `limits` | `RunLimits` | Yes | Step, retry, and replanning budgets |
| `terminal_reason` | Structured reason | No | Populated when task reaches a terminal state |

### Validation Rules

- `goal` must not be empty.
- `status` must always be a valid lifecycle state.
- `limits` must define positive maximum step counts and non-negative recovery budgets.
- `terminal_reason` must be present whenever `status` is terminal.

### Relationships

- Owns exactly one `TaskContext`.
- Owns exactly one active `Plan` at a time.
- Owns exactly one `ExecutionTrace` per run.

### State Transitions

`planned` -> `running` -> `succeeded`

`planned` -> `running` -> `failed`

`planned` -> `running` -> `exhausted`

`planned` -> `running` -> `aborted`

## TaskContext

Represents the shared, mutable state available to every step in a task run.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `session_id` | Stable session identifier | Yes | Groups work performed in one task session |
| `workspace_ref` | Path or workspace descriptor | Yes | Points to the active workspace |
| `constraints` | Snapshot of `RunLimits` and related policies | Yes | Used by decision and recovery logic |
| `state` | Key-value map | Yes | Stores accumulated structured state across steps |
| `history_refs` | Ordered list of event or step-attempt identifiers | Yes | Allows later steps to inspect prior activity |
| `last_result` | Summary object | No | Most recent successful or failed step outcome |

### Validation Rules

- `workspace_ref` must be resolvable for the current run.
- `history_refs` must stay ordered by execution time.
- `last_result` must correspond to the most recent completed step attempt when present.

### Relationships

- Belongs to one `Task`.
- Is read by every `Step` execution.
- Is updated by the orchestrator after each completed step attempt.

## Plan

Represents the current ordered path of execution.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `revision` | Integer | Yes | Increments after each replanning event |
| `steps` | Ordered list of `Step` | Yes | Includes pending, running, completed, and superseded steps |
| `current_step_index` | Integer | Yes | Points to the next executable step |
| `status` | Enum | Yes | `active`, `completed`, or `superseded` |

### Validation Rules

- `current_step_index` must remain within the bounds of the active step list.
- At least one executable step must exist before a plan can transition to `active`.
- Replanning must preserve completed steps and only replace or extend remaining work.

### Relationships

- Belongs to one `Task`.
- Contains one or more `Step` records.
- Emits `ReplanningEvent` records when revised.

### State Transitions

`active` -> `completed`

`active` -> `superseded`

## Step

Represents one executable unit of work.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `id` | Stable identifier | Yes | Unique within the task run |
| `kind` | Enum | Yes | `agent`, `tool`, or `decision` |
| `target_name` | String | No | Required for agent/tool steps |
| `input` | Structured payload | Yes | Snapshot of inputs passed to execution |
| `status` | Enum | Yes | `pending`, `running`, `succeeded`, `failed`, or `skipped` |
| `attempt_count` | Integer | Yes | Number of execution attempts made |
| `output` | Structured payload | No | Present after successful completion |
| `error` | Structured failure payload | No | Present after failed completion |
| `recoverability` | Enum | No | `retryable`, `replan_required`, or `terminal` after failure evaluation |

### Validation Rules

- `target_name` must be present for `agent` and `tool` kinds.
- `attempt_count` must start at `0` and increase monotonically.
- `output` and `error` must not both be populated for the same finalized attempt.

### Relationships

- Belongs to one `Plan` revision.
- Produces one or more `StepAttempt` records.
- May reference one `ExecutionEndpoint` by name.

### State Transitions

`pending` -> `running` -> `succeeded`

`pending` -> `running` -> `failed`

`pending` -> `skipped`

## StepAttempt

Captures one concrete attempt to execute a step.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `attempt_id` | Stable identifier | Yes | Unique for each attempt |
| `step_id` | Step identifier | Yes | Links back to the owning step |
| `started_at` | Timestamp | Yes | Attempt start time |
| `ended_at` | Timestamp | No | Attempt end time |
| `input_snapshot` | Structured payload | Yes | Immutable view of the request sent to the endpoint |
| `result_snapshot` | Structured payload | No | Immutable normalized result |
| `failure_kind` | Enum | No | Captures recoverable vs unrecoverable failure shape |

### Validation Rules

- `ended_at` must not be earlier than `started_at`.
- `result_snapshot` or `failure_kind` must be present when an attempt completes.

## ExecutionTrace

Represents the inspectable record of the task run.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `task_id` | Task identifier | Yes | Links the trace to its task |
| `events` | Ordered list of trace events | Yes | Includes step changes, retry events, replanning events, and terminal outcome |
| `terminal_status` | Enum | No | Populated when the run ends |
| `terminal_reason` | Structured reason | No | Explains why the task stopped |
| `trace_location` | File reference | No | Local path to persisted trace output |

### Validation Rules

- Events must remain time-ordered.
- Terminal fields must be present together once the task ends.
- Every retry or replanning decision must have a corresponding event.

## ReplanningEvent

Represents one revision to the remaining plan.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `event_id` | Stable identifier | Yes | Traceable within the execution history |
| `from_revision` | Integer | Yes | Prior plan revision |
| `to_revision` | Integer | Yes | New plan revision |
| `trigger` | Structured reason | Yes | Evidence that invalidated the prior path |
| `replaced_step_ids` | List of step identifiers | No | Steps removed from the active path |
| `added_step_ids` | List of step identifiers | No | Steps introduced by replanning |

### Validation Rules

- `to_revision` must be greater than `from_revision`.
- `trigger` must reference evidence captured in the trace.

## ExecutionEndpoint

Represents a named agent or tool available to the orchestrator.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `name` | Unique string | Yes | Lookup key used by steps |
| `kind` | Enum | Yes | `agent` or `tool` |
| `capabilities` | List of strings | No | Used for diagnostics and future planning hints |
| `status` | Enum | Yes | `available` or `unavailable` for the current run |

### Validation Rules

- `(name, kind)` must be unique within the registry.
- Unavailable endpoints must not be selected for new step execution.

## RunLimits

Represents runtime limits and recovery budgets.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `max_steps` | Positive integer | Yes | Hard cap on total executed step attempts |
| `max_retries` | Non-negative integer | Yes | Hard cap on retry attempts across the task |
| `max_replans` | Non-negative integer | Yes | Hard cap on plan revisions after the initial plan |
| `terminal_precedence` | Ordered list of terminal conditions | Yes | Defines deterministic stop ordering |

### Validation Rules

- `max_steps` must be greater than zero.
- `terminal_precedence` must define a total order across all terminal conditions supported in v1.
