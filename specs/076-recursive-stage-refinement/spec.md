# Feature Specification: Recursive Stage Refinement Profiles

**Feature Branch**: `076-recursive-stage-refinement`

**Created**: 2026-06-07

**Status**: Draft

**Input**: User description: "Recursive Stage Refinement Profiles — bounded, inspectable stage-refinement loops over the existing session-native runtime with compact structured round packets, hard round limits, no-progress detection, blocker stops, and trace-visible outcomes."

## Clarifications

### Session 2026-06-07

- Q: How is the refinement profile enabled — CLI flag, workspace config, or both? → A: Both. Primary mechanism is `.boundline/refinement-profiles.toml` mapping stages to profiles with `stage`, `profile`, `enabled`, `max_rounds`, `max_elapsed_time_seconds`, and a `[roles]` section with `planner_provider_id`, `critic_provider_id`, `finalizer_provider_id`. CLI overrides (`--refine`, `--no-refine`, `--max-rounds N`) override workspace config for the current command only. `--no-refine` bypasses refinement without editing the config. The trace records whether activation came from config or CLI.
- Q: How are planner/critic/finalizer roles mapped to providers? → A: Via a `[roles]` section in `refinement-profiles.toml` with explicit `planner_provider_id`, `critic_provider_id`, `finalizer_provider_id` fields. Each ID must resolve through the existing provider registry. Providers must pass health and permission admission before the loop starts. No provider connection details are allowed inline. The trace records the role-to-provider mapping.
- Q: What constitutes a "material delta" for the closure check? → A: Structural or semantic plan changes that affect execution: task count, ordering, dependency graph, scope boundary, validation strategy, risk/blocker handling, execution readiness, or unresolved finding status changes. Pure wording, formatting, or rephrasing is not material. The runtime validates materiality against the structured plan representation; it does not rely solely on provider self-assessment.
- Q: What is the confidence scale and who determines it? → A: Structured enum (`insufficient`/`low`/`sufficient`/`high`). The critic proposes a value; the runtime validates against findings, blockers, and deltas. The runtime may downgrade but not silently upgrade. `high` is forbidden when blocking findings are unresolved. The round packet records both `critic_confidence` and `effective_confidence` with adjustment reason when they differ.
- Q: What are the defaults for max_rounds and max_elapsed_time? → A: Built-in defaults: `max_rounds = 3`, `max_elapsed_time_seconds = 300`. `max_rounds` must be ≥ 1 after resolving config and CLI overrides; zero values fail visibly. The trace records the effective limits and their source (config, CLI, or built-in). There must never be an unbounded refinement loop.. See ./specs/076-recursive-stage-refinement/spec-recursive-stage-refinement-profiles.md for details

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run a Bounded Planning Refinement Loop (Priority: P1)

An operator enables a sequential planning refinement profile (`planner → critic → planner → finalizer`) for a workspace. When `boundline plan` executes, the runtime activates the profile, produces a planning candidate, submits it for structured critique, applies bounded revision deltas, and produces a final plan — or stops explicitly if no material improvement occurs within the configured round limit.

**Why this priority**: This is the first slice — one refinement profile for one stage (planning). It proves the refinement loop architecture without introducing unbounded agent debate. Every downstream refinement profile depends on this foundation.

**Independent Test**: Enable the `plan_refinement` profile on a workspace, run `boundline plan`, and verify that (a) at least one critique-revision round executes, (b) a compact round packet is emitted per round, (c) the loop stops at or before `max_rounds`, and (d) `boundline inspect` surfaces the active profile, current round, stop reason, and final outcome.

**Acceptance Scenarios**:

1. **Given** a workspace with the `plan_refinement` profile enabled and `max_rounds = 3`, **When** `boundline plan` executes, **Then** the runtime produces an initial planner candidate, submits it to the critic, applies revision deltas, and either produces a final plan or stops with a no-progress reason within 3 rounds.
2. **Given** a refinement loop where the critic finds no material issues with the second-round candidate, **When** the closure check runs, **Then** the loop stops at round 2 with stop reason `no_material_delta` and the final plan is produced.
3. **Given** a refinement loop where a blocking finding from the critic remains unresolved after `max_rounds`, **When** the loop exhausts the round budget, **Then** the runtime stops with stop reason `round_limit_exhausted` and the outcome is `incomplete`.
4. **Given** a provider failure during the critic phase, **When** the refinement loop is mid-execution, **Then** the runtime follows the existing claimed-stage failure boundary — the stage fails visibly, the loop stops, and the failure is trace-visible.

---

### User Story 2 - Inspect Refinement State and Stop Reasons (Priority: P2)

An operator runs `boundline inspect`, `boundline status`, or `boundline next` and sees the active refinement profile, the current round number, findings from the last round, the stop reason (if stopped), and the final stage outcome. The operator never needs to read intermediate transcripts to understand why a round started or stopped.

**Why this priority**: Refinement without inspectability is prompt theater. Operators must understand why extra rounds happened and whether they improved the outcome. This story ensures every refinement decision is trace-visible.

**Independent Test**: Run a refinement loop, then execute `boundline inspect` and verify the output includes profile activation, round history, stop reason, and final outcome — all in compact inspectable form, not raw transcripts.

**Acceptance Scenarios**:

1. **Given** a completed refinement loop that stopped at round 2 with `no_material_delta`, **When** the operator runs `boundline inspect`, **Then** the output shows profile `plan_refinement`, rounds 1-2, stop reason `no_material_delta`, and the final plan artifact reference.
2. **Given** an active refinement loop mid-execution, **When** the operator runs `boundline status`, **Then** the output shows the current stage (`plan`), active profile (`plan_refinement`), current round, and the next action (`continue refinement` or `finalize`).
3. **Given** a refinement loop that exhausted its round budget with unresolved findings, **When** the operator runs `boundline next`, **Then** the output recommends resolving the blocking findings before re-running the plan stage.

---

### User Story 3 - Refinement Produces Compact Trace-Linked Packets (Priority: P3)

Every round in a refinement loop produces one compact structured round packet referencing artifacts (candidate, findings, deltas) instead of copying full transcripts or source files. Packets are trace-linked, versioned, and deduplicated across the session trace.

**Why this priority**: Compact packets prevent trace bloat and keep the refinement history inspectable without overwhelming storage or operator attention. This is the structural foundation that makes refinement sustainable at scale.

**Independent Test**: Run a 3-round refinement loop, export the trace as JSONL, and verify that each round has exactly one packet with fields `schema_version`, `profile`, `stage`, `round`, `candidate_ref`, `findings`, `requested_deltas`, `applied_deltas`, `critic_confidence`, `effective_confidence`, `confidence_adjustment_reason`, `stop_reason` — and that no packet copies full artifact content inline.

**Acceptance Scenarios**:

1. **Given** a refinement round that produced a plan candidate, **When** the round packet is persisted, **Then** the packet's `candidate_ref` is a trace artifact reference (e.g., `trace://plan-candidate-2`), not an inline copy of the plan text.
2. **Given** two consecutive refinement rounds with identical findings, **When** the second round packet is persisted, **Then** the `findings` field references the same finding IDs as the prior round where no new findings were added, avoiding duplication.
3. **Given** a refinement loop that applied three specific deltas, **When** the trace is exported as JSONL, **Then** each delta is recorded in the `applied_deltas` array of the corresponding round packet with its provenance.

---

### Edge Cases

- What happens when the `plan_refinement` profile is enabled but `max_rounds` is zero or unset? Unset uses built-in defaults (`max_rounds=3`, `max_elapsed_time_seconds=300`). A value of zero fails visibly before any refinement round starts.
- What happens when `max_elapsed_time` is zero? The runtime must fail visibly before any refinement round starts; a zero time budget is invalid.
- What happens when the critic provider is the same as the planner provider? The runtime must still execute the loop — role separability is a configuration concern, not a runtime restriction.
- What happens when a round packet is malformed or missing required fields? The runtime must fail the stage visibly and emit a structured error; it must not silently skip the round.
- What happens when a revision delta references an artifact that doesn't exist? The runtime must reject the delta and record the rejection reason in the round packet.
- What happens when `max_elapsed_time` is exceeded mid-round? The current round must complete (no partial artifacts), then the loop must stop with stop reason `time_limit_exhausted`.
- What happens if a provider returns an empty candidate? The loop stops with stop reason `empty_candidate` and the previous round's outcome is preserved.
- What happens when a provider call times out during a round? The runtime must not wait indefinitely. Provider timeout or elapsed-time exhaustion must fail or stop visibly without producing partial artifacts.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support refinement profile activation through `.boundline/refinement-profiles.toml` (primary) and CLI flags `--refine` / `--no-refine` / `--max-rounds N` (per-run overrides). The trace MUST record whether activation came from config or CLI.
- **FR-002**: System MUST resolve refinement roles to provider IDs through the existing provider registry. Each provider MUST pass health and permission admission before the refinement loop starts. Missing, inactive, or unauthorized providers MUST fail visibly before the first round.
- **FR-003**: System MUST enforce a hard `max_rounds` limit per refinement loop; the loop MUST stop when the limit is reached regardless of outcome quality. Built-in defaults: `max_rounds = 3`, `max_elapsed_time_seconds = 300`. `max_rounds` MUST be ≥ 1 after resolving config and CLI overrides; zero values fail visibly. The trace MUST record the effective limits and their source (config, CLI, or built-in).
- **FR-004**: System MUST evaluate a closure check after each round: if no material delta exists between the current and previous plan candidate, the loop MUST stop with stop reason `no_material_delta`. A material delta is a structural or semantic change affecting execution — task count, ordering, dependency graph, scope boundary, validation strategy, risk/blocker handling, execution readiness, or unresolved finding status. Wording, formatting, or rephrasing without execution impact is not material. The runtime MUST validate materiality against the structured plan representation and MUST NOT rely solely on provider self-assessment.
- **FR-005**: System MUST persist one compact structured round packet per refinement round containing: `schema_version`, `profile`, `stage`, `round`, `candidate_ref`, `findings`, `requested_deltas`, `applied_deltas`, `critic_confidence`, `effective_confidence`, `confidence_adjustment_reason`, and `stop_reason`.
- **FR-006**: System MUST reference artifacts by trace identifier (e.g., `trace://plan-candidate-N`) in round packets rather than copying full artifact content inline.
- **FR-007**: System MUST expose the active refinement profile, current round, findings, stop reason, and final outcome through `boundline status`, `boundline next`, and `boundline inspect`.
- **FR-008**: System MUST NOT allow a refinement loop with unresolved blocking findings to produce a `finalized` outcome; the outcome MUST be `incomplete` with the blocking findings listed.
- **FR-009**: System MUST stop the refinement loop when `max_elapsed_time` is exceeded, completing the current round before stopping. The runtime MUST NOT wait indefinitely for a provider call; provider timeout or elapsed-time exhaustion MUST fail or stop visibly without partial artifacts.
- **FR-010**: System MUST reuse existing session, trace, finding, and stop-semantics surfaces; it MUST NOT create a parallel orchestration or trace system.
- **FR-011**: System MUST emit a structured runtime event (`refinement.round.completed`) into the trace for each completed round with the round packet as payload.

### Stop Reason Vocabulary

The `stop_reason` field in a round packet MUST be one of:

- `no_material_delta` — closure check found no structural or semantic change between rounds
- `round_limit_exhausted` — max_rounds reached
- `time_limit_exhausted` — max_elapsed_time exceeded
- `empty_candidate` — provider returned an empty or missing candidate
- `unresolved_blocker` — blocking findings remain and round budget exhausted
- `provider_failure` — provider failed mid-round
- `malformed_packet` — round packet missing required fields or structurally invalid
- `invalid_delta` — a requested or applied delta references a non-existent artifact
- `invalid_configuration` — config validation failed (zero limits, missing provider, etc.)

### Key Entities

- **Refinement Profile**: A named, versioned configuration enabling a specific refinement pattern for a specific stage. The first profile is `plan_refinement` with the pattern `planner → critic → planner → finalizer`.
- **Round Packet**: A compact structured record of one refinement round, containing the candidate reference, findings, requested and applied deltas, confidence assessment, and stop reason. Persisted in the trace store and linked to the session.
- **Closure Check**: An evaluation executed after each round that determines whether refinement should continue or stop based on: material delta existence, round budget exhaustion, time budget exhaustion, and unresolved blocking findings.
- **Revision Delta**: A structured description of a change to a stage artifact, requested by the critic and applied by the planner. Deltas reference artifacts by trace identifier.
- **Refinement Outcome**: The final result of a refinement loop — either a `finalized` stage artifact or an `incomplete` outcome with stop reason and outstanding findings. The term `success` is not used as an outcome label; `finalized` means the artifact is ready for the next stage, `incomplete` means it is not.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can enable the `plan_refinement` profile and complete a bounded refinement loop within 5 minutes of first use, guided by `boundline status` and `boundline next` output.
- **SC-002**: The runtime stops every refinement loop at or before the configured `max_rounds`; no loop exceeds its round budget by even one round.
- **SC-003**: 100% of refinement rounds produce a trace-linked compact packet with all required fields present and no inline artifact content.
- **SC-004**: A refinement loop with an unresolved blocking finding never produces a `finalized` outcome.
- **SC-005**: `boundline inspect` surfaces the complete refinement history (profile, rounds, stop reason, outcome) within 30 seconds of the loop completing.
- **SC-006**: The feature remains fully functional and testable without `sqlite-vec`; all acceptance scenarios pass with only the existing trace store.

## Assumptions

- The existing `boundline plan` command and session-native runtime provide the stage execution surface that refinement profiles extend.
- Council review and finding resolution remain owned by feature 074; this feature consumes findings as inputs but does not modify council behavior.
- Confidence, degradation, and escalation remain owned by feature 075; refinement loops stop or escalate based on calibration policy, not on their own authority.
- Route cost policy remains owned by feature 14; this feature enforces `max_elapsed_time` but does not compute or enforce monetary cost.
- Providers are already registered and activated through the existing provider protocol (feature 071); this feature does not add new provider registration surfaces.
- The first slice supports exactly one stage (plan) and one profile (plan_refinement); multi-stage and multi-profile expansion is deferred to later slices.
- Round packet type is versioned with a `schema_version` field to enable future evolution.
