# Data Model: Developer UX for Orchestrator Core

## DeveloperCommandSession

Represents one invocation of the developer-facing command surface.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `command_name` | Enum | Yes | `run`, `inspect`, or `doctor` |
| `workspace_ref` | Path-like string | No | Present for `doctor`, `run`, and workspace-local trace lookup |
| `goal` | Non-empty string | No | Present for `run` |
| `trace_ref` | Path-like string | No | Present for `inspect` when a trace is explicitly selected |
| `started_at` | Timestamp | Yes | Command start time |
| `completed_at` | Timestamp | No | Command completion time |
| `exit_status` | Enum | No | `succeeded`, `non_success`, or `invalid_invocation` |
| `trace_location` | Path-like string | No | Present when a run creates or surfaces a persisted trace |

### Validation Rules

- `workspace_ref` must be present for `doctor` and `run`.
- `goal` must be present and non-empty for `run`.
- `trace_ref` or `workspace_ref` must be present for `inspect`.
- `exit_status` must be populated when the command completes.

### Relationships

- A `DeveloperCommandSession` may load one `WorkspaceFixture`.
- A `DeveloperCommandSession` may create or inspect one `TraceSummaryView`.
- A `DeveloperCommandSession` may emit one `DiagnosticsReport`.

### State Transitions

`requested` -> `validating` -> `executing` -> `completed`

`requested` -> `validating` -> `failed`

## WorkspaceFixture

Represents the deterministic repository-local manifest used by the run command.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `name` | String | Yes | Stable fixture name for output and trace context |
| `test_command` | Structured payload | Yes | Local verification command executed before and after patching |
| `limits` | Run-limits snapshot | Yes | Bounded steps, retries, and replans used by the fixture-backed slice |
| `file_patches` | Ordered list | Yes | Patch instructions with `path`, `find`, and `replace` |

### Validation Rules

- `test_command.program` must be present and runnable from the workspace.
- `file_patches` must contain at least one patch instruction.
- Each patch path must be relative to the workspace and each `find` pattern must be non-empty.
- `limits` must satisfy the existing run-limit validation rules.

### Relationships

- Used by `DeveloperCommandSession` when `command_name = run`.
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

### Validation Rules

- `goal` must not be empty.
- `workspace_ref` must point to a local writable workspace.
- `limits` must satisfy the existing run-limit validation rules.

### Relationships

- Consumed by one `DeveloperCommandSession` with `command_name = run`.
- Consumes one `WorkspaceFixture` loaded from `.synod/fixture.json`.
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
- May be consumed before `run` starts.