# Adaptive Execution in Synod 0.17.0

Synod `0.17.0` keeps bounded adaptive execution as an explicit compatibility mode inside the broader session-native runtime. The primary operator path is `start -> capture -> plan -> run -> status -> next -> inspect` with a bounded `GoalPlan`; adaptive behavior still uses `<workspace>/.synod/execution.json` when the operator intentionally chooses the manifest-backed compatibility path.

Instead of requiring every attempt to be pre-authored in `<workspace>/.synod/execution.json`, a compatibility execution profile can describe an `adaptive` block and let Synod choose one workspace slice and one deterministic candidate at a time.

In `0.17.0`, operators can still generate the baseline compatibility profile with
`synod init`; adaptive behavior is still configured through the execution
manifest itself.

## What the runtime supports

- bounded workspace-slice scoring from `read_targets`
- deterministic candidate generation from the selected slice
- signature-based non-repeat behavior across replans
- bounded replanning after failed validation
- adaptive evidence projected into `.synod/session.json` and `.synod/traces/`
- adaptive summaries surfaced in `synod run`, `synod status`, `synod next`, and `synod inspect`

## Manifest shape

```json
{
  "name": "adaptive-red-to-green",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "attempts": [],
  "adaptive": {
    "max_selected_targets": 1,
    "max_generated_attempts": 4,
    "path_preferences": ["src/"],
    "allowed_change_kinds": ["arithmetic_swap"]
  }
}
```

At least one of `attempts` or `adaptive` must be present. When `adaptive` is configured, `read_targets` remain required because Synod selects the slice from that bounded set.

## How selection works

Synod scores each `read_target` using only bounded local evidence:

- path preference matches such as `src/`
- goal terms from the active task
- validation-command terms
- whether the file supports one of the allowed adaptive change kinds

The highest-scoring bounded slice becomes the `workspace_slice` surfaced in CLI output and persisted into task context state.

## Candidate generation

The initial adaptive slice keeps candidate synthesis deterministic and local.

- `arithmetic_swap`: swaps one arithmetic operator for another in a stable order
- `comparison_flip`: flips `==` and `!=`
- `boolean_flip`: flips `true` and `false`

Each candidate gets a stable signature derived from the file path and replacement pair. Synod records those signatures and does not repeat the same candidate after a failed validation.

## What users see

When adaptive execution is active, the local CLI now exposes:

- `synod run`: explicit compatibility routing, `execution_condition`, `workspace_slice`, `attempt_lineage` after replans, changed files, validation result, terminal status, and trace path
- `synod status`: explicit `routing`, `execution_condition`, `latest_workspace_slice`, `latest_selection_headline`, `latest_attempt_lineage` when present, and `latest_validation_status`
- `synod next`: the same adaptive session projection plus the CLI-reported next command
- `synod inspect`: adaptive slice-selection headlines, `execution_condition`, attempt-specific change headlines, validation results, recovery events, and final terminal reason

## Current scope

The `0.17.0` adaptive slice is intentionally bounded:

- candidate generation is deterministic, not open-ended
- workspace selection only considers manifest-declared `read_targets`
- replanning remains sequential and local
- adaptive change kinds are limited to the built-in local heuristics above
- review councils continue to work on top of adaptive attempts when configured

For an executable walkthrough, see `specs/008-adaptive-execution-engine/quickstart.md`.
