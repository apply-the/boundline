# Data Model: Governed Delivery With Canon Inside The Loop

**Feature**: 031-canon-delivery-loop  
**Date**: 2026-05-02

## Core Entities

### Governed Delivery Session

The persisted session-native state for one bounded delivery attempt where
Boundline executes work and Canon may govern stage boundaries.

```text
GovernedDeliverySession
├── session_id: String
├── workspace_ref: String
├── goal: String
├── active_flow: bug-fix | change | delivery | none
├── latest_status: initialized | running | succeeded | failed | exhausted
├── latest_trace_ref: String?
├── latest_governance_stage: String?
├── latest_governance_state: governed_ready | awaiting_approval | blocked | failed | none
├── latest_governance_packet_ref: String?
├── latest_changed_files: [String]?
└── latest_validation_status: passed | failed | none
```

**Behavioral rules**:
- The session remains Boundline-owned even when Canon governs one or more stages.
- `latest_governance_state` must remain visible on the same read-side surfaces
  used for non-governed runs.
- Terminal success is invalid when `latest_changed_files` is empty or
  `latest_validation_status` is absent or not `passed`.

### Governed Stage Packet

The Canon-sourced packet or evidence that Boundline may reuse across governed
stages.

```text
GovernedStagePacket
├── stage_key: String
├── runtime: local | canon
├── canon_mode: discovery | implementation | security-assessment | ...
├── packet_ref: String
├── readiness: pending | incomplete | reusable | rejected
├── expected_document_refs: [String]
├── document_refs: [String]
├── missing_sections: [String]
└── headline: String
```

**Behavioral rules**:
- Only `reusable` packets may allow downstream continuation.
- Non-reusable packets convert governed continuation into an explicit block.
- Packet lineage stays inspectable through session and trace projections.

### Delivery Completion Gate

The minimal set of evidence required before Boundline can mark a governed delivery
as successfully completed.

```text
DeliveryCompletionGate
├── governance_allows_completion: bool
├── material_workspace_diff_present: bool
├── validation_evidence_credible: bool
└── terminal_success_allowed: bool
```

**Behavioral rules**:
- `governance_allows_completion` is false when governance is blocked, failed,
  awaiting approval, or backed by a non-reusable packet.
- `material_workspace_diff_present` is true only when `latest_changed_files`
  contains at least one bounded changed file.
- `validation_evidence_credible` is true only when the latest validation state
  proves the run passed its bounded validation command.
- `terminal_success_allowed` is true only when all three predicates are true.

### Governance Continuity Cue

The read-side projection that tells the operator why the current governed run
may continue, pause, or stop.

```text
GovernanceContinuityCue
├── latest_governance_stage: String?
├── latest_governance_state: String?
├── latest_governance_packet_ref: String?
├── latest_governance_approval: String?
├── latest_changed_files: [String]?
├── latest_validation_status: String?
└── next_command: String?
```

**Behavioral rules**:
- The cue must remain on `run`, `status`, `next`, and `inspect` instead of
  moving governed runs onto a separate UX.
- Approval-pending or blocked governance must produce an explicit next action.
- Delivery-gate failure must be visible without requiring trace spelunking.

## Relationships

- One `GovernedDeliverySession` may carry zero or more `GovernedStagePacket`
  records over time, but only one latest governed packet is authoritative for
  continuation.
- One `GovernedDeliverySession` is evaluated through exactly one
  `DeliveryCompletionGate` at terminal-success time.
- One `GovernanceContinuityCue` is a projection of the session plus its latest
  governed packet and delivery gate state.

## State Transitions

### Governed Delivery Lifecycle

```text
goal_captured -> plan_ready -> governed_change_framing -> implementation
implementation -> verify -> terminal_success
```

### Explicit Stop Lifecycle

```text
governed_stage_started -> awaiting_approval -> paused_for_resume
governed_stage_started -> blocked -> terminal_not_credible
implementation_or_verify -> no_material_diff_or_no_validation -> terminal_not_credible
```

The model stays intentionally small: it adds a terminal success gate over
existing session and governance state rather than inventing a second runtime or
new persistence surface.