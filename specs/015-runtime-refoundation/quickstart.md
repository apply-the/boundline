# Quickstart: Runtime Refoundation

**Feature**: 015-runtime-refoundation  
**Date**: 2026-04-29

## Scenario 1: Session-Native Path Is The Default

```bash
cd /tmp/runtime-refoundation-workspace
cargo init --lib runtime-refoundation-workspace

cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "fix the failing add test"
cargo run --bin synod -- plan --workspace . --flow bug-fix
cargo run --bin synod -- run --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- Planning persists a bounded task draft derived from current workspace state.
- Execution uses the session-native route rather than requiring an execution profile.
- Inspect output shows explicit decisions, evidence, and terminal reasoning.

## Scenario 2: Unconfirmed Flow Proposal Blocks Silent Auto-Run

```bash
cd /tmp/runtime-refoundation-workspace
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "fix the failing auth test"
cargo run --bin synod -- plan --workspace .
cargo run --bin synod -- run --workspace .
```

**Expected**:
- Planning proposes `bug-fix` with a visible rationale.
- Execution does not silently auto-confirm the proposal.
- The CLI explains how to confirm the flow or skip constraints before continuing.

## Scenario 3: Failure Evidence Remains Inspectable During Recovery

```bash
cd /tmp/runtime-refoundation-workspace
cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "repair the broken parser behavior"
cargo run --bin synod -- plan --workspace . --no-flow
cargo run --bin synod -- run --workspace .
cargo run --bin synod -- status --workspace .
cargo run --bin synod -- inspect --workspace .
```

**Expected**:
- If a bounded action fails verification, the failed decision remains visible.
- Recovery or replan behavior references the preserved failure evidence.
- Terminal output explains whether Synod recovered, exhausted its limits, or stopped because no credible next action remained.

## Scenario 4: Compatibility Mode Remains Explicit

```bash
cd /tmp/runtime-refoundation-compat
mkdir -p .synod
cat > .synod/execution.json <<'EOF'
{
  "name": "compat-profile",
  "limits": { "max_steps": 5, "max_retries": 1 },
  "attempts": []
}
EOF

cargo run --bin synod -- run --workspace .
```

**Expected**:
- Synod uses the compatibility path because declarative execution is the only available route.
- Status and inspect surfaces make the compatibility route explicit.

## Scenario 5: Canon Inputs Stay At Planning And Stage Boundaries

```bash
cd /tmp/runtime-refoundation-governed
mkdir -p .canon
cat > .canon/requirements.md <<'EOF'
# Governed Requirements

- keep the change bounded
- preserve auditability
EOF

cargo run --bin synod -- start --workspace .
cargo run --bin synod -- capture --workspace . --goal "implement a bounded workspace summary"
cargo run --bin synod -- plan --workspace . --no-flow
```

**Expected**:
- Planning may cite Canon artifacts as bounded evidence inputs.
- Per-action runtime control remains Synod-owned and does not require Canon to choose each next action.

## Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo deny check licenses advisories bans sources
```