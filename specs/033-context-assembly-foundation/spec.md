# Feature Specification: Context Assembly Foundation

**Feature Branch**: `033-context-assembly-foundation`  
**Created**: 2026-05-02  
**Status**: Draft  
**Input**: User description: "Build a first-class ContextBuilder that assembles bounded context packs from workspace signals, authored briefs, negotiated delivery state, recent traces, and reusable Canon artifacts; replace implicit planning input with explicit selective file, symbol, and evidence retrieval; surface context-pack provenance and narrowing summaries through plan, run, status, next, and inspect; ship the feature complete with version bump, docs, changelog, roadmap update, coverage, clippy, and fmt."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.
  When both a session-native workflow and a compatibility workflow exist, the spec MUST name which path is primary and keep compatibility behavior explicit rather than implicit.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Build A Bounded Context Pack Before Planning (Priority: P1)

An operator can record a goal and have Boundline derive a bounded context pack
before planning so the resulting tasks point to explicit workspace, authored,
trace, and governed evidence instead of relying on ambient repository state.

**Why this priority**: Without a first-class context pack, Boundline still plans
mostly from keywords and coarse workspace signals, which makes the runtime look
capable but leaves the planner effectively blind.

**Independent Test**: Capture a representative goal with authored input and
existing workspace evidence, run planning, and verify the resulting plan names
the bounded context inputs, narrowed targets, and evidence provenance required
for the planned tasks.

**Acceptance Scenarios**:

1. **Given** a workspace with authored brief input, recent traces, and source
  files related to the goal, **When** the operator runs `goal` followed by
  `plan`, **Then** Boundline creates one bounded context pack that records the
  relevant files, the evidence sources used to select them, and the summary
  that explains why those inputs matter to the planned work.
2. **Given** a workspace with reusable Canon artifacts that match the current
  delivery goal, **When** planning runs, **Then** the context pack includes
  those governed artifacts as live evidence for planning rather than treating
  them only as end-of-stage output.

---

### User Story 2 - Inspect Context Narrowing On The Primary Boundline Path (Priority: P2)

An operator can inspect the primary session-native path and understand which
files, symbols, traces, authored inputs, and Canon artifacts were selected for
the current bounded task without reconstructing that story from raw traces or
filesystem guesses.

**Why this priority**: The context pack only becomes operationally useful if it
is visible on the same plan, run, status, next, and inspect surfaces that the
operator already uses to understand bounded execution.

**Independent Test**: Plan and run a representative task, then verify that the
operator can recover context provenance, narrowing rationale, and primary input
references from the standard CLI surfaces without opening the full trace file.

**Acceptance Scenarios**:

1. **Given** a confirmed goal plan with a bounded context pack, **When** the
  operator asks for `status`, `next`, or `inspect`, **Then** the surfaced
  output includes context-pack summary and provenance cues that explain the
  currently authoritative planning inputs.
2. **Given** an explicit compatibility follow-up path, **When** the operator
  inspects that path, **Then** Boundline keeps compatibility authority explicit
  while still projecting the same context-pack vocabulary used on the primary
  session-native path.

---

### User Story 3 - Stop Explicitly When Credible Context Cannot Be Built (Priority: P3)

An operator sees an explicit non-success planning or follow-through outcome
when Boundline cannot build a credible bounded context pack, instead of receiving a
plan that silently falls back to broad repository guesses.

**Why this priority**: Context assembly is only trustworthy if failure to build
credible context is visible and blocks planning rather than degrading silently.

**Independent Test**: Use a workspace where the goal has insufficient relevant
files or where selected evidence conflicts with the current goal, then verify
that planning or follow-through stops with explicit context credibility
guidance and inspectable failure evidence.

**Acceptance Scenarios**:

1. **Given** a recorded goal whose workspace has no credible relevant files,
  traces, authored evidence, or Canon artifacts for the bounded task,
  **When** planning runs, **Then** Boundline stops explicitly with a non-success
  explanation that the context pack is not credible enough for planning.
2. **Given** a context pack whose selected evidence becomes stale or conflicts
  with the later bounded task state, **When** the operator runs `next` or
  `inspect`, **Then** Boundline surfaces that the current context guidance is no
  longer credible and points to the bounded recovery action rather than hiding
  the mismatch.

---

### User Story 4 - Ship Context Assembly As 0.33.0 (Priority: P4)

A maintainer can ship `0.33.0` with the new context-assembly story reflected in
runtime behavior, roadmap, docs, version metadata, changelog, and validation
evidence, without leaving old roadmap items or stale release guidance behind.

**Why this priority**: The feature is incomplete if the runtime changes land
but the product narrative, versioned artifacts, and release validation do not
describe or verify the same operating model.

**Independent Test**: Follow the updated docs on a representative workspace,
run the release validation suite, and confirm the version bump, roadmap,
changelog, docs, coverage, clippy, and formatting all align with `0.33.0`.

**Acceptance Scenarios**:

1. **Given** the `0.33.0` release artifacts, **When** a maintainer follows the
  documented session-native path, **Then** the runtime, roadmap, docs, and
  changelog all describe bounded context assembly as part of the primary Boundline
  execution model.
2. **Given** modified or newly created Rust files for this slice, **When** the
  maintainer runs the release validation suite, **Then** touched Rust coverage
  remains above 95%, clippy is clean for the slice, and formatting completes
  successfully.

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when context selection finds multiple plausible files but no
  bounded justification for narrowing them to the current goal?
- How does the system handle a stale context pack whose trace or Canon inputs no
  longer match the current bounded task?
- How does the system surface the primary session-native route versus any
  explicit compatibility route when both project context-pack summaries?
- What happens when a workspace has relevant authored brief material but no code
  files yet, or governed packet refs exist without reusable document bodies?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST build one explicit bounded context pack during goal
  planning on the primary session-native path before confirming a goal plan.
- **FR-002**: System MUST derive the context pack from bounded workspace
  signals, authored brief inputs, negotiated delivery state, recent trace
  evidence, and reusable Canon artifacts when those inputs are available.
- **FR-003**: System MUST record the provenance and narrowing rationale for each
  context-pack input so an operator can tell why a file, symbol, trace, or
  governed artifact was selected.
- **FR-004**: System MUST attach context-pack state to the goal plan and keep
  it available to later plan, run, status, next, and inspect surfaces.
- **FR-005**: System MUST keep context-pack vocabulary aligned across the
  primary session-native path and any explicit compatibility follow-up surfaces
  while preserving route authority.
- **FR-006**: System MUST stop planning explicitly when no credible bounded
  context pack can be built for the requested goal.
- **FR-007**: System MUST surface explicit non-success guidance when the latest
  context pack is stale, contradictory, or insufficient for the next bounded
  action.
- **FR-008**: System MUST treat reusable Canon packet and artifact evidence as
  live planning input when it is relevant to the current bounded task.
- **FR-009**: System MUST include contract, integration, and unit coverage for
  context-pack creation, projection, and non-credible failure paths.
- **FR-010**: System MUST include an explicit task for the `0.33.0` version
  bump across versioned release surfaces.
- **FR-011**: System MUST include an explicit task for updating all impacted
  docs, the roadmap, and the changelog for the release.
- **FR-012**: System MUST refresh focused validation for modified or newly
  created Rust files, keep touched-Rust coverage above 95%, resolve clippy
  issues introduced by the slice, and finish with repository formatting
  applied.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: bounded context-pack assembly for goal planning; provenance and
  narrowing summaries on plan, run, status, next, and inspect; Canon packet and
  trace evidence reuse as planning input; explicit context-credibility failure
  handling; `0.33.0` release closeout including version bump, docs, roadmap,
  changelog, coverage, clippy, and formatting.
- **Out of Scope**: a full decision-driven runtime refoundation; dynamic flow
  inference replacement beyond consuming the new context pack; background
  memory rewriting; unbounded repository retrieval; GUI work; provider-routing
  refactors; Canon mode-surface expansion beyond what current bounded stages
  already model.

### Key Entities *(include if feature involves data)*

- **Context Pack**: the bounded planning input bundle for one goal, including a
  stable identifier, summary, credibility state, narrowed file references,
  narrowed symbol or evidence references, Canon inputs, and provenance records.
- **Context Input**: one selected planning input such as a file, symbol,
  authored brief fragment, trace event, or Canon artifact, including its source
  kind, selection rationale, and bounded relevance to the goal.
- **Context Credibility State**: the explicit status that says whether a
  context pack is credible, blocked, stale, or insufficient for planning or
  follow-through.
- **Context Projection**: the operator-facing summary of context-pack content
  and provenance surfaced through plan, run, status, next, and inspect.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative session-native planning runs, operators can see
  the bounded context inputs and why they were selected in under 2 minutes from
  standard Boundline output.
- **SC-002**: 100% of representative context-assembly validation scenarios end
  in an explicit credible or non-credible state rather than silently planning
  from ambient repository guesses.
- **SC-003**: In representative trace and inspect scenarios, operators can
  identify the files, authored inputs, traces, and Canon artifacts that shaped
  the current bounded task without reading the raw persisted trace.
- **SC-004**: All modified or newly created Rust files in this slice finish the
  release validation suite above 95% line coverage with clean formatting and
  lint results.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators continue to use the session-native Boundline path as the primary
  delivery surface and rely on explicit compatibility only when they opt in.
- The current repository shape and bounded execution model remain authoritative;
  this feature may improve planning inputs but does not replace the execution
  engine or introduce parallel runtime branches.
- Existing authored brief ingestion, trace persistence, workflow projection, and
  Canon packet reuse surfaces are available and can be extended without adding a
  new top-level runtime.
- The `0.33.0` release may update docs, roadmap, assistant guidance, version
  metadata, and changelog as part of the same delivered macrofeature.
