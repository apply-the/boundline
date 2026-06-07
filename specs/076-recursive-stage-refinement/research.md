# Research: Recursive Stage Refinement Profiles

**Feature**: 076-recursive-stage-refinement
**Date**: 2026-06-07

## 1. Material Delta Detection

### Decision
Compare plan candidates structurally using the existing `GoalPlan` / task representation rather than raw text diffing. A material delta exists when any of these change between rounds: task count, task ordering, dependency graph edges, scope boundary, validation strategy, risk/blocker handling, execution readiness, or unresolved finding status.

### Rationale
- The spec requires runtime-validated materiality, not provider self-assessment.
- Boundline already has structured plan representations (`GoalPlan`, `PlannedTask`, dependency edges) in `src/orchestrator/`.
- Text-level diffing (e.g., `similar` crate) would flag pure wording changes as material, violating the spec.
- Structural comparison is deterministic and testable.

### Alternatives Considered
- **Provider self-assessment**: Rejected — violates spec requirement that runtime validates materiality.
- **LLM-as-judge for delta detection**: Rejected — adds latency, cost, and non-determinism to a gate that must be fast and reliable.
- **Full semantic embedding comparison**: Rejected — would require `sqlite-vec`, which the spec explicitly excludes for this feature.

### Implementation Approach
- Extract a `PlanStructureDigest` from the plan candidate: `{task_count, task_ids_ordered, dependency_pairs, scope_boundary_hash, validation_strategy_hash, risk_count, blocker_count, readiness_flags, unresolved_finding_ids}`.
- Compare digests between rounds; any difference in the enumerated dimensions is material.
- Pure wording changes (which don't alter the digest) are correctly classified as non-material.

## 2. Round Packet Schema Versioning

### Decision
Use a `schema_version` field (string, e.g., `"1.0"`) in every round packet, following the existing pattern in `EventType::schema_version()`. The schema version is owned by the refinement domain, not the observability domain.

### Rationale
- The spec requires `schema_version` in the round packet (FR-005, US3).
- Existing event types already use this pattern (`SCHEMA_VERSION_PLANNING_ANALYSIS`, etc.).
- Versioning enables future packet evolution without breaking trace compatibility.

### Implementation Approach
- Define `const ROUND_PACKET_SCHEMA_VERSION: &str = "1.0";` in `src/domain/refinement.rs`.
- Include `schema_version` as the first field in the `RoundPacket` struct, serialized in every packet.
- Future schema changes increment the version and add migration logic.

## 3. Refinement Profile Configuration Format

### Decision
Use TOML for `.boundline/refinement-profiles.toml`, consistent with existing `.boundline/config.toml`. Structure:

```toml
[profiles.plan_refinement]
enabled = true
max_rounds = 3
max_elapsed_time_seconds = 300

[profiles.plan_refinement.roles]
planner_provider_id = "openai-gpt-5"
critic_provider_id = "openai-gpt-5"
finalizer_provider_id = "openai-gpt-5"
```

### Rationale
- TOML is already the config format for Boundline (`.boundline/config.toml`).
- Per-profile sections allow future multi-profile expansion without format changes.
- Role-to-provider mapping uses `provider_id` naming per the spec clarifications.
- The `[roles]` subsection keeps provider mapping colocated with the profile.

### Alternatives Considered
- **JSON**: Rejected — less human-editable for configuration files.
- **CLI-only flags**: Rejected — spec requires workspace-persistent configuration as primary mechanism.
- **Embedding in `config.toml`**: Considered but rejected — the refinement config is substantial enough to warrant its own file, keeping `config.toml` focused.

## 4. Trace Event Integration

### Decision
Add a new `TraceEventType::RefinementRoundCompleted` variant to the existing trace event system, following the established pattern. Use a typed payload struct (`RefinementRoundCompletedPayload`) rather than ad hoc JSON assembly.

### Rationale
- The existing trace system (`src/domain/trace.rs`, `src/domain/observability.rs`) already has 40+ event type variants with a well-established pattern.
- Adding a new variant is low-risk and follows the existing convention.
- Using a typed payload struct (with `serde` derives) avoids magic string keys and ensures schema consistency.

### Implementation Approach
1. Add `RefinementRoundCompleted` to the `TraceEventType` enum.
2. Define `RefinementRoundCompletedPayload` struct with fields: `schema_version`, `profile`, `stage`, `round`, `candidate_ref`, `findings`, `requested_deltas`, `applied_deltas`, `critic_confidence`, `effective_confidence`, `confidence_adjustment_reason`, `stop_reason`.
3. Emit via `trace.record_event(TraceEventType::RefinementRoundCompleted, json!(payload))` in the refinement orchestrator.
4. The existing `boundline inspect` trace projection system will automatically surface the new event type.

## 5. Provider Role Resolution

### Decision
Resolve refinement roles to providers through the existing `AgentRegistry` and `FrameworkAdapterProfileRegistry` in `src/registry/agent_registry.rs`. The refinement orchestrator reads `planner_provider_id` / `critic_provider_id` / `finalizer_provider_id` from the profile config and resolves each through `AgentRegistry::get(name)` or `FrameworkAdapterProfileRegistry::resolve_profile(id)`.

### Rationale
- The spec requires reusing the existing provider registry (FR-002).
- The registry already supports named agent lookup and adapter profile resolution.
- No new registration surface is needed.

### Implementation Approach
- In the refinement orchestrator startup, read the profile's `[roles]` section.
- For each role, attempt resolution through `AgentRegistry::get(provider_id)`.
- Fail visibly if any provider is missing, inactive, or unauthorized (per FR-002).
- Record the resolved provider mapping in the trace for auditability.

## 6. Confidence Model Validation

### Decision
Implement a `ConfidenceValidator` that takes the critic's proposed `critic_confidence` and the round's findings/blockers/deltas, then produces an `effective_confidence`. The validator may downgrade but never silently upgrade. `high` is forbidden when blocking findings are unresolved.

### Rationale
- The spec requires runtime-validated confidence (clarification Q4).
- The critic's self-assessed confidence may be optimistic; the runtime must enforce the invariant that `high` confidence cannot coexist with unresolved blockers.
- Recording both `critic_confidence` and `effective_confidence` with an adjustment reason preserves auditability.

### Implementation Approach
- Define `Confidence` enum: `Insufficient`, `Low`, `Sufficient`, `High`.
- Define `ConfidenceValidator` with a single method: `validate(critic_confidence, findings, blockers) -> (effective_confidence, adjustment_reason)`.
- Rules: if blockers present → cap at `Sufficient`; if ≥1 high-severity finding → cap at `Sufficient`; if ≥3 medium-severity findings → cap at `Sufficient`.
- The adjustment reason is `None` when critic and effective match; otherwise a structured enum (`BlockersUnresolved`, `HighSeverityFindings`, `MultipleMediumFindings`).

## 7. Closure Check Algorithm

### Decision
The closure check runs after each round in this order:
1. **Invalid configuration?** → stop with `invalid_configuration` (zero limits, missing provider — should be caught before loop start, but defend in depth).
2. **Malformed packet?** → stop with `malformed_packet`.
3. **Invalid delta?** → stop with `invalid_delta`.
4. **Provider failure?** → stop with `provider_failure`.
5. **Empty candidate?** → stop with `empty_candidate`.
6. **Max elapsed time exceeded?** → stop with `time_limit_exhausted`.
7. **No material delta?** → stop with `no_material_delta`.
8. **Max rounds exhausted with unresolved blockers?** → stop with `unresolved_blocker`.
9. **Max rounds exhausted (no blockers)?** → stop with `round_limit_exhausted`.
10. Otherwise → continue to next round.

### Rationale
- Error/failure conditions are checked first (fail fast).
- Budget exhaustion is checked before quality gates.
- The `no_material_delta` check is the primary quality gate — it stops the loop early when convergence is reached.
- Blocking findings must prevent a `finalized` outcome (FR-008).

### Implementation Approach
- Implement as a `ClosureCheck::evaluate()` method returning `StopReason` or `Continue`.
- The orchestrator calls it after each round and branches on the result.
- The stop reason is recorded in the round packet.

## 8. CLI Flag Integration

### Decision
Add `--refine`, `--no-refine`, and `--max-rounds <N>` flags to the existing `boundline plan` subcommand using `clap`. These flags override the workspace config for the current command only. The trace records the activation source (config or CLI).

### Rationale
- The spec requires both config and CLI activation (FR-001).
- CLI overrides enable one-off refinement without editing config files.
- `--no-refine` provides an escape hatch to bypass refinement entirely.

### Implementation Approach
- Add optional fields to the plan command's `clap` struct.
- In the plan execution flow, merge CLI overrides with config defaults (CLI wins).
- Record the effective settings and their source in the trace.

## Summary of Key Design Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Structural plan digest comparison for material delta | Deterministic, testable, no ML dependency |
| 2 | `schema_version` field in round packets | Follows existing event versioning pattern |
| 3 | TOML config in `.boundline/refinement-profiles.toml` | Consistent with existing config format |
| 4 | New `TraceEventType::RefinementRoundCompleted` | Follows existing trace event pattern |
| 5 | Provider resolution via existing `AgentRegistry` | No new registration surface needed |
| 6 | Runtime confidence validation with downgrade-only | Enforces invariant: no high confidence with blockers |
| 7 | Ordered closure check (errors → budget → quality) | Fail-fast, deterministic stop behavior |
| 8 | `--refine`/`--no-refine`/`--max-rounds` CLI flags | Config + CLI overrides per spec |
