# Feature Specification: Governed Stage Depth

**Feature Branch**: `020-governed-stage-depth`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Estendere la copertura dei governed stage oltre il solo verify-stage security-assessment, migliorare riuso dei packet, approval refresh e blocked-state guidance su piu transizioni, mantenere invariati i vincoli di orchestrazione session-native e projection workflow-aware senza Canon come owner dell'orchestrazione."

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

### User Story 1 - Govern Investigate Before Verify On One Session Route (Priority: P1)

A developer can drive a bounded bug-fix task through a governed `investigate` stage and later a governed `verify` stage on the primary session-native route instead of treating Canon governance as a verify-only detour.

**Why this priority**: The current governed path is most visible at verify-stage `security-assessment`. The smallest next delivery gain is to make earlier or intermediate governed stages credible on the same session-owned route.

**Independent Test**: Can be fully tested by running a representative bug-fix session with governance configured for `investigate`, then confirming that Synod governs that stage, persists the governed packet and stage record, and later reuses bounded governance lineage when the same session reaches governed `verify`.

**Acceptance Scenarios**:

1. **Given** an active bounded bug-fix session whose next configured governed stage is `investigate`, **When** the developer runs or resumes the session, **Then** Synod executes that governed stage on the same session-owned route and records the stage key, selected mode, packet reference, and next action in the active session and trace.
2. **Given** the same session later reaches governed `verify`, **When** Synod prepares that later governed transition, **Then** it reuses bounded packet context from the earlier governed stage when credible and records the upstream source stage plus binding reason instead of silently discarding lineage.
3. **Given** a governed stage cannot continue because approval is still pending, the packet is incomplete or rejected, or the required governance runtime is unavailable, **When** the developer runs or resumes the session, **Then** Synod stops in an explicit waiting, blocked, or failed condition before the next delivery stage advances.

---

### User Story 2 - Refresh Governance State And Guidance Across Transitions (Priority: P2)

An operator can understand what happened at each governed stage transition through `run`, `status`, `next`, `inspect`, and workflow-aware surfaces without inferring hidden Canon behavior.

**Why this priority**: Broader governed-stage coverage is only useful if state refresh, packet lineage, approval status, and blocked reasons remain visible and actionable across more than one transition.

**Independent Test**: Can be fully tested by pausing a session at different governed stages with reusable, approval-pending, and blocked packets, then verifying that session-native and workflow-aware surfaces refresh the governance state and expose consistent next-step guidance.

**Acceptance Scenarios**:

1. **Given** an active session paused at a governed stage with approval pending or newly refreshed governance state, **When** the developer runs `status`, `next`, `run`, or workflow-aware resume/status, **Then** Synod refreshes the approval state before advancing work and reports the updated governance condition plus next command.
2. **Given** a governed packet reused from an earlier bounded stage, **When** the developer inspects the session or workflow state, **Then** Synod exposes the packet headline, readiness, source stage, and binding reason through the same routing and execution-condition story already used by the session-native runtime.
3. **Given** a named workflow is active over the same bounded session, **When** the workflow reaches `run`, `review`, `govern`, or later inspect steps after a governed stage transition, **Then** the workflow surface preserves the same explicit session-native route ownership and governance guidance instead of implying Canon-owned orchestration.

---

### User Story 3 - Author And Ship Bounded Governed Depth Clearly (Priority: P3)

A maintainer can author or update bounded governance profiles and release documentation for deeper governed-stage coverage without widening Synod into a generic governance engine.

**Why this priority**: The slice is only sustainable if maintainers can see which stage transitions are supported, which packet reuse stories are intentional, and which ownership boundaries remain fixed.

**Independent Test**: Can be fully tested by following the shipped guidance to configure a representative governed stage before `verify`, validating that unsupported stage or mode combinations stay explicit, and confirming that release docs describe the new bounded governed-stage depth coherently.

**Acceptance Scenarios**:

1. **Given** a maintainer configures bounded governance for `bug-fix:investigate` ahead of the existing governed `verify` path, **When** they follow the shipped guidance and examples, **Then** Synod accepts the supported configuration and keeps direct session-native plus workflow-aware routing semantics explicit.
2. **Given** a maintainer configures an unsupported governed stage shape, Canon mode, or hidden background progression expectation, **When** Synod validates or executes that configuration, **Then** it rejects the unsupported behavior explicitly instead of passing it through as unchecked Canon orchestration.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a downstream governed stage sees a prior packet reference but that packet is no longer reusable or is missing required documents?
- What happens when approval refresh changes a governed stage from waiting to blocked, or from waiting to reusable, between two operator commands?
- What happens when governance is configured for an earlier stage but the active task has already entered a non-success terminal state?
- What happens when a named workflow projects an active session that contains governed packet lineage from multiple bounded stage transitions?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST preserve the session-native route as the primary control plane while allowing more than one bounded governed stage transition within the same active session.
- **FR-002**: System MUST support governed execution for `bug-fix:investigate` on the primary session-native route in addition to the existing governed `verify` path.
- **FR-003**: System MUST persist explicit governed stage state for each active transition, including stage key, runtime, lifecycle state, approval state, attempt lineage, blocked reason, and packet reference when present.
- **FR-004**: System MUST evaluate bounded packet reuse across governed stage transitions and MUST record packet source stage plus binding reason whenever previously produced governance evidence is reused.
- **FR-005**: System MUST refresh governance approval and packet-readiness state on later operator commands before resuming a governed stage or allowing downstream stage progression.
- **FR-006**: System MUST stop at the first unmet governance condition across governed stage transitions and MUST surface whether work is waiting, blocked, failed, or terminal together with the next action required to continue or inspect.
- **FR-007**: System MUST preserve explicit packet-readiness outcomes such as reusable, incomplete, and rejected rather than collapsing them into generic terminal messaging.
- **FR-008**: System MUST expose governed stage identity, selected mode, packet readiness, packet provenance, approval state, and blocked reason consistently across `run`, `status`, `next`, `inspect`, and workflow-aware projection surfaces.
- **FR-009**: System MUST preserve direct session-native commands and the bounded workflow layer as projections of the same underlying session state, and MUST NOT imply that Canon owns workflow progression or session orchestration.
- **FR-010**: System MUST reject unsupported governed-stage configurations, unsupported Canon-mode combinations, and hidden background progression expectations explicitly.
- **FR-011**: System MUST preserve inspectable trace evidence for governance start, completion, approval waits, packet rejection, reuse lineage, refresh, and blocked outcomes across the expanded governed-stage slice.
- **FR-012**: System MUST ship release-aligned maintainer and operator guidance for the new governed-stage depth, including version bump, changed usage examples, and updated changelog coverage for the delivered slice.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: deeper governed-stage coverage on the existing session-native runtime by adding governed `bug-fix:investigate` to the already governed `verify` story; bounded packet reuse across those governed transitions; approval refresh and blocked-state guidance across more than one governed transition; workflow-aware projection of the same governance state; release-aligned docs for the delivered slice.
- **Out of Scope**: Canon-owned orchestration; generic governance graphs or background progression; new built-in flow families; full Canon artifact exposure; distributed governance coordination; UI work; provider-routing expansion unrelated to the bounded `bug-fix` governed-depth slice; broadening all supported governed stages in one release.

### Key Entities *(include if feature involves data)*

- **Governed Stage Transition**: The active bounded stage transition that routes a current session step through governance, including stage key, runtime, lifecycle state, approval state, and any blocked or terminal reason.
- **Governed Stage Packet**: The bounded governance evidence packet produced or reused for one governed transition, including readiness, headline, packet reference, expected documents, and selected Canon mode when applicable.
- **Packet Reuse Lineage**: The persisted relationship between a current governed stage and earlier governance evidence, including upstream stage key, binding reason, and whether the packet remains credible for reuse.
- **Governance Refresh Outcome**: The operator-visible result of re-checking approval state or packet readiness on later commands before work resumes.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative governed bug-fix scenarios, Synod can execute governed `investigate` before governed `verify` and still keep the session on the same primary route.
- **SC-002**: 100% of representative packet-reuse, approval-pending, packet-rejected, and governance-blocked scenarios stop in an explicit waiting, blocked, failed, or terminal state before downstream work advances.
- **SC-003**: Developers can identify the active governed stage, selected mode, packet provenance, approval state, and next command from `status`, `next`, `inspect`, or workflow-aware status in under 2 minutes.
- **SC-004**: Maintainers can configure and validate a representative deeper governed-stage profile, plus find the changed operator guidance, in under 15 minutes without relying on undocumented behavior.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators continue to use one active workspace session and rely on `.synod/session.json` plus `.synod/traces/` as the authoritative state and evidence surfaces.
- The existing governance model already declares a bounded set of supported governed stages and Canon modes, and this slice should deepen the credible execution story specifically for `bug-fix:investigate` before broadening other stage families.
- Packet reuse must remain bounded to explicit packet references, readiness metadata, headlines, and binding reasons rather than exposing or depending on the full `.canon/` artifact tree.
- The slice closes as the next versioned delivery increment and therefore includes release-aligned documentation, changelog, coverage refresh, clippy cleanup, and formatting work.
