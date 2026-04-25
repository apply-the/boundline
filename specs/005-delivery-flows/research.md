# Research: Delivery Flows (SDLC Backbone)

## Decision 1: Model flows as built-in static definitions

- Decision: Represent delivery flows through a built-in registry of static flow definitions inside the Rust domain layer, with each flow exposing an ordered list of stage identifiers and display labels.
- Rationale: The specification requires deterministic flows, minimal runtime complexity, and no external configuration. A built-in registry keeps flow behavior inspectable, versioned with the code, and aligned with the existing CLI-first runtime.
- Alternatives considered:
  - External configuration files: rejected because they introduce extra loading, validation, and backward-compatibility complexity for the first slice.
  - Dynamic flow generation from goals: rejected because it violates the spec's deterministic non-goals and would blur planning with flow selection.

## Decision 2: Persist optional flow state directly in the existing session record

- Decision: Extend the existing active session record with an optional flow-state object that stores the selected flow name, current stage identifier, current stage index, and total stage count.
- Rationale: Flow progression must survive across CLI invocations and remain visible in status, next, and inspect surfaces. Keeping flow state inside the existing session record preserves the single source of truth for active work and maintains backward compatibility by allowing sessions without flow state.
- Alternatives considered:
  - Separate flow-state file: rejected because it would split delivery state across multiple artifacts and increase the chance of invalid or mismatched runtime state.
  - Deriving stage state only from traces: rejected because status and next guidance need direct access to current flow state without replaying trace history.

## Decision 3: Keep plans flat and attach stage identity to steps

- Decision: Preserve the existing flat plan model and associate each step with a stage identifier through step input metadata produced by flow-aware planning.
- Rationale: The current orchestrator already executes a sequential flat list of steps with bounded retries and replans. Adding stage metadata to existing steps allows stage-aware execution and transitions without introducing a nested plan engine or replacing the current recovery semantics.
- Alternatives considered:
  - Nested stage containers inside the plan model: rejected because they would require broader engine and serialization changes than the minimal capability slice needs.
  - Inferring stage boundaries from step names alone: rejected because it would be brittle and would hide important execution logic in naming conventions instead of explicit data.

## Decision 4: Reuse the existing planner and runtime with flow-aware fixture plans

- Decision: Generate plans according to the selected built-in flow by choosing a flow-aware fixture plan builder inside the existing session runtime, rather than creating a second execution engine.
- Rationale: The feature is about delivery-flow structure, not new execution mechanics. Reusing the current planner and runtime preserves the existing bounded step loop while allowing deterministic stage-specific step sequences for `bug-fix`, `change`, and `delivery` over the repository-local fixture-backed execution slice.
- Alternatives considered:
  - A dedicated flow engine layered beside the current runtime: rejected because it duplicates orchestration responsibilities and increases maintenance cost.
  - Generating flows only at status time while leaving plans unchanged: rejected because stage-aware execution and recovery need the runtime to know which steps belong to which stage.

## Decision 5: Surface flow lifecycle changes in both session output and traces

- Decision: Record flow selection and stage transitions in persisted trace events and expose active flow and stage progress in status and next-command output.
- Rationale: The constitution requires inspectability and explicit decision visibility. Session JSON alone is not enough for debugging stage progression, while trace-only visibility is not enough for quick operator guidance.
- Alternatives considered:
  - Status output only: rejected because it would hide historical flow transitions from later inspection.
  - Trace events only: rejected because users need direct stage visibility without loading traces for routine progress checks.