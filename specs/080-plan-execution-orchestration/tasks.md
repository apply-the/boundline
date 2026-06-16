# Tasks: Plan Execution Orchestration

**Input**: Design documents from `specs/080-plan-execution-orchestration/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/execution-orchestration-projection.md, quickstart.md

**Tests**: Tests are required per the feature spec — unit, contract, and integration coverage for dependency ordering, checkpointing, blocked-state handling, resume, and status projection.

**Organization**: Tasks grouped by user story so each story can be implemented and verified independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (`US1`, `US2`, `US3`)
- Every task includes exact file paths

---

## Phase 1: Setup — Shared Infrastructure

**Purpose**: Create the module scaffolding, typed models, and test fixtures that all user stories depend on.

- [X] T001 Create `src/domain/execution_orchestration.rs` with typed models per data-model.md: `ExecutionRun`, `ExecutionCheckpoint`, `TaskDependencyGraph`, `ExecutionPlanState`, `BlockedTaskRecord`, `TerminalOutcome`
- [X] T002 [P] Extend `PlannedTask` in `src/domain/goal_plan.rs` with `depends_on: Option<Vec<String>>` field (serde default, skip_serializing_if None)
- [X] T003 [P] Extend `ActiveSessionRecord` in `src/domain/session.rs` with `active_execution_run_id: Option<String>` (serde default)
- [X] T004 [P] Extend `SessionStatusView` in `src/domain/session.rs` with additive execution projection fields per contracts/execution-orchestration-projection.md: `execution_run_id`, `execution_plan_state`, `execution_current_task_id`, `execution_next_task_id`, `execution_completed_task_count`, `execution_blocked_task_ids`, `execution_checkpoint_ref`, `execution_resume_command`
- [X] T005 Create `src/orchestrator/execution_orchestrator.rs` with module scaffolding and `ExecutionOrchestrator` struct
- [X] T006 [P] Register test entry points in `tests/unit/execution_orchestrator.rs`, `tests/contract/execution_orchestration_contract.rs`, and `tests/integration/execution_orchestration_flow.rs`

**Depends on**: Nothing (parallel)

**Verification**: `cargo check -p boundline-core`

---

## Phase 2: Foundational — Dependency Graph & Checkpoint I/O

**Purpose**: Task dependency validation and atomic checkpoint persistence — blocking prerequisites for all user stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T007 Implement Kahn's algorithm for topological sort with cycle detection in `src/domain/execution_orchestration.rs`
- [X] T008 Implement dependency graph validation: self-dependencies, missing references, duplicate normalization in `src/domain/execution_orchestration.rs`
- [X] T009 Implement `TaskDependencyGraph::from_plan(plan: &GoalPlan) -> Result<Self, ValidationError>` in `src/domain/execution_orchestration.rs`
- [X] T010 Implement atomic checkpoint write (temp file → flush → sync_all → rename) in `src/orchestrator/execution_orchestrator.rs`
- [X] T011 Implement checkpoint read and schema validation in `src/orchestrator/execution_orchestrator.rs`
- [X] T012 [P] Add unit tests for dependency graph: valid linear chain, valid DAG, cycle detection, self-dependency, missing reference, duplicate normalization, plan-order tie-breaking when multiple tasks are simultaneously runnable in `tests/unit/execution_orchestrator.rs`
- [X] T013 [P] Add unit tests for checkpoint: write, atomicity (previous preserved on failure), schema validation, read, incompatible plan fingerprint rejection in `tests/unit/execution_orchestrator.rs`

**Checkpoint**: Dependency graphs validate correctly; checkpoints persist and reload atomically.

---

## Phase 3: User Story 1 — Execute Plans One Task At A Time (Priority: P1) 🎯 MVP

**Goal**: `boundline run --plan <ref>` loads an accepted plan, builds the dependency graph, and executes tasks sequentially with checkpointing after each terminal outcome.

**Independent Test**: Load a validated three-task plan, start execution, verify tasks execute in dependency order, checkpoints are written after each task, and execution can resume from the last checkpoint.

### Implementation for User Story 1

- [X] T014 [US1] Implement `ExecutionOrchestrator::start(plan: &GoalPlan, workspace: &Path) -> Result<ExecutionRun>` in `src/orchestrator/execution_orchestrator.rs`
- [X] T015 [US1] Implement `ExecutionOrchestrator::advance(&mut self) -> Result<TerminalOutcome>` — select next runnable task, lock mutation surface, dispatch via existing `SessionRuntime`, checkpoint outcome in `src/orchestrator/execution_orchestrator.rs`
- [X] T016 [US1] Implement `ExecutionOrchestrator::resume(run_id: &str, workspace: &Path) -> Result<ExecutionRun>` — reload checkpoint, validate plan identity, continue from next uncompleted task in `src/orchestrator/execution_orchestrator.rs`
- [X] T017 [US1] Integrate completion-verification gate: call `block_completion_closeout` from spec 079 before marking task complete; pause with `verification_pending` when 079 is unavailable in `src/orchestrator/execution_orchestrator.rs`
- [X] T018 [US1] Extend `boundline run` with `--plan <ref>` flag in `src/cli.rs` — parse plan file, create `ExecutionOrchestrator`, start execution
- [X] T019 [US1] Extend `boundline run` with `--accepted-plan` flag in `src/cli.rs` — resolve session-attached plan, start execution
- [X] T020 [US1] Extend `boundline run` with `--resume <id>` flag in `src/cli.rs` — reload checkpoint, resume execution
- [X] T021 [US1] Guard `boundline run` (no plan flags) to preserve existing single-task behavior unchanged

### Tests for User Story 1

- [X] T022 [P] [US1] Add unit tests for executor: start with valid plan, start with empty plan, start with cyclic plan (blocked at plan-load by T008/T009), advance through three-task chain, resume from checkpoint in `tests/unit/execution_orchestrator.rs`
- [X] T023 [P] [US1] Add contract tests for CLI: `boundline run --plan <valid-ref>` (succeeds), `--plan <cyclic-ref>` (blocks), `--plan <missing-ref>` (errors), `--plan` on already-running run (returns state), `--resume` after interruption in `tests/contract/execution_orchestration_contract.rs`
- [X] T024 [US1] Add integration test: full three-task plan execution from CLI, verify dependency order, checkpoints, and terminal state in `tests/integration/execution_orchestration_flow.rs`

**Checkpoint**: `boundline run --plan <ref>` executes tasks sequentially with checkpointing and resume.

---

## Phase 4: User Story 2 — Handle Blocked Tasks (Priority: P2)

**Goal**: When a task is blocked (proof failed, verification unavailable, governance pending), downstream execution halts, the blocked state is checkpointed, and the resume command is projected.

**Independent Test**: Execute a plan where the second task hits a blocking finding. Verify the first task's checkpoint is preserved, the blocked task is projected with its stop reason, downstream tasks remain unstarted, and resume restores the blocked state.

### Implementation for User Story 2

- [X] T025 [US2] Implement blocked-task detection: hook into `block_completion_closeout` result, classify blocking reason (`completion_proof_failed`, `verification_unavailable`, `governance_blocked`) in `src/orchestrator/execution_orchestrator.rs`
- [X] T026 [US2] Implement `ExecutionOrchestrator::pause_reason(&self) -> Option<String>` — derive human-readable stop reason from blocked task state
- [X] T027 [US2] Implement downstream halt: when a task is blocked, mark all `depends_on` dependents as not-runnable in the dependency graph state in `src/orchestrator/execution_orchestrator.rs`
- [X] T028 [US2] Implement blocked-task resolution: when operator resumes, re-evaluate the blocked task through the completion-verification path before unblocking dependents in `src/orchestrator/execution_orchestrator.rs`
- [X] T029 [US2] Implement `last_terminal_outcome` recording in checkpoint: track which task triggered the checkpoint and why

### Tests for User Story 2

- [X] T030 [P] [US2] Add unit tests for blocked-task detection: completion_proof_failed, verification_unavailable, governance_blocked stop reasons in `tests/unit/execution_orchestrator.rs`
- [X] T031 [P] [US2] Add unit tests for downstream halt: blocked task prevents dependent execution, resolution unblocks dependents in `tests/unit/execution_orchestrator.rs`
- [X] T032 [US2] Add integration test: three-task plan with middle task blocked, verify halt, resume after resolution in `tests/integration/execution_orchestration_flow.rs`

**Checkpoint**: Blocked tasks halt downstream execution; checkpoints preserve blocked state; resume continues from blocked task after resolution.

---

## Phase 5: User Story 3 — Surface Execution State In Output (Priority: P3)

**Goal**: `boundline status`, `boundline inspect`, and assistant output project the six execution fields additively without breaking existing consumers.

**Independent Test**: Execute a multi-task plan and inspect status at each state transition. Verify all six execution fields are present and accurate.

### Implementation for User Story 3

- [X] T033 [US3] Implement `ExecutionOrchestrator::project_status(&self, view: &mut SessionStatusView)` — populate all eight execution projection fields from checkpoint state in `src/orchestrator/execution_orchestrator.rs`
- [X] T034 [US3] Wire execution projection into `SessionRuntime::build_status_view` in `src/cli/session.rs`: when `active_execution_run_id` is present, read checkpoint and call `project_status`
- [X] T035 [US3] Extend `boundline status` output rendering in `src/cli/output.rs` to display execution fields when present
- [X] T036 [US3] Extend `boundline inspect` to show dependency graph, completed tasks, blocked tasks with evidence refs when execution is active

### Tests for User Story 3

- [X] T037 [P] [US3] Add unit tests for status projection: all eight fields populated correctly for running, blocked, completed states in `tests/unit/execution_orchestrator.rs`
- [X] T038 [P] [US3] Add contract tests for status output: verify existing status fields unchanged, new fields present when execution active, absent when inactive in `tests/contract/execution_orchestration_contract.rs`
- [X] T039 [US3] Add integration test: execute three-task plan, inspect status at each state transition, verify all projection fields in `tests/integration/execution_orchestration_flow.rs`

**Checkpoint**: Status and inspect surfaces show execution state additively; existing consumers are unbroken.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge case handling, error recovery, and documentation.

- [X] T040 Handle empty task registry: transition to `completed` immediately with zero tasks in `src/orchestrator/execution_orchestrator.rs`
- [X] T041 Handle unreadable/missing checkpoint: produce degraded projection, do not silently clear session linkage in `src/orchestrator/execution_orchestrator.rs`
- [X] T042 Handle duplicate `--plan` invocation: return existing run state rather than creating a duplicate in `src/orchestrator/execution_orchestrator.rs`
- [X] T043 Handle incompatible checkpoint on resume: reject with explicit mismatch finding (plan fingerprint, workspace) in `src/orchestrator/execution_orchestrator.rs`
- [X] T044 Add edge case tests: empty plan, duplicate start, missing checkpoint, incompatible checkpoint, skipped/deferred dependencies in `tests/unit/execution_orchestrator.rs`

**Depends on**: Phase 3-5

---

---

## Phase 7: Roadmap & Docs Synchronization

**Purpose**: Convert the roadmap seed into a spec artifact per Boundline move-on-conversion policy and update all roadmap references.

- [X] T055 Copy roadmap seed to spec folder: `roadmap/features/19-plan-execution-orchestration.md` → `specs/080-plan-execution-orchestration/spec-plan-execution-orchestration.md`
- [X] T056 Remove original roadmap seed file `roadmap/features/19-plan-execution-orchestration.md`
- [X] T057 Update `roadmap/features/README.md` sequencing table: mark 19 as "In Progress (spec 080)"
- [X] T058 Update `roadmap/joint-roadmap-graph.md` if B19 dependencies changed
- [X] T059 Update `roadmap/Next - forward-roadmap.md` priority table

**Depends on**: Phase 6 (implementation complete before roadmap sync)

---

## Phase 8: Release, Quality, And Verification

**Purpose**: Final quality gates per Boundline wrapper rules (Rule 1, 3, 4).

- [X] T060 Update Cargo.toml version according to Boundline versioning policy (feature bump: increment minor, reset patch) in `Cargo.toml`
- [X] T061 Update `CHANGELOG.md` with entry for this version
- [X] T062 Run `./scripts/update-docs-versions.sh`
- [X] T063 Run `cargo fmt`
- [X] T064 Run `scripts/clippy.sh` and fix all warnings
- [X] T065 Run `scripts/test.sh` and verify all tests pass
- [X] T066 Run `scripts/coverage.sh` and confirm ≥95% coverage for every modified or created Rust file
- [X] T067 Run `scripts/check-no-local-paths.sh`
- [X] T068 Run `scripts/check-rust-no-panic.sh`
- [X] T069 Run `scripts/sync-distribution-metadata.sh` and commit updated distribution files
- [X] T070 Final review: verify all 20 FRs mapped to tasks, all 3 user stories independently testable, all 9 edge cases covered

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ─────────────────────────────────────────────┐
    │                                                         │
Phase 2 (Foundational: Graph + Checkpoint I/O) ───┐           │
    │                                               │           │
Phase 3 (US1: Sequential Execution) ◄──────────────┘           │
    │                                                         │
Phase 4 (US2: Blocked Tasks) ◄─────────────────────────────────┘
    │
Phase 5 (US3: Surface State) ◄── depends on Phase 3-4
    │
Phase 6 (Polish & Edge Cases)
    │
Phase 7 (Roadmap & Docs Sync) ◄── after Phase 6
    │
Phase 8 (Release & Quality)
```

### User Story Dependencies

| Story | Depends On | Can Start After |
|-------|-----------|-----------------|
| US1 (P1) | Phase 2 | Phase 2 complete |
| US2 (P2) | US1 (T017 completion-verification integration) | Phase 3 complete |
| US3 (P3) | US1 + US2 (checkpoint state needed for projection) | Phase 4 complete |

### Parallel Opportunities

- T001–T006 (Phase 1): all six tasks are [P], different files
- T012–T013 (Phase 2 tests): independent of each other
- T022–T023 (US1 tests): independent of each other
- T030–T031 (US2 tests): independent of each other
- T037–T038 (US3 tests): independent of each other
- T055–T059 (Phase 7): independent of each other
- T063–T069 (Phase 8): quality checks can run in parallel

---

## Task Summary

| Phase | Tasks | Story |
|-------|-------|-------|
| Phase 1: Setup | T001–T006 | Shared infrastructure |
| Phase 2: Foundational | T007–T013 | Graph validation + checkpoint I/O |
| Phase 3: US1 | T014–T024 | Sequential plan execution (MVP) |
| Phase 4: US2 | T025–T032 | Blocked-task handling |
| Phase 5: US3 | T033–T039 | Surface execution state |
| Phase 6: Polish | T040–T044 | Edge cases |
| Phase 7: Roadmap & Docs | T055–T059 | Seed conversion + reference updates |
| Phase 8: Release | T060–T070 | Quality gates per wrapper rules |

**Total**: 60 tasks

### MVP Scope

Phase 1 + Phase 2 + Phase 3 (T001–T024) = **24 tasks** deliver a working `boundline run --plan` with checkpointing and resume.

### Wrapper Rule Compliance

| Rule | Task IDs | Status |
|------|----------|--------|
| Rule 1: Cargo Version Bump | T060 | ✅ |
| Rule 2: Docs & Roadmap Sync | T055–T059, T061 | ✅ |
| Rule 3: Docs Version Sync | T062 | ✅ |
| Rule 4: Quality & Coverage | T063–T069 | ✅ |
