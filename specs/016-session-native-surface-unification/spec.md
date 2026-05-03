# Feature Specification: Session-Native Surface Unification

**Feature Branch**: `016-session-native-surface-unification`  
**Created**: 2026-04-29  
**Status**: Draft  
**Input**: User description: "Refound the remaining Boundline operator-facing surfaces so the session-native runtime becomes the single dominant mental model after 015-runtime-refoundation."

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

### User Story 1 - One Coherent Session View (Priority: P1)

A developer working through Boundline's primary workflow can move from planning to execution to inspection without having to reinterpret the product each time. The same session-owned summary explains which route is active, what flow state applies, what the latest bounded decision is, whether the session is running, blocked, waiting, or terminal, and what the next action should be.

**Why this priority**: If the core operator surfaces still feel like separate products, the runtime refoundation remains incomplete even if the underlying execution model is correct.

**Independent Test**: Can be fully tested by preparing a session-native task and verifying that `run`, `status`, `next`, and `inspect` all present the same route explanation, decision summary, and remediation guidance across active, blocked, and terminal states.

**Acceptance Scenarios**:

1. **Given** a workspace with a ready bounded task draft and no special review, adaptive, or governance activity, **When** the developer runs work and then checks status and inspection surfaces, **Then** each surface explains the same active route, current flow state, latest decision state, and next recommended action.
2. **Given** a workspace where execution is blocked because confirmation or other required context is missing, **When** the developer checks the operator surfaces, **Then** Boundline reports the blocked state explicitly, preserves the relevant reason, and recommends a concrete next step instead of leaving the developer to infer what is wrong.
3. **Given** a session that reaches a non-success terminal state such as exhaustion or no actionable next step, **When** the developer inspects the outcome, **Then** the terminal condition, latest evidence, and follow-up guidance remain visible across the session surfaces.

---

### User Story 2 - Unified Optional Mode Summaries (Priority: P2)

A developer using bounded review, adaptive execution, or governed stages still experiences those capabilities as part of one session-native workflow rather than as separate runtime modes. Optional bounded behaviors appear as extensions of the same session summary and inspection story.

**Why this priority**: Review, adaptive, and governance features are already valuable, but they still weaken the product story when they appear to replace the primary runtime model instead of enriching it.

**Independent Test**: Can be fully tested by running representative review, adaptive, and governed scenarios and verifying that each one projects through the same session-owned summary model with consistent route explanation, current state, and next-command guidance.

**Acceptance Scenarios**:

1. **Given** a task that triggers review or adaptive behavior, **When** the developer checks `run`, `status`, `next`, or `inspect`, **Then** the review or adaptive details appear as bounded additions to the same session summary rather than as a separate execution story.
2. **Given** a governed stage that is waiting for approval or otherwise blocked, **When** the developer checks the operator surfaces, **Then** Boundline reports the wait or block state as part of the same session-owned summary and provides explicit follow-up guidance.

---

### User Story 3 - Explicit Compatibility Path (Priority: P3)

Developers who intentionally use declarative compatibility profiles can continue to do so, but Boundline makes that path visibly distinct from the primary session-native workflow. When both a ready session-native plan and compatibility artifacts exist, the session-native path remains authoritative unless the developer explicitly requests compatibility behavior.

**Why this priority**: Backward-compatible behavior still matters, but the old path cannot remain the hidden default if Boundline is supposed to feel session-native first.

**Independent Test**: Can be fully tested by comparing a compatibility-only run with a workspace that also has a ready session-native plan, then verifying that route choice, precedence, and explanations remain explicit in every operator-facing surface.

**Acceptance Scenarios**:

1. **Given** a workspace that only has a declarative compatibility profile, **When** the developer runs work, **Then** Boundline allows the compatibility path and clearly labels it as compatibility behavior.
2. **Given** a workspace that has both compatibility artifacts and a ready session-native plan, **When** the developer runs work without explicitly choosing compatibility, **Then** Boundline follows the session-native path and explains why that route took precedence.
3. **Given** a developer intentionally choosing compatibility behavior, **When** the run completes or stops, **Then** the operator surfaces preserve the compatibility explanation without overwriting the authoritative session-native summary model.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a ready session-native plan and a compatibility profile are both present but imply different routes?
- What happens when review, adaptive, or governance activity is present but no new decision has been dispatched yet?
- What happens when a surface is asked to explain a blocked or waiting state that is not terminal but still requires operator action?
- What happens when the latest trace, latest decision, and latest summary state do not all exist yet because execution has not started?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST treat the session-native workflow as the primary operator journey across planning, execution, status, next-step guidance, and inspection.
- **FR-002**: System MUST persist a session-owned summary that explicitly represents route choice, flow state, latest bounded decision state, and current execution condition.
- **FR-003**: System MUST present route choice and route rationale consistently across `run`, `status`, `next`, and `inspect` so the developer does not need to infer why a route was selected.
- **FR-004**: System MUST preserve explicit blocked, waiting, running, and terminal explanations and expose matching next-step guidance whenever operator action is still possible.
- **FR-005**: System MUST project review, adaptive, and governance state through the same session-owned summary model as native runtime decisions when those bounded capabilities are active.
- **FR-006**: System MUST keep declarative compatibility behavior available as an explicit path while clearly distinguishing it from the primary session-native route.
- **FR-007**: System MUST give a ready session-native plan precedence over compatibility artifacts unless the developer explicitly chooses compatibility behavior.
- **FR-008**: System MUST preserve the latest relevant review, adaptive, governance, and decision evidence needed to explain the current session state without forcing the developer to reconstruct it from raw traces.
- **FR-009**: System MUST ensure that compatibility runs, waiting states, and blocked states do not silently overwrite or obscure the authoritative session-owned summary of what the developer should understand next.
- **FR-010**: System MUST omit optional state cleanly when review, adaptive, governance, or compatibility behavior is not active, without implying that a different runtime mode is in control.
- **FR-011**: System MUST keep Canon as a stage-boundary governance overlay whose operator-facing state is visible through the unified summary model, not as the per-action control plane.
- **FR-012**: System MUST keep failure and non-success outcomes inspectable across the operator surfaces, including the latest reason work cannot proceed normally.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: unifying route explanations, flow summaries, latest decision state, review projections, adaptive projections, governance wait or block projections, compatibility-mode explanations, and next-step guidance across the session-native operator surfaces.
- **Out of Scope**: new flow families, provider abstraction or model gateway work, distributed or multi-repository execution planning, expanded Canon escalation beyond bounded governance and evidence projection, a full orchestrator redesign, and open-ended autonomous execution.

### Key Entities *(include if feature involves data)*

- **Unified Session Summary**: The operator-facing session record that explains the selected route, flow state, latest bounded decision state, optional review or adaptive or governance state, and what the developer should do next.
- **Route Explanation**: The explicit statement of which execution path is active, why it was chosen, and whether it is primary session-native behavior or an intentional compatibility path.
- **Execution Condition**: The current state of the session from the operator perspective, such as running, blocked, waiting, succeeded, failed, exhausted, or no actionable next step.
- **Optional Mode Projection**: A bounded projection of review, adaptive, or governance information attached to the unified session summary when those capabilities are active.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative native, review, adaptive, governance, and compatibility scenarios, developers can identify the active route and the reason it was chosen from `status`, `next`, or `inspect` in under 2 minutes.
- **SC-002**: 100% of representative blocked, waiting, and terminal scenarios expose an explicit current condition and at least one clear next action or terminal explanation through the operator-facing surfaces.
- **SC-003**: 100% of representative cases where optional review, adaptive, or governance behavior is active keep those details visible through the same session-owned summary model rather than requiring a separate operator workflow.
- **SC-004**: 100% of representative precedence scenarios where both compatibility artifacts and a ready session-native plan exist follow the intended default route unless compatibility behavior was explicitly requested.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators continue to run Boundline in local workspaces with one active session per workspace and persisted session and trace state available between commands.
- Existing bounded review, adaptive, governance, and compatibility capabilities remain available; this slice changes how they are projected and understood, not whether they exist.
- Declarative compatibility profiles remain a supported explicit input surface for teams that still rely on them.
- Governed workflows may depend on Canon-backed evidence and approvals at stage boundaries, but core operator understanding must not depend on Canon becoming the per-action runtime controller.
