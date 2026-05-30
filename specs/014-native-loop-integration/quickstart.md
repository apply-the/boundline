# Quickstart: Native Loop Integration

**Feature**: 014-native-loop-integration  
**Date**: 2026-04-29

## Scenario 1: Native Planning Persists GoalPlan And Flow Proposal

```bash
cd /tmp/native-loop-workspace
cargo init --lib native-loop-workspace

cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "fix the broken add function"
cargo run --bin boundline -- plan
```

**Expected**:
- The session record contains a non-empty `goal_plan`.
- Planning output explains whether a flow was confirmed, proposed, or skipped.
- The next recommended action is aligned with the session-native path.

## Scenario 2: Native Run Uses DecisionLoop When GoalPlan Exists

```bash
cd /tmp/native-loop-workspace
cargo run --bin boundline -- run
cargo run --bin boundline -- inspect
```

**Expected**:
- `run` uses the native route because a goal plan exists.
- The trace includes decision-oriented events.
- Inspect output shows persisted decisions and terminal reasoning.

## Scenario 3: Compatibility Path Remains Explicit

```bash
cd /tmp/fixture-compatible-workspace
mkdir -p .boundline
cat > .boundline/execution.json <<'EOF'
{
  "name": "fixture-profile",
  "limits": { "max_steps": 5, "max_retries": 1 },
  "attempts": []
}
EOF

cargo run --bin boundline -- run
```

**Expected**:
- Without a session-native goal plan, routing stays on the compatibility path.
- Output indicates declarative execution routing instead of decision-loop routing.

## Scenario 4: Unconfirmed Flow Proposal Blocks Silent Auto-Run

```bash
cd /tmp/native-loop-workspace
cargo run --bin boundline -- start
cargo run --bin boundline -- goal --goal "fix the failing auth test"
cargo run --bin boundline -- plan
cargo run --bin boundline -- run
```

**Expected**:
- If planning stored a proposed-but-unconfirmed flow, `run` does not silently treat it as confirmed.
- The CLI explains how to confirm the flow or proceed without one.

## Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --no-run --all-targets
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```
