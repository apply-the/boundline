# Feature Specification: Delivery Orchestrator Core

**Feature Branch**: `001-delivery-orchestrator-core`  
**Created**: 2026-04-23  
**Status**: Draft  
**Input**: User description: "Build the first Synod spec around a stateful delivery orchestrator that can coordinate multi-step work, route across agents, manage retries and replanning, and act as the minimal execution brain for later delivery flows."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Execute Bounded Delivery Work (Priority: P1)

As a developer using Synod, I can submit a bounded engineering objective and have Synod carry it through an ordered sequence of steps until the task succeeds or reaches a credible stop condition.

**Why this priority**: This is the minimum value of the orchestrator. Without reliable multi-step execution, Synod cannot move beyond isolated agent or tool calls.

**Independent Test**: Can be fully tested by running a task with at least three ordered steps and confirming Synod completes the task or terminates it cleanly without manual intervention between steps.

**Acceptance Scenarios**:

1. **Given** a bounded task with an initial plan containing analysis, change, and verification work, **When** Synod executes the plan, **Then** it runs the steps in order, preserves shared task context between them, and reaches a terminal status.
2. **Given** a task whose goal is satisfied before the configured limits are reached, **When** the final validating step completes, **Then** Synod stops immediately in a success state and records the outcome in the execution trace.

---

### User Story 2 - Recover From Failed Steps (Priority: P2)

As a Synod maintainer, I can rely on the orchestrator to respond to failed steps with bounded retries or bounded replanning so recoverable problems do not cause immediate task loss.

**Why this priority**: Delivery work encounters failed commands, unusable outputs, and invalidated plans. Controlled recovery is required for the orchestrator to be credible in real workflows.

**Independent Test**: Can be fully tested by running a task where one step fails in a recoverable way and confirming Synod either retries or revises the remaining plan while preserving prior context and history.

**Acceptance Scenarios**:

1. **Given** a task with retry budget remaining, **When** a step fails in a recoverable way, **Then** Synod retries the step or selects a replanning path, and the prior task history remains available to subsequent steps.
2. **Given** a task whose current plan is no longer viable after new evidence is observed, **When** Synod evaluates the result, **Then** it can replace or extend the remaining plan within configured limits and continue execution from the revised path.
3. **Given** a task that exhausts its retry or replanning budget, **When** no credible next action remains, **Then** Synod terminates in a non-success terminal state and records why recovery stopped.

---

### User Story 3 - Inspect Execution History (Priority: P3)

As an operator or platform owner, I can inspect the full execution history of a task so I can understand how Synod progressed, where it failed, and why it stopped.

**Why this priority**: Inspectability is necessary for debugging, trust, and future governance features. Without it, task failures become opaque and hard to improve.

**Independent Test**: Can be fully tested by reviewing the recorded trace from both a successful run and a failed run and confirming the step sequence, retries, replanning events, and terminal outcome are understandable without rerunning the task.

**Acceptance Scenarios**:

1. **Given** a completed task run, **When** a developer inspects its trace, **Then** they can see the ordered steps, each step's status, the relevant inputs and outputs, and the terminal outcome.
2. **Given** a task that retried or replanned, **When** an operator inspects its trace, **Then** they can identify which recovery actions occurred and what evidence caused those decisions.

### Edge Cases

- A task starts with no executable first step or an invalid current step position.
- A step references an agent or tool name that is not registered for the current run.
- A step completes with unusable or incomplete output that cannot safely update shared context.
- A task reaches its maximum step count, retry budget, or replanning budget at the same time that another stop condition is detected.
- A step changes task context in a way that invalidates all remaining planned steps.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST represent every user-submitted objective as a bounded task with a unique identity, a stated goal, task input, execution constraints, and a current status.
- **FR-002**: System MUST maintain a session-scoped task context for each task run that includes a workspace reference, runtime constraints, accumulated state, prior step history, and the last known result.
- **FR-003**: System MUST make the current task context available to every step executed within the same task run.
- **FR-004**: System MUST create an initial ordered plan before execution begins, and that plan MUST be sufficient to start task progress.
- **FR-005**: System MUST represent executable work as ordered steps and support at least agent steps, tool steps, and decision steps.
- **FR-006**: System MUST store, for each step, a unique identity, step type, execution input, current status, and either output details or error details.
- **FR-007**: System MUST execute tasks through one central sequential loop that selects the current step, executes it, updates task context, evaluates recovery and stop conditions, and advances at most one step at a time.
- **FR-008**: System MUST provide a registry of named agents that the orchestrator can invoke during agent steps.
- **FR-009**: System MUST provide a registry of named tools that the orchestrator can invoke during tool steps.
- **FR-010**: System MUST support bounded retries for recoverable step failures and enforce per-task retry limits so a task cannot retry indefinitely.
- **FR-011**: System MUST support bounded replanning when new evidence makes the remaining plan non-credible, and replanning MUST preserve completed step history and current task context.
- **FR-012**: System MUST distinguish recoverable failures from unrecoverable failures so that retry, replan, or termination decisions are explicit and inspectable.
- **FR-013**: System MUST allow later steps to consume state changes or outputs produced by earlier steps in the same task run.
- **FR-014**: System MUST stop every task in a clear terminal status when the goal is satisfied, further progress is no longer credible, configured step limits are exceeded, recovery budgets are exhausted, or an unrecoverable error occurs.
- **FR-015**: System MUST apply a deterministic precedence order when multiple terminal conditions are true at the same time and MUST record which condition ended the task.
- **FR-016**: System MUST record an execution trace for every task run that includes ordered step history, step inputs, step outputs, step status changes, retry attempts, replanning events, and the terminal outcome.
- **FR-017**: System MUST preserve the execution trace for both successful and non-successful runs until it can be inspected by operators or downstream systems.
- **FR-018**: System MUST allow configuration of runtime limits for maximum steps, maximum retries, and maximum replanning attempts, and MUST apply documented default limits when task-specific overrides are absent.
- **FR-019**: System MUST terminate safely rather than stalling when the current plan contains no credible executable next step.

### Scope Boundaries

- This feature covers sequential orchestration only and does not include parallel execution or graph scheduling.
- This feature does not include multi-agent voting, council behavior, or advanced provider-routing policies.
- This feature does not include persistent memory shared across unrelated tasks.
- This feature does not include human approval gates, CI/CD orchestration, or domain-specific delivery flows built on top of the orchestrator.
- This feature does not require governance artifact persistence or external governance runtime integration.

### Key Entities *(include if feature involves data)*

- **Task**: A bounded delivery objective with identity, goal, input, execution limits, current status, and a reference to its active plan and task context.
- **Task Context**: The mutable, session-scoped state shared across steps in a single task run, including workspace reference, runtime constraints, accumulated state, prior results, and execution history.
- **Plan**: The ordered set of remaining and completed steps for a task, including the current step position and any plan revisions created during replanning.
- **Step**: One executable unit of work with a type, input, status, output or error details, and optional routing metadata that identifies the named agent or tool to invoke.
- **Execution Trace**: The inspectable history of a task run, including step-by-step activity, context-relevant outcomes, retry attempts, replanning events, and the final terminal state.
- **Execution Endpoint**: A named capability available to the orchestrator, represented either as an agent or a tool that can be selected by steps during execution.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In validation scenarios containing at least three ordered steps, Synod completes the task or reaches an explicit terminal status without manual intervention between steps in 100% of runs.
- **SC-002**: In validation scenarios where a later step depends on an earlier result, later steps can access the required prior context in 100% of successful and non-successful runs.
- **SC-003**: In validation scenarios with recoverable failures, Synod retries or replans within configured limits and reaches a terminal status without exceeding the configured maximum step count in 100% of runs.
- **SC-004**: In validation scenarios that exhaust step, retry, or replanning limits, Synod stops within one additional decision cycle and records the exhaustion reason in 100% of runs.
- **SC-005**: In review exercises using sampled task traces, operators can identify the executed step order, each retry or replanning event, and the final outcome within 5 minutes for at least 90% of sampled runs.

## Assumptions

- The first release targets bounded engineering tasks executed within one active workspace and one task session at a time.
- A minimal catalog of named agents and named tools is available when orchestration begins, even if the available catalog is small.
- Default runtime limits are defined by the platform and may be overridden for individual task runs within allowed bounds.
- Later presentation surfaces for execution traces may vary, but the trace itself must be inspectable by developers or downstream systems in this release.
- Human approvals, long-running background workflows, and cross-task memory sharing remain outside the scope of this first orchestration core.
