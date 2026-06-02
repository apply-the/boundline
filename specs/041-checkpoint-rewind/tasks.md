# Tasks: Checkpoint Rewind

**Input**: Design documents from `/specs/041-checkpoint-rewind/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes the
workspace layout, mutating execution safety, restore behavior, clustered state,
and operator-facing CLI summaries.

**Organization**: Tasks are grouped by user story so each delivered behavior
remains independently testable while still shipping as one full `0.41.0`
release slice.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the 041 spec pack and workspace skeleton.

- [ ] T001 Keep `specs/041-checkpoint-rewind/spec.md`, `specs/041-checkpoint-rewind/plan.md`, `specs/041-checkpoint-rewind/research.md`, `specs/041-checkpoint-rewind/data-model.md`, `specs/041-checkpoint-rewind/contracts/`, and `specs/041-checkpoint-rewind/quickstart.md` synchronized with the implementation
- [ ] T002 Convert `Cargo.toml` into the workspace root manifest and add member manifests under `crates/boundline-core/Cargo.toml`, `crates/boundline-adapters/Cargo.toml`, and `crates/boundline-cli/Cargo.toml`
- [ ] T003 [P] Create the workspace source layout under `crates/boundline-core/src/`, `crates/boundline-adapters/src/`, and `crates/boundline-cli/src/` while preserving repo-root test directories

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Split the monolith and add shared checkpoint primitives that every story depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Move the current domain and orchestrator modules from `src/domain/`, `src/domain.rs`, `src/orchestrator/`, and `src/orchestrator.rs` into `crates/boundline-core/src/`
- [ ] T005 [P] Move the current adapter and fixture modules from `src/adapters/`, `src/adapters.rs`, and `src/fixture.rs` into `crates/boundline-adapters/src/`
- [ ] T006 [P] Move the CLI modules and binary entrypoint from `src/cli/`, `src/cli.rs`, and `src/bin/` into `crates/boundline-cli/src/`
- [ ] T007 Rewire `src/lib.rs`, `src/registry/`, and repo-root public exports so workspace members compile without cyclic dependencies and repo-root command entry remains stable
- [ ] T008 [P] Add shared checkpoint domain primitives in `crates/boundline-core/src/domain/checkpoint.rs` and re-export them through the appropriate crate `lib.rs` files
- [ ] T009 [P] Add shared checkpoint store and persistence primitives in `crates/boundline-adapters/src/adapters/checkpoint_store.rs` plus any supporting fixture helpers
- [ ] T010 [P] Extend `tests/unit.rs`, `tests/integration.rs`, and `tests/contract.rs` if the workspace refoundation requires harness updates for Cargo discovery

**Checkpoint**: The workspace layout and shared checkpoint primitives exist, compile boundaries are explicit, and feature work can proceed on top of them.

---

## Phase 3: User Story 1 - Capture A Reversible Workspace Snapshot (Priority: P1) 🎯 MVP

**Goal**: Create implicit bounded checkpoints before mutating `run` and `step` actions, including clustered ownership.

**Independent Test**: Execute mutating `run` and `step` flows and verify that checkpoint manifests are created before mutation with explicit file-state capture.

### Tests for User Story 1

- [ ] T011 [P] [US1] Add unit coverage for checkpoint manifest and captured-file validation in `tests/unit/checkpoint_model.rs`
- [ ] T012 [P] [US1] Add integration coverage for implicit checkpoint creation on mutating `run` and `step` in `tests/integration/checkpoint_creation_flow.rs`
- [ ] T013 [P] [US1] Add contract coverage for observable checkpoint creation in `tests/contract/checkpoint_contract.rs`

### Implementation for User Story 1

- [ ] T014 [US1] Implement checkpoint manifest and captured-file state logic in `crates/boundline-core/src/domain/checkpoint.rs`
- [ ] T015 [US1] Implement filesystem-backed checkpoint persistence in `crates/boundline-adapters/src/adapters/checkpoint_store.rs`
- [ ] T016 [US1] Extend `crates/boundline-core/src/orchestrator/session_runtime.rs` to compute bounded checkpoint file sets from runtime evidence and create checkpoints before mutating execution
- [ ] T017 [US1] Extend `crates/boundline-cli/src/cli/session.rs` so `run` and `step` capture checkpoints on the session-native surface and preserve cluster ownership explicitly

**Checkpoint**: Mutating execution creates an inspectable checkpoint before the first mutation lands.

---

## Phase 4: User Story 2 - Restore A Checkpoint Explicitly And Safely (Priority: P2)

**Goal**: Expose checkpoint list and restore commands with safe refusal by default and explicit forced override.

**Independent Test**: Create a checkpoint, introduce newer conflicting edits, and verify explicit refusal or successful restore with `--force`.

### Tests for User Story 2

- [ ] T018 [P] [US2] Add unit coverage for restore conflict detection and restore-record state in `tests/unit/checkpoint_restore.rs`
- [ ] T019 [P] [US2] Add integration coverage for `checkpoint list` and `checkpoint restore` in `tests/integration/checkpoint_restore_flow.rs`
- [ ] T020 [P] [US2] Add contract coverage for safe refusal and forced restore output in `tests/contract/checkpoint_restore_contract.rs`

### Implementation for User Story 2

- [ ] T021 [US2] Extend `crates/boundline-adapters/src/adapters/checkpoint_store.rs` with restore conflict detection, restore execution, and restore history persistence
- [ ] T022 [US2] Add the `Checkpoint` command group to `crates/boundline-cli/src/cli.rs` and implement handlers in `crates/boundline-cli/src/cli/checkpoint.rs`
- [ ] T023 [US2] Extend `crates/boundline-core/src/domain/session.rs`, `crates/boundline-core/src/domain/trace.rs`, and any supporting projection models with checkpoint restore state
- [ ] T024 [US2] Extend `crates/boundline-cli/src/cli/output.rs` and `crates/boundline-cli/src/cli/inspect.rs` to render restore refusal, conflicts, and forced restore outcomes clearly

**Checkpoint**: Operators can list checkpoints and restore them safely without silent overwrite.

---

## Phase 5: User Story 3 - Keep Checkpoint Authority Visible Across CLI Surfaces (Priority: P3)

**Goal**: Surface latest checkpoint and restore guidance through `run`, `status`, `next`, and `inspect` without confusing route or governance authority.

**Independent Test**: Cause a mutating run to fail or block after checkpoint creation and verify the same checkpoint story appears across the read-side surfaces.

### Tests for User Story 3

- [ ] T025 [P] [US3] Add integration coverage for checkpoint projection on `status`, `next`, and `inspect` in `tests/integration/checkpoint_projection_flow.rs`
- [ ] T026 [P] [US3] Add contract coverage for CLI-visible checkpoint projection in `tests/contract/checkpoint_projection_contract.rs`
- [ ] T027 [P] [US3] Add unit coverage for checkpoint projection models in `tests/unit/checkpoint_projection.rs`

### Implementation for User Story 3

- [ ] T028 [US3] Extend `crates/boundline-core/src/domain/follow_through.rs`, `crates/boundline-core/src/domain/session.rs`, and `crates/boundline-core/src/domain/trace.rs` with checkpoint projection fields and authority rules
- [ ] T029 [US3] Extend `crates/boundline-cli/src/cli/session.rs`, `crates/boundline-cli/src/cli/output.rs`, and `crates/boundline-cli/src/cli/inspect.rs` to render checkpoint identity and restore hints on `run`, `status`, `next`, and `inspect`
- [ ] T030 [US3] Extend `crates/boundline-core/src/orchestrator/session_runtime.rs` so checkpoint guidance persists through native, clustered, and explicit compatibility follow-up paths

**Checkpoint**: Checkpoint authority is visible from the normal follow-through surfaces.

---

## Phase 6: User Story 4 - Ship 0.41.0 As A Rust Workspace Without Changing The Product Boundary (Priority: P4)

**Goal**: Close the feature as a coherent `0.41.0` release with a stable repo-root command surface and clearer docs.

**Independent Test**: Build, run, and test from the repository root, then confirm the updated docs preserve the Boundline-versus-Canon boundary and two-layer README.

### Tests for User Story 4

- [ ] T031 [P] [US4] Add or update focused contract coverage for release metadata, README layering, and workspace-root command continuity in `tests/contract/`

### Implementation for User Story 4

- [ ] T032 [US4] Bump the release version to `0.41.0` in `Cargo.toml`, generated workspace member manifests, and `Cargo.lock`
- [ ] T033 [US4] Update README, architecture docs, getting-started, contributor guidance, and assistant guidance in `README.md`, `tech-docs/`, `CONTRIBUTING.md`, `assistant/`, and `AGENTS.md` to keep a brutal quick path separate from advanced architecture
- [ ] T034 [US4] Update release metadata and workspace-root validation surfaces in `distribution/`, `CHANGELOG.md`, and `ROADMAP.md` so `0.41.0` is recorded as delivered and the future roadmap no longer includes 041 as upcoming

**Checkpoint**: The `0.41.0` product, docs, and release surface describe the same checkpoint-and-workspace story.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the full slice and close remaining quality gates.

- [ ] T035 [P] Run formatting with `cargo fmt --all`
- [ ] T036 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] T037 Run compile-oriented and broader validation with `cargo test --workspace --all-targets` and `cargo nextest run --workspace --all-features`
- [ ] T038 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [ ] T039 Mark completed tasks in `specs/041-checkpoint-rewind/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because restore depends on persisted checkpoint state.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because projection depends on created and restorable checkpoint state.
- **User Story 4 (Phase 6)**: Depends on runtime behavior and workspace refoundation being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T003 can run in parallel with T002 once the workspace member list is fixed.
- T005, T006, and T010 can proceed in parallel after the workspace root manifest is defined.
- Within each user story, tasks marked `[P]` can run in parallel before implementation tasks that touch the same files.
- T031 can be prepared while docs and release metadata are being updated, but final contract assertions wait for the completed release surface.

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational work.
2. Complete User Story 1 and validate that mutating execution always captures a checkpoint first.
3. Use that bounded checkpoint state as the base for restore commands, projection, and release closure.

### Incremental Delivery

1. Refound the repository into the Rust workspace and add shared checkpoint primitives.
2. Add implicit checkpoint creation on mutating `run` and `step`.
3. Add explicit list and restore commands with safe refusal.
4. Project checkpoint guidance across the existing follow-through surfaces.
5. Close the release with docs, roadmap, changelog, version bump, and validation.

## Notes

- This feature is intentionally full-slice: checkpoint safety and Rust workspace refoundation ship together.
- The final implementation summary must include a descriptive commit message for the completed feature.