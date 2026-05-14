# Tasks: Delivery Control Consumer

**Input**: Design documents from `specs/051-delivery-control-consumer/`  
**Prerequisites**: plan.md (required), spec.md (required), research.md,
data-model.md, quickstart.md, contracts/

**Tests**: Validation tasks are required because this slice changes execution
decisions, stop behavior, inspection output, and repo-visible consumption of
Canon knowledge. This task list also includes the required catalog-refresh
verification task.

**Organization**: Tasks are grouped by user story so each slice can deliver
bounded value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Consumer Pin & Catalog Check)

**Purpose**: Freeze the Boundline-side consumer stance before runtime changes.

- [ ] T001 Pin the Canon control-layer contract line in `specs/051-delivery-control-consumer/contracts/canon-project-memory-consumer-contract.md` and reference the stable Canon owner-side path
- [ ] T002 [P] Re-check current public provider docs against `assistant/catalog/model-catalog.toml` and update or confirm the no-change result in `specs/051-delivery-control-consumer/research.md`
- [ ] T003 [P] Finalize the V1 project-index and workflow semantics in `specs/051-delivery-control-consumer/contracts/project-index-contract.md` and `specs/051-delivery-control-consumer/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared consumer primitives for project index, workflow reuse, and
tiered stop behavior.

**⚠️ CRITICAL**: No user story work starts until this phase is complete.

- [ ] T004 Create `src/domain/project_index.rs` with parsing and validation models for `project.boundline.toml`
- [ ] T005 [P] Extend `src/domain/project_memory.rs` with V1 hard-stop versus warning classification primitives and producer-attribution helpers
- [ ] T006 [P] Extend `src/domain/workflow.rs` with delivery-path entry support inside the existing workflow registry model
- [ ] T007 Register `src/domain/project_index.rs` in `src/domain.rs` and `src/lib.rs`
- [ ] T008 [P] Add deterministic fixtures plus a foundational checkpoint validation case for `project.boundline.toml`, workflow delivery-path entries, and mixed-producer evidence under `tests/fixtures/delivery_control_consumer/` and `tests/unit/`

**Checkpoint**: Boundline has stable consumer primitives for project index,
workflow reuse, fixtures, and explicit stop classification.

---

## Phase 3: User Story 1 - Plan Credibly From Repo-Visible Canon Knowledge (Priority: P1) 🎯 MVP

**Goal**: Boundline consumes stable Canon knowledge, warns on non-fatal gaps,
and hard-stops on missing required producer facts.

**Independent Test**: Run planning against stable, stale, blocked, and
missing-artifact fixtures and verify continue, warning, or hard-stop behavior.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add unit tests for V1 hard-stop and warning classification in `src/domain/project_memory.rs`
- [ ] T010 [P] [US1] Add integration test for stable, stale, and blocked planning inputs in `tests/integration/delivery_control_consumer.rs`
- [ ] T011 [P] [US1] Add integration test for missing required source artifacts and blocked governance in `tests/integration/delivery_control_consumer_blocked.rs`

### Implementation for User Story 1

- [ ] T012 [US1] Read `project.boundline.toml` and Canon repo-visible inputs during planning-context assembly in `src/orchestrator/session_runtime.rs`, using `[docs]` overrides when present and `docs/project/` plus `docs/evidence/` otherwise
- [ ] T013 [US1] Apply credible, warning, and hard-stop outcomes in `src/orchestrator/goal_planner.rs`
- [ ] T014 [US1] Persist and surface project-memory-derived outcomes in `src/orchestrator/session_runtime.rs` and `src/domain/trace.rs`
- [ ] T015 [US1] Keep session-native planning functional when Canon inputs are absent but other credible context remains available in `src/orchestrator/session_runtime.rs`

**Checkpoint**: Boundline can plan credibly from Canon knowledge without hidden fallbacks.

---

## Phase 4: User Story 2 - Extend Existing Runtime Surfaces Without Registry Collisions (Priority: P1)

**Goal**: Delivery-control consumption reuses current workflow and cluster
surfaces instead of introducing competing registries.

**Independent Test**: Resolve a project index plus cluster state and verify that
delivery paths are carried through `.boundline/workflows.toml` without adding a
second registry file.

### Tests for User Story 2

- [ ] T016 [P] [US2] Add unit tests for `project.boundline.toml` parsing and cluster linkage in `src/domain/project_index.rs`
- [ ] T017 [P] [US2] Add integration test for delivery-path resolution via `.boundline/workflows.toml` in `tests/integration/delivery_path_registry.rs`

### Implementation for User Story 2

- [ ] T018 [US2] Implement `project.boundline.toml` loading and semantics joins with cluster state in `src/orchestrator/session_runtime.rs` and `src/adapters/cluster_store.rs`
- [ ] T019 [US2] Extend workflow discovery and resolution to carry `delivery_paths` entries in `src/domain/workflow.rs` and `src/cli/workflow.rs`, rejecting unsupported V1 stage identifiers explicitly
- [ ] T020 [US2] Surface project-index and delivery-path context in `src/cli/session.rs` and `src/cli/workflow.rs`

**Checkpoint**: Project semantics, cluster topology, and workflow delivery paths are distinct and inspectable.

---

## Phase 5: User Story 3 - Expose Contract Compatibility And Mixed Evidence Authorship (Priority: P2)

**Goal**: Session-native inspection surfaces explain contract compatibility,
Canon refs, and mixed-producer evidence attribution.

**Independent Test**: Inspect supported and unsupported contract scenarios plus
mixed-producer evidence fixtures and verify the user can tell whether the block
is producer-side or consumer-side.

### Tests for User Story 3

- [ ] T021 [P] [US3] Add unit tests for unknown-major rejection and additive-field tolerance in `src/domain/project_memory.rs`
- [ ] T022 [P] [US3] Add integration test for mixed-producer evidence attribution in `tests/integration/delivery_control_consumer_attribution.rs`

### Implementation for User Story 3

- [ ] T023 [US3] Surface Canon refs and consumer compatibility state in `src/cli/session.rs`
- [ ] T024 [US3] Record producer-attributed evidence and compatibility facts in `src/domain/trace.rs` and `src/adapters/trace_store.rs`
- [ ] T025 [US3] Preserve the shared `project-memory:managed` attribution rules when consuming `docs/evidence/` in `src/domain/project_memory.rs`

**Checkpoint**: Users can tell whether a session outcome came from Canon facts, Boundline policy, or missing evidence.

---

## Final Phase: Verification & Cross-Cutting Concerns

**Purpose**: Validate consumer behavior, docs, and coverage end to end.

- [ ] T026 [P] Update `docs/architecture.md`, `docs/getting-started.md`, and `docs/configuration.md` for project index, workflow reuse, and tiered stop rules
- [ ] T027 [P] Update `CHANGELOG.md` and `ROADMAP.md`
- [ ] T028 Run targeted Boundline tests for `src/domain/project_memory.rs`, `src/domain/project_index.rs`, `src/orchestrator/session_runtime.rs`, `src/orchestrator/goal_planner.rs`, and `src/cli/session.rs`
- [ ] T029 Run `cargo test --no-run --all-targets`
- [ ] T030 Run `cargo fmt --check`
- [ ] T031 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] T032 Run `cargo nextest run`
- [ ] T033 Refresh `lcov.info` if coverage is regenerated for modified files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately and freezes the consumer contract pin plus catalog result
- **Foundational (Phase 2)**: depends on Setup completion and blocks all story work
- **User Stories (Phase 3-5)**: depend on Foundational completion; US1 establishes the primary planning slice, US2 reuses workflow and cluster surfaces, US3 builds on the consumer and inspection model
- **Final Phase**: depends on all selected stories completing

### User Story Dependencies

- **User Story 1 (P1)**: starts after Foundational and delivers the smallest useful control-layer capability
- **User Story 2 (P1)**: starts after Foundational and depends on the new project-index and workflow primitives
- **User Story 3 (P2)**: starts after Foundational and should land after the consumer compatibility model is stable

### Within Each User Story

- Test tasks must fail before implementation when the story changes executable behavior
- Domain primitives land before orchestration or CLI wiring
- Planning and state updates land before inspection-surface polish
- Story checkpoints must pass before final verification

## Parallel Opportunities

- T002 and T003 can run in parallel during Setup
- T005, T006, and T008 can run in parallel during Foundational
- T009, T010, and T011 can run in parallel for US1
- T016 and T017 can run in parallel for US2
- T021 and T022 can run in parallel for US3
- T026 and T027 can run in parallel during the final phase

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Validate explicit continue, warning, and hard-stop behavior before expanding scope

### Incremental Delivery

1. Ship credible planning from repo-visible Canon knowledge
2. Add project-index and workflow-registry reuse without collisions
3. Expose inspection-state and producer attribution once the consumer model is stable
4. Run full verification only after the runtime, docs, and tests align

## Notes

- The first task is intentionally a consumer pin, not a release-version bump.
- The catalog refresh task is mandatory even when the result is no change.
- The consumer note stays in Boundline, but the canonical contract source of truth remains in Canon.