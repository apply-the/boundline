# Tasks: Project Memory Delivery Integration

**Input**: Design documents from `specs/050-project-memory-delivery-integration/`
**Prerequisites**: plan.md (required), spec.md (required), research.md,
data-model.md, quickstart.md

**Validation**: Layered validation is mandatory. Test tasks verify behavior.
Independent review and evidence-capture tasks are required.

**Organization**: Tasks are grouped by user story for independent
implementation, validation, and auditability.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1, US2, US3)
- Exact file paths included

---

## Phase 0: Version Bump

**Purpose**: Establish the release identity for this slice

- [x] T001 Bump workspace version from `0.50.0` to `0.51.0` in `Cargo.toml` `[workspace.package]` and update version references in `README.md`, `assistant/plugin-metadata.json`, `distribution/channel-metadata.toml`

---

## Phase 1: Governance & Artifacts

**Purpose**: Confirm controls are in place before implementation

- [x] T002 Verify execution mode, risk, scope, and invariants are recorded in `specs/050-project-memory-delivery-integration/spec.md` and `plan.md`
- [x] T003 Create decision log at `specs/050-project-memory-delivery-integration/decision-log.md`

---

## Phase 2: Consumer Domain Types (Blocking Prerequisites)

**Purpose**: Core consumer-side types that MUST be complete before integration logic

- [x] T004 Create `src/domain/project_memory.rs` with `PromotionStateView` enum (`Stable`, `PendingOrIndex`, `EvidenceOnly`, `Manual`, `Unknown`)
- [x] T005 [P] Add `LineageRef` struct with consumer fields (`contract_version`, `source_run`, `mode`, `profile`, `promotion_state`, `approval_state`, `readiness`, `published_at`, `update_strategy`, `source_artifacts`) in `src/domain/project_memory.rs`
- [x] T006 [P] Add `CompatibilityOutcome` enum (`Compatible`, `Unsupported`) in `src/domain/project_memory.rs`
- [x] T007 [P] Add `ProjectMemoryContext` struct (`status`, `compatibility`, `surfaces`, `evidence_refs`, `effective_promotion_state`) in `src/domain/project_memory.rs`
- [x] T008 [P] Add `ProjectMemorySurface` struct (`path`, `lineage`, `promotion_view`, `category`) in `src/domain/project_memory.rs`
- [x] T009 Register module in `src/domain.rs` and re-export from `src/lib.rs`

**Checkpoint**: Consumer types compile and serde round-trips pass

---

## Phase 3: User Story 1 - Use Stable Project Memory Without Treating Pending As Truth (Priority: P1)

**Goal**: Stage planner uses Canon-promoted project memory when stable while
keeping pending or index-only outputs visible but non-authoritative.

**Independent Test**: Present stable, pending, evidence-only, and absent Canon
states and verify stage selection behavior follows Boundline-owned logic.

### Validation for US1 (MANDATORY)

- [x] T010 [P] [US1] Write unit tests for `PromotionStateView` mapping from Canon vocabulary in `src/domain/project_memory.rs` (inline `#[cfg(test)]`)
- [x] T011 [P] [US1] Write unit tests for `CompatibilityOutcome::check(contract_version)` across supported and unsupported contract lines in `src/domain/project_memory.rs`

### Implementation for US1

- [x] T012 [US1] Implement Canon output reader function `read_project_memory(workspace_root) -> ProjectMemoryContext` in `src/domain/project_memory.rs`: read Canon's named `docs/project/*.md` surfaces, adjacent `<surface>.packet-metadata.json` sidecars, and supporting evidence roots under `docs/evidence/<mode>/<RUN_ID>/`
- [x] T013 [US1] Implement contract-version compatibility check in `src/domain/project_memory.rs`: `CompatibilityOutcome::check(contract_version: &str) -> CompatibilityOutcome` pinned to the supported `0.1.x` line
- [x] T014 [US1] Implement Canon-vocabulary-to-consumer mapping in `src/domain/project_memory.rs`, including approval-aware handling for `auto-if-approved`
- [x] T015 [US1] Call `read_project_memory()` at stage-planning time in `src/orchestrator/session_runtime.rs` and bridge the result into the existing compacted Canon memory path for planner/task-context reuse
- [x] T016 [US1] Surface `ProjectMemoryContext.effective_promotion_state` in stage-planner decisions in `src/orchestrator/planner.rs` or `src/orchestrator/goal_planner.rs`: stable = credible context, pending = visible-only, absent = continue without Canon
- [x] T017 [US1] Capture validation evidence in `specs/050-project-memory-delivery-integration/decision-log.md`

**Checkpoint**: Stage planner distinguishes stable from non-authoritative Canon output

---

## Phase 4: User Story 2 - Integrate Canon Refs Into Assurance And Governed Stage Flow (Priority: P1)

**Goal**: Boundline consumes Canon refs and promoted evidence in assurance and
governed-stage orchestration without surrendering orchestration to Canon.

**Independent Test**: Run delivery stages with Canon evidence fixtures and verify
consumption and trace recording.

### Validation for US2 (MANDATORY)

- [x] T018 [P] [US2] Write integration test with fixture Canon output (stable + evidence) verifying `ProjectMemoryContext` is populated and traced in `tests/`

### Implementation for US2

- [x] T019 [US2] Feed Canon project-memory provenance from the compacted Canon-memory path into governance input documents in `src/orchestrator/governance.rs`
- [x] T020 [US2] Surface Canon project-memory summaries and artifact refs in session-native `status` and `next` output through the existing compacted Canon memory projection
- [x] T021 [US2] Surface Canon project-memory summaries and artifact refs in session-native `inspect` output through the existing compacted Canon memory projection
- [x] T022 [US2] Record consumed Canon memory summary, credibility, and artifact refs in execution trace through the existing compacted Canon memory projection
- [ ] T023 [US2] Capture validation evidence

**Checkpoint**: Governed stages consume Canon evidence; status/inspect show Canon refs

---

## Phase 5: User Story 3 - Fail Explicitly On Contract Incompatibility (Priority: P2)

**Goal**: Unsupported contract-line changes result in bounded stop with repair guidance.

**Independent Test**: Exercise supported `0.1.x`, future-line, and malformed
version scenarios and verify consumer behavior.

### Validation for US3 (MANDATORY)

- [x] T024 [P] [US3] Write unit tests for supported and unsupported contract-line scenarios in `src/domain/project_memory.rs`

### Implementation for US3

- [x] T025 [US3] Add bounded-stop behavior when `CompatibilityOutcome::Unsupported` is resolved during planning-context assembly: surface repair guidance in `src/orchestrator/session_runtime.rs`
- [x] T026 [US3] Collapse the compatibility ladder to `Compatible` or `Unsupported` and surface update guidance for future contract lines in trace and status
- [ ] T027 [US3] Capture validation evidence

**Checkpoint**: Incompatible contract versions produce explicit repair guidance

---

## Final Phase: Verification & Compliance

**Purpose**: Ensure everything compiles, passes, and is well-formatted

- [x] T028 Update `docs/getting-started.md` or `docs/architecture.md` with Canon project-memory integration documentation
- [x] T029 [P] Update `CHANGELOG.md` with 0.51.0 entry
- [x] T030 [P] Update `ROADMAP.md` if applicable
- [x] T031 Run `cargo fmt` and verify clean with `cargo fmt --check`
- [ ] T032 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix any issues
- [ ] T033 Run `cargo nextest run` and verify all tests pass
- [ ] T034 Increase coverage of modified files to ≥95% using `cargo llvm-cov`

---

## Dependencies

```text
T001 → T004..T009 (version bump before domain types)
T004 + T009 → T010..T016 (domain types before consumer logic)
T012 → T015 (reader before session_runtime integration)
T015 → T019..T022 (session context before assurance/view integration)
T013 → T025, T026 (compatibility check before stop/update-guidance behavior)
T033 → T034 (tests pass before coverage)
```

## Summary

- **Total tasks**: 34
- **Phase 0 (Version Bump)**: 1 task
- **Phase 1 (Governance)**: 2 tasks
- **Phase 2 (Domain Types)**: 6 tasks
- **Phase 3 (US1 - Stable Memory)**: 8 tasks (2 validation + 6 implementation)
- **Phase 4 (US2 - Assurance/Governed)**: 6 tasks (1 validation + 5 implementation)
- **Phase 5 (US3 - Compatibility)**: 4 tasks (1 validation + 3 implementation)
- **Final Phase (Verification)**: 7 tasks
- **Parallel opportunities**: T005/T006/T007/T008, T010/T011, T018, T024, T029/T030
- **MVP scope**: Phase 0-3 (version bump + governance + domain types + stage-planner integration)
