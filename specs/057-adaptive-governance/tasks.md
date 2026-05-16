# Tasks: Control Graduation And Adaptive Governance

**Input**: Design documents from `/specs/057-adaptive-governance/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are required for this Boundline feature because it changes executable governance behavior, compatibility handling, failure paths, trace output, and CLI-visible runtime summaries.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. `US1`, `US2`, `US3`)
- Include exact file paths in every task description

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Refresh shared planning evidence and create the repo-visible document surfaces this slice will fill.

- [x] T001 Refresh the provider-doc audit and record the applied delta or no-change result in `assistant/catalog/model-catalog.toml` and `specs/057-adaptive-governance/research.md`
- [x] T002 Create the operator-facing S4 document stubs and section outlines in `docs/control-graduation-model.md`, `docs/adaptive-governance.md`, `docs/runtime-confidence-and-calibration.md`, and `docs/degradation-and-escalation.md`
- [x] T003 [P] Align the feature-local consumer contract, projection contract, and scenario wording in `specs/057-adaptive-governance/contracts/adaptive-governance-consumer-contract.md`, `specs/057-adaptive-governance/contracts/adaptive-governance-projection-contract.md`, and `specs/057-adaptive-governance/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core typed models and projection scaffolding that all user stories depend on.

**⚠️ CRITICAL**: Full story sign-off depends on this phase completing. The
contract-boundary slice may proceed once the compatibility and projection
subset is in place.

- [x] T004 Create shared adaptive-governance enums, constants, and typed records in `src/domain/governance.rs` and `crates/boundline-core/src/domain.rs`
- [x] T005 [P] Extend Canon companion-contract parsing and compatibility scaffolding in `src/adapters/governance_runtime.rs` and `tests/unit/governance_runtime.rs`
- [x] T006 [P] Add session and trace projection scaffolding for adaptive-governance fields in `src/domain/session.rs`, `src/orchestrator/engine.rs`, and `src/orchestrator/session_runtime.rs`
- [x] T007 Implement shared stop-semantic mapping and governance transition helpers in `src/domain/governance.rs` and `tests/unit/governance_policy.rs`
- [x] T008 Wire adaptive-governance fixture, export, and task-context support in `src/fixture.rs`, `src/lib.rs`, and `tests/unit/task_context_state.rs`

**Checkpoint**: The adaptive-governance models, compatibility hooks, and
projection scaffolds required for contract-boundary work are ready.

---

## Phase 3: User Story 1 - Adopt Governance Progressively (Priority: P1) 🎯 MVP

**Goal**: Start new or low-trust governed work in advisory mode, then graduate to stronger governance states and rollout profiles explicitly.

**Independent Test**: Run representative governed boundaries through the session-native flow and verify that `plan`, `run`, `status`, `next`, and `inspect` show one explicit governance state, one rollout profile, and one continue, degrade, escalate, wait, or stop outcome.

### Tests for User Story 1

- [x] T009 [P] [US1] Add failing unit coverage for advisory defaults, rollout-profile resolution, and governance-state promotion rules in `tests/unit/governance_policy.rs`
- [x] T010 [P] [US1] Add failing integration coverage for advisory-first, operator-approved graduation, and resumed-automation boundary evaluation in `tests/integration/session_governance_flow.rs` and `tests/integration/session_adaptive_flow.rs`
- [x] T011 [P] [US1] Add failing contract coverage for session and trace projection of governance state in `tests/contract/governance_session_contract.rs` and `tests/contract/governance_trace_contract.rs`

### Implementation for User Story 1

- [x] T012 [US1] Implement runtime governance-state and rollout-profile resolution in `src/domain/governance.rs`
- [x] T013 [US1] Persist governance-state rationale, rollout-profile state, and operator-approval provenance in `src/orchestrator/governance.rs`, `src/domain/session.rs`, and `src/orchestrator/review_trace.rs`
- [x] T014 [US1] Surface governance state, rollout profile, and next-action summaries in `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [x] T015 [US1] Integrate advisory-first startup, operator-approved graduated state transitions, and resumed-automation gates into the runtime flow in `src/orchestrator/decision_loop.rs` and `src/orchestrator/session_runtime.rs`

**Checkpoint**: User Story 1 is fully functional and independently testable as the MVP slice.

---

## Phase 4: User Story 2 - Degrade And Escalate Safely (Priority: P2)

**Goal**: Make low-confidence, missing-evidence, and blocked-governance scenarios degrade or escalate explicitly instead of weakening governance silently.

**Independent Test**: Trigger reviewer gaps, low-confidence evidence, repeated overrides, and blocked compatibility scenarios and verify that the runtime records one explicit degradation or escalation outcome with rationale and next action.

### Tests for User Story 2

- [ ] T016 [P] [US2] Add failing unit coverage for confidence levels, reviewer-credibility inputs, calibration-evidence handling, trust decay, and degradation-mode mapping in `tests/unit/governance_policy.rs`
- [ ] T017 [P] [US2] Add failing integration coverage for low-confidence degradation and escalation flows in `tests/integration/session_governance_flow.rs`, `tests/integration/workflow_follow_through.rs`, and `tests/integration/workflow_follow_through_blocked.rs`
- [ ] T018 [P] [US2] Add failing contract coverage for degraded and escalated execution projection in `tests/contract/governance_trace_contract.rs` and `tests/contract/governance_execution_profile_contract.rs`

### Implementation for User Story 2

- [ ] T019 [US2] Implement confidence assessment, reviewer-credibility weighting, calibration-evidence persistence, trust evolution, and degradation selection in `src/domain/governance.rs` and `src/orchestrator/governance.rs`
- [ ] T020 [US2] Implement escalation triggers, override recording, and blocked-governance handling in `src/orchestrator/governance.rs` and `src/orchestrator/decision_loop.rs`
- [ ] T021 [US2] Persist calibration evidence together with degraded and escalated outcomes in `src/domain/session.rs`, `src/orchestrator/review_trace.rs`, and `src/fixture.rs`
- [ ] T022 [US2] Surface degradation, escalation, override, and recovery rationale in `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`

**Checkpoint**: User Stories 1 and 2 both work independently with explicit degradation and escalation behavior.

---

## Phase 5: User Story 3 - Preserve The Canon And Boundline Contract Boundary (Priority: P3)

**Goal**: Consume the required Canon posture baseline plus the optional adaptive companion contract without giving Canon runtime control of Boundline behavior.

**Independent Test**: Compare baseline-only, companion-present, and unsupported-companion runs and verify that Boundline remains locally authoritative for confidence, trust, degradation, escalation, and stop behavior.

### Tests for User Story 3

- [x] T023 [P] [US3] Add failing unit coverage for baseline-only, companion-present, and unsupported-companion parsing in `tests/unit/governance_runtime.rs`
- [x] T024 [P] [US3] Add failing integration coverage for Canon baseline and optional companion behavior in `tests/integration/canon_governance_flow.rs` and `tests/integration/governance_autopilot_flow.rs`
- [x] T025 [P] [US3] Add failing contract coverage for contract-line projection and companion-compatibility failures in `tests/contract/local_governance_runtime_contract.rs` and `tests/contract/governance_trace_contract.rs`

### Implementation for User Story 3

- [x] T026 [US3] Implement optional `adaptive_governance` companion consumption and stage-policy gating in `src/adapters/governance_runtime.rs` and `src/domain/governance.rs`
- [x] T027 [US3] Keep Canon contract lines, approval/readiness/project-memory/lineage/promotion semantics, and local runtime authority separated in `src/orchestrator/governance.rs`, `src/domain/session.rs`, and `src/cli/inspect.rs`
- [x] T028 [US3] Document the S4 contract boundary in `docs/control-graduation-model.md`, `docs/adaptive-governance.md`, and `specs/057-adaptive-governance/contracts/adaptive-governance-consumer-contract.md`

**Checkpoint**: All three user stories are independently functional and the cross-repo contract boundary remains explicit.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Close documentation, validation, and coverage across all user stories.

- [ ] T029 [P] Update operator documentation and release notes in `README.md`, `CHANGELOG.md`, `docs/runtime-confidence-and-calibration.md`, and `docs/degradation-and-escalation.md`
- [ ] T030 [P] Refresh contributor and assistant guidance for the new governance surfaces in `AGENTS.md` and `assistant/README.md`
- [x] T031 Run focused and full validation for `tests/unit/governance_policy.rs`, `tests/unit/governance_runtime.rs`, `tests/integration/session_governance_flow.rs`, `cargo test --no-run --all-targets`, and `cargo nextest run --workspace --all-features`
- [ ] T032 Validate the operator walkthrough and recorded runtime scenarios in `specs/057-adaptive-governance/quickstart.md` and `docs/adaptive-governance.md`
- [ ] T033 Increase the workspace version and release-facing references in `Cargo.toml`, `CHANGELOG.md`, `assistant/plugin-metadata.json`, and distribution metadata under `distribution/`
- [ ] T034 Apply an appropriate design pattern to break up oversized files or functions in `src/orchestrator/governance.rs`, `src/domain/governance.rs`, `src/cli/output.rs`, and any newly expanded governance modules before final sign-off
- [ ] T035 Ensure modified-file and feature-level coverage remains at 95% or higher by refreshing `lcov.info`, any package-scoped coverage artifacts, and the recorded evidence in `specs/057-adaptive-governance/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies, can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on Foundational completion and reuses the governance-state primitives completed in User Story 1.
- **User Story 3 (Phase 5)**: Depends on Foundational completion and integrates with the projection surfaces stabilized in User Story 1.
- **Final Phase**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Independent MVP after Foundational.
- **User Story 2 (P2)**: Uses the runtime governance-state model from US1 but remains independently testable through degradation and escalation scenarios.
- **User Story 3 (P3)**: Uses the shared Canon-consumer and projection primitives from Foundational, but it does not require US1 or US2 runtime-progression work to close the contract boundary independently.

### Within Each User Story

- Validation tasks MUST fail before implementation when the feature changes executable behavior.
- Domain models and compatibility contracts come before orchestration logic.
- Persistence and projection changes come before CLI summary work.
- Story-level documentation or contract updates must land before the story is signed off.

### Parallel Opportunities

- `T003` can run in parallel with `T001` and `T002`.
- `T005` and `T006` can run in parallel after `T004`.
- Validation tasks within each user story marked `[P]` can run in parallel.
- Once Foundational is complete, US2 and US3 can be prepared in parallel while US1 stabilizes shared runtime primitives.

---

## Parallel Example: User Story 1

```bash
# Launch the failing validation work together:
Task: "Add failing unit coverage for advisory defaults, rollout-profile resolution, and governance-state promotion rules in tests/unit/governance_policy.rs"
Task: "Add failing integration coverage for advisory-first and graduated boundary evaluation in tests/integration/session_governance_flow.rs and tests/integration/session_adaptive_flow.rs"
Task: "Add failing contract coverage for session and trace projection of governance state in tests/contract/governance_session_contract.rs and tests/contract/governance_trace_contract.rs"
```

## Parallel Example: User Story 2

```bash
# Launch the failing degradation and escalation checks together:
Task: "Add failing unit coverage for confidence levels, trust decay, and degradation-mode mapping in tests/unit/governance_policy.rs"
Task: "Add failing integration coverage for low-confidence degradation and escalation flows in tests/integration/session_governance_flow.rs, tests/integration/workflow_follow_through.rs, and tests/integration/workflow_follow_through_blocked.rs"
Task: "Add failing contract coverage for degraded and escalated execution projection in tests/contract/governance_trace_contract.rs and tests/contract/governance_execution_profile_contract.rs"
```

## Parallel Example: User Story 3

```bash
# Launch the companion-contract compatibility checks together:
Task: "Add failing unit coverage for baseline-only, companion-present, and unsupported-companion parsing in tests/unit/governance_runtime.rs"
Task: "Add failing integration coverage for Canon baseline and optional companion behavior in tests/integration/canon_governance_flow.rs and tests/integration/governance_autopilot_flow.rs"
Task: "Add failing contract coverage for contract-line projection and companion-compatibility failures in tests/contract/local_governance_runtime_contract.rs and tests/contract/governance_trace_contract.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: Verify advisory-first and rollout-profile progression independently before expanding into degradation or companion-contract work.

### Incremental Delivery

1. Complete Setup + Foundational to establish typed runtime primitives.
2. Deliver User Story 1 and validate the MVP session-native behavior.
3. Deliver User Story 2 and validate degradation and escalation without breaking US1.
4. Deliver User Story 3 and validate Canon baseline versus optional companion behavior.
5. Finish with the Final Phase validation, docs, and coverage refresh.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together.
2. Once Foundational is stable:
   - Developer A: User Story 1 runtime progression and projection.
   - Developer B: User Story 2 degradation and escalation paths.
   - Developer C: User Story 3 Canon companion-contract compatibility and docs.
3. Merge back only after story-level validation passes independently.

---

## Notes

- [P] tasks touch different files and can proceed without waiting on another incomplete task in the same phase.
- Every user story includes explicit validation before implementation sign-off.
- `authority-governance-v1` remains the required Canon baseline throughout this task list; `adaptive-governance-v1` is additive and optional unless local stage policy explicitly requires it.
- Keep runtime confidence, trust, degradation, escalation, and stop behavior locally owned in Boundline even when Canon companion semantics are present.