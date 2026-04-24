# Data Model: Developer UX for Orchestrator Core

## DeveloperCommandSession

Represents one invocation of the developer-facing command surface.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `command_name` | Enum | Yes | `demo`, `run`, `inspect`, or `doctor` |
| `workspace_ref` | Path-like string | Yes | Local workspace used for diagnostics, execution, or trace lookup |
| `goal` | Non-empty string | No | Present for `run`; derived from the demo profile for `demo` |
| `trace_ref` | Path-like string | No | Present for `inspect` when a trace is explicitly selected |
| `started_at` | Timestamp | Yes | Command start time |
| `completed_at` | Timestamp | No | Command completion time |
| `exit_status` | Enum | No | `succeeded`, `non_success`, or `invalid_invocation` |
| `trace_location` | Path-like string | No | Present when a run creates or surfaces a persisted trace |

### Validation Rules

- `workspace_ref` must be present for all commands.
- `goal` must be present and non-empty for `run`.
- `trace_ref` must be present for `inspect` unless the command uses a supported local default such as the latest trace in a workspace.
- `exit_status` must be populated when the command completes.

### Relationships

- A `DeveloperCommandSession` may use one `DemoRunProfile`.
- A `DeveloperCommandSession` may create or inspect one `TraceSummaryView`.
- A `DeveloperCommandSession` may emit one `DiagnosticsReport`.

### State Transitions

`requested` -> `validating` -> `executing` -> `completed`

`requested` -> `validating` -> `failed`

## DemoRunProfile

Represents the deterministic predefined task used by the guided demo command.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `name` | String | Yes | Stable profile name for the built-in demo |
| `goal` | Non-empty string | Yes | Human-readable bounded objective |
| `initial_input` | Structured payload | Yes | Seed input used to bootstrap the orchestrator request |
| `step_outline` | Ordered list | Yes | Describes the intended analysis, change, and verification path |
| `recovery_trigger_step` | Step identifier | Yes | The step where the demo intentionally shows a recoverable failure path |
| `limits` | Run-limits snapshot | Yes | Bounded steps, retries, and replans used by the demo |

### Validation Rules

- `step_outline` must contain at least one executable step.
- `recovery_trigger_step` must reference a step in `step_outline`.
- `limits` must preserve bounded execution and leave room for exactly the recovery path the profile intends to demonstrate.

### Relationships

- Used by `DeveloperCommandSession` when `command_name = demo`.
- Produces one orchestrator task run and one persisted trace.

## CustomRunRequest

Represents the bounded developer-supplied objective launched by the `run` command.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `goal` | Non-empty string | Yes | Developer-supplied local objective |
| `workspace_ref` | Path-like string | Yes | Workspace used for trace persistence and context |
| `limits` | Run-limits snapshot | Yes | Explicit or default bounded execution limits |
| `initial_context` | Structured payload | No | Optional initial state made available before planning |
| `profile_name` | String | No | Optional default developer flow selection when more than one local profile exists |

### Validation Rules

- `goal` must not be empty.
- `workspace_ref` must point to a local writable workspace.
- `limits` must satisfy the existing run-limit validation rules.

### Relationships

- Consumed by one `DeveloperCommandSession` with `command_name = run`.
- Produces one orchestrator task run and one persisted trace.

## TraceSummaryView

Represents the readable inspection view derived from a persisted trace.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `trace_ref` | Path-like string | Yes | Source trace location |
| `goal` | String | Yes | Goal recorded in the trace |
| `started_at` | Timestamp | Yes | Run start time |
| `ended_at` | Timestamp | No | Run end time when present |
| `executed_steps` | Ordered list of step summaries | Yes | Includes step order, step kind, attempts, and final per-step outcome |
| `recovery_events` | Ordered list of recovery summaries | Yes | Includes retries and replans with their triggering reason |
| `terminal_status` | Enum | Yes | Final task status |
| `terminal_reason` | Structured reason | Yes | Readable final reason for task stop |

### Validation Rules

- `executed_steps` must preserve trace order.
- `terminal_status` and `terminal_reason` must be present together.
- `recovery_events` may be empty, but if present they must map to events recorded in the source trace.

### Relationships

- Produced by `DeveloperCommandSession` when `command_name = inspect`.
- Derived from one persisted execution trace.

## DiagnosticsReport

Represents the readiness result produced by the `doctor` command.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `workspace_ref` | Path-like string | Yes | Workspace being checked |
| `checks` | Ordered list of readiness checks | Yes | Each check has a name, status, and actionable message |
| `ready` | Boolean | Yes | Indicates whether the command surface can proceed without a blocking setup problem |
| `missing_prerequisites` | List of strings | No | Blocking prerequisites surfaced by the checks |
| `suggested_actions` | List of strings | No | Follow-up guidance for unresolved readiness problems |

### Validation Rules

- `checks` must include every readiness concern the CLI depends on.
- `missing_prerequisites` must be empty when `ready = true`.
- Each failed check must have an actionable message.

### Relationships

- Produced by `DeveloperCommandSession` when `command_name = doctor`.
- May be consumed before `demo` or `run` starts.