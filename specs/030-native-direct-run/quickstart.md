# Quickstart: Native Direct Run

**Feature**: 030-native-direct-run  
**Date**: 2026-05-02

This walkthrough shows the intended operator story for `0.30.0`: direct
`run --goal` enters the primary native session route by default, explicit
compatibility stays subordinate, and the resulting session remains inspectable
through the existing CLI surfaces.

## 1. Start from one direct native run command

```bash
cargo run --bin boundline -- run --goal "Fix the failing add test"
```

Expected behavior:
- Boundline bootstraps native session state automatically.
- The command reports native routing instead of compatibility routing.
- The workspace is mutated and validated through the existing bounded native
  route.

## 2. Continue from persisted native session state

```bash
cargo run --bin boundline -- status
cargo run --bin boundline -- next
cargo run --bin boundline -- inspect
```

Expected behavior:
- `status`, `next`, and `inspect` continue from the persisted native session and
  trace story created by the direct run bootstrap.
- `execution_path`, routing, and next-step guidance remain native and coherent.

## 3. Keep compatibility execution explicit

```bash
cargo run --bin boundline -- run --goal "Fix the failing add test" --compatibility
```

Expected behavior:
- Boundline uses the explicit compatibility route only because the operator asked
  for it deliberately.
- Run, inspect, and follow-up output keep compatibility ownership explicit.

## 4. Protect active session state

```bash
cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "Fix the failing add test"
cargo run --bin boundline -- run --goal "Ship the checkout change"
```

Expected behavior:
- The last command does not silently overwrite meaningful active session state.
- Boundline stops explicitly and tells the operator whether to continue, inspect, or
  reset the session first.

## 5. Validate the release story

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected behavior:
- The `0.30.0` docs and assistant guidance describe direct run as native-first.
- Modified or created Rust files remain above 95% coverage.
- Formatting, clippy, and compilation complete cleanly.