# Research: Governed Stage Depth

**Feature**: 020-governed-stage-depth  
**Date**: 2026-05-01

## R1: Reuse the existing stage-boundary governance runtime instead of introducing a new governed execution engine

**Decision**: Expand the credible governed-stage story by exercising the existing session-native governance runtime at `bug-fix:investigate` before the already-governed verify path.

**Rationale**: The repository already models supported stage and mode mappings, packet reuse, approval refresh, and session projection. The next delivery gain comes from making that earlier-stage path explicit and well-tested, not from adding a second orchestration surface.

**Alternatives Considered**:
- Introduce a workflow-only or Canon-owned governed execution runner: rejected because it would duplicate control flow and violate the session-native ownership boundary.
- Expand every supported governed stage in one release: rejected because it widens the slice before one earlier-stage path is proven end to end.

## R2: Use `bug-fix:investigate` as the first deeper governed stage

**Decision**: Scope the first deeper governed-stage slice to `bug-fix:investigate`, keeping later governed `verify` behavior in scope only as the downstream comparison and reuse target.

**Rationale**: `bug-fix:investigate` already has bounded mode support and sits early enough in the flow to prove that governance can happen before verify without changing Boundline's execution model.

**Alternatives Considered**:
- Start with `implement`: rejected because it skips the earliest meaningful proof that governance can shape the task before implementation begins.
- Start with `change` or `delivery`: rejected because the current bug-fix fixtures and stage expectations provide the smallest credible slice.

## R3: Treat packet reuse as bounded lineage, not cross-stage hidden state

**Decision**: Preserve packet reuse through explicit packet reference, readiness, upstream stage key, and binding reason only.

**Rationale**: The repository already has `PacketReuseBinding` and task-context storage for bounded lineage. Extending that visible story is consistent with the constitution and avoids leaking the full Canon artifact tree into Boundline's core state.

**Alternatives Considered**:
- Hide reuse inside runtime-only logic: rejected because operators could not explain why a downstream governed stage reused prior evidence.
- Expose the full `.canon/` artifact tree: rejected because it widens the public contract and over-couples Boundline to Canon internals.

## R4: Refresh approval and packet-readiness state on later commands before allowing progression

**Decision**: Keep approval refresh and packet-readiness refresh on explicit later commands such as `status`, `run`, `next`, workflow status, and workflow resume.

**Rationale**: This preserves explicit bounded progression and matches the current `refresh_governance_state` behavior. Operators keep one coherent session story without hidden background polling.

**Alternatives Considered**:
- Background polling for approval updates: rejected because it violates bounded sequential execution and weakens inspectability.
- Refresh only on `run`: rejected because `status`, `next`, and workflow-aware surfaces also need current guidance.

## R5: Close the slice as 0.20.0 with docs, coverage refresh, clippy cleanup, and formatting

**Decision**: Encode version bump, documentation updates, changelog work, coverage refresh for modified Rust files, clippy cleanup, and `cargo fmt` in the implementation tasks.

**Rationale**: This repository treats each roadmap slice as a versioned delivery unit. The governed-stage depth story is incomplete if runtime behavior, examples, and release artifacts drift apart.

**Alternatives Considered**:
- Defer docs and release hygiene until after implementation: rejected because the user explicitly requested release-aligned completion.
- Limit validation to targeted tests only: rejected because the slice changes operator-facing governance behavior and should finish with repository-standard validation gates.