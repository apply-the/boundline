# Feature Specification: Runtime Refoundation

**Feature Branch**: `015-runtime-refoundation`  
**Created**: 2026-04-29  
**Status**: Draft  
**Input**: User description: "Refound Boundline around a session-native runtime that derives bounded plans from recorded goals, selects explicit next decisions from live state, treats flow as policy constraints, demotes fixture execution to explicit compatibility, and keeps Canon as a stage-boundary governance input rather than the orchestration brain"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Session-Native Runtime Path (Priority: P1)

A developer starts a session, captures a goal, runs planning, and then runs execution without treating `init` or a declarative execution profile as the normal starting point. Boundline derives a bounded task draft from the recorded goal, current workspace state, collected authored input, and any available Canon artifacts. During execution, Boundline chooses the next bounded action from live evidence rather than replaying a static declaration, and it persists each decision plus the terminal outcome so the developer can inspect what happened.

**Why this priority**: This is the product refoundation itself. If the primary path is still init-first or static-profile-first, Boundline remains misaligned with the intended session-native product story.

**Independent Test**: Can be fully tested by running `goal -> plan -> run -> inspect` in a local workspace that has no pre-authored execution profile and verifying that Boundline derives a bounded task draft, executes at least one bounded decision, and reaches an explicit terminal state with inspectable evidence.

**Acceptance Scenarios**:

1. **Given** an active session with a recorded goal and a usable workspace, **When** the developer runs planning and then execution, **Then** Boundline derives a bounded task draft from current evidence, executes one bounded decision at a time from live state, and records the resulting decisions plus the terminal outcome for inspection.
2. **Given** a running session-native execution where verification fails for a bounded action, **When** Boundline chooses what to do next, **Then** it preserves the failure evidence, selects a bounded recovery or replan action, and keeps the full failure path inspectable instead of silently discarding it.
3. **Given** a running session-native execution that reaches its configured limits or has no credible next action, **When** Boundline evaluates whether to continue, **Then** it stops in an explicit non-success terminal state with remediation cues visible through session output and inspection.

---

### User Story 2 - Flow As Confirmed Policy (Priority: P2)

A developer planning work on the session-native path gets a proposed flow inferred from the recorded goal and current workspace signals. The developer can confirm it, override it, or skip flow constraints entirely. Once confirmed, flow no longer behaves like a rigid script; it acts as a bounded policy surface that constrains which families of decisions are allowed at each stage and requires verifiable outcomes before stage transitions.

**Why this priority**: The runtime cannot become adaptive and inspectable if flow remains either manual busywork or an ornamental label. Flow needs to stay visible, lightweight, and safely bounded.

**Independent Test**: Can be fully tested by planning a bug-fix-shaped goal, verifying that Boundline proposes the expected flow with an explicit reason, confirming or skipping it, and then observing that execution either honors the confirmed policy or blocks silent auto-run when confirmation is still pending.

**Acceptance Scenarios**:

1. **Given** a recorded goal whose wording indicates a bug fix, **When** the developer runs planning, **Then** Boundline proposes the `bug-fix` flow with a visible rationale and allows the developer to confirm, override, or skip it.
2. **Given** a confirmed flow and an active stage, **When** Boundline selects the next bounded action, **Then** only decision families allowed by the current stage may run, and stage transitions occur only after verifiable outcomes are recorded.
3. **Given** a proposed flow that has not been confirmed or skipped, **When** the developer runs execution, **Then** Boundline blocks silent auto-run and explains how to confirm or skip the proposal before execution can continue under that policy.

---

### User Story 3 - Explicit Compatibility And Canon Boundaries (Priority: P3)

Developers who still rely on declarative execution profiles can continue using them, but only as an explicit compatibility path. When both a session-native plan and compatibility artifacts exist, the session-native path remains authoritative unless the operator explicitly chooses compatibility behavior. Canon artifacts may inform planning and stage-boundary governance decisions, but Boundline remains independently executable and does not delegate its per-action runtime choices to Canon.

**Why this priority**: Refoundation cannot break existing compatibility surfaces, but it also cannot leave them as the hidden default. This story keeps backward compatibility without letting the old path define the product.

**Independent Test**: Can be fully tested by comparing two runs in the same workspace family: one using only an explicit compatibility profile, and one using a persisted session-native plan plus the same compatibility artifacts, then verifying that routing, inspection, and terminal reporting clearly distinguish the two paths.

**Acceptance Scenarios**:

1. **Given** a workspace that has only an explicit declarative execution profile, **When** the developer runs execution, **Then** Boundline uses the compatibility path and makes that routing decision visible to the operator.
2. **Given** a workspace that has both a persisted session-native plan and compatibility artifacts, **When** the developer runs execution without explicitly opting into compatibility mode, **Then** Boundline uses the session-native path and reports that route choice through its status and inspection surfaces.
3. **Given** a workspace with available Canon artifacts or stage-boundary governance evidence, **When** Boundline plans or evaluates a stage transition, **Then** it may consume those artifacts as bounded inputs without making Canon the per-action control plane or a prerequisite for core execution.

### Edge Cases

- What happens when execution is requested after `goal` but before planning has produced a bounded task draft? Boundline stops immediately with an explicit remediation message instead of silently falling back to compatibility execution.
- What happens when a proposed flow remains unconfirmed at run time? Boundline does not auto-confirm it; it blocks policy-bound execution until the operator confirms or skips the proposal.
- What happens when the observe phase finds evidence but no credible next bounded action? Boundline terminates with an explicit no-actionable-state outcome and preserves the evidence that led to the stop.
- What happens when a tool or adapter cannot execute the selected decision? Boundline records the failure as evidence, preserves the interrupted decision, and either chooses a bounded recovery action or terminates explicitly.
- What happens when both compatibility inputs and session-native state are present but they imply different routes? Session-native state wins by default unless the operator explicitly selects the compatibility path.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST treat `goal -> plan -> run -> status -> inspect` as the primary operator journey for bounded delivery work.
- **FR-002**: System MUST derive a bounded task draft from recorded goal text, workspace state, collected authored inputs, and available Canon artifacts without requiring a pre-authored execution profile.
- **FR-003**: System MUST select the next bounded action during execution from live runtime state, prior evidence, and current bounded plan rather than replaying static declarations as the default control model.
- **FR-004**: System MUST represent each chosen bounded action as an explicit decision object containing decision family, target, rationale, expected outcome, evidence inputs, execution result, and lifecycle status.
- **FR-005**: System MUST dispatch bounded actions through concrete runtime operations that can read files, write or patch files, run validation commands, and capture execution output as evidence for later decisions.
- **FR-006**: System MUST enforce explicit execution limits, including bounded start conditions, bounded retry or recovery behavior, and explicit terminal states for success, failure, exhaustion, and no-actionable-state outcomes.
- **FR-007**: System MUST preserve failure evidence and keep failed decisions inspectable even when a bounded recovery or replan action follows.
- **FR-008**: System MUST infer a flow when sufficient evidence exists, present the reasoning for that proposal, and require explicit confirmation, override, or skip before the proposal becomes active execution policy.
- **FR-009**: System MUST use confirmed flow state as a bounded policy surface that constrains allowed decision families by stage and requires verifiable outcomes before stage transitions.
- **FR-010**: System MUST preserve an explicit compatibility path for declarative execution profiles and surface which route was used for every run.
- **FR-011**: System MUST remain independently executable without Canon and treat Canon artifacts only as bounded planning or stage-boundary inputs rather than the runtime control plane.
- **FR-012**: System MUST expose bounded task draft summaries, route choice, decision history, failure or recovery evidence, and terminal reasoning through operator-facing status and inspection surfaces.

### Scope Boundaries *(mandatory)*

- **In Scope**: session-native runtime refoundation, bounded task draft derivation from captured context, explicit decision objects, tool-driven execution, flow inference and policy constraints, explicit compatibility routing, stage-boundary Canon inputs, and operator-facing inspectability for routing and recovery.
- **Out of Scope**: new flow families beyond current built-ins, parallel or distributed execution, provider-routing expansion, generalized long-term memory, UI work, deployment automation, Canon-owned orchestration logic, and unbounded review or voting systems.

### Key Entities *(include if feature involves data)*

- **BoundedTaskDraft**: The persisted plan produced after goal capture that summarizes intended bounded work, evidence inputs, expected outcomes, and any proposed or confirmed flow state. It exists before execution begins and remains the session-native starting point for run decisions.
- **RuntimeDecision**: The explicit record of one bounded action chosen during execution, including what Boundline chose to do, why it chose it, what evidence informed it, what action ran, and how the result changed the next step.
- **FlowConstraintState**: The operator-visible flow proposal or confirmed flow policy that determines whether flow is pending, skipped, or actively constraining decision families at a given stage.
- **RoutingOutcome**: The explicit execution-mode state that records whether a run followed the session-native path, the compatibility path, or a blocked path that requires operator remediation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can complete `goal -> plan -> run -> inspect` on representative local workspaces without needing `init` or a pre-authored execution profile for the normal path.
- **SC-002**: 100% of session-native runs terminate in an explicit terminal state within configured execution limits.
- **SC-003**: Developers can identify the chosen route, the last failed or recovered decision, and the terminal reason from recorded session output and inspection surfaces in under 5 minutes.
- **SC-004**: Planning produces a bounded task draft and any flow proposal or skip outcome in under 5 seconds for workspaces with up to 1000 files.
- **SC-005**: Compatibility-mode runs that intentionally use explicit declarative execution profiles remain available and are clearly distinguished from session-native runs.

## Assumptions

- Operators use Boundline in local engineering workspaces where the CLI can read and write repo-local state and invoke the existing runtime adapters.
- Existing workspace-local session and trace files remain the authoritative persistence surfaces for session-native planning and execution history.
- The current built-in flow taxonomy remains the initial policy surface for this slice; defining new flow families is deferred.
- Canon artifacts are available as local bounded inputs when present, and their absence must not prevent core session-native planning or execution.
