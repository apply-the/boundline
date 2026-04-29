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
synod start --workspace .
synod capture --goal "fix the broken function in src/lib.rs"

# Plan from goal
synod plan

# Run the decision loop
synod run

# Inspect the decisions
synod inspect
```

**Expected**: `synod run` produces at least one decision object in the trace. Each
decision has type, target, rationale, expected_outcome, and evidence_inputs. The
session terminates in an explicit terminal state.

**Verify**:

```bash
# Check trace for decision objects
cat .synod/traces/*.json | jq '.events[] | select(.event_type == "decision_created")'

# Check terminal state
cat .synod/session.json | jq '.status'
```

## Test Scenario 2: Goal-Derived Planning (US2)

```bash
cd /tmp/test-workspace
synod start --workspace .
synod capture --goal "add input validation to the parse function"
synod plan
```

**Expected**: `synod plan` produces a GoalPlan with tasks derived from workspace
state. The plan references files actually present in the workspace.

**Verify**:

```bash
cat .synod/session.json | jq '.goal_plan'
# Should show tasks with targets matching real files
```

## Test Scenario 3: Flow Inference (US3)

```bash
# Bug-fix goal
synod start --workspace .
synod capture --goal "fix the failing test in auth.rs"
synod plan
# Expected: proposes bug-fix flow

# Change goal
synod start --workspace .
synod capture --goal "add a new validation layer to the API"
synod plan
# Expected: proposes change flow
```

## Test Scenario 4: Fixture Compatibility (US6)

```bash
# Existing fixture workflow still works
cd /tmp/test-workspace
cat > .synod/execution.json << 'EOF'
{
  "goal": "test fixture compat",
  "workspace_ref": ".",
  "attempts": [...]
}
EOF

synod run
# Expected: uses fixture path, same output as v0.12.0
```

## Test Scenario 5: Decision Verification Failure and Recovery (US1 edge case)

```bash
synod start --workspace .
synod capture --goal "fix the broken test"
synod plan
synod run
# If a verification fails, inspect the recovery decision
synod inspect
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
