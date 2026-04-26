# Data Model: Adaptive Execution Engine

## WorkspaceExecutionProfile

- Purpose: Defines the bounded delivery configuration for one workspace run.
- Fields:
  - `name`: Stable profile identifier used in diagnostics and traces.
  - `read_targets`: Relative workspace paths that bound adaptive slice selection.
  - `validation_command`: Command specification used to validate each delivery attempt.
  - `attempts`: Optional ordered list of deterministic authored attempts preserved for compatibility with Spec 006.
  - `adaptive`: Optional `AdaptiveExecutionProfile` used when attempts are not pre-authored or when adaptive selection is explicitly requested.
  - `limits`: Task-level step, retry, and replan limits used by the orchestrator.
  - `review`: Optional bounded review configuration preserved from Spec 007.
  - `legacy_source`: Optional marker showing whether the profile was loaded from the new execution manifest or converted from the legacy fixture manifest.
- Validation rules:
  - `name` must not be empty.
  - `validation_command.program` must not be empty.
  - At least one of `attempts` or `adaptive` must be present.
  - `read_targets` must remain inside the workspace boundary.
  - `adaptive` requires at least one readable target and bounded candidate limits.

## AdaptiveExecutionProfile

- Purpose: Declares the bounded rules for adaptive slice selection and candidate generation.
- Fields:
  - `max_selected_targets`: Maximum number of paths that may form one workspace slice for an attempt.
  - `max_generated_attempts`: Maximum number of synthesized candidate attempts available to one run.
  - `path_preferences`: Optional ordered hints that bias slice scoring toward source, tests, or specific subtrees.
  - `allowed_change_kinds`: Optional bounded list of deterministic repair heuristic kinds.
- Validation rules:
  - `max_selected_targets` must be greater than zero.
  - `max_generated_attempts` must be greater than zero.
  - `allowed_change_kinds`, when present, must come from the supported bounded vocabulary.

## WorkspaceSliceSelection

- Purpose: Captures the bounded subset of the workspace chosen for one adaptive attempt.
- Fields:
  - `selection_id`: Stable identifier for trace and lineage output.
  - `selected_targets`: Ordered list of relative file paths chosen for the current attempt.
  - `scored_candidates`: Bounded list of path-score pairs explaining what was considered.
  - `headline`: Short human-readable summary of the selected slice.
- Validation rules:
  - `selected_targets` must not be empty.
  - Every selected target must come from configured `read_targets`.
  - `selected_targets.len()` must not exceed `max_selected_targets`.

## AdaptiveAttemptCandidate

- Purpose: Represents one synthesized delivery attempt derived from the current workspace slice.
- Fields:
  - `attempt_id`: Stable identifier surfaced to traces and CLI output.
  - `candidate_signature`: Stable content-based signature used to avoid materially identical retries.
  - `slice_selection`: The `WorkspaceSliceSelection` used to derive the attempt.
  - `changes`: Ordered list of `WorkspaceChange` records generated for the attempt.
  - `failure_mode`: Declares whether failed validation should request retry, replan, or terminal stop.
  - `selection_evidence`: `SelectionEvidence` describing why this attempt was chosen now.
- Validation rules:
  - `candidate_signature` must be stable for the same synthesized change set.
  - `changes` must not be empty.
  - `changes` must stay within the currently selected workspace slice.

## SelectionEvidence

- Purpose: Makes adaptive decisions inspectable.
- Fields:
  - `goal_terms`: Bounded set of goal-derived tokens used during scoring.
  - `validation_terms`: Optional bounded set of terms extracted from the latest validation output.
  - `path_scores`: Ordered summary of why candidate paths ranked as they did.
  - `reason`: Short statement explaining why this slice and candidate were selected.
- Validation rules:
  - `reason` must not be empty.
  - Score output must remain bounded enough for CLI and trace rendering.

## AttemptLineage

- Purpose: Relates adaptive attempts across one bounded run.
- Fields:
  - `previous_attempt_id`: Optional earlier adaptive attempt.
  - `current_attempt_id`: The active attempt.
  - `transition_kind`: One of `initial`, `narrowed`, `broadened`, `replaced`, or `terminated`.
  - `reason`: Short explanation of why the planner moved to this next path.
- Validation rules:
  - Every attempt after the first must reference a previous attempt.
  - `transition_kind` must map cleanly to the planner decision taken.

## ValidationRecord

- Purpose: Captures the validation result for one attempt.
- Fields:
  - `command`: Rendered validation command.
  - `exit_code`: Process exit code.
  - `stdout`: Captured standard output.
  - `stderr`: Captured standard error.
  - `succeeded`: Boolean outcome used to determine terminal success or replanning.
- Validation rules:
  - Every delivery attempt must end with one `ValidationRecord`.
  - Validation failures remain inspectable even when the run replans.

## AdaptiveSessionProjection

- Purpose: Defines the subset of adaptive evidence surfaced through session status and next guidance.
- Fields:
  - `latest_workspace_slice`: Ordered list of the currently selected workspace targets.
  - `latest_selection_headline`: Short rendered summary of the current slice and reason.
  - `latest_attempt_lineage`: Short rendered summary of how the current attempt differs from the previous one.
  - `latest_validation_status`: One of `passed`, `failed`, or `not_run`.
  - `latest_trace_ref`: Persisted path to the current trace file.
- Validation rules:
  - Adaptive projections remain optional for non-adaptive runs.
  - When present, they must remain consistent with task context and latest trace evidence.
