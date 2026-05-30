# Quickstart: Decision Continuity And Guided Follow-Through

**Feature**: 028-decision-followthrough  
**Date**: 2026-05-01

This walkthrough shows the intended operator story for `0.28.0`: run the
existing bounded workflow, reach a non-terminal or inspect-only follow-up, and
confirm that `status`, `next`, and `inspect` explain the next bounded action by
reusing persisted session and trace evidence.

## 1. Start a bounded session-native workflow

```bash
cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "Fix the failing add test"
cargo run --bin boundline -- plan
cargo run --bin boundline -- run
```

Expected behavior:
- The command workflow remains unchanged.
- If the run reaches retry, replanning, blocked governance, or another explicit
  follow-up state, Boundline persists enough continuity evidence to explain what
  should happen next.

## 2. Check guided follow-through on status and next

```bash
cargo run --bin boundline -- status
cargo run --bin boundline -- next
```

Expected behavior:
- `status` and `next` report one concrete next bounded action or one explicit
  stop condition.
- The output explains why that action is credible using persisted decision,
  validation, recovery, or governance evidence.
- The winning evidence source stays visible when it materially changes the
  recommended follow-up.

## 3. Reuse continuity after reload or inspect-only follow-up

```bash
cargo run --bin boundline -- status
cargo run --bin boundline -- inspect
```

Expected behavior:
- A reloaded status call keeps the same guided follow-through story when the
  native session remains authoritative.
- `inspect` preserves the same continuity explanation instead of showing only
  generic trace history.

## 4. Preserve explicit compatibility authority

```bash
cargo run --bin boundline -- run --goal "Fix the failing add test"
cargo run --bin boundline -- next
cargo run --bin boundline -- inspect
```

Expected behavior:
- When the latest authoritative follow-up comes from an explicit compatibility
  trace, `next` and `inspect` reuse that trace evidence to explain the next
  bounded action.
- The output keeps compatibility ownership explicit instead of pretending that a
  resumable native session exists.

## 5. Validate the release story

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected behavior:
- The `0.28.0` docs and assistant guidance match the runtime follow-through
  story.
- Validation and coverage complete cleanly for the touched Rust slice.