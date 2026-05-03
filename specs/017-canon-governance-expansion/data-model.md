# Data Model: Canon Governance Expansion

**Feature**: 017-canon-governance-expansion  
**Date**: 2026-04-29

## Core Projection Entities

### Expanded Stage Governance Policy

The bounded policy that decides whether a Boundline stage may use a newer Canon governed analysis mode.

```text
ExpandedStageGovernancePolicy
├── flow_name: String
├── stage_id: String
├── runtime: local | canon
├── required: Boolean
├── autopilot: Boolean
├── allowed_canon_modes: Vec<CanonMode>
├── selected_canon_mode: Option<CanonMode>
├── system_context: Option<existing | new>
├── risk: Option<String>
├── zone: Option<String>
└── owner: Option<String>
```

**Behavioral rules**:
- The first slice expands only the targeted existing-system verification stages.
- `security-assessment` requires `existing` system context before governed execution can continue.
- Unsupported Canon modes remain invalid even if Canon itself supports them.

### Governed Analysis Mode Selection

The explicit session-visible decision that binds one bounded stage to one Canon mode.

```text
GovernedAnalysisModeSelection
├── stage_key: String
├── candidate_modes: Vec<CanonMode>
├── selected_mode: Option<CanonMode>
├── selected_action: Option<AutopilotAction>
├── rationale: String
└── blocked_reason: Option<String>
```

**Behavioral rules**:
- Mode selection remains explicit even when only one mode is valid.
- If no compliant mode exists, the selection must remain blocked rather than invent a fallback.
- The first slice may add `security-assessment` without changing Boundline's top-level flow names.

### Governed Analysis Packet Summary

The bounded summary of the Canon packet that Boundline uses for governance state and downstream reuse.

```text
GovernedAnalysisPacketSummary
├── run_ref: Option<String>
├── packet_ref: Option<String>
├── readiness: pending | incomplete | reusable | rejected
├── headline: String
├── missing_sections: Vec<String>
└── binding_reason: Option<String>
```

**Behavioral rules**:
- Boundline reuses only packet refs, headlines, readiness, and missing metadata.
- A `rejected` or `incomplete` packet is never treated as credible completion.
- Approval refresh may change the packet's usable state without changing the current flow model.

### Governed Analysis Condition

The operator-facing state of the governed follow-on analysis path.

```text
GovernedAnalysisCondition
├── kind: running | waiting | blocked | terminal
├── approval_state: not_needed | requested | granted | rejected | expired
├── message: String
└── next_action: Option<String>
```

**Behavioral rules**:
- `waiting` is used for approval-gated analysis that has not yet resolved.
- `blocked` is used when mode validation, packet readiness, or runtime availability prevents credible continuation.
- The condition must remain visible in the same operator surfaces as the rest of the session-native runtime.

## Relationships

- `ExpandedStageGovernancePolicy` constrains which Canon modes can be selected for a Boundline stage.
- `GovernedAnalysisModeSelection` records the explicit decision to route a stage through `security-assessment`.
- `GovernedAnalysisPacketSummary` captures the bounded Canon evidence returned by the runtime.
- `GovernedAnalysisCondition` explains whether the current governed analysis path can continue, must wait, or must stop.

## State Transitions

### Governed Analysis Lifecycle

```text
pending_selection -> running -> governed_ready
pending_selection -> blocked
running -> awaiting_approval
awaiting_approval -> governed_ready
awaiting_approval -> blocked
running -> blocked
running -> failed
```

### Packet Reuse Lifecycle

```text
absent -> reusable
reusable -> reused_on_rerun
reusable -> blocked_by_refresh
pending -> reusable | blocked
```

### Bounded Expansion Rule

```text
unsupported_mode -> rejected
supported_mode + invalid_context -> blocked
supported_mode + valid_context -> running
```

The model deliberately keeps the bounded expansion small: it adds one newer Canon analysis mode to the existing stage-boundary governance model while preserving room for later `supply-chain-analysis` support.