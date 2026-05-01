# Quickstart: Session And Compatibility Continuity

**Feature**: 022-session-compatibility-continuity  
**Date**: 2026-05-01

## Scenario 1: Show Compatibility Follow-Up Without Replacing Native Session State

```bash
cd /tmp/synod-session-compatibility-continuity
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "Fix the failing add test"
cargo run --bin synod -- plan --workspace . --flow bug-fix
cargo run --bin synod -- run --workspace . --goal "Fix the failing add test"
cargo run --bin synod -- status --workspace .
```

**Expected**:
- The compatibility run remains explicitly attributed to the compatibility route.
- The active native session is not silently replaced.
- `status` explains the authoritative follow-up state and does not blur route ownership.

## Scenario 2: Recommend Inspect-Oriented Continuity When No Compatibility Session Exists

```bash
cd /tmp/synod-session-compatibility-continuity
cargo run --bin synod -- run --workspace . --goal "Fix the failing add test"
cargo run --bin synod -- next --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- `next` recommends an inspect-oriented follow-up when only a latest compatibility trace is authoritative.
- `inspect` resolves the latest compatibility trace without requiring a manual trace path.
- The output keeps routing, execution condition, and terminal or recovery condition explicit.

## Scenario 3: Reuse Shared Summary Wording Across Routes

```bash
cd /tmp/synod-session-compatibility-continuity
cargo run --bin synod -- run --workspace . --goal "Recover after adaptive validation points to helper.rs"
cargo run --bin synod -- inspect --workspace .
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "Recover after adaptive validation points to helper.rs"
cargo run --bin synod -- plan --workspace . --flow bug-fix
cargo run --bin synod -- run --workspace .
```

**Expected**:
- Native and compatibility outputs use aligned wording for overlapping adaptive, review, governance, and terminal summaries.
- Route ownership stays explicit even where summary vocabulary converges.

## Validation Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo test --test contract runtime_routing_contract
cargo test --test integration runtime_refoundation_compat
cargo test --test integration session_adaptive_flow
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo nextest run --workspace --all-features
cargo deny check licenses advisories bans sources
```

## Release Closeout

```bash
grep '^version =' Cargo.toml
git diff -- README.md docs/getting-started.md docs/configuration.md docs/adaptive-execution.md assistant/README.md CONTRIBUTING.md ROADMAP.md CHANGELOG.md
```

**Expected**:
- The crate version is updated to `0.22.0` before the slice lands.
- Route continuity docs and assistant guidance describe native versus compatibility follow-up clearly.
- `lcov.info` is refreshed after the final modified Rust files and tests settle.