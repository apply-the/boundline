# Feature Specification: Canon-Grounded Reasoning And Structured Memory

**Feature Branch**: `036-canon-grounded-memory`  
**Created**: 2026-05-03  
**Status**: Draft  
**Input**: User description: "Implement Spec 036 Canon-Grounded Reasoning And Structured Memory: align Boundline with Canon 0.39.0 as the stable governance release, treat Canon packets, governed artifacts, artifact summaries, and capability signals as live planning and decision inputs instead of stage-end output only, add durable bounded summarization and context compaction so long-running sessions can carry forward important evidence across loops without replaying the whole workspace, preserve bounded operator control and inspectability on the existing native session path, and ship the feature complete with version bump, docs, changelog, cargo fmt, cargo clippy, and modified Rust file coverage above 95 percent."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Plan With Canon-Grounded Context (Priority: P1)

An operator can run the primary session-native path and have Boundline treat current
Canon packets, governed artifacts, artifact summaries, and capability signals as
live planning evidence rather than as downstream governance residue.

**Why this priority**: This is the operating-model change named by Spec 036. If
planning still treats Canon as a side channel, the feature has not changed how
Boundline reasons about bounded delivery.

**Independent Test**: Capture a bounded goal in a workspace that already
contains reusable Canon artifacts and capability signals, then run
`start -> capture -> plan`. The resulting proposal must reflect Canon-grounded
constraints, packet reuse, or capability-aware verification intent in a way that
changes the proposed plan compared with workspace-only reasoning.

**Acceptance Scenarios**:

1. **Given** a credible native session and reusable Canon artifacts that narrow
   the bounded change surface, **When** the operator runs `plan`, **Then**
   Boundline proposes a bounded plan whose rationale and verification strategy cite
   that Canon-grounded evidence rather than treating it as optional decoration.
2. **Given** a credible native session where Canon capability signals show that
   a downstream governed step can or cannot support a bounded follow-through,
   **When** the operator runs `plan`, **Then** Boundline shapes the proposal and
   next action around those capability limits instead of discovering them only
   after execution starts.

---

### User Story 2 - Carry Forward Compacted Canon Memory Across Loops (Priority: P2)

An operator can continue a long-running session without replaying the whole
workspace because Boundline carries forward a durable, compact reasoning memory of
the Canon-grounded evidence that still matters to the next bounded decision.

**Why this priority**: Canon-grounded reasoning is incomplete if the evidence is
only visible at the moment it is first read. The feature must keep important
bounded evidence alive across replanning, retries, and later loop iterations.

**Independent Test**: Run a representative native session that spans multiple
decisions, includes at least one replanning or retry moment, and verify that a
later decision can rely on compact persisted Canon-grounded summaries instead of
re-reading the same full workspace or governed artifact set.

**Acceptance Scenarios**:

1. **Given** a session that already consumed Canon packets and governed
   artifacts, **When** later loop iterations continue or replan, **Then** Boundline
   carries forward a compact reasoning memory that preserves the still-relevant
   constraints, packet lineage, and evidence headlines needed for the next
   bounded decision.
2. **Given** a long-running session where prior Canon-grounded evidence becomes
   stale, contradicted, or insufficient, **When** Boundline reaches a later planning
   or decision point, **Then** it records the memory boundary explicitly and
   stops, refreshes, or replans instead of silently treating stale compacted
   memory as authoritative.

---

### User Story 3 - Inspect Canon Influence And Bounded Stops (Priority: P3)

An operator can see exactly how Canon-grounded evidence influenced planning,
decision selection, and bounded stop conditions through the normal Boundline
read-side surfaces.

**Why this priority**: The feature adds more reasoning input, so inspectability
must rise with it. If developers cannot see why Canon changed a plan or blocked
execution, the feature becomes hidden intelligence instead of bounded delivery
control.

**Independent Test**: Run representative planning, execution, and failure
scenarios on the native path and verify that `run`, `status`, `next`, and
`inspect` show the active compacted Canon memory, the decisive Canon-grounded
evidence, and any explicit stop or refresh requirement.

**Acceptance Scenarios**:

1. **Given** a plan or decision that used Canon-grounded evidence, **When** the
   operator reads `status`, `next`, or `inspect`, **Then** Boundline surfaces the
   compacted Canon memory, the decisive evidence headline, and the packet or
   capability reason that changed the bounded execution path.
2. **Given** a session where Canon-grounded memory is missing, stale,
   contradictory, or insufficient for the next bounded action, **When** the
   operator attempts to continue, **Then** Boundline reports an explicit stop,
   refresh, or replanning requirement instead of falling back to opaque
   workspace-only reasoning.

---

### User Story 4 - Ship Canon-Grounded Reasoning As 0.36.0 (Priority: P4)

A maintainer can ship `0.36.0` with the Canon-grounded reasoning model and
structured memory behavior reflected consistently in the runtime, docs,
assistant guidance, roadmap, changelog, validation, and modified-file coverage.

**Why this priority**: The feature changes operator-visible reasoning behavior.
It is incomplete if the runtime lands without release closure, updated guidance,
and validation evidence.

**Independent Test**: Follow the updated docs on a representative workspace,
exercise Canon-grounded planning and follow-through, and run the release
validation suite to confirm the runtime, documentation, and quality gates align
with the shipped behavior.

**Acceptance Scenarios**:

1. **Given** the `0.36.0` release artifacts, **When** a maintainer follows the
   documented primary native path, **Then** the runtime, roadmap, docs, and
   assistant guidance all describe Canon-grounded reasoning plus durable compact
   memory as part of the same bounded execution model.
2. **Given** the modified or new Rust files for this slice, **When** the
   maintainer runs release validation, **Then** formatting succeeds, lint is
   clean, and every touched Rust file remains above 95% line coverage.

### Edge Cases

- What happens when reusable Canon packets or governed artifacts exist but point
  to a wider scope than the currently accepted bounded change?
- How does the system behave when Canon capability signals conflict with current
  workspace evidence, negotiated acceptance boundaries, or operator-selected
  flow constraints?
- What happens when compacted Canon-grounded memory omits details that a later
  step unexpectedly needs to continue credibly?
- How does the system surface Canon-grounded influence on the primary
  session-native route versus an explicit compatibility-governed follow-up that
  remains trace-authoritative?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST treat reusable Canon packets, governed artifacts,
  artifact summaries, and capability signals as bounded planning and decision
  inputs on the primary session-native path when they are relevant to the active
  goal.
- **FR-002**: System MUST preserve explicit provenance for Canon-grounded
  evidence so later planning, execution, and inspection can identify which
  packet, artifact bundle, or capability signal influenced the bounded result.
- **FR-003**: System MUST derive a compact Canon-grounded memory summary that
  carries forward only the still-relevant constraints, lineage, and evidence
  headlines needed for later bounded reasoning.
- **FR-004**: System MUST persist that compact Canon-grounded memory in the same
  authoritative session-native state story used by planning and decision
  follow-through.
- **FR-005**: System MUST use compact Canon-grounded memory during replanning,
  retries, and later decision selection instead of requiring full workspace or
  artifact replay when the compact memory remains credible.
- **FR-006**: System MUST detect when Canon-grounded memory is stale,
  contradicted, missing, or insufficient and MUST stop, refresh, or replan
  explicitly rather than silently treating that memory as authoritative.
- **FR-007**: System MUST keep Canon influence bounded by the captured goal,
  negotiated delivery packet, current acceptance boundary, and explicit execution
  limits instead of allowing Canon artifacts to widen scope opportunistically.
- **FR-008**: System MUST preserve bounded operator control by surfacing when
  Canon-grounded evidence changed a plan, constrained a decision, or blocked the
  next action.
- **FR-009**: System MUST expose the active compacted Canon memory, the decisive
  Canon-grounded evidence, and any required refresh or stop reason through the
  existing `plan`, `run`, `status`, `next`, and `inspect` surfaces plus
  persisted traces.
- **FR-010**: System MUST preserve explicit compatibility continuity when the
  latest authoritative follow-up state comes from a compatibility-governed run,
  while reusing the same Canon-grounded reasoning vocabulary where that trace
  contains it.
- **FR-011**: System MUST remain independently executable and testable even when
  Canon-governed evidence is absent by falling back to explicit bounded behavior
  instead of making external governance data a hidden prerequisite for all work.
- **FR-012**: System MUST include unit, integration, and contract validation for
  Canon-grounded planning influence, compact-memory persistence, stale-memory
  handling, and read-side projection.
- **FR-013**: System MUST include explicit release-closeout work for bumping the
  Boundline version to `0.36.0`, updating impacted docs and assistant guidance,
  refreshing the roadmap, and recording the release in the changelog.
- **FR-014**: System MUST finish with clean formatting, clean lint results, and
  line coverage above 95% for every modified or newly created Rust file in this
  slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: Canon-grounded planning and decision influence on the primary
  native path; durable compact reasoning memory across loops; explicit stale or
  contradictory Canon-memory handling; inspectable Canon-grounded read-side
  projection; release closeout for `0.36.0`.
- **Out of Scope**: a new standalone memory subsystem beyond bounded task and
  session scope; distributed or parallel execution; new UI surfaces; Canon-led
  takeover of Boundline control flow; review councils or generic voting; provider
  abstraction refoundation; unrelated Canon compatibility churn that does not
  change bounded reasoning behavior.

### Key Entities *(include if feature involves data)*

- **Canon Context Snapshot**: the bounded session-owned projection of the Canon
  packets, governed artifacts, artifact summaries, and capability signals that
  are currently relevant to planning or decision selection.
- **Compacted Reasoning Memory**: the durable summary Boundline carries across loops
  so later bounded steps can reuse Canon-grounded evidence without replaying the
  whole workspace or full artifact bundle.
- **Canon Influence Projection**: the operator-visible explanation of how
  Canon-grounded evidence changed a proposal, constrained a decision, or forced
  a bounded refresh, stop, or replan.
- **Memory Credibility State**: the explicit status that says whether compacted
  Canon-grounded memory is still credible, stale, contradicted, or insufficient
  for the next bounded action.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative native planning scenarios with relevant Canon
  evidence, operators can identify which Canon-grounded inputs changed the plan
  and why in under 2 minutes using standard Boundline output.
- **SC-002**: In representative long-running native sessions, 100% of later
  bounded decisions either reuse credible compacted Canon-grounded memory or stop
  explicitly with a refresh, replan, or insufficiency reason.
- **SC-003**: In representative inspection scenarios, developers can identify
  Canon packet lineage, decisive artifact summary, and compact-memory credibility
  state from `status`, `next`, or `inspect` without opening raw persisted JSON.
- **SC-004**: All modified or newly created Rust files in this slice complete
  the release validation suite above 95% line coverage with clean formatting and
  lint results.

## Assumptions

- Canon `0.39.0` is the stable governance release to align against for this
  slice, and Boundline can treat its current packet, artifact, and capability
  surfaces as stable for the near term.
- The primary operator path remains the session-native route; explicit
  compatibility-governed follow-up remains an opt-in or continuity path rather
  than the default execution owner.
- Existing session, goal-plan, task-context, decision, and trace surfaces can be
  extended to carry compact Canon-grounded memory without introducing a second
  persistence subsystem.
- Release closeout for `0.36.0` may update repository docs, assistant guidance,
  roadmap entries, version metadata, and changelog in the same delivered
  macrofeature.
