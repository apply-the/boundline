# Research: Decision-Driven Orchestrator

## Decision 1: Add Explicit Action Selectors On Top Of The Existing Decision Model

**Decision**: Extend the current decision model with one explicit selector layer
for `read`, `search`, `modify`, `test`, `ask`, and `replan` instead of
replacing the existing decision record entirely.

**Rationale**: The current `Decision` object, persisted decision history, flow
policy linkage, and trace projection surfaces already exist. Layering explicit
selectors onto that model keeps the runtime change focused on execution control
while avoiding unnecessary churn in unrelated session and trace plumbing.

**Alternatives considered**:
- Replace `DecisionType` outright with a new selector-only model: rejected
  because it would force a broad rewrite of flow-policy, goal-plan hints, and
  existing follow-through surfaces before the runtime behavior change is proven.
- Keep the current decision model unchanged and infer selector wording only in
  output: rejected because selector choice would remain presentation-only rather
  than runtime-authoritative.

## Decision 2: Deepen The Existing Native Decision Loop Instead Of Reviving A Second Engine Path

**Decision**: Make the current native `DecisionLoop` in `session_runtime` the
authoritative bounded execution path for goal-plan work rather than routing 034
through the legacy static-step orchestrator engine.

**Rationale**: The native `DecisionLoop` is already the live path for
session-native goal-plan execution and already persists decision history. The
smallest credible 034 slice is to improve how that loop chooses and projects
actions, not to refactor execution ownership across multiple runtime models.

**Alternatives considered**:
- Move native execution back through the legacy orchestrator engine: rejected
  because it would widen scope and blur the session-native operating model.
- Keep the loop as-is and only enrich tests or traces: rejected because the
  roadmap requires decision state to control the next bounded action.

## Decision 3: Treat Ask As A Bounded Selector That Produces Explicit Clarification State

**Decision**: Model `ask` as an explicit selector that records why the loop no
longer has a credible engineering action and surfaces a clarification or
capture-style recovery path through existing follow-through surfaces.

**Rationale**: The roadmap requires `ask` to be a first-class next-action
selector, but Synod still needs bounded sequential behavior and an operator
visible stop condition. Treating `ask` as an explicit selector plus surfaced
clarification state meets that need without introducing an interactive runtime
conversation loop.

**Alternatives considered**:
- Skip `ask` and map every insufficient-evidence case to `replan`: rejected
  because it hides the operator-facing distinction between missing information
  and a bounded replanning step.
- Implement live multi-turn prompting inside `run`: rejected because it would
  widen scope into new interactive control flow instead of bounded delivery.

## Decision 4: Enrich Existing Decision Trace Events Instead Of Adding A Parallel Event Family

**Decision**: Keep the current `DecisionCreated`, `DecisionDispatched`,
`DecisionVerified`, `DecisionFailed`, and `DecisionRecovered` event types, but
enrich their payloads with selector kind, selector rationale, evidence basis,
verification intent, and recovery metadata.

**Rationale**: Existing inspect and output code already understands the current
decision event family. Enriching payloads preserves backward readability for
older traces and keeps the 034 implementation focused on runtime behavior plus
projection rather than on a second event taxonomy.

**Alternatives considered**:
- Add a brand new trace event family for selector choice and evidence
  evaluation: rejected because it would duplicate current decision semantics and
  enlarge inspect compatibility work.
- Persist selector data only in session state: rejected because inspect must be
  able to explain authoritative traces independently of the active session.

## Decision 5: Use Deterministic Selector Rules Derived From Existing Evidence And Context

**Decision**: Implement selector choice through deterministic rules derived from
current observation, context-pack evidence, latest decision outcome, changed
files, validation state, and bounded recovery state, without adding external
retrieval or semantic indexing dependencies.

**Rationale**: The roadmap change is about making decision state authoritative,
not about introducing a new search subsystem. Deterministic rules are inspectable,
bounded, and sufficient to prove the operating-model shift in this slice.

**Alternatives considered**:
- Add semantic search or indexing dependencies immediately: rejected because it
  widens scope beyond the minimal delivery value of 034.
- Keep selector choice implicit inside adapters: rejected because the
  constitution requires explicit visible intelligence and inspectable control
  flow.

## Decision 6: Preserve Explicit Compatibility Authority While Reusing Selector Vocabulary

**Decision**: Keep explicit compatibility follow-up clearly trace-authoritative,
but allow `inspect` and related read-side projections to reuse selector
terminology when compatibility traces contain the enriched decision payloads.

**Rationale**: Synod's product hierarchy already makes native execution primary.
034 should improve the shared explanation surface without letting compatibility
follow-up read like a second primary runtime.

**Alternatives considered**:
- Ignore compatibility traces entirely for selector projection: rejected because
  inspect surfaces still need coherent vocabulary when authoritative state comes
  from compatibility execution.
- Recenter the feature around compatibility routing: rejected because it would
  violate the roadmap's product hierarchy.