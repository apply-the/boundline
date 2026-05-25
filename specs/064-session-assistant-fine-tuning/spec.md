# Feature Specification: Session and Assistant Fine-Tuning

**Feature Branch**: `064-session-assistant-fine-tuning`  
**Created**: 2026-05-25  
**Status**: Implemented (Retrospective)  
**Input**: User description: "crea una spec 064 in stile speckit per le feature implementate, relativa al fine tuning"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Readable Session References (Priority: P1)

As an operator running multiple sessions per day, I want session references that are date-ordered and human-readable, so I can quickly identify, compare, and communicate sessions without opaque suffixes.

**Why this priority**: Session references are operator-visible across the full workflow and directly affect traceability and support operations.

**Independent Test**: Create multiple sessions in the same day and verify each generated reference follows `YYYYMMDD-NNN-slug` with a monotonically increasing daily sequence.

**Acceptance Scenarios**:

1. **Given** a new session initialized with a goal slug, **When** Boundline generates a session reference, **Then** it returns `YYYYMMDD-NNN-slug` and preserves slug normalization constraints.
2. **Given** an existing set of same-day sessions, **When** another session is created, **Then** the sequence increments by one and remains zero-padded.
3. **Given** a day boundary change, **When** the first session of the new day is created, **Then** sequence numbering restarts from `001` for the new date prefix.

---

### User Story 2 - Reliable Local Install Refresh (Priority: P2)

As a maintainer testing local CLI changes, I want a single local install script, so I can rebuild and refresh the active Homebrew binary quickly and consistently.

**Why this priority**: Local install friction slows validation loops and increases risk of testing stale binaries.

**Independent Test**: Run the script on macOS with a valid Homebrew installation and verify the target `boundline` binary is updated from a fresh release build.

**Acceptance Scenarios**:

1. **Given** the workspace compiles successfully, **When** the install script runs, **Then** it builds release output and copies the resulting binary into the active Homebrew keg path.
2. **Given** the install script completes, **When** the operator runs the installed `boundline`, **Then** the execution reflects the freshly built local version.

---

### User Story 3 - Two-Button Assistant Routing (Priority: P1)

As an assistant user in Copilot chat, I want a consistent primary and conditional secondary action pattern, so I can advance quickly while still having an explicit refine or inspect option when needed.

**Why this priority**: Next-action UX is part of the main operator loop and affects completion speed and error recovery.

**Independent Test**: Open each updated prompt command and verify the response guidance always presents one primary action and only shows the secondary action when its condition is met.

**Acceptance Scenarios**:

1. **Given** normal progression with no refine condition, **When** the assistant renders next actions, **Then** only the primary action is shown.
2. **Given** a refine, inspect, or reset condition, **When** the assistant renders next actions, **Then** the secondary action is shown with its documented condition.
3. **Given** an emitted `phase_request.assistant_resume_command`, **When** next actions are rendered, **Then** the emitted resume command overrides the primary path.

---

### User Story 4 - Prompt and CLI Behavior Alignment (Priority: P2)

As a maintainer, I want prompt contracts and runtime behavior aligned with corrected tests, so assistant guidance and CLI output semantics remain consistent.

**Why this priority**: Mismatch between tests, prompt contracts, and runtime semantics causes regressions and operator confusion.

**Independent Test**: Run lint and test gates and confirm no assertion or contract mismatch remains for the adjusted behavior.

**Acceptance Scenarios**:

1. **Given** updated runtime semantics for clarification requests, **When** tests execute, **Then** expectations match emitted behavior.
2. **Given** updated prompt routing sections, **When** commands are exercised, **Then** allowed follow-up boundaries remain intact.

## Edge Cases

- Session sequence counting must ignore non-session files and malformed filenames.
- Two sessions created close together must still produce deterministic sequence values.
- Goal prompts with active inline question gates must not hide the required direct question behind action links.
- Status and next prompts must avoid suggesting inspect or goal reset paths when no qualifying condition exists.
- Local install refresh must fail clearly when the Homebrew destination path is unavailable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST generate session references in `YYYYMMDD-NNN-slug` format.
- **FR-002**: The system MUST derive the date prefix from runtime time state using deterministic calendar conversion.
- **FR-003**: The system MUST assign a daily sequence that increments from existing same-date session references.
- **FR-004**: The system MUST keep slug normalization and max-length constraints for session references.
- **FR-005**: The CLI session initialization paths MUST use the same session reference contract.
- **FR-006**: The repository MUST provide a local install script that rebuilds and installs the release binary into the active Homebrew path.
- **FR-007**: The seven Copilot prompt files in scope MUST use a two-action next-step pattern: one always-visible primary and one conditionally-visible secondary action.
- **FR-008**: Prompt routing rules MUST preserve `phase_request.assistant_resume_command` override behavior.
- **FR-009**: Prompt routing rules MUST preserve each command's allowed follow-up safety boundary.
- **FR-010**: Runtime and tests MUST stay aligned for clarification semantics and answer typing.
- **FR-011**: Changes MUST pass formatting, linting, and representative test validation before closeout.

### Scope Boundaries *(mandatory)*

- **In Scope**: session reference readability and sequencing; local install helper script; two-button routing updates for the seven Copilot prompt files; test and semantic alignment required by these updates.
- **Out of Scope**: new orchestration phases; changes to Canon contracts; additional assistant hosts beyond the affected Copilot prompt files; redesign of the full command-pack architecture.

### Key Entities *(include if feature involves data)*

- **Session Reference**: Human-readable identifier with date prefix, daily sequence, and normalized slug.
- **Daily Session Sequence**: Per-day counter used to disambiguate session references.
- **Routing Action Pair**: Prompt-level next-step policy with one primary action and one conditional secondary action.
- **Local Install Refresh Script**: Maintainer utility that rebuilds and replaces the installed local binary.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of newly created session references match `YYYYMMDD-NNN-slug` in representative tests.
- **SC-002**: For same-day session creation scenarios, sequence continuity is preserved with no duplicate `NNN` values in representative tests.
- **SC-003**: All seven in-scope Copilot prompts expose the two-button routing structure with explicit primary and conditional secondary behavior.
- **SC-004**: Linting and tests used for this slice complete cleanly with no failures introduced by these changes.
- **SC-005**: Maintainers can refresh local installation from one script invocation without manual copy steps.

## Assumptions

- Existing workspace session files remain the authoritative source for deriving same-day sequence counts.
- The install helper targets Apple Silicon Homebrew layout in the current maintainer environment.
- Prompt changes are guidance-contract updates and do not require Rust runtime changes by themselves.
- This specification documents and consolidates already-implemented fine-tuning work for traceability.
