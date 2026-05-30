# Feature Specification: Chat-First Host-Integrated Runtime

**Feature Branch**: `045-chat-first-runtime`  
**Created**: 2026-05-09  
**Status**: Draft  
**Input**: User description: "Make Boundline a chat-native orchestration runtime that is used inside host chat surfaces for everyday work, while keeping guided init as a separate standalone experience. The first slice should prioritize VS Code chat."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Continue Delivery From Host Chat (Priority: P1)

An operator can start, continue, and recover bounded delivery work from the host
chat surface already attached to the workspace, so everyday work does not
require opening a separate standalone Boundline conversation surface.

**Why this priority**: This is the core product decision for the slice. If the
operator still has to leave the host chat to do routine delivery work, the
feature adds ceremony instead of reducing it.

**Independent Test**: Start bounded work from the primary host chat surface in
a workspace with no active session, then continue the same session through at
least one non-success turn. The operator must be able to reach a plan,
clarification, blocked state, or explicit failure outcome without leaving the
host surface.

**Acceptance Scenarios**:

1. **Given** a workspace with no active session, **When** an operator asks the
   primary host chat to begin bounded work on a concrete goal, **Then**
   Boundline records or creates the active session and returns the current
   state, next action, and planning outcome in the same host surface.
2. **Given** a workspace whose active session requires clarification or plan
   confirmation, **When** the operator continues from the host chat, **Then**
   Boundline surfaces the pending clarification or confirmation explicitly and
   tells the operator what must happen next.
3. **Given** a workspace whose active session is blocked, failed, exhausted, or
   no longer credible, **When** the operator asks the host chat to continue,
   **Then** Boundline returns the reason execution cannot proceed and the
   correct follow-up action instead of silently retrying or restarting.

---

### User Story 2 - Resume From Persisted Workspace State (Priority: P2)

An operator can come back after editor reload, chat reset, or a later follow-up
and still recover the current Boundline state, so conversational convenience
does not come at the cost of continuity or inspectability.

**Why this priority**: Host chat history is transient. The runtime is only
credible if the delivery state survives beyond one visible thread.

**Independent Test**: Start bounded work from the primary host chat, lose the
visible conversation context, then reconnect from the same workspace. Boundline
must restore the active state or explain exactly why it cannot resume.

**Acceptance Scenarios**:

1. **Given** a workspace with an active non-terminal session, **When** the
   operator returns later and asks for status or continuation, **Then**
   Boundline resumes from persisted workspace state without requiring the
   original goal to be re-entered.
2. **Given** a workspace whose latest activity produced a success, failure,
   blocked state, or follow-up requirement, **When** the operator asks what
   happened, **Then** Boundline summarizes the current state and points to a
   deeper inspection path without depending on prior host chat messages.
3. **Given** a stored workspace session that can no longer be resumed credibly,
   **When** the operator reconnects from the host chat, **Then** Boundline
   explains why resumption is invalid and what bounded repair or restart action
   is required.

---

### User Story 3 - Keep Bootstrap And Automation Explicit (Priority: P3)

An operator can still tell when guided bootstrap is required and when direct CLI
inspection or automation is more appropriate, so host chat becomes the primary
daily UX without swallowing setup and non-conversational workflows.

**Why this priority**: A chat-first runtime becomes confusing if it tries to
hide bootstrap, repair, or automation boundaries behind generic chat wording.

**Independent Test**: Exercise an uninitialized workspace, a workspace that
needs explicit setup or repair, and a direct inspection flow outside the host
chat. Boundline must direct the operator to the right surface without mixing
bootstrap and everyday execution responsibilities.

**Acceptance Scenarios**:

1. **Given** a workspace that is not ready for everyday host-chat execution,
   **When** the operator tries to begin work from the host chat, **Then**
   Boundline explains that explicit bootstrap or repair is required and points
   to the next bounded setup action.
2. **Given** a workspace with an active session, **When** an operator uses
   direct inspection or follow-up commands outside the host chat, **Then**
   Boundline exposes the same underlying state and outcome rather than a
   host-specific reinterpretation.
3. **Given** a supported compatibility continuation path is the active
   continuity authority for a workspace, **When** the operator asks the host
   chat to continue work, **Then** Boundline makes that route explicit instead
   of masking it behind generic conversational wording.

### Edge Cases

- What happens when the operator asks for bounded work from a workspace that has
  no credible goal, insufficient authored context, or conflicting repository
  evidence?
- How does the system behave when host chat history is lost mid-session but the
  persisted workspace session remains active?
- What happens when the workspace is present but not writable, not clearly
  resolvable, or not the primary owner of the active delivery state?
- How does the system surface blocked approval, governance, review, or
  clarification requirements in a conversational surface without implying that
  execution can continue immediately?
- What happens when a host chat request arrives while the active session is
  already terminal, exhausted, or contradicted by newer workspace state?
- How does the system surface the primary host-chat route versus an explicit
  compatibility continuation path when both are available?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST let an operator start, continue, and inspect bounded
  work from a supported host chat surface without requiring a separate
  standalone Boundline chat application for everyday execution.
- **FR-002**: System MUST make the current delivery state and the next required
  action explicit on host-chat follow-up surfaces.
- **FR-003**: System MUST persist enough workspace-owned session state that a
  host chat can resume bounded work after lost conversational context, editor
  reload, or later follow-up.
- **FR-004**: System MUST distinguish active progress, clarification required,
  approval blocked, workspace not ready, non-credible state, failure,
  exhaustion, and terminal success as separate user-facing outcomes.
- **FR-005**: System MUST preserve inspectable continuity between host-chat
  execution and direct Boundline inspection surfaces so both describe the same
  underlying session outcome.
- **FR-006**: System MUST allow an operator to resume an existing credible
  session without restating the original goal or reconstructing prior host chat
  history manually.
- **FR-007**: System MUST keep guided repository bootstrap as an explicit
  bounded surface separate from everyday host-chat execution.
- **FR-008**: System MUST tell the operator when host-chat execution can proceed
  in place versus when explicit initialization, repair, or clarification is
  required first.
- **FR-009**: System MUST make any explicit compatibility continuation path
  visible when that path, rather than the primary host-chat route, governs the
  active session.
- **FR-010**: System MUST return actionable recovery guidance for blocked,
  failed, exhausted, or non-credible turns.
- **FR-011**: System MUST preserve direct CLI inspection and automation
  surfaces so chat is primary for daily use but not the only credible way to
  inspect or continue work.
- **FR-012**: System MUST keep the meaning of persisted session state
  independent of any one host surface so additional supported hosts can adopt
  the same runtime behavior later.
- **FR-013**: System MUST keep host-chat responses concise enough for
  conversational use while preserving a deeper explicit inspection path when
  the operator needs more detail.
- **FR-014**: System MUST surface which workspace owns the active delivery
  state whenever clustered or multi-workspace context would otherwise make
  conversational continuation ambiguous.

### Scope Boundaries *(mandatory)*

- **In Scope**: host-chat execution as the primary everyday surface; persisted
  session continuity across transient chat history; conversational status and
  next-action summaries; clear separation between host-chat execution and
  guided bootstrap; coherence between host-chat and direct inspection surfaces;
  explicit handling of blocked, failed, exhausted, and non-credible states; VS
  Code chat as the primary first-slice surface.
- **Out of Scope**: a standalone full-screen Boundline chat application; broad
  redesign of guided `init`; arbitrary theming or branding work;
  provider-specific packaging or marketplace work; expansion of governance
  scope beyond the current bounded runtime behavior; replacing direct CLI
  automation with chat-only control.

### Key Entities *(include if feature involves data)*

- **Host Work Session**: The operator's active bounded conversation with
  Boundline in a host surface, anchored to one workspace and backed by
  persisted delivery state.
- **Persisted Delivery Session**: The durable workspace-owned record of goal,
  plan, active task, latest outcome, follow-up state, and inspection references
  that survives beyond any one visible chat thread.
- **Conversational Follow-Up Summary**: The bounded user-facing summary of what
  state the session is in, why it is in that state, and what the operator
  should do next.
- **Continuation Authority**: The explicit rule that determines whether the
  active workspace should proceed on the primary host-chat path or on a
  compatibility continuation path.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least 90% of representative bounded delivery tasks started
  from the primary host chat surface reach either a first actionable plan or an
  explicit clarification outcome without requiring a separate standalone
  Boundline chat application.
- **SC-002**: 100% of resume attempts after lost conversational context or
  later follow-up restore the current delivery state or return an explicit
  reason recovery cannot proceed.
- **SC-003**: 100% of blocked, failed, exhausted, or non-credible host-chat
  turns return a next-action summary that tells the operator how to proceed or
  inspect deeper.
- **SC-004**: For representative session states, host-chat status and direct
  inspection surfaces report the same active outcome and continuation
  authority.
- **SC-005**: In representative workspace bootstrap and repair scenarios,
  operators are directed to the explicit setup surface without ambiguous chat
  guidance at least 95% of the time.

## Assumptions

- Guided repository bootstrap remains a distinct product surface and is not
  collapsed into everyday conversational execution for this slice.
- The existing workspace-owned session and trace model remains the
  authoritative source of delivery continuity.
- The first slice prioritizes VS Code chat as the primary host surface, but
  the runtime meaning does not become specific to one host product.
- Operators may move between host chat and direct CLI inspection within the
  same workspace during one bounded delivery journey.
- Deeper inspection remains explicit rather than always rendered inline in the
  conversational response.
- The slice improves daily execution UX and continuity, not the visual design
  of a standalone terminal application.
