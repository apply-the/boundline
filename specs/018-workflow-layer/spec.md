# Feature Specification: Session-Native Workflow Layer

**Feature Branch**: `018-workflow-layer`  
**Created**: 2026-04-30  
**Status**: Draft  
**Input**: User description: "Add a thin session-native workflow layer above the existing Boundline runtime with resumable named workflows, a TOML-based workflow definition surface, and a minimal boundline workflow command family that preserves session-native routing, governance, review, and inspect behavior without turning Boundline into a generic workflow engine or delegating orchestration to Canon."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run A Named Delivery Workflow (Priority: P1)

A developer can start one named workflow for a local engineering task and have Boundline drive that work through the same bounded session-native path it already uses for direct `goal -> plan -> run` delivery, instead of requiring the developer to remember or script each phase manually.

**Why this priority**: This is the smallest independently valuable workflow slice. If a named workflow cannot start real delivery work through the existing runtime, the rest of the workflow layer is ornamental.

**Independent Test**: Can be fully tested by defining one valid named workflow in a workspace, invoking it for a bounded engineering task, and confirming that Boundline creates or resumes the expected session-native work path without introducing a second execution engine.

**Acceptance Scenarios**:

1. **Given** a workspace that contains one valid named workflow and no active session, **When** the developer runs that workflow for a bounded engineering task, **Then** Boundline starts the workflow through the primary session-native route and advances only through the bounded phases required by that workflow.
2. **Given** a named workflow whose next phase still requires missing authored input, planning, or confirmation, **When** the developer runs the workflow, **Then** Boundline stops at the first unmet bounded condition and returns one explicit next action instead of silently skipping ahead.
3. **Given** a named workflow definition that is invalid or references an unsupported phase, **When** the developer runs the workflow, **Then** Boundline blocks execution before work starts and explains the invalid workflow state without falling back to a different hidden route.

---

### User Story 2 - Resume And Inspect Workflow Progress (Priority: P2)

A developer can resume, inspect, and understand an in-progress named workflow through the same `status`, `next`, and `inspect` surfaces already used for session-native delivery work.

**Why this priority**: A workflow layer is only credible if developers can see which workflow is active, which phase is current, why execution stopped, and how to continue without reverse-engineering internal state.

**Independent Test**: Can be fully tested by starting a named workflow, forcing it to stop at a bounded non-terminal condition, then verifying that `status`, `next`, or `inspect` expose the workflow identity, active phase, route, execution condition, and resume guidance consistently.

**Acceptance Scenarios**:

1. **Given** an in-progress named workflow that has paused on a bounded non-terminal condition, **When** the developer runs `status`, `next`, or `inspect`, **Then** each surface reports the workflow name, active phase, execution condition, and the next command needed to continue.
2. **Given** a named workflow that has already completed one or more phases, **When** the developer resumes it, **Then** Boundline continues from persisted workflow progress instead of replaying already satisfied phases or discarding earlier trace evidence.

---

### User Story 3 - Keep Workflow Definitions Bounded And Session-Owned (Priority: P3)

A maintainer can add or evolve named workflows without turning Boundline into a generic workflow engine, introducing concurrent orchestration, or giving Canon responsibility for workflow progression.

**Why this priority**: The first workflow slice should improve delivery ergonomics now while protecting the session-native runtime from premature DSL complexity or external orchestration drift.

**Independent Test**: Can be fully tested by validating that a named workflow can only bind to supported bounded phases, that unsupported control-flow behavior is rejected explicitly, and that existing non-workflow session-native commands remain authoritative.

**Acceptance Scenarios**:

1. **Given** a workflow definition that attempts to introduce unsupported looping, fan-out, or hidden concurrency semantics, **When** Boundline validates or runs that workflow, **Then** it rejects the unsupported behavior explicitly instead of interpreting it as implicit orchestration logic.
2. **Given** a workspace that continues to use direct session-native commands or the explicit compatibility path, **When** the developer does not invoke a named workflow, **Then** Boundline preserves the current behavior without forcing workflow definitions into the primary runtime path.

### Edge Cases

- What happens when a named workflow reaches its configured end while the underlying session has already stopped in a non-success terminal state?
- What happens when a workflow phase is requested but its prerequisites are already satisfied from persisted session state?
- What happens when a workflow definition exists alongside an explicit compatibility execution path and the operator chooses the non-workflow route?
- What happens when a workflow definition is present but the workspace has no credible session input, active goal, or resumable context needed for the requested phase?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support named delivery workflows that execute through Boundline's primary session-native route rather than through a separate workflow runtime.
- **FR-002**: System MUST represent workflow identity, current workflow phase, and workflow progression state explicitly in the persisted session state.
- **FR-003**: System MUST validate a workflow definition before execution and MUST reject unsupported phases or unsupported control-flow behavior explicitly.
- **FR-004**: System MUST stop workflow execution at the first unmet bounded condition, such as missing input, pending confirmation, blocked governance, failed validation, or no credible next action, without silently skipping phases.
- **FR-005**: System MUST preserve the existing bounded start conditions, terminal conditions, and retry limits of the underlying session-native runtime when a named workflow is used.
- **FR-006**: System MUST allow a paused named workflow to resume from persisted workflow progress without re-running already satisfied phases by default.
- **FR-007**: System MUST surface workflow identity, active phase, routing, execution condition, and next action consistently through `run`, `status`, `next`, and `inspect` when a named workflow is active.
- **FR-008**: System MUST preserve the current direct session-native command path for operators who do not invoke a named workflow.
- **FR-009**: System MUST preserve the explicit compatibility execution path as an operator-selected alternative rather than letting workflow definitions silently override it.
- **FR-010**: System MUST keep named workflows sequential and bounded in the first slice, with one active phase at a time and no hidden background progression.
- **FR-011**: System MUST keep Canon limited to bounded governance and evidence behavior and MUST NOT delegate workflow ownership or progression to Canon.
- **FR-012**: System MUST keep workflow failures, invalid definitions, and blocked progression inspectable through the same trace and session surfaces used for other bounded delivery work.

### Scope Boundaries *(mandatory)*

- **In Scope**: a thin named-workflow layer over existing session-native delivery phases; workflow validation; persisted workflow progress; workflow-aware status, next, run, and inspect surfaces; explicit preservation of current direct session-native and compatibility paths.
- **Out of Scope**: generic workflow-programming semantics; arbitrary loops, switches, or fan-out; background or concurrent workflow execution; Canon-owned orchestration; new built-in flow families; provider-routing expansion; UI work; deployment automation.

### Key Entities *(include if feature involves data)*

- **Workflow Definition**: A developer-authored named delivery workflow that identifies the bounded phases Boundline may run, the entry point, and the conditions that decide whether the workflow can continue or must stop.
- **Workflow Progress State**: The session-owned record of which named workflow is active, which phase is current, which phases are already satisfied, and what bounded next action remains.
- **Workflow Phase Binding**: The explicit relationship between one workflow phase and an existing session-native delivery phase such as capture, clarify, plan, run, review, govern, or inspect.
- **Workflow Execution Condition**: The operator-visible reason a workflow is progressing, paused, blocked, failed, or complete, together with the next command or remediation needed to continue.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can start one named workflow for a representative local engineering task in under 2 minutes without manually chaining individual session-native commands.
- **SC-002**: 100% of representative invalid or unsupported workflow definitions are rejected before hidden execution begins.
- **SC-003**: Developers can identify the active workflow, current phase, execution condition, and next command from `status`, `next`, or `inspect` in under 2 minutes.
- **SC-004**: 100% of representative workflow-driven runs terminate or pause in an explicit bounded state that preserves inspectable trace evidence.

## Assumptions

- Operators continue to use one active Boundline session per workspace and rely on persisted `.boundline/session.json` plus `.boundline/traces/` state between commands.
- The first workflow slice targets local developer workspaces and bounded sequential delivery work rather than generalized automation.
- Existing session-native phases remain the authoritative building blocks for workflow progression in the first slice.
- Direct session-native commands and the explicit compatibility path remain supported even when a workspace later adds named workflow definitions.
