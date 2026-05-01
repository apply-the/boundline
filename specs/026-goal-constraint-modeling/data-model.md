# Data Model: Goal Negotiation And Constraint Modeling

**Feature**: 026-goal-constraint-modeling  
**Date**: 2026-05-01

## Core Entities

### Negotiated Delivery Packet

The explicit session-owned summary of the requested outcome, acceptance
boundary, active constraints, clarification state, and selected tradeoff before
planning begins.

```text
NegotiatedDeliveryPacket
├── negotiation_id: String
├── session_id: String
├── workspace_ref: String
├── goal_summary: String
├── acceptance_boundary: AcceptanceBoundary
├── constraints: Vec<NegotiationConstraint>
├── tradeoff: Option<TradeoffDecision>
├── clarification_headline: Option<String>
├── resolution_state: credible | pending_clarification | conflicting | blocked
├── source_summary: String
└── created_at: u64
```

**Behavioral rules**:
- Exactly one current negotiated packet may be authoritative for a session at a
  time.
- Re-capturing a goal replaces or supersedes the previous packet for that
  session.
- Planning may continue only when `resolution_state` is `credible`.
- Clustered sessions keep authoritative packet ownership in the primary
  workspace session.

### Acceptance Boundary

The operator-visible statement of what must be true for the goal to count as
satisfied and what evidence later follow-up surfaces should use.

```text
AcceptanceBoundary
├── success_headline: String
├── required_outcomes: Vec<String>
├── excluded_outcomes: Vec<String>
├── expected_evidence: Vec<String>
└── bounded_scope_summary: String
```

**Behavioral rules**:
- The acceptance boundary must be specific enough to distinguish success from a
  merely plausible change.
- `required_outcomes` and `excluded_outcomes` cannot contradict one another.
- `expected_evidence` must be inspectable from later plan, run, or trace
  summaries rather than hidden in planner internals.

### Negotiation Constraint

One explicit rule or limit that shapes what planning and execution may do.

```text
NegotiationConstraint
├── constraint_id: String
├── kind: scope | acceptance | risk | governance | execution_limit | routing
├── summary: String
├── source: goal | brief | governance_intent | workspace_signal | default
├── state: binding | proposed | conflicting | satisfied
└── blocks_planning: bool
```

**Behavioral rules**:
- A `binding` constraint must appear in operator-facing summaries when it
  materially shapes the plan or follow-up decision.
- A `conflicting` constraint must explain why planning stopped or which tradeoff
  must be resolved.
- Execution-limit constraints remain distinct from goal or acceptance
  constraints even when they are surfaced together.

### Tradeoff Decision

The explicit explanation of why one bounded plan shape or constraint priority
was chosen over another.

```text
TradeoffDecision
├── prioritized_constraint_id: String
├── rejected_alternative_summary: String
├── rationale: String
└── surfaced_as_blocker: bool
```

**Behavioral rules**:
- A tradeoff decision must always identify which constraint or boundary won.
- `surfaced_as_blocker` is true when no credible bounded choice exists yet and
  the operator must resolve the conflict before planning.
- Later follow-up surfaces may summarize the tradeoff, but they must preserve
  the core rationale.

## Relationships

- `NegotiatedDeliveryPacket` owns one `AcceptanceBoundary`, zero or more
  `NegotiationConstraint` records, and an optional `TradeoffDecision`.
- `NegotiationConstraint` values explain why the `AcceptanceBoundary` is bounded
  the way it is and what planning must honor.
- `TradeoffDecision` explains how a conflict or prioritization among
  `NegotiationConstraint` values was resolved or why planning is blocked.
- Planned tasks and follow-up traces should project a summary of the active
  packet rather than redefining the packet independently.

## State Transitions

### Negotiation Packet Lifecycle

```text
captured_goal -> packet_drafted
packet_drafted -> packet_credible
packet_drafted -> packet_pending_clarification
packet_drafted -> packet_conflicting
packet_pending_clarification -> packet_credible
packet_conflicting -> packet_credible
packet_credible -> packet_superseded_by_recapture
```

### Constraint Lifecycle

```text
proposed -> binding
proposed -> conflicting
binding -> satisfied
conflicting -> binding
```

The model stays intentionally local and session-owned: it adds explicit
negotiation state to the existing capture/planning story without creating a new
runtime surface or unbounded interaction loop.