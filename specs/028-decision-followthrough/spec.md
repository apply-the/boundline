# Feature Specification: Decision Continuity And Guided Follow-Through

**Feature Branch**: `028-decision-followthrough`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Deepen status/next/inspect follow-through and decision continuity while using persisted session plus trace evidence to guide the next bounded action without expanding the control plane. Include release closeout tasks for version bump, impacted docs and changelog, coverage for modified Rust files, clippy cleanup, and cargo fmt."

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

### User Story 1 - Explain The Next Bounded Action (Priority: P1)

An operator can use `status`, `next`, or `inspect` to understand not only what
Boundline wants to do next, but why that next step is now the credible bounded
action given the latest decision, recovery, validation, or governance evidence.

**Why this priority**: The smallest valuable follow-through improvement is to
stop forcing operators to infer intent from raw state names when the real reason
for the next bounded action already exists in recent session or trace evidence.

**Independent Test**: Run a representative native or explicit compatibility flow
that reaches a retry, replan, blocked, inspect-only, or waiting state, then
verify that `status`, `next`, and `inspect` explain the next bounded action in
terms of persisted evidence rather than only generic lifecycle labels.

**Acceptance Scenarios**:

1. **Given** a bounded delivery task whose latest decision triggered retry,
  replanning, or another explicit follow-up path, **When** the operator runs
  `boundline status` or `boundline next`, **Then** Boundline reports the next bounded
  action together with the evidence that made that action credible.
2. **Given** a task paused by governance, blocked evidence, or inspect-only
  compatibility follow-up, **When** the operator runs `boundline inspect`,
  **Then** the output preserves the same next-action story instead of falling
  back to a generic route or terminal label.
3. **Given** a task whose evidence is insufficient for another bounded action,
  **When** the operator checks `status`, `next`, or `inspect`, **Then** Boundline
  explains the stop condition explicitly and does not imply hidden recovery.

---

### User Story 2 - Preserve Decision Continuity Across Reload And Follow-Up (Priority: P2)

An operator can resume follow-up after a session reload, compatibility run, or
trace inspection and still see the latest bounded decision context without
having to reconstruct it manually from raw trace history.

**Why this priority**: Guided next actions are incomplete if the evidence story
disappears once the active session is reloaded or continuity shifts to the
latest authoritative trace.

**Independent Test**: Execute a representative flow, persist a non-terminal or
inspect-only follow-up state, reload from the saved session or workspace trace,
and verify that `status`, `next`, and `inspect` still project the current
decision continuity and recommended bounded action.

**Acceptance Scenarios**:

1. **Given** a native session whose latest decision is not terminal,
  **When** the operator reloads and runs `boundline status`, **Then** Boundline
  preserves the latest bounded decision continuity needed to explain the next
  action without requiring manual trace reconstruction.
2. **Given** an explicit compatibility run that owns the latest authoritative
  follow-up state, **When** the operator runs `boundline next` or `boundline inspect`,
  **Then** Boundline reuses persisted trace evidence to explain the follow-up path
  while keeping compatibility ownership explicit.
3. **Given** older or conflicting evidence between session state and the latest
  authoritative trace, **When** Boundline projects follow-up guidance, **Then** it
  makes the winning evidence source explicit instead of silently mixing them.

---

### User Story 3 - Ship Guided Follow-Through As One Release (Priority: P3)

A maintainer can ship one coherent `0.28.0` release where the runtime behavior,
trace or session continuity, documentation, version metadata, and validation
evidence all describe the same guided follow-through story.

**Why this priority**: This slice changes how operators interpret what Boundline
should do next. The release is incomplete if the runtime becomes more explicit
but docs, changelog, prompts, or validation discipline still describe the old
generic follow-up behavior.

**Independent Test**: Follow the updated docs on a representative workspace,
confirm that runtime output matches the documented continuity story, and finish
release validation including version bump, impacted docs and changelog,
coverage refresh for touched Rust files, clippy cleanup, and formatting.

**Acceptance Scenarios**:

1. **Given** the `0.28.0` release artifacts, **When** a maintainer follows the
  documented follow-through workflow, **Then** the observed `status`, `next`,
  and `inspect` output matches the documented decision-continuity story.
2. **Given** changed Rust sources for this slice, **When** maintainers run the
  release validation suite, **Then** formatting, clippy, focused tests, and
  coverage refresh for modified or created Rust files complete without
  undocumented regressions.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when the latest evidence does not justify any next bounded
  action and Boundline must stop instead of suggesting another step?
- How does the system handle a session reload where persisted session evidence
  and the latest authoritative trace disagree about the most credible follow-up?
- How does the system surface primary session-native continuity versus explicit
  compatibility continuity without implying that both are equally authoritative?
- What happens when follow-up evidence exists in raw traces but has not yet been
  projected into the operator-facing continuity story?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST project the next bounded action on existing
  `status`, `next`, and `inspect` surfaces in terms of the latest credible
  decision, recovery, validation, or governance evidence.
- **FR-002**: System MUST explain why the projected next action is credible,
  including the winning evidence source when session state and trace evidence
  could both inform follow-up.
- **FR-003**: System MUST preserve enough decision continuity across session
  persistence and workspace trace follow-up that operators can resume without
  reconstructing recent decision history manually.
- **FR-004**: System MUST keep the primary session-native route authoritative
  when it owns the latest bounded follow-up state and MUST keep explicit
  compatibility continuity visibly separate when compatibility owns it instead.
- **FR-005**: System MUST project at least one non-success path such as retry,
  replanning, blocked governance, inspect-only continuity, or evidence
  exhaustion without falling back to generic lifecycle wording alone.
- **FR-006**: System MUST make explicit when no further bounded action is
  credible and MUST stop instead of implying hidden recovery or silent retry.
- **FR-007**: System MUST reuse existing session records, trace summaries, and
  follow-up surfaces before introducing any new control plane or background
  orchestration path.
- **FR-008**: System MUST preserve or improve inspectability of decision
  continuity on both session-native and explicit compatibility routes.
- **FR-009**: System MUST update tests, version metadata, impacted docs,
  assistant guidance, and changelog together for the `0.28.0` release.
- **FR-010**: System MUST refresh coverage for modified or created Rust files,
  resolve clippy issues introduced by the slice, and finish with repository
  formatting applied.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: guided next-action projection on existing `status`, `next`, and
  `inspect` surfaces; explicit decision continuity across persisted session and
  authoritative trace follow-up; evidence-source precedence that stays visible
  to operators; `0.28.0` release closeout including version bump, impacted
  docs, changelog, coverage refresh, clippy cleanup, and formatting.
- **Out of Scope**: new background workers; autonomous control loops; generic
  provider gateways; broader planning heuristics unrelated to follow-through;
  new UI surfaces; distributed execution; long-term memory outside current task
  scope; deployment-pipeline changes.

### Key Entities *(include if feature involves data)*

- **Decision Continuity Snapshot**: The bounded summary of the latest decision,
  recovery or validation implication, and winning evidence source needed to
  explain what Boundline should do next.
- **Follow-Through Guidance**: The operator-facing explanation that connects a
  current session or authoritative trace state to one concrete next bounded
  action or explicit stop condition.
- **Continuity Evidence Source**: The named source, such as persisted session
  state or latest authoritative trace, that Boundline chooses when projecting the
  next bounded action.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative retry, replan, blocked, and inspect-only
  scenarios, operators can identify both the next bounded action and the reason
  for it from `status`, `next`, or `inspect` in under 2 minutes.
- **SC-002**: 100% of representative session reload and explicit compatibility
  follow-up scenarios preserve an explicit decision-continuity story instead of
  forcing manual raw-trace reconstruction.
- **SC-003**: 100% of representative scenarios where no further bounded action
  is credible stop with an explicit guidance or exhaustion explanation rather
  than implying hidden recovery.
- **SC-004**: Maintainers can validate the `0.28.0` guided follow-through
  story, including touched-Rust coverage output, in under 20 minutes using the
  shipped docs and repository validation commands.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Session-native execution remains the primary operator path for this slice,
  and explicit compatibility follow-up remains a separately named continuity
  source instead of becoming resumable session authority.
- Existing session records and trace summaries already capture enough bounded
  evidence to support better follow-through projection without widening the
  orchestration model.
- The slice should prefer projecting and preserving evidence that already exists
  over inventing broader planning or recovery heuristics.
- The `0.28.0` release should remain a minimal continuity and guidance slice
  rather than expanding into provider routing, new review systems, or UI work.
