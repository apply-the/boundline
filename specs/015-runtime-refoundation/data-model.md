# Data Model: Runtime Refoundation

**Feature**: 015-runtime-refoundation  
**Date**: 2026-04-29

## Core Runtime Entities

### GoalPlan (bounded task draft)

The existing goal-derived plan becomes the authoritative bounded task draft for the primary session-native path.

```text
GoalPlan
├── plan_id: String
├── goal_text: String
├── tasks: Vec<PlannedTask>
│   └── PlannedTask
│       ├── task_id: String
│       ├── description: String
│       ├── target: String
│       ├── expected_outcome: String
│       └── decision_type_hint: Option<DecisionType>
├── source_evidence: Vec<EvidenceRef>
├── workspace_signals: WorkspaceSignals
├── flow_state: FlowConstraintState
├── created_at: u64
└── status: draft | confirmed | superseded
```

**Behavioral rules**:
- `GoalPlan` is the default planning artifact for `goal -> plan -> run`.
- A confirmed `GoalPlan` is sufficient to route execution to the session-native path.
- `GoalPlan` may be superseded only by bounded replanning behavior that remains inspectable.

### RuntimeDecision

The persisted record of one bounded action chosen during execution.

```text
RuntimeDecision
├── id: String
├── decision_type: analyze | code | test | fix | replan
├── target: String
├── rationale: String
├── expected_outcome: String
├── evidence_inputs: Vec<EvidenceRef>
├── action_result: ToolResult | AgentResult | null
├── status: pending | dispatched | verified | failed | recovered
├── created_at: u64
└── completed_at: Option<u64>
```

**Behavioral rules**:
- Every runtime iteration produces at most one new `RuntimeDecision`.
- Failed decisions remain persisted even when a later recovery decision succeeds.
- Recovery decisions must reference evidence from the failed decision they follow.

### FlowConstraintState

The operator-visible state that determines whether flow is pending, active, or skipped.

```text
FlowConstraintState
├── mode: proposed | confirmed | skipped | absent
├── flow_name: Option<String>
├── confidence_reason: Option<String>
├── current_stage: Option<String>
├── allowed_decision_families: Vec<DecisionType>
└── transition_rule: verifiable_outcome | explicit_operator_change
```

**Behavioral rules**:
- `proposed` means planning inferred a flow but execution must not silently treat it as active.
- `confirmed` means the flow constrains decision selection by stage.
- `skipped` means the operator explicitly chose to run without flow policy.

### RoutingOutcome

The explicit mode-selection result for `run`.

```text
RoutingOutcome
├── mode: native | compatibility | blocked
├── reason: String
├── source: goal_plan | execution_profile | explicit_operator_choice | missing_context
├── operator_override: bool
└── requires_remediation: bool
```

**Behavioral rules**:
- `native` is selected whenever a confirmed or flow-skipped `GoalPlan` is present unless the operator explicitly chooses compatibility mode.
- `compatibility` is selected only when declarative execution is the intended path.
- `blocked` is used instead of silent fallback when required session-native state is incomplete.

### StageBoundaryEvidence

Bounded external input used during planning or stage transitions.

```text
StageBoundaryEvidence
├── kind: canon_artifact | authored_input | trace_summary | workspace_signal
├── reference: String
├── relevance: String
└── consumed_at: planning | stage_transition
```

**Behavioral rules**:
- Canon artifacts may appear here, but they do not choose per-action runtime decisions.
- Evidence must remain inspectable and attributable to a bounded planning or transition point.

## Modified Aggregate

### ActiveSessionRecord

The active session record becomes the operator-facing aggregate for the refounded runtime.

```text
ActiveSessionRecord
├── session_id: String
├── workspace_ref: String
├── goal: Option<String>
├── authored_brief: Option<AuthoredBriefBundle>
├── goal_plan: Option<GoalPlan>
├── decisions: Vec<RuntimeDecision>
├── flow_state: Option<FlowConstraintState>
├── routing_outcome: Option<RoutingOutcome>
├── latest_status: SessionStatus
├── latest_terminal_reason: Option<TerminalReason>
├── latest_trace_ref: Option<String>
└── updated_at: u64
```

**Aggregate rules**:
- Session status must be explainable from `goal_plan`, `decisions`, `flow_state`, and `routing_outcome` without requiring implicit fixture assumptions.
- Route choice, latest failure, and latest terminal reason must be derivable from session state plus trace state.

## Relationships

- `GoalPlan` seeds the initial set of candidate bounded tasks for `RuntimeDecision` selection.
- `FlowConstraintState` constrains which `RuntimeDecision` families are legal at a given stage.
- `RoutingOutcome` determines whether the session executes through `RuntimeDecision` or compatibility behavior.
- `StageBoundaryEvidence` enriches `GoalPlan` derivation and flow or stage transitions without replacing Boundline-owned control flow.

## State Transitions

### GoalPlan lifecycle

```text
draft ──confirm──→ confirmed ──replan──→ superseded
```

### FlowConstraintState lifecycle

```text
absent ──infer──→ proposed ──confirm──→ confirmed
                  └─skip──→ skipped
```

### RuntimeDecision lifecycle

```text
pending ──dispatch──→ dispatched ──verify_ok──→ verified
                             └──verify_fail──→ failed ──recover──→ recovered
```

### RoutingOutcome lifecycle

```text
blocked ──remediate──→ native
blocked ──explicit_compatibility──→ compatibility
```