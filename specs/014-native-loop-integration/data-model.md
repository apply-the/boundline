# Data Model: Native Loop Integration

**Feature**: 014-native-loop-integration  
**Date**: 2026-04-29

## Modified Entities

### ActiveSessionRecord

The active session record becomes the authoritative state container for native planning and native execution routing.

```text
ActiveSessionRecord
├── session_id: String
├── workspace_ref: String
├── goal: Option<String>
├── authored_brief: Option<AuthoredBriefBundle>
├── active_flow: Option<SessionFlowState>
├── active_task: Option<Task>
├── goal_plan: Option<GoalPlan>
├── decisions: Vec<Decision>
├── active_flow_policy: Option<FlowPolicy>
├── latest_status: SessionStatus
├── latest_terminal_reason: Option<TerminalReason>
├── latest_trace_ref: Option<String>
├── created_at: u64
└── updated_at: u64
```

**New behavioral rules**:
- A session in planned native state may be represented by `goal_plan` even when `active_task` is absent.
- `decisions` records the persisted native-loop history for the current or most recent run.
- `active_flow_policy` is present only when a flow has been explicitly confirmed or otherwise made authoritative for execution.

**Validation changes**:
- `SessionStatus::Planned` must accept a session-native planned state when `goal_plan` exists.
- `SessionStatus::Running` on the native path must allow `decisions` plus trace state to serve as the active execution history.
- Route-selection failures must remain explicit instead of being normalized into silent fixture fallback.

### GoalPlan

The existing goal plan becomes a durable planning record for the primary session path rather than advisory metadata.

```text
GoalPlan
├── plan_id: String
├── goal_text: String
├── tasks: Vec<PlannedTask>
├── source_evidence: Vec<EvidenceRef>
├── workspace_signals: WorkspaceSignals
├── flow: Option<InferredFlow>
├── created_at: u64
└── status: GoalPlanStatus
```

**New behavioral rules**:
- `flow.confirmed = false` represents a pending operator decision, not an active execution constraint.
- `flow.confirmed = true` means the plan may seed `active_flow` and `active_flow_policy`.
- `flow = None` after planning means execution proceeds unconstrained by flow.

### Decision

The decision entity remains the loop primitive introduced in feature 013, but it now becomes session-persisted runtime state rather than a local return value only.

```text
Decision
├── id: String
├── decision_type: DecisionType
├── target: String
├── rationale: String
├── expected_outcome: String
├── evidence_inputs: Vec<EvidenceRef>
├── status: DecisionStatus
├── tool_result: Option<ToolResult>
├── created_at: u64
└── completed_at: Option<u64>
```

**New behavioral rules**:
- Decisions are appended to `session.decisions` in execution order.
- Failed decisions remain in session state even when a recovery decision follows.
- Recovery decisions must reference failure evidence from a prior persisted decision.

### CompatibilityRoutingOutcome

A new conceptual routing state used by the session runtime and CLI to choose between native execution and fixture compatibility.

```text
CompatibilityRoutingOutcome
├── mode: native | fixture | blocked
├── reason: String
├── requires_flow_confirmation: bool
└── source: goal_plan | execution_profile | explicit_operator_choice | missing_context
```

**Purpose**:
- Makes route selection explicit and testable.
- Prevents hidden fallback behavior.
- Allows CLI surfaces to tell the operator what to do next.

## Relationships

- `ActiveSessionRecord.goal_plan` determines whether the session-native planning path is ready for execution.
- `ActiveSessionRecord.active_flow_policy` is derived from a confirmed flow and constrains decision selection.
- `ActiveSessionRecord.decisions` mirrors the decision sequence also emitted into trace events.
- `CompatibilityRoutingOutcome` is derived from session state plus explicit operator intent and selects either `DecisionLoop` or fixture compatibility.

## State Transitions

### Native Planning State

```text
GoalCaptured
  └── plan succeeds ──→ Planned(goal_plan present)
  └── plan blocked by unresolved clarification ──→ GoalCaptured
```

### Flow Confirmation State

```text
No proposal
  └── infer flow ──→ Proposed(flow.confirmed = false)
  └── explicit --flow ──→ Confirmed(flow.confirmed = true)
  └── explicit --no-flow ──→ NoFlowConfirmed(flow = None)
```

### Native Run Routing

```text
Planned(goal_plan present)
  └── run without block ──→ NativeLoop
  └── run with explicit compatibility opt-in ──→ FixtureCompatibility
  └── run with unresolved flow confirmation block ──→ BlockedWithRemediation
```

### Decision Persistence

```text
Pending
  └── dispatched ──→ Dispatched
       └── verified ──→ Verified
       └── failed ──→ Failed
            └── recovery chosen ──→ Recovered (original decision retained)
```
