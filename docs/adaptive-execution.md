# Adaptive Execution in Synod 0.22.0

Synod `0.22.0` keeps bounded adaptive execution as an explicit compatibility mode inside the broader session-native runtime. The primary operator path is still `start -> capture -> plan -> run -> status -> next -> inspect` with a bounded `GoalPlan`; adaptive behavior still uses `<workspace>/.synod/execution.json` when the operator intentionally chooses the manifest-backed compatibility path.

Instead of requiring every attempt to be pre-authored in `<workspace>/.synod/execution.json`, a compatibility execution profile can describe an `adaptive` block and let Synod choose one bounded workspace slice and one bounded candidate at a time. In `0.22.0`, failed validation can also guide the next adaptive slice selection when the latest validation record points to a more credible manifest-declared target, and the read-side commands can still point operators at the latest compatibility trace when no active session is resumable.

In `0.22.0`, operators can still generate the baseline compatibility profile with
`synod init`; adaptive behavior is still configured through the execution
manifest itself.

## What the runtime supports

- bounded workspace-slice scoring from `read_targets`
- validation-guided slice reselection after failed verification when the latest bounded evidence points elsewhere
- deterministic local candidate generation from the selected slice
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

When validation fails, Synod can also derive bounded guidance from the latest persisted validation record and the current failure message. If that evidence points to another file already listed in `read_targets`, the next adaptive attempt may shift to that file while preserving explicit `workspace_slice`, `selection_headline`, and `attempt_lineage` evidence.

## Candidate generation

Adaptive candidate synthesis remains deterministic and local.

- `arithmetic_swap`: swaps one arithmetic operator for another in a stable order
- `comparison_flip`: flips `==` and `!=`
- `boolean_flip`: flips `true` and `false`

Each candidate gets a stable signature derived from the file path and replacement pair. Synod records those signatures and does not repeat the same candidate after a failed validation.

Validation guidance changes ranking, not boundedness. Synod still only considers manifest-declared `read_targets`, built-in local change kinds, and sequential replans.

## What users see

When adaptive execution is active, the local CLI now exposes:

- `synod run`: explicit compatibility routing, `execution_condition`, `workspace_slice`, validation-guided `attempt_lineage` after replans, changed files, validation result, terminal status, and trace path
- `synod status`: explicit `routing`, `execution_condition`, `latest_workspace_slice`, `latest_selection_headline`, `latest_attempt_lineage` when present, `latest_validation_status`, and `continuity_authority` when the latest compatibility trace is the authoritative follow-up state
- `synod next`: the same adaptive session projection plus the CLI-reported next command, including inspect-only compatibility follow-up when no active session exists
- `synod inspect`: adaptive slice-selection headlines for both the initial and replacement attempts, `execution_condition`, validation results, recovery events, and final terminal reason

## Current scope

The `0.22.0` adaptive slice is intentionally bounded:

- candidate generation stays deterministic and bounded to the built-in local heuristics
- workspace selection only considers manifest-declared `read_targets`
- replanning remains sequential and local even when validation shifts the selected target
- adaptive change kinds are limited to the built-in local heuristics above
- review councils continue to work on top of adaptive attempts when configured

For an executable walkthrough, see `specs/008-adaptive-execution-engine/quickstart.md`.
