# Tasks: Canon Governance Expansion

**Input**: Design documents from `/specs/017-canon-governance-expansion/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes Canon mode selection, governed stage routing, approval refresh behavior, packet-readiness handling, and operator-facing session summaries.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Release boundary and test-harness setup for the governance expansion slice

- [X] T001 Bump crate version to `0.17.0` in `/Users/rt/workspace/synod/Cargo.toml` and `/Users/rt/workspace/synod/Cargo.lock`
- [X] T002 Create governed security-analysis fixture helpers in `/Users/rt/workspace/synod/tests/support/workspace_fixture.rs`
- [X] T003 Register new governance-expansion test modules in `/Users/rt/workspace/synod/tests/contract.rs`, `/Users/rt/workspace/synod/tests/integration.rs`, and `/Users/rt/workspace/synod/tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared mode-selection, governance-state, and surface primitives needed by every story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Extend Canon mode and stage-validation primitives for the bounded expansion in `/Users/rt/workspace/synod/src/domain/governance.rs`
- [ ] T005 [P] Extend autopilot selection, escalation hooks, and bounded packet-reuse helpers in `/Users/rt/workspace/synod/src/orchestrator/governance.rs`
- [X] T006 [P] Extend session-visible governance projection helpers in `/Users/rt/workspace/synod/src/domain/session.rs`, `/Users/rt/workspace/synod/src/cli/output.rs`, and `/Users/rt/workspace/synod/src/cli/inspect.rs`
- [X] T007 Add foundational unit coverage for mode validation and governance projection invariants in `/Users/rt/workspace/synod/tests/unit/canon_stage_mapping.rs`, `/Users/rt/workspace/synod/tests/unit/governance_policy.rs`, and `/Users/rt/workspace/synod/tests/unit/cli_output.rs`

**Checkpoint**: The bounded governance-expansion model exists before any story-specific routing or rendering changes widen behavior.

---

## Phase 3: User Story 1 - Route Existing Verification Through Governed Security Analysis (Priority: P1) 🎯 MVP

**Goal**: Let targeted existing-system verification stages route through Canon `security-assessment` while staying inside the same bounded session-native execution loop.

**Independent Test**: Prepare a `bug-fix` or `change` session that reaches `verify`, enable Canon governance for that stage, and confirm that Synod starts a governed `security-assessment` path, records the selected mode, and either continues, waits, or blocks explicitly.

### Tests for User Story 1

- [X] T008 [P] [US1] Add Canon runtime contract coverage for `security-assessment` start and refresh payloads in `/Users/rt/workspace/synod/tests/contract/canon_runtime_contract.rs`
- [X] T009 [P] [US1] Add integration coverage for bug-fix and change verification routing through `security-assessment` in `/Users/rt/workspace/synod/tests/integration/governance_autopilot_flow.rs`
- [X] T010 [P] [US1] Add integration coverage for invalid context or unsupported governed-mode blocking in `/Users/rt/workspace/synod/tests/integration/governance_autopilot_flow.rs`

### Implementation for User Story 1

- [X] T011 [US1] Add `security-assessment` to the Canon mode model and targeted verify-stage mappings in `/Users/rt/workspace/synod/src/domain/governance.rs`
- [ ] T012 [US1] Extend Canon request and response handling for governed security analysis in `/Users/rt/workspace/synod/src/adapters/governance_runtime.rs`
- [ ] T013 [US1] Route targeted verification stages through governed `security-assessment` using existing start and refresh semantics in `/Users/rt/workspace/synod/src/orchestrator/engine.rs`, `/Users/rt/workspace/synod/src/orchestrator/session_runtime.rs`, and `/Users/rt/workspace/synod/src/orchestrator/governance.rs`
- [ ] T014 [US1] Persist selected Canon mode, packet provenance, and blocked reasons for governed security-analysis runs in `/Users/rt/workspace/synod/src/orchestrator/governance.rs` and `/Users/rt/workspace/synod/src/cli/session.rs`

**Checkpoint**: Targeted verification stages can run through governed security analysis and stop explicitly on approval or blocked conditions.

---

## Phase 4: User Story 2 - Surface Governed Follow-On Analysis Through One Session Story (Priority: P2)

**Goal**: Make governed security analysis appear through the same session-native summaries already used for routing, execution condition, and next-step guidance.

**Independent Test**: Run a governed verification session that selects `security-assessment` and verify that `run`, `status`, `next`, and `inspect` all expose the selected Canon mode, governance condition, packet provenance, and next action consistently.

### Tests for User Story 2

- [X] T015 [P] [US2] Add contract coverage for governed-analysis session fields in `/Users/rt/workspace/synod/tests/contract/governance_session_contract.rs`
- [ ] T016 [P] [US2] Add integration coverage for coherent run, status, next, and inspect summaries in `/Users/rt/workspace/synod/tests/integration/governance_autopilot_flow.rs`
- [ ] T017 [P] [US2] Add unit coverage for governed-analysis rendering and packet binding headlines in `/Users/rt/workspace/synod/tests/unit/cli_output.rs` and `/Users/rt/workspace/synod/tests/unit/session_record.rs`

### Implementation for User Story 2

- [ ] T018 [US2] Extend session-status and run rendering for selected Canon mode, packet provenance, and next-action guidance in `/Users/rt/workspace/synod/src/cli/output.rs` and `/Users/rt/workspace/synod/src/cli/session.rs`
- [ ] T019 [US2] Extend trace summarization and inspect output for governed analysis mode selection and packet binding in `/Users/rt/workspace/synod/src/cli/inspect.rs`
- [ ] T020 [US2] Keep approval refresh and suggested-next-command behavior aligned for governed security-analysis sessions in `/Users/rt/workspace/synod/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/synod/src/domain/session.rs`

**Checkpoint**: Governed security analysis is visible through one coherent session-native operator story.

---

## Phase 5: User Story 3 - Keep The Governance Expansion Bounded And Extensible (Priority: P3)

**Goal**: Reject unsupported Canon modes explicitly while shaping the mode-selection and surface model so later `supply-chain-analysis` support can reuse the same bounded primitives.

**Independent Test**: Validate that unsupported Canon modes fail explicitly, existing non-expanded workflows remain unchanged, and the widened mode-selection model remains bounded to the current slice.

### Tests for User Story 3

- [X] T021 [P] [US3] Add unit coverage for unsupported-mode rejection and future-compatible selection behavior in `/Users/rt/workspace/synod/tests/unit/governance_policy.rs` and `/Users/rt/workspace/synod/tests/unit/canon_stage_mapping.rs`
- [ ] T022 [P] [US3] Add contract or integration coverage that non-expanded sessions preserve current behavior in `/Users/rt/workspace/synod/tests/contract/governance_session_contract.rs` and `/Users/rt/workspace/synod/tests/integration/governance_autopilot_flow.rs`

### Implementation for User Story 3

- [ ] T023 [US3] Reject unsupported newer Canon modes explicitly while preserving bounded expansion hooks in `/Users/rt/workspace/synod/src/domain/governance.rs` and `/Users/rt/workspace/synod/src/orchestrator/governance.rs`
- [ ] T024 [US3] Keep the widened packet-provenance and mode-selection model compatible with a later `supply-chain-analysis` slice in `/Users/rt/workspace/synod/src/domain/governance.rs`, `/Users/rt/workspace/synod/src/cli/output.rs`, and `/Users/rt/workspace/synod/src/cli/inspect.rs`

**Checkpoint**: The first slice stays bounded to `security-assessment` while leaving a clear structural path for later Canon governance expansion.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release alignment, generated context, validation, coverage, and docs

- [ ] T025 [P] Refresh generated agent and contributor context in `/Users/rt/workspace/synod/AGENTS.md` and `/Users/rt/workspace/synod/CONTRIBUTING.md`
- [X] T026 [P] Run focused governance validation and refresh `lcov.info` via `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and `cargo deny check licenses advisories bans sources`
- [X] T027 Update `/Users/rt/workspace/synod/README.md`, `/Users/rt/workspace/synod/ROADMAP.md`, `/Users/rt/workspace/synod/CHANGELOG.md`, `/Users/rt/workspace/synod/docs/`, `/Users/rt/workspace/synod/assistant/`, `/Users/rt/workspace/synod/.specify/templates/`, and touched tests or `lcov.info` for the `0.17.0` Canon governance expansion release, guarantee coverage on touched Rust files, run `cargo fmt`, and resolve all `cargo clippy` warnings and errors

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on User Story 1 because the operator surfaces should reflect real governed-analysis behavior.
- User Story 3 depends on Foundational and should reconcile with User Story 1 before final sign-off.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on US1 because surfaces must render real governed-analysis state.
- **US3**: Depends on Foundational and should align with US1 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Mode validation and state helpers come before session or renderer widening.
- Session and trace rendering must be finished before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T025 and T026 can run in parallel once implementation is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add Canon runtime contract coverage for security-assessment start and refresh payloads in tests/contract/canon_runtime_contract.rs"
Task: "Add integration coverage for bug-fix and change verification routing through security-assessment in tests/integration/governance_autopilot_flow.rs"

# Launch independent implementation work together after validations exist:
Task: "Add security-assessment to the Canon mode model and targeted verify-stage mappings in src/domain/governance.rs"
Task: "Extend Canon request and response handling for governed security analysis in src/adapters/governance_runtime.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate governed `security-assessment` routing before widening operator surfaces.

### Incremental Delivery

1. Add the bounded `security-assessment` mode expansion to the existing governance model.
2. Surface the governed analysis path coherently across run, status, next, and inspect.
3. Tighten unsupported-mode rejection and future-compatible selection hooks.
4. Finish with release alignment, docs, coverage, clippy cleanup, and fmt.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- The first task intentionally reserves the release bump to `0.17.0`.
- The final task is intentionally reserved for docs, assistant assets, templates, coverage on touched Rust files, `cargo fmt`, and clippy cleanup.