# Feature Specification: Bounded Delegated Execution

**Feature Branch**: `037-bounded-delegation`  
**Created**: 2026-05-03  
**Status**: Draft  
**Input**: User description: "Create macrofeature 037 for bounded delegated execution with runtime capability descriptors, explicit handoff and escalation packets, effort-aware routing policies, evidence-based stuck detection, version bump, docs and changelog updates, cargo fmt, cargo clippy, and modified Rust file coverage above 95 percent."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Route Work Through Declared Runtime Capabilities (Priority: P1)

An operator can run the primary session-native path and have Boundline shape the
next bounded action around declared runtime capabilities and effort policies
instead of discovering backend limits only after execution stalls.

**Why this priority**: This is the core operating-model change. If Boundline still
chooses and explains work as if all runtimes were interchangeable, delegation is
implicit guesswork rather than a bounded delivery feature.

**Independent Test**: Configure a workspace with different runtime capabilities
and effort policies for planning, implementation, verification, and review, then
run `goal -> plan -> run`. The resulting bounded plan and next
action must reflect those declared limits before execution attempts the blocked
step.

**Acceptance Scenarios**:

1. **Given** a credible native session where the configured implementation
   runtime lacks a required delivery capability, **When** the operator runs
   `plan` or `run`, **Then** Boundline chooses a bounded delegation or stop path
   explicitly instead of pretending the direct route can still execute.
2. **Given** a credible native session where multiple declared runtimes can
   perform the next bounded step with different effort policies, **When** the
   operator runs `plan`, **Then** Boundline records which route it selected and why
   using the declared effort and capability policy rather than hidden fallback
   behavior.

---

### User Story 2 - Persist Handoff And Escalation Packets (Priority: P2)

An operator can continue bounded delivery through explicit handoff and
escalation packets that preserve what blocked the work, what evidence matters,
and what next command or role should pick up the task.

**Why this priority**: Capability-aware routing is incomplete if the system has
no authoritative object for continuity. Delegation must become explicit state,
not a human-only convention.

**Independent Test**: Run a native session that encounters a bounded delivery
block requiring continuation by another route or a human escalation, then read
`status`, `next`, and `inspect`. The session must preserve one explicit packet
that explains the boundary and drives the next bounded action.

**Acceptance Scenarios**:

1. **Given** a native session where a bounded step cannot continue on the
   current route but can continue credibly through another declared runtime or
   slot, **When** `run` reaches that boundary, **Then** Boundline persists a
   handoff packet with the needed evidence, recommended target, and next
   bounded command instead of failing with an opaque routing error.
2. **Given** a native session where no declared runtime can continue credibly,
   **When** `run` reaches that boundary, **Then** Boundline persists an escalation
   packet that names the blocking reason, preserves the evidence basis, and
   stops in an explicit terminal state.

---

### User Story 3 - Detect Stuck Delegation And Preserve Recovery (Priority: P3)

An operator can rely on Boundline to detect when delegated delivery has become stuck
or non-credible and to preserve a bounded recovery path instead of looping
silently.

**Why this priority**: Delegation increases control-flow complexity. The feature
is only credible if it treats failure, retry, and stale continuation evidence as
first-class behavior.

**Independent Test**: Run representative sessions that produce repeated blocked
actions, unchanged workspace state, failed validations, or unresolved packets.
Boundline must convert those patterns into an explicit stuck condition, a bounded
recovery recommendation, and inspectable packet history.

**Acceptance Scenarios**:

1. **Given** a delegated session that keeps revisiting the same blocked
   condition without new evidence, **When** Boundline evaluates the next bounded
   action, **Then** it marks the continuity state as stuck and recommends a
   replan, escalation, or resolution command instead of repeating the same
   attempt.
2. **Given** a delegated session where a handoff packet becomes obsolete because
   the workspace, routing declaration, or validation evidence changed,
   **When** the operator continues the session, **Then** Boundline resolves or
   supersedes that packet explicitly rather than leaving stale delegation state
   as the active source of truth.

---

### User Story 4 - Ship Delegated Execution As 0.37.0 (Priority: P4)

A maintainer can ship `0.37.0` with bounded delegated execution reflected
consistently in runtime behavior, docs, assistant guidance, roadmap, changelog,
formatting, lint, and modified-file coverage.

**Why this priority**: This feature changes the operator-facing execution model.
It is incomplete if the runtime lands without release closure and validation
evidence.

**Independent Test**: Follow the updated documentation on a representative
workspace, exercise runtime capability policy, handoff and escalation packet
creation, and stuck detection, then run the release validation suite.

**Acceptance Scenarios**:

1. **Given** the `0.37.0` release artifacts, **When** a maintainer follows the
   documented native path, **Then** Boundline, the roadmap, and assistant guidance
   all describe delegated execution as part of the same session-owned bounded
   delivery model.
2. **Given** the modified or new Rust files for this slice, **When** the
   maintainer runs release validation, **Then** formatting succeeds, lint is
   clean, and every touched Rust file remains above 95% line coverage.

### Edge Cases

- What happens when a route is declared but missing one required capability only
  for a later bounded step, such as validation, resume, or structured output?
- How does the system behave when effort policy prefers a route that is credible
  but a current open handoff or escalation packet still points somewhere else?
- What happens when a session accumulates repeated handoff or escalation packets
  for the same bounded target without any new evidence or state change?
- How does the system surface delegated continuity on the primary
  session-native route versus an explicit compatibility follow-up that remains
  trace-authoritative?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST let operators declare bounded runtime capability
  descriptors for the routed execution slots that materially affect delivery,
  including whether a route can continue, resume, validate, or support the next
  bounded action credibly.
- **FR-002**: System MUST let operators declare effort-aware route policy for
  the routed execution slots and MUST use that policy as explicit selection
  evidence rather than as an invisible heuristic.
- **FR-003**: System MUST project the effective runtime capability and effort
  policy that shaped the current bounded session through the same configuration,
  session, and trace surfaces that already explain route ownership.
- **FR-004**: System MUST create an explicit handoff packet when the current
  bounded step cannot continue on its present route but can continue credibly
  through another declared route, slot, or operator action.
- **FR-005**: System MUST create an explicit escalation packet when no declared
  route can continue the current bounded step credibly inside the session’s
  configured limits.
- **FR-006**: System MUST persist open and historical handoff or escalation
  packets in authoritative session-owned state so later planning, execution, and
  inspection can reason about continuity without reconstructing it from prose.
- **FR-007**: System MUST preserve the blocking reason, decisive evidence,
  recommended next action, and target continuity owner for every persisted
  packet.
- **FR-008**: System MUST detect delegated stuck conditions from explicit
  evidence such as repeated blocked attempts, unresolved packet reuse, unchanged
  workspace or validation state, or stale routing declarations.
- **FR-009**: System MUST resolve, supersede, or retire delegation packets
  explicitly when new evidence, route declarations, or workspace results make
  the previously active packet obsolete.
- **FR-010**: System MUST stop in an explicit terminal or blocked state when the
  current delegation path is non-credible, exhausted, or awaiting resolution,
  instead of silently retrying or hiding fallback behavior.
- **FR-011**: System MUST expose active and recent delegation packets, their
  evidence basis, their target owner, their continuity reason, and any stuck or
  superseded state through `config show`, `run`, `status`, `next`, `inspect`,
  and persisted traces.
- **FR-012**: System MUST preserve explicit compatibility continuity when the
  latest authoritative follow-up state comes from an explicit compatibility run,
  while reusing the same delegation and escalation vocabulary where that path
  provides it.
- **FR-013**: System MUST keep delegated execution bounded by the recorded goal,
  negotiated acceptance boundary, current goal plan, and session execution
  limits instead of turning delegation into generic multi-agent orchestration.
- **FR-014**: System MUST remain sequential-first on the main operator path;
  delegation state may recommend or hand work across routes, but it MUST NOT
  introduce hidden background concurrency or unbounded fan-out in this slice.
- **FR-015**: System MUST include validation for runtime capability projection,
  effort-policy routing, packet persistence, stuck detection, packet
  supersession, and read-side projection on the native and compatibility-aware
  surfaces it changes.
- **FR-016**: System MUST include explicit release-closeout work for bumping the
  Boundline version to `0.37.0`, updating impacted docs and assistant guidance,
  refreshing the roadmap, and recording the release in the changelog.
- **FR-017**: System MUST finish with clean formatting, clean lint results, and
  line coverage above 95% for every modified or newly created Rust file in this
  slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: declared runtime capability and effort policy for routed slots;
  session-owned handoff and escalation packets; evidence-based stuck detection;
  explicit packet supersession and continuity projection on the existing native
  and compatibility-aware surfaces; release closeout for `0.37.0`.
- **Out of Scope**: tmux-backed agent lifecycle management; inbox or mailbox
  systems; distributed or parallel execution; background daemons; long-term
  memory beyond session and trace scope; review councils or generic voting;
  provider-abstraction refoundation; UI work; deployment pipelines.

### Key Entities *(include if feature involves data)*

- **Runtime Capability Profile**: the bounded declaration of what an execution
  route can do credibly for a given slot, including the continuity and
  validation behaviors that matter to delivery.
- **Effort Policy**: the operator-declared statement of how much reasoning or
  delivery effort a slot should prefer when multiple credible routes exist.
- **Delegation Packet**: the authoritative handoff or escalation record that
  preserves a continuity boundary, the evidence that caused it, the target owner
  or action, and the recommended next command.
- **Delegation Continuity State**: the session-owned projection that says
  whether current delegated continuity is active, resolved, superseded, stuck,
  or exhausted.
- **Stuck Evidence Marker**: the compact summary of repeated blocked attempts,
  unchanged output, or stale route state that justifies a bounded stuck verdict.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative routing scenarios with differing declared
  runtime capabilities, operators can identify which capability or effort rule
  changed the selected bounded path in under 2 minutes using standard Boundline
  output.
- **SC-002**: In representative delivery blocks, 100% of blocked native runs
  either persist a handoff packet, persist an escalation packet, or stop with an
  explicit bounded reason rather than failing through an opaque routing error.
- **SC-003**: In representative delegated continuity scenarios, developers can
  identify the active packet, its decisive evidence, and its next recovery or
  continuation action from `status`, `next`, or `inspect` without opening raw
  persisted JSON.
- **SC-004**: In representative repeated-block scenarios, Boundline detects and
  surfaces a stuck continuity state before repeating the same blocked action more
  than the configured limit.
- **SC-005**: All modified or newly created Rust files in this slice complete
  the release validation suite above 95% line coverage with clean formatting and
  lint results.

## Assumptions

- The primary operator path remains the session-native route; explicit
  compatibility follow-up remains subordinate but must use the same bounded
  continuity vocabulary when it owns the latest authoritative state.
- Existing routing, session, goal-plan, task-context, follow-through, and trace
  surfaces can be extended to carry runtime capability, effort policy, and
  delegation packets without introducing a second orchestration runtime.
- This slice stays sequential-first even while making handoff and escalation
  explicit; any future fan-out or parallel dispatch remains a separate roadmap
  decision.
- Release closeout for `0.37.0` may update repository docs, assistant guidance,
  roadmap entries, version metadata, and changelog in the same delivered
  macrofeature.
