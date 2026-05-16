# Quickstart: Adaptive Execution Engine

## Prerequisites

- Work from the repository root.
- Use a clean workspace or start with no active `.boundline/session.json`.
- Provide `.boundline/execution.json` in the target workspace.
- Run commands through `cargo run --bin boundline -- ...` when validating locally.

## Example adaptive execution profile

Create `.boundline/execution.json` in a small Rust workspace:

```json
{
  "name": "adaptive-red-to-green",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "adaptive": {
    "max_selected_targets": 1,
    "max_generated_attempts": 4,
    "path_preferences": ["src/", "tests/"],
    "allowed_change_kinds": ["arithmetic_swap", "comparison_flip", "boolean_flip"]
  }
}
```

## Scenario 1: Run an adaptive delivery task directly

1. Execute a delivery run against the workspace:

   ```bash
   cargo run --bin boundline -- run --goal "Fix the failing add test"
   ```

2. Inspect the latest trace:

   ```bash
   cargo run --bin boundline -- inspect
   ```

Expected outcome:

- `run` selects one bounded workspace slice from the configured `read_targets`.
- One adaptive candidate attempt is synthesized from the selected file content.
- Validation runs after the adaptive change attempt.
- The terminal output includes the selected workspace slice, changed files, validation summary, trace reference, and succeeded or non-success terminal status.
- `inspect` exposes slice-selection evidence, attempt lineage, validation output, and the terminal reason.

## Scenario 2: Replan to a new bounded attempt after failed validation

1. Configure a workspace where the first deterministic adaptive repair candidate does not satisfy validation.
2. Run the task.
3. Inspect the latest trace and status output.

Expected outcome:

- The failed validation remains visible in the trace.
- The next attempt uses a different candidate signature or a different bounded slice instead of repeating the same failed path.
- `status` and `next` surface the latest adaptive slice, validation outcome, and attempt-lineage summary.
- The run stops explicitly if no credible next candidate remains.

## Scenario 3: Adaptive execution with review enabled

1. Add a bounded review profile to the same execution manifest.
2. Run a delivery task that adaptive execution can complete successfully.
3. Inspect `run`, `status`, `next`, and `inspect`.

Expected outcome:

- Adaptive execution still records slice selection and attempt lineage.
- Review triggers only after a reviewable terminal delivery result exists.
- Delivery and review evidence appear together in the same session and trace surfaces.

## Validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected outcome:

- All tests pass.
- Adaptive profile, planner, trace, and session scenarios pass.
- `lcov.info` is regenerated from the same command used in CI.
- The crate version and user-facing docs are updated to `0.8.0`.
