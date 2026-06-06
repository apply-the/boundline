# Tasks: Contextual Help And Documentation Architecture (Boundline)

**Input**: Design documents from `/specs/073-contextual-help-docs/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/help-next-output-contract.md`, `quickstart.md`

**Tests**: Test tasks are required. Every new or modified Rust implementation
file must reach at least 95% changed-file coverage.

**Organization**: Tasks are grouped by user story.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Confirm the next release version aligns with the in-progress spec 073 and no version conflict exists with sibling specs
- [x] T002 [P] Reconfirm the provider-catalog no-change audit from `specs/073-contextual-help-docs/research.md` against `assistant/catalog/model-catalog.toml`

---

## Phase 2: Foundational

**Purpose**: Define domain types, wire the new event type, create the link map file.

**Critical**: Complete before user-story implementation.

- [x] T003 [P] Define `HelpNextState` enum, `DiagnosticSeverity` enum, `HelpNextDiagnostic` struct, `HelpNextRecommendation` struct, and `HelpNextEvent` struct in `src/domain/help_next.rs`
- [x] T004 [P] Add `HelpNextRequested` variant to the `EventType` enum, its `type_name` and `schema_version` mappings, and the `HelpNextEvent` payload struct in `src/domain/observability.rs`
- [x] T005 [P] Create the versioned `.boundline/help-links.toml` link map file with all diagnostic keys from the data model and the generic `fallback` entry
- [x] T006 Wire `help_next` module into `crates/boundline-core/src/domain.rs`
- [x] T007 [P] Add focused failing domain regressions for state detection and diagnostic prioritization in `tests/unit/help_next_model.rs`

**Checkpoint**: Domain types compile, event type registered, link map committed.

---

## Phase 3: User Story 1 - Discover Next Action From Any Runtime State (Priority: P1) 🎯 MVP

**Goal**: `boundline help-next` works across all 5+ core states with human-readable and `--json` output.

- [x] T008 [US1] Implement the workspace state inspector: detect uninitialized, initialized (no session), active, blocked, failed, ready, and corrupt-session states from `.boundline/` and session model in `src/domain/help_next.rs`
- [x] T009 [US1] Implement the diagnostic collector: gather diagnostics from missing config, unregistered providers, blocked planning gates, guardian findings, stop rules, and execution failures in `src/domain/help_next.rs`
- [x] T010 [US1] Implement the link map loader: parse `.boundline/help-links.toml`, resolve diagnostic keys to URLs, fallback to generic link on missing keys in `src/domain/help_next.rs`
- [x] T011 [US1] Implement the recommendation resolver: select the highest-priority blocking issue, compute the next recommended action and command, and assemble the `HelpNextRecommendation` in `src/domain/help_next.rs`
- [x] T012 [US1] Implement the `boundline help-next` CLI command with `--json`, `--all` flags in `src/cli/help_next.rs`
- [x] T013 [US1] Implement human-readable output rendering per the output contract in `src/cli/help_next.rs`
- [x] T014 [US1] Implement `--json` output rendering per the output contract in `src/cli/help_next.rs`
- [x] T015 [US1] Implement the `boundline.help_next.requested` structured event emission hook in `src/orchestrator/session_runtime_observability.rs`
- [x] T016 [US1] Run and expand domain regressions in `tests/unit/help_next_model.rs`
- [x] T017 [P] [US1] Add output contract tests for human-readable and `--json` formats covering all 6 states in `tests/contract/help_next_output_contract.rs`
- [x] T018 [P] [US1] Add event contract tests verifying `boundline.help_next.requested` payload shape and sensitive-data exclusion in `tests/contract/help_next_event_contract.rs`
- [x] T019 [US1] Add integration tests covering the full flow from uninitialized through healthy in `tests/integration/help_next_flow.rs`

**Checkpoint**: `boundline help-next` returns correct guidance across all states.

---

## Phase 4: User Story 2 - Diagnose Missing Configuration Or Provider Readiness (Priority: P2)

**Goal**: `help-next` identifies missing config keys, unregistered/unactivated providers, and missing context packs.

- [x] T020 [US2] Implement config-key diagnostics: inspect `config.toml` for required keys, produce `DiagnosticSeverity::Warning` for missing optional keys and `::Blocking` for missing required keys in `src/domain/help_next.rs`
- [x] T021 [US2] Implement provider readiness diagnostics: check registered providers for activation state, setup requirements, and health in `src/domain/help_next.rs`
- [x] T022 [US2] Implement context-pack diagnostics: verify required context packs are present and fresh in `src/domain/help_next.rs`
- [x] T023 [US2] Expand config/provider/context-pack regression coverage in `tests/unit/help_next_model.rs`

**Checkpoint**: Configuration and provider gaps surface actionable diagnostics.

---

## Phase 5: Release, Documentation, and Quality Closure

- [x] T024 Bump workspace version if needed (determine from current `Cargo.toml`) and propagate to release metadata
- [x] T025 [P] Bump Canon companion version in `canon/Cargo.toml` and propagate Canon release metadata, then update Canon compatibility references in boundline (`tests/contract/canon_reasoning_posture_contract.rs`, version alignment contracts, CHANGELOG)
- [x] T026 [P] Update `CHANGELOG.md` with the feature entry
- [x] T027 [P] Update `README.md` with `boundline help-next` usage
- [x] T028 [P] Update `tech-docs/architecture.md` with help-next module description
- [x] T029 [P] Add runtime doc in `docs/runtime/help-next.md`
- [x] T030 Run `cargo fmt` and verify with `cargo fmt --check`
- [x] T031 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T032 Run `cargo test` and resolve any failures
- [x] T033 Generate `lcov.info` and verify ≥95% changed-file coverage
- [x] T034 Validate quickstart scenarios without running Boundline CLI against the repository root
- [x] T033 Validate quickstart scenarios without running Boundline CLI against the repository root

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001-T002 must complete before domain types.
- **Foundational → US1**: T003-T007 must complete before `help-next` logic.
- **US1 → US2**: US2 depends on the diagnostic collector from US1 (T009).
- **US1+US2 → Release**: All implementation must stabilize before Phase 5.

### Parallel Opportunities

- T002 with T001
- T003, T004, T005 can run in parallel
- T017, T018 can run in parallel
- T025-T028 can run in parallel

---

## Implementation Strategy

### MVP First (Phase 1-3)

1. Setup + Foundational → domain types and link map
2. US1 → `boundline help-next` works across 6 states with `--json`
3. Validate one healthy and one blocked scenario

### Incremental Delivery

1. US1 delivers the core help-next experience
2. US2 adds config/provider diagnostics
3. Phase 5 closes docs, versioning, and quality
