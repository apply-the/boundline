# Quickstart: Canon Governance Expansion

**Feature**: 017-canon-governance-expansion  
**Date**: 2026-04-29

## Scenario 1: Route A Verification Stage Through `security-assessment`

```bash
cd /tmp/synod-governed-security
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "fix the credential leak in the API handler"
cargo run --bin synod -- flow bug-fix --workspace .
cargo run --bin synod -- plan --workspace .
cargo run --bin synod -- run --workspace .
```

**Expected**:
- The `verify` stage can select `security-assessment` as the governed Canon mode.
- `run` reports the selected Canon mode, the governance condition, packet provenance, and the next suggested command.

## Scenario 2: Approval-Gated Security Analysis Refreshes Through Status

```bash
cd /tmp/synod-governed-security
cargo run --bin synod -- status --workspace .
cargo run --bin synod -- next --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- If the governed security packet is waiting for approval, `status` refreshes the Canon governance state before reporting it.
- `next` and `inspect` agree on the approval or blocked condition and the next command.

## Scenario 3: Unsupported Canon Mode Is Rejected Explicitly

```bash
cd /tmp/synod-governed-security-invalid
cargo run --bin synod -- run --workspace .
```

**Expected**:
- An unsupported Canon mode configuration does not pass through unchecked.
- The session reports an explicit governance-blocked outcome with a corrective next action.

## Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo deny check licenses advisories bans sources
```