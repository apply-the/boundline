# Research: Plan Execution Orchestration

## Decision 1: Topological Sort Implementation

**Decision**: Hand-rolled Kahn's algorithm using `std::collections`.

**Rationale**: Plans have ≤50 tasks, so a simple adjacency-list BFS implementation is sufficient. Avoids adding `petgraph` or similar dependency for a bounded problem. Kahn's algorithm is well-understood, deterministic, and produces the same order as plan-order tie-breaking.

**Alternatives considered**:
- `petgraph` crate: rejected — adds a new dependency for ≤50-node graphs
- DFS-based topological sort: rejected — Kahn's produces a natural BFS order closer to plan order

## Decision 2: Atomic Checkpoint Write

**Decision**: Manual temp-file + flush + sync + rename using `std::fs`.

**Rationale**: Consistent with existing Boundline file patterns (no tempfile crate dependency for this use case). The write sequence is: write to `<run-id>.json.tmp`, `flush()`, `sync_all()`, `fs::rename` over `<run-id>.json`. On failure, the previous valid checkpoint is preserved (rename is atomic on all target platforms).

**Alternatives considered**:
- `tempfile` crate: rejected — adds dependency for a simple atomic write pattern
- Append-only checkpoint log: rejected — resume needs latest state, not full history

## Decision 3: Session Linkage Pattern

**Decision**: Add `active_execution_run_id: Option<String>` to `ActiveSessionRecord` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.

**Rationale**: Minimal change, backward-compatible (new field with default). Existing sessions with no execution run will deserialize without error. `SessionStatusView` reads the linked checkpoint file to project execution fields.

**Alternatives considered**:
- New top-level session field group: rejected — unnecessary complexity for a single linkage field
- Separate execution-run registry file: rejected — session linkage is simpler and more discoverable

## Decision 4: Completion-Verification Integration (Spec 079)

**Decision**: Consume `CompletionVerificationState` and `CompletionVerificationProjection` through the existing `block_completion_closeout` path.

**Rationale**: Spec 079 already gates task closeout in `session_runtime_finalization.rs`. The execution orchestrator calls the same closeout path for each task. When verification is unavailable (079 not initialized), the orchestrator receives a blocked projection with `completion_verification_unavailable` and pauses execution.

## Decision 5: Plan Format for Task Dependencies

**Decision**: Extend the existing `PlannedTask` struct with `depends_on: Option<Vec<String>>` (serde default to empty).

**Rationale**: Minimal change to the accepted-plan schema. An empty or absent `depends_on` means no task-level prerequisite. The dependency graph is validated at plan-load time before any execution.
