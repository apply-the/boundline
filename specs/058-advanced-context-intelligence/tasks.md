# Tasks: Advanced Context Intelligence

**Input**: Design documents from `/specs/058-advanced-context-intelligence/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are required for this Boundline feature because it changes executable context assembly, failure handling, Canon-consumer compatibility, trace output, and CLI-visible runtime summaries.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. `US1`, `US2`, `US3`)
- Include exact file paths in every task description

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Refresh shared planning evidence and prepare the repo-visible scaffolding this slice will fill.

- [ ] T001 Refresh the provider-doc audit and record the applied delta or no-change result in `assistant/catalog/model-catalog.toml` and `specs/058-advanced-context-intelligence/research.md`
- [ ] T002 [P] Create the feature module and export scaffolding in `src/lib.rs`, `crates/boundline-core/src/domain.rs`, `crates/boundline-adapters/src/lib.rs`, and `crates/boundline-cli/src/lib.rs`
- [ ] T003 [P] Create test scaffolding for context-intelligence unit, contract, and integration coverage in `tests/unit/context_intelligence_state.rs`, `tests/unit/context_intelligence_projection.rs`, `tests/unit/context_intelligence_policy.rs`, `tests/contract/context_intelligence_consumer_contract.rs`, `tests/contract/context_intelligence_projection_contract.rs`, `tests/integration/context_intelligence_flow.rs`, `tests/integration/context_intelligence_impact_flow.rs`, and `tests/integration/context_intelligence_remote_policy.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core typed models, persistence hooks, and bounded retrieval scaffolding that all user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Create shared retrieval-mode, authority-rank, query-state, relationship-kind, impact-finding, and limit models in `src/domain/context_intelligence.rs` and `crates/boundline-core/src/domain.rs`
- [ ] T005 [P] Create the workspace-local retrieval-index adapter scaffolding in `src/adapters/retrieval_index.rs` and `crates/boundline-adapters/src/lib.rs`
- [ ] T006 [P] Extend persisted session, task-context, and trace scaffolding for retrieval state in `src/domain/session.rs`, `src/domain/task_context.rs`, `src/domain/trace.rs`, `src/adapters/session_store.rs`, and `src/adapters/trace_store.rs`
- [ ] T007 Wire repository and Canon-backed evidence discovery hooks into `src/domain/project_index.rs`, `src/domain/project_memory.rs`, and `src/orchestrator/session_runtime.rs`
- [ ] T008 Implement shared stale-refresh, exhaustion, and compatibility-failure helpers in `src/domain/context_intelligence.rs`, `src/orchestrator/context_intelligence.rs`, and `src/fixture.rs`

**Checkpoint**: The retrieval models, persistence scaffolding, and bounded execution primitives required for story work are ready.

---

## Phase 3: User Story 1 - Expand Context Without Losing Authority (Priority: P1) 🎯 MVP

**Goal**: Expand structured runtime context with bounded retrieved evidence while preserving explicit authority ordering and Canon-consumer compatibility.

**Independent Test**: Run `plan`, `status`, and `inspect` in a workspace with repository signals and compatible Canon artifacts, and verify that structured inputs remain authoritative while selected evidence and rejection rationale stay inspectable.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add failing unit coverage for authority ordering, local retrieval mode, and evidence selection in `tests/unit/context_intelligence_state.rs`
- [ ] T010 [P] [US1] Add failing contract coverage for Canon-consumer compatibility and selected-evidence projection in `tests/contract/context_intelligence_consumer_contract.rs` and `tests/contract/context_intelligence_projection_contract.rs`
- [ ] T011 [P] [US1] Add failing integration coverage for local hybrid retrieval during `plan`, `status`, and `inspect` in `tests/integration/context_intelligence_flow.rs`

### Implementation for User Story 1

- [ ] T012 [US1] Implement retrieval-query planning, authority ordering, and candidate selection in `src/domain/context_intelligence.rs` and `src/orchestrator/context_intelligence.rs`
- [ ] T013 [US1] Implement local index refresh and search across repository, trace, review, verification, and Canon-backed sources in `src/adapters/retrieval_index.rs`, `src/domain/project_index.rs`, and `src/domain/project_memory.rs`
- [ ] T014 [US1] Persist retrieval queries, selected evidence, and Canon-consumer compatibility outcomes in `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/orchestrator/session_runtime.rs`
- [ ] T015 [US1] Surface retrieval mode, authority order, selected evidence, and rejection rationale in `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [ ] T016 [US1] Integrate context expansion into the primary bounded workflow in `src/cli/run.rs` and `src/orchestrator/session_runtime.rs`

**Checkpoint**: User Story 1 is independently functional as the MVP slice.

---

## Phase 4: User Story 2 - See Impact And Review Gaps Early (Priority: P1)

**Goal**: Project relationships and impact findings from retrieved evidence so operators can see affected systems, contract exposure, missing tests, and reviewer gaps before execution or review continues.

**Independent Test**: Run a bounded change against a representative workspace and verify that `status` and `inspect` show explainable relationship projections plus actionable impact findings without reading internal code.

### Tests for User Story 2

- [ ] T017 [P] [US2] Add failing unit coverage for relationship credibility, impact-finding validation, and unsupported inference handling in `tests/unit/context_intelligence_projection.rs`
- [ ] T018 [P] [US2] Add failing contract coverage for relationship and impact projection in `tests/contract/context_intelligence_projection_contract.rs`
- [ ] T019 [P] [US2] Add failing integration coverage for affected-system, missing-test, contract-exposure, and reviewer-gap journeys in `tests/integration/context_intelligence_impact_flow.rs`

### Implementation for User Story 2

- [ ] T020 [US2] Implement relationship-projection and impact-finding models plus validation in `src/domain/context_intelligence.rs` and `src/domain/trace.rs`
- [ ] T021 [US2] Implement impact-analysis orchestration over retrieved evidence in `src/orchestrator/context_intelligence.rs` and `src/orchestrator/session_runtime.rs`
- [ ] T022 [US2] Persist relationship projections and impact findings into session and trace records in `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/adapters/trace_store.rs`
- [ ] T023 [US2] Surface relationship reasoning, impact findings, and rejected inferences in `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`

**Checkpoint**: User Stories 1 and 2 both work independently with explicit impact reasoning.

---

## Phase 5: User Story 3 - Keep Retrieval Optional, Bounded, And Local-First (Priority: P2)

**Goal**: Preserve offline-friendly operation and make disabled, local, and explicit remote retrieval modes degrade or stop visibly rather than leaking into hidden provider behavior.

**Independent Test**: Run the same bounded workflow in disabled, local, and remote-permitted configurations and verify that remote use only occurs when explicitly allowed, while blocked or exhausted paths remain inspectable.

### Tests for User Story 3

- [ ] T024 [P] [US3] Add failing unit coverage for retrieval limits, stale-refresh retries, remote-policy gates, and degraded terminal states in `tests/unit/context_intelligence_state.rs` and `tests/unit/context_intelligence_policy.rs`
- [ ] T025 [P] [US3] Add failing contract coverage for retrieval mode, blocked remote transmission, and degradation projection in `tests/contract/context_intelligence_projection_contract.rs`
- [ ] T026 [P] [US3] Add failing integration coverage for disabled/local/remote behavior and stale-refresh exhaustion in `tests/integration/context_intelligence_remote_policy.rs`

### Implementation for User Story 3

- [ ] T027 [US3] Implement retrieval-mode policy, remote opt-in gating, and explicit limit enforcement in `src/domain/context_intelligence.rs`, `src/orchestrator/context_intelligence.rs`, and `src/cli/config.rs`
- [ ] T028 [US3] Implement degraded, insufficient, exhausted, and unavailable projection paths in `src/domain/trace.rs`, `src/domain/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [ ] T029 [US3] Ensure Canon incompatibility and remote-policy failures fall back or stop explicitly in `src/domain/project_memory.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/session.rs`

**Checkpoint**: All three user stories are independently functional and preserve the local-first boundary.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Close documentation, validation, and coverage across all user stories.

- [ ] T030 [P] Update operator documentation and release-facing guidance in `docs/configuration.md`, `README.md`, `CHANGELOG.md`, and `specs/058-advanced-context-intelligence/quickstart.md`
- [ ] T031 [P] Refresh contributor and assistant guidance for the new retrieval surfaces in `AGENTS.md` and `assistant/README.md`
- [ ] T032 Run focused validation for `tests/unit/context_intelligence_state.rs`, `tests/unit/context_intelligence_projection.rs`, `tests/unit/context_intelligence_policy.rs`, `tests/contract/context_intelligence_consumer_contract.rs`, `tests/contract/context_intelligence_projection_contract.rs`, `tests/integration/context_intelligence_flow.rs`, `tests/integration/context_intelligence_impact_flow.rs`, `tests/integration/context_intelligence_remote_policy.rs`, and `cargo test --no-run --all-targets`
- [ ] T033 Refresh coverage and recorded validation evidence in `lcov.info`, `assistant/catalog/model-catalog.toml`, and `specs/058-advanced-context-intelligence/research.md`
- [ ] T034 Validate the operator walkthrough and trace wording in `specs/058-advanced-context-intelligence/quickstart.md` and `specs/058-advanced-context-intelligence/contracts/advanced-context-intelligence-projection-contract.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies, can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on Foundational completion and reuses retrieval and persistence primitives stabilized by US1.
- **User Story 3 (Phase 5)**: Depends on Foundational completion and reuses retrieval-state and projection primitives from US1.
- **Final Phase**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Independent MVP after Foundational.
- **User Story 2 (P1)**: Uses the retrieval and projection primitives from US1 but remains independently testable through impact-analysis scenarios.
- **User Story 3 (P2)**: Uses the shared retrieval-state model from Foundational and US1 but remains independently testable through mode and policy scenarios.

### Within Each User Story

- Validation tasks MUST fail before implementation when the feature changes executable behavior.
- Typed domain models and contracts come before orchestration logic.
- Persistence and trace updates come before CLI summary and inspect work.
- Each story must pass its independent test before the next story is considered complete.

### Parallel Opportunities

- `T002` and `T003` can run in parallel after `T001` starts.
- `T005` and `T006` can run in parallel after `T004`.
- Validation tasks within each user story marked `[P]` can run in parallel.
- After Foundational is complete, US2 and US3 validation preparation can begin while US1 implementation stabilizes.

---

## Parallel Example: User Story 1

```bash
# Launch the failing retrieval checks together:
Task: "Add failing unit coverage for authority ordering, local retrieval mode, and evidence selection in tests/unit/context_intelligence_state.rs"
Task: "Add failing contract coverage for Canon-consumer compatibility and selected-evidence projection in tests/contract/context_intelligence_consumer_contract.rs and tests/contract/context_intelligence_projection_contract.rs"
Task: "Add failing integration coverage for local hybrid retrieval during plan, status, and inspect in tests/integration/context_intelligence_flow.rs"
```

## Parallel Example: User Story 2

```bash
# Launch the failing impact-analysis checks together:
Task: "Add failing unit coverage for relationship credibility and impact-finding validation in tests/unit/context_intelligence_projection.rs"
Task: "Add failing contract coverage for relationship and impact projection in tests/contract/context_intelligence_projection_contract.rs"
Task: "Add failing integration coverage for affected-system, missing-test, contract-exposure, and reviewer-gap journeys in tests/integration/context_intelligence_impact_flow.rs"
```

## Parallel Example: User Story 3

```bash
# Launch the retrieval-mode and policy checks together:
Task: "Add failing unit coverage for retrieval limits, stale-refresh retries, remote-policy gates, and degraded terminal states in tests/unit/context_intelligence_state.rs and tests/unit/context_intelligence_policy.rs"
Task: "Add failing contract coverage for retrieval mode, blocked remote transmission, and degradation projection in tests/contract/context_intelligence_projection_contract.rs"
Task: "Add failing integration coverage for disabled/local/remote behavior and stale-refresh exhaustion in tests/integration/context_intelligence_remote_policy.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: Verify local-first hybrid retrieval, Canon-consumer compatibility, and authority ordering independently before expanding into impact analysis or remote-mode policy.

### Incremental Delivery

1. Complete Setup + Foundational to establish typed retrieval primitives.
2. Deliver User Story 1 and validate the MVP context-expansion behavior.
3. Deliver User Story 2 and validate impact analysis without breaking US1.
4. Deliver User Story 3 and validate disabled/local/remote policy behavior without weakening the authority model.
5. Finish with the Final Phase validation, docs, and coverage refresh.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together.
2. Once Foundational is stable:
   - Developer A: User Story 1 retrieval selection and authority ordering.
   - Developer B: User Story 2 relationship and impact projection.
   - Developer C: User Story 3 retrieval-mode policy and degraded paths.
3. Merge back only after story-level validation passes independently.

---

## Notes

- [P] tasks touch different files and can proceed without waiting on another incomplete task in the same phase.
- Every user story includes explicit validation before implementation sign-off.
- Canon remains a producer-side dependency only through the existing artifact-indexing surface; there is no separate Canon implementation slice for this feature unless the producer contract itself changes.
- Keep structured runtime context authoritative over semantic expansion throughout the task list.