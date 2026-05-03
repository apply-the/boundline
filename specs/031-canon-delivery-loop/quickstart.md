# Quickstart: Governed Delivery With Canon Inside The Loop

**Feature**: 031-canon-delivery-loop  
**Date**: 2026-05-02

This walkthrough captures the intended `0.31.0` operator story: one primary
session-native delivery path, Canon inside governed stages, real code change
evidence, and explicit stop conditions when delivery is not credible.

## 1. Run one governed delivery flow on the primary path

```bash
cargo run --bin boundline -- run --workspace <workspace> --goal "Fix the failing add test"
```

Expected behavior:
- Boundline stays on the native session path.
- Canon participates in the governed stage boundaries configured for the flow.
- The workspace gains a material bounded code diff.
- Validation passes before the run is allowed to complete.

## 2. Inspect the same governed follow-through story

```bash
cargo run --bin boundline -- status --workspace <workspace>
cargo run --bin boundline -- next --workspace <workspace>
cargo run --bin boundline -- inspect --workspace <workspace>
```

Expected behavior:
- The same session and trace story remains available after the run.
- Governance state, packet lineage, changed files, and validation evidence stay
  visible on current CLI surfaces.
- `next_command` stays aligned with the same authoritative follow-through.

## 3. Stop explicitly when governance or delivery evidence blocks completion

```bash
cargo run --bin boundline -- run --workspace <workspace> --goal "Fix the failing add test"
```

Expected behavior in blocked scenarios:
- If Canon blocks or awaits approval, Boundline stops explicitly and does not claim
  success.
- If no material workspace diff exists, Boundline stops explicitly.
- If validation evidence is missing or not credible, Boundline stops explicitly.

## 4. Keep explicit compatibility subordinate

```bash
cargo run --bin boundline -- run --workspace <workspace> --goal "Fix the failing add test" --compatibility
```

Expected behavior:
- Compatibility remains explicitly operator-chosen.
- Governed delivery on the native path does not become compatibility-owned by
  implication.

## 5. Validate the release story

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected behavior:
- Docs and assistant guidance describe the same `0.31.0` governed-delivery
  story.
- Modified or newly created Rust files remain above 95% coverage.
- Formatting and clippy complete cleanly.