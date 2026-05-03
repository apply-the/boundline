# Quickstart: Adaptive Repair Depth

**Feature**: 021-adaptive-repair-depth  
**Date**: 2026-05-01

## Scenario 1: Replan Adaptive Repair From Validation Hints

```bash
cd /tmp/boundline-adaptive-repair-depth
cargo run --bin boundline -- run --goal "Recover after adaptive validation points to a different file" --workspace .
cargo run --bin boundline -- status --workspace .
cargo run --bin boundline -- inspect --workspace .
```

**Expected**:
- The first adaptive validation failure produces explicit failure evidence.
- Boundline chooses a materially different bounded adaptive candidate because of that validation evidence.
- `status` and `inspect` show the updated workspace slice, selection headline, attempt lineage, and terminal or recovery condition.

## Scenario 2: Keep Route Ownership Explicit In A Workspace With Other Surfaces

```bash
cd /tmp/boundline-adaptive-repair-depth
cargo run --bin boundline -- workflow list --workspace .
cargo run --bin boundline -- run --goal "Recover after adaptive validation points to a different file" --workspace .
cargo run --bin boundline -- next --workspace .
```

**Expected**:
- Adaptive execution remains visibly on the compatibility route.
- The presence of named workflows does not imply workflow-owned adaptive control.
- Any review or governance projection remains additive and does not replace the adaptive route explanation.

## Scenario 3: Stop Explicitly When Guidance Cannot Produce A New Candidate

```bash
cd /tmp/boundline-adaptive-repair-depth
cargo run --bin boundline -- run --goal "Fail after no adaptive candidate remains" --workspace .
cargo run --bin boundline -- inspect --workspace .
```

**Expected**:
- Boundline stops in an explicit failed or exhausted terminal state.
- The trace explains that validation guidance did not make any materially different bounded candidate credible.
- Attempt lineage remains visible even though execution stopped non-successfully.

## Validation Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo test --test unit adaptive_execution
cargo test --test integration cli_adaptive_execution
cargo test --test contract adaptive_
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo nextest run --workspace --all-features
cargo deny check licenses advisories bans sources
```

## Release Closeout

```bash
grep '^version =' Cargo.toml
git diff -- README.md docs/adaptive-execution.md docs/getting-started.md docs/configuration.md assistant/README.md CONTRIBUTING.md ROADMAP.md CHANGELOG.md
```

**Expected**:
- The crate version is updated to `0.21.0` before implementation lands.
- Adaptive docs and changelog describe validation-guided bounded repair and the explicit compatibility-route story coherently.
- `lcov.info` is refreshed after the final modified Rust files and tests settle.