# Data Model: Session-Native Surface Unification

**Feature**: 016-session-native-surface-unification  
**Date**: 2026-04-29

## Core Projection Entities

### UnifiedSessionSummary

The operator-facing projection that makes one session-native story visible across `run`, `status`, `next`, and `inspect`.

```text
UnifiedSessionSummary
├── session_id: String
├── workspace_ref: String
├── route: RouteExplanation
├── flow_state: Option<String>
├── execution_condition: ExecutionCondition
├── latest_decision: Option<LatestDecisionSummary>
├── review_projection: Option<ReviewProjection>
├── adaptive_projection: Option<AdaptiveProjection>
├── governance_projection: Option<GovernanceProjection>
├── next_action: Option<String>
└── explanation: String
```

**Behavioral rules**:
- Every operator surface must be able to derive this summary from persisted session state and persisted traces.
- Optional projections may be absent, but the route and execution condition must still remain explicit.
- Compatibility behavior may appear in the route explanation, but it must not erase the session-native summary model.

### RouteExplanation

The explicit answer to "which path is active and why?"

```text
RouteExplanation
├── mode: native | compatibility | blocked
├── source: goal_plan | execution_profile | explicit_operator_choice | missing_context
├── reason: String
└── precedence_note: Option<String>
```

**Behavioral rules**:
- A ready session-native plan takes precedence unless the operator explicitly chooses compatibility behavior.
- `blocked` is used when the system cannot proceed credibly and must recommend remediation.

### ExecutionCondition

The normalized operator-facing condition of the session.

```text
ExecutionCondition
├── kind: running | blocked | waiting | terminal
├── state: planned | running | waiting_review | waiting_governance | blocked_confirmation | blocked_missing_context | succeeded | failed | exhausted | no_actionable
├── reason: String
└── next_action: Option<String>
```

**Behavioral rules**:
- `waiting` is non-terminal and indicates that bounded work may continue after the expected external or operator action occurs.
- `blocked` requires operator remediation before execution can resume.
- `terminal` must preserve the final stop reason without implying that work is still in progress.

### LatestDecisionSummary

The normalized operator-facing summary of the most recent bounded decision.

```text
LatestDecisionSummary
├── status: pending | dispatched | verified | failed | recovered
├── target: String
├── rationale: Option<String>
└── expected_outcome: Option<String>
```

**Behavioral rules**:
- When no decision has been dispatched yet, this projection is absent rather than filled with placeholders.
- The projection may be enriched from trace evidence when the latest session state is not sufficient by itself.

## Optional Mode Projections

### ReviewProjection

```text
ReviewProjection
├── trigger: Option<String>
├── vote: Option<String>
├── outcome: Option<String>
└── headline: Option<String>
```

### AdaptiveProjection

```text
AdaptiveProjection
├── workspace_slice: Option<String>
├── selection_headline: Option<String>
├── attempt_lineage: Option<String>
└── validation_status: Option<String>
```

### GovernanceProjection

```text
GovernanceProjection
├── stage: Option<String>
├── runtime: Option<String>
├── mode: Option<String>
├── run_ref: Option<String>
├── state: Option<String>
├── blocked_reason: Option<String>
├── approval: Option<String>
├── decision: Option<String>
├── candidates: Vec<String>
└── next_action: Option<String>
```

**Behavioral rules**:
- Optional projections extend the unified session summary and do not replace route explanation or execution condition.
- Governance projection may describe waiting or blocked states, but Canon remains a stage-boundary overlay rather than the per-action controller.

## Read-Side Projection

### UnifiedTraceSummary

The trace-facing read model that reuses unified route and condition semantics while preserving trace detail.

```text
UnifiedTraceSummary
├── trace_ref: String
├── route_summary: String
├── execution_condition: ExecutionCondition
├── goal_plan_summary: Option<String>
├── decision_timeline: Vec<String>
├── failure_evidence: Vec<String>
├── review_projection: Option<ReviewProjection>
├── adaptive_projection: Option<AdaptiveProjection>
├── governance_projection: Option<GovernanceProjection>
└── next_action: Option<String>
```

**Behavioral rules**:
- `inspect` must preserve ordered decision and failure detail while staying semantically aligned with `status` and `next`.
- Missing optional projections must not make `inspect` imply a different route or runtime mode.

## Relationships

- `UnifiedSessionSummary.route` is derived from existing routing outcome rules plus explicit compatibility choice.
- `UnifiedSessionSummary.execution_condition` is derived from session status, latest terminal reasoning, flow-confirmation state, and optional mode projections.
- `ReviewProjection`, `AdaptiveProjection`, and `GovernanceProjection` enrich the summary only when the corresponding bounded mode is active.
- `UnifiedTraceSummary` reuses the same route and condition semantics but adds trace-specific decision and failure evidence.

## State Transitions

### ExecutionCondition lifecycle

```text
planned -> running -> terminal
planned -> blocked -> running
running -> waiting -> running
running -> terminal
```

### RouteExplanation lifecycle

```text
blocked -> native
blocked -> compatibility
native -> terminal report
compatibility -> terminal report
```

### Optional projection lifecycle

```text
absent -> present -> updated -> absent
```

The summary model tolerates optional projections appearing and disappearing as bounded review, adaptive, or governance state activates or resolves.