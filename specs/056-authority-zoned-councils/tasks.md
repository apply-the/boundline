---
description: "Task list for authority-zoned delivery councils implementation"
---

# Tasks: Authority-Zoned Delivery Councils

**Input**: Design documents from `/specs/056-authority-zoned-councils/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md, contracts/

**Validation**: Validation tasks are mandatory because this slice changes
governance resolution, review persistence, session traces, Canon consumer
compatibility, and operator-visible CLI surfaces.

**Organization**: Tasks are grouped by user story so each slice can be
implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story or closeout group this task belongs to (`US1`, `US2`, `US3`, `Closeout`)
- Include exact file paths in descriptions

## Phase 0: Release & Catalog Baseline

**Purpose**: Establish the release move and provider-doc baseline for the branch

- [ ] T001 Bump the Boundline workspace version from `0.55.0` to `0.56.0` in `Cargo.toml` and update `CHANGELOG.md`
- [ ] T002 [P] Re-check current OpenAI, Anthropic, and Google provider docs against `assistant/catalog/model-catalog.toml` and record the explicit no-change result in `specs/056-authority-zoned-councils/research.md`
- [ ] T003 [P] Align `specs/056-authority-zoned-councils/contracts/authority-governance-consumer-contract.md`, `specs/056-authority-zoned-councils/contracts/council-projection-contract.md`, and `specs/056-authority-zoned-councils/quickstart.md` with the released Canon `authority-governance-v1` surface

---

## Phase 1: Foundational (Blocking Prerequisites)

**Purpose**: Shared authority-resolution primitives, Canon compatibility gates, and persisted review state

**⚠️ CRITICAL**: No user story work should begin until this phase is complete

- [ ] T004 Extend shared authority-control, council-profile, and stop-semantics primitives in `src/domain/governance.rs` and `crates/boundline-core/src/domain.rs`
- [ ] T005 [P] Extend Canon governance adapter consumption and compatibility gates in `src/adapters/governance_runtime.rs` and `tests/unit/governance_runtime.rs`
- [ ] T029 [P] Add explicit local-governance fallback coverage for sessions where Canon input is absent and governance is not required in `src/adapters/governance_runtime.rs`, `tests/unit/governance_runtime.rs`, and `tests/integration/session_governance_flow.rs`
- [ ] T006 [P] Extend persisted review, producer-response, and adjudication state in `src/domain/review.rs`, `src/domain/session.rs`, and `src/orchestrator/review_trace.rs`
- [ ] T007 [P] Reuse governance test entry points in `tests/unit/governance_policy.rs`, `tests/unit/review_voting.rs`, `tests/contract/governance_session_contract.rs`, `tests/contract/governance_trace_contract.rs`, and `tests/integration/session_governance_flow.rs`

**Checkpoint**: Shared authority and council primitives are ready

---

## Phase 2: User Story 1 - Resolve Control Before Acceptance (Priority: P1) 🎯 MVP

**Goal**: Consume Canon authority inputs, resolve one explicit control class, and map it to one bounded council profile before progression continues

**Independent Test**: A governed stage with compatible Canon `authority-governance-v1` metadata produces a deterministic control class, council profile, and explicit proceed or stop posture

### Validation for User Story 1

- [ ] T008 [P] [US1] Add required-field, unsupported-contract, and optional-provenance coverage in `src/adapters/governance_runtime.rs` and `tests/unit/governance_runtime.rs`
- [ ] T009 [P] [US1] Add effective-control and council-profile resolution coverage in `src/domain/governance.rs` and `tests/unit/governance_policy.rs`
- [ ] T030 [P] [US1] Add runtime-role and domain-expert selection coverage in `src/domain/governance.rs`, `tests/unit/governance_policy.rs`, and `tests/integration/governance_autopilot_flow.rs`

### Implementation for User Story 1

- [x] T010 [P] [US1] Implement Canon `authority-governance-v1` consumer parsing and fail-closed compatibility rules in `src/adapters/governance_runtime.rs`
- [x] T011 [US1] Implement effective control-class and bounded council-profile resolution in `src/domain/governance.rs`
- [ ] T031 [US1] Implement inspectable runtime-role and domain-expert selection with local rationale capture in `src/domain/governance.rs`, `src/orchestrator/governance.rs`, and `src/domain/session.rs`
- [ ] T012 [US1] Persist resolved governance outcomes in `src/domain/session.rs` and register shared projection types through `crates/boundline-core/src/domain.rs`
- [ ] T013 [US1] Align the MVP consumer wording in `specs/056-authority-zoned-councils/data-model.md` and `specs/056-authority-zoned-councils/contracts/authority-governance-consumer-contract.md`

**Checkpoint**: User Story 1 is independently valid as the authority-resolution MVP

---

## Phase 3: User Story 2 - Make Findings Operational (Priority: P2)

**Goal**: Persist council findings, require producer responses, and project adjudication or blocked outcomes through the same session story

**Independent Test**: Concern and blocking findings survive in session state, require explicit producer responses, and produce remediation or stop outcomes without detaching from the active session

### Validation for User Story 2

- [ ] T014 [P] [US2] Add finding, producer-response, and adjudication coverage in `src/domain/review.rs` and `tests/unit/review_voting.rs`
- [ ] T015 [P] [US2] Add blocked and adjudication flow coverage in `tests/integration/session_governance_flow.rs` and `tests/integration/workflow_follow_through.rs`

### Implementation for User Story 2

- [ ] T016 [P] [US2] Extend structured finding and producer-response state in `src/domain/review.rs`
- [ ] T017 [US2] Persist adjudication, remediation, and blocked-stop results in `src/domain/session.rs` and `src/orchestrator/review_trace.rs`
- [ ] T018 [US2] Project findings, producer responses, adjudication summaries, and stop semantics through `src/cli/output.rs` and `src/cli/session.rs`

**Checkpoint**: User Stories 1 and 2 both work independently and preserve the same session-native story

---

## Phase 4: User Story 3 - Preserve The Canon And Boundline Boundary (Priority: P3)

**Goal**: Keep Canon inputs strictly semantic and advisory while Boundline retains local control of runtime roles, council composition, and stop behavior

**Independent Test**: Optional Canon provenance and `stage_role_hints` can be surfaced and reasoned about without becoming executable runtime directives, and reviewer-independence failures still stop locally

### Validation for User Story 3

- [ ] T019 [P] [US3] Add advisory-only provenance and `stage_role_hints` coverage in `src/adapters/governance_runtime.rs` and `tests/contract/governance_trace_contract.rs`
- [ ] T020 [P] [US3] Add reviewer-independence and mandatory-role failure coverage in `tests/integration/governance_autopilot_flow.rs` and `tests/integration/governed_stage_depth_workflow.rs`

### Implementation for User Story 3

- [x] T021 [P] [US3] Keep optional Canon provenance separated from required control inputs in `src/adapters/governance_runtime.rs` and `src/domain/governance.rs`
- [ ] T022 [US3] Enforce reviewer independence, mandatory-role failures, and restricted-gate projection in `src/domain/governance.rs` and `src/orchestrator/governance.rs`
- [ ] T032 [US3] Keep councils, deterministic validation, security scanning, and human approval as separate inspectable controls in `src/orchestrator/governance.rs`, `src/cli/output.rs`, `src/cli/session.rs`, and `tests/integration/workflow_follow_through.rs`
- [ ] T023 [US3] Refresh `specs/056-authority-zoned-councils/quickstart.md` and `specs/056-authority-zoned-councils/contracts/council-projection-contract.md` to match the implemented consumer boundary

**Checkpoint**: All user stories are independently functional and preserve the Canon/Boundline ownership boundary

---

## Phase 5: Verification & Closeout

**Purpose**: Finish docs, formatting, lint, tests, and coverage

- [ ] T024 [P] Update operator-facing docs in `README.md`, `ROADMAP.md`, `docs/authority-zones-and-stop-semantics.md`, `docs/council-adoption-guide.md`, and `docs/review-council-algorithms.md`
- [ ] T025 Run `cargo fmt --all` in `repo root`
- [ ] T026 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in `repo root`
- [ ] T027 Run `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features` in `repo root`
- [ ] T028 Run focused modified-file coverage in `repo root` and confirm at least 95% coverage for every modified file

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 0** starts immediately and includes the required first-task version bump
- **Phase 1** depends on Phase 0 and blocks story work
- **Phases 2-4** depend on Phase 1 and should proceed in priority order
- **Phase 5** depends on all desired user stories

### User Story Dependencies

- **US1** delivers the authority-resolution MVP and depends only on the foundational consumer and projection primitives
- **US2** builds on US1 resolution state and adds findings, responses, and adjudication behavior
- **US3** builds on US1 and US2 and hardens the Canon/Boundline ownership boundary plus reviewer-independence handling

### Parallel Opportunities

- T002 and T003 can run in parallel once release intent is fixed
- T005, T006, and T007 can run in parallel after T004 starts
- Validation tasks marked [P] can run in parallel within each story
- T024 can run in parallel with final validation once behavior stabilizes

## Notes

- The first task is the Boundline version bump, as requested
- The final task is modified-file coverage verification at 95% or higher, as requested
- Canon authority semantics remain optional when governance is not required, but required Canon inputs fail closed when governance is required
- Existing governance, review, session, and trace surfaces are reused intentionally instead of introducing a second review runtime