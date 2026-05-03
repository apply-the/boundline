# Quickstart: Session-Native Workflow Layer

**Feature**: 018-workflow-layer  
**Date**: 2026-04-30

## Scenario 1: Run A Named Workflow Through The Session-Native Route

```bash
cd /tmp/boundline-workflow-layer
cargo run --bin boundline -- workflow run default --workspace .
```

**Expected**:
- Boundline validates the named workflow before execution begins.
- The workflow starts through the same session-native route already used by direct delivery commands.
- Output exposes the active workflow name, the current phase, `routing`, `execution_condition`, and the next suggested command.

## Scenario 2: Resume A Paused Workflow And Inspect The Current Phase

```bash
cd /tmp/boundline-workflow-layer
cargo run --bin boundline -- workflow status --workspace .
cargo run --bin boundline -- workflow resume --workspace .
cargo run --bin boundline -- workflow inspect --workspace .
```

**Expected**:
- `workflow status` reports the active workflow, current phase, and why it is paused or waiting.
- `workflow resume` continues from persisted workflow progress instead of replaying already satisfied phases.
- `workflow inspect` agrees with the session trace story and surfaces the same next action.

## Scenario 3: Reject An Invalid Workflow Definition Explicitly

```bash
cd /tmp/boundline-workflow-layer-invalid
cargo run --bin boundline -- workflow run invalid-flow --workspace .
```

**Expected**:
- Unsupported workflow semantics are rejected before hidden execution begins.
- Boundline reports an explicit blocked condition and a corrective next action.
- The existing direct session-native commands remain available even though the named workflow was invalid.

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
git diff -- README.md ROADMAP.md CHANGELOG.md assistant/ docs/
```

**Expected**:
- The crate version is updated to `0.18.0` before implementation lands.
- Docs, roadmap, changelog, and assistant assets reflect the workflow-layer slice coherently.