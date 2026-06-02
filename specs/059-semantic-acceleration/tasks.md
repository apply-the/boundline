# Tasks: Advanced Context Intelligence Semantic Acceleration

**Input**: Design documents from `/specs/059-semantic-acceleration/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Validation is mandatory because this slice changes advanced-context
runtime behavior, fallback semantics, config resolution, Canon-consumer
compatibility, and CLI-visible planning and inspection output. Include focused
unit, contract, integration, compile, and coverage tasks for each user story.
Every implementation pass must also preserve the provider-catalog no-change or
delta audit against `assistant/catalog/model-catalog.toml`.

**Organization**: Tasks are grouped by user story so each slice can deliver
bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`US1`, `US2`, `US3`)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm design inputs and prepare the optional semantic-acceleration
integration surface.

- [x] T001 Record the provider-catalog refresh result for S5.v2 in `specs/059-semantic-acceleration/research.md` and `assistant/catalog/model-catalog.toml`
- [x] T002 Add optional semantic-acceleration dependency scaffolding in `Cargo.toml` and `crates/boundline-adapters/Cargo.toml`
- [x] T003 [P] Create semantic-acceleration test scaffolding in `tests/contract/context_intelligence_semantic_projection_contract.rs`, `tests/integration/context_intelligence_semantic_flow.rs`, and `tests/integration/context_intelligence_semantic_fallback.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core semantic policy, persistence, and index primitives that MUST
exist before any user story work can complete.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 Extend typed semantic policy, hybrid outcome, and chunk-state models in `src/domain/context_intelligence.rs`
- [x] T005 [P] Introduce the dedicated `semantic_acceleration` config surface plus the `boundline config set-semantic-acceleration` mutation path in `src/domain/configuration.rs`, `src/cli/config.rs`, and `tech-docs/configuration.md`
- [x] T006 [P] Add shared persistence slots for semantic projection state in `src/domain/goal_plan.rs`, `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/domain/trace.rs`
- [x] T007 Implement semantic index manifest, vector-capability detection, and shared refresh helpers in `src/orchestrator/context_intelligence.rs`
- [x] T008 [P] Add foundational semantic-model validation coverage in `tests/unit/context_intelligence_state.rs`, `tests/unit/session_cli_runtime.rs`, and `tests/unit/cluster_config_resolution.rs`

**Checkpoint**: Foundation ready. Semantic policy resolution, persistence, and
index lifecycle are stable enough for user-story work.

---

## Phase 3: User Story 1 - Recover Relevant Evidence Beyond Keywords (Priority: P1) 🎯 MVP

**Goal**: Add an optional local semantic accelerator that can expand or rerank
V1 evidence without changing authority order or blocking the V1 fallback path.

**Independent Test**: Enable local semantic acceleration for a workspace where
relevant evidence has weak lexical overlap, run `plan`, and verify that the
runtime either selects semantically relevant evidence or explicitly falls back
to the V1 path.

### Tests for User Story 1

- [x] T009 [P] [US1] Write contract coverage for hybrid retrieval and V1 fallback in `tests/contract/context_intelligence_semantic_projection_contract.rs`
- [x] T010 [P] [US1] Write integration coverage for local semantic expansion in `tests/integration/context_intelligence_semantic_flow.rs`
- [x] T011 [P] [US1] Write integration coverage for explicit semantic fallback in `tests/integration/context_intelligence_semantic_fallback.rs`

### Implementation for User Story 1

- [x] T012 [US1] Implement semantic chunk refresh and local embedding eligibility on the shared retrieval index in `src/orchestrator/context_intelligence.rs` and `src/domain/context_intelligence.rs`
- [x] T013 [US1] Implement hybrid expansion and rerank selection over the V1 candidate set in `src/orchestrator/context_intelligence.rs` and `src/orchestrator/goal_planner.rs`
- [x] T014 [US1] Persist semantic policy, capability state, and hybrid outcome in `src/domain/goal_plan.rs`, `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/orchestrator/session_runtime.rs`
- [x] T015 [US1] Surface baseline-only versus semantic-selected outcomes in `src/cli/session.rs` and `src/cli/output.rs`

**Checkpoint**: User Story 1 should now deliver local semantic retrieval as an
additive capability while preserving explicit V1 fallback behavior.

---

## Phase 4: User Story 2 - Explain Hybrid Ranking And Rejection (Priority: P1)

**Goal**: Make semantic expansion, reranking, downgrade, rejection, and fallback
behavior inspectable on normal runtime surfaces.

**Independent Test**: Run a bounded task that produces both lexical and semantic
candidates, then verify that `status` and `inspect` explain which candidates
were expanded, reranked, downgraded, rejected, or skipped.

### Tests for User Story 2

- [x] T016 [P] [US2] Write unit coverage for semantic decision annotations and rejected candidates in `tests/unit/context_intelligence_projection.rs` and `tests/unit/cli_output.rs`
- [x] T017 [P] [US2] Write contract coverage for inspect and trace semantic fields in `tests/contract/context_intelligence_semantic_inspect_contract.rs`
- [x] T018 [P] [US2] Write integration coverage for status and inspect explanation output in `tests/integration/context_intelligence_semantic_inspect.rs`

### Implementation for User Story 2

- [x] T019 [US2] Extend advanced-context projection types with match origin, score, and fallback metadata in `src/domain/context_intelligence.rs` and `src/domain/trace.rs`
- [x] T020 [US2] Record semantic expansion, rerank, downgrade, rejection, and fallback trace events in `src/orchestrator/context_intelligence.rs` and `src/orchestrator/goal_planner.rs`
- [x] T021 [US2] Render semantic reasoning and fallback details in `src/cli/inspect.rs`, `src/cli/output.rs`, and `src/cli/session.rs`
- [x] T022 [US2] Keep degraded and exhausted semantic outcomes visible in plan payloads and inspection summaries in `src/domain/goal_plan.rs` and `src/cli/inspect.rs`

**Checkpoint**: User Story 2 should make hybrid retrieval behavior explainable
without leaving the normal CLI and trace surfaces.

---

## Phase 5: User Story 3 - Respect Canon And Workspace Boundaries (Priority: P2)

**Goal**: Consume Canon semantic metadata only through the documented producer
contract and make workspace policy plus compatibility outcomes explicit.

**Independent Test**: Run the same bounded task with compatible and incompatible
Canon semantic artifacts and verify that Boundline preserves provenance for
supported artifacts, skips unsupported ones with explicit reasons, and keeps
compatibility routes labeled as secondary.

### Tests for User Story 3

- [x] T023 [P] [US3] Write contract coverage for Canon semantic consumer compatibility in `tests/contract/context_intelligence_canon_semantic_contract.rs`
- [x] T024 [P] [US3] Write unit coverage for semantic policy precedence and Canon skip reasons in `tests/unit/cluster_config_resolution.rs` and `tests/unit/session_cli_runtime.rs`
- [x] T025 [P] [US3] Write integration coverage for compatible and incompatible Canon semantic artifacts in `tests/integration/context_intelligence_canon_semantic_flow.rs`

### Implementation for User Story 3

- [x] T026 [US3] Implement semantic-acceleration precedence, workspace validation, and Canon-aware policy enforcement in `src/domain/configuration.rs`, `src/orchestrator/goal_planner.rs`, and `src/cli/config.rs`
- [x] T027 [US3] Implement Canon semantic descriptor parsing and compatibility rejection handling in `src/orchestrator/context_intelligence.rs` and `src/domain/project_memory.rs`
- [x] T028 [US3] Preserve Canon semantic contract line, provenance, and compatibility-route labeling in `src/domain/context_intelligence.rs`, `src/orchestrator/goal_planner.rs`, and `src/cli/inspect.rs`

**Checkpoint**: User Story 3 should keep Canon producer boundaries and workspace
policy explicit while preserving the primary session-native workflow.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Finish operator guidance, validation, and closeout for all stories.

- [x] T029 [P] Update operator and contributor guidance for semantic acceleration in `tech-docs/configuration.md`, `README.md`, and `assistant/README.md`
- [x] T030 [P] Refresh repository agent guidance and release notes for semantic acceleration in `AGENTS.md` and `CHANGELOG.md`
- [x] T031 [P] Create the semantic recall evaluation corpus and threshold harness in `tests/integration/context_intelligence_semantic_recall.rs` and `tests/fixtures/context_intelligence_semantic_eval/`
- [x] T032 Run the semantic-acceleration quickstart walkthrough and capture any follow-up notes in `specs/059-semantic-acceleration/quickstart.md` and `specs/059-semantic-acceleration/checklists/requirements.md`
- [x] T033 Realign Canon consumer references from the feature-local semantic contract to the promoted stable Canon contract in `specs/059-semantic-acceleration/contracts/canon-semantic-acceleration-consumer-contract.md` and `tests/contract/context_intelligence_canon_semantic_contract.rs`
- [x] T034 Bump the Boundline workspace version and record release notes in `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md`
- [x] T035 Remove completed S5 roadmap specs from `roadmap/S5 - advanced-context-intelligence.md`, `roadmap/S5.addendum - advanced-context-intelligence-technology-evaluation.md`, and `roadmap/S5.v2 - advanced-context-intelligence-semantic-acceleration.md`, then update `ROADMAP.md`
- [x] T036 Run format, lint, compile, focused integration, semantic-recall threshold evaluation, and coverage refresh for touched files and record outputs in `lcov.info` and `specs/059-semantic-acceleration/checklists/requirements.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all user stories.
- **User Stories (Phases 3-5)**: Depend on Foundational completion.
- **Polish (Final Phase)**: Depends on the desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Foundational and is the MVP.
- **User Story 2 (P1)**: Starts after Foundational and builds on the semantic evidence surfaced by US1.
- **User Story 3 (P2)**: Starts after Foundational and should integrate with US1/US2 without breaking their independent validation.

### Within Each User Story

- Validation tasks MUST fail before implementation when the behavior is executable.
- Domain and policy models before orchestration logic.
- Orchestration before CLI or inspection surfaces.
- Canon compatibility handling before final story sign-off.
- Story completion requires visible fallback or failure behavior where the spec calls for it.

### Parallel Opportunities

- Setup and Foundational tasks marked `[P]` can run in parallel.
- Validation tasks within a user story marked `[P]` can run in parallel.
- After Foundational completion, different user stories can be worked on in parallel if shared advanced-context models remain stable.

---

## Parallel Example: User Story 1

```bash
# Launch semantic validation coverage together:
Task: "Write contract coverage for hybrid retrieval and V1 fallback in tests/contract/context_intelligence_semantic_projection_contract.rs"
Task: "Write integration coverage for local semantic expansion in tests/integration/context_intelligence_semantic_flow.rs"
Task: "Write integration coverage for explicit semantic fallback in tests/integration/context_intelligence_semantic_fallback.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate semantic expansion plus explicit fallback before proceeding.

### Incremental Delivery

1. Deliver US1 for local semantic retrieval over the V1 baseline.
2. Add US2 for explainable hybrid reasoning on operator surfaces.
3. Add US3 for Canon compatibility and workspace-policy enforcement.
4. Finish with documentation, validation, and coverage closeout.

### Parallel Team Strategy

1. One contributor completes semantic policy and index foundations.
2. A second contributor prepares contract and integration coverage.
3. Once the foundation stabilizes, story work can split across US2 explanation surfaces and US3 Canon compatibility.

---

## Notes

- `[P]` tasks touch different files and do not depend on unfinished work.
- `[US#]` labels keep traceability back to the feature spec.
- Keep S5 V1 correct and inspectable at every stage.
- Do not introduce hidden fallback or remote semantic behavior in this slice.
