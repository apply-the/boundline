# Contract: Governed Delivery CLI Surfaces

**Feature**: 031-canon-delivery-loop  
**Date**: 2026-05-02

This contract captures the operator-visible behavior Boundline must preserve while
bringing Canon inside one real delivery loop.

## 1. Governed Native Run Success

### Preconditions

- A workspace can execute the primary session-native `bug-fix` flow.
- Canon governance is enabled for the relevant governed stages.
- Execution produces at least one changed file and validation passes.

### Contract

- `boundline run --goal <goal>` remains the primary entry.
- The run output must keep governance activity visible on the same operator
  surface through `governance_selected`, `governance_started`, and
  `governance_completed` events where applicable.
- Terminal success is allowed only if the session carries both material changed
  files and passed validation evidence.
- `boundline status`, `boundline next`, and `boundline inspect` must remain able to project
  latest governance state, latest changed files, latest validation status, and
  the same `next_command` story from the persisted session or trace.

## 2. Governed Approval Or Block Stop

### Preconditions

- Canon returns `awaiting_approval`, `blocked`, `failed`, or a non-reusable
  packet for the current governed stage.

### Contract

- The active session must not continue implementation or completion implicitly.
- The run output must surface the blocking governance state explicitly.
- `status` and `next` must remain usable and keep the current governance stage,
  governance state, and any approval posture inspectable.
- Resume must continue from the same persisted session after approval or other
  blocking conditions are cleared.

## 3. Delivery Completion Gate Failure

### Preconditions

- The native plan reaches terminal evaluation but either no material changed
  file exists or validation evidence is missing or not credible.

### Contract

- Boundline must not mark the task as succeeded.
- Boundline must record an explicit terminal reason that explains why the delivery
  claim is not credible.
- `status` and `inspect` must still surface the latest governance and delivery
  evidence so the operator can diagnose the stop condition from current CLI
  surfaces.

## 4. Explicit Compatibility Separation

### Preconditions

- The operator chooses the explicit compatibility route.

### Contract

- Compatibility routing remains explicit and subordinate.
- Canon-governed native delivery must not collapse back into compatibility by
  implication.
- Read-side surfaces must continue to distinguish route ownership while keeping
  one shared follow-through vocabulary.