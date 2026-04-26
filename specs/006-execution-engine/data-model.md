# Data Model: Execution Engine (Code Delivery)

## WorkspaceExecutionProfile

- Purpose: Defines the bounded delivery configuration for one workspace run.
- Fields:
  - `name`: Stable profile identifier used in diagnostics and traces.
  - `read_targets`: Relative workspace paths that the analysis step snapshots before any mutation.
  - `validation_command`: Command specification used to validate each delivery attempt.
  - `attempts`: Ordered list of `ExecutionAttemptDefinition` records.
  - `limits`: Task-level step, retry, and replan limits used by the orchestrator.
  - `legacy_source`: Optional marker that records whether the profile was loaded from the new execution manifest or converted from the legacy fixture manifest.
- Validation rules:
  - `name` must not be empty.
  - `validation_command.program` must not be empty.
  - `attempts` must contain at least one attempt.
  - Every `read_targets` path must stay inside the workspace boundary.
  - Converted legacy profiles must remain behaviorally equivalent to the old fixture definition for the same workspace.

## ExecutionAttemptDefinition

- Purpose: Represents one bounded change attempt that Synod can apply before validating the result.
- Fields:
  - `attempt_id`: Stable identifier for traces and task state.
  - `changes`: Ordered list of `WorkspaceChange` records.
  - `failure_mode`: Declares whether validation failure should request retry, replan, or terminal stop.
  - `summary`: Human-readable description of what the attempt is trying to change.
- Validation rules:
  - `attempt_id` must be unique within the parent profile.
  - `changes` must contain at least one entry.
  - `failure_mode` must map to one of the existing recovery classes supported by the orchestrator.
- State transitions:
  - Attempt `n` is selected for the initial code step.
  - If validation requests replanning and another attempt exists, the planner replaces the remaining steps with attempt `n + 1`.
  - If no later attempt exists, the orchestrator terminates according to the configured limits and failure mode.

## WorkspaceChange

- Purpose: Describes one file mutation inside the workspace.
- Fields:
  - `path`: Relative path to the file being changed.
  - `find`: Existing content that must be located before replacement.
  - `replace`: New content written into the file.
- Validation rules:
  - `path` must be relative and must not escape the workspace root.
  - `find` must not be empty.
  - Applying the change must either mutate the file or be reported as already applied with explicit evidence.
- Relationships:
  - Many `WorkspaceChange` records belong to one `ExecutionAttemptDefinition`.

## ChangeEvidence

- Purpose: Stores inspectable proof of the file mutation performed by one attempt.
- Fields:
  - `path`: Relative workspace path.
  - `change_status`: One of `updated`, `already_applied`, or `missing_target`.
  - `before_excerpt`: Bounded snapshot of the original matched content.
  - `after_excerpt`: Bounded snapshot of the written content.
  - `diff_preview`: Stable text summary showing the attempted change.
- Validation rules:
  - Evidence must be emitted for every attempted file mutation.
  - `diff_preview` must stay bounded enough for status and inspect output.
  - `missing_target` evidence must cause the attempt to fail visibly.

## ValidationRecord

- Purpose: Captures the validation result for one attempt.
- Fields:
  - `command`: Rendered validation command.
  - `exit_code`: Process exit code.
  - `stdout`: Captured standard output.
  - `stderr`: Captured standard error.
  - `succeeded`: Boolean outcome used to determine terminal success or recovery.
- Validation rules:
  - Every delivery attempt must end with one `ValidationRecord`.
  - Missing or failed command launches are represented as non-success validation records or explicit terminal execution errors.

## ExecutionCapabilityProfile

- Purpose: Declares the concrete workspace operations that the runtime is allowed to perform during the current slice.
- Fields:
  - `can_read_workspace`: Whether analysis may snapshot files.
  - `can_write_workspace`: Whether change attempts may mutate files.
  - `can_run_validation`: Whether validation commands may be launched.
  - `supports_change_evidence`: Whether the runtime can emit bounded diff-style evidence.
- Validation rules:
  - The initial execution-engine slice requires all four capabilities.
  - Diagnostics must fail if the workspace or manifest prevents any required capability.

## Session Evidence Projection

- Purpose: Defines the subset of execution evidence surfaced in session status and next guidance.
- Fields:
  - `latest_changed_files`: Ordered list of relative file paths from the most recent successful or failed change attempt.
  - `latest_validation_status`: One of `passed`, `failed`, or `not_run`.
  - `latest_attempt_id`: Identifier of the most recent attempt.
  - `latest_trace_ref`: Persisted path to the current trace file.
- Validation rules:
  - Session projections are derived from task context and must remain optional for older sessions.
  - Status output must omit fields cleanly when no execution evidence exists yet.