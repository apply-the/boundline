# Quickstart: Decision-Driven Orchestrator

Exercise the complete `0.34.0` story: selector-driven native execution,
decision-authoritative recovery, explicit ask or stop behavior, and release
validation.

## Preconditions

- Work from the repository root.
- Use a workspace with a bounded Rust task and at least one relevant source or test file.
- Ensure no stale `SPECIFY_FEATURE` environment variable points at an older slice when using Speckit scripts.

## Scenario 1: Native execution selects explicit bounded actions

1. Start a fresh session:
   `cargo run --bin boundline -- start`
2. Record a bounded goal:
   `cargo run --bin boundline -- goal --goal "fix the failing bounded add behavior"`
3. Plan the task:
   `cargo run --bin boundline -- plan`
4. Run the native path:
   `cargo run --bin boundline -- run`
5. Verify the output and trace expose a selector-driven sequence such as
   `read` or `search` before `modify`, followed by `test` when verification is due.

## Scenario 2: Decision-driven state remains visible on read-side surfaces

1. Inspect the active session:
   `cargo run --bin boundline -- status`
2. Ask for the next recommended action:
   `cargo run --bin boundline -- next`
3. Inspect the authoritative trace:
   `cargo run --bin boundline -- inspect`
4. Verify these surfaces expose the current selector, its rationale, the
   evidence basis, and any verification or recovery intent.

## Scenario 3: Ask, replan, or stop occurs explicitly

1. Run a bounded task where current evidence is insufficient or validation keeps failing.
2. Continue with:
   `cargo run --bin boundline -- run`
3. Verify Boundline surfaces `ask`, `replan`, retry, or terminal stop explicitly
   rather than silently exhausting a static plan.
4. Verify `status`, `next`, or `inspect` keep the same bounded recovery or stop story visible.

## Scenario 4: Release closeout

1. Run formatting:
   `cargo fmt --all`
2. Run linting:
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. Run compile-oriented validation when needed:
   `cargo test --no-run --all-targets`
4. Run the broader suite:
   `cargo nextest run --workspace --all-features`
5. Refresh coverage and confirm touched Rust files remain above 95%.