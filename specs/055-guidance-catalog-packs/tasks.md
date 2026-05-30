---
description: "Task list for guidance catalog and guardian rule packs implementation"
---

# Tasks: Guidance Catalog And Guardian Rule Packs

**Input**: Design documents from `/specs/055-guidance-catalog-packs/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Validation**: Validation tasks are mandatory because this slice changes pack discovery, validation behavior, persisted trace projection, and operator-visible CLI surfaces.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`US1`, `US2`, `US3`, `US4`)
- Include exact file paths in descriptions

## Phase 0: Release & Planning Baseline

**Purpose**: Establish the release move and required planning audit before catalog runtime changes begin

- [x] T001 Bump the Boundline workspace version from `0.54.0` to `0.55.0` in `Cargo.toml` and update `CHANGELOG.md`
- [x] T002 [P] Re-check current OpenAI, Anthropic, and Google provider docs against `assistant/catalog/model-catalog.toml` and record the explicit no-change or delta result in `specs/055-guidance-catalog-packs/research.md`
- [x] T003 [P] Audit the former Phase 7 design input, record normalization decisions in `specs/055-guidance-catalog-packs/research.md`, and keep `specs/055-guidance-catalog-packs/reference/` aligned with the canonical 055 contracts

---

## Phase 1: Foundational (Blocking Prerequisites)

**Purpose**: Shared catalog models, persisted projection, and validation entry points

**⚠️ CRITICAL**: No user story work should begin until this phase is complete

- [x] T004 Create typed catalog manifest, guidance entry, guardian rule seed, and validation finding primitives in `src/domain/guidance_catalog.rs` and export them from `src/domain.rs` and `src/lib.rs`
- [x] T005 [P] Extend runtime consumer, persisted planning, and trace projection models for loaded packs, skipped packs, validation findings, and canonical catalog vocabulary in `src/domain/guidance.rs`, `src/domain/goal_plan.rs`, and `src/domain/trace.rs`
- [x] T006 [P] Add shared discovery, parsing, precedence, and validation scaffolding in `src/orchestrator/guidance_catalog_runtime.rs` and export it from `src/orchestrator.rs`
- [x] T007 [P] Add foundational unit and contract coverage entry points in `tests/unit/guidance_catalog_runtime.rs`, `tests/contract/catalog_manifest_contract.rs`, `tests/contract/guidance_index_contract.rs`, and `tests/contract/guardian_index_contract.rs`

**Checkpoint**: Shared catalog models and validation scaffolding are ready

---

## Phase 2: User Story 1 - Install A Guidance Catalog Pack (Priority: P1) 🎯 MVP

**Goal**: Discover and validate a catalog pack with explicit loaded and skipped outcomes before runtime resolution consumes it

**Independent Test**: A workspace with a valid pack manifest, catalog manifest, guidance index, and guardian index produces a stable loaded-pack result during `boundline plan`

### Validation for User Story 1

- [x] T008 [P] [US1] Add contract coverage for pack manifest parsing and minimum catalog manifest requirements in `tests/contract/catalog_manifest_contract.rs`
- [x] T009 [P] [US1] Add integration coverage for pack discovery, manifest loading, and missing-manifest failure paths in `tests/integration/cli_guidance_catalog.rs`

### Implementation for User Story 1

- [x] T010 [P] [US1] Create a built-in reference guidance catalog pack under `assistant/packs/guidance-catalog/` by normalizing the absorbed Phase 7 design package to the canonical 055 taxonomy and vocabulary
- [x] T011 [US1] Implement pack discovery and manifest parsing in `src/orchestrator/guidance_catalog_runtime.rs`
- [x] T012 [US1] Persist loaded-pack summaries and catalog provenance in `src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`
- [x] T013 [US1] Surface loaded pack information through `src/cli/output.rs` and `src/cli/inspect.rs`

**Checkpoint**: User Story 1 is independently valid with inspectable pack discovery

---

## Phase 3: User Story 2 - Inspect Guidance Resolution From Catalog (Priority: P2)

**Goal**: Project selected entries, authority sources, strength, and override decisions through existing runtime surfaces

**Independent Test**: A workspace with a catalog pack plus a conflicting workspace override shows selected entries, authority source, and override decisions in the resolution trace

### Validation for User Story 2

- [x] T014 [P] [US2] Add unit coverage for authority-source selection, inherited strength, and override decisions in `tests/unit/guidance_catalog_runtime.rs`
- [x] T015 [P] [US2] Add integration coverage for inspectable resolution traces, skipped-source disclosure, and the normalized Phase 7 trace shape in `tests/integration/cli_guidance_catalog.rs`

### Implementation for User Story 2

- [x] T016 [P] [US2] Implement authority-source, strength, and default-disposition projection for catalog entries in `src/orchestrator/guidance_catalog_runtime.rs`, `src/orchestrator/guidance_runtime.rs`, and `src/domain/trace.rs`
- [x] T017 [US2] Extend inspect and status projection with selected entries, skipped entries, and override summaries in `src/cli/output.rs` and `src/cli/inspect.rs`

**Checkpoint**: User Stories 1 and 2 both work independently with trace-visible precedence decisions

---

## Phase 4: User Story 4 - Validate Catalog Shape (Priority: P2)

**Goal**: Reject malformed manifests and entries explicitly while continuing to load valid content where safe

**Independent Test**: Invalid lifecycle labels, unsupported guardian kinds, duplicate IDs, and missing files produce explicit warnings or errors with entry-level detail

### Validation for User Story 4

- [x] T018 [P] [US4] Add contract coverage for invalid lifecycle labels, unsupported guardian kinds, duplicate IDs, missing referenced files, and outdated Phase 7 alias values in `tests/contract/guidance_index_contract.rs` and `tests/contract/guardian_index_contract.rs`
- [x] T019 [P] [US4] Add integration coverage for explicit warning and error visibility in `tests/integration/cli_guidance_catalog.rs`

### Implementation for User Story 4

- [x] T020 [P] [US4] Implement schema-level validation, canonical vocabulary checks, explicit load findings, and Phase 7 normalization diagnostics in `src/orchestrator/guidance_catalog_runtime.rs` and `src/domain/guidance_catalog.rs`
- [x] T021 [US4] Persist validation findings and skipped-pack reasons in `src/domain/goal_plan.rs` and `src/domain/trace.rs`

**Checkpoint**: User Story 4 is independently valid with explicit validation behavior

---

## Phase 5: User Story 3 - Promote Guidance Through Canon (Priority: P3)

**Goal**: Preserve Canon-promotion-compatible content and disclose when Canon authority supersedes shared pack authority

**Independent Test**: A Canon-promotable pack and a Canon-governed version of the same guidance show higher-authority Canon selection without rewriting the content file

### Validation for User Story 3

- [x] T022 [P] [US3] Add unit coverage for Canon-promotable metadata and Canon-over-pack precedence in `tests/unit/guidance_catalog_runtime.rs`
- [x] T023 [P] [US3] Add integration coverage for Canon-versus-pack authority disclosure in `tests/integration/cli_guidance_catalog.rs`

### Implementation for User Story 3

- [x] T024 [P] [US3] Implement Canon-promotion-compatible authority metadata handling in `src/domain/guidance_catalog.rs` and `src/orchestrator/guidance_catalog_runtime.rs`
- [x] T025 [US3] Project Canon-versus-pack authority disclosure through `src/domain/trace.rs` and `src/cli/output.rs`

**Checkpoint**: All user stories are independently functional and inspectable

---

## Phase 6: Verification & Closeout

**Purpose**: Finish docs, code comments, lint, tests, and coverage

- [x] T026 [P] Update operator-facing docs and roadmap in `docs/architecture.md`, `docs/configuration.md`, `docs/getting-started.md`, `README.md`, and `ROADMAP.md`
- [x] T027 [P] Add concise code documentation in `src/domain/guidance_catalog.rs`, `src/orchestrator/guidance_catalog_runtime.rs`, and any other modified runtime files where catalog precedence or validation behavior is non-obvious
- [x] T028 Run `cargo fmt --all` in `repo root`
- [x] T029 Fix all clippy warnings in modified code and run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in `repo root`
- [x] T030 Run `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and refresh coverage for all modified or new Rust files in `repo root`, confirming at least 95% coverage for each file

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 0** starts immediately and includes the required first-task version bump
- **Phase 1** depends on Phase 0 and blocks all story work
- **Phases 2-5** depend on Phase 1 and should proceed in priority order
- **Phase 6** depends on all desired user stories

### User Story Dependencies

- **US1** delivers the MVP and depends only on the foundational catalog primitives
- **US2** builds on US1 discovery state and adds inspectable precedence disclosure
- **US4** builds on US1 discovery behavior to validate catalog shape and emit explicit findings
- **US3** builds on US1 and US2 authority handling to support Canon-promotion-compatible metadata

### Parallel Opportunities

- T002 and T003 can run in parallel with the version bump once release intent is fixed
- T005, T006, and T007 can run in parallel after T004 starts
- Validation tasks marked [P] can run in parallel within each story
- T025 and T026 can run in parallel once behavior stabilizes

## Notes

- The first task is the Boundline version bump, as required by repository convention
- The final task is modified-file coverage verification at 95% or higher
- The feature keeps provider-model routing out of functional scope while still preserving the constitution-required provider-doc audit in planning and task closeout
- S2.1 remains authoritative for guidance and guardian execution; this slice owns the catalog-pack contract consumed by that runtime