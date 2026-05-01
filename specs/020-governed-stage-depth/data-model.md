# Data Model: Governed Stage Depth

**Feature**: 020-governed-stage-depth  
**Date**: 2026-05-01

## Core Entities

### Governed Stage Transition

The active bounded governance overlay for one stage in the session-owned bug-fix flow.

```text
GovernedStageTransition
├── stage_key: String
├── runtime: local | canon
├── lifecycle_state: pending_selection | running | awaiting_approval | governed_ready | blocked | failed | completed
├── required: Boolean
├── autopilot_enabled: Boolean
├── approval_state: not_needed | requested | approved | rejected
├── governance_attempt_id: String
├── previous_governance_attempt_id: Option<String>
├── packet_ref: Option<String>
├── decision_ref: Option<String>
└── blocked_reason: Option<String>
```

**Behavioral rules**:
- One active governed stage transition is authoritative at a time.
- A transition may halt the session while awaiting approval or after a blocked packet outcome.
- Earlier governed transitions remain inspectable through lineage and trace history even after the active transition moves on.

### Governed Stage Packet

The bounded evidence packet produced or reused for a governed stage.

```text
GovernedStagePacket
├── packet_ref: String
├── runtime: local | canon
├── canon_mode: Option<CanonMode>
├── readiness: reusable | incomplete | rejected
├── headline: String
├── expected_document_refs: Vec<String>
├── document_refs: Vec<String>
└── missing_sections: Vec<String>
```

**Behavioral rules**:
- Packet state remains bounded to references, readiness, headline, and document expectations.
- `incomplete` and `rejected` packets stop downstream governed progression explicitly.
- Packet structure must remain sufficient for inspectability without exposing full Canon internals.

### Packet Reuse Lineage

The explicit relationship between an active governed stage and previously produced governance evidence.

```text
PacketReuseLineage
├── upstream_stage_key: String
├── packet_ref: String
└── binding_reason: same_stage_rerun | upstream_stage_context
```

**Behavioral rules**:
- Reuse lineage must be persisted whenever a downstream governed transition binds prior governance evidence.
- Lineage stays explicit across session-native and workflow-aware surfaces.
- Missing or non-credible lineage must not be inferred silently.

### Governance Refresh Outcome

The result of re-checking approval and packet state on a later operator command.

```text
GovernanceRefreshOutcome
├── refreshed: Boolean
├── stage_key: String
├── lifecycle_state: awaiting_approval | governed_ready | blocked | failed
├── approval_state: not_needed | requested | approved | rejected
├── next_action: Option<String>
└── visible_reason: String
```

**Behavioral rules**:
- Refresh happens on explicit later commands only.
- A refreshed state may allow progression, continue to wait, or stop work with a clearer blocked reason.
- Refresh outcome must remain visible to both direct session and workflow projections.

### Governance Profile Guidance Example

The bounded configuration example used in shipped docs to explain the deeper governed slice.

```text
GovernanceProfileGuidanceExample
├── flow_name: String
├── governed_stages: Vec<String>
├── runtime: local | canon
├── selected_modes: Vec<String>
└── bounded_non_goals: Vec<String>
```

**Behavioral rules**:
- The guidance example must remain within the supported bug-fix slice.
- It must describe direct session-native and workflow-aware routing clearly.
- It must not imply Canon-owned orchestration or background governance progression.

## Relationships

- `GovernedStageTransition` is the active persisted governance record for the current stage boundary.
- `GovernedStagePacket` may be attached to a `GovernedStageTransition` as fresh or reused evidence.
- `PacketReuseLineage` links a current `GovernedStageTransition` to an earlier `GovernedStagePacket`.
- `GovernanceRefreshOutcome` derives from an existing `GovernedStageTransition` plus refreshed runtime state on a later command.
- `GovernanceProfileGuidanceExample` documents one supported authored shape for the deeper governed slice.

## State Transitions

### Governed Stage Lifecycle

```text
pending_selection -> running
running -> governed_ready
running -> awaiting_approval
running -> blocked
running -> failed
awaiting_approval -> governed_ready
awaiting_approval -> blocked
awaiting_approval -> failed
governed_ready -> completed
blocked -> failed
```

### Packet Readiness

```text
fresh_packet -> reusable
fresh_packet -> incomplete
fresh_packet -> rejected
reused_packet -> reusable
reused_packet -> blocked_when_not_credible
```

### Session Progression Around Governance

```text
investigate -> governed_investigate
governed_investigate -> implement
implement -> verify
verify -> governed_verify
governed_stage -> halted_until_refresh_or_resume
```

The model stays intentionally narrow: it extends the existing governance state and packet lineage story just enough to make `bug-fix:investigate` credible ahead of the existing governed verify path.