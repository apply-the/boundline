# Contract: Governance Trace Events

## Purpose

Define the new trace events and payload expectations required to make governed stage selection, approval waits, packet quality, and autopilot decisions inspectable.

## New Trace Event Types

- `GovernanceSelected`
- `GovernanceStarted`
- `GovernanceDecisionRecorded`
- `GovernanceAwaitingApproval`
- `GovernanceCompleted`
- `GovernanceBlocked`
- `GovernancePacketRejected`

## Required Payload Fields

### GovernanceSelected

- `stage_key`
- `required`
- `autopilot_enabled`
- `requested_runtime`
- `selected_runtime`

### GovernanceStarted

- `stage_key`
- `runtime`
- `canon_mode` when runtime is `canon`
- `system_context` when runtime is `canon`
- `risk` when runtime is `canon`
- `zone` when runtime is `canon`
- `owner` when runtime is `canon`

### GovernanceDecisionRecorded

- `stage_key`
- `candidate_actions`
- `candidate_modes` when `select_mode` is a candidate
- `selected_action` when a compliant path exists
- `selected_mode` when `selected_action = select_mode`
- `selected_target_stage_key` when `selected_action` is an escalation action
- `reason`
- `blocked_reason` when no compliant path exists

### GovernanceAwaitingApproval

- `stage_key`
- `runtime`
- `approval_state`
- `run_ref` when a Canon run exists

### GovernanceCompleted

- `stage_key`
- `runtime`
- `packet_ref`
- `packet_readiness`
- `document_refs`
- `headline`

### GovernanceBlocked

- `stage_key`
- `runtime`
- `required`
- `reason`

### GovernancePacketRejected

- `stage_key`
- `packet_ref` when a packet exists
- `packet_readiness`
- `missing_sections`
- `reason`

## Trace Semantics

- `GovernanceSelected` must occur before a governed stage starts.
- `GovernanceDecisionRecorded` must occur whenever autopilot or runtime-selection logic chooses among multiple compliant actions.
- `GovernanceAwaitingApproval` and `GovernanceBlocked` are terminal for the current stage boundary until a later explicit resolution occurs.
- `GovernanceCompleted` is the only event that allows the governed stage to continue normally.
- `GovernancePacketRejected` must be emitted before any blocked or retry decision caused by packet-quality failure.

## Inspect Rendering Rules

- Governance trace events must be rendered in a governance timeline rather than collapsed into generic recovery lines.
- Event ordering must make it possible to answer these questions from one trace:
  - Which runtime was selected for the stage?
  - Which Canon mode was used when applicable?
  - Was approval needed and granted?
  - Was a packet rejected for quality reasons?
  - Did autopilot choose a compliant path or stop?