# Tasks: Recursive Stage Refinement Profiles

**Input**: Design documents from `specs/076-recursive-stage-refinement/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/
**Version bump**: 0.75.0 → 0.76.0 (feature)

**Tests**: Included per Boundline quality rules. Every behavior-changing task has a corresponding test task.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create new module scaffolding and wire into existing crate structure

- [X] T001 Create refinement domain module at `src/domain/refinement.rs` with module-level doc comment explaining the refinement domain types and their role in bounded stage-refinement loops (note: this file lives in the workspace-root `src/domain/` tree, not under `crates/boundline-core/src/`)
- [X] T002 Add `#[path = "../../../src/domain/refinement.rs"] pub mod refinement;` declaration to `crates/boundline-core/src/domain.rs` following the existing pattern used by calibration, observability, and other domain modules
- [X] T003 [P] Add `pub mod refinement_cmd;` declaration to `crates/boundline-cli/src/cli.rs` for CLI refinement rendering helpers, and create `src/cli/refinement_cmd.rs` with module-level doc comment

---

## Phase 2: Foundational (Domain Types)

**Purpose**: Define all refinement domain types, enums, and validation logic. Must complete before any user story implementation.

**⚠️ CRITICAL**: All domain types must be defined and unit-tested before user story work begins.

### Core Enums

- [X] T004 [P] Define `Confidence` enum (`Insufficient`, `Low`, `Sufficient`, `High`) with `serde` derives in `src/domain/refinement.rs`
- [X] T005 [P] Define `StopReason` enum with all 9 canonical values (`NoMaterialDelta`, `RoundLimitExhausted`, `TimeLimitExhausted`, `EmptyCandidate`, `UnresolvedBlocker`, `ProviderFailure`, `MalformedPacket`, `InvalidDelta`, `InvalidConfiguration`) with `serde` derives in `src/domain/refinement.rs`
- [X] T006 [P] Define `DeltaKind` enum with 8 variants (`AddTask`, `RemoveTask`, `ReorderTask`, `UpdateDependency`, `UpdateScope`, `UpdateValidation`, `UpdateRisk`, `UpdateBlocker`) with `serde` derives in `src/domain/refinement.rs`
- [X] T007 [P] Define `ConfidenceAdjustment` enum (`BlockersUnresolved`, `HighSeverityFindings`, `MultipleMediumFindings`) with `serde` derives in `src/domain/refinement.rs`
- [X] T008 [P] Define `RefinementOutcome` enum (`Finalized`, `Incomplete`) with `serde` derives in `src/domain/refinement.rs`

### Core Structs

- [X] T009 Define `RevisionDelta` struct with fields `artifact_ref: String`, `kind: DeltaKind`, `target: String`, `description: String`, `provenance: FindingId` (use the existing finding identifier type from the review domain; define `FindingId` as a newtype over `String` in `src/domain/refinement.rs` if it does not already exist) and `serde` derives in `src/domain/refinement.rs`
- [X] T010 Define `RefinementRoles` struct with fields `planner_provider_id: String`, `critic_provider_id: String`, `finalizer_provider_id: String` and `serde` derives in `src/domain/refinement.rs`
- [X] T011 Define `RefinementProfile` struct with fields `profile: String`, `stage: String`, `enabled: bool`, `max_rounds: u32`, `max_elapsed_time_seconds: u64`, `roles: RefinementRoles` and `serde` derives (TOML-compatible) in `src/domain/refinement.rs`
- [X] T012 Define `RoundPacket` struct with all required fields (`schema_version`, `profile`, `stage`, `round`, `candidate_ref`, `findings`, `requested_deltas`, `applied_deltas`, `critic_confidence`, `effective_confidence`, `confidence_adjustment_reason`, `stop_reason`) and `serde` derives in `src/domain/refinement.rs`
- [X] T013 Define `PlanStructureDigest` struct with fields for material delta detection (`task_count`, `task_ids_ordered`, `dependency_pairs`, `scope_boundary_hash`, `validation_strategy_hash`, `risk_count`, `blocker_count`, `readiness_flags`, `unresolved_finding_ids`) in `src/domain/refinement.rs`

### Constants

- [X] T014 [P] Define `const ROUND_PACKET_SCHEMA_VERSION: &str = "1.0";` in `src/domain/refinement.rs`
- [X] T015 [P] Define `const DEFAULT_MAX_ROUNDS: u32 = 3;` and `const DEFAULT_MAX_ELAPSED_TIME_SECONDS: u64 = 300;` in `src/domain/refinement.rs`
- [X] T016 [P] Define `const REFINEMENT_CONFIG_FILE: &str = "refinement-profiles.toml";` in `src/domain/refinement.rs`

### Domain Validation

- [X] T017 Implement `Confidence::validate_effective(critic: Confidence, has_blockers: bool, high_severity_count: usize, medium_severity_count: usize) -> (Confidence, Option<ConfidenceAdjustment>)` enforcing: effective never exceeds critic, `High` forbidden with unresolved blockers or high-severity findings, downgrade-only semantics in `src/domain/refinement.rs`
- [X] T018 Implement `PlanStructureDigest::compute_from(plan: &GoalPlan) -> Self` extracting structural features from the plan representation in `src/domain/refinement.rs`
- [X] T019 Implement `PlanStructureDigest::is_material_delta_from(&self, previous: &PlanStructureDigest) -> bool` returning true when any structural dimension differs in `src/domain/refinement.rs`
- [X] T020 Implement `RefinementProfile::validate(&self, registry: &AgentRegistry) -> Result<(), RefinementConfigError>` checking max_rounds ≥ 1, max_elapsed_time_seconds > 0, and all three role provider IDs resolve through registry in `src/domain/refinement.rs`
- [X] T021 Implement `RoundPacket::validate(&self) -> Result<(), RoundPacketValidationError>` checking required fields present, schema_version matches, candidate_ref uses trace:// prefix, round ≥ 1, confidence invariant holds, and when round > 1 candidate_ref must reference the updated candidate from the prior round (cross-round continuity) in `src/domain/refinement.rs`

### Domain Unit Tests

- [X] T022 [P] Create `tests/unit/refinement_model.rs` with unit tests covering all Confidence variants and validation rules (at least 8 tests: each variant, adjustment reasons, High-blocked invariant, downgrade-only, no-upgrade)
- [X] T023 [P] Add unit tests to `tests/unit/refinement_model.rs` covering all StopReason variants serialization/deserialization (9 variant tests)
- [X] T024 [P] Add unit tests to `tests/unit/refinement_model.rs` covering DeltaKind variants (8 variant tests)
- [X] T025 [P] Add unit tests to `tests/unit/refinement_model.rs` covering RevisionDelta struct fields and serialization
- [X] T026 [P] Add unit tests to `tests/unit/refinement_model.rs` covering RefinementRoles struct fields and serialization
- [X] T027 [P] Add unit tests to `tests/unit/refinement_model.rs` covering RefinementProfile TOML deserialization (valid config, missing fields, zero max_rounds, zero time)
- [X] T028 [P] Add unit tests to `tests/unit/refinement_model.rs` covering RoundPacket JSON serialization/deserialization (valid packet, missing fields, malformed candidate_ref, round=0)
- [X] T029 [P] Add unit tests to `tests/unit/refinement_model.rs` covering PlanStructureDigest equality/inequality scenarios (identical plans produce equal digests; task count change, ordering change, dependency change each produce material delta; wording-only change does not)
- [X] T030 [P] Add unit tests to `tests/unit/refinement_model.rs` covering ConfidenceAdjustment enum variants
- [X] T031 [P] Add unit tests to `tests/unit/refinement_model.rs` covering RefinementOutcome enum variants

**Checkpoint**: All domain types defined, validated, and unit-tested. Refinement profiles, round packets, stop reasons, confidence, and deltas are ready for use.

---

## Phase 3: User Story 1 — Run a Bounded Planning Refinement Loop (Priority: P1) 🎯 MVP

**Goal**: Enable the `plan_refinement` profile, run `boundline plan`, and execute a bounded `planner → critic → planner → finalizer` loop producing trace-visible round packets. The loop stops on no-material-delta, round limit, time limit, or blocker.

**Independent Test**: Enable the `plan_refinement` profile on a workspace, run `boundline plan`, and verify that (a) at least one critique-revision round executes, (b) a compact round packet is emitted per round, (c) the loop stops at or before max_rounds, and (d) `boundline inspect` surfaces the active profile, current round, stop reason, and final outcome.

### Config Loading & CLI Integration

- [X] T032 [US1] Implement `RefinementLoopState` enum with variants `Pending`, `Running`, and `Stopped(StopReason)` in `src/domain/refinement.rs` to provide explicit, testable state tracking for the refinement loop lifecycle
- [X] T033 [US1] Implement `load_refinement_profile(workspace_root: &Path, profile_name: &str) -> Result<Option<RefinementProfile>, RefinementConfigError>` reading from `.boundline/refinement-profiles.toml` using TOML deserialization in `src/domain/refinement.rs`
- [X] T034 [US1] Implement `resolve_effective_profile(config_profile: Option<RefinementProfile>, cli_refine: bool, cli_no_refine: bool, cli_max_rounds: Option<u32>, cli_max_elapsed_time: Option<u64>) -> Result<RefinementProfile, RefinementConfigError>` merging config and CLI overrides with built-in defaults in `src/domain/refinement.rs`
- [X] T035 [US1] Add `--refine`, `--no-refine`, and `--max-rounds <N>` optional flags to the `boundline plan` clap subcommand in `src/cli/cli.rs` (or the plan command module)
- [X] T036 [US1] Wire CLI flag values into the plan execution path so `resolve_effective_profile` receives both config and CLI inputs in `src/cli/plan_cmd.rs`

### Provider Resolution

- [X] T037 [US1] Implement `resolve_refinement_roles(roles: &RefinementRoles, registry: &AgentRegistry) -> Result<ResolvedRefinementRoles, RefinementConfigError>` resolving each provider ID through the existing `AgentRegistry` in `src/registry/agent_registry.rs` and returning resolved provider handles (add a `#[path]` declaration in `crates/boundline-adapters/src/registry.rs` if creating a new file)
- [X] T038 [US1] Implement health and permission admission check for each resolved provider before the refinement loop starts, failing visibly with the failing provider ID if any provider is inactive or unauthorized in `src/registry/agent_registry.rs`

### Refinement Orchestrator

- [X] T039 [US1] Implement `execute_refinement_loop(profile: &RefinementProfile, initial_candidate: &GoalPlan, roles: &ResolvedRefinementRoles, trace: &mut ExecutionTrace, session: &Session) -> Result<RefinementOutcome, RefinementError>` as the main refinement orchestrator in `src/orchestrator/refinement.rs` (new file)
- [X] T040 [US1] Implement planner phase: call planner provider with current candidate and previous round's findings, receive updated candidate, compute `PlanStructureDigest` in `src/orchestrator/refinement.rs`
- [X] T041 [US1] Implement critic phase: call critic provider with current candidate, receive structured critique (findings, deltas, critic_confidence) in `src/orchestrator/refinement.rs`
- [X] T042 [US1] Implement finalizer phase: when loop stops, call finalizer provider to produce the final plan artifact. The runtime (not the provider) determines `RefinementOutcome` from the last round packet's `stop_reason` and the presence of unresolved blocking findings per the outcome derivation rules in `data-model.md` in `src/orchestrator/refinement.rs`

### Closure Check & Stop Logic

- [X] T043 [US1] Implement `ClosureCheck` struct with `evaluate(packet: &RoundPacket, digest: &PlanStructureDigest, previous_digest: Option<&PlanStructureDigest>, elapsed: Duration, max_elapsed: Duration) -> Option<StopReason>` enforcing the ordered stop-reason evaluation (invalid config → malformed packet → invalid delta → provider failure → empty candidate → time limit → no material delta → round limit → unresolved blocker) in `src/domain/refinement.rs`
- [X] T044 [US1] Implement material delta detection: compare current `PlanStructureDigest` with previous round's digest; if no structural difference exists, return `StopReason::NoMaterialDelta` in `src/domain/refinement.rs`
- [X] T045 [US1] Implement round budget enforcement: track current round against max_rounds; stop with `StopReason::RoundLimitExhausted` (or `UnresolvedBlocker` if blocking findings remain) in `src/orchestrator/refinement.rs`
- [X] T046 [US1] Implement time budget enforcement: track elapsed time since loop start; stop with `StopReason::TimeLimitExhausted` when max_elapsed_time_seconds is exceeded, completing the current round before stopping in `src/orchestrator/refinement.rs`

### Trace Integration

- [X] T047 [US1] Add `RefinementRoundCompleted` variant to the `TraceEventType` enum in `src/domain/trace.rs` (path-mapped from `crates/boundline-core/src/domain.rs`)
- [X] T048 [US1] Implement `emit_round_completed_event(trace: &mut ExecutionTrace, packet: &RoundPacket)` emitting a trace event with the round packet as payload in `src/orchestrator/refinement.rs`. Also emit trace metadata recording refinement activation source (config or CLI) and effective limits source (config, CLI, or built-in) at loop start, satisfying FR-001 and FR-003 trace requirements.
- [X] T049 [US1] Integrate refinement loop into the plan command execution path: after `plan_task()` produces a candidate, if refinement is enabled, execute the refinement loop and replace the plan candidate with the refined result in `src/cli/plan_cmd.rs`

### Refinement Error Types

- [X] T050 [US1] Define `RefinementConfigError` enum with variants for zero limits, missing provider, inactive provider, unauthorized provider, invalid TOML, missing config file in `src/domain/refinement.rs`
- [X] T051 [US1] Define `RefinementError` enum with variants for provider failure, empty candidate, malformed packet, invalid delta, timeout, and generic execution failure in `src/domain/refinement.rs`
- [X] T052 [US1] Define `RoundPacketValidationError` enum with variants for missing required field, invalid schema version, invalid candidate ref, invalid round number, confidence invariant violation in `src/domain/refinement.rs`

### US1 Tests

- [X] T053 [US1] Create `tests/contract/refinement_config_contract.rs` with contract tests for refinement profile config loading (valid config loads, missing file uses defaults, zero max_rounds fails, zero time fails, unresolved provider fails, CLI override precedence, --no-refine bypass, --refine activation)
- [X] T054 [US1] Create `tests/integration/refinement_flow.rs` with integration tests for end-to-end refinement loop (plan_refinement profile enabled completes at least one round, loop stops at max_rounds, loop stops on no_material_delta, loop stops on time_limit_exhausted, outcome is incomplete when blockers remain, outcome is finalized when loop converges, --no-refine bypasses entirely, same provider for planner and critic still executes loop successfully, provider failure mid-critic phase stops the loop with trace-visible failure and no partial artifact produced per US1 acceptance scenario 4, loop completes within 10-minute timeout)

---

## Phase 4: User Story 2 — Inspect Refinement State and Stop Reasons (Priority: P2)

**Goal**: `boundline inspect`, `boundline status`, and `boundline next` surface the active refinement profile, current round, findings, stop reason, and final outcome.

**Independent Test**: Run a refinement loop, then execute `boundline inspect` and verify the output includes profile activation, round history, stop reason, and final outcome — all in compact inspectable form, not raw transcripts.

### Status Command

- [X] T055 [US2] Implement `render_refinement_status(session: &Session) -> Option<RefinementStatusView>` extracting active refinement state from session and trace data in `src/cli/status_cmd.rs`
- [X] T056 [US2] Integrate refinement status into the existing `boundline status` output: when a refinement loop is active, show profile, stage, current round of max_rounds, and next action; when stopped, show rounds completed, stop reason, and outcome in `src/cli/status_cmd.rs`

### Next Command

- [X] T057 [US2] Implement `suggested_next_after_refinement(outcome: &RefinementOutcome, stop_reason: &Option<StopReason>, findings: &[FindingId]) -> String` producing actionable recommendations: if `Incomplete` with blockers → "Resolve blocking findings before re-running plan stage"; if `Incomplete` with round_limit → "Consider increasing max_rounds"; if `Finalized` → "run" in `src/cli/next_cmd.rs`
- [X] T058 [US2] Integrate refinement-aware next suggestions into the existing `boundline next` command output in `src/cli/next_cmd.rs`

### Inspect Command

- [X] T059 [US2] Implement `render_refinement_inspection(trace: &ExecutionTrace) -> Option<RefinementInspectionView>` extracting round history from trace events and building the refinement section in `src/cli/inspect_cmd.rs`
- [X] T060 [US2] Implement human-readable refinement inspection rendering: show profile, stage, rounds count, stop reason, outcome, then per-round details (candidate_ref, confidence with adjustment reason if any, finding count, delta counts) in `src/cli/inspect_cmd.rs`
- [X] T061 [US2] Implement JSON refinement inspection rendering: produce valid JSON with all round packet fields, confidence adjustment reasons, and stop reasons in `src/cli/inspect_cmd.rs`
- [X] T062 [US2] Ensure inspect output never copies full artifact content inline — only trace references (`trace://plan-candidate-N`) — in `src/cli/inspect_cmd.rs`

### US2 Tests

- [X] T063 [US2] Create `tests/contract/refinement_output_contract.rs` with contract tests for inspect output (inspect after refinement shows profile/rounds/stop reason/outcome, status mid-refinement shows current round, next after blocked refinement recommends resolving findings, inspect without refinement shows no refinement section, JSON output valid, no inline content, confidence adjustment visible when critic/effective differ)

---

## Phase 5: User Story 3 — Refinement Produces Compact Trace-Linked Packets (Priority: P3)

**Goal**: Every round produces exactly one compact structured round packet with all required fields, trace artifact references, and no inline content. Packets are schema-versioned and deduplicated.

**Independent Test**: Run a 3-round refinement loop, export the trace as JSONL, and verify that each round has exactly one packet with all required fields — and that no packet copies full artifact content inline.

### Round Packet Schema Validation

- [X] T064 [US3] Implement `RoundPacket::to_json_value(&self) -> serde_json::Value` serializing the packet with all required fields present, schema_version first, and no inline artifact content in `src/domain/refinement.rs`
- [X] T065 [US3] Implement `RoundPacket::from_json_value(value: &serde_json::Value) -> Result<RoundPacket, RoundPacketValidationError>` deserializing and validating per the round packet schema contract in `src/domain/refinement.rs`
- [X] T066 [US3] Implement schema version enforcement: round packets with a `schema_version` field that does not match `ROUND_PACKET_SCHEMA_VERSION` must be rejected with a clear error in `src/domain/refinement.rs`

### Trace Event Integration

- [X] T067 [US3] Add `schema_version()` and `type_name()` implementations for the new `RefinementRoundCompleted` event type variant in the `EventType` enum in `src/domain/observability.rs`
- [X] T068 [US3] Implement trace projection for refinement events: `boundline inspect` must surface `RefinementRoundCompleted` events as part of the refinement history section in `src/cli/inspect_cmd.rs`
- [X] T069 [US3] Implement deduplication: when two consecutive rounds have identical finding IDs, the second round's packet references the same finding IDs rather than duplicating the list (findings are idempotent references) in `src/orchestrator/refinement.rs`

### Event Type Registration

- [X] T070 [US3] Add `RefinementRoundCompleted` to the `EventType` enum's `all()` array so it appears in observability summaries in `src/domain/observability.rs`
- [X] T071 [US3] Add `RefinementRoundCompleted` to any `EventType` match arms that need exhaustive coverage (type_name, schema_version, event category classification) in `src/domain/observability.rs`

### US3 Tests

- [X] T072 [US3] Add contract tests to `tests/contract/refinement_output_contract.rs` for round packet schema (all required fields present, no inline content, confidence invariant, confidence downgrade-only, stop reason vocabulary, round numbering, delta validity, null stop_reason semantics, schema version comparison for valid match, unknown major version rejection, malformed version rejection)
- [X] T073 [US3] Add integration test to `tests/integration/refinement_flow.rs` verifying that a 3-round loop produces exactly 3 trace events of type `RefinementRoundCompleted`, each with all required fields present and no inline artifact content

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, version bump, quality verification, and release readiness.

### Documentation

- [X] T074 [P] Update `README.md` with refinement feature description and quickstart reference if release-facing behavior changed
- [X] T075 [P] Update `CHANGELOG.md` with `[0.76.0]` entry describing recursive stage refinement profiles feature
- [X] T076 [P] Update `docs/runtime/refinement.md` with runtime reference documentation for the refinement feature
- [X] T077 [P] Update `roadmap/Next - forward-roadmap.md` marking the refinement feature as delivered or updating its status
- [X] T078 [P] Update `tech-docs/` markdown files if any are affected by the refinement feature

### Roadmap Conversion

- [X] T079 Preserve the roadmap seed: `specs/076-recursive-stage-refinement/spec-recursive-stage-refinement-profiles.md` is already in place; verify it remains as the preserved seed
- [X] T080 Remove the roadmap seed from `roadmap/` folder if the roadmap uses move-on-conversion semantics (inspect roadmap README or conversion policy first)
- [X] T081 Update any roadmap index files or forward-roadmap files that reference the old seed location

### Final Phase: Release, Quality, And Verification

- [X] T082 Update `Cargo.toml` version from `0.75.0` to `0.76.0` (feature bump: 0.x.y → 0.(x+1).0)
- [X] T083 Run `./scripts/update-docs-versions.sh` to synchronize version references across docs
- [X] T084 Run `cargo fmt` to ensure all code is formatted
- [X] T085 Run `scripts/clippy.sh` and fix all warnings (`cargo clippy --workspace --all-targets --all-features -- -D warnings`)
- [X] T086 Run `scripts/test.sh` and fix all failing tests (`cargo nextest run --workspace --all-features`)
- [X] T087 Run `scripts/coverage.sh` and confirm at least 95% coverage for every modified or created Rust file (`src/domain/refinement.rs`, `src/orchestrator/refinement.rs`, modified files in `src/cli/`, `src/registry/agent_registry.rs`)
- [X] T088 Verify that `cargo tree -p boundline --no-dev-dependencies` shows no `sqlite-vec` in the dependency graph (SC-006: feature must be functional without sqlite-vec)
- [X] T089 Run `scripts/check-no-local-paths.sh` to verify no absolute paths are committed
- [X] T090 Run `scripts/check-rust-no-panic.sh` to verify no panic-prone patterns (`unwrap`, `expect`, `panic!`, etc.) outside `main.rs`
- [X] T091 Run `scripts/sync-distribution-metadata.sh` to synchronize Homebrew formula and Winget manifests with the new version

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (module scaffolding)
- **User Story 1 (Phase 3)**: Depends on Phase 2 (domain types). This is the MVP.
- **User Story 2 (Phase 4)**: Depends on Phase 3 (needs refinement data in trace). Can partially overlap with US3.
- **User Story 3 (Phase 5)**: Depends on Phase 3 (needs refinement loop to produce packets). Can partially overlap with US2.
- **Polish (Phase 6)**: Depends on all user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Depends on Foundational (Phase 2). No other story dependencies.
- **US2 (P2)**: Depends on US1 (needs refinement loop to have executed). Independent of US3.
- **US3 (P3)**: Depends on US1 (needs refinement loop to produce packets). Independent of US2.

### Within Each User Story

- **US1**: Config loading → Provider resolution → Orchestrator → Trace integration → Plan command wiring
- **US2**: Status rendering → Next command → Inspect command (can parallelize: T054+T056 can run in parallel with T058)
- **US3**: Schema validation → Trace event registration → Deduplication → Projection (T063-T065 parallel with T066-T067)

### Parallel Opportunities

- All T004-T008 (enums) can be defined in parallel
- All T022-T031 (unit tests) can be written in parallel after their corresponding types are defined
- T073-T077 (documentation) can all run in parallel
- T054+T056 (status+next) can run in parallel with T058 (inspect)
- T063-T065 (schema validation) can run in parallel with T066-T070 (trace event registration)

---

## Implementation Strategy

### MVP First (Internal Smoke-Test Milestone)

**US1 alone is an internal smoke-test milestone, not a releasable feature.** The feature is not complete, inspectable, or trace-quality until US2 and US3 are also complete.

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational domain types (T004-T031)
3. Complete Phase 3: US1 — refinement loop (T032-T054)
4. **STOP and VALIDATE**: Run a refinement loop end-to-end with test fixtures as an internal smoke test
5. Continue to US2 and US3 before considering the feature deliverable

### Incremental Delivery

1. **Smoke Test**: US1 — Refinement loop executes end-to-end with trace events (internal validation only)
2. **+US2**: Operators can inspect refinement state through status/next/inspect (feature becomes operator-visible)
3. **+US3**: Round packets are compact, schema-versioned, and trace-linked (feature is complete)
4. **+Polish**: Documentation, version bump, quality gates pass (feature is releasable)

### Feature Completion Bar

All three user stories (US1, US2, US3) must be complete before the feature is considered ready for release. US1 alone provides internal validation but does not meet the inspectability or trace-quality requirements of the spec.

---

## Task Count Summary

| Phase | Story | Task Range | Count |
|-------|-------|-----------|-------|
| Phase 1: Setup | — | T001-T003 | 3 |
| Phase 2: Foundational | — | T004-T031 | 28 |
| Phase 3: User Story 1 | US1 | T032-T054 | 23 |
| Phase 4: User Story 2 | US2 | T055-T063 | 9 |
| Phase 5: User Story 3 | US3 | T064-T073 | 10 |
| Phase 6: Polish | — | T074-T081 | 8 |
| Phase 6: Final Verification | — | T082-T091 | 10 |
| **Total** | | **T001-T091** | **91** |
