# Data Model: Decision Continuity And Guided Follow-Through

**Feature**: 028-decision-followthrough  
**Date**: 2026-05-01

## Core Entities

### Decision Continuity Snapshot

The bounded summary of the latest decision, recovery, validation, or governance
fact that materially explains what Boundline should do next.

```text
DecisionContinuitySnapshot
├── authority: native_session | compatibility_trace
├── decision_status: pending | dispatched | verified | failed | recovered | none
├── decision_target: String?
├── guidance_headline: String
├── evidence_reason: String
├── stop_reason: String?
└── source_refs: Vec<String>
```

**Behavioral rules**:
- A snapshot must identify one authoritative continuity source.
- `guidance_headline` must explain the credible next bounded action or the
  explicit reason no further bounded action is currently credible.
- `source_refs` must stay compact enough for existing CLI surfaces.

### Follow-Through Guidance

The operator-facing explanation that turns a continuity snapshot into the next
bounded action shown by `status`, `next`, or `inspect`.

```text
FollowThroughGuidance
├── next_action: String?
├── next_command: String?
├── explanation: String
├── winning_evidence_source: session | trace
└── terminal: bool
```

**Behavioral rules**:
- `next_action` and `next_command` must remain aligned.
- `terminal` is true when the credible outcome is to stop and inspect rather
  than to continue execution.
- The explanation must preserve continuity authority instead of flattening
  compatibility and native follow-up into one generic story.

### Continuity Evidence Source

The explicit evidence source Boundline chooses when projecting guided follow-through.

```text
ContinuityEvidenceSource
├── source_kind: session | trace
├── authority: native_session | compatibility_trace
├── freshness_reason: String
└── visible_on_output: bool
```

**Behavioral rules**:
- Only one evidence source can win for one projected follow-up explanation.
- `freshness_reason` must explain why this source beat the alternative.
- `visible_on_output` must remain true whenever evidence precedence materially
  changes the recommended next bounded action.

## Relationships

- One `DecisionContinuitySnapshot` produces at most one authoritative
  `FollowThroughGuidance` at a time.
- A `FollowThroughGuidance` always references exactly one winning
  `ContinuityEvidenceSource`.
- Session-native and compatibility follow-up can both produce continuity
  snapshots, but only one source is authoritative for the projected guidance.

## State Transitions

### Continuity Snapshot Lifecycle

```text
captured_from_session -> projected_to_status
captured_from_session -> superseded_by_trace_authority
captured_from_trace -> projected_to_inspect
captured_from_trace -> projected_to_status_or_next
projected_to_status -> refreshed_after_new_decision
projected_to_inspect -> terminal_stop_explained
```

### Follow-Through Guidance Lifecycle

```text
derived_from_snapshot -> shown_to_operator
shown_to_operator -> reused_after_reload
shown_to_operator -> updated_after_retry_or_replan
shown_to_operator -> replaced_with_stop_condition
```

The model stays intentionally narrow: it adds explicit continuity guidance to
the existing Boundline session and trace story without creating a new runtime,
background loop, or separate operator surface.