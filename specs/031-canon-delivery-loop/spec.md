# Feature Specification: Governed Delivery With Canon Inside The Loop

**Feature Branch**: `031-canon-delivery-loop`  
**Created**: 2026-05-02  
**Status**: Draft  
**Input**: User description: "Make the next Boundline feature deliver at least one real bounded code change through the primary session-native path while calling Canon through the governed adapter at change framing, approval-gated review, and verification evidence stages. Keep Boundline authoritative for orchestration, retries, routing, next-step choice, and workspace execution. Keep Canon authoritative for governed artifacts, approval state, provenance, and policy gates. Governed and non-governed runs must share the same session, trace, follow-through, and next_command story. The feature must stop explicitly when implementation produces no material workspace diff, no credible validation evidence, or Canon blocks or awaits approval. Include version bump, impacted docs and changelog, focused Rust coverage for modified files, cargo clippy, and cargo fmt."

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

### User Story 1 - Deliver A Governed Code Change (Priority: P1)

An operator can start from a workspace and a bounded goal, run Boundline on the
primary session-native route, and complete one governed delivery flow that uses
Canon for change framing, governed review, and verification evidence while
still ending in real code changes and validation in the same workspace.

**Why this priority**: The roadmap now lives or dies on whether Canon improves
real code delivery inside Boundline. If this slice cannot produce one complete
governed code change, the Boundline and Canon split remains product noise instead
of product value.

**Independent Test**: In a representative Rust workspace with a bounded failing
task and Canon governance configured, run the primary Boundline delivery flow and
verify that Boundline produces a material workspace diff, records Canon-governed
stage evidence, completes validation, and leaves one inspectable session and
trace story.

**Acceptance Scenarios**:

1. **Given** a writable workspace, a bounded goal, and Canon governance enabled
  for change framing, review, and verification, **When** the operator starts
  the primary Boundline delivery flow, **Then** Boundline obtains governed change
  framing before implementation, applies a real bounded code change, captures
  governed review and verification evidence, and reaches terminal completion on
  the same session-native route.
2. **Given** a governed delivery flow that reaches review or verification,
  **When** Canon returns reusable governed evidence for the current stage,
  **Then** Boundline persists that packet or evidence, uses it to continue bounded
  delivery, and keeps the resulting follow-through visible on the same `run`,
  `status`, `next`, and `inspect` surfaces.
3. **Given** a governed delivery flow that completes, **When** the operator
  inspects the workspace state and persisted follow-through, **Then** the same
  session, trace, governed evidence, and `next_command` story remain available
  without switching to a second product surface.

---

### User Story 2 - Stop Safely When Delivery Is Not Credible (Priority: P2)

An operator can trust Boundline to stop instead of pretending completion when
Canon blocks a stage, approval is still outstanding, implementation produces no
material workspace diff, or validation does not produce credible evidence.

**Why this priority**: A governed delivery story is worse than useless if it
can produce policy packets while still claiming success for non-delivery,
non-reusable output, or unapproved work.

**Independent Test**: Exercise governed runs that hit blocked Canon responses,
approval gates, no-diff implementation results, and failed or non-credible
validation, then verify that each run stops explicitly with persisted reason,
governance context, and a bounded next action.

**Acceptance Scenarios**:

1. **Given** a governed stage where Canon returns blocked or awaiting-approval
  status, **When** Boundline reaches that stage, **Then** Boundline stops the same
  delivery session in an explicit blocked or approval-gated state and does not
  continue implementation, review, or completion implicitly.
2. **Given** a governed delivery attempt whose implementation produces no
  material workspace diff or whose verification evidence is absent, stale, or
  contradicted by workspace results, **When** Boundline evaluates completion,
  **Then** it stops explicitly before terminal success and records the bounded
  repair or operator action still required.
3. **Given** a governed session that paused for approval or blocked governance,
  **When** the blocking condition changes and the operator resumes, **Then**
  Boundline continues from the same persisted session and trace story instead of
  creating an unrelated new run.

---

### User Story 3 - Keep One Follow-Through Model Across Governed And Non-Governed Runs (Priority: P3)

An operator can move between governed and non-governed delivery work without
learning a second product surface because `run`, `status`, `next`, and
`inspect` keep one shared follow-through model while making governance
ownership explicit.

**Why this priority**: Feature 031 only closes the roadmap if Canon becomes
useful inside the existing Boundline product, not beside it as a separate runtime
story.

**Independent Test**: Compare one governed native run, one non-governed native
run, and one explicit compatibility run, then verify that the same read-side
surfaces remain aligned while route ownership, governance state, and any
subordinate compatibility cues stay explicit.

**Acceptance Scenarios**:

1. **Given** one governed session and one non-governed session, **When** the
  operator uses `run`, `status`, `next`, or `inspect`, **Then** the same
  session, trace, follow-through, and `next_command` model is used with
  governance-specific cues layered in rather than split into a separate Canon
  workflow surface.
2. **Given** an explicit compatibility execution alongside governed native
  delivery, **When** the operator inspects follow-up state, **Then** the
  compatibility path remains clearly subordinate and explicit instead of
  becoming the implied route for governed delivery.

---

### User Story 4 - Ship Governed Delivery As 0.31.0 (Priority: P4)

A maintainer can ship `0.31.0` with runtime behavior, docs, assistant guidance,
roadmap, changelog, and validation evidence all describing the same story:
Boundline now delivers at least one real governed code flow with Canon inside the
loop.

**Why this priority**: If the runtime changes but version metadata,
documentation, assistant surfaces, and validation evidence still describe Canon
as a sidecar, the release will contradict the product claim that motivated this
feature.

**Independent Test**: Follow the updated governed delivery guidance on a
representative workspace, then run the release validation suite and confirm
that version metadata, docs, coverage, lint, and formatting align with the
shipped behavior.

**Acceptance Scenarios**:

1. **Given** the `0.31.0` release artifacts, **When** a maintainer follows the
  documented governed delivery path, **Then** the observed runtime behavior,
  Canon participation, and follow-through cues match the release narrative.
2. **Given** modified or newly created Rust sources for this slice, **When**
  the maintainer runs the release validation suite, **Then** formatting,
  clippy, focused tests, and refreshed coverage complete successfully and the
  modified Rust files remain above 95% coverage.

---

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when Canon returns a non-reusable governed packet at a stage
  Boundline expected to continue from?
- How does the system handle implementation that produces no material diff or
  touches files outside the bounded workspace target?
- How does the system stop when verification evidence is missing, stale, or
  contradicted by workspace results?
- How does the system surface the primary session-native governed route versus
  an explicit compatibility route after Canon has participated in the same
  workspace?
- What happens when approval arrives after the active session has already been
  paused or inspected and the operator resumes from persisted state?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST let the primary session-native delivery path invoke
  Canon-governed stages inside the same bounded delivery session so at least
  one complete flow can progress through goal, change framing,
  implementation, review, verification, and completion without splitting into a
  second product surface.
- **FR-002**: System MUST keep Boundline authoritative for orchestration, retries,
  routing, next-step choice, and workspace execution even when a stage is
  governed by Canon.
- **FR-003**: System MUST consume Canon through the machine-facing governance
  adapter contract and persist Canon outcomes as governed packets or evidence
  that later Boundline stages can inspect and reuse.
- **FR-004**: System MUST require successful governed change-framing evidence
  before native implementation begins on a governed delivery flow.
- **FR-005**: System MUST require both a material workspace diff and credible
  validation evidence before a governed delivery can be marked completed.
- **FR-006**: System MUST stop in an explicit blocked, awaiting approval,
  failed, or exhausted state when Canon blocks, Canon awaits approval, a
  governed packet is non-reusable, implementation yields no material workspace
  diff, or validation evidence is not credible.
- **FR-007**: System MUST preserve the same session, trace, follow-through, and
  `next_command` surfaces across governed and non-governed flows while making
  governance ownership, approval state, and latest governed packet or evidence
  inspectable.
- **FR-008**: System MUST allow a governed delivery session that paused for
  approval or blocked governance to resume from the same persisted session
  after the blocking condition changes.
- **FR-009**: System MUST reuse relevant governed packets or evidence from
  earlier governed stages in later governed stages when that material remains
  reusable and in scope.
- **FR-010**: System MUST keep explicit compatibility execution available only
  as a subordinate, explicit route and MUST NOT let compatibility become the
  implied path for governed delivery.
- **FR-011**: System MUST update version metadata, impacted docs, assistant
  guidance, roadmap, contributor guidance, and changelog together for the
  `0.31.0` release.
- **FR-012**: System MUST refresh focused validation for modified or newly
  created Rust files, keep modified-Rust coverage above 95%, resolve clippy
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

- **In Scope**: at least one complete governed code-delivery flow on the
  primary session-native route; Canon participation at change framing,
  review or approval, and verification evidence stages; shared
  session, trace, follow-through, and `next_command` surfaces; explicit
  stop conditions for blocked, approval-gated, no-diff, and no-evidence
  outcomes; `0.31.0` release closeout including version bump, docs,
  assistant guidance, changelog, coverage, clippy, and formatting.
- **Out of Scope**: turning Canon into an orchestrator or execution engine;
  widening governed delivery to every flow or every language in one slice;
  product-unification work planned for Feature 032; new remote services or
  daemons; replacing bounded execution with an unconstrained autonomous coding
  system.

### Key Entities *(include if feature involves data)*

- **Governed Delivery Session**: persisted session-native delivery state for a
  goal that may include governed stage outcomes, implementation progress,
  validation evidence, and terminal follow-through.
- **Governed Stage Packet**: Canon-sourced packet or evidence bundle attached
  to a stage, including readiness or approval posture and reuse eligibility for
  downstream stages.
- **Delivery Completion Gate**: the combined condition requiring governed stage
  success, material workspace diff, and credible validation evidence before
  Boundline can mark delivery complete.
- **Governance Continuity Cue**: surfaced state on `run`, `status`, `next`, and
  `inspect` that tells the operator who currently owns the next step, whether
  approval is pending, and which governed evidence is authoritative.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative Rust workspaces with Canon governance
  configured, at least one end-to-end governed flow reaches a real code diff,
  governed review or approval handling, verification evidence, and terminal
  completion on the primary session-native route without switching to a
  separate product surface.
- **SC-002**: In representative governed non-success scenarios, 100% of runs
  stop in an explicit blocked, awaiting approval, failed, or exhausted state
  before false completion and preserve enough session or trace evidence for a
  maintainer to understand the next action in under 5 minutes.
- **SC-003**: Operators can use the same `run`, `status`, `next`, and
  `inspect` surfaces to understand both governed and non-governed delivery
  state, including latest governed packet or approval posture, in under 2
  minutes per session.
- **SC-004**: Maintainers can validate and ship the `0.31.0` governed delivery
  story, including version alignment, modified-Rust coverage above 95%, clippy,
  and formatting, within the normal release workflow.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Canon will expose the machine-facing governance adapter surface defined by
  its current governance-adapter feature before Boundline closes this slice.
- One complete governed delivery flow in representative Rust workspaces is
  sufficient to prove the product model before widening governance coverage
  further.
- Existing `.boundline/session.json`, `.boundline/traces/`, and current follow-through
  surfaces remain the authoritative state model for this feature.
- Boundline's current bounded execution and validation mechanisms are sufficient to
  demonstrate real delivery value in this slice without introducing a broader
  autonomous execution architecture.
