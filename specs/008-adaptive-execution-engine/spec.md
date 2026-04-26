# Feature Specification: Adaptive Execution Engine

**Feature Branch**: `008-adaptive-execution-engine`  
**Created**: 2026-04-26  
**Status**: Draft  
**Input**: User description: "Expand Synod beyond fixed pre-authored delivery attempts so it can identify the relevant workspace slice for a delivery goal, choose bounded code changes from the current repository state, validate them, adapt its next attempt when validation fails, and preserve explicit evidence for every decision and terminal outcome."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Select And Change The Relevant Workspace Slice (Priority: P1)

As a developer using Synod to deliver code, I want Synod to identify the most relevant bounded part of the workspace for my goal so it can attempt a real delivery change even when no fixed attempt list has been authored in advance.

**Why this priority**: The current execution slice still depends on pre-authored attempts. Synod does not become a credible delivery engine until it can decide where to work from the current workspace and goal.

**Independent Test**: Start a delivery run for a real workspace slice that has no pre-authored change sequence, then verify that Synod selects a bounded set of relevant files, applies one coherent change attempt, runs validation, and stops in either a succeeded or explicit non-success terminal state.

**Acceptance Scenarios**:

1. **Given** an active session with a delivery goal and a reachable workspace, **When** Synod begins execution without a fixed attempt list, **Then** it identifies a bounded relevant workspace slice, performs one coherent change attempt against that slice, and records the selected files and resulting validation outcome in the same run.
2. **Given** a delivery goal that could plausibly touch multiple files, **When** Synod chooses where to work first, **Then** it preserves explicit evidence of the selected workspace slice and leaves unselected files untouched for that attempt.
3. **Given** a delivery goal for which Synod cannot identify a credible bounded workspace slice or next change, **When** it reaches that decision point, **Then** it stops in an explicit failed or exhausted terminal state and records why no credible next step existed.

---

### User Story 2 - Adapt After Failed Validation (Priority: P2)

As a developer running a delivery task that does not pass on the first attempt, I want Synod to adapt its next bounded attempt using the failure evidence so it can continue credibly without silently looping or restarting from scratch.

**Why this priority**: Adaptive delivery only adds value if failed validation changes what Synod does next. Otherwise the system just repeats an untrusted attempt pattern.

**Independent Test**: Run a delivery task where the first attempt fails validation, then verify that Synod preserves the failure evidence, changes the next bounded attempt credibly, and either succeeds or terminates explicitly within configured limits.

**Acceptance Scenarios**:

1. **Given** a delivery attempt that fails validation and remaining execution budget still exists, **When** Synod continues the run, **Then** it uses the failure evidence to narrow, broaden, or replace the next bounded change attempt without losing prior attempt history.
2. **Given** a previous attempt already tested the same credible change path, **When** no materially different next action exists, **Then** Synod terminates explicitly instead of repeating the same failed attempt indefinitely.
3. **Given** repeated adaptive attempts that consume the configured execution limits, **When** no validated delivery result is reached, **Then** Synod stops in an explicit exhausted or failed terminal state and preserves the latest failure evidence.

---

### User Story 3 - Inspect Adaptive Decisions And Output (Priority: P3)

As a developer reviewing what Synod did, I want status and inspection output to show which workspace slice was chosen, how that choice changed over time, and why execution stopped so I can trust or reject the result quickly.

**Why this priority**: Once Synod starts selecting its own delivery scope, inspectability becomes mandatory. Without visible evidence, adaptive execution looks like hidden heuristic behavior.

**Independent Test**: Complete or fail an adaptive delivery run, then verify that status, next guidance, and inspection surfaces expose the selected workspace slice, attempt sequence, validation outcomes, recovery path, and final terminal reason.

**Acceptance Scenarios**:

1. **Given** an adaptive delivery run that succeeds or stops non-successfully, **When** the user asks for status, next guidance, or inspection output, **Then** Synod surfaces the selected workspace slice, attempt sequence, validation outcomes, recovery path, and terminal reason.
2. **Given** an adaptive delivery run where the selected workspace slice changes between attempts, **When** the developer inspects the trace, **Then** Synod shows how the scope changed and why that new bounded path was chosen.

---

### Edge Cases

- If several workspace slices are equally credible and execution limits cannot support exploring them all, Synod must choose one bounded path visibly rather than silently expanding scope.
- If a selected file becomes unreadable or unwritable during the run, Synod must stop or fail the attempt explicitly while preserving the evidence already gathered.
- If a candidate workspace slice includes generated, vendored, or otherwise out-of-scope content, Synod must exclude that content or fail explicitly rather than mutating it silently.
- If validation passes but the attempt produces no meaningful delivery change for the stated goal, Synod must not report success without visible rationale.
- If adaptive execution oscillates between previously tried slices or strategies, Synod must detect the repetition and stop explicitly instead of looping.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Synod MUST support a bounded delivery run that identifies relevant workspace state from the active goal and current workspace context without requiring a fixed pre-authored attempt sequence.
- **FR-002**: Synod MUST represent the selected workspace slice for each attempt as explicit task state rather than as hidden internal behavior.
- **FR-003**: Synod MUST keep each attempt bounded to a limited workspace slice and explicit execution limits.
- **FR-004**: Synod MUST preserve inspectable evidence of why the current workspace slice was selected for an attempt.
- **FR-005**: Synod MUST apply delivery changes only within the addressed workspace boundary and the currently selected bounded slice.
- **FR-006**: Synod MUST run validation for each delivery attempt before declaring success.
- **FR-007**: Synod MUST preserve validation outcomes and attempt history across adaptive attempts in the same run.
- **FR-008**: Synod MUST use failure evidence to adapt the next attempt when a credible alternative path still exists.
- **FR-009**: Synod MUST NOT repeat a materially identical failed attempt unless new evidence or a narrower stated goal makes that retry credible.
- **FR-010**: Synod MUST stop every adaptive delivery run in an explicit succeeded, failed, or exhausted terminal state with a visible reason.
- **FR-011**: Synod MUST expose the selected workspace slice, attempt history, validation outcomes, recovery path, and terminal outcome through user-visible status, next, and inspection surfaces.
- **FR-012**: Synod MUST preserve compatibility with the existing session, flow, and bounded review lifecycle instead of introducing a separate hidden execution path.
- **FR-013**: Synod MUST keep adaptive behavior bounded by explicit attempt, step, and scope limits.
- **FR-014**: Synod MUST reject or explicitly stop execution when no credible workspace slice or next change can be identified.
- **FR-015**: Synod MUST preserve traceable evidence for workspace-slice selection, change attempts, validation results, adaptation decisions, and terminal completion.
- **FR-016**: Synod MUST allow the selected workspace slice to narrow, broaden, or shift between attempts while preserving the lineage between attempts inside the same bounded run.

### Scope Boundaries *(mandatory)*

- **In Scope**: bounded workspace-slice selection, adaptive delivery attempts, validation-driven recovery, explicit non-success termination, inspectable scope-selection evidence, and compatibility with the existing session, flow, and review surfaces.
- **Out of Scope**: open-ended repository-wide exploration, hidden background execution, distributed execution, provider-routing frameworks, governance delegated to Canon, long-term memory beyond task scope, UI or UX work, and deployment pipelines.

### Key Entities *(include if feature involves data)*

- **Workspace Slice**: The bounded subset of the workspace selected for one adaptive attempt, including the files or paths considered relevant to the current delivery goal.
- **Adaptive Attempt**: One coherent delivery cycle that selects a workspace slice, applies a change, runs validation, and records whether the result justifies continuation, adaptation, or termination.
- **Selection Evidence**: The inspectable explanation of why a given workspace slice or next attempt path was chosen at that point in the run.
- **Attempt Lineage**: The explicit relationship between attempts, showing whether the next attempt narrowed, broadened, replaced, or terminated the previous path.
- **Validation Outcome**: The recorded result of the current attempt’s validation step, including the evidence needed to justify success, adaptation, or terminal stop.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative delivery scenarios without pre-authored attempt sequences, Synod can identify a bounded workspace slice and complete at least one end-to-end change-and-validation cycle without manual file selection during the run.
- **SC-002**: 100% of adaptive delivery runs stop in an explicit succeeded, failed, or exhausted terminal state within configured limits.
- **SC-003**: Developers can identify the selected workspace slice, latest validation result, and reason for any scope change from status or inspection output in under 60 seconds.
- **SC-004**: In representative failure scenarios, second and later attempts either materially change the attempted path or terminate explicitly; no adaptive run loops indefinitely on the same failed path.
- **SC-005**: No adaptive delivery run mutates files outside the addressed workspace boundary or outside explicitly allowed bounded scope.

## Assumptions

- Users run Synod against a local workspace that contains enough source context and at least one meaningful validation path for Synod to infer a credible bounded delivery slice.
- The initial adaptive execution release only needs to pursue one bounded delivery path at a time rather than exploring multiple candidate paths concurrently.
- Existing session, flow, trace, and review surfaces remain the primary way developers interact with and inspect this feature.
- Human approval and repository policy remain outside the scope of this slice; the feature is limited to bounded local delivery execution with explicit evidence.
