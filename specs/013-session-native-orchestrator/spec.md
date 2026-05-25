# Feature Specification: Session-Native Orchestrator

**Feature Branch**: `013-session-native-orchestrator`  
**Created**: 2026-04-29  
**Status**: Draft  
**Input**: User description: "Realign Boundline from fixture-driven execution to a bounded observe-decide-act-verify loop with explicit decision objects, inferred flows, goal-derived planning, and tool-driven execution"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bounded Decision Loop (Priority: P1)

A developer runs `boundline run` on an active session. Instead of replaying static manifest-declared attempts, Boundline enters a bounded loop: it observes the current workspace state and accumulated evidence, selects the next bounded action as an explicit decision object, dispatches that action through an agent or tool adapter, verifies the outcome, and updates session context. The loop continues until a terminal condition is reached (success, exhaustion, or configured limit). Each decision is persisted in the session trace and can be inspected later through `boundline inspect`.

**Why this priority**: This is the core product shift. Without an explicit observe→decide→act→verify→update loop, Boundline remains a static plan replayer. Every other story in this spec depends on the runtime owning next-action selection from live state.

**Independent Test**: Can be tested by running `boundline run` on a session with a recorded goal and verifying that the engine produces a sequence of typed, inspectable decision objects in the trace, each with type, target, rationale, expected outcome, and evidence inputs.

**Acceptance Scenarios**:

1. **Given** a session with a recorded goal and workspace context, **When** `boundline run` is invoked, **Then** the engine enters an observe→decide→act→verify→update loop, produces at least one decision object in the trace, and terminates in an explicit terminal state (success, failure, or limit-reached).
2. **Given** a running loop where the previous action's verification fails, **When** the engine selects the next decision, **Then** the decision references the failed verification evidence and selects a bounded recovery action (fix or replan), and the recovery decision is recorded in the trace.
3. **Given** a session that has reached its configured maximum step count, **When** the engine evaluates whether to continue, **Then** execution stops with an explicit exhaustion terminal state and a trace entry documenting the limit and accumulated evidence.

---

### User Story 2 - Goal-Derived Planning (Priority: P2)

A developer runs `boundline plan` on an active session that has a recorded goal. Instead of requiring a pre-authored execution profile or init template, Boundline derives an initial bounded task draft from the goal text, the current workspace state (file tree, existing config, language/framework signals), collected documents (if any were captured), and available Canon artifacts (if the workspace has `.canon/` with governed outputs). The resulting plan is shown to the developer for confirmation before execution begins.

**Why this priority**: Shifting the planning input from static templates to goal + workspace + documents is the second-largest product change. The decision loop (US1) needs a plan to execute against, and that plan must come from real context rather than a predeclared manifest.

**Independent Test**: Can be tested by running `boundline start` then `boundline goal --goal "fix the broken auth middleware"` then `boundline plan`, and verifying that the session produces a bounded task list derived from workspace state without requiring `boundline init` or a pre-existing execution profile.

**Acceptance Scenarios**:

1. **Given** a session with a recorded goal and a Rust workspace, **When** `boundline plan` is invoked, **Then** Boundline produces a bounded task draft that references files and structures actually present in the workspace, and persists the plan in the session state.
2. **Given** a session with a recorded goal and a workspace containing `.canon/` artifacts, **When** `boundline plan` is invoked, **Then** the plan includes references to relevant Canon artifacts as evidence inputs, and the plan is bounded (limited number of tasks, each with an explicit expected outcome).
3. **Given** a session where `boundline plan` is invoked without a prior recorded goal, **When** the command runs, **Then** it returns an explicit error indicating that a goal must be captured first.

---

### User Story 3 - Inferred Flow with Lightweight Confirmation (Priority: P3)

A developer runs `boundline plan` on an active session. Instead of requiring explicit flow selection via `boundline select-flow`, Boundline infers the most appropriate flow from the goal text and workspace signals (e.g., "fix" keywords map to `bug-fix`, presence of failing tests suggests `bug-fix`, broad changes suggest `delivery`). The inferred flow is shown to the developer with a one-line confirmation prompt. The developer can accept, override, or skip flow entirely.

**Why this priority**: Flow inference removes a manual selection step that breaks the session-native feel. It depends on having a plan (US2) and the loop uses flow constraints (US1), so it is the natural third step.

**Independent Test**: Can be tested by capturing a goal containing "fix" and verifying that `boundline plan` proposes the `bug-fix` flow with a confirmation prompt, and that accepting it constrains subsequent decisions to the allowed decision families for that flow.

**Acceptance Scenarios**:

1. **Given** a session with a recorded goal containing "fix the failing test in auth.rs", **When** `boundline plan` is invoked, **Then** Boundline proposes the `bug-fix` flow with an explicit confirmation prompt showing the inferred flow and the reason for inference.
2. **Given** an inferred flow proposal, **When** the developer overrides with `--flow delivery`, **Then** the session uses the `delivery` flow instead and records the override in the session state.
3. **Given** an inferred flow proposal, **When** the developer skips flow entirely with `--no-flow`, **Then** the session proceeds without flow constraints and the decision loop operates with the full set of decision types.

---

### User Story 4 - Flow as Decision Policy (Priority: P4)

When a flow is active on a session, the decision loop uses flow stage metadata to constrain which decision types are allowed at each stage. For example, during the `investigate` stage of a `bug-fix` flow, only `analyze` decisions are permitted; during `implement`, `code` and `fix` decisions are allowed; during `verify`, `test` and `replan` decisions are allowed. Stage transitions occur only after verifiable outcomes. The flow acts as a bounded policy surface over the decision loop, not a rigid script.

**Why this priority**: This connects flow and the decision loop so that flow becomes meaningful constraints rather than ornamental metadata. It depends on both the decision loop (US1) and flow inference (US3).

**Independent Test**: Can be tested by running a session with the `bug-fix` flow active and verifying that the engine rejects a `code` decision during the `investigate` stage, and that stage transitions require verification evidence.

**Acceptance Scenarios**:

1. **Given** a session running under the `bug-fix` flow in the `investigate` stage, **When** the engine selects the next decision, **Then** only `analyze`-type decisions are produced.
2. **Given** a session in the `implement` stage, **When** the engine produces a `code` or `fix` decision and the action's verification succeeds, **Then** the flow transitions to the `verify` stage, and the transition is recorded in the session trace.
3. **Given** a session in the `verify` stage where all tests pass, **When** the engine evaluates the next decision, **Then** the flow transitions to terminal success and the session reaches a completed state.

---

### User Story 5 - Tool-Driven Execution (Priority: P5)

The decision loop dispatches actions through tool adapters that perform concrete workspace operations: reading files, writing or patching files, running tests and validation commands, and inspecting diffs and execution output. Each tool invocation produces structured output that feeds back into the observe phase of the next loop iteration. If the loop is not grounded in tool use, Boundline regresses into a text-emitting planner.

**Why this priority**: Tool-driven execution is what makes the loop real. Without concrete workspace mutation and verification, the decision loop is only generating plans. This depends on the loop (US1) being in place.

**Independent Test**: Can be tested by running `boundline run` on a session whose plan includes a file-modification task, and verifying that the engine invokes a tool adapter to write the file, runs a verification command, and records the tool output as evidence in the next decision's inputs.

**Acceptance Scenarios**:

1. **Given** a decision of type `code` with target `src/lib.rs`, **When** the engine dispatches the action, **Then** a tool adapter writes or patches the target file, and the tool output (diff, exit code, stderr) is persisted as evidence in the session trace.
2. **Given** a decision of type `test` with target `cargo test`, **When** the engine dispatches the action, **Then** a tool adapter runs the command, captures stdout/stderr and exit code, and feeds the result back as evidence for the next loop iteration.
3. **Given** a tool invocation that fails (non-zero exit code or write error), **When** the engine enters the next observe phase, **Then** the failure evidence is included in the next decision's evidence inputs and the decision explicitly references the failure.

---

### User Story 6 - Fixture as Compatibility Layer (Priority: P6)

The existing `fixture.rs` execution path continues to work for explicitly declarative workflows and test-oriented execution. Developers who have authored `.boundline/execution.json` manifests can still run them. However, the fixture path is no longer the default product entry point. `boundline run` without an explicit execution profile uses the new goal-derived plan and decision loop. Fixture becomes a low-level helper module for workspace mutation primitives and a compatibility fallback.

**Why this priority**: Preserving backward compatibility ensures existing workflows and tests are not broken by the realignment. This is lower priority because it requires no new capability, only correct routing between the new and old paths.

**Independent Test**: Can be tested by verifying that `boundline run` with an existing `.boundline/execution.json` still executes the fixture path, while `boundline run` on a session with only a recorded goal uses the new decision loop.

**Acceptance Scenarios**:

1. **Given** a workspace with an existing `.boundline/execution.json` and no active session goal, **When** `boundline run` is invoked, **Then** the fixture execution path is used and behavior matches the current v0.12.0 output.
2. **Given** a workspace with an active session that has a recorded goal and no execution profile, **When** `boundline run` is invoked, **Then** the new decision loop is used, not the fixture path.
3. **Given** a workspace with both an active session goal and an existing execution profile, **When** `boundline run` is invoked, **Then** the session-native path takes precedence and the execution profile is ignored unless the developer explicitly opts into it with `--profile`.

### Edge Cases

- What happens when the decision loop reaches the configured maximum step count without achieving the goal? Execution terminates with an explicit `Exhausted` terminal state, and the trace records all accumulated evidence and the step count.
- What happens when the observe phase finds no actionable workspace state (e.g., empty workspace, no files matching the goal)? The engine produces a terminal decision of type `NoActionableState` with a rationale explaining what was observed and why no action could be selected.
- What happens when flow inference cannot determine a flow from the goal text? The engine defaults to no-flow mode (unconstrained decisions) and logs a warning suggesting the developer use `--flow` to set one explicitly.
- What happens when a tool adapter is unavailable at dispatch time? The decision is marked as failed with a `ToolUnavailable` error, the evidence is recorded, and the loop selects a recovery decision (replan or skip).
- What happens when a decision's verification produces ambiguous results (e.g., some tests pass, some fail)? The engine records the partial evidence and selects a bounded follow-up decision that targets only the failing subset.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST implement a bounded execution loop with explicit phases: observe workspace state, decide next action, act through adapter dispatch, verify the outcome, update session context.
- **FR-002**: System MUST represent each next-action selection as a first-class decision object with fields: type (analyze, code, test, fix, replan), target (file, test, subsystem), rationale, expected outcome, and evidence inputs.
- **FR-003**: System MUST persist every decision object into the session trace, making the full decision sequence inspectable through `boundline inspect`.
- **FR-004**: System MUST derive initial task plans from goal text, workspace state, and optionally collected documents and Canon artifacts, without requiring a pre-authored execution profile.
- **FR-005**: System MUST infer an appropriate flow from goal text and workspace signals, present it for lightweight confirmation, and allow explicit override or skip.
- **FR-006**: System MUST enforce flow stage constraints on decision types when a flow is active, treating flow as a bounded policy surface rather than a rigid script.
- **FR-007**: System MUST dispatch actions through tool adapters that perform concrete workspace operations (file read, file write/patch, command execution, diff inspection) and feed structured output back as evidence.
- **FR-008**: System MUST apply explicit execution limits (maximum steps, maximum retries per decision) and terminate with an explicit terminal state when limits are reached.
- **FR-009**: System MUST handle decision verification failures by selecting a bounded recovery action (fix or replan) and recording the failure evidence in the next decision's inputs.
- **FR-010**: System MUST preserve backward compatibility with the existing fixture-based execution path when an explicit execution profile is present.
- **FR-011**: System MUST route `boundline run` to the new decision loop when a session has a recorded goal, and to the fixture path only when an explicit execution profile is the sole input.
- **FR-012**: System MUST produce explicit terminal states for all execution outcomes: success, failure, exhaustion, and no-actionable-state.

### Scope Boundaries *(mandatory)*

- **In Scope**: bounded decision loop, decision object model, goal-derived planning, flow inference and confirmation, flow-as-policy over decisions, tool-driven action dispatch, fixture compatibility routing, trace persistence for decision objects
- **Out of Scope**: parallel or concurrent execution, distributed multi-workspace execution, model gateway or provider abstraction, new assistant command packs, UI/UX surfaces, Canon governance deepening beyond current stage overlay, council or voting system changes, new template variants, deployment pipelines, long-term memory beyond session scope

### Key Entities

- **Decision**: The atomic unit of the execution loop. Fields: `id` (UUID), `decision_type` (Analyze, Code, Test, Fix, Replan), `target` (file path, test name, or subsystem identifier), `rationale` (human-readable string), `expected_outcome` (verifiable claim), `evidence_inputs` (list of references to files, traces, failures, docs, Canon artifacts), `status` (Pending, Dispatched, Verified, Failed, Recovered). Lifecycle: created during decide phase → dispatched during act phase → verified or failed during verify phase → context-updated during update phase.
- **GoalPlan**: A bounded task draft derived from goal, workspace, documents, and Canon artifacts. Fields: `tasks` (ordered list of task descriptions with targets and expected outcomes), `source_evidence` (what inputs were used to derive the plan), `flow` (optional inferred or explicit flow). Lifecycle: created during `boundline plan` → consumed by the decision loop during `boundline run` → mutated through replan decisions.
- **FlowPolicy**: A mapping from flow stages to allowed decision types. Fields: `flow_id`, `stages` (ordered list), `allowed_decisions_by_stage` (map from stage name to set of decision types), `transition_conditions` (what evidence triggers a stage transition). Lifecycle: derived from flow metadata when a flow is active → consulted by the engine before each decision selection → transitions recorded in trace.
- **ToolResult**: Structured output from a tool adapter invocation. Fields: `tool_id`, `invocation` (command or operation), `exit_code`, `stdout`, `stderr`, `diff` (optional), `duration`. Lifecycle: produced during act phase → consumed as evidence input in the next observe phase.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can complete a bounded engineering task (e.g., fix a broken test, implement a small function) through `goal → plan → run → inspect` without invoking `boundline init` or authoring an execution profile.
- **SC-002**: 100% of `boundline run` executions terminate in an explicit terminal state (success, failure, exhaustion, or no-actionable-state) within configured step limits.
- **SC-003**: Every decision made during execution is recorded as a typed, inspectable decision object in the session trace, and can be retrieved through `boundline inspect` within 2 seconds.
- **SC-004**: `boundline plan` derives a bounded task draft from goal and workspace state in under 5 seconds for workspaces with up to 1000 files.
- **SC-005**: Existing fixture-based workflows continue to produce identical output when invoked with an explicit execution profile, with zero regressions from the v0.12.0 baseline.
- **SC-006**: Flow inference correctly proposes the appropriate flow for at least 3 canonical goal patterns (bug-fix keywords, broad change keywords, delivery keywords) with lightweight confirmation.

## Assumptions

- Developers have access to the Boundline CLI and a configured workspace with at least one supported runtime (Claude, Codex, Copilot, or Gemini).
- Workspace state (file tree, language indicators, existing config) provides sufficient signal for bounded plan derivation without requiring external services.
- The existing session persistence model (`.boundline/session.json`) and trace store (`.boundline/traces/`) are sufficient for the new decision object model without schema migration.
- Canon artifacts under `.canon/` are readable as file-system inputs when present; no Canon CLI invocation is required during planning.
- The bounded execution loop operates sequentially (one decision at a time) per the constitution's sequential-first design principle.
- Flow stage metadata in `src/domain/flow.rs` can be extended with allowed-decision-type mappings without breaking existing flow definitions.
- [Assumption about data/environment, e.g., "Existing authentication system will be reused"]
- [Dependency on existing system/service, e.g., "Requires access to the existing user profile API"]
