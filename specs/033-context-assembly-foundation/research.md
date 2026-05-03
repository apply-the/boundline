# Research: Context Assembly Foundation

## Decision 1: Add a first-class Context Pack to the goal-plan model

**Decision**: Add a dedicated context-pack structure to the goal-plan model instead of keeping context assembly implicit in coarse workspace signals and ad hoc evidence lists.

**Rationale**: The current planner already owns the bounded pre-execution draft, so it is the narrowest place to attach explicit context inputs, provenance, and credibility state without refactoring the whole runtime.

**Alternatives considered**:
- Build context only inside the decision loop: rejected because planning would stay blind and the operator would still not see why the plan was created.
- Store context only in traces: rejected because session and plan surfaces would still need to reconstruct the same state from raw events.

## Decision 2: Build context from existing bounded sources before adding new retrieval systems

**Decision**: Assemble the context pack from existing bounded sources first: workspace signals, selected file paths, authored briefs, negotiated delivery state, recent traces, and reusable Canon artifacts.

**Rationale**: These inputs already exist in Boundline and can be made explicit without introducing a new search service, hidden indexing layer, or long-term memory subsystem.

**Alternatives considered**:
- Add a new repository-wide retrieval/indexing subsystem first: rejected because it increases complexity before Boundline proves the value of explicit context on the current runtime path.
- Ignore Canon and traces until later: rejected because the roadmap calls out those sources as part of the bounded context story.

## Decision 3: Represent credibility explicitly and block non-credible planning

**Decision**: Context assembly must produce an explicit credibility state and planning must stop when the resulting pack is insufficient for bounded work.

**Rationale**: Silent fallback to ambient repository state would defeat the feature. A visible credibility decision preserves trust and makes failure a first-class path.

**Alternatives considered**:
- Always build a best-effort context pack and continue: rejected because it hides when planning is effectively guessing.
- Treat missing context as a warning only: rejected because the feature’s core value is credible planning, not advisory output.

## Decision 4: Surface context-pack summaries through existing CLI projections

**Decision**: Reuse existing `plan`, `run`, `status`, `next`, and `inspect` output surfaces for context-pack projection rather than adding a new command.

**Rationale**: The feature should change Boundline’s operating model, not add another surface the operator must remember.

**Alternatives considered**:
- Add a dedicated `context` command first: rejected because it fragments the operator story and delays integration with the primary path.
- Limit output to traces only: rejected because inspectability would remain too indirect for normal use.

## Decision 5: Keep compatibility vocabulary aligned without making compatibility primary

**Decision**: Explicit compatibility follow-up should reuse the same context-pack vocabulary when available, while preserving compatibility authority and without recentering the product on the compatibility path.

**Rationale**: The roadmap already fixes the product hierarchy. Context assembly must strengthen the primary session-native path without creating a second product story.

**Alternatives considered**:
- Ignore compatibility surfaces completely: rejected because inspect and follow-through still need aligned explanation.
- Move context assembly ownership to compatibility execution profiles: rejected because it would invert the current Boundline product model.

## Decision 6: Keep implementation dependency-free within the current crate

**Decision**: Use only current crate dependencies and Rust standard-library scanning/parsing heuristics for the first slice.

**Rationale**: The feature is about explicit bounded context, not sophisticated semantic indexing. Heavier retrieval or parser dependencies can wait for later macrofeatures if the simpler version proves insufficient.

**Alternatives considered**:
- Add external symbol-indexing or search crates immediately: rejected because the first slice only needs bounded, inspectable narrowing.
