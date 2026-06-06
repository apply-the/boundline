# Tasks: Review Councils And Role-Gated Governance

**Input**: Design documents from `/specs/074-review-councils-governance/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/council-output-contract.md`, `quickstart.md`

**Boundline wrapper**: Version 0.73.0 → 0.74.0 (feature), quality scripts,
docs sync, coverage ≥95%.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Determine the next release version (currently 0.73.0 → 0.74.0, feature) per boundline versioning policy in `Cargo.toml`
- [x] T002 [P] Reconfirm the provider-catalog no-change audit from `specs/074-review-councils-governance/research.md` against `assistant/catalog/model-catalog.toml`

---

## Phase 2: Foundational

**Critical**: Complete before user-story implementation.

- [x] T003 [P] Define `GuardianRule` struct, `GuardianRuleset` struct, `RulesetSource` enum, and TOML deserialization in `src/domain/council.rs`
- [x] T004 [P] Define `GuardianActivationPlan` struct, `GuardianSkipRecord` struct, `GuardianExecutionRecord` struct, and `ExecutionStatus` enum in `src/domain/council.rs`
- [x] T005 [P] Define `CouncilDecision` struct, `CouncilOutcome` enum, `ProfileSource` enum in `src/domain/council.rs`
- [x] T006 [P] Add `GuardianActivationPlanProduced` and `CouncilDecisionProduced` variants to `EventType` enum, `type_name`, and `schema_version` mappings in `src/domain/observability.rs`
- [x] T007 Create the versioned `.boundline/guardian-rules.toml` with four built-in rules (Rust runtime, docs-only, contract, security-sensitive) and the `[metadata]` schema_version header
- [x] T008 Wire `council` module into `crates/boundline-core/src/domain.rs`
- [x] T009 [P] Add focused failing domain regressions for ruleset deserialization, contradiction detection, and activation plan production in `tests/unit/council_model.rs`

**Checkpoint**: Domain types compile, ruleset loads from TOML, contradictions fail closed.

---

## Phase 3: User Story 1 - Guardian Activation Router (Priority: P1) 🎯 MVP

**Goal**: Router evaluates change surface against TOML rules and produces an activation plan.

- [x] T010 [US1] Implement the TOML ruleset loader with typed deserialization, contradiction detection (same guardian both activate and skip under same condition), and fail-closed validation in `src/domain/council.rs`
- [x] T011 [US1] Implement the built-in default ruleset fallback when `.boundline/guardian-rules.toml` is missing in `src/domain/council.rs`
- [x] T012 [US1] Implement the activation router: iterate matched rules, collect activated/skipped guardians, check mandatory availability, produce `GuardianActivationPlan` in `src/domain/council.rs`
- [x] T013 [US1] Implement the `guardian.activation.plan.produced` structured event emission in `src/orchestrator/session_runtime_observability.rs`
- [x] T014 [US1] Run and expand router domain regressions in `tests/unit/council_model.rs`
- [x] T015 [P] [US1] Add contract tests for ruleset validation (valid TOML, missing file → built-in fallback, contradictory rules → fail closed) in `tests/contract/council_ruleset_contract.rs`
- [x] T016 [US1] Add router integration tests in `tests/integration/council_router_flow.rs`

**Checkpoint**: Router activates correct guardians for all four built-in rule types, fails closed on invalid ruleset.

---

## Phase 4: User Story 2 - Council Adjudication (Priority: P2)

**Goal**: `boundline council adjudicate` CLI examines guardian findings and produces clean/blocked decision.

- [x] T017 [US2] Implement the single-adjudicator decision model: read guardian execution records, classify findings (accepted/rejected/deferred), apply mandatory guardian evidence rules, produce binary outcome in `src/domain/council.rs`
- [x] T018 [US2] Implement the `boundline council adjudicate` CLI command with `--json` flag in `src/cli/council.rs`
- [x] T019 [US2] Implement human-readable council output rendering in `src/cli/council.rs`
- [x] T020 [US2] Implement JSON council output rendering in `src/cli/council.rs`
- [x] T021 [US2] Implement the `council.decision.produced` structured event emission in `src/orchestrator/session_runtime_observability.rs`
- [x] T022 [US2] Run and expand council domain regressions in `tests/unit/council_model.rs`
- [x] T023 [P] [US2] Add council output contract tests for human-readable and JSON formats, all outcomes (clean/blocked), missing mandatory guardian, and default adjudicator in `tests/contract/council_output_contract.rs`
- [x] T024 [US2] Add council integration tests in `tests/integration/council_adjudication_flow.rs`
- [x] T024a [P] [US2] Add council adjudication performance contract test verifying the decision completes within 1 second for up to 100 findings in `tests/contract/council_output_contract.rs`

**Checkpoint**: Council produces trace-visible clean/blocked decisions with full finding context.

---

## Final Phase: Release, Quality, And Verification

- [x] T025 Bump workspace version from `0.73.0` to `0.74.0` (feature) in `Cargo.toml` and propagate to `Cargo.lock`
- [x] T026 [P] Update release metadata to `0.74.0` in `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, and `assistant/plugin-metadata.json`
- [x] T027 [P] Add WinGet release manifests for `0.74.0` under `distribution/winget/manifests/a/ApplyThe/Boundline/0.74.0/`
- [x] T028 [P] Update `assistant/global/manifest.json` and plugin manifests to `0.74.0`
- [x] T029 [P] Update `CHANGELOG.md` with the `0.74.0` release entry for review councils and guardian activation router
- [x] T030 [P] Update `README.md` with `boundline council adjudicate` usage
- [x] T031 [P] Update `tech-docs/architecture.md` with council and guardian router module description
- [x] T032 [P] Add runtime doc in `docs/runtime/council.md`
- [x] T033 [P] Update `roadmap/Next - forward-roadmap.md` to mark feature 10 as spec'd in 074
- [x] T034 [P] Update `roadmap/joint-roadmap-graph.md` to remove B10 (now being spec'd)
- [x] T035 Run `scripts/update-docs-versions.sh` to synchronize version references
- [x] T036 Run `cargo fmt` and verify with `cargo fmt --check`
- [x] T037 Run `scripts/clippy.sh` and fix all warnings
- [x] T038 Run `scripts/test.sh` and fix failing tests
- [x] T039 Run `scripts/coverage.sh` and confirm ≥95% coverage for every modified or created Rust file
- [x] T040 Run `scripts/check-no-local-paths.sh`
- [x] T041 Run `scripts/check-rust-no-panic.sh`

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001-T002 before domain types.
- **Foundational → US1**: T003-T009 before router logic.
- **US1 → US2**: Council depends on activation plan and guardian execution records from US1.
- **US1+US2 → Release**: All implementation must stabilize before Phase 5.

### Parallel Opportunities

- T003, T004, T005, T006 can run in parallel
- T015 and T023 can run in parallel
- T026-T034 (all docs/release tasks) can run in parallel

---

## Implementation Strategy

### MVP First

1. Setup + Foundational → domain types and ruleset file
2. US1 → router activates correct guardians, fails closed on invalid rules
3. US2 → council adjudicates findings, produces clean/blocked decision
4. Final Phase → version bump, docs, quality scripts

## Boundline Wrapper Compliance

| Rule | Task IDs | Status |
|------|----------|--------|
| Rule 1: Version bump | T025 | 0.73.0 → 0.74.0 (feature) |
| Rule 2: Docs/roadmap sync | T029-T034 | CHANGELOG, README, tech-docs, docs, roadmap |
| Rule 3: docs-versions sync | T035 | `scripts/update-docs-versions.sh` |
| Rule 4: Quality scripts | T036-T041 | fmt, clippy, test, coverage, no-local-paths, no-panic |
