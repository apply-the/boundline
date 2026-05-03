# Data Model: Bounded Delegated Execution

## RuntimeCapabilityProfile

- **Purpose**: Declares which routed execution behaviors a runtime can support
  credibly for Synod's bounded delivery path.
- **Fields**:
  - `runtime`: the declared runtime identity.
  - `supports_continuation`: whether the runtime can own a direct bounded next
    action once selected.
  - `supports_resume`: whether the runtime can credibly resume a previously
    delegated continuity path.
  - `supports_validation`: whether the runtime can own validation-oriented
    follow-through without an extra handoff.
  - `supports_handoff_target`: whether the runtime may be named as a handoff
    destination.
  - `supports_escalation_context`: whether the runtime can emit enough context
    for later inspection when escalation occurs.
  - `notes`: bounded operator-facing explanation of declared limits.
- **Validation rules**:
  - A declared profile must be tied to a known runtime.
  - Empty notes are allowed only when every declared capability is straightforward.
  - A runtime cannot be selected as a handoff target when
    `supports_handoff_target` is false.

## SlotEffortPolicy

- **Purpose**: Declares how much reasoning effort a route slot should prefer
  when multiple credible paths exist.
- **Fields**:
  - `slot`: planning, implementation, verification, review, or adjudication.
  - `effort_level`: low, medium, high, or max.
  - `rationale`: bounded explanation of why this level fits the slot.
  - `fallback_behavior`: whether lower-effort routes may be chosen when the
    preferred level is unavailable.
- **Validation rules**:
  - Every routed slot may carry at most one active effort policy.
  - `fallback_behavior` must be explicit so route selection never depends on a
    hidden downgrade rule.

## DelegationPacket

- **Purpose**: The authoritative record of a continuity boundary that hands work
  to another route or stops for escalation.
- **Fields**:
  - `packet_id`: stable packet identifier.
  - `kind`: handoff or escalation.
  - `state`: active, resolved, superseded, stuck, or exhausted.
  - `created_at`: timestamp.
  - `resolved_at`: optional timestamp.
  - `source_route_owner`: native, workflow, governance, review, or
    compatibility.
  - `target_owner`: target route, operator, or unresolved escalation owner.
  - `continuity_reason`: why direct continuation stopped.
  - `recommended_next_action`: bounded next step or command.
  - `evidence_refs`: decisive evidence references.
  - `capability_summary`: route or slot capability facts that triggered the
    packet.
  - `stuck_marker`: optional current stuck evidence.
  - `superseded_by_packet_id`: optional link to the newer packet.
- **Validation rules**:
  - Every packet must include at least one evidence reference or one explicit
    continuity reason.
  - Active packets cannot have `resolved_at`.
  - Superseded packets must name the packet that replaced them.

## DelegationContinuityState

- **Purpose**: Summarizes the continuity status that the runtime and CLI should
  treat as authoritative for the current bounded goal.
- **Fields**:
  - `active_packet_id`: optional pointer to the active packet.
  - `mode`: none, handoff-required, escalation-required, resolved, stuck,
    exhausted, or inspect-only.
  - `authority_source`: active native session, workflow state, or compatibility
    trace.
  - `next_command`: bounded continuation or recovery command.
  - `headline`: short status line for `status`, `next`, and `inspect`.
  - `evidence_summary`: compact explanation of why this state is authoritative.
- **Validation rules**:
  - `handoff-required` and `escalation-required` require an active packet.
  - `inspect-only` may not imply resumable native state.

## StuckEvidenceMarker

- **Purpose**: Captures why delegated continuity is no longer progressing
  credibly.
- **Fields**:
  - `repeated_attempts`: bounded count of repeated blocked attempts.
  - `same_reason_count`: how many times the same continuity reason repeated.
  - `unchanged_workspace_signal`: whether changed-files or validation evidence
    remained materially unchanged.
  - `stale_route_policy`: whether the governing route declaration changed or is
    missing.
  - `recommended_recovery`: replan, resolve packet, update config, rerun
    validation, or escalate.
- **Validation rules**:
  - A stuck marker must cite at least one explicit repeated or unchanged signal.
  - Recovery recommendation must map to a supported bounded action.

## GoalPlan And Session Projection Updates

- **GoalPlan new responsibilities**:
  - Persist the route capability and effort rationale that shaped plan selection.
  - Persist the active delegation continuity summary when planning is blocked by
    an unresolved packet.
- **Session projection new responsibilities**:
  - Expose the active continuity mode, delegation packet headline, route
    capability projection, effort projection, and stuck status.

## Trace Projection Updates

- **Purpose**: Records capability-aware route selection, packet creation,
  packet resolution or supersession, and stuck detection as part of the same
  inspectable trace vocabulary as native execution.
- **Required payload concepts**:
  - Capability summary.
  - Effort policy summary.
  - Active or superseded packet identifier.
  - Continuity authority.
  - Stuck marker and recommended recovery.