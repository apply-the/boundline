# Quickstart: Runtime Refoundation

**Feature**: 015-runtime-refoundation  
**Date**: 2026-04-29

## Scenario 1: Session-Native Path Is The Default

```bash
cd /tmp/runtime-refoundation-workspace
cargo init --lib runtime-refoundation-workspace

cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "fix the failing add test"
cargo run --bin boundline -- plan --flow bug-fix
cargo run --bin boundline -- run
cargo run --bin boundline -- inspect
```

**Expected**:
- Planning persists a bounded task draft derived from current workspace state.
- Execution uses the session-native route rather than requiring an execution profile.
- Inspect output shows explicit decisions, evidence, and terminal reasoning.

## Scenario 2: Unconfirmed Flow Proposal Blocks Silent Auto-Run

```bash
cd /tmp/runtime-refoundation-workspace
cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "fix the failing auth test"
cargo run --bin boundline -- plan
cargo run --bin boundline -- run
```

**Expected**:
- Planning proposes `bug-fix` with a visible rationale.
- Execution does not silently auto-confirm the proposal.
- The CLI explains how to confirm the flow or skip constraints before continuing.

## Scenario 3: Failure Evidence Remains Inspectable During Recovery

```bash
cd /tmp/runtime-refoundation-workspace
cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "repair the broken parser behavior"
cargo run --bin boundline -- plan --no-flow
cargo run --bin boundline -- run
cargo run --bin boundline -- status
cargo run --bin boundline -- inspect
```

**Expected**:
- If a bounded action fails verification, the failed decision remains visible.
- Recovery or replan behavior references the preserved failure evidence.
- Terminal output explains whether Boundline recovered, exhausted its limits, or stopped because no credible next action remained.

## Scenario 4: Compatibility Mode Remains Explicit

```bash
cd /tmp/runtime-refoundation-compat
mkdir -p .boundline
cat > .boundline/execution.json <<'EOF'
{
  "name": "compat-profile",
  "limits": { "max_steps": 5, "max_retries": 1 },
  "attempts": []
}
EOF

cargo run --bin boundline -- run
```

**Expected**:
- Boundline uses the compatibility path because declarative execution is the only available route.
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

cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "implement a bounded workspace summary"
cargo run --bin boundline -- plan --no-flow
```

**Expected**:
- Planning may cite Canon artifacts as bounded evidence inputs.
- Per-action runtime control remains Boundline-owned and does not require Canon to choose each next action.

## Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo deny check licenses advisories bans sources
```