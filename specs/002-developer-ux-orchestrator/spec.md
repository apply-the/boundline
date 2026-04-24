# Feature Specification: Developer UX for Orchestrator Core

**Feature Branch**: `002-developer-ux-orchestrator`  
**Created**: 2026-04-24  
**Status**: Draft  
**Input**: User description: "Make the orchestrator core usable and testable by a developer without reading tests, by providing a minimal command-line experience, runnable examples, readable execution output, and inspectable traces."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories MUST be prioritized as delivery journeys ordered by importance.
  Each story must improve bounded engineering-task execution and be independently testable.
  If implementing just one story would not produce immediate delivery value, the slice is too broad.

  At least one acceptance scenario in the spec MUST cover a non-success path such as
  retry, replanning, failure, or exhaustion. Avoid stories that describe generic agent
  collaboration, dashboards, chat UX, or abstract reasoning without real execution.
-->

### User Story 1 - Run a Guided Demo Task (Priority: P1)

As a Synod developer, I can start a guided demo run from the command line so I can observe the orchestrator completing a bounded task without reading test code first.

**Why this priority**: The current core works, but its behavior is hidden behind library usage and integration tests. A guided demo is the fastest path to make the orchestrator understandable and experienceable.

**Independent Test**: Can be fully tested by starting the demo command from a fresh repository checkout and confirming that it executes a bounded task, prints visible step progression, and ends in an explicit terminal state.

**Acceptance Scenarios**:

1. **Given** a developer with a working local checkout, **When** they start the demo run, **Then** Synod executes a predefined bounded task and prints step-by-step progress until the task reaches a terminal outcome.
2. **Given** a demo run that encounters a recoverable failure, **When** the orchestrator retries or replans, **Then** the terminal output shows those recovery events clearly enough that the developer can follow why execution continued.

---

### User Story 2 - Run a Simple Custom Task (Priority: P2)

As a developer evaluating Synod, I can start a simple custom task from the command line so I can see how the orchestrator behaves on an objective I supply myself.

**Why this priority**: After the guided demo, developers need one small step toward real usage that still stays deterministic and easy to understand.

**Independent Test**: Can be fully tested by providing a custom goal through the command interface and confirming that Synod executes the default bounded flow, reports progress, and persists a trace for later inspection.

**Acceptance Scenarios**:

1. **Given** a developer-supplied bounded goal, **When** the run command starts execution, **Then** Synod executes the default developer flow, reports the active steps, and ends with a trace location the developer can inspect.
2. **Given** a custom run that stops in a non-success terminal state, **When** the command exits, **Then** Synod reports the final reason in actionable terms and leaves behind a trace that explains the path to failure.

---

### User Story 3 - Inspect a Recorded Run (Priority: P3)

As a developer troubleshooting Synod, I can inspect a recorded run through a dedicated trace-view command so I can understand executed steps, recovery events, and final outcome without opening raw trace data manually.

**Why this priority**: The core already records traces, but raw files are a poor developer experience. A readable inspection surface makes the existing observability actually usable.

**Independent Test**: Can be fully tested by running a task, invoking the trace inspection command on the stored trace, and confirming that the output reconstructs step order, retries or replans, and the terminal result.

**Acceptance Scenarios**:

1. **Given** a completed run with a persisted trace, **When** the developer invokes the trace inspection command, **Then** Synod presents the executed steps, recovery events, and final status in a readable summary.
2. **Given** a trace from a failed or exhausted run, **When** the developer inspects it, **Then** the output highlights the terminal reason and the last meaningful recovery action before the run stopped.

### Edge Cases

<!--
  ACTION REQUIRED: Capture execution limits, invalid state transitions, missing context,
  traceability gaps, and failure-handling boundaries. Synod features are invalid if they
  ignore how work stops, fails, or becomes non-credible.
-->

- What happens when the local developer environment is missing a prerequisite needed to start the command-line experience?
- How does the system handle a run that ends before success because retries, replans, or step limits are exhausted?
- What happens when a developer tries to inspect a missing, unreadable, or malformed trace file?
- How does the output stay understandable when a step emits large or noisy intermediate data that should not flood the terminal?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: Fill this section with testable requirements focused on delivery value,
  bounded execution, state updates, failure handling, and inspectability. Avoid abstract
  platform language or implementation details.
-->

### Functional Requirements

- **FR-001**: System MUST expose a developer-facing command entrypoint that allows a contributor to start and inspect orchestrator runs without writing integration code against the library surface.
- **FR-002**: System MUST provide a guided demo command that executes a predefined bounded task and demonstrates the orchestrator’s normal step progression.
- **FR-003**: System MUST provide a command that accepts a developer-supplied bounded goal and executes it through a default developer flow suitable for exploration and manual validation.
- **FR-004**: System MUST print readable execution progress during runs, including the active step, the step category, and whether the step succeeded or failed.
- **FR-005**: System MUST surface retries, replanning events, and terminal outcomes in the command-line output when those events occur.
- **FR-006**: System MUST persist an execution trace for every run started through the developer command surface.
- **FR-007**: System MUST provide a dedicated trace inspection command that converts a persisted run trace into a readable summary for developers.
- **FR-008**: System MUST report local setup problems through a dedicated diagnostic command so a developer can determine whether the environment is ready before attempting a run.
- **FR-009**: System MUST return explicit success and non-success exit outcomes from each developer-facing command so shell usage and follow-up guidance remain reliable.
- **FR-010**: System MUST present actionable error messages when a run cannot start, a trace cannot be read, or execution terminates before success.
- **FR-011**: System MUST keep the developer experience deterministic enough that the demo path consistently shows meaningful orchestration behavior, including at least one recovery path.
- **FR-012**: System MUST make the core orchestrator behavior understandable without requiring developers to read integration tests or raw trace files as the primary learning path.

### Scope Boundaries *(mandatory)*

<!--
  ACTION REQUIRED: Name the deferred or excluded capabilities explicitly.
  Synod specs should normally exclude councils, provider-routing complexity,
  distributed execution, long-term memory, UI/UX work, and deployment pipelines
  unless the constitution has been amended.
-->

- **In Scope**: A minimal developer-facing command surface for demo runs, simple custom runs, setup diagnostics, readable progress output, and trace inspection over the existing orchestrator core.
- **Out of Scope**: Production-grade workflow automation, rich interactive interfaces, remote execution, Canon integration, real provider integrations, advanced configuration systems, and multi-agent review behavior.

### Key Entities *(include if feature involves data)*

- **Developer Command Session**: One invocation of the developer-facing command surface, including the requested action, console-visible progress, exit outcome, and any referenced trace location.
- **Demo Run Profile**: The predefined bounded task shape used to demonstrate orchestration behavior consistently for onboarding and exploration.
- **Custom Run Request**: The developer-supplied objective and defaults used to launch a simple exploratory run through the orchestrator.
- **Trace Summary View**: The readable representation of a stored execution trace, including step order, recovery events, and terminal outcome.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria tied to working outcomes.
  Include execution quality, bounded termination, or inspectability metrics when relevant.
  These must stay technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: New contributors can reach a first successful demo run from a documented local checkout in under 5 minutes without reading test files.
- **SC-002**: In validation runs of the demo command, 100% of runs display at least one visible step transition and end in an explicit terminal outcome.
- **SC-003**: In validation runs designed to exercise recovery, developers can identify the retry or replanning path from terminal output alone in under 2 minutes.
- **SC-004**: In trace inspection exercises, developers can identify the executed step order and final terminal reason from the inspection command output in under 2 minutes for at least 90% of sampled runs.

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
  Assumptions must reduce ambiguity without expanding scope.
-->

- Developers will use the feature from a local repository checkout on a workstation where the documented local prerequisites are already available.
- The first release may rely on deterministic built-in execution behavior rather than external providers so the command experience remains stable and easy to debug.
- Existing orchestrator core behavior, trace persistence, and bounded recovery semantics remain the underlying execution model for this feature.
- Canon integration, approval flows, and durable governance records remain outside the scope of this slice even if they are part of the longer-term architecture.
