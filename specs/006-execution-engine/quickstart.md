# Quickstart: Execution Engine (Code Delivery)

## Prerequisites

- Work from the repository root.
- Use a clean workspace or start with no active `.boundline/session.json`.
- Provide either `.boundline/execution.json` or the legacy `.boundline/fixture.json` in the target workspace.
- Run commands through `cargo run --bin boundline -- ...` when validating locally.

## Example execution profile

Create `.boundline/execution.json` in a small Rust workspace:

```json
{
  "name": "red-to-green-execution",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "attempts": [
    {
      "attempt_id": "fix-add",
      "summary": "Replace subtraction with addition",
      "failure_mode": "replan",
      "changes": [
        {
          "path": "src/lib.rs",
          "find": "left - right",
          "replace": "left + right"
        }
      ]
    }
  ]
}
```

## Scenario 1: Run a bounded delivery task directly

1. Execute a delivery run against the workspace:

   ```bash
   cargo run --bin boundline -- run --goal "Fix the failing add test" --workspace <workspace>
   ```

2. Inspect the latest trace:

   ```bash
   cargo run --bin boundline -- inspect --workspace <workspace>
   ```

Expected outcome:

- `run` applies the configured change set inside the workspace.
- Validation runs after the change attempt.
- The terminal output includes `changed_files`, `validation`, a trace reference, and a succeeded or non-success terminal status.
- `inspect` exposes changed files, validation evidence, and the terminal reason.

## Scenario 2: Use the execution engine through the session workflow

1. Start and capture a goal:

   ```bash
   cargo run --bin boundline -- start --workspace <workspace>
   cargo run --bin boundline -- capture --workspace <workspace> --goal "Fix the failing add test"
   ```

2. Plan and run the task:

   ```bash
   cargo run --bin boundline -- plan --workspace <workspace>
   cargo run --bin boundline -- run --workspace <workspace>
   ```

3. Inspect status:

   ```bash
   cargo run --bin boundline -- status --workspace <workspace>
   cargo run --bin boundline -- next --workspace <workspace>
   ```

Expected outcome:

- Session state persists the task, trace reference, and latest execution evidence.
- `status` surfaces the latest changed files and validation outcome when a delivery attempt has run.
- `next` points to `inspect` on success and to the next bounded action on non-success.

## Scenario 3: Replan after failed validation

1. Configure an execution profile with at least two attempts.
  Optional `limits` overrides may be partial; omitted fields inherit the default run limits.
2. Make the first attempt fail validation.
3. Re-run the task until it succeeds or exhausts its limits.

Expected outcome:

- The failed validation remains visible in the trace.
- A bounded replan or retry occurs according to the profile and limits.
- The final terminal state is explicit even when no later attempt succeeds.

## Coverage validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected outcome:

- All tests pass.
- `lcov.info` is regenerated from the same command used in CI.
- Every Rust source file under `src/` reaches at least 90% line coverage in the regenerated report.