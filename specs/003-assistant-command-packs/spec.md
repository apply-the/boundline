# Feature Specification: Assistant Command Packs

**Feature Branch**: `003-assistant-command-packs`  
**Created**: 2026-04-24  
**Status**: Draft  
**Input**: User description: "Expose Boundline workflows as assistant-native slash commands for Copilot, Codex, and Claude, instead of relying on long CLI commands."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start a Workflow from Chat (Priority: P1)

A developer working in a chat-first assistant environment can start a Boundline workflow through a short assistant-native command and receive the exact next action without memorizing CLI syntax.

**Why this priority**: Starting work is the highest-friction point today. If users cannot begin from chat quickly, the rest of the workflow remains inaccessible.

**Independent Test**: Can be fully tested by invoking the start or plan command in a supported assistant environment with and without direct shell access, and confirming the user receives a clear path to begin a Boundline workflow.

**Acceptance Scenarios**:

1. **Given** a developer is using a supported assistant environment and has not started a Boundline workflow, **When** they invoke the planning command from chat, **Then** the assistant asks only for the missing goal information, provides or performs the correct next action, and explains what will happen next.
2. **Given** a developer is using a chat-only environment without direct command execution, **When** they invoke the start or planning command, **Then** the assistant provides copyable instructions, accepts pasted command output, and continues the workflow without forcing the user to restate prior context.

---

### User Story 2 - Continue and Complete Work from Chat (Priority: P2)

A developer can continue an existing Boundline workflow from chat, run the next bounded action, and receive a concise summary of progress, failures, and next steps.

**Why this priority**: Once a workflow has started, users need a consistent way to advance it without dropping into manual CLI discovery or losing track of state.

**Independent Test**: Can be tested by invoking run, step, status, and next commands during an in-progress workflow and verifying that each command either executes directly or guides the user through the same outcome.

**Acceptance Scenarios**:

1. **Given** a Boundline workflow is in progress, **When** the developer invokes a run, step, status, or next command from chat, **Then** the assistant executes or guides the requested action and responds with a summary of the current state, recent result, and recommended follow-up.
2. **Given** the requested action ends in failure, retry, replanning, or exhaustion, **When** the assistant reports the outcome, **Then** it identifies the failed or blocked step, explains the terminal or recovery state in plain language, and proposes the most relevant next command.

---

### User Story 3 - Inspect Prior Runs from Chat (Priority: P3)

A developer can inspect a completed or failed Boundline run from chat and understand what happened without manually reading raw trace output.

**Why this priority**: Inspection is valuable after execution exists, but it depends on workflows already being startable and runnable from chat.

**Independent Test**: Can be tested by providing a completed run or trace reference to the inspection command and confirming the assistant returns an understandable summary with outcome, important events, and next action guidance.

**Acceptance Scenarios**:

1. **Given** a developer has a completed or failed run to inspect, **When** they invoke the inspection command and provide the required trace reference if needed, **Then** the assistant summarizes the final status, notable recovery events, and the most useful next action.

---

### Edge Cases

- A supported assistant environment does not allow direct shell execution and the user must switch to copy-paste guidance without losing workflow continuity.
- The user invokes a workflow command without providing the minimum required context, such as a goal or trace reference.
- The underlying Boundline action fails, retries, replans, or reaches a terminal limit, and the assistant must explain the outcome without dumping raw logs.
- The user pastes partial or noisy command output, and the assistant must still identify whether more information is required before proceeding.
- A command pack is available in one supported assistant environment before another, and users still need a clearly documented fallback path.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an assistant-native command set that lets users start, continue, inspect, and get next-step guidance for Boundline workflows from supported assistant environments.
- **FR-002**: The system MUST support an initial assistant-native command set covering workflow start, planning, single-step guidance, full run execution, status lookup, next-step guidance, and run inspection.
- **FR-003**: Each assistant-native command MUST state the user intent it serves, the minimum input it needs, the action it will take when direct execution is available, and the fallback guidance it will provide when direct execution is unavailable.
- **FR-004**: The system MUST allow users in chat-only environments to continue a workflow by pasting command output back into the conversation and receiving the next guided action without re-entering previously supplied context.
- **FR-005**: The system MUST summarize workflow outcomes in plain language, including current status, important step results, failures, retries, replans, exhaustion, and recommended next steps when applicable.
- **FR-006**: The system MUST provide a consistent command experience across the supported assistant environments so that the same Boundline workflow can be started and continued with equivalent user guidance.
- **FR-007**: The system MUST include repository documentation that explains which assistant environments are supported, how users enable the command packs, and how users proceed when direct execution is not available.
- **FR-008**: The system MUST remain compatible with Boundline's CLI-first workflow model and MUST not require a new external runtime service or one-to-one new Boundline CLI subcommands in order for assistant-native commands to function.

### Scope Boundaries *(mandatory)*

- **In Scope**: Assistant-native command packs for supported chat environments, command guidance for start/plan/step/run/status/next/inspect workflows, fallback behavior for chat-only sessions, and documentation for installation and usage.
**In Scope**: Assistant-native command packs for supported chat environments, assistant-level workflow routing over the existing CLI for start/plan/step/run/status/next/inspect flows, fallback behavior for chat-only sessions, and documentation for installation and usage.
- **Out of Scope**: Replacing the existing CLI, adding one-to-one Boundline CLI subcommands solely to mirror assistant command names, introducing new orchestration logic, building a general plugin marketplace, adding platform-specific APIs or servers, or expanding into broader UI or deployment work.

### Key Entities *(include if feature involves data)*

- **Assistant Command Pack**: A packaged set of user-facing assistant commands for one supported environment, including the workflow intent each command serves and the guidance needed for direct or fallback execution.
- **Workflow Command Definition**: A single assistant-facing command that captures the action name, required inputs, execution path, fallback path, and summary expectations for one Boundline workflow step.
- **Execution Context**: The minimum conversational state needed to continue a workflow across commands, including goal details, recent command output, and any run or trace reference supplied by the user.
- **Inspection Summary**: A user-readable explanation of what happened during a Boundline run, including final state, meaningful recovery events, and the next recommended action.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In usability checks, first-time users can start a Boundline workflow from a supported assistant environment in under 2 minutes without consulting separate CLI documentation.
- **SC-002**: For the initial command set, 100% of supported workflow intents can be completed through either direct execution or a documented chat-only fallback path.
- **SC-003**: In representative chat-only trials, at least 90% of workflows can continue from pasted command output without the user needing to restate previously supplied goal or run context.
- **SC-004**: In representative completed and failed runs, users can identify the final status, the most important failure or recovery event, and the next recommended action within one assistant response.

## Assumptions

- Supported assistant environments can expose either slash-style commands, reusable prompts, or an equivalent user-triggered command surface.
- The existing Boundline CLI remains the source of truth for workflow execution and inspection behavior during the initial release of this feature.
- Users can provide pasted command output or missing references when direct execution is unavailable.
- The initial release targets Copilot, Codex, and Claude because they are the primary assistant environments named in the feature request.
