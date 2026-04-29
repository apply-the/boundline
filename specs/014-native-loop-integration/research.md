# Research: Native Loop Integration

**Feature**: 014-native-loop-integration  
**Date**: 2026-04-29

## R1: Planned Session State Should Be GoalPlan-First

**Question**: What session state should represent a successfully planned native session?

**Decision**: Treat `goal_plan` as the primary planned artifact for the session-native path, while keeping `active_task` as a fixture-oriented or compatibility artifact.

**Rationale**: The session-native path is supposed to own planning semantics. If `Plan` continues to be represented only through `active_task`, the session remains structurally tied to the declarative runtime. Persisting `goal_plan` as the primary planning state makes route selection explicit and debuggable.

**Alternatives Considered**:
- Dual-write `goal_plan` and a fixture-shaped `active_task`: rejected because it preserves ambiguity about which runtime is authoritative.
- Keep `active_task` as the only planned state and treat `goal_plan` as advisory metadata: rejected because it fails to move the product onto the native path.

## R2: Lightweight Flow Confirmation Must Be Persisted Without Interactive Prompts

**Question**: How should flow confirmation work on a non-interactive CLI?

**Decision**: Use persisted session state plus explicit CLI options as the lightweight confirmation model: planning may store an inferred flow proposal, explicit `--flow` confirms a flow, `--no-flow` records deliberate unconstrained execution, and a previously selected flow remains authoritative.

**Rationale**: The CLI already works best as a resumable command surface rather than an interactive prompt shell. Persisting the inference outcome keeps the control flow explicit while avoiding hidden auto-confirmation.

**Alternatives Considered**:
- Interactive prompt during `plan`: rejected because it complicates scripting and breaks the current CLI execution model.
- Silent auto-confirmation of inferred flow: rejected by the review and constitution because it hides a material routing decision.

## R3: Run Routing Should Prefer Session-Native Work Over Fixture Fallback

**Question**: What predicate should decide whether `run` uses the native loop or the fixture compatibility path?

**Decision**: Route in this order: if a session has a persisted goal plan and no unresolved flow-confirmation block, use `DecisionLoop`; otherwise if the operator explicitly opts into declarative execution or only an execution profile exists, use fixture compatibility; otherwise surface an explicit remediation error.

**Rationale**: This makes the route choice inspectable, stable, and aligned with the product direction. Goal-plan presence is the authoritative signal that a session-native run is ready.

**Alternatives Considered**:
- Keep fixture as default unless a new flag opts into the native loop: rejected because it leaves the product centered on the legacy path.
- Auto-merge fixture and native loop inputs: rejected because it hides control flow and makes failures harder to debug.

## R4: Native Decision Execution Should Reuse Registries, Not Inline Filesystem Calls

**Question**: How should the decision loop execute real work without embedding filesystem and process logic directly in the loop?

**Decision**: Build native agent and tool registries for the session-native path and make the decision loop dispatch through those registries using `StepExecutionRequest`/`StepExecutionResult` as the bridge to `ToolResult`.

**Rationale**: The adapter harness already exists and is the right abstraction boundary. Removing direct file reads and process calls from the loop keeps action selection separate from action execution.

**Alternatives Considered**:
- Keep direct `std::fs` and `Command` calls inside `DecisionLoop`: rejected because it bypasses the runtime harness and keeps the loop synthetic.
- Depend on `fixture.rs` runtime assembly for all native runs: rejected because it requires declarative execution profiles and undermines the native path.

## R5: Persisted Decisions Must Be Written to Both Session State and Trace Surface

**Question**: Where should native-loop decisions live after execution?

**Decision**: Persist decisions back into the active session record and emit matching trace events so that `status`, `run`, and `inspect` all see the same execution story.

**Rationale**: Session state is the operator-facing continuity surface; traces are the detailed inspection surface. Writing only one of them leaves the runtime half-visible.

**Alternatives Considered**:
- Trace-only persistence: rejected because session commands cannot reliably answer routing and latest-decision questions.
- Session-only persistence: rejected because it weakens detailed auditability and existing inspect behavior.

## R6: Compatibility Scope Remains Explicit and Narrow

**Question**: What parts of fixture execution should remain after native routing is introduced?

**Decision**: Keep fixture execution only as the explicit path for declarative execution profiles and as a source of low-level mutation and validation primitives that may be reused by native adapters.

**Rationale**: This preserves backward compatibility without keeping fixture semantics at the center of the product.

**Alternatives Considered**:
- Delete fixture support now: rejected because existing compatibility tests and declarative workflows still rely on it.
- Continue synthesizing native plans from fixture plan builders: rejected because it preserves the architecture mismatch called out in the review.
