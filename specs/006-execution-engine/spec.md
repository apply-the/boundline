# Feature Specification: Execution Engine (Code Delivery)

**Feature Branch**: `006-execution-engine`  
**Created**: 2026-04-25  
**Status**: Draft  
**Input**: User description: "Expand Boundline beyond the current fixture-backed red-to-green slice so it can perform real code delivery: read and write workspace files, generate diffs, run validation hooks, and iterate generate -> run -> fix -> retry within bounded execution limits while preserving explicit traces and terminal outcomes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Deliver a bounded code change (Priority: P1)

As a developer with an active Boundline session, I want Boundline to perform a real workspace delivery loop so it can read the current code, modify files, run validation, and stop only when the task succeeds or reaches an explicit terminal outcome.

**Why this priority**: Without real workspace mutation and validation, Boundline still stops at orchestration and does not deliver working code.

**Independent Test**: Start a session for a real failing workspace slice, run the delivery path to terminal completion, and verify that Boundline updates files inside the workspace, records the resulting change set, runs validation, and leaves the workspace in either a passing or explicitly failed terminal state.

**Acceptance Scenarios**:

1. **Given** an active session with a captured delivery goal and a reachable workspace, **When** the user plans and runs the task, **Then** Boundline reads the current workspace state, applies bounded file changes for the task, runs the configured validation hook, and records the outcome as part of the same execution trace.
2. **Given** an active session with a delivery task that reaches passing validation, **When** the final validation succeeds, **Then** Boundline stops in a succeeded terminal state and exposes the files changed, validation result, and trace location without requiring manual reconstruction.
3. **Given** an active session with a delivery task that cannot produce a valid change set, **When** Boundline determines there is no credible next change inside execution limits, **Then** Boundline stops in an explicit failed or exhausted terminal state and preserves the failure evidence.

---

### User Story 2 - Recover inside the validation loop (Priority: P2)

As a developer running an execution attempt that fails validation, I want Boundline to retry or regenerate within explicit limits so it can continue the same delivery task without losing the current context or silently looping forever.

**Why this priority**: Real delivery is only credible if failed validation stays inside a bounded fix loop instead of collapsing into a one-shot attempt.

**Independent Test**: Run a delivery task where the first generated change fails validation, then verify that Boundline records the failure, preserves the task context, attempts a bounded retry or replacement change, and either succeeds or terminates explicitly within configured limits.

**Acceptance Scenarios**:

1. **Given** an active delivery task whose first validation run fails, **When** Boundline still has retry or replan budget remaining, **Then** it records the failure, keeps the task in the same execution run, and performs another bounded change attempt against the same workspace goal.
2. **Given** an active delivery task that repeatedly fails validation, **When** Boundline reaches the configured retry, replan, or step limit, **Then** it stops in an explicit exhausted or failed terminal state and preserves the latest change and validation evidence for inspection.

---

### User Story 3 - Inspect delivered output and evidence (Priority: P3)

As a developer reviewing what Boundline changed, I want an inspectable summary of modified files, diff evidence, validation runs, and terminal reasoning so I can judge whether the delivery result is safe to keep or continue from.

**Why this priority**: Once Boundline starts writing code, the output must be inspectable enough for a developer to trust or reject it quickly.

**Independent Test**: Complete or fail a delivery run, then inspect the task and confirm that the surfaced evidence includes touched files, change or diff references, validation outcomes, recovery history, and the final terminal reason.

**Acceptance Scenarios**:

1. **Given** a completed or failed delivery run, **When** the user asks for status, next guidance, or trace inspection, **Then** Boundline reports the latest change attempt, validation outcome, recovery history, and final terminal state using inspectable execution evidence.
2. **Given** a delivery run that touched multiple files, **When** the user inspects the outcome, **Then** Boundline exposes a stable summary of which files changed and where the diff or change evidence can be found.

---

### Edge Cases

- If the workspace path is missing, unreadable, or not writable, Boundline must reject execution before any partial write occurs.
- If a generated change attempts to target files outside the workspace boundary, Boundline must reject the attempt and surface the violation as explicit failure evidence.
- If validation tooling is unavailable, times out, or returns unusable output, Boundline must treat that as a visible failed attempt rather than silently continuing.
- If a change attempt produces no effective file modification, Boundline must not report progress that did not happen and must either retry credibly or terminate explicitly.
- If a previous attempt left partial changes in the workspace, Boundline must preserve traceable evidence of the current workspace state before continuing another attempt.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST execute a bounded delivery task against the active workspace using the current session goal and context.
- **FR-002**: Boundline MUST read the relevant workspace state needed to attempt the requested delivery change before mutating files.
- **FR-003**: Boundline MUST apply file changes only within the addressed workspace boundary.
- **FR-004**: Boundline MUST record the concrete file changes produced by each delivery attempt as inspectable change evidence.
- **FR-005**: Boundline MUST run at least one validation hook for each delivery attempt before declaring success.
- **FR-006**: Boundline MUST preserve the latest validation outcome, including non-success outcomes, as part of task state and trace output.
- **FR-007**: Boundline MUST keep retries, replans, and regenerated change attempts within the configured execution limits for the active task.
- **FR-008**: Boundline MUST stop every delivery run in an explicit succeeded, failed, or exhausted terminal state with a visible reason.
- **FR-009**: Boundline MUST expose status and inspection output that identifies touched files, latest validation result, recovery history, and terminal outcome.
- **FR-010**: Boundline MUST reject execution when the workspace is inaccessible or when an attempt would escape workspace boundaries.
- **FR-011**: Boundline MUST preserve traceable evidence for workspace reads, file writes, validation runs, retries, replans, and terminal completion.
- **FR-012**: Boundline MUST preserve compatibility with the existing session and flow model instead of introducing a separate hidden execution path.
- **FR-013**: Boundline MUST support delivery attempts that update one or more files before running validation.
- **FR-014**: Boundline MUST make the most recent change evidence available after terminal completion so a developer can inspect what Boundline actually delivered.

### Scope Boundaries *(mandatory)*

- **In Scope**: bounded workspace reads and writes, change or diff evidence, validation hook execution, retry or replan loops inside execution limits, terminal runtime error handling, and inspectable delivery traces.
- **Out of Scope**: full CI or deployment pipelines, multi-agent review or voting, provider councils, distributed execution, long-term memory beyond task scope, adaptive workflow generation, and background automation outside the active run.

### Key Entities *(include if feature involves data)*

- **Delivery Attempt**: The bounded unit of work that reads the workspace, applies one coherent change set, runs validation, and records success or failure before a retry, replan, or terminal stop.
- **Workspace Change Set**: The inspectable record of file mutations produced by a delivery attempt, including touched files and stable change evidence for later inspection.
- **Validation Record**: The per-attempt outcome of a validation hook, including what was run, whether it passed, and the evidence needed to decide whether to retry, replan, or stop.
- **Execution Capability Profile**: The declared set of workspace and validation actions that Boundline can perform for the current delivery run, including the boundaries that prevent out-of-scope mutation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative execution-engine validation scenarios, Boundline can complete a real code-delivery task end to end by applying file changes and reaching passing validation without manual file editing during the run.
- **SC-002**: 100% of execution-engine validation runs stop in an explicit succeeded, failed, or exhausted terminal state within configured limits.
- **SC-003**: Developers can identify the files changed, latest validation result, and terminal reason from status or inspect output in under 30 seconds.
- **SC-004**: In representative failure scenarios, 100% of retries or replans remain inside the same bounded delivery run unless the session is explicitly reset.
- **SC-005**: No execution-engine validation scenario writes outside the addressed workspace boundary.

## Assumptions

- Users run Boundline against a local workspace that already contains the code and validation command needed for the targeted slice.
- The initial execution-engine release can reuse the existing session model, flow model, and trace store instead of introducing a separate persistence system.
- The first delivery slice may rely on a bounded set of built-in execution capabilities rather than arbitrary external tools.
- Review and voting remain deferred to the next roadmap slice, so this feature only needs one explicit delivery path with inspectable evidence.
