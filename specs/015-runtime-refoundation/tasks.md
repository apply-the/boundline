# Tasks: Runtime Refoundation

**Input**: Design documents from `/specs/015-runtime-refoundation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes execution control, failure handling, routing, trace guarantees, and operator-visible inspection.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Release and test harness setup for the refoundation slice

- [X] T001 Bump crate version to `0.15.0` in `Cargo.toml` and `Cargo.lock`
- [X] T002 Create shared runtime refoundation fixtures in `tests/support/runtime_refoundation.rs`
- [X] T003 Register runtime refoundation test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared runtime state, routing, and inspection primitives that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Extend session-native aggregate and routing projections in `src/domain/session.rs`, `src/domain/goal_plan.rs`, and `src/domain/trace.rs`
- [X] T005 [P] Add shared flow-state and route-decision helpers in `src/domain/flow_policy.rs` and `src/orchestrator/session_runtime.rs`
- [X] T006 [P] Add shared route-aware status and inspect rendering helpers in `src/cli/output.rs` and `src/cli/inspect.rs`
- [X] T007 Add foundational unit coverage for aggregate, routing, and rendering invariants in `tests/unit/runtime_routing.rs`, `tests/unit/session_model.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: The runtime can represent native, compatibility, and blocked states before story-specific behavior is added.

---

## Phase 3: User Story 1 - Session-Native Runtime Path (Priority: P1) 🎯 MVP

**Goal**: Make `goal -> plan -> run -> inspect` the primary bounded delivery path driven by live state.

**Independent Test**: Run the full session-native CLI journey on a workspace without a declarative execution profile and verify that Boundline persists a bounded task draft, executes bounded decisions from live state, and records explicit terminal reasoning.

### Tests for User Story 1

- [X] T008 [P] [US1] Add contract coverage for bounded task draft handoff and persisted decisions in `tests/contract/runtime_refoundation_contract.rs`
- [X] T009 [P] [US1] Add integration coverage for `goal -> plan -> run -> inspect` without declarative profiles in `tests/integration/runtime_refoundation_flow.rs`
- [X] T010 [P] [US1] Add integration coverage for recovery, exhaustion, and no-actionable termination in `tests/integration/runtime_refoundation_failure.rs`

### Implementation for User Story 1

- [X] T011 [US1] Persist the authoritative bounded task draft during planning in `src/cli/session.rs`, `src/orchestrator/goal_planner.rs`, and `src/orchestrator/session_runtime.rs`
- [X] T012 [US1] Rework live-state next-action selection and bounded recovery in `src/orchestrator/decision_loop.rs` and `src/domain/decision.rs`
- [X] T013 [US1] Persist decision history, evidence, and terminal outcomes in `src/domain/session.rs`, `src/domain/tool_result.rs`, and `src/domain/trace.rs`
- [X] T014 [US1] Surface route choice, decision summaries, and remediation cues in `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/session.rs`

**Checkpoint**: The session-native runtime path works independently and terminates explicitly with inspectable evidence.

---

## Phase 4: User Story 2 - Flow As Confirmed Policy (Priority: P2)

**Goal**: Make flow lightweight, operator-confirmed, and stage-constraining rather than silent or ornamental.

**Independent Test**: Plan a bug-fix-shaped goal, verify that flow is proposed with rationale, confirm or skip it explicitly, and observe that execution either honors the confirmed policy or blocks when confirmation is still pending.

### Tests for User Story 2

- [X] T015 [P] [US2] Add contract coverage for proposed, confirmed, and skipped flow behavior in `tests/contract/flow_policy_contract.rs`
- [X] T016 [P] [US2] Add integration coverage for inferred flow confirmation, override, and blocked run behavior in `tests/integration/runtime_refoundation_flow.rs`
- [X] T017 [P] [US2] Add unit coverage for stage-constrained decisions and transition rules in `tests/unit/flow_confirmation.rs` and `tests/unit/flow_policy_model.rs`

### Implementation for User Story 2

- [X] T018 [US2] Extend flow proposal and confirmation state handling in `src/orchestrator/flow_inference.rs`, `src/orchestrator/session_runtime.rs`, and `src/domain/goal_plan.rs`
- [X] T019 [US2] Apply confirmed flow constraints to decision selection and stage transitions in `src/orchestrator/decision_loop.rs` and `src/domain/flow_policy.rs`
- [X] T020 [US2] Update CLI plan and run guidance for confirm, override, and skip actions in `src/cli.rs`, `src/cli/session.rs`, and `src/cli/output.rs`

**Checkpoint**: Flow is visible, bounded, and operator-controlled without changing the session-native path back into a script.

---

## Phase 5: User Story 3 - Explicit Compatibility And Canon Boundaries (Priority: P3)

**Goal**: Preserve explicit compatibility behavior while keeping Canon at planning and stage boundaries instead of in the per-action control loop.

**Independent Test**: Compare a run driven by an explicit compatibility profile with a run driven by a persisted bounded task draft in a similar workspace and verify that routing, inspection, and Canon participation are all explicit.

### Tests for User Story 3

- [X] T021 [P] [US3] Add contract coverage for routing precedence and Canon boundary rules in `tests/contract/runtime_routing_contract.rs`
- [X] T022 [P] [US3] Add integration coverage for explicit compatibility mode and mixed-context routing precedence in `tests/integration/runtime_refoundation_compat.rs`
- [X] T023 [P] [US3] Add integration coverage for Canon artifacts as bounded planning and stage-boundary inputs in `tests/integration/runtime_refoundation_governance.rs`

### Implementation for User Story 3

- [X] T024 [US3] Implement explicit routing precedence and blocked remediation behavior in `src/cli/run.rs`, `src/cli/session.rs`, and `src/orchestrator/session_runtime.rs`
- [X] T025 [US3] Narrow `fixture.rs` to explicit compatibility helpers and keep compatibility routing visible in `src/fixture.rs` and `src/cli/output.rs`
- [X] T026 [US3] Restrict Canon participation to planning and stage-boundary evidence in `src/adapters/governance_runtime.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Compatibility remains available without defining the product, and Canon remains downstream from Boundline's per-action runtime logic.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release hygiene, generated context, coverage, and product-story alignment

- [X] T027 [P] Refresh generated agent and contributor context in `AGENTS.md` and `CONTRIBUTING.md`
- [X] T028 [P] Run release validation and refresh `lcov.info` via `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo deny check licenses advisories bans sources`
- [X] T029 Increase coverage in `tests/unit/`, `tests/integration/`, and `lcov.info` and update `README.md`, `ROADMAP.md`, `tech-docs/session-native-orchestrator-review.md`, `tech-docs/adaptive-execution.md`, `assistant/README.md`, `.specify/templates/spec-template.md`, `.specify/templates/plan-template.md`, `.specify/templates/tasks-template.md`, `assistant/claude/commands/`, `assistant/codex/commands/`, and `assistant/copilot/prompts/` for the `0.15.0` runtime refoundation

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the preferred MVP path.
- User Story 2 depends on User Story 1 because flow policy only matters once the native path is authoritative.
- User Story 3 depends on Foundational and should integrate after User Story 1 stabilizes routing and inspection surfaces.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on US1 session-native planning and execution surfaces.
- **US3**: Depends on Foundational and should reconcile with US1 routing state before sign-off.

### Within Each User Story

- Contract, integration, and unit validations should fail before implementation when the runtime behavior is not yet present.
- State models and route helpers come before CLI or inspection wiring.
- Trace and failure-handling behavior must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T027 and T028 can run in parallel once the implementation is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for bounded task draft handoff and persisted decisions in tests/contract/runtime_refoundation_contract.rs"
Task: "Add integration coverage for goal -> plan -> run -> inspect without declarative profiles in tests/integration/runtime_refoundation_flow.rs"

# Launch independent User Story 1 implementation work together after validations exist:
Task: "Persist the authoritative bounded task draft during planning in src/cli/session.rs, src/orchestrator/goal_planner.rs, and src/orchestrator/session_runtime.rs"
Task: "Persist decision history, evidence, and terminal outcomes in src/domain/session.rs, src/domain/tool_result.rs, and src/domain/trace.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate the session-native runtime path independently before widening scope.

### Incremental Delivery

1. Make the session-native path authoritative.
2. Add flow as confirmed policy.
3. Reconcile compatibility routing and Canon boundaries.
4. Finish with release, coverage, and documentation rollout.

## Notes

- `[P]` tasks touch different files or surfaces and can be split safely.
- Each story is independently testable even though the preferred implementation order is US1 → US2 → US3.
- The final cross-cutting task is intentionally reserved for coverage growth and documentation/template/example alignment with the `0.15.0` runtime refoundation.