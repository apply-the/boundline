# Data Model: Recursive Stage Refinement Profiles

**Feature**: 076-recursive-stage-refinement
**Date**: 2026-06-07

## Entity Overview

```
RefinementProfile ──1:N──> RoundPacket
        │                       │
        │                       ├── candidate_ref ──> trace://plan-candidate-N
        │                       ├── findings ──> [FindingId] (existing finding system)
        │                       ├── requested_deltas ──> [RevisionDelta]
        │                       ├── applied_deltas ──> [RevisionDelta]
        │                       ├── critic_confidence ──> Confidence
        │                       ├── effective_confidence ──> Confidence
        │                       └── stop_reason ──> StopReason
        │
        └── roles ──> {planner_provider_id, critic_provider_id, finalizer_provider_id}
                           │
                           └── resolved via AgentRegistry / FrameworkAdapterProfileRegistry
```

## Entities

### RefinementProfile

A named, versioned configuration enabling a specific refinement pattern for a specific stage.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `profile` | `String` | Yes | Profile name, e.g., `"plan_refinement"` |
| `stage` | `String` | Yes | Stage this profile applies to, e.g., `"plan"` |
| `enabled` | `bool` | Yes | Whether refinement is active for this stage |
| `max_rounds` | `u32` | Yes | Hard round limit; must be ≥ 1 after resolving config + CLI |
| `max_elapsed_time_seconds` | `u64` | Yes | Hard time limit in seconds; zero is invalid |
| `roles` | `RefinementRoles` | Yes | Provider ID mapping for planner, critic, finalizer |

**Validation Rules**:
- `max_rounds` must be ≥ 1 (zero fails visibly before loop starts)
- `max_elapsed_time_seconds` must be > 0 (zero fails visibly)
- All three role provider IDs must resolve through the provider registry
- Missing, inactive, or unauthorized providers fail visibly before the first round

**Persistence**: `.boundline/refinement-profiles.toml`

**State Transitions**: Profiles are statically configured; no runtime state transitions. Activation is determined by `enabled` flag and CLI overrides.

### RefinementRoles

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `planner_provider_id` | `String` | Yes | Provider ID for the planner role |
| `critic_provider_id` | `String` | Yes | Provider ID for the critic role |
| `finalizer_provider_id` | `String` | Yes | Provider ID for the finalizer role |

### RoundPacket

A compact structured record of one refinement round. Persisted in the trace store and linked to the session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | `String` | Yes | Schema version, e.g., `"1.0"` |
| `profile` | `String` | Yes | Profile name, e.g., `"plan_refinement"` |
| `stage` | `String` | Yes | Stage name, e.g., `"plan"` |
| `round` | `u32` | Yes | 1-based round number within the loop |
| `candidate_ref` | `String` | Yes | Trace artifact reference, e.g., `"trace://plan-candidate-2"` |
| `findings` | `Vec<FindingId>` | Yes | References to findings from this round (existing finding IDs) |
| `requested_deltas` | `Vec<RevisionDelta>` | Yes | Deltas the critic requested for this round |
| `applied_deltas` | `Vec<RevisionDelta>` | Yes | Deltas the planner applied for this round |
| `critic_confidence` | `Confidence` | Yes | Confidence level proposed by the critic |
| `effective_confidence` | `Confidence` | Yes | Confidence level validated by the runtime |
| `confidence_adjustment_reason` | `Option<ConfidenceAdjustment>` | Yes | Reason for adjustment when critic and effective differ |
| `stop_reason` | `StopReason` | No | Reason the loop stopped (absent if loop continues) |

**Validation Rules**:
- All required fields must be present; malformed packets fail the stage visibly
- `candidate_ref` must be a trace artifact reference, never inline content
- `effective_confidence` must not be `High` when blocking findings are unresolved
- `effective_confidence` may be lower than `critic_confidence` but never higher
- Duplicate finding references across consecutive rounds are permitted (deduplication is a concern for the finding system, not the packet)
- When `round` > 1, `candidate_ref` must reference the updated candidate from the prior round

**Persistence**: Emitted as `TraceEventType::RefinementRoundCompleted` events in the trace store.

**Relationships**:
- `candidate_ref` → links to a plan artifact in the trace store
- `findings` → links to existing `Finding` entities in the review/finding system
- `requested_deltas` / `applied_deltas` → revision deltas referencing artifacts by trace ID

### Confidence

A structured enum representing the assessed quality of a plan candidate.

| Variant | Ordinal | Description |
|---------|---------|-------------|
| `Insufficient` | 0 | Plan is not ready; significant rework needed |
| `Low` | 1 | Plan has major gaps; requires substantial revision |
| `Sufficient` | 2 | Plan is adequate; may proceed with noted findings |
| `High` | 3 | Plan is thorough and complete; no material issues |

**Validation Rule**: `High` is forbidden when blocking findings are unresolved or high-severity findings are present.

### ConfidenceAdjustment

Reason the runtime adjusted the critic's proposed confidence level.

| Variant | Description |
|---------|-------------|
| `BlockersUnresolved` | Blocking findings remain unresolved |
| `HighSeverityFindings` | One or more high-severity findings present |
| `MultipleMediumFindings` | Three or more medium-severity findings present |

### StopReason

Closed vocabulary of reasons a refinement loop stopped.

| Variant | Description | Category |
|---------|-------------|----------|
| `NoMaterialDelta` | Closure check found no structural or semantic change | Quality gate |
| `RoundLimitExhausted` | `max_rounds` reached | Budget exhaustion |
| `TimeLimitExhausted` | `max_elapsed_time` exceeded | Budget exhaustion |
| `EmptyCandidate` | Provider returned an empty or missing candidate | Provider error |
| `UnresolvedBlocker` | Blocking findings remain after round budget exhausted | Quality gate |
| `ProviderFailure` | Provider failed mid-round | Provider error |
| `MalformedPacket` | Round packet missing required fields or structurally invalid | System error |
| `InvalidDelta` | A requested or applied delta references a non-existent artifact | System error |
| `InvalidConfiguration` | Config validation failed (zero limits, missing provider, etc.) | System error |

**Validation Rule**: The `stop_reason` field in a round packet must be exactly one of these nine values. The runtime must not emit unrecognized values.

### RevisionDelta

A structured description of a change to a stage artifact, requested by the critic and applied by the planner.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `artifact_ref` | `String` | Yes | Trace artifact reference, e.g., `"trace://plan-candidate-2"` |
| `kind` | `DeltaKind` | Yes | Type of change |
| `target` | `String` | Yes | Specific element being changed (task ID, section, dependency edge) |
| `description` | `String` | Yes | Human-readable description of the change |
| `provenance` | `FindingId` | Yes | Finding that motivated this delta |

### DeltaKind

| Variant | Description |
|---------|-------------|
| `AddTask` | Add a new task to the plan |
| `RemoveTask` | Remove a task from the plan |
| `ReorderTask` | Change task ordering |
| `UpdateDependency` | Add or remove a dependency edge |
| `UpdateScope` | Change scope boundary |
| `UpdateValidation` | Change validation strategy |
| `UpdateRisk` | Change risk assessment or mitigation |
| `UpdateBlocker` | Resolve a blocker |

### RefinementOutcome

The final result of a refinement loop.

| Variant | Description |
|---------|-------------|
| `Finalized` | Artifact is ready for the next stage |
| `Incomplete` | Artifact is not ready; stop reason and outstanding findings provided |

**Relationship**: The outcome is derived from the last round packet's `stop_reason` and the presence of unresolved blocking findings. If `stop_reason` is `NoMaterialDelta` and no blockers remain → `Finalized`. Otherwise → `Incomplete`.

### PlanStructureDigest

Internal type used for material delta detection. Not persisted independently; computed per-round.

| Field | Type | Description |
|-------|------|-------------|
| `task_count` | `usize` | Number of tasks in the plan |
| `task_ids_ordered` | `Vec<String>` | Task IDs in execution order |
| `dependency_pairs` | `BTreeSet<(String, String)>` | Dependency edges as (from_id, to_id) pairs |
| `scope_boundary_hash` | `u64` | Hash of scope boundary description |
| `validation_strategy_hash` | `u64` | Hash of validation strategy description |
| `risk_count` | `usize` | Number of identified risks |
| `blocker_count` | `usize` | Number of identified blockers |
| `readiness_flags` | `u8` | Bitmask of readiness indicators |
| `unresolved_finding_ids` | `BTreeSet<String>` | IDs of findings that remain unresolved |

**Comparison Rule**: Two digests are equal (no material delta) when all fields match exactly. Any difference in the enumerated dimensions is material.

## Entity Relationships

```
RefinementProfile (1) ──activates──> RefinementLoop (1 per plan execution)
                                           │
                                           │ produces
                                           ▼
                                    RoundPacket[N] (1 per round)
                                           │
                          ┌────────────────┼────────────────┐
                          ▼                ▼                 ▼
                    candidate_ref      findings          deltas
                    (trace link)    (FindingId[])    (RevisionDelta[])
```

## State Machine

```
                  ┌──────────────────────────────┐
                  │   RefinementLoop::Pending     │
                  └──────────────┬───────────────┘
                                 │ config validated, providers resolved
                                 ▼
                  ┌──────────────────────────────┐
          ┌──────>│   RefinementLoop::Running    │
          │       └──────────────┬───────────────┘
          │                      │ round completed
          │                      ▼
          │       ┌──────────────────────────────┐
          │       │      ClosureCheck            │
          │       └──────────────┬───────────────┘
          │                      │
          │         ┌────────────┴────────────┐
          │         ▼                         ▼
          │   StopReason                 Continue
          │         │                         │
          │         ▼                         │
          │   RefinementLoop::Stopped    ─────┘
          │         │
          │         ▼
          │   RefinementOutcome::Finalized
          │         or
          │   RefinementOutcome::Incomplete
          │
          └── (loop continues to next round)
```
