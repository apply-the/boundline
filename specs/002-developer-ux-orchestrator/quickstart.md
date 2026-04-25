# Quickstart: Developer UX for Orchestrator Core

## Prerequisites

1. Install Rust 1.95.0 with `cargo`, `rustfmt`, and `clippy` available.
2. Work from the repository root on branch `002-developer-ux-orchestrator`.
3. Use a writable workspace so Synod can persist traces under `.synod/traces/` and load `.synod/fixture.json`.
4. Build and run the CLI from the same repository checkout as the library crate.

## Command Surface

- `synod doctor`: checks local readiness for developer runs.
- `synod run`: starts a simple bounded custom objective through the default developer flow.
- `synod inspect`: renders a readable summary from a persisted trace.

## Local Walkthrough

### 1. Verify local readiness

```bash
cargo run --bin synod -- doctor --workspace "$PWD"
```

Expected outcome:

- The command reports whether the current checkout is ready.
- Blocking problems are listed with actionable guidance.

### 2. Run the fixture-backed validation slice

```bash
cargo run --bin synod -- run --goal "Fix the failing fixture" --workspace "$PWD"
```

Expected outcome:

- The command validates the workspace fixture and confirms the initial verification command is red.
- The command prints step-by-step progress for the analyze/code/verify slice.
- The fixture's failing verification command turns green after the patch is applied.
- The command ends with an explicit terminal outcome and a trace location.

### 3. Inspect the recorded trace

```bash
cargo run --bin synod -- inspect --trace "$PWD/.synod/traces/<task-id>.json"
```

Expected outcome:

- The command reconstructs executed step order.
- Retry and replanning events are summarized in readable text when present.
- The final terminal status and terminal reason are immediately visible.

You can also inspect the latest trace in a workspace directly:

```bash
cargo run --bin synod -- inspect --workspace "$PWD"
```

### 4. Run a simple custom objective

```bash
cargo run --bin synod -- run --goal "Summarize the current bounded developer flow" --workspace "$PWD"
```

Expected outcome:

- The command validates the goal and local workspace fixture.
- The default analyze/code/verify slice executes through the existing orchestrator core.
- The command reports the terminal reason and where the trace was written.

### 5. Exercise a non-success fixture run

```bash
cargo run --bin synod -- run --goal "Attempt the broken fixture" --workspace "$BROKEN_FIXTURE"
```

Expected outcome:

- The command exits with a non-success status.
- The terminal reason remains readable from command output alone, such as a missing patch target or failing verification step.
- A trace is still persisted for later inspection.

## Validation Commands

Run these commands from the repository root:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Minimum Validation Scenarios

1. A contributor can run `doctor` and get either a ready state or actionable setup guidance.
2. The fixture-backed run reaches a terminal outcome and proves that a failing verification command can become passing.
3. A simple custom run reports progress, exit status, and trace location without requiring library code.
4. Trace inspection reconstructs step order, recovery events, and terminal reason from a stored trace alone.

## Exit Criteria

- A first-time contributor can reach a working fixture-backed run in under 5 minutes from the documented local checkout.
- Each command returns an explicit success or non-success exit outcome.
- The fixture-backed run path remains deterministic and locally debuggable.
- Persisted traces remain the source of truth for inspection and troubleshooting.