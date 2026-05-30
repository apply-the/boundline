# Tasks: Governed Reasoning Profile Contracts

**Input**: Design documents from `/specs/061-reasoning-profile-contracts/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This feature changes runtime governance, review, trace, and cross-repo compatibility surfaces. Unit, contract, and integration coverage are required, plus final clippy and coverage closeout.

**Organization**: Tasks are grouped by user story so each story remains independently testable once the shared reasoning-profile substrate exists.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Include exact file paths in descriptions

## Path Conventions

- Boundline repo root paths are repo-relative (`src/`, `tests/`, `docs/`)
- Canon sibling repo paths are absolute under `../canon/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Lock the release pair and baseline compatibility targets before implementation expands the runtime surface.

- [x] T001 Bump Boundline to `0.61.0`, Canon to `0.57.0`, and seed version-alignment expectations in `Cargo.toml`, `../canon/Cargo.toml`, `tests/contract/canon_reasoning_posture_contract.rs`, and `../canon/tests/contract/governed_reasoning_posture_contract.rs`
- [x] T002 [P] Create the feature validation closeout stub in `specs/061-reasoning-profile-contracts/validation-report.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared models, contracts, fixtures, and compatibility scaffolding required by all user stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Publish the Canon provider-side posture contract in `../canon/docs/integration/governed-reasoning-posture-contract.md`
- [x] T004 Create typed reasoning-profile domain models in `src/domain/reasoning.rs`
- [x] T005 [P] Extend routing and governance types for profile activation and confidence handoff in `src/domain/configuration.rs` and `src/domain/governance.rs`
- [x] T006 [P] Extend session and trace projection models for reasoning-profile state in `src/domain/session.rs` and `src/domain/trace.rs`
- [x] T007 Create deterministic posture and profile fixture support in `src/fixture.rs`
- [x] T008 [P] Create Canon-side contract coverage for the provider contract in `../canon/tests/contract/governed_reasoning_posture_contract.rs`

**Checkpoint**: The feature has a typed substrate, a published Canon posture contract, and deterministic fixtures for isolated testing.

---

## Phase 3: User Story 1 - Activate Bounded Challenge In Delivery Flow (Priority: P1) 🎯 MVP

**Goal**: Allow a governed stage to activate one bounded reasoning profile inside the existing session lifecycle.

**Independent Test**: Representative stages activate `independent_pair_review`, `bounded_self_consistency`, and `heterogeneous_security_review`, preserve the unchanged path when no profile is required, and degrade, block, or interrupt explicitly when the requested challenge cannot continue.

### Tests for User Story 1

- [x] T009 [P] [US1] Contract test for reasoning-profile runtime vocabulary across `bounded_self_consistency`, `independent_pair_review`, `heterogeneous_security_review`, and `bounded_reflexion` in `tests/contract/reasoning_profile_contract.rs`
- [x] T010 [P] [US1] Integration test for verification-stage activation and unchanged no-profile behavior in `tests/integration/reasoning_profile_activation.rs`
- [x] T011 [P] [US1] Integration test for insufficient-independence fail-closed and human-interruption behavior in `tests/integration/reasoning_profile_degradation.rs`

### Implementation for User Story 1

- [x] T012 [P] [US1] Implement budget validation and activation-state helpers for self-consistency, heterogeneous review, reflexion, and debate-enabled profiles in `src/domain/reasoning.rs`
- [x] T013 [US1] Implement profile selection and stage attachment in `src/orchestrator/session_runtime.rs`
- [x] T014 [US1] Integrate reasoning-profile activation, unchanged-path handling, and interruption handling with session runtime flow in `src/orchestrator/session_runtime.rs`
- [x] T015 [US1] Resolve participant assignments for blind, heterogeneous, critic, reviser, and arbiter roles from existing routing slots and reviewer roles in `src/domain/configuration.rs`
- [x] T016 [US1] Persist activation state, posture provenance, and outcome summaries in `src/domain/session.rs`

**Checkpoint**: User Story 1 is independently functional and can activate, block, degrade, or escalate bounded reasoning profiles without a second workflow.

---

## Phase 4: User Story 2 - Inspect Reasoning Evidence And Confidence Handoff (Priority: P2)

**Goal**: Surface reasoning-profile lifecycle, disagreement, and confidence contribution through the normal operator-facing projections.

**Independent Test**: `plan`, `status`, `next`, and `inspect` explain profile activation, participant topology, disagreement or convergence, debate or reflexion progress, confidence contribution, and next action for a representative run.

### Tests for User Story 2

- [x] T017 [P] [US2] Contract test for additive reasoning trace events including debate rounds and reflexion revisions in `tests/contract/reasoning_profile_trace_contract.rs`
- [x] T018 [P] [US2] Integration test for inspect and status reasoning-profile summaries in `tests/integration/reasoning_profile_activation.rs`
- [x] T019 [P] [US2] Unit test for bounded reflexion, debate stagnation, and confidence contribution in `tests/unit/reasoning_profile_trace.rs`

### Implementation for User Story 2

- [x] T020 [P] [US2] Add additive reasoning trace events and projection helpers in `src/domain/trace.rs`
- [x] T021 [US2] Record profile lifecycle, disagreement, debate, reflexion, adjudication, and confidence events in `src/orchestrator/session_runtime.rs` and `src/orchestrator/review_trace.rs`
- [x] T022 [US2] Surface reasoning-profile summaries and next actions in `src/cli/inspect.rs`
- [x] T023 [US2] Surface reasoning-profile summaries in `src/cli/output.rs` and `src/cli/session.rs` for `plan`, `status`, and `next`
- [x] T024 [US2] Merge reasoning confidence contribution into governance, admission, and session projections in `src/domain/governance.rs` and `src/domain/session.rs`

**Checkpoint**: User Stories 1 and 2 are independently testable, and operators can inspect why a profile ran and what it changed.

---

## Phase 5: User Story 3 - Keep Boundline And Canon Contract-Aligned (Priority: P3)

**Goal**: Fail closed on version drift or contract drift and keep the bilateral posture vocabulary aligned.

**Independent Test**: Contract tests reject unsupported Canon posture inputs or incompatible Boundline↔Canon version windows before runtime execution begins.

### Tests for User Story 3

- [x] T025 [P] [US3] Boundline contract test for provider contract line and version-window alignment in `tests/contract/canon_reasoning_posture_contract.rs`
- [x] T026 [P] [US3] Canon contract test for the published provider vocabulary and consumer window in `../canon/tests/contract/governed_reasoning_posture_contract.rs`
- [x] T027 [P] [US3] Integration test for contract-drift blocked outcomes in `tests/integration/reasoning_profile_degradation.rs`

### Implementation for User Story 3

- [x] T028 [P] [US3] Wire Canon challenge-posture compatibility parsing and fail-closed validation in `src/domain/reasoning.rs`
- [x] T029 [US3] Enforce version-window and contract-line checks during activation in `src/orchestrator/session_runtime.rs`
- [x] T030 [US3] Surface contract-mismatch guidance and remediation text in `src/cli/output.rs` and `src/cli/inspect.rs`

**Checkpoint**: All three user stories work independently, and the feature fails closed on bilateral contract drift.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release-facing closeout across both repositories.

- [x] T031 Update `README.md`, `ROADMAP.md`, `CHANGELOG.md`, `../canon/README.md`, `../canon/ROADMAP.md`, `../canon/CHANGELOG.md`, and `specs/061-reasoning-profile-contracts/validation-report.md`, then run and record final `cargo clippy --workspace --all-targets --all-features -- -D warnings` plus `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` in Boundline and the matching final `cargo clippy --workspace --all-targets --all-features -- -D warnings` plus focused coverage validation in Canon

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; starts immediately and anchors the release pair.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all user stories.
- **User Story 1 (Phase 3)**: Starts after Foundational; delivers the MVP runtime activation path.
- **User Story 2 (Phase 4)**: Depends on User Story 1 runtime state existing.
- **User Story 3 (Phase 5)**: Depends on Foundational contracts and can overlap late User Story 2 work where files do not conflict.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1**: No dependency on other user stories.
- **US2**: Depends on US1 activation and persisted state.
- **US3**: Depends on foundational contracts; its runtime mismatch guidance should land after US1 activation exists.

### Within Each User Story

- Tests should fail before the corresponding implementation tasks.
- Domain model updates come before orchestrator integration.
- Trace and projection changes come before CLI summary changes.
- Contract alignment checks come before release-facing documentation.

## Parallel Opportunities

- `T002`, `T005`, `T006`, and `T008` can run in parallel during the foundational phase.
- All story test tasks marked `[P]` can run in parallel before implementation.
- `T020` and `T024` can run in parallel once the core reasoning types exist.
- `T025` and `T026` can run in parallel because they target different repositories.

## Parallel Example: User Story 1

```bash
# Run the core US1 tests together
Task: "Contract test for reasoning-profile runtime vocabulary in tests/contract/reasoning_profile_contract.rs"
Task: "Integration test for verification-stage profile activation in tests/integration/reasoning_profile_activation.rs"
Task: "Integration test for insufficient-independence fail-closed behavior in tests/integration/reasoning_profile_degradation.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate US1 independently before expanding the inspection and alignment story.

### Incremental Delivery

1. Deliver the activation substrate and bounded profile execution.
2. Add operator-facing inspect and confidence surfaces.
3. Close the bilateral Canon alignment and release-facing validation.

## Notes

- The first task is intentionally the version bump and version-test anchor requested by the user.
- The final task is intentionally the release-facing docs, roadmap, changelog, clippy, and coverage closeout requested by the user.
- Canon-side contract publication and tests are included inside the same task plan because this feature is bilateral by design.