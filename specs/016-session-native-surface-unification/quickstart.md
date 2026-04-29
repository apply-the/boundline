# Quickstart: Session-Native Surface Unification

**Feature**: 016-session-native-surface-unification  
**Date**: 2026-04-29

## Scenario 1: Native Status, Next, And Inspect Tell The Same Story

```bash
cd /tmp/session-native-surface
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "fix the failing add test"
cargo run --bin synod -- plan --workspace . --flow bug-fix
cargo run --bin synod -- run --workspace .
cargo run --bin synod -- status --workspace .
cargo run --bin synod -- next --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- `run`, `status`, `next`, and `inspect` agree on the chosen route and current execution condition.
- The latest decision state is explained consistently across the operator surfaces.

## Scenario 2: Adaptive Or Review Detail Extends The Same Summary Model

```bash
cd /tmp/session-native-surface-adaptive
cargo run --bin synod -- run --workspace . --goal "repair the bounded parser failure"
cargo run --bin synod -- status --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- Adaptive or review details appear as bounded projections of the same session-owned summary.
- The route explanation and next-step guidance remain consistent with the primary session-native story.

## Scenario 3: Governed Waiting State Stays Part Of The Session Story

```bash
cd /tmp/session-native-surface-governed
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "ship the governed change"
cargo run --bin synod -- plan --workspace . --no-flow
cargo run --bin synod -- run --workspace .
cargo run --bin synod -- status --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- A governance wait or block appears as an explicit execution condition rather than as an unrelated mode.
- `status` and `inspect` provide the same next action for the governed state.

## Scenario 4: Compatibility Mode Remains Explicit

```bash
cd /tmp/session-native-surface-compat
cargo run --bin synod -- run --workspace . --goal "fix the failing add test"
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- If compatibility behavior is the chosen path, operator surfaces label it explicitly as compatibility.
- If a ready session-native plan exists, compatibility does not silently take precedence.

## Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo deny check licenses advisories bans sources
```