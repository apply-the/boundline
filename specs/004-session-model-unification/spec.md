# Feature Specification: Session & Interaction Model Unification

**Feature Branch**: `004-session-model-unification`  
**Created**: 2026-04-25  
**Status**: Draft  
**Input**: User description: "Unify CLI, orchestrator, and assistant command interaction through a shared session model that removes the need for repeated inputs and enables chat-first workflows."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Start and Reuse Active Work Context (Priority: P1)

As a Synod developer, I can establish and reuse one active session for a workspace so that follow-up commands and assistant interactions continue the same bounded task without forcing me to restate known context.

**Why this priority**: Without a shared active session, every later interaction remains repetitive, fragile, and inconsistent across CLI and assistant surfaces.

**Independent Test**: Can be fully tested by starting a new session in a workspace, invoking follow-up commands without repeating the original task context, and confirming that Synod either continues from the active session or fails with explicit recovery guidance.

**Acceptance Scenarios**:

1. **Given** no active session exists for a workspace, **When** the developer starts a new session, **Then** Synod creates an active session record for that workspace and reports that later interactions can use it automatically.
2. **Given** an active session already exists with known task context, **When** the developer invokes a follow-up command that depends on that context, **Then** Synod resolves the active session automatically and continues against the same work state without requiring repeated inputs.
3. **Given** no active session exists, **When** the developer invokes a command that requires one, **Then** Synod stops immediately with a clear message that explains how to establish a session first.

---

### User Story 2 - Plan and Execute Through Shared Session State (Priority: P2)

As a Synod developer, I can capture a goal, plan it, and execute work step-by-step or end-to-end through the same session so that progress, traces, and latest outcome remain coherent across invocations.

**Why this priority**: Once an active session exists, the next delivery value is being able to move through the bounded execution flow without manually reconstructing state between commands.

**Independent Test**: Can be fully tested by creating an active session, defining a goal, planning it, executing one step or a full run, and confirming that session state, latest trace, and latest outcome stay synchronized after each command.

**Acceptance Scenarios**:

1. **Given** an active session with a captured goal, **When** the developer plans the work, **Then** Synod attaches the new plan to the session and resets execution position so subsequent commands operate on the latest plan.
2. **Given** an active session with a current plan, **When** the developer executes one step at a time or runs the task to a terminal state, **Then** Synod updates the shared session after each meaningful transition, including latest progress and latest trace reference.
3. **Given** execution fails, retries, replans, exhausts its limits, or aborts, **When** Synod updates the session, **Then** the session preserves the latest actionable state so the developer can inspect what happened and continue or recover deliberately.

---

### User Story 3 - Inspect Session State and Route the Next Action (Priority: P3)

As a Synod developer working from CLI or assistant commands, I can inspect the current session and get the next recommended action so that both interaction surfaces behave consistently over the same bounded task.

**Why this priority**: Shared state only delivers full value when both CLI and assistant guidance read from the same session and present the same next-step logic.

**Independent Test**: Can be fully tested by partially executing a session, inspecting status from CLI-oriented and assistant-oriented flows, and confirming that both surfaces report the same current state and recommend the same next valid action.

**Acceptance Scenarios**:

1. **Given** an active session in progress, **When** the developer inspects status or asks for the next action, **Then** Synod reports the current goal, execution position, overall state, latest trace reference, and one valid next command.
2. **Given** an assistant command is invoked while a valid active session already contains the necessary context, **When** the assistant routes through Synod, **Then** it reuses the active session instead of asking the user to repeat already known goal or trace information.
3. **Given** the active session is stale, corrupted, or no longer matches the current workspace, **When** the developer asks for status or continuation guidance, **Then** Synod surfaces the problem explicitly and routes the developer to a safe recovery action instead of continuing with hidden assumptions.

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- An active session exists but its latest trace reference is missing or unreadable.
- An active session exists but contains no captured goal, and the developer requests planning or execution.
- A new plan replaces a previous plan after partial progress, and execution position must not silently continue against stale plan state.
- A session reaches a terminal outcome and the developer issues another execution command without first resetting or replacing the active task context.
- The session record is malformed, partially written, or otherwise unreadable.
- CLI and assistant surfaces request the next action from the same session after a failure, retry, or replan and must not diverge in guidance.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: The system MUST maintain at most one active bounded work session per workspace for the initial release of this capability.
- **FR-002**: The system MUST persist a workspace-scoped session record that preserves the minimum state required to continue an in-progress bounded task across invocations.
- **FR-003**: The system MUST preserve in the session record the current task goal when known, current execution position, latest overall status, latest meaningful outcome, and latest trace reference.
- **FR-004**: The system MUST automatically resolve the active session for commands and assistant interactions that depend on existing context, without requiring the user to provide a session identifier explicitly.
- **FR-005**: The system MUST fail clearly and immediately when a session-dependent command is invoked without an active valid session.
- **FR-006**: The system MUST allow a developer to establish or replace the active task goal within the active session without requiring unrelated context to be re-entered.
- **FR-007**: The system MUST bind plan creation to the active session and MUST reset execution position when a new plan supersedes a previous one.
- **FR-008**: The system MUST support both single-step progression and bounded end-to-end execution against the active session while persisting updated state after each meaningful transition.
- **FR-009**: The system MUST preserve usable recovery state after retries, replanning, failure, exhaustion, or aborted execution so that follow-up commands can inspect and act on the latest known state.
- **FR-010**: The system MUST provide a status view over the active session that exposes enough information for a developer to understand what task is in progress, where execution stands, and what trace to inspect next.
- **FR-011**: The system MUST provide a next-action recommendation for the active session that returns exactly one valid follow-up command together with a brief rationale grounded in the current session state.
- **FR-012**: Assistant command surfaces MUST reuse the active Synod session before asking the user to restate information already preserved in that session.
- **FR-013**: The system MUST detect corrupted, stale, missing, or workspace-mismatched session state and MUST surface explicit recovery guidance rather than continuing with hidden assumptions.
- **FR-014**: This feature MUST unify interaction state across CLI and assistant surfaces without changing the orchestrator's core execution semantics.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils, provider-routing complexity,
  distributed execution, long-term memory, UI/UX work, and deployment pipelines
  unless the constitution has been amended.
-->

- **In Scope**: A single workspace-scoped session model, automatic session resolution, session-backed planning and execution continuity, session-backed status and next-action guidance, and consistent reuse of session state across CLI and assistant command surfaces.
- **Out of Scope**: Multi-session management, remote or shared sessions, distributed execution, Canon integration, long-term memory across projects, advanced session branching or versioning, hidden background execution, and broader delivery-flow libraries beyond the minimal session-backed interaction model.

### Key Entities *(include if feature involves data)*

- **Active Session Record**: The persisted workspace-scoped representation of the current bounded task, including the known goal, latest execution position, latest overall status, latest outcome, and latest trace reference needed for continuation.
- **Session Transition**: A meaningful state change applied to the active session after goal capture, plan creation, step execution, full execution, retry, replan, failure, exhaustion, or reset-worthy terminal completion.
- **Session Status View**: The user-facing summary of the active session that makes current task context, execution progress, latest trace, and next recommended action inspectable from CLI and assistant surfaces.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: In representative validation flows, developers can start, scope, plan, and execute a bounded task through consecutive Synod interactions without re-entering already confirmed context in at least 90% of sampled sessions.
- **SC-002**: For 100% of session-dependent commands in the initial release, Synod either resolves a valid active session successfully or fails with explicit recovery guidance in a single response.
- **SC-003**: In representative interrupted or failed runs, developers can identify the current session state, latest trace reference, and next recommended action in under 1 minute using the provided status and next-action surfaces.
- **SC-004**: CLI and assistant command surfaces return equivalent continuation guidance for the same underlying session state across all primary start, plan, execute, inspect, and recovery workflows in the initial scope.
- **SC-005**: In representative recovery scenarios involving retry, replanning, failure, or exhaustion, developers can continue from the preserved session state without manually reconstructing prior execution context in at least 90% of sampled cases.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- The initial release uses a single active session per workspace rather than introducing multi-session selection or switching.
- The existing orchestrator remains the source of truth for execution behavior, terminal conditions, retries, and replanning semantics.
- Existing execution traces remain available and continue to serve as the inspectable history of what happened during execution.
- Assistant command surfaces can invoke or guide Synod commands, but they do not maintain a separate durable state model outside the shared Synod session.
- Reset, replacement, or cleanup behaviors may be introduced later as dedicated command refinements, but this feature must already make invalid or terminal session states explicit and safe to reason about.
