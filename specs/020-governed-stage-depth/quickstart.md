# Quickstart: Governed Stage Depth

**Feature**: 020-governed-stage-depth  
**Date**: 2026-05-01

## Scenario 1: Govern Investigate Before Verify On The Session-Native Route

```bash
cd /tmp/synod-governed-stage-depth
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "Fix the failing add test"
cargo run --bin synod -- flow bug-fix --workspace .
cargo run --bin synod -- plan --workspace .
cargo run --bin synod -- run --workspace .
cargo run --bin synod -- status --workspace .
```

**Expected**:
- Synod can govern `bug-fix:investigate` on the same session-owned route before later verify work.
- If governance produces reusable evidence, the session preserves packet reference, readiness, and next action.
- If investigate governance blocks or waits for approval, `status` reports that state explicitly without advancing hidden work.

## Scenario 2: Refresh Approval And Resume Governed Progression

```bash
cd /tmp/synod-governed-stage-depth
cargo run --bin synod -- status --workspace .
cargo run --bin synod -- next --workspace .
cargo run --bin synod -- run --workspace .
```

**Expected**:
- Later commands refresh approval and packet-readiness state before a governed stage resumes.
- The operator sees whether the session is still waiting, can continue, or is blocked.
- The reported next command stays coherent across `status`, `next`, and `run`.

## Scenario 3: Project The Same Governed State Through A Named Workflow

```bash
cd /tmp/synod-governed-stage-depth
cargo run --bin synod -- workflow run governed-delivery --workspace . --goal "Fix the failing add test"
cargo run --bin synod -- workflow status --workspace .
cargo run --bin synod -- workflow inspect --workspace .
```

**Expected**:
- Workflow-aware surfaces preserve the same session-native governance route story.
- Governance packet provenance, readiness, and blocked or waiting conditions remain visible through workflow projection.
- The workflow layer never implies Canon-owned orchestration.

## Validation Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo test --test integration governance_
cargo test --test integration workflow_
cargo test --test contract governance_
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo nextest run --workspace --all-features
cargo deny check licenses advisories bans sources
```

## Release Closeout

```bash
grep '^version =' Cargo.toml
git diff -- README.md docs/getting-started.md docs/configuration.md assistant/README.md CONTRIBUTING.md ROADMAP.md CHANGELOG.md
```

**Expected**:
- The crate version is updated to `0.20.0` before implementation lands.
- Release docs and changelog describe the deeper governed bug-fix slice coherently.
- `lcov.info` is refreshed after the final modified Rust files and tests settle.