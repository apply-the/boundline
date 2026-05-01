# Quickstart: Workflow Follow-Through

**Feature**: 019-workflow-follow-through  
**Date**: 2026-05-01

## Scenario 1: Continue A Named Workflow Through Review And Govern

```bash
cd /tmp/synod-workflow-follow-through
cargo run --bin synod -- workflow run governed-delivery --workspace .
cargo run --bin synod -- workflow resume --workspace .
cargo run --bin synod -- workflow inspect --workspace .
```

**Expected**:
- Synod progresses through review and govern when their bounded prerequisites are satisfied.
- If either phase cannot continue, Synod reports an explicit paused, blocked, or failed condition instead of a declaration-only blocker.
- `workflow inspect` preserves the same session and trace story as the direct session-native path.

## Scenario 2: Discover Available Named Workflows And Choose One Correctly

```bash
cd /tmp/synod-workflow-follow-through
cargo run --bin synod -- workflow list --workspace .
cargo run --bin synod -- workflow run governed-delivery --workspace .
cargo run --bin synod -- workflow status --workspace .
```

**Expected**:
- The workflow discovery surface exposes the available named workflows in the workspace.
- Each workflow entry provides enough summary and invocation guidance to support correct operator or assistant selection.
- Once a workflow is active, workflow-aware status exposes the workflow identity, active phase, routing, execution condition, and next action consistently.

## Scenario 3: Author A Review Or Govern Workflow From Shipped Guidance

```bash
cd /tmp/synod-workflow-follow-through
cat .synod/workflows.toml
cargo run --bin synod -- workflow list --workspace .
```

**Expected**:
- The shipped guidance explains how to author a valid workflow that includes review and govern.
- The documented example stays within the supported bounded model.
- The relationship between workflow commands, the direct session-native path, and the explicit compatibility path is clear from the documentation.

## Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo deny check licenses advisories bans sources
```

## Release Closeout

```bash
grep '^version =' Cargo.toml
git diff -- README.md ROADMAP.md CHANGELOG.md assistant/README.md docs/getting-started.md docs/configuration.md
```

**Expected**:
- The crate version is updated to `0.19.0` before implementation lands.
- README, docs, roadmap, changelog, and assistant guidance all describe the workflow follow-through slice coherently.