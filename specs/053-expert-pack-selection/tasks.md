---
description: "Task list for expert pack selection implementation"
---

# Tasks: Expert Pack Selection

**Input**: Design documents from `/specs/053-expert-pack-selection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Validation**: Validation tasks are mandatory because this slice changes
planning behavior, runtime-role recommendation, trace projection, and
operator-visible CLI surfaces.

**Organization**: Tasks are grouped by user story so each slice can be
implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story or closeout group this task belongs to (`US1`, `US2`, `US3`, `Closeout`)
- Include exact file paths in descriptions

## Phase 0: Release & Catalog Baseline

**Purpose**: Establish the first required release move and provider-doc audit
for the branch

- [x] T001 Bump the Boundline workspace version from `0.52.0` to `0.53.0` in `Cargo.toml` and update `CHANGELOG.md`
- [x] T002 [P] Re-check current OpenAI, Anthropic, and Google provider docs against `assistant/catalog/model-catalog.toml` and record the explicit no-change result in `specs/053-expert-pack-selection/research.md`

---

## Phase 1: Foundational (Blocking Prerequisites)

**Purpose**: Shared expert-pack primitives and persisted selection state

**⚠️ CRITICAL**: No user story work should begin until this phase is complete

- [x] T003 Extend shared expert-pack selection state and projection helpers in `src/domain/goal_plan.rs`
- [x] T004 [P] Define deterministic built-in expert-pack selection helpers in `src/orchestrator/goal_planner.rs`
- [x] T005 [P] Add Canon-governed expertise input read-side helpers and compatibility gates in `src/domain/project_memory.rs` and `src/orchestrator/goal_planner.rs`
- [x] T006 [P] Reuse existing unit and integration coverage surfaces for this slice in `src/domain/goal_plan.rs`, `src/orchestrator/goal_planner.rs`, and `tests/integration/cli_trace_inspection.rs`

**Checkpoint**: Shared expert-pack primitives and test entry points are ready

---

## Phase 2: User Story 1 - Select Built-In Experts Before Planning (Priority: P1) 🎯 MVP

**Goal**: Select deterministic built-in expert packs and runtime-role recommendations before planning continues

**Independent Test**: A workspace with matching domain templates can produce a stable local-only expert-pack selection outcome during planning

### Validation for User Story 1

- [x] T007 [P] [US1] Add deterministic selection and ordering coverage in `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`
- [x] T008 [P] [US1] Add local-only planning-path coverage in `src/orchestrator/goal_planner.rs`

### Implementation for User Story 1

- [x] T009 [P] [US1] Implement built-in expert-pack definitions and family matching in `src/orchestrator/goal_planner.rs`
- [x] T010 [US1] Persist expert-pack selection outcome and summary helpers in `src/domain/goal_plan.rs`
- [x] T011 [US1] Compute deterministic expert selection during planning in `src/orchestrator/goal_planner.rs`
- [x] T012 [US1] Align the MVP contract wording in `specs/053-expert-pack-selection/data-model.md` and `specs/053-expert-pack-selection/contracts/expert-pack-selection-contract.md`

**Checkpoint**: User Story 1 is independently valid with local-only expert selection

---

## Phase 3: User Story 2 - Apply Effective Overrides And Governed Expertise Inputs (Priority: P1)

**Goal**: Respect effective Boundline precedence while treating supported Canon `v1` `domain-language` and `domain-model` expertise input as optional supporting evidence

**Independent Test**: Conflicting local overrides and compatible or incompatible Canon `v1` expertise input produce explicit selected, suppressed, or rejected candidates without breaking the local-only path

### Validation for User Story 2

- [x] T013 [P] [US2] Add coverage for effective precedence, unroutable runtime roles, and explicit suppression in `src/orchestrator/goal_planner.rs`
- [x] T014 [P] [US2] Add compatibility coverage for supported Canon `v1` `domain-language` and `domain-model` inputs plus ignored unsupported-line, unknown-kind, non-matching-domain, blocked, pending, proposal, evidence, index, or conflicting-state paths in `src/domain/project_memory.rs` and `src/orchestrator/goal_planner.rs`

### Implementation for User Story 2

- [x] T015 [P] [US2] Apply effective reviewer-role and domain-template precedence to candidate suppression in `src/orchestrator/goal_planner.rs`
- [x] T016 [US2] Integrate optional Canon expertise inputs and explicit rejection reasons for unsupported-line, unknown-kind, non-matching-domain, blocked, pending, proposal, evidence, index, or conflicting inputs in `src/orchestrator/goal_planner.rs`
- [x] T017 [US2] Preserve `none-selected`, rejection projection fields, and Canon metadata handoff requirements including `expertise_input.domain_families` in `src/domain/goal_plan.rs` and `specs/053-expert-pack-selection/contracts/expert-pack-selection-contract.md`

**Checkpoint**: User Stories 1 and 2 both work independently and preserve the Canon boundary

---

## Phase 4: User Story 3 - Inspect Selected And Rejected Candidates (Priority: P2)

**Goal**: Surface expert-pack selection and rejection reasoning through session-native and trace views

**Independent Test**: `status`, `next`, and `inspect` expose the same persisted expert-selection outcome without recomputing hidden choices

### Validation for User Story 3

- [x] T018 [P] [US3] Add session and CLI projection coverage in `tests/integration/cli_trace_inspection.rs` and `src/domain/goal_plan.rs`
- [x] T019 [P] [US3] Add persisted expert-selection trace coverage in `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`

### Implementation for User Story 3

- [x] T020 [P] [US3] Project expert-selection summaries through existing session-native surfaces via `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`
- [x] T021 [US3] Extend persisted provenance used by inspect trace surfaces in `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`
- [x] T022 [US3] Refresh `specs/053-expert-pack-selection/quickstart.md` and `specs/053-expert-pack-selection/contracts/expert-selection-trace-contract.md`

**Checkpoint**: All user stories are independently functional and inspectable

---

## Phase 5: Verification & Closeout

**Purpose**: Finish docs, formatting, lint, tests, and coverage

- [x] T023 [P] Update operator-facing docs in `tech-docs/configuration.md`, `tech-docs/architecture.md`, `tech-docs/getting-started.md`, `README.md`, and `ROADMAP.md`
- [x] T024 Run `cargo fmt --all` in `repo root`
- [x] T025 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in `repo root`
- [x] T026 Run `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features` in `repo root`
- [x] T027 Run focused modified-file coverage in `repo root` and confirm at least 95% coverage for every modified file

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 0** starts immediately and includes the required first task version bump
- **Phase 1** depends on Phase 0 and blocks story work
- **Phases 2-4** depend on Phase 1 and should proceed in priority order
- **Phase 5** depends on all desired user stories

### User Story Dependencies

- **US1** delivers the MVP and depends only on the foundational selection primitives
- **US2** builds on US1 selection state and adds precedence plus Canon-boundary behavior
- **US3** depends on the persisted outcome from US1 and US2 and projects it through operator-visible surfaces

### Parallel Opportunities

- T002 can run in parallel with the version bump once release intent is fixed
- T004, T005, and T006 can run in parallel after T003 starts
- Validation tasks marked [P] can run in parallel within each story
- T023 can run in parallel with validation once behavior stabilizes

## Notes

- The first task is the Boundline version bump, as requested
- The final task is modified-file coverage verification at 95% or higher, as requested
- Canon expertise input remains optional throughout the slice
- Implementation reused existing planner and goal-plan coverage surfaces instead of creating new dedicated `expert_pack_selection*.rs` test files.
