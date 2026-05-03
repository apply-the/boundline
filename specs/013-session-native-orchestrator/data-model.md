# Data Model: Session-Native Orchestrator

**Feature**: 013-session-native-orchestrator  
**Date**: 2026-04-29

## New Entities

### Decision

The atomic unit of the execution loop. Each iteration of observe→decide→act→verify→update
produces exactly one Decision.

```text
Decision
├── id: String (UUID)
├── decision_type: DecisionType
│   ├── Analyze
│   ├── Code
│   ├── Test
│   ├── Fix
│   └── Replan
├── target: String (file path, test name, or subsystem identifier)
├── rationale: String (human-readable explanation)
├── expected_outcome: String (verifiable claim)
├── evidence_inputs: Vec<EvidenceRef>
│   └── EvidenceRef
│       ├── kind: trace | file | canon | tool_result
│       └── reference: String (event ID, file path, artifact path)
├── status: DecisionStatus
│   ├── Pending
│   ├── Dispatched
│   ├── Verified
│   ├── Failed
│   └── Recovered
├── tool_result: Option<ToolResult> (populated after act phase)
├── created_at: u64 (timestamp millis)
└── completed_at: Option<u64> (timestamp millis)
```

**Relationships**:
- Decision references evidence from prior decisions, tool results, and files
- Decision status transitions: Pending → Dispatched → Verified | Failed → Recovered
- Decision is persisted in session state (`session.decisions[]`) and in trace events

**Validation**:
- `id` must be non-empty UUID
- `decision_type` must be a valid variant
- `target` must be non-empty
- `rationale` must be non-empty
- `evidence_inputs` may be empty for the first decision in a session

### GoalPlan

A bounded task draft derived from goal text, workspace state, collected documents,
and Canon artifacts. Created during `boundline plan`, consumed during `boundline run`.

```text
GoalPlan
├── plan_id: String (UUID)
├── goal_text: String (captured goal from session)
├── tasks: Vec<PlannedTask>
│   └── PlannedTask
│       ├── task_id: String
│       ├── description: String
│       ├── target: String (file or subsystem)
│       ├── expected_outcome: String
│       └── decision_type_hint: Option<DecisionType>
├── source_evidence: Vec<EvidenceRef>
├── workspace_signals: WorkspaceSignals
│   ├── language: Option<String> (detected from manifest)
│   ├── file_count: usize
│   ├── has_config: bool (.boundline/config.toml exists)
│   ├── has_canon: bool (.canon/ exists)
│   └── has_tests: bool (test directory or test files detected)
├── flow: Option<InferredFlow>
│   ├── flow_name: String
│   ├── confidence_reason: String
│   └── confirmed: bool
├── created_at: u64
└── status: GoalPlanStatus
    ├── Draft
    ├── Confirmed
    └── Superseded
```

**Relationships**:
- GoalPlan is stored in session state under `session.goal_plan`
- GoalPlan tasks are converted to Plan steps when `boundline run` begins
- GoalPlan may be superseded by a Replan decision during execution

**Validation**:
- `tasks` must be non-empty
- `goal_text` must be non-empty
- Each `PlannedTask` must have non-empty `task_id`, `description`, and `target`

### FlowPolicy

Maps flow stages to allowed decision types. Derived from flow metadata when a
flow is active.

```text
FlowPolicy
├── flow_name: String
├── stage_policies: Vec<StagePolicy>
│   └── StagePolicy
│       ├── stage_id: String
│       ├── allowed_decisions: Vec<DecisionType>
│       └── transition_condition: TransitionCondition
│           ├── AllVerified (all decisions in stage verified)
│           └── ExplicitAdvance (operator confirms)
└── current_stage_index: usize
```

**Built-in Policies**:

| Flow     | Stage             | Allowed Decisions       | Transition Condition |
| -------- | ----------------- | ----------------------- | -------------------- |
| bug-fix  | investigate       | Analyze                 | AllVerified          |
| bug-fix  | implement         | Code, Fix               | AllVerified          |
| bug-fix  | verify            | Test, Replan            | AllVerified          |
| change   | understand-change | Analyze                 | AllVerified          |
| change   | implement         | Code, Fix               | AllVerified          |
| change   | verify            | Test, Replan            | AllVerified          |
| delivery | requirements      | Analyze                 | AllVerified          |
| delivery | architecture      | Analyze, Code           | AllVerified          |
| delivery | backlog           | Analyze, Replan         | AllVerified          |
| delivery | implementation    | Code, Test, Fix, Replan | AllVerified          |

**Relationships**:
- FlowPolicy is derived from FlowDefinition + hardcoded policy tables
- FlowPolicy constrains DecisionType selection in the decide phase
- Stage transitions are recorded as trace events

### ToolResult

Structured output from a tool adapter invocation.

```text
ToolResult
├── tool_id: String
├── invocation: String (command or operation description)
├── exit_code: Option<i32>
├── stdout: String
├── stderr: String
├── diff: Option<String> (file diff if applicable)
├── duration_ms: u64
└── success: bool
```

**Relationships**:
- ToolResult is attached to the Decision that triggered it
- ToolResult fields are used as evidence inputs for the next decision
- ToolResult maps to existing StepExecutionResult output

**Validation**:
- `tool_id` must be non-empty
- `invocation` must be non-empty

### EvidenceRef

Reference to a piece of evidence used as input to a decision.

```text
EvidenceRef
├── kind: EvidenceKind
│   ├── Trace (reference to a trace event by event_id)
│   ├── File (reference to a workspace file by path)
│   ├── Canon (reference to a Canon artifact by path)
│   └── ToolOutput (reference to a ToolResult by decision_id)
└── reference: String
```

## Modified Entities

### Session (existing: `src/domain/session.rs`)

Add fields:
- `goal_plan: Option<GoalPlan>` — populated by `boundline plan`, consumed by `boundline run`
- `decisions: Vec<Decision>` — decision log for the current execution
- `active_flow_policy: Option<FlowPolicy>` — active flow constraints

### FlowDefinition (existing: `src/domain/flow.rs`)

Extend `FlowStageDefinition` with:
- `allowed_decision_types: &'static [DecisionType]` — which decision types are valid at this stage

### StepExecutionResult (existing: `src/domain/step.rs`)

No structural change. `ToolResult` is built from `StepExecutionResult` output plus
command metadata, not embedded within it.

## State Transitions

### Decision Status

```text
Pending ──dispatch──→ Dispatched ──verify_ok──→ Verified
                            │
                            └──verify_fail──→ Failed ──recover──→ Recovered
```

### GoalPlan Status

```text
Draft ──confirm──→ Confirmed ──replan──→ Superseded
```

### Flow Stage (existing, extended)

```text
Stage[n] ──all_verified──→ Stage[n+1] ──...──→ Terminal
                                                 │
                                          ┌──────┘
                                          ↓
                                       Success
```
