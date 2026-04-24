# Quickstart: Developer UX for Orchestrator Core

## Prerequisites

1. Install Rust 1.95.0 with `cargo`, `rustfmt`, and `clippy` available.
2. Work from the repository root on branch `002-developer-ux-orchestrator`.
3. Use a writable workspace so Synod can persist traces under `.synod/traces/`.
4. Build and run the CLI from the same repository checkout as the library crate.

## Command Surface

- `synod doctor`: checks local readiness for developer runs.
- `synod demo`: runs the deterministic guided demo with visible progress and at least one recovery event.
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

### 2. Run the guided demo

```bash
cargo run --bin synod -- demo --workspace "$PWD"
```

Expected outcome:

- The command prints step-by-step progress for the deterministic demo profile.
- At least one retry or replanning event is visible.
- The command ends with an explicit terminal outcome and a trace location.

### 3. Run a simple custom objective

```bash
cargo run --bin synod -- run --goal "Summarize the current bounded developer flow" --workspace "$PWD"
```

Expected outcome:

- The command validates the goal and local workspace.
- The default developer flow executes through the existing orchestrator core.
- The command reports the terminal reason and where the trace was written.

### 4. Inspect the recorded trace

```bash
cargo run --bin synod -- inspect --trace "$PWD/.synod/traces/<task-id>.json"
```

Expected outcome:

- The command reconstructs executed step order.
- Retry and replanning events are summarized in readable text.
- The final terminal status and terminal reason are immediately visible.

You can also inspect the latest trace in a workspace directly:

```bash
cargo run --bin synod -- inspect --workspace "$PWD"
```

### 5. Exercise a non-success custom run

```bash
cargo run --bin synod -- run --goal "Force a non-success failure for the default developer flow" --workspace "$PWD"
```

Expected outcome:

- The command exits with a non-success status.
- The terminal reason remains readable from command output alone.
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
2. The guided demo reaches a terminal outcome and visibly demonstrates recovery behavior.
3. A simple custom run reports progress, exit status, and trace location without requiring library code.
4. Trace inspection reconstructs step order, recovery events, and terminal reason from a stored trace alone.

## Exit Criteria

- A first-time contributor can reach a working demo run in under 5 minutes from the documented local checkout.
- Each command returns an explicit success or non-success exit outcome.
- The demo and default custom flow remain deterministic and locally debuggable.
- Persisted traces remain the source of truth for inspection and troubleshooting.