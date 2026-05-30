# Feature Specification: Delivery Flows (SDLC Backbone)

**Feature Branch**: `005-delivery-flows`  
**Created**: 2026-04-25  
**Status**: Draft  
**Input**: User description: "Introduce explicit delivery flows that transform a problem into executable steps across SDLC stages using the existing session model."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run a standard bug-fix flow (Priority: P1)

As a developer working on a bounded repair task, I want to bind a known delivery flow to the active session so Boundline can move through investigation, implementation, and verification in a predictable order.

**Why this priority**: Bug-fix work is the smallest high-value delivery slice because it exercises stage sequencing, shared session state, and failure handling without requiring broader workflow modeling.

**Independent Test**: Start a session for a failing engineering task, select the bug-fix flow, execute it to completion, and confirm the session records each stage transition and terminal outcome without manual restructuring of the flow.

**Acceptance Scenarios**:

1. **Given** an active session with a captured repair goal and no flow selected, **When** the user selects the bug-fix flow, **Then** Boundline records the selected flow, sets the first stage as current, and exposes the session as ready for execution.
2. **Given** an active session using the bug-fix flow, **When** the current stage completes successfully, **Then** Boundline advances to the next stage, preserves prior stage context, and records the transition in inspectable session output.
3. **Given** an active session using the bug-fix flow, **When** a step inside the implementation or verification stage fails, **Then** Boundline keeps execution within the same stage, records the failure, and allows bounded retry or replan without changing the selected flow.

---

### User Story 2 - Run a standard change flow (Priority: P2)

As a developer making a scoped product or maintenance change, I want a lighter-weight delivery flow so Boundline can move from change understanding into implementation and verification using the same session model.

**Why this priority**: Change work is a common path that validates the feature beyond bug fixing while reusing the same stage-tracking primitives.

**Independent Test**: Start a session for a requested change, select the change flow, execute through all stages, and confirm status and next-command guidance reflect the current stage until completion.

**Acceptance Scenarios**:

1. **Given** an active session with a captured change goal, **When** the user selects the change flow, **Then** Boundline binds that flow to the session and initializes stage progress for the change-oriented sequence.
2. **Given** a session using the change flow, **When** the user requests status or next guidance mid-flow, **Then** Boundline reports the active flow, current stage, stage progress, and the next valid action for continuing the flow.

---

### User Story 3 - Run a full delivery flow (Priority: P3)

As a developer tackling a broader engineering request, I want Boundline to guide work across requirements, architecture, backlog shaping, and implementation so the session reflects a full delivery path instead of isolated steps.

**Why this priority**: This story extends the same deterministic flow model to a longer SDLC path after the core flow infrastructure is proven with smaller slices.

**Independent Test**: Start a session for a broader delivery goal, select the full delivery flow, and confirm Boundline can progress stage by stage while keeping stage order deterministic and visible across the entire session.

**Acceptance Scenarios**:

1. **Given** an active session with a broader delivery goal, **When** the user selects the delivery flow, **Then** Boundline initializes the full ordered stage sequence and exposes the first stage as current.
2. **Given** a session in the delivery flow, **When** all stages reach terminal completion, **Then** Boundline ends the flow in an explicit completed state and retains an inspectable record of each stage transition.

### Edge Cases

- If a user tries to select a flow without an active session, Boundline must reject the request with a clear terminal outcome and must not create implicit session state.
- If a session already has an active flow and the user selects a different flow before completion, Boundline must require an explicit reset or replacement path rather than silently overwriting stage history.
- If the current stage exhausts its configured execution bounds for retry or replan, or cannot produce a credible next step within the current stage, Boundline must stop in an explicit failure state while keeping the active stage and failure evidence inspectable.
- If a previously valid session is missing stage-tracking state, Boundline must stop execution, surface the invalid state, and avoid advancing the flow until the session is repaired or restarted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST allow a user to bind one predefined delivery flow to the active session for a bounded engineering task.
- **FR-002**: Boundline MUST persist the selected flow, the current stage identifier, and the current stage position within session state.
- **FR-003**: Boundline MUST define each supported flow as a deterministic ordered list of stages that does not change during normal execution.
- **FR-004**: Boundline MUST execute work against the current stage only and MUST advance to the next stage only after the current stage reaches a successful terminal outcome.
- **FR-005**: Boundline MUST keep stage execution on top of the existing session model so previously recorded goal, context, plan state, and traces remain available across stage transitions.
- **FR-006**: Boundline MUST allow bounded retry or bounded replanning within the current stage after a failed step without replacing the selected flow or skipping stages.
- **FR-007**: Boundline MUST expose the active flow, current stage, stage progress, and current step progress through status-oriented session output.
- **FR-008**: Boundline MUST provide next-action guidance that reflects the active flow, current stage, and current execution state.
- **FR-009**: Boundline MUST support at least these predefined flows: bug-fix, change, and delivery.
- **FR-010**: Boundline MUST reject invalid flow operations, including selecting a flow without an active session or attempting to advance from an invalid stage state, with explicit user-visible errors.
- **FR-011**: Boundline MUST emit inspectable evidence for flow selection, stage transitions, retries, replans, failures, and terminal outcomes.
- **FR-012**: Boundline MUST preserve existing non-flow session usage so a user can continue to run session commands without selecting a delivery flow.

### Scope Boundaries *(mandatory)*

- **In Scope**: deterministic predefined delivery flows, stage tracking within session state, stage-aware execution and recovery, stage-aware status visibility, and stage-aware next-command guidance.
- **Out of Scope**: adaptive flow generation, custom user-defined flows, multi-agent execution, voting or review councils, Canon integration, background automation, and complex branching or concurrent stage execution.

### Key Entities *(include if feature involves data)*

- **Flow Definition**: A named static delivery sequence containing the ordered stages for a supported flow and the rules for the first stage and terminal completion.
- **Flow Session State**: The session-bound record of the selected flow, current stage, stage index, total stage count, and any stage-level terminal status needed to continue, retry, or stop.
- **Stage Progress Record**: The inspectable record of the active or completed stage, including stage identifier, entered-at point, terminal result, and any retry or replan evidence associated with that stage.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative validation scenarios for bug-fix, change, and delivery work, 100% of runs with a selected flow start in an explicit first stage and end in an explicit completed or failed terminal state.
- **SC-002**: In representative validation scenarios, Boundline preserves correct stage ordering for every supported flow with zero skipped stages or silent stage replacements.
- **SC-003**: Developers can determine the active flow, current stage, stage progress, and most recent stage outcome from status or inspect output in under 30 seconds.
- **SC-004**: When a step fails during a flow validation scenario, Boundline keeps recovery within the current stage for 100% of observed retry or replan cases unless the session is explicitly reset.
- **SC-005**: Existing non-flow session usage remains functional in validation scenarios without requiring a selected flow.

## Assumptions

- An active session already provides the shared goal, trace references, and step state needed for stage-aware execution.
- The initial release only needs a small fixed catalog of built-in flows and does not need external configuration.
- Each stage can reuse the existing planning and execution primitives rather than introducing a new execution engine.
- Stage transitions can be represented and inspected through the current session and trace surfaces without adding a separate persistence system.
