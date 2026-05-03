# Quickstart: Session-Native Orchestrator

**Feature**: 013-session-native-orchestrator  
**Date**: 2026-04-29

## Test Scenario 1: Bounded Decision Loop (US1)

```bash
# Setup: workspace with a known goal
cd /tmp/test-workspace
cargo init --name test-project
echo 'fn broken() { panic!("fix me"); }' > src/lib.rs

# Start a session and capture a goal
boundline start --workspace .
boundline capture --goal "fix the broken function in src/lib.rs"

# Plan from goal
boundline plan

# Run the decision loop
boundline run

# Inspect the decisions
boundline inspect
```

**Expected**: `boundline run` produces at least one decision object in the trace. Each
decision has type, target, rationale, expected_outcome, and evidence_inputs. The
session terminates in an explicit terminal state.

**Verify**:

```bash
# Check trace for decision objects
cat .boundline/traces/*.json | jq '.events[] | select(.event_type == "decision_created")'

# Check terminal state
cat .boundline/session.json | jq '.status'
```

## Test Scenario 2: Goal-Derived Planning (US2)

```bash
cd /tmp/test-workspace
boundline start --workspace .
boundline capture --goal "add input validation to the parse function"
boundline plan
```

**Expected**: `boundline plan` produces a GoalPlan with tasks derived from workspace
state. The plan references files actually present in the workspace.

**Verify**:

```bash
cat .boundline/session.json | jq '.goal_plan'
# Should show tasks with targets matching real files
```

## Test Scenario 3: Flow Inference (US3)

```bash
# Bug-fix goal
boundline start --workspace .
boundline capture --goal "fix the failing test in auth.rs"
boundline plan
# Expected: proposes bug-fix flow

# Change goal
boundline start --workspace .
boundline capture --goal "add a new validation layer to the API"
boundline plan
# Expected: proposes change flow
```

## Test Scenario 4: Fixture Compatibility (US6)

```bash
# Existing fixture workflow still works
cd /tmp/test-workspace
cat > .boundline/execution.json << 'EOF'
{
  "goal": "test fixture compat",
  "workspace_ref": ".",
  "attempts": [...]
}
EOF

boundline run
# Expected: uses fixture path, same output as v0.12.0
```

## Test Scenario 5: Decision Verification Failure and Recovery (US1 edge case)

```bash
boundline start --workspace .
boundline capture --goal "fix the broken test"
boundline plan
boundline run
# If a verification fails, inspect the recovery decision
boundline inspect
```

**Expected**: Failed verification produces a recovery decision (fix or replan)
that references the failure evidence. The trace shows the full
observe→decide→act→verify→(fail)→decide(recovery) sequence.

## Validation Commands

```bash
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
