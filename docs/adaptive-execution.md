# Adaptive Execution in Synod 0.24.0

Synod `0.24.0` keeps bounded adaptive execution as an explicit compatibility
mode inside the broader session-native runtime. The primary operator path is
still `start -> capture -> plan -> run -> status -> next -> inspect` with a
bounded `GoalPlan`; adaptive behavior still uses
`<workspace>/.synod/execution.json` when the operator intentionally chooses the
manifest-backed compatibility path.

Instead of requiring every attempt to be pre-authored in
`<workspace>/.synod/execution.json`, a compatibility execution profile can
describe an `adaptive` block and let Synod choose one bounded workspace slice
and one bounded candidate at a time. In `0.24.0`, failed validation can guide
the next adaptive slice selection, rank one bounded mutation family over the
rest, and stop explicitly when the latest bounded evidence is absent or
insufficient for another materially different candidate.

Operators can still generate the baseline compatibility profile with
`synod init`; adaptive behavior is still configured through the execution
manifest itself rather than a new routing surface.

## What the runtime supports

- bounded workspace-slice scoring from `read_targets`
- validation-guided slice reselection after failed verification when the latest
  bounded evidence points elsewhere
- deterministic local candidate generation from the selected slice across
  multiple built-in bounded mutation families
- signature-based non-repeat behavior across replans
- family-aware candidate credibility and rejection summaries
- bounded replanning after failed validation when the latest evidence supports
  it
- explicit exhaustion when no remaining bounded candidate is credible enough or
  when validation evidence is absent or insufficient
- adaptive evidence projected into `.synod/session.json` and `.synod/traces/`
- adaptive summaries surfaced in `synod run`, `synod status`, `synod next`,
  and `synod inspect`

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
    "max_generated_attempts": 6,
    "path_preferences": ["src/"],
    "allowed_change_kinds": [
      "arithmetic_swap",
      "ordering_boundary_flip",
      "numeric_literal_flip"
    ]
  }
}
```

At least one of `attempts` or `adaptive` must be present. When `adaptive` is
configured, `read_targets` remain required because Synod selects the slice from
that bounded set.

## How selection works

Synod scores each `read_target` using only bounded local evidence:

- path preference matches such as `src/`
- goal terms from the active task
- validation-command terms
- whether the file supports one of the allowed adaptive change kinds

The highest-scoring bounded slice becomes the `workspace_slice` surfaced in CLI
output and persisted into task context state.

When validation fails, Synod can also derive bounded guidance from the latest
persisted validation record and the current failure message. If that evidence
points to another file already listed in `read_targets`, the next adaptive
attempt may shift to that file while preserving explicit `workspace_slice`,
`selection_headline`, `candidate_family`, `selection_reason`, and
`attempt_lineage` evidence.

## Candidate generation

Adaptive candidate synthesis remains deterministic and local.

- `arithmetic_swap`: swaps one arithmetic operator for another in a stable
  order
- `comparison_flip`: flips `==` and `!=`
- `boolean_flip`: flips `true` and `false`
- `ordering_boundary_flip`: flips `>`, `>=`, `<`, and `<=` at inclusive or
  exclusive boundaries
- `result_status_flip`: flips bounded `Ok(` and `Err(` result constructors
- `numeric_literal_flip`: flips bounded `0` and `1` literal patterns in common
  comparison, assignment, and return contexts

Each candidate gets a stable signature derived from the file path and
replacement pair. Synod records those signatures and does not repeat the same
candidate after a failed validation.

Validation guidance changes ranking, not boundedness. Synod still only
considers manifest-declared `read_targets`, built-in local change kinds, and
sequential replans. If failed validation does not provide enough bounded
evidence for another materially different candidate, Synod stops explicitly
instead of drifting into open-ended retries.

## What users see

When adaptive execution is active, the local CLI now exposes:

- `synod run`: explicit compatibility routing, `execution_condition`,
  `route_owner`, `route_config_projection` when workspace-local routing defaults
  or requested governance intent materially explain the run,
  `workspace_slice`, `candidate_family`, selection reason, rejected candidate
  summaries, validation-guided `attempt_lineage` after replans, changed files,
  validation result, terminal status, and trace path
- `synod status`: explicit `routing`, `execution_condition`,
  `route_owner`, `route_config_projection`,
  `latest_workspace_slice`, `latest_selection_headline`,
  `latest_candidate_family`, `latest_selection_reason`,
  `latest_rejected_candidates`, `latest_attempt_lineage`,
  `latest_validation_status`, `latest_exhaustion_reason` when present, and
  `continuity_authority` when the latest compatibility trace is the
  authoritative follow-up state
- `synod next`: the same adaptive session projection plus the CLI-reported next
  command, including inspect-only compatibility follow-up when no active
  session exists
- `synod inspect`: adaptive slice-selection headlines, `adaptive_evidence`
  lines, `route_owner`, `route_config_projection` when workspace-local routing
  defaults materially explain the trace, validation results, recovery events,
  and final terminal reason for
  both the selected candidate and explicit exhaustion cases

## Current scope

The `0.24.0` adaptive slice is intentionally bounded:

- candidate generation stays deterministic and bounded to the built-in local
  heuristics
- workspace selection only considers manifest-declared `read_targets`
- replanning remains sequential and local even when validation shifts the
  selected target
- adaptive change kinds are limited to the built-in local heuristics above
- absent or insufficient validation evidence ends bounded adaptive recovery
  explicitly
- review councils continue to work on top of adaptive attempts when configured

For an executable walkthrough, see
`specs/008-adaptive-execution-engine/quickstart.md`.
