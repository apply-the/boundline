# Tasks: Adaptive Governance Calibration

**Feature**: 075-adaptive-governance-calibration
**Branch**: `075-adaptive-governance-calibration`
**Plan**: [plan.md](./plan.md) | **Spec**: [spec.md](./spec.md)
**Generated**: 2026-06-06

## Phase 1: Setup & Project Initialization

- [x] T001 Reconfirm the provider-catalog no-change audit from `specs/075-adaptive-governance-calibration/research.md` against `assistant/catalog/model-catalog.toml`
- [x] T002 [P] Copy roadmap seed into feature spec folder if not already present: verify `specs/075-adaptive-governance-calibration/feat-adaptive-governance-calibration.md` exists
- [x] T003 [P] Update `roadmap/Next - forward-roadmap.md` line 39 to reference `../specs/075-adaptive-governance-calibration/spec.md` instead of `features/11-adaptive-governance-calibration.md`

---

## Phase 2: Foundational – Domain Types & Contracts

**Goal**: All domain types, enums, and validation logic that underpin US1, US2, and US3.

- [x] T004 [P] Define `ControlLevel` enum (`Advisory`, `Catch`, `Rule`, `Hook`), `AuthorityZone` enum (`Green`, `Yellow`, `Red`), and `RiskLevel` enum (`Low`, `Medium`, `High`) in `src/domain/calibration.rs`
- [x] T005 [P] Define `OverridePolicy` struct with `allowed_roles`, `required_evidence`, `time_limited`, `max_duration_hours` in `src/domain/calibration.rs`
- [x] T006 [P] Define `ControlLevelEntry` struct with `rule_id`, `authority_source`, `default_level`, `green_level`, `yellow_level`, `red_level`, `confidence_threshold`, `override_policy` in `src/domain/calibration.rs`
- [x] T007 Define `CalibrationPolicy` struct with `schema_version`, `evidence_window`, `minimum_evidence_threshold`, `entries: Vec<ControlLevelEntry>` plus TOML deserialization (`serde::Deserialize`) in `src/domain/calibration.rs`
- [x] T008 Define `GuardianTrustRecord` struct with true/false positive counts, deferred count, override count, violation count, incident correlation, eval pass rate, plus `true_positive_rate()` method (returns `None` when sample below threshold) in `src/domain/calibration.rs`
- [x] T009 Define `ControlLevelAssignment` struct with `rule_id`, `assigned_level`, `guardian_confidence`, `calibrated_confidence`, `guidance_strength`, `authority_source`, `authority_zone`, `risk_level`, `reason`, `degraded_from`, `degradation_reason` in `src/domain/calibration.rs`
- [x] T010 [P] Define `OverrideRecord` struct with `finding_id`, `control_id`, `guardian_id`, `requested_level`, `reason`, `operator_identity`, `timestamp`, `expiry`, `satisfies_policy` in `src/domain/calibration.rs`
- [x] T011 [P] Define `DegradationEvent` and `EscalationEvent` structs with trigger enums (`DegradationTrigger`, `EscalationTrigger`) in `src/domain/calibration.rs`
- [x] T012 Implement `CalibrationPolicy::validate()` with contradiction detection (same `rule_id` + zone + risk → fail closed, stricter level), red-zone advisory guard, missing `rule_id` reference handling, and confidence range check in `src/domain/calibration.rs`
- [x] T013 Implement `CalibrationPolicy::resolve_level()` that takes `rule_id`, `guidance_strength`, `authority_source`, `authority_zone`, `risk_level`, `lifecycle_phase`, and optional `GuardianTrustRecord`, returns `ControlLevel` (with no-trust-data fallback to `default_level`) in `src/domain/calibration.rs`
- [x] T014 Implement `CalibrationPolicy::load_from_workspace()` that reads `.boundline/calibration-policy.toml`, deserializes, validates, and returns policy or falls back to a built-in all-advisory default in `src/domain/calibration.rs`
- [x] T015 Wire `calibration` module into `crates/boundline-core/src/domain.rs` with `#[path = "../../../src/domain/calibration.rs"]`
- [x] T016 [P] Add domain unit tests for `ControlLevel`, `AuthorityZone`, `RiskLevel` enum serialization roundtrips and `OverridePolicy` deserialization in `tests/unit/calibration_model.rs`
- [x] T017 [P] Add domain unit tests for `CalibrationPolicy` validation (valid policy, contradictory entries → fail closed, red-zone advisory → error, missing rule_id → warning, invalid confidence range → error) in `tests/unit/calibration_model.rs`
- [x] T018 [P] Add domain unit tests for `GuardianTrustRecord::true_positive_rate()` (sufficient sample, insufficient sample, zero denominator) and `CalibrationPolicy::resolve_level()` (trust-based promotion, demotion, cold start) in `tests/unit/calibration_model.rs`

---

## Phase 3: User Story 1 – Inspect a Guardian's Control Level Decision (P1)

**Goal**: `boundline inspect` surfaces control level, guardian confidence, calibrated confidence, override policy state, degradation state, and terminal outcome per guardian.

**Independent Test**: Run `boundline inspect` on a workspace with guardian findings and verify the output includes control-level summary for each guardian.

- [x] T019 [US1] Add `ControlLevelAssignment`, `OverrideRecord`, `DegradationEvent`, and `EscalationEvent` rendering helpers to `src/cli/inspect.rs` — produce human-readable and JSON output showing: level, guardian confidence, calibrated confidence, override record references, degradation indicators, escalation triggers
- [x] T020 [US1] Wire calibration policy loading into inspect dispatch: read `.boundline/calibration-policy.toml`, resolve the current level for each activated guardian, surface in `boundline inspect` output in `src/cli/inspect.rs`
- [x] T021 [US1] Ensure `boundline inspect --json` includes `control_levels` array with full assignment details per guardian in `src/cli/inspect.rs`
- [x] T022 [P] [US1] Add inspect output contract tests verifying human-readable format includes control level, confidence scores, override state, degradation state, and terminal outcome in `tests/contract/calibration_output_contract.rs`
- [x] T023 [P] [US1] Add inspect output contract tests for JSON format: `control_levels` array present, each entry has `rule_id`, `assigned_level`, `guardian_confidence`, `calibrated_confidence`, `guidance_strength`, `authority_source`, `authority_zone`, `risk_level`, `reason` in `tests/contract/calibration_output_contract.rs`
- [x] T024 [US1] Add integration test: create workspace with calibration policy, run `boundline run`, then `boundline inspect`, assert control-level summary matches expected levels in `tests/integration/calibration_flow.rs`
- [x] T025 [US1] Add trace visibility test: verify `control_level.assigned` event emitted after council adjudication and appears in `boundline inspect --json` trace output in `tests/integration/calibration_flow.rs`

---

## Phase 4: User Story 2 – Council Adjudication Applies Graduated Control Levels (P2)

**Goal**: `boundline council adjudicate` reads `.boundline/calibration-policy.toml`, applies the correct control level per guardian based on authority zone and risk level, and blocks/allows according to level semantics (advisory=visible, catch=bypassable, rule=block+override, hook=unconditional-block).

**Independent Test**: Create workspace with calibration policy mapping zones/risks to levels, run `boundline run`, assert council decision applies correct level per guardian.

- [x] T026 [US2] Integrate calibration policy loading into `src/cli/council.rs` adjudication path: after guardian activation, load policy, resolve control level for each activated guardian, and attach `ControlLevelAssignment` to each finding in the council decision
- [x] T027 [US2] Implement control-level enforcement in council adjudication: advisory findings → visible in decision but outcome stays `clean`; catch findings → visible with bypass hint, outcome stays `clean`; rule findings → `blocked` unless overridden; hook findings → `blocked` unconditionally in `src/cli/council.rs`
- [x] T028 [US2] Implement override record consumption in `src/cli/council.rs`: before adjudication, read `.boundline/overrides.toml`, match records to findings, accept if `satisfies_policy = true` and not expired, remove consumed records
- [x] T029 [US2] Implement the `boundline override` CLI command with `--workspace`, `--guardian-id`, `--control-id`, `--level`, `--reason`, `--expiry` flags; writes `OverrideRecord` to `.boundline/overrides.toml` in `src/cli/override.rs`
- [x] T030 [US2] Register `boundline override` as a `DeveloperCommand` variant (`DeveloperCommand::Override { .. }`) and wire dispatch in `src/cli.rs` following the existing council command pattern
- [x] T031 [P] [US2] Add override command contract tests: valid override written, invalid level rejected, hook bypass rejected, missing required fields rejected in `tests/contract/calibration_output_contract.rs`
- [x] T032 [US2] Add council adjudication contract tests: advisory finding → clean outcome, catch finding → clean with bypass hint, rule finding → blocked without override, rule finding → clean with valid override, hook finding → blocked even with override, contradictory policy → fail closed in `tests/contract/calibration_output_contract.rs`
- [x] T033 [US2] Add integration test: full flow with calibration policy → `boundline run` → council blocks on rule-level finding → operator writes `boundline override` → `boundline continue` succeeds in `tests/integration/calibration_flow.rs`
- [x] T034 [US2] Add integration test: calibration policy file missing → council defaults all guardians to advisory, `boundline run` proceeds without blocks in `tests/integration/calibration_flow.rs`

---

## Phase 5: User Story 3 – Control Level Graduates Based on Trust and Evidence (P3)

**Goal**: Trust metrics accumulate after every adjudication; after configurable evidence window, guardians with high TPR promote, guardians with high FPR demote or stay at advisory. Evals failing or incident correlation blocks promotion.

**Independent Test**: Simulate guardian historical performance data, run calibration evaluation, assert correct promotion/demotion.

[x] - [x] T035
- [x] [US3] Implement trust metric accumulation in `src/domain/calibration.rs`: after each council adjudication, update `GuardianTrustRecord` true/false positive counts (upheld → TP, rejected → FP, deferred → excluded) and persist in trace store
[x] - [x] T036
- [x] [US3] Implement calibration evaluation in `src/domain/calibration.rs`: after the evidence window (default 5 adjudicated sessions), compute TPR, check against `confidence_threshold`, evaluate promotion (advisory→catch, catch→rule), demotion (rule→catch, catch→advisory), and incident lock
[x] - [x] T037
- [x] [US3] Implement `control_level.graduated` structured event emission with before/after level, trigger (trust_promotion/trust_demotion/incident_lock/insufficient_evidence), and confidence metrics in `src/orchestrator/session_runtime_observability.rs`
[x] - [x] T038
- [x] [US3] Prevent promotion when eval pass rate is below confidence threshold or evidence is insufficient; emit `control_level.assigned` with reason `insufficient_evidence` in `src/domain/calibration.rs`
[x] - [x] T039
- [x] [P] [US3] Add trust metric unit tests: TP/FP counting, deferred exclusion, TPR computation with sufficient/insufficient sample, incident lock prevents promotion in `tests/unit/calibration_model.rs`
- [x] T0*40 [P] [US3] Add calibration evaluation unit tests: promotion after window with high TPR, demotion after window with high FPR, no change with insufficient evidence, eval failure blocks promotion in `tests/unit/calibration_model.rs`
- [x] T0*41 [US3] Add integration test: simulate 5 sessions with 100% TPR → guardian promotes from advisory to catch in `tests/integration/calibration_flow.rs`
- [x] T0*42 [US3] Add integration test: simulate 5 sessions with high FPR → guardian demotes or stays advisory; simulate incident correlation → guardian locked at advisory in `tests/integration/calibration_flow.rs`

---

## Phase 6: Degradation & Escalation

**Goal**: Controls degrade when providers/tools are unavailable; findings escalate on repeated unresolved, red zone, low-confidence/high-impact, missing evidence, or boundary risk.

- [x] T0*43 Implement degradation rule evaluation in `src/domain/calibration.rs`: when provider/model/tool unavailable → downgrade to advisory if safe, require human gate if unsafe, block if mandatory evidence cannot be produced
- [x] T0*44 Implement `control.degraded` structured event emission with original level, degraded level, trigger, safety flag, and human-gate flag in `src/orchestrator/session_runtime_observability.rs`
- [x] T0*45 Implement escalation trigger evaluation in `src/domain/calibration.rs`: check repeated unresolved findings, red zone, low-confidence/high-impact, missing evidence, boundary risk; emit `control.escalated` event
- [x] T0*46 [P] Add degradation unit tests: provider unavailable safe → advisory, provider unavailable unsafe → human gate required, hook never silently downgrades in `tests/unit/calibration_model.rs`
- [x] T0*47 [P] Add escalation unit tests: repeated unresolved triggers escalation, red zone triggers escalation, low-confidence/high-impact triggers escalation in `tests/unit/calibration_model.rs`
- [x] T0*48 Add integration test: simulate provider unavailability mid-session → control degrades with trace event in `tests/integration/calibration_flow.rs`
- [x] T0*49 Add integration test: repeated unresolved findings across sessions → escalation event emitted, inspect surfaces escalation in `tests/integration/calibration_flow.rs`

---

## Final Phase: Release, Quality, And Verification

- [x] T0*50 Bump workspace version from `0.74.0` to `0.75.0` (feature) in `Cargo.toml` and propagate to `Cargo.lock`
- [x] T0*51 [P] Update `CHANGELOG.md` with the `0.75.0` release entry for adaptive governance calibration
- [x] T0*52 [P] Update `README.md` with `boundline override` and calibration policy usage
- [x] T0*53 [P] Update `tech-docs/architecture.md` with calibration and guardian trust module description
- [x] T0*54 [P] Add runtime doc in `docs/runtime/calibration.md`
- [x] T0*55 [P] Update `roadmap/Next - forward-roadmap.md` to mark feature 11 as delivered in `0.75.0`
- [x] T0*56 [P] Update `roadmap/joint-roadmap-graph.md` if B11 should now be marked as shipped (remove from graph)
- [x] T0*57 [P] Update `assistant/plugin-metadata.json` and `assistant/global/manifest.json` to `0.75.0`
- [x] T0*58 [P] Update release metadata to `0.75.0` in `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.75.0/`
- [x] T0*59 Run `scripts/update-docs-versions.sh` to synchronize version references
- [x] T0*60 Run `cargo fmt` and verify with `cargo fmt --check`
- [x] T0*61 Run `scripts/clippy.sh` and fix all warnings
- [x] T0*62 Run `scripts/test.sh` and fix failing tests
- [x] T0*63 Run `scripts/coverage.sh` and confirm ≥95% coverage for every modified or created Rust file
- [x] T0*64 Run `scripts/check-no-local-paths.sh`
- [x] T0*65 Run `scripts/check-rust-no-panic.sh`

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001-T003 before domain types
- **Foundational → US1**: T004-T018 must complete before inspect rendering
- **US1 → US2**: US2 depends on calibration policy types and inspect visibility from US1
- **US2 → US3**: US3 depends on council adjudication producing trust metrics from US2
- **US3 → Degradation/Escalation**: Degradation/escalation builds on control level assignment and trust tracking
- **All → Release**: Implementation must stabilize before Phase 6 release tasks

### Parallel Opportunities

- T002, T003 can run in parallel (independent file edits)
- T004-T011 (domain type definitions) can run in parallel
- T016-T018 (domain unit tests) can run in parallel
- T022-T023 (inspect contract tests) can run in parallel
- T031-T032 (override + council contract tests) can run in parallel
- T039-T040 (trust unit tests) can run in parallel
- T046-T047 (degradation/escalation unit tests) can run in parallel
- T051-T058 (all docs/release tasks) can run in parallel

---

## Implementation Strategy

### MVP First (US1 only)

1. Setup + Foundational → domain types and calibration policy file
2. US1 → `boundline inspect` shows control-level decisions
3. Stop and validate: operator can understand why a guardian blocked/allowed
4. Final Phase → version bump, docs, quality scripts

### Incremental Delivery

1. US1 → inspectability (P1)
2. US2 → graduated enforcement (P2)
3. US3 → trust evolution (P3)
4. Degradation & Escalation → robustness

---

## Boundline Wrapper Compliance

| Rule | Task IDs | Status |
|------|----------|--------|
| Rule 1: Version bump | T050 | 0.74.0 → 0.75.0 (feature) |
| Rule 2: Docs/roadmap sync | T002-T003, T051-T056 | Roadmap seed, CHANGELOG, README, tech-docs, docs, roadmap |
| Rule 3: docs-versions sync | T059 | `scripts/update-docs-versions.sh` |
| Rule 4: Quality scripts | T060-T065 | fmt, clippy, test, coverage, no-local-paths, no-panic |
