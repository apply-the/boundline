# Tasks: Boundline Completion Verification Runtime

**Input**: Design documents from `/specs/079-completion-verification-runtime/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Tests are required for this feature because the spec explicitly calls for unit, contract, and integration coverage across blocked, stale, failed, and passing closeout paths.

**Organization**: Tasks are grouped by user story so each story can be implemented and verified independently where the feature design allows.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete-task dependency)
- **[Story]**: User story label for story-specific work (`[US1]`, `[US2]`, ...)
- Every task includes exact file paths

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish reusable test and fixture scaffolding for completion-verification development.

- [X] T001 Create completion-verification fixture workspace assets in `tests/fixtures/completion_verification_runtime/` and shared helpers in `tests/support/completion_verification.rs`
- [X] T002 [P] Register completion-verification suites in `tests/unit.rs`, `tests/contract.rs`, and `tests/integration.rs`
- [X] T003 [P] Add assistant-asset regression scaffolding for proof-gated output in `tests/unit/assistant_assets.rs` and `assistant/codex/commands/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Introduce the typed domain and persistence surfaces that all user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Add typed completion-verification models in `src/domain/completion_verification.rs`
- [X] T005 Extend persisted task and session records with completion-verification projections in `src/domain/task.rs` and `src/domain/session.rs`
- [X] T006 [P] Extend trace state for proof refs, fingerprint refs, and evidence summaries in `src/domain/trace.rs`
- [X] T007 Implement claim-source resolution, proof-command registry, and confirmation-policy helpers in `src/domain/completion_verification.rs`
- [X] T008 Implement normalized workspace fingerprint capture and diff helpers, including deterministic claim-relevant documentation inclusion rules, in `src/domain/completion_verification.rs` and `src/orchestrator/session_runtime_observability.rs`
- [X] T009 [P] Add foundational unit coverage for typed models, malformed input handling, claim-source resolution, and fingerprint invalidation in `tests/unit/completion_verification_model.rs`, `tests/unit/completion_verification_selection.rs`, and `tests/unit/completion_verification_fingerprint.rs`

**Checkpoint**: Typed completion-verification state, fingerprint primitives, and proof-selection helpers are available for all story phases.

---

## Phase 3: User Story 1 - Block Unsafe Completion (Priority: P1) 🎯 MVP

**Goal**: Prevent task closeout when proof is missing, unsupported, interrupted, or otherwise unavailable.

**Independent Test**: Attempt closeout for a task with a claimed outcome but no valid proof path and confirm that the task stays open with a blocked or proof-required projection plus the exact next proving action.

### Tests for User Story 1

- [X] T010 [P] [US1] Add integration coverage for missing-proof, no-proof-command, and interrupted-proof blocked closeout in `tests/integration/completion_verification_task_flow.rs`
- [X] T011 [P] [US1] Add contract coverage for blocked task projections and fail-closed findings in `tests/contract/completion_verification_projection_contract.rs`

### Implementation for User Story 1

- [X] T012 [US1] Implement task closeout gating for missing proof and unsupported claim states in `src/orchestrator/session_runtime_finalization.rs`
- [X] T013 [US1] Persist blocked claims and proof-required findings on task and session records in `src/domain/task.rs` and `src/domain/session.rs`
- [X] T014 [US1] Render blocked task next actions and proof-required summaries in `src/cli/output_session_status.rs` and `src/cli/output_runtime.rs`

**Checkpoint**: User Story 1 is complete when unsafe task closeout fails closed and operators can see why the task stayed open.

---

## Phase 4: User Story 2 - Prove The Claimed Outcome (Priority: P1)

**Goal**: Derive a concrete claim, run the selected proving command against the current working state, and make stale or mismatched proof visible.

**Independent Test**: Close a task with a selected proof command and verify the runtime captures fresh evidence on success, blocks on failure, blocks on stale fingerprints, and asks for confirmation when inference is low-confidence or ambiguous.

### Tests for User Story 2

- [X] T015 [P] [US2] Add integration coverage for passing proof, failing proof, low-confidence confirmation, and metadata/runtime claim conflicts in `tests/integration/completion_verification_task_flow.rs`
- [X] T016 [P] [US2] Add integration coverage for stale proof invalidation, claim-relevant documentation inclusion or exclusion, and changed-path truncation in `tests/integration/completion_verification_stale_flow.rs`

### Implementation for User Story 2

- [X] T017 [US2] Implement proof execution, exit-summary capture, and evidence-ref attachment in `src/orchestrator/session_runtime_finalization.rs` and `src/domain/trace.rs`
- [X] T018 [US2] Implement runtime claim inference, confidence gating, and operator confirm/override flow in `src/orchestrator/session_runtime_finalization.rs` and `src/orchestrator/session_runtime_surface.rs`
- [X] T019 [US2] Implement stale-proof finding construction with fingerprint comparisons and changed-path samples in `src/domain/completion_verification.rs` and `src/orchestrator/session_runtime_observability.rs`

**Checkpoint**: User Story 2 is complete when fresh proof can unblock closeout and stale, failed, or ambiguous proof paths remain safely blocked.

---

## Phase 5: User Story 3 - Surface Verification State In Runtime Output (Priority: P2)

**Goal**: Surface additive completion-verification state consistently in status, inspect, orchestrate, and assistant-facing run/status assets.

**Independent Test**: Inspect blocked and ready closeout states and verify that status, inspect, and orchestrate projections include the new fields while assistant-facing output suppresses success language until proof is ready.

### Tests for User Story 3

- [X] T020 [P] [US3] Add contract coverage for task-scope status, inspect, and orchestrate completion-verification projections in `tests/contract/completion_verification_projection_contract.rs`
- [X] T021 [P] [US3] Add unit coverage that assistant run/status assets suppress success language while proof is missing or stale in `tests/unit/assistant_assets.rs`

### Implementation for User Story 3

- [X] T022 [US3] Extend session status rendering with completion-verification state, findings, blocked claims, and evidence refs in `src/cli/output_session_status.rs` and `src/cli/output_runtime.rs`
- [X] T023 [US3] Extend inspect projections for completion-verification summaries and stale-proof details in `src/cli/inspect.rs` and `src/cli/inspect/projections.rs`
- [X] T024 [US3] Update assistant run/status command assets to surface blocked claims and proof commands in `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, and `assistant/copilot/prompts/boundline-status.prompt.md`

**Checkpoint**: User Story 3 is complete when all primary closeout surfaces expose the same additive verification state and no pre-proof success language leaks through.

---

## Phase 6: User Story 4 - Preserve Canon Boundary While Emitting Runtime Evidence (Priority: P3)

**Goal**: Emit the runtime-owned `claim -> proof -> evidence_ref` projection without requiring Canon packet generation or readiness logic.

**Independent Test**: Run closeout without Canon packet generation and verify that Boundline still blocks or completes correctly while exposing claim/proof/evidence records that Canon can consume later.

### Tests for User Story 4

- [X] T025 [P] [US4] Add contract coverage that claim/proof/evidence projections exclude Canon-owned readiness fields in `tests/contract/completion_verification_projection_contract.rs`
- [X] T026 [P] [US4] Add integration coverage that closeout works without Canon packet generation in `tests/integration/completion_verification_task_flow.rs`

### Implementation for User Story 4

- [X] T027 [US4] Persist Canon-independent claim/proof/evidence projection records in `src/domain/session.rs` and `src/domain/trace.rs`
- [X] T028 [US4] Surface Canon-boundary explanations in runtime closeout projections in `src/orchestrator/session_runtime_surface.rs` and `src/cli/output_orchestrate.rs`

**Checkpoint**: User Story 4 is complete when Boundline remains independently testable and Canon-facing evidence remains additive only.

---

## Phase 7: User Story 5 - Aggregate Stage And Run Verification (Priority: P3)

**Goal**: Aggregate required child verification state for stage and run closeout without replacing task-level proof ownership.

**Independent Test**: Attempt stage or run closeout with a mix of ready, stale, missing-proof, and deferred children and verify that parent closeout stays blocked until all required children are verification-ready.

### Tests for User Story 5

- [X] T029 [P] [US5] Add contract coverage for stage/run child aggregation projections in `tests/contract/completion_verification_parent_scope_contract.rs`
- [X] T030 [P] [US5] Add integration coverage for blocked child aggregation, deferred children, and explicit parent claims in `tests/integration/completion_verification_stage_run_flow.rs`

### Implementation for User Story 5

- [X] T031 [US5] Implement child verification summary models and parent-scope counters in `src/domain/completion_verification.rs` and `src/domain/session.rs`
- [X] T032 [US5] Implement stage and run closeout aggregation gates in `src/orchestrator/session_runtime_finalization.rs` and `src/orchestrator/session_runtime_surface.rs`
- [X] T033 [US5] Render parent-scope blocked-child findings in `src/cli/output_session_status.rs`, `src/cli/output_orchestrate.rs`, and `src/cli/inspect/projections.rs`

**Checkpoint**: User Story 5 is complete when parent closeout never hides required child verification failures and explicit parent claims remain additive.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Finish cross-story validation and implementation hygiene before release tasks.

- [X] T034 [P] Validate the documented operator scenarios in `specs/079-completion-verification-runtime/quickstart.md` against `tests/integration/completion_verification_task_flow.rs`, `tests/integration/completion_verification_stale_flow.rs`, and `tests/integration/completion_verification_stage_run_flow.rs`
- [ ] T035 [P] Review duplication across `src/domain/completion_verification.rs`, `src/orchestrator/session_runtime_finalization.rs`, and `src/cli/output_runtime.rs` and extract shared helpers where practical

---

## Final Phase: Release, Quality, And Verification

- [X] T036 Update the package version from `0.77.0` to `0.78.0` in `Cargo.toml`
- [X] T037 Update release-facing behavior notes in `README.md` and `CHANGELOG.md`
- [X] T038 Reconcile legacy roadmap/spec source-of-truth references that still point to feature 18 or other outdated completion-verification seeds in `roadmap/features/README.md`, `roadmap/Next - forward-roadmap.md`, `roadmap/joint-roadmap-graph.md`, `roadmap/features/22-session-memory-and-repository-knowledge-distillation.md`, and `specs/079-completion-verification-runtime/feat-completion-verification-runtime.md` (inspect `roadmap/features/README.md` before deleting or rewriting any seed reference)
- [X] T039 Update completion-verification documentation in `docs/reference/cli.md`, `docs/runtime/trace.md`, and `tech-docs/project-memory-and-evidence-structure.md`
- [ ] T040 Run `./scripts/update-docs-versions.sh`
- [ ] T041 Run `./scripts/sync-distribution-metadata.sh`
- [ ] T042 Run `cargo fmt`
- [ ] T043 Run `scripts/clippy.sh` and fix all warnings
- [ ] T044 Run `scripts/test.sh` and fix failing tests
- [ ] T045 Run `scripts/coverage.sh` and confirm at least 95% coverage for every modified or created Rust file
- [ ] T046 Run `scripts/check-no-local-paths.sh`
- [ ] T047 Run `scripts/check-rust-no-panic.sh`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies; can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user story work.
- **Phase 3: US1**: Depends on Phase 2.
- **Phase 4: US2**: Depends on Phase 2 and builds directly on US1 closeout gating.
- **Phase 5: US3**: Depends on US1 and US2 because it renders their runtime state.
- **Phase 6: US4**: Depends on US2 because the Canon boundary is expressed through the claim/proof/evidence projection.
- **Phase 7: US5**: Depends on US1, US2, and US3 because parent aggregation reuses child verification state and rendered findings.
- **Phase 8: Polish**: Depends on all implemented user stories.
- **Final Phase**: Depends on all story and polish work being complete.

### User Story Dependencies

- **US1 (P1)**: First MVP slice; no dependency on later stories.
- **US2 (P1)**: Extends US1 by making proof execution and freshness real.
- **US3 (P2)**: Depends on US1 and US2 runtime data.
- **US4 (P3)**: Depends on the US2 projection model but remains independently testable once proof execution exists.
- **US5 (P3)**: Depends on US1-US3 to aggregate child state and render parent-scope findings.

### Within Each User Story

- Tests should be written before implementation in that story and must fail first where practical.
- Domain model changes precede orchestrator closeout logic.
- Orchestrator logic precedes CLI rendering.
- Rendering changes precede release-doc updates that describe the new behavior.

### Parallel Opportunities

- `T002`, `T003`, `T004`, `T006`, and `T009` can run in parallel during setup/foundations.
- US1 test tasks `T010` and `T011` can run in parallel.
- US2 test tasks `T015` and `T016` can run in parallel.
- US3 test tasks `T020` and `T021` can run in parallel.
- US4 test tasks `T025` and `T026` can run in parallel.
- US5 test tasks `T029` and `T030` can run in parallel.
- `T034` and `T035` can run in parallel after story work is complete.

---

## Parallel Example: User Story 2

```bash
# Launch the proof-execution and stale-proof integration checks together:
Task: "Add integration coverage for passing proof, failing proof, low-confidence confirmation, and metadata/runtime claim conflicts in tests/integration/completion_verification_task_flow.rs"
Task: "Add integration coverage for stale proof invalidation and changed-path truncation in tests/integration/completion_verification_stale_flow.rs"
```

---

## Implementation Strategy

### MVP First

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1
4. Complete Phase 4: US2
5. Validate blocked and passing task closeout before moving on

### Incremental Delivery

1. Ship the task-scope proof gate first (`US1` + `US2`)
2. Add operator-visible projections and assistant asset alignment (`US3`)
3. Add Canon-boundary persistence guarantees (`US4`)
4. Finish with parent-scope aggregation (`US5`)

### Parallel Team Strategy

1. One developer handles domain and persistence foundations (`T004`-`T008`)
2. One developer prepares tests and fixtures (`T001`-`T003`, `T009`-`T011`)
3. After Phase 2, story work can split by scope:
   - Developer A: task closeout and proof execution (`US1`, `US2`)
   - Developer B: output surfaces and assistant assets (`US3`)
   - Developer C: parent aggregation and Canon-boundary follow-through (`US4`, `US5`)

---

## Notes

- Every behavior-changing story includes corresponding test tasks.
- Runtime closeout projections are covered in status, inspect, orchestrate, and trace-oriented tasks where applicable.
- No Canon runtime changes are included; Canon remains an external consumer of emitted evidence refs only.
- Final verification must hold the release bar: tests green, coverage at or above 95% for modified or created Rust files, clippy clean, formatting applied, docs versions synchronized, and roadmap/spec duplication resolved.
