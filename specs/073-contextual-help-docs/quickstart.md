# Quickstart: Contextual Help And Documentation Architecture (Boundline)

**Feature**: 073-contextual-help-docs
**Date**: 2026-06-06

## Prerequisites

- Rust 1.96.0 toolchain installed
- Boundline workspace built (`cargo build`)
- An isolated temporary workspace (NOT the Boundline repository root)

## Scenario 1: Uninitialized Workspace

```bash
WORKSPACE=$(mktemp -d)
cd "$WORKSPACE"
/path/to/target/debug/boundline help-next
```

Expected human output:
```
State: uninitialized
Next action: initialize workspace
Command: boundline init
Why: no .boundline/ directory found — initialization is required before any workflow
Docs: wiki/Getting-Started
```

Expected `--json`:
```bash
/path/to/target/debug/boundline help-next --json | jq '.state'
# "uninitialized"
```

## Scenario 2: Initialized, No Session

```bash
/path/to/target/debug/boundline init
/path/to/target/debug/boundline help-next
```

Expected: state=initialized, recommends `boundline goal`.

## Scenario 3: Active Session, Blocked Planning

1. Create a session with a goal and a plan that triggers a planning-analysis block.
2. Run `boundline help-next`.

Expected: state=blocked, shows the blocking finding, recommends `boundline plan`.

## Scenario 4: Healthy Session

1. Create a session with no blockers.
2. Run `boundline help-next`.

Expected: state=ready, no blockers found, recommends `boundline run`.

## Scenario 5: Multiple Issues With `--all`

1. Create a session with both a missing config key and a blocked planning gate.
2. Run `boundline help-next`.

Expected: shows the top blocking issue with "1 additional issue" count.
3. Run `boundline help-next --all`.

Expected: lists both issues ordered by priority.

## Scenario 6: Missing Link Map Key

1. Remove a link from `.boundline/help-links.toml`.
2. Run `boundline help-next`.

Expected: diagnostic still shown, docs link marked as "unavailable" with a non-blocking warning.

## Scenario 7: Structured Event

```bash
/path/to/target/debug/boundline help-next
/path/to/target/debug/boundline trace events | jq 'select(.event_type == "boundline.help_next.requested")'
```

Expected: one event with correct state, diagnostics count, and output format.

## Validation Commands

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --test unit
cargo test --test contract
cargo test --test integration
cargo test
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
scripts/common/coverage/intersect_patch_coverage.py
```
