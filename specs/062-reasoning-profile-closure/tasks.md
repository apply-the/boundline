# Tasks: Reasoning Profile Closure

**Input**: Design documents from `/specs/062-reasoning-profile-closure/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This feature changes runtime reasoning, operator-visible projections,
release claims, and compatibility alignment. Focused unit, integration, and
contract coverage are required during implementation, followed by full-workspace
formatting, clippy, nextest, and refreshed `lcov.info` closeout.

**Organization**: Tasks are grouped by user story so each closure slice remains
independently testable after the shared release-alignment foundation is in
place.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Include exact file paths in descriptions

## Path Conventions

- Boundline repo root paths are repo-relative (`src/`, `tests/`, `docs/`)
- Canon sibling repo paths are absolute under `/Users/rt/workspace/apply-the/canon/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Lock the release pair, seed validation artifacts, and capture the
required catalog audit before runtime work expands.

- [ ] T001 Bump Boundline to `0.62.0` and seed the release-alignment expectations in `Cargo.toml`, `CHANGELOG.md`, `tests/contract/canon_reasoning_posture_contract.rs`, and `specs/062-reasoning-profile-closure/contracts/release-alignment-contract.md`; bump `/Users/rt/workspace/apply-the/canon/Cargo.toml` to `0.59.0` and align its version assertions in `/Users/rt/workspace/apply-the/canon/tests/contract/governed_reasoning_posture_contract.rs`
- [ ] T002 [P] Create `specs/062-reasoning-profile-closure/validation-report.md` and record the provider-doc catalog audit result referencing `assistant/catalog/model-catalog.toml` plus the public OpenAI, Anthropic, and Google model docs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extend the shared closure vocabulary, fixtures, and compatibility
artifacts required by every user story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T003 Extend closure classification, operator-claim, and release-alignment helpers in `src/domain/reasoning.rs`, `src/domain/governance.rs`, and `src/domain/session.rs`
- [ ] T004 [P] Refresh deterministic residual-profile fixtures and helper builders in `src/fixture.rs`, `tests/integration/reasoning_profile_activation.rs`, and `tests/integration/reasoning_profile_degradation.rs`
- [ ] T005 [P] Refresh Boundline-local Canon compatibility artifacts and published contract baselines in `tests/contract/canon_reasoning_posture_contract.rs`, `specs/061-reasoning-profile-contracts/contracts/canon-governed-reasoning-posture-contract.snapshot.md`, and `specs/061-reasoning-profile-contracts/contracts/canon-challenge-posture-consumer-contract.md`, and update `/Users/rt/workspace/apply-the/canon/docs/integration/governed-reasoning-posture-contract.md` in the same step

**Checkpoint**: The release-alignment substrate exists, fixtures are ready, and
compatibility validation can run with or without the sibling Canon repo.

---

## Phase 3: User Story 1 - Close Concrete Residual Profiles (Priority: P1) 🎯 MVP

**Goal**: Make `independent_pair_review`, `heterogeneous_security_review`, and
`bounded_reflexion` fully credible as shipped concrete profiles inside the
existing session-native workflow.

**Independent Test**: Representative scenarios reach positive-path and bounded
non-success outcomes with aligned `run`, `status`, `inspect`, and trace
evidence for each residual concrete profile.

### Tests for User Story 1

- [ ] T006 [P] [US1] Extend runtime contract coverage for `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion` in `tests/contract/reasoning_profile_contract.rs`
- [ ] T007 [P] [US1] Extend positive-path activation and operator-projection coverage in `tests/integration/reasoning_profile_activation.rs`
- [ ] T008 [P] [US1] Extend bounded interruption, exhaustion, and degraded-path coverage in `tests/integration/reasoning_profile_degradation.rs`

### Implementation for User Story 1

- [ ] T009 [P] [US1] Add closure-specific profile definitions, classification helpers, and confidence semantics in `src/domain/reasoning.rs` and `src/domain/governance.rs`
- [ ] T010 [US1] Implement missing positive-path activation, participant handling, and terminal outcome logic in `src/orchestrator/session_runtime.rs`
- [ ] T011 [US1] Persist aligned reasoning outcome, next-action, and confidence summaries in `src/domain/session.rs` and `src/orchestrator/review_trace.rs`
- [ ] T012 [US1] Surface residual concrete profile results consistently through `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/session.rs`

**Checkpoint**: User Story 1 is independently functional and proves the
remaining concrete shipped profiles end-to-end.

---

## Phase 4: User Story 2 - Make Debate And Adjudication Claims Honest (Priority: P2)

**Goal**: Make every runtime, trace, contract, and documentation surface agree
that debate is bounded substrate and adjudication is a shared primitive.

**Independent Test**: The repository no longer exposes standalone shipped-profile
claims for debate or adjudication when operators inspect runtime or release
artifacts.

### Tests for User Story 2

- [ ] T013 [P] [US2] Extend classification and trace contract coverage for debate-as-substrate and adjudication-as-primitive claims in `tests/contract/reasoning_profile_contract.rs` and `tests/contract/reasoning_profile_trace_contract.rs`
- [ ] T014 [P] [US2] Add projection-level unit coverage for honest classification wording in `tests/unit/reasoning_profile_trace.rs` and `tests/unit/workflow_session_projection.rs`
- [ ] T015 [P] [US2] Extend integration coverage to ensure operator-visible reasoning summaries never invent standalone debate or adjudication profiles in `tests/integration/reasoning_profile_activation.rs` and `tests/integration/reasoning_profile_degradation.rs`

### Implementation for User Story 2

- [ ] T016 [P] [US2] Update reasoning trace vocabulary and classification helpers in `src/domain/trace.rs`, `src/domain/reasoning.rs`, and `src/orchestrator/review_trace.rs`
- [ ] T017 [US2] Align runtime and projection wording so debate surfaces only as bounded substrate and adjudication only as a shared primitive in `src/domain/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [ ] T018 [US2] Keep the closure contracts and validation report aligned with the final classification in `specs/062-reasoning-profile-closure/contracts/profile-closure-classification-contract.md`, `specs/062-reasoning-profile-closure/contracts/release-alignment-contract.md`, and `specs/062-reasoning-profile-closure/validation-report.md`

**Checkpoint**: User Stories 1 and 2 are independently testable, and the
repository no longer overstates debate or adjudication.

---

## Phase 5: User Story 3 - Ship A Release-Ready Closure Slice (Priority: P3)

**Goal**: Finish the release-facing alignment, pass the maintainability gate,
and keep Boundline↔Canon compatibility validation honest.

**Independent Test**: The release pair, fallback compatibility checks,
maintainability refactors, and release-facing artifacts all pass without manual
exceptions.

### Tests for User Story 3

- [ ] T019 [P] [US3] Extend Boundline compatibility coverage for the release pair, sibling-Canon alignment, and local-snapshot fallback in `tests/contract/canon_reasoning_posture_contract.rs`
- [ ] T020 [P] [US3] Add behavior-preserving coverage for refactored governance validation in `tests/unit/workflow_session_projection.rs` and `tests/unit/session_model.rs`
- [ ] T021 [P] [US3] Add behavior-preserving coverage for refactored reasoning-independence assessment in `tests/unit/reasoning_profile_independence.rs`, `tests/integration/reasoning_profile_degradation.rs`, and `/Users/rt/workspace/apply-the/canon/tests/contract/governed_reasoning_posture_contract.rs`

### Implementation for User Story 3

- [ ] T022 [P] [US3] Refactor `SessionStatusView::validate_governance` into smaller helper validators in `src/domain/session.rs`
- [ ] T023 [P] [US3] Refactor `assess_reasoning_independence` into smaller helper evaluators in `src/orchestrator/session_runtime.rs`
- [ ] T024 [US3] Align release-pair compatibility windows, fallback snapshot usage, and version assertions in `Cargo.toml`, `tests/contract/canon_reasoning_posture_contract.rs`, `specs/061-reasoning-profile-contracts/contracts/canon-governed-reasoning-posture-contract.snapshot.md`, `/Users/rt/workspace/apply-the/canon/Cargo.toml`, `/Users/rt/workspace/apply-the/canon/docs/integration/governed-reasoning-posture-contract.md`, and `/Users/rt/workspace/apply-the/canon/tests/contract/governed_reasoning_posture_contract.rs`
- [ ] T025 [US3] Update release-facing change records in `CHANGELOG.md`, `specs/062-reasoning-profile-closure/validation-report.md`, `/Users/rt/workspace/apply-the/canon/CHANGELOG.md`, and `/Users/rt/workspace/apply-the/canon/ROADMAP.md`

**Checkpoint**: All user stories are independently functional and the closure
slice is release-ready from a behavior, compatibility, and maintainability
standpoint.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish the repo-wide documentation and validation closeout.

- [ ] T026 [P] Refresh release-facing docs and roadmap language in `README.md`, `ROADMAP.md`, `docs/adaptive-execution.md`, `docs/runtime-confidence-and-calibration.md`, `docs/architecture.md`, `docs/reasoning-profile-algorithms.md`, retire the completed `roadmap/S6*` drafts, and update `/Users/rt/workspace/apply-the/canon/README.md`, `/Users/rt/workspace/apply-the/canon/ROADMAP.md`, and `/Users/rt/workspace/apply-the/canon/CHANGELOG.md`
- [ ] T027 Run focused story validations, then run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov clean --workspace`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` in Boundline; run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo nextest run --workspace --all-features` in `/Users/rt/workspace/apply-the/canon/`; then record those results plus the `.github/workflows/quality.yml` SonarCloud quality-gate result in `specs/062-reasoning-profile-closure/validation-report.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; starts immediately and anchors the release pair.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all user stories.
- **User Story 1 (Phase 3)**: Starts after Foundational; delivers the MVP closure slice.
- **User Story 2 (Phase 4)**: Depends on the core runtime evidence from User Story 1.
- **User Story 3 (Phase 5)**: Depends on the shared closure vocabulary and can overlap late User Story 2 work where files do not conflict.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1**: No dependency on other user stories.
- **US2**: Depends on US1 activation, projection, and trace behavior existing.
- **US3**: Depends on foundational release-alignment artifacts; its refactors and compatibility updates should land after US1 and US2 settle the final shipped claim set.

### Within Each User Story

- Tests should fail before the corresponding implementation tasks.
- Domain helpers come before orchestrator integration.
- Runtime persistence and trace alignment come before CLI summary changes.
- Version-window and changelog alignment come before full validation closeout.

## Parallel Opportunities

- `T002`, `T004`, and `T005` can run in parallel during the foundational phase.
- All story test tasks marked `[P]` can run in parallel before implementation.
- `T009` and `T016` can run in parallel once foundational vocabulary is stable.
- `T022` and `T023` can run in parallel because they target different core files.

## Parallel Example: User Story 1

```bash
# Run the core US1 tests together
Task: "Extend runtime contract coverage in tests/contract/reasoning_profile_contract.rs"
Task: "Extend positive-path activation coverage in tests/integration/reasoning_profile_activation.rs"
Task: "Extend bounded non-success coverage in tests/integration/reasoning_profile_degradation.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate US1 independently before expanding claim-honesty and release work.

### Incremental Delivery

1. Deliver the residual concrete profile closure slice.
2. Align debate and adjudication claims across runtime and contracts.
3. Finish the release-quality, maintainability, and compatibility closeout.

## Notes

- The first task intentionally anchors the release version work requested by the user.
- The catalog audit task is explicit because every Boundline feature must record the provider-doc refresh result even when there is no bundled catalog delta.
- Canon companion publication is required for this closure because the released Boundline pair changes the published compatibility window.