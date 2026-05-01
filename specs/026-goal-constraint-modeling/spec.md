# Feature Specification: Goal Negotiation And Constraint Modeling

**Feature Branch**: `026-goal-constraint-modeling`  
**Created**: 2026-05-01  
**Status**: Draft  
**Input**: User description: "Add goal negotiation and constraint modeling before planning locks execution so operators can review acceptance boundaries, scope limits, risk constraints, and surfaced tradeoffs through the session-native story while keeping compatibility behavior explicit. Include release closeout tasks for version bump, impacted docs and changelog, coverage for modified Rust files, clippy cleanup, and cargo fmt."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture A Negotiated Delivery Packet (Priority: P1)

An operator can capture a delivery goal and immediately get one inspectable
negotiation packet that makes the intended outcome, acceptance boundary,
binding constraints, and unresolved clarifications explicit before planning
starts.

**Why this priority**: Planning quality is currently limited by a captured goal
that is inspectable but still too implicit. The smallest valuable next step is
to make bounded acceptance and constraint decisions visible before Synod turns
that goal into a plan.

**Independent Test**: Start a session, capture a goal with or without authored
brief inputs, and verify that Synod stores one negotiation packet showing the
normalized goal, acceptance boundary, key constraints, and any blocking
clarification state without requiring planning to run.

**Acceptance Scenarios**:

1. **Given** a new session with a goal and one or more authored brief inputs,
   **When** the operator runs the capture step, **Then** Synod records one
   negotiated delivery packet that summarizes the intended outcome, in-scope
   work, out-of-scope work, acceptance checks, and binding constraints for the
   session-native story.
2. **Given** a new session with only a direct goal and no authored brief,
   **When** the operator runs the capture step, **Then** Synod still derives a
   negotiation packet with explicit defaults rather than leaving planning to
   infer hidden constraints.
3. **Given** a captured goal whose acceptance boundary or required constraints
   are materially ambiguous, **When** Synod cannot derive a credible negotiated
   packet, **Then** it stops before planning with an explicit clarification or
   blocked state that names what must be resolved.

---

### User Story 2 - Carry Constraints Through Planning And Follow-Up (Priority: P2)

An operator can see which constraints, tradeoffs, and acceptance boundaries are
currently shaping the plan, the next step, and any failure or exhaustion state,
instead of reverse-engineering those decisions from planner output.

**Why this priority**: Capturing a negotiation packet only matters if later
surfaces continue to explain which constraints are binding and why a tradeoff or
stop condition was chosen.

**Independent Test**: Capture a negotiated goal, generate a plan, run or inspect
the session through representative success and non-success paths, and verify
that `plan`, `run`, `status`, `next`, and `inspect` expose the active
acceptance boundary, binding constraints, and selected tradeoff story.

**Acceptance Scenarios**:

1. **Given** a session with a negotiated delivery packet, **When** the operator
   plans the work, **Then** the resulting plan preserves the active acceptance
   boundary, constraint summary, and chosen tradeoff rationale instead of
   collapsing back to goal-only output.
2. **Given** a planned or running session whose bounded execution reaches a
   blocked, failed, or exhausted state, **When** the operator checks follow-up
   output, **Then** Synod identifies which constraint, acceptance boundary, or
   unresolved tradeoff is currently binding the next action.
3. **Given** an explicit compatibility route is used instead of the primary
   session-native path, **When** the operator inspects follow-up behavior,
   **Then** Synod keeps that compatibility route explicit and does not imply
   that hidden session-native negotiation authority exists.

---

### User Story 3 - Ship The Negotiation Story As One Release (Priority: P3)

A maintainer can ship one `0.26.0` release where runtime behavior,
operator-facing summaries, docs, assistant guidance, version metadata,
changelog, validation evidence, and coverage all describe the same goal
negotiation and constraint-modeling story.

**Why this priority**: This slice changes how operators understand and challenge
planning decisions. The release is incomplete if only the runtime changes while
the docs, assistant prompts, version metadata, or validation expectations still
describe the old implicit planning story.

**Independent Test**: Follow the updated docs on a representative session,
verify that runtime surfaces match the documented negotiation behavior, and
confirm that formatting, clippy, coverage refresh for touched Rust files, and
the required validation suite all pass for the release.

**Acceptance Scenarios**:

1. **Given** the `0.26.0` release artifacts, **When** a maintainer follows the
   documented capture-to-plan workflow, **Then** the observed negotiation,
   constraint, and follow-up output matches the documented operator story.
2. **Given** changed Rust sources for this slice, **When** maintainers run the
   release validation suite, **Then** formatting, clippy, required tests, and
   coverage refresh for modified or created Rust files complete without
   undocumented regressions.

### Edge Cases

- What happens when capture produces mutually conflicting constraints, such as a
  request for broad mutation but a bounded acceptance rule that limits changes
  to one file or one workspace?
- What happens when a goal is short and credible enough to plan, but lacks
  enough acceptance detail to determine whether a later run actually satisfied
  the intended outcome?
- What happens when a non-success terminal state is reached because a constraint
  becomes binding only after planning has already started?
- What happens when a session is resumed after a negotiation packet was created
  under one set of constraints but new authored inputs materially change the
  acceptance boundary?
- What happens when explicit compatibility follow-up exists for the workspace
  but no session-native negotiation packet is authoritative for the current
  route?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST create one explicit negotiated delivery packet during
  capture for the primary session-native route, even when the operator provides
  only a simple goal.
- **FR-002**: System MUST represent the negotiated packet with a normalized goal
  summary, explicit acceptance boundary, explicit scope boundaries, and a set of
  binding or proposed constraints.
- **FR-003**: System MUST distinguish between resolved negotiation output and
  unresolved clarification or conflict state before planning can continue.
- **FR-004**: System MUST prevent planning from continuing when required
  acceptance boundaries or binding constraints remain materially ambiguous or
  contradictory.
- **FR-005**: System MUST preserve the negotiated packet or a derived summary in
  session state so later commands can explain what planning was allowed to do.
- **FR-006**: System MUST project the active acceptance boundary, binding
  constraints, and selected tradeoff summary through operator-facing planning
  and follow-up surfaces.
- **FR-007**: System MUST surface which constraint, acceptance boundary, or
  unresolved tradeoff is binding when execution becomes paused, blocked, failed,
  exhausted, or inspect-only.
- **FR-008**: System MUST make any selected tradeoff explicit enough that an
  operator can understand why one bounded plan shape was chosen over another.
- **FR-009**: System MUST preserve existing bounded execution limits and
  terminal-state behavior rather than introducing open-ended negotiation loops
  or hidden background reasoning.
- **FR-010**: System MUST keep the explicit compatibility route distinct from
  the primary session-native negotiation path and MUST NOT imply hidden session
  authority when only compatibility follow-up exists.
- **FR-011**: System MUST preserve existing goal-capture and plan behavior for
  workflows that do not require additional negotiation detail, using explicit
  defaults instead of breaking previously valid bounded sessions.
- **FR-012**: System MUST update runtime behavior, tests, version metadata,
  impacted documentation, assistant guidance, and changelog together for the
  `0.26.0` release.
- **FR-013**: System MUST refresh coverage for modified or created Rust files,
  resolve clippy issues introduced by the slice, and finish with repository
  formatting applied.

### Scope Boundaries *(mandatory)*

- **In Scope**: explicit negotiated packet capture before planning; acceptance
  boundary and constraint modeling for the session-native route; operator-visible
  tradeoff summaries through plan and follow-up surfaces; explicit non-success
  reporting when negotiation remains unresolved or later becomes binding; release
  closeout for `0.26.0` including version bump, impacted docs, changelog,
  coverage refresh, clippy cleanup, and formatting.
- **Out of Scope**: provider-agnostic negotiation engines; background
  negotiation loops; automatic distributed orchestration; generic policy DSLs;
  Canon-owned planning control flow; UI work outside the existing CLI and
  assistant surfaces; unconstrained long-term memory of past negotiations.

### Key Entities *(include if feature involves data)*

- **Negotiated Delivery Packet**: The explicit session-owned summary of the
  requested outcome, acceptance boundary, scope boundary, constraint set,
  clarification state, and tradeoff summary captured before planning.
- **Constraint Record**: One inspectable rule or limit that shapes planning or
  execution, including whether it is binding, proposed, conflicting, or
  satisfied.
- **Acceptance Boundary**: The operator-visible statement of what must be true
  for the goal to count as satisfied and what evidence later surfaces should use
  to justify that claim.
- **Tradeoff Summary**: The inspectable explanation of why Synod preserved one
  bounded plan shape or constraint priority instead of another.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative goal-only and authored-brief session-native
  capture scenarios, operators can identify the negotiated outcome, acceptance
  boundary, and binding constraints from the reported capture output in under 2
  minutes.
- **SC-002**: 100% of representative planning attempts with materially missing
  or conflicting required constraints stop before plan confirmation with an
  explicit clarification or blocked explanation.
- **SC-003**: In representative success, blocked, failed, exhausted, and
  inspect-only follow-up scenarios, operators can identify the currently binding
  constraint or tradeoff from `status`, `next`, or `inspect` output in under 2
  minutes.
- **SC-004**: Maintainers can validate the `0.26.0` negotiation story,
  including touched-Rust coverage output, in under 20 minutes using the shipped
  docs and repository validation commands.

## Assumptions

- Session-native orchestration remains the primary operator path for this slice,
  and explicit compatibility behavior remains a separate, clearly named route.
- Existing goal capture already provides enough source material to derive a
  bounded negotiation packet when reasonable defaults are used.
- Operators benefit more from one explicit bounded negotiation summary than from
  a broader interactive question loop in this initial slice.
- The `0.26.0` release deepens the existing capture, plan, run, status, next,
  and inspect story instead of introducing a separate planning subsystem.
