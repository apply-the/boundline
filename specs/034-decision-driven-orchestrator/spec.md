# Feature Specification: Decision-Driven Orchestrator

**Feature Branch**: `034-decision-driven-orchestrator`  
**Created**: 2026-05-02  
**Status**: Draft  
**Input**: User description: "Implement Spec 034 Decision-Driven Orchestrator: make observe -> decide -> act -> verify the controlling runtime loop rather than a trace-friendly layer on top of static planning; evolve decisions into explicit next-action selectors like read, search, modify, test, ask, and replan; keep recovery, verification, and stop conditions authoritative from decision state so execution remains bounded and explainable; ship the feature complete with version bump, docs, changelog, roadmap update, cargo fmt, cargo clippy, and line coverage above 95% for modified Rust files."

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

### User Story 1 - Select The Next Bounded Action From Decision State (Priority: P1)

An operator can run the primary session-native path and have Synod choose the
next bounded engineering action from explicit decision state and current
evidence instead of walking a mostly static step order.

**Why this priority**: This is the operating-model change named by roadmap Spec
034. Without it, the runtime still behaves like a pre-shaped planner with a
decision trace layered on top.

**Independent Test**: Run a representative bounded task through the native path
and verify that the controlling loop selects concrete next actions such as
read, search, modify, test, ask, or replan from the current decision state and
evidence, rather than only replaying a static task sequence.

**Acceptance Scenarios**:

1. **Given** a bounded task whose failing area is not yet localized,
  **When** the operator runs native execution, **Then** Synod selects an
  evidence-gathering action such as read or search before attempting modify or
  test, and records why that action is the next bounded step.
2. **Given** a bounded task whose evidence already identifies a concrete code
  target, **When** execution advances, **Then** Synod can select modify or
  test directly from decision state and keep the selected action explainable
  from the same recorded evidence.

---

### User Story 2 - Inspect Decision-Driven Execution On Existing Surfaces (Priority: P2)

An operator can inspect the active session or authoritative trace and recover
the selected next action, the evidence that justified it, the verification or
stop condition guarding it, and the current recovery story without reading raw
trace payloads.

**Why this priority**: Decision-driven execution is only credible if the same
operator-facing surfaces already used for follow-through can explain why the
runtime chose one bounded action instead of another.

**Independent Test**: Execute a representative native run, then verify that
`run`, `status`, `next`, and `inspect` surface the current selected action,
decision rationale, evidence basis, and verification or stop condition in a way
that lets the operator continue without opening raw JSON.

**Acceptance Scenarios**:

1. **Given** an active native session with a current decision, **When** the
  operator asks for `status` or `next`, **Then** Synod shows the selected next
  action, why it was chosen, and what verification or recovery condition will
  control the next transition.
2. **Given** an authoritative compatibility follow-up trace, **When** the
  operator uses `inspect`, **Then** Synod keeps compatibility authority
  explicit while still reusing the same decision-driven vocabulary when the
  trace contains it.

---

### User Story 3 - Stop, Ask, Or Replan Explicitly When No Credible Action Exists (Priority: P3)

An operator sees explicit bounded non-success behavior when the current
decision state cannot justify another read, search, modify, or test step,
including clarification requests, replanning, retries, or terminal stop.

**Why this priority**: The roadmap requires recovery, verification, and stop
conditions to remain authoritative from decision state. A decision-driven loop
that cannot explain failure or recovery would be less trustworthy than the
current runtime.

**Independent Test**: Run representative tasks that hit missing evidence,
verification failure, or exhausted recovery options and verify that Synod asks,
replans, retries, or stops explicitly from decision state without silently
falling back to generic lifecycle wording.

**Acceptance Scenarios**:

1. **Given** a bounded task whose current evidence is not sufficient to justify
  another credible engineering action, **When** execution reaches that point,
  **Then** Synod selects ask or replan explicitly, records why the previous
  action was insufficient, and keeps the next bounded recovery action visible.
2. **Given** a bounded task that continues failing after allowed recovery work,
  **When** the configured execution limits or stop conditions are reached,
  **Then** Synod stops explicitly with a terminal reason derived from decision
  state instead of silently ending because a static task list ran out.

---

### User Story 4 - Ship The Decision-Driven Runtime As 0.34.0 (Priority: P4)

A maintainer can ship `0.34.0` with the new decision-driven execution model
reflected consistently in runtime behavior, roadmap, docs, changelog, version
metadata, and repository validation evidence.

**Why this priority**: The feature is incomplete if the runtime changes land
without the release surfaces, roadmap closure, and repository validation needed
to present one coherent product story.

**Independent Test**: Follow the updated docs on a representative workspace,
run the release validation suite, and verify that the version bump, roadmap,
docs, changelog, coverage, linting, and formatting all align with the shipped
decision-driven operating model.

**Acceptance Scenarios**:

1. **Given** the `0.34.0` release artifacts, **When** a maintainer follows the
  documented native path, **Then** the runtime, roadmap, and docs all describe
  explicit decision-driven execution as the primary Synod operating model.
2. **Given** modified or newly created Rust files for this slice, **When** the
  maintainer runs release validation, **Then** those files remain above 95%
  line coverage, lint issues introduced by the slice are resolved, and
  formatting completes successfully.

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when no credible decision action remains but execution has not
  yet reached a terminal task outcome?
- How does the system handle a verification result that invalidates the current
  decision rationale and requires a different action family than the previous
  step?
- How does the system surface decision-driven state on the primary
  session-native path versus an authoritative explicit compatibility trace that
  may contain only partial decision vocabulary?
- What happens when the correct bounded next step is to ask for clarification
  rather than reading, modifying, or testing code?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST make `observe -> decide -> act -> verify` the
  controlling native execution loop for bounded engineering work instead of
  treating decisions as a trace-friendly layer on top of mostly static step
  order.
- **FR-002**: System MUST represent each bounded next action as an explicit
  selector such as read, search, modify, test, ask, or replan, including the
  action target, rationale, and intended outcome.
- **FR-003**: System MUST choose the next bounded action from current decision
  state and current evidence rather than only from pre-shaped task order.
- **FR-004**: System MUST persist enough decision state for later execution and
  operator follow-through to explain the active action, the evidence basis for
  it, and the verification or stop condition that now controls progress.
- **FR-005**: System MUST keep recovery, verification, and terminal outcomes
  authoritative from decision state, including retry, ask, replan, failure,
  exhaustion, and success paths.
- **FR-006**: System MUST stop explicitly when no credible bounded next action
  can be selected within configured limits.
- **FR-007**: System MUST preserve explicit compatibility authority when the
  latest authoritative follow-up state comes from a compatibility trace, while
  reusing decision-driven vocabulary on inspect surfaces when the trace
  contains it.
- **FR-008**: System MUST allow the active bounded action family to shift among
  evidence gathering, code change, verification, clarification, and replanning
  as new evidence arrives, without losing acceptance-boundary or flow
  visibility.
- **FR-009**: System MUST surface decision rationale, evidence basis, selected
  action, verification intent, recovery state, and terminal reason through the
  existing operator-facing plan, run, status, next, and inspect surfaces plus
  persisted traces.
- **FR-010**: System MUST include contract, integration, and unit validation
  for decision selection, recovery handling, ask or replan behavior, and output
  projection.
- **FR-011**: System MUST include explicit release-closeout work for the
  version bump, roadmap closure, impacted docs, assistant guidance, and
  changelog.
- **FR-012**: System MUST finish with repository formatting, lint cleanliness,
  and line coverage above 95% for modified or newly created Rust files in this
  slice.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: decision-driven bounded execution on the primary native path;
  explicit next-action selector vocabulary; decision-authoritative recovery,
  verification, and stop conditions; projection of decision-driven state across
  existing CLI and trace surfaces; release closeout for `0.34.0`.
- **Out of Scope**: dynamic planning and flow inference beyond what is required
  to let decision state drive the next bounded action; new long-term memory
  systems; Canon-grounded reasoning beyond existing bounded reuse surfaces;
  distributed or parallel execution; new UI work; provider abstraction
  refoundation.

### Key Entities *(include if feature involves data)*

- **Decision Action Selector**: the explicit bounded next-action record for one
  loop iteration, including action family, target, rationale, expected outcome,
  verification intent, and any recovery linkage.
- **Decision State**: the persisted execution state that ties observations,
  evidence, selected action, action result, verification result, recovery
  state, and terminal evaluation together for bounded follow-through.
- **Decision Evidence Snapshot**: the bounded evidence set that justified the
  currently selected action, including the inputs used to support or reject
  candidate next actions.
- **Decision Projection**: the operator-facing summary surfaced through plan,
  run, status, next, and inspect so developers can understand what the runtime
  is about to do, why, and what would cause the next transition or stop.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative bounded native runs, operators can identify
  the currently selected next action, why it was chosen, and what verification
  or stop condition governs it in under 2 minutes from standard Synod output.
- **SC-002**: 100% of representative decision-driven execution runs end each
  loop iteration with either one explicit selected action or one explicit ask,
  replan, or terminal stop state rather than falling back to hidden static-step
  progression.
- **SC-003**: In representative non-success runs, developers can identify the
  failure, retry, replan, or clarification path from `status`, `next`, or
  `inspect` without reading raw persisted trace JSON.
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

- The primary operator path remains session-native, and explicit compatibility
  execution remains a subordinate, opt-in route.
- Existing session, goal-plan, decision, trace, and CLI projection surfaces can
  be extended without introducing a second execution runtime.
- Sequential bounded execution remains authoritative for this slice; parallel or
  distributed orchestration stays out of scope.
- Release closeout for `0.34.0` may update repository docs, assistant guidance,
  roadmap entries, version metadata, and changelog in the same delivered
  macrofeature.
