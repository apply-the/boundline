# Feature Specification: Workflow Follow-Through

**Feature Branch**: `019-workflow-follow-through`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Deepen bounded workflow follow-through for Boundline's session-native workflow layer by making review and govern executable from the workflow surface, improving workflow discovery and invocation guidance, and clarifying authored workflow registry guidance without broadening Boundline into a generic workflow engine."

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

### User Story 1 - Continue Through Review And Govern (Priority: P1)

A developer can run a named workflow that reaches bounded review or governance work and have Boundline continue through those phases using the same session-owned runtime instead of stopping at them as declaration-only blockers.

**Why this priority**: Until review and govern can execute from the workflow surface, the first workflow slice remains incomplete for any bounded delivery journey that requires quality control or governance before completion.

**Independent Test**: Can be fully tested by defining one workflow that includes review and govern, running it through a representative bounded engineering task, and confirming that Boundline either completes those phases or stops in an explicit blocked or failed state without manual session edits.

**Acceptance Scenarios**:

1. **Given** an active named workflow whose next declared phase is review and whose bounded review prerequisites are satisfied, **When** the developer resumes the workflow, **Then** Boundline executes the review phase through the primary session-owned route and records the resulting workflow state, execution condition, and next action.
2. **Given** an active named workflow whose next declared phase is govern and whose governance prerequisites are satisfied, **When** the developer resumes the workflow, **Then** Boundline executes the govern phase without transferring workflow ownership to Canon and preserves the resulting session and trace evidence.
3. **Given** an active named workflow that reaches review or govern but the phase cannot complete because approval, reviewer outcome, or bounded prerequisites are missing, **When** the developer resumes the workflow, **Then** Boundline stops in an explicit blocked, paused, or failed state and reports the next action needed to continue.

---

### User Story 2 - Discover And Invoke Named Workflows Reliably (Priority: P2)

An operator or assistant can discover which named workflows are available in a workspace, understand when to invoke them, and continue using the same workflow-aware routing and resume guidance already provided by Boundline.

**Why this priority**: Workflow follow-through is harder to use if developers and assistants still need out-of-band knowledge to know which workflow exists, when it is appropriate, or how to continue it correctly.

**Independent Test**: Can be fully tested by preparing a workspace with multiple named workflows, discovering the available choices and guidance from Boundline's operator-facing workflow surface, starting one of them, and confirming that subsequent workflow status and resume guidance stay consistent.

**Acceptance Scenarios**:

1. **Given** a workspace that defines one or more named workflows, **When** an operator or assistant requests the available workflow options, **Then** Boundline exposes the available workflow identities together with enough summary and invocation guidance to choose the correct workflow.
2. **Given** an active named workflow in a non-terminal state, **When** the operator or assistant uses workflow-aware status, next-step, resume, or inspect surfaces, **Then** Boundline reports the workflow identity, active phase, routing, execution condition, and next action consistently.
3. **Given** a workspace with no valid named workflows, **When** an operator or assistant attempts workflow discovery or invocation, **Then** Boundline explains the missing or invalid workflow state explicitly without hiding the direct session-native path.

---

### User Story 3 - Author Workflow Registries With Clear Boundaries (Priority: P3)

A maintainer can author or update workspace-local workflow registries that include review and govern phases while understanding the supported boundaries, examples, and relationship between workflow commands and direct session-native commands.

**Why this priority**: Shipping executable review and govern phases without clear authorship guidance would increase operator ambiguity and make invalid workflow definitions more likely.

**Independent Test**: Can be fully tested by following the shipped guidance to author or update a representative workflow registry that includes review and govern, validating that the documented example remains within the supported bounded model, and confirming that operators can still choose the direct session-native route when needed.

**Acceptance Scenarios**:

1. **Given** a maintainer who wants to add review and govern to a named workflow, **When** they follow the documented guidance and examples, **Then** they can author a valid workflow registry without relying on undocumented behavior.
2. **Given** a maintainer reading the workflow guidance, **When** they compare workflow commands with direct session-native commands, **Then** the documentation makes the primary path, compatibility path, and workflow boundaries explicit.

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Boundline features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when a workflow reaches review or govern after the underlying session has already entered a non-success terminal state?
- What happens when review or govern is declared in the workflow, but the bounded prerequisites for that phase were never produced by prior execution?
- What happens when governance remains blocked or awaiting approval while the workflow is otherwise resumable?
- What happens when workflow discovery is requested in a workspace whose registry is missing, invalid, or contains only unsupported definitions?
- What happens when an operator chooses the direct session-native or explicit compatibility path even though valid workflow definitions exist in the workspace?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST allow a named workflow to execute bounded review and govern phases through Boundline's primary session-owned workflow surface when the declared prerequisites for those phases are satisfied.
- **FR-002**: System MUST preserve explicit workflow identity, active phase, progression state, and resulting execution condition in persisted session state while review and govern are executing or stopping.
- **FR-003**: System MUST stop workflow progression at the first unmet bounded condition during review or govern and MUST surface whether the workflow is paused, blocked, failed, or complete together with the next action needed to continue.
- **FR-004**: System MUST preserve the existing session-native execution limits, terminal states, retry rules, and inspectable trace behavior when review or govern is reached through a named workflow.
- **FR-005**: System MUST keep named workflow progression sequential, session-owned, and bounded, with one active phase at a time and no hidden background advancement.
- **FR-006**: System MUST NOT delegate workflow ownership or progression to Canon; Canon MAY continue to provide bounded governance or evidence behavior only.
- **FR-007**: System MUST preserve the direct session-native path for operators who do not invoke a named workflow.
- **FR-008**: System MUST preserve the explicit compatibility path as an operator-selected alternative and MUST NOT let workflow definitions silently override it.
- **FR-009**: System MUST expose available named workflows and enough operator-facing discovery guidance to support correct workflow selection and invocation in workspaces that define workflow registries.
- **FR-010**: System MUST expose workflow identity, active phase, routing, execution condition, and next action consistently across workflow-aware run, status, resume, next-step, and inspect surfaces while a named workflow is active.
- **FR-011**: System MUST reject unsupported workflow behavior explicitly, including attempts to reintroduce generic workflow-engine semantics such as branching, loops, fan-out, fan-in, hidden concurrency, or background progression.
- **FR-012**: System MUST provide maintainer guidance and examples for authoring workflow registries that include review and govern while making supported boundaries and non-goals explicit.
- **FR-013**: System MUST preserve inspectable evidence for review and govern outcomes, including non-success and blocked states, through the same session and trace surfaces used for other bounded delivery work.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Boundline specs should normally exclude councils and voting unless the roadmap and
  constitution explicitly prioritize a bounded review slice; they should otherwise
  exclude provider-routing complexity, distributed execution, long-term memory,
  UI/UX work, and deployment pipelines.
-->

- **In Scope**: executable review and govern phases from the named workflow surface; workflow discovery and invocation guidance for operators and assistants; workflow-aware continuation, blocked-state reporting, and inspectability; authored workflow registry guidance and examples; release-aligned documentation for the 0.19.0 slice.
- **Out of Scope**: generic workflow-programming semantics; branching, loops, fan-out, fan-in, or concurrent workflow execution; Canon-owned orchestration; provider-routing expansion; UI work; deployment automation; long-term memory or distributed agent systems.

### Key Entities *(include if feature involves data)*

- **Workflow Progress State**: The session-owned record of the active named workflow, current phase, satisfied phases, stop reason, and next operator action required to continue bounded execution.
- **Workflow Discovery View**: The operator-facing summary of which named workflows exist in a workspace, what each one is intended to do, and how to invoke it correctly.
- **Workflow Registry Guidance**: The documented examples, boundaries, and authoring rules that define how maintainers create supported workflow definitions without drifting into generic workflow-engine semantics.
- **Review Or Governance Outcome**: The inspectable result of executing a bounded review or govern phase, including success, blocked state, non-success terminal state, and the evidence needed to explain what happened.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: 100% of representative named workflows that include review or govern either progress through those phases or stop in an explicit paused, blocked, failed, or completed state without requiring manual session-file edits.
- **SC-002**: Developers can identify the active workflow, active phase, routing, execution condition, and next action for a workflow paused at review or govern in under 2 minutes using Boundline's operator-facing workflow surfaces.
- **SC-003**: Operators or assistants can discover the valid named workflows available in a representative workspace and choose the correct invocation path in under 2 minutes.
- **SC-004**: Maintainers can author or update a representative workflow registry that includes review and govern using shipped guidance and examples in under 15 minutes while staying within the documented bounded model.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Operators continue to use one active Boundline session per workspace and rely on `.boundline/session.json` plus `.boundline/traces/` as the authoritative persisted state surfaces.
- Review and governance capabilities already exist on the direct session-native path and can be extended to named workflows without redefining Boundline as a second orchestration system.
- Workflow registries remain workspace-local and bounded to the existing workflow-definition model rather than introducing a second configuration dialect or a generalized automation language.
- The 0.19.0 slice includes documentation and assistant-guidance updates when those updates are required for operators to use the new bounded workflow behavior correctly.
