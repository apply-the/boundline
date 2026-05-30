---
description: "Task list for guidance and guardian capabilities implementation"
---

# Tasks: Guidance And Guardian Capabilities

**Input**: Design documents from `/specs/054-guidance-guardian-capabilities/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Validation**: Validation tasks are mandatory because this slice changes planning behavior, post-step verification, persisted trace projection, and operator-visible CLI surfaces.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`US1`, `US2`, `US3`, `US4`, `US5`)
- Include exact file paths in descriptions

## Phase 0: Release & Planning Baseline

**Purpose**: Establish the required release move and planning compliance notes before runtime changes begin

- [x] T001 Bump the Boundline workspace version from `0.53.0` to `0.54.0` in `Cargo.toml` and update `CHANGELOG.md`
- [x] T002 [P] Re-check current OpenAI, Anthropic, and Google provider docs against `assistant/catalog/model-catalog.toml` and keep the explicit no-change audit current in `specs/054-guidance-guardian-capabilities/research.md`

---

## Phase 1: Foundational (Blocking Prerequisites)

**Purpose**: Shared capability primitives, persisted state, and test entry points

**⚠️ CRITICAL**: No user story work should begin until this phase is complete

- [x] T003 Create typed capability, execution, and finding primitives in `src/domain/guidance.rs` and export them from `src/domain.rs` and `src/lib.rs`
- [x] T004 [P] Extend persisted planning and trace projection models for guidance and guardian state in `src/domain/goal_plan.rs` and `src/domain/trace.rs`
- [x] T005 [P] Add shared discovery, precedence, runtime-evidence ranking, and execution-limit scaffolding in `src/orchestrator/guidance_runtime.rs` and export it from `src/orchestrator.rs`
- [x] T006 [P] Add foundational unit and contract coverage entry points in `tests/unit/guidance_runtime.rs` and `tests/contract/capability_manifest_contract.rs`

**Checkpoint**: Shared capability models and scaffolding are ready

---

## Phase 2: User Story 1 - Load And Resolve Guidance During A Bounded Delivery Session (Priority: P1) 🎯 MVP

**Goal**: Resolve guidance sources before planning or bounded execution and persist source authority plus precedence decisions

**Independent Test**: A workspace with shared assets, workspace overrides, and optional Canon standards produces a stable resolution result with loaded and skipped source provenance during `boundline plan`

### Validation for User Story 1

- [x] T007 [P] [US1] Add contract coverage for guidance manifests, workspace overrides, and precedence disclosure in `tests/contract/capability_manifest_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for plan-time guidance resolution, local-only execution, and missing Canon disclosure in `tests/integration/cli_guidance_resolution.rs`

### Implementation for User Story 1

- [x] T009 [P] [US1] Create shared built-in clean-code, language, framework, testing-framework, and architecture guidance assets plus pack manifests in `assistant/guidance/` and `assistant/packs/`
- [x] T010 [US1] Implement guidance discovery, runtime-evidence-aware precedence resolution, and skipped-source disclosure in `src/orchestrator/guidance_runtime.rs`
- [x] T011 [US1] Persist guidance-resolution summaries and provenance in `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`
- [x] T012 [US1] Project resolved guidance sources through `src/cli/session.rs` and `src/cli/output.rs`

**Checkpoint**: User Story 1 is independently valid with inspectable guidance resolution

---

## Phase 3: User Story 2 - Execute Guardian Checks And Emit Structured Findings (Priority: P1)

**Goal**: Run guardian checks after bounded work and emit structured findings, failures, or degraded outcomes

**Independent Test**: A bounded step that violates a configured guardian produces persisted findings, while failed or incomplete guardian execution remains explicit and inspectable

### Validation for User Story 2

- [x] T013 [P] [US2] Add unit coverage for finding emission, raw deterministic failure capture, and degraded guardian outcomes in `tests/unit/guidance_runtime.rs`
- [x] T014 [P] [US2] Add integration coverage for guardian findings and failure projection in `tests/integration/cli_guardian_findings.rs`

### Implementation for User Story 2

- [x] T015 [P] [US2] Create shared built-in grouped guardian assets for clean-code, language/framework, testing, and architecture checks in `assistant/guardians/` and extend manifest parsing in `src/orchestrator/guidance_runtime.rs`
- [x] T016 [US2] Implement structured finding, failure, and degradation recording in `src/domain/guidance.rs` and `src/domain/trace.rs`
- [x] T017 [US2] Invoke guardian execution after bounded work with explicit per-phase count and timeout enforcement in `src/orchestrator/session_runtime.rs`
- [x] T018 [US2] Persist guardian finding summaries and failure visibility in `src/domain/goal_plan.rs`, `src/cli/session.rs`, and `src/cli/output.rs`

**Checkpoint**: User Stories 1 and 2 both work independently with persisted finding output

---

## Phase 4: User Story 3 - Deterministic-Before-LLM Guardian Execution (Priority: P2)

**Goal**: Enforce deterministic-before-LLM ordering, explicit skip logic, and explicit degradation when routing is unavailable

**Independent Test**: A phase with deterministic and semantic guardians shows ordered execution, redundant semantic skips after blocking findings, and explicit route-unavailable degradation when needed

### Validation for User Story 3

- [x] T019 [P] [US3] Add unit coverage for guardian ordering, redundant-skip behavior, and hybrid execution staging in `tests/unit/guidance_runtime.rs`
- [x] T020 [P] [US3] Add integration coverage for route-unavailable degradation and deterministic-block skip paths in `tests/integration/cli_guardian_routing.rs`

### Implementation for User Story 3

- [x] T021 [P] [US3] Reuse existing planning, implementation, verification, and review route slots for semantic guardians in `src/domain/configuration.rs` and `src/orchestrator/guidance_runtime.rs`
- [x] T022 [US3] Implement deterministic-before-LLM ordering, hybrid staging, and explicit skip rules in `src/orchestrator/guidance_runtime.rs`
- [x] T023 [US3] Record route-slot provenance and degraded execution outcomes in `src/domain/guidance.rs`, `src/domain/trace.rs`, and `src/orchestrator/session_runtime.rs`

**Checkpoint**: User Story 3 is independently valid with explicit ordering and degradation semantics

---

## Phase 5: User Story 4 - Workspace Injection Of Custom Guidance And Guardians (Priority: P2)

**Goal**: Support workspace-local override files without modifying shared packs and disclose loaded versus skipped override sources

**Independent Test**: Valid `.boundline/guidance/` and `.boundline/guardians/` inputs override shared entries, while invalid files are skipped with explicit load errors

### Validation for User Story 4

- [x] T024 [P] [US4] Add contract coverage for workspace override discovery, invalid guardian TOML, and skipped-source reporting in `tests/contract/capability_manifest_contract.rs`
- [x] T025 [P] [US4] Add integration coverage for workspace override precedence and invalid-file handling in `tests/integration/cli_guidance_resolution.rs`

### Implementation for User Story 4

- [x] T026 [P] [US4] Implement workspace-local guidance and guardian discovery under `src/orchestrator/guidance_runtime.rs`
- [x] T027 [US4] Persist loaded and skipped workspace override disclosure in `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`

**Checkpoint**: User Story 4 is independently valid with explicit override and skipped-source behavior

---

## Phase 6: User Story 5 - Guidance And Guardian Lifecycle Integration (Priority: P3)

**Goal**: Apply guidance and guardians only in their declared lifecycle phases and project the same persisted story through session-native surfaces

**Independent Test**: Planning-only guidance and implementation-only guardians apply in the correct phases, and `status`, `next`, and `inspect` show the same persisted capability and finding story without recomputation

### Validation for User Story 5

- [x] T028 [P] [US5] Add integration coverage for `applies_to` phase gating and persisted projection in `tests/integration/cli_guidance_guardian_projection.rs`
- [x] T029 [P] [US5] Add unit coverage for projection helpers and phase filtering in `tests/unit/guidance_runtime.rs` and `src/domain/goal_plan.rs`

### Implementation for User Story 5

- [x] T030 [P] [US5] Implement phase-aware capability selection and persistence in `src/orchestrator/goal_planner.rs` and `src/orchestrator/session_runtime.rs`
- [x] T031 [US5] Extend session-native and inspect output for capability resolution and guardian timelines in `src/cli/output.rs` and `src/cli/session.rs`

**Checkpoint**: All user stories are independently functional and inspectable

---

## Phase 7: Verification & Closeout

**Purpose**: Finish documentation, code comments, lint, tests, and coverage

- [x] T032 [P] Update operator-facing docs and roadmap in `docs/architecture.md`, `docs/configuration.md`, `docs/getting-started.md`, `README.md`, and `ROADMAP.md`
- [x] T033 [P] Add concise code documentation in `src/domain/guidance.rs`, `src/orchestrator/guidance_runtime.rs`, and any other modified runtime files where source resolution or guardian ordering is non-obvious
- [x] T034 Run `cargo fmt --all` in `repo root`
- [x] T035 Fix all clippy warnings in modified code and run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in `repo root`
- [x] T036 Run `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features` in `repo root`
- [x] T037 Refresh coverage for all modified or new Rust files in `repo root` and confirm at least 95% coverage for each file

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 0** starts immediately and includes the required first task version bump
- **Phase 1** depends on Phase 0 and blocks all story work
- **Phases 2-6** depend on Phase 1 and should proceed in priority order
- **Phase 7** depends on all desired user stories

### User Story Dependencies

- **US1** delivers the MVP and depends only on the foundational capability primitives
- **US2** builds on US1 resolution state and adds guardian execution plus findings
- **US3** builds on US2 guardian execution and tightens ordering and routing degradation
- **US4** builds on US1 and US2 discovery behavior to support workspace-local overrides
- **US5** depends on the persisted resolution and finding state from earlier stories and projects it through the existing session-native surfaces

### Parallel Opportunities

- T002 can run in parallel with the version bump once release intent is fixed
- T004, T005, and T006 can run in parallel after T003 starts
- Validation tasks marked [P] can run in parallel within each story
- T032 and T033 can run in parallel once behavior stabilizes

## Notes

- The first task is the Boundline version bump, as requested
- The final task is modified-file coverage verification at 95% or higher, as requested
- The feature keeps model catalog management out of functional scope while still preserving the constitution-required provider-doc audit in planning and task closeout
- Existing routing slots remain authoritative for semantic guardian execution
