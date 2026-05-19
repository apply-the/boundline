# Feature Specification: S7.1 Assistant Delight Follow-Through

**Feature Branch**: `063-assistant-delight-followthrough`  
**Created**: 2026-05-19  
**Status**: Draft  
**Input**: User description: "procediamo con la prossima spec per S7.1 Crea la spec in boundline e se serve anche in canon"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reasoning-Profile-Aware Explanations (Priority: P1)

As an operator working through an active Boundline session, I want explanation
surfaces such as challenge, hidden-impact, and explain-plan to disclose the
active reasoning profile, why it was chosen, and how it changed the answer, so
I can trust high-value assistance without guessing whether advanced reasoning is
actually in effect.

**Why this priority**: S7.1 exists to deepen the already-delivered delight layer
only where mature reasoning profiles materially improve judgment. If reasoning
profile disclosure is missing, operators cannot distinguish a profile-aware
answer from a basic fallback.

**Independent Test**: Activate a session that qualifies for profile-aware
reasoning, run the explanation surfaces, and confirm the output identifies the
active profile, the selection rationale, the profile-specific contribution, and
the fallback disclosure when the capability is absent or degraded.

**Acceptance Scenarios**:

1. **Given** an active session where a reasoning profile is selected,
   **When** the operator asks for a challenge or plan explanation,
   **Then** Boundline explains which profile is active, why it was selected for
   that work, and what part of the answer depended on that profile.
2. **Given** an active session where profile-aware reasoning is unavailable,
   incompatible, or degraded, **When** the operator asks for the same
   explanation, **Then** Boundline returns the best bounded fallback answer and
   explicitly states what profile support is missing and why the answer is less
   informed.

---

### User Story 2 - Human-Facing Inspect Closure (Priority: P2)

As an operator reviewing session history, I want human-facing inspect surfaces
for context, council, and timeline, so I can understand how the current session
state was assembled, challenged, and changed without reading raw trace payloads.

**Why this priority**: The first delight slice already improved inspect for
risk, assumptions, evidence, blockers, and next. S7.1 should close the
remaining roadmap-level inspect views so the operator can navigate the rest of
the session-native loop with the same clarity.

**Independent Test**: Run inspect on representative sessions that include normal
progress, degradation, and council participation, then verify that context,
council, and timeline views remain human-facing, source-attributed, and clear
about fallback or missing evidence.

**Acceptance Scenarios**:

1. **Given** a session with assembled evidence, review activity, and multiple
  execution steps, **When** the operator opens inspect views for context,
  council, and timeline, **Then** Boundline shows those views in operator
  language with source attribution and clear transitions instead of raw trace
  reading.
2. **Given** a session where some inspectable evidence is missing, stale, or
  downgraded, **When** the operator opens the same inspect views,
  **Then** Boundline discloses the gap, preserves the authoritative state that
  is available, and explains what the operator can inspect next.

---

### User Story 3 - Host Parity And Feedback Signals (Priority: P3)

As a maintainer responsible for Boundline's assistant surfaces, I want explicit
parity decisions for Cursor and Gemini plus lightweight product feedback
signals, so we can improve the delight layer where it is credible and avoid
carrying vague or noisy host promises.

**Why this priority**: The existing delight layer already shipped richer command
assets for Claude, Codex, and Copilot. S7.1 should finish the follow-through by
making host parity decisions explicit and by measuring whether the delight layer
is actually useful.

**Independent Test**: Review the generated host assets and operator-facing
feedback surfaces, then confirm that Cursor and Gemini have either an explicit
parity path or an explicit fallback path and that usefulness signals are
available without leaving the session-native workflow.

**Acceptance Scenarios**:

1. **Given** a host surface where full parity is worth delivering,
   **When** an operator uses the assistant package for that host,
   **Then** the relevant delight commands are exposed with the same bounded
   meaning and fallback disclosure expected on the primary hosts.
2. **Given** a host surface where full parity is not worth the complexity,
   **When** an operator uses the documented fallback path,
   **Then** Boundline makes the fallback explicit and still points the operator
   to the next useful session-native command instead of implying missing
   capability will appear automatically.
3. **Given** a representative set of delight interactions,
   **When** maintainers inspect product feedback signals,
   **Then** they can see whether operators reached a useful first answer,
   accepted the next action, or overrode the suggestion often enough to justify
   follow-up work.

---

### Edge Cases

- A reasoning profile was selected earlier in the session, but the current
  explanation request no longer has enough evidence to justify profile-specific
  claims.
- `inspect council` is requested on a session that never activated a council and
  must explain that absence without implying silent failure.
- `inspect timeline` encounters blocked, failed, or exhausted steps and must
  preserve the authoritative stop reason rather than flattening the sequence
  into a success narrative.
- Cursor or Gemini cannot support the same package affordance depth as the
  primary hosts and the system must make that parity boundary explicit.
- Feedback signals show high override rates for a delight surface and the system
  must preserve enough evidence for maintainers to understand whether the issue
  is noise, weak context, or a poor next-action suggestion.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST keep the session-native workflow as the primary
  route for S7.1 explanation and inspect behavior, with any compatibility or
  fallback route explicitly disclosed rather than implied.
- **FR-002**: The system MUST let explanation surfaces disclose whether a
  reasoning profile is active for the current answer.
- **FR-003**: When a reasoning profile is active, the system MUST identify why
  that profile was selected for the current work.
- **FR-004**: When a reasoning profile materially changed the answer, the system
  MUST explain where the profile-specific challenge, synthesis, or adjudication
  changed the output.
- **FR-005**: When profile-aware reasoning is unavailable, degraded, or
  incompatible, the system MUST provide a bounded fallback answer and explicitly
  state the fallback condition.
- **FR-006**: The system MUST provide human-facing inspect surfaces for context,
  council, and timeline that are navigable without reading raw traces.
- **FR-007**: The inspect context surface MUST explain the current evidence
  assembly in operator-facing terms and identify important missing or weak
  context.
- **FR-008**: The inspect council surface MUST show whether council activity was
  used, skipped, or unavailable and preserve the authority semantics attached to
  that state.
- **FR-009**: The inspect timeline surface MUST preserve execution order,
  bounded stop states, and recovery attempts in a form an operator can follow.
- **FR-010**: All S7.1 explanation and inspect outputs MUST preserve source
  attribution and distinguish authoritative session state from inferred or
  unavailable inputs.
- **FR-011**: The system MUST make explicit parity decisions for Cursor and
  Gemini so each host either receives a credible delight surface or a documented
  bounded fallback path.
- **FR-012**: The system MUST avoid expanding the default assistant surface into
  a noisy command palette when adding S7.1 follow-through behavior.
- **FR-013**: The system MUST expose lightweight operator-facing feedback
  signals for delight usefulness, including time to first useful answer,
  evidence-attributed explanation rate, and next-action acceptance or override
  behavior.
- **FR-014**: The system MUST make those feedback signals inspectable without
  requiring maintainers to reconstruct them manually from unrelated logs.
- **FR-015**: The system MUST preserve governance, stop, and authority
  boundaries while adding S7.1 follow-through behavior.
- **FR-016**: This feature MUST NOT require new Canon provider artifact classes
  or a new Canon-side contract line; any newly discovered Canon contract gap
  MUST be captured as a separate follow-on specification instead of being folded
  silently into this slice.

### Scope Boundaries *(mandatory)*

- **In Scope**: reasoning-profile-aware delight explanations; human-facing
  inspect context, inspect council, and inspect timeline surfaces; explicit host
  parity decisions for Cursor and Gemini; lightweight product feedback signals
  for delight usefulness; Boundline-side updates needed to keep these behaviors
  inspectable and bounded.
- **Out of Scope**: new Canon provider semantics or Canon artifact classes; a
  second assistant runtime; new reasoning-profile algorithms; a dashboard or
  analytics product; unbounded telemetry programs; reopening the already
  delivered base delight command set except where S7.1 deepens its existing
  behavior.

### Key Entities *(include if feature involves data)*

- **Delight Explanation Surface**: An operator-facing explanation request such
  as challenge, hidden-impact, or explain-plan that may incorporate
  reasoning-profile-specific disclosure.
- **Reasoning Profile Disclosure**: The structured statement of whether a
  profile is active, why it was selected, what it changed, and what fallback was
  used when profile-aware support is missing.
- **Inspect Closure View**: A human-facing inspect surface for context,
  council, or timeline that projects authoritative session state into operator
  language.
- **Host Parity Decision**: The explicit decision for a host surface to support
  equivalent delight affordances or to expose a bounded fallback instead.
- **Delight Feedback Signal**: A lightweight measure that helps maintainers see
  whether delight surfaces produced a useful answer or a noisy one.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In at least 90% of representative reasoning-profile-aware delight
  scenarios, operators can identify within one read whether a reasoning profile
  was active and what it changed.
- **SC-002**: In 100% of representative degraded or fallback scenarios, delight
  outputs disclose the missing or incompatible capability instead of implying
  full profile-aware support.
- **SC-003**: Operators can inspect context, council, and timeline for a
  representative session in under five minutes without opening raw trace files.
- **SC-004**: For each supported host surface, maintainers can state whether the
  host has delight parity or a bounded fallback path with no undocumented middle
  state.
- **SC-005**: Maintainers can review lightweight delight usefulness signals for
  representative sessions without reconstructing them manually from unrelated
  artifacts.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: Anthropic Models Overview,
  OpenAI Models reference, and Google Gemini Models reference, reviewed on
  2026-05-19.
- **Catalog Delta**: No bundled catalog changes were applied during this
  specification pass.
- **No-Change Rationale**: The current bundled catalog already includes the
  current provider families relevant to Boundline routing for this slice,
  including GPT-5.5 and GPT-5.4 variants, Claude Opus 4.7 and Sonnet 4.6, and
  Gemini 2.5 and 3.1 entries. S7.1 changes the delight and inspect surfaces, not
  the supported routing model families, so no catalog delta was required.

## Assumptions

- Reasoning-profile closure is sufficiently mature that profile-aware delight
  behavior is now credible for operator-facing use.
- The primary value of S7.1 remains Boundline-side follow-through, so no new
  Canon-side specification is needed unless implementation later uncovers a real
  contract gap.
- Existing session state, trace state, and inspect projections remain the
  authoritative substrate for delight follow-through.
- Cursor and Gemini may require a more explicit fallback posture than Claude,
  Codex, or Copilot, but the parity decision itself must still be visible and
  reviewable.
- Lightweight usefulness signals are sufficient for this slice; a separate
  analytics program is not required to judge whether delight follow-through is
  working.
