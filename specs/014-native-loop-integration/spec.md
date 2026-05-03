# Feature Specification: Native Loop Integration

**Feature Branch**: `014-native-loop-integration`  
**Created**: 2026-04-29  
**Status**: Draft  
**Input**: User description: "Wire GoalPlan and inferred flow into session planning, route session run to DecisionLoop when a goal plan exists, and replace synthetic decision dispatch with real adapters and persisted decisions on the CLI path"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Session Planning Uses Goal Plan (Priority: P1)

A developer starts a session, captures a goal, and runs planning. Instead of producing only a fixture-oriented task snapshot, Boundline derives and persists a bounded goal plan from the goal and workspace state, proposes an inferred flow, and records the operator's lightweight confirmation so the session carries the plan forward into execution.

**Why this priority**: The session-native path cannot become the primary product story until `plan` produces session-owned planning state instead of stopping at fixture-era planning semantics.

**Independent Test**: Can be tested by running the real session CLI flow through `start`, `capture`, and `plan` on a workspace without an explicit execution profile and verifying that the session stores a goal plan plus either a confirmed inferred flow or an explicit no-flow decision.

**Acceptance Scenarios**:

1. **Given** an active session with a captured goal and no execution profile, **When** the developer runs planning, **Then** Boundline stores a bounded goal plan in the session and exposes the derived tasks in session-facing output.
2. **Given** an active session with a captured goal whose wording implies a bug fix, **When** planning runs, **Then** Boundline proposes the matching flow, records the confirmation outcome in session state, and keeps the session resumable.
3. **Given** an active session where planning cannot infer a credible flow, **When** planning completes, **Then** Boundline still persists the goal plan and records that execution will proceed without flow constraints.

---

### User Story 2 - Session Run Uses Decision Loop By Default (Priority: P2)

A developer runs execution on a planned session. If the session already contains a goal plan, Boundline executes through the bounded decision loop and only falls back to the fixture compatibility path when the operator is explicitly using a declarative execution profile. The primary `run` path therefore follows session-native state instead of implicit fixture defaults.

**Why this priority**: This is the behavioral switch that makes the new planning model matter. Without it, GoalPlan remains metadata while the real runtime still behaves like the old product.

**Independent Test**: Can be tested by driving the CLI through `start`, `capture`, `plan`, `run`, and `inspect` and verifying that a planned session uses the decision loop, while an explicitly declarative fixture profile still follows the compatibility path.

**Acceptance Scenarios**:

1. **Given** an active session with a persisted goal plan, **When** the developer runs execution, **Then** Boundline chooses the decision loop path and emits decision-oriented trace output instead of fixture-only step playback.
2. **Given** a workspace with an explicit declarative execution profile and no goal plan, **When** the developer runs execution, **Then** Boundline uses the compatibility path and preserves the existing fixture-oriented behavior.
3. **Given** a workspace that contains both a goal plan and a declarative execution profile, **When** the developer runs execution without an explicit compatibility opt-in, **Then** the goal-plan path takes precedence.

---

### User Story 3 - Real Adapter-Backed Decisions Are Persisted (Priority: P3)

When Boundline executes the decision loop on the session-native CLI path, each decision is dispatched through real runtime adapters and the chosen decision sequence is persisted back into session state and traces. Developers can inspect what was observed, what decision was chosen, which tool or adapter ran, and why execution succeeded, failed, replanned, or exhausted its budget.

**Why this priority**: This closes the trust gap identified in the review. The loop is only credible if it uses the real adapter harness and records the resulting decisions in durable session-owned state.

**Independent Test**: Can be tested end-to-end from the CLI by executing a real session-native run on a workspace that requires file reads, file writes, and validation, then verifying that session state and trace inspection show persisted decisions with structured tool results.

**Acceptance Scenarios**:

1. **Given** a planned session whose next action requires reading files and running validation, **When** execution runs, **Then** Boundline dispatches those actions through the adapter harness and records structured results for each decision.
2. **Given** a decision whose verification fails, **When** the loop continues, **Then** the failed decision remains inspectable in session state and the follow-up recovery decision references the failure evidence.
3. **Given** a session-native run that reaches a terminal state, **When** the developer inspects the session or trace, **Then** the persisted decision list and tool evidence explain how the run terminated.

### Edge Cases

- What happens when planning derives a goal plan but the operator declines the proposed flow? The goal plan remains valid, the session records that no confirmed flow is active, and execution continues without implicit flow constraints.
- What happens when execution is requested on a session that has a captured goal but no persisted goal plan? Boundline returns an explicit error or remediation message telling the operator to plan first instead of silently falling back to fixture behavior.
- What happens when the decision loop selects an action but no registered adapter can credibly execute it? Boundline records a failed decision with adapter-unavailable evidence and terminates or replans explicitly.
- What happens when decision execution reaches configured step limits before all planned work is resolved? Boundline terminates in an explicit exhaustion state and preserves the accumulated decisions in session state and trace output.
- What happens when a compatibility profile is present but the operator intends to use the session-native path? Session-owned planning state takes precedence unless the operator explicitly chooses the compatibility path.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist a bounded goal plan in active session state when planning succeeds on a captured goal.
- **FR-002**: System MUST derive planning state from the captured goal, workspace signals, and available Canon artifacts before execution begins on the session-native path.
- **FR-003**: System MUST propose an inferred flow during planning, preserve the operator's confirmation outcome, and allow the session to continue without a confirmed flow when inference is declined or unavailable.
- **FR-004**: System MUST route session execution to the decision loop whenever a persisted goal plan is present, unless the operator explicitly selects the declarative compatibility path.
- **FR-005**: System MUST preserve the existing declarative fixture behavior when no goal plan exists and the operator is using an explicit execution profile.
- **FR-006**: System MUST dispatch decision-loop actions through the runtime adapter harness used for concrete workspace operations instead of relying on synthetic in-loop stand-ins.
- **FR-007**: System MUST persist each chosen decision into session state together with its status, evidence inputs, and structured execution result.
- **FR-008**: System MUST emit trace events that allow inspection of goal-plan creation, flow inference outcome, decision creation, dispatch, verification, failure, recovery, and terminal state.
- **FR-009**: System MUST preserve failure evidence when a decision fails so that the next bounded recovery action remains inspectable and reproducible.
- **FR-010**: System MUST stop execution in an explicit terminal state when no credible next action exists, an adapter cannot execute the required action, or configured execution limits are reached.
- **FR-011**: System MUST let the session CLI present the session-native path as the default operator journey for planned work.
- **FR-012**: System MUST support end-to-end CLI validation that demonstrates the session-native path without requiring `init` or a declarative execution profile.

### Scope Boundaries *(mandatory)*

- **In Scope**: session-owned GoalPlan persistence, inferred-flow confirmation, session run routing to DecisionLoop, adapter-backed decision execution, persisted decision history, CLI-visible traceability, fixture compatibility as explicit fallback
- **Out of Scope**: new flow families, provider-routing expansion, parallel execution, multi-workspace orchestration changes, Canon governance redesign, UI work, new template systems, deployment automation, and long-term memory beyond the existing session model

### Key Entities *(include if feature involves data)*

- **Session Planning Record**: The planning portion of active session state that binds the captured goal, the persisted goal plan, the flow confirmation outcome, and the routing decision that determines whether the session follows the native loop or the compatibility path.
- **Persisted Decision History**: The ordered set of decisions chosen during session-native execution, including target, rationale, expected outcome, evidence inputs, execution result, and terminal or recovery status.
- **Compatibility Routing State**: The explicit execution-selection state that decides whether `run` follows the decision loop or the fixture fallback based on goal-plan presence and operator intent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can complete `start → capture → plan → run → inspect` on a workspace without an execution profile and observe session-owned planning plus decision-oriented execution from the real CLI path.
- **SC-002**: 100% of session-native runs with a persisted goal plan terminate through an explicit terminal state and record at least one persisted decision when work begins.
- **SC-003**: Developers can determine from session state and trace output whether `run` used the native loop or the compatibility path in under 2 minutes.
- **SC-004**: Planning records a bounded goal plan and the flow confirmation outcome for all supported inference cases without requiring a separate flow-selection command in the common path.
- **SC-005**: Compatibility runs that intentionally use explicit declarative execution profiles continue to pass their existing regression scenarios unchanged.

## Assumptions

- Active sessions remain persisted in the existing workspace-local session file and do not require a separate storage backend for this slice.
- The current adapter and registry harness is sufficient to back decision-loop actions once routing is moved onto the real session path.
- Flow inference remains lightweight and operator-confirmed; this slice does not introduce silent auto-execution based on inferred flow alone.
- The compatibility path remains necessary for declarative execution profiles and existing regression coverage.
