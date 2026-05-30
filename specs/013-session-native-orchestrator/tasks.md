# Tasks: Session-Native Orchestrator

**Input**: Design documents from `/specs/013-session-native-orchestrator/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are included for all stories — this feature defines
executable behavior, failure handling, replanning, and trace guarantees.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded,
inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Version bump and project structure for the new feature

- [x] T001 Bump crate version to 0.13.0 in Cargo.toml
- [x] T002 [P] Create `src/domain/decision.rs` with module stub and register in `src/domain.rs`
- [x] T003 [P] Create `src/domain/goal_plan.rs` with module stub and register in `src/domain.rs`
- [x] T004 [P] Create `src/domain/flow_policy.rs` with module stub and register in `src/domain.rs`
- [x] T005 [P] Create `src/domain/tool_result.rs` with module stub and register in `src/domain.rs`
- [x] T006 [P] Create `src/orchestrator/decision_loop.rs` with module stub and register in `src/orchestrator.rs`
- [x] T007 [P] Create `src/orchestrator/goal_planner.rs` with module stub and register in `src/orchestrator.rs`
- [x] T008 [P] Create `src/orchestrator/flow_inference.rs` with module stub and register in `src/orchestrator.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core domain models and primitives that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T009 Implement `DecisionType` enum (Analyze, Code, Test, Fix, Replan) and `DecisionStatus` enum (Pending, Dispatched, Verified, Failed, Recovered) in `src/domain/decision.rs`
- [x] T010 [P] Implement `EvidenceRef` struct with `EvidenceKind` enum (Trace, File, Canon, ToolOutput) and `reference: String` in `src/domain/decision.rs`
- [x] T011 Implement `Decision` struct with all fields (id, decision_type, target, rationale, expected_outcome, evidence_inputs, status, tool_result, created_at, completed_at) and validation logic in `src/domain/decision.rs`
- [x] T012 [P] Implement `ToolResult` struct (tool_id, invocation, exit_code, stdout, stderr, diff, duration_ms, success) with validation in `src/domain/tool_result.rs`
- [x] T013 Implement `PlannedTask` struct and `GoalPlan` struct with validation and status transitions (Draft→Confirmed→Superseded) in `src/domain/goal_plan.rs`
- [x] T014 [P] Implement `WorkspaceSignals` struct (language, file_count, has_config, has_canon, has_tests) in `src/domain/goal_plan.rs`
- [x] T015 [P] Implement `InferredFlow` struct (flow_name, confidence_reason, confirmed) in `src/domain/goal_plan.rs`
- [x] T016 Implement `StagePolicy` and `FlowPolicy` structs with `TransitionCondition` enum (AllVerified, ExplicitAdvance) and built-in policy tables for bug-fix, change, delivery in `src/domain/flow_policy.rs`
- [x] T017 Extend `FlowStageDefinition` in `src/domain/flow.rs` with `allowed_decision_types` field and populate for all built-in flow stages
- [x] T018 [P] Add `DecisionCreated`, `DecisionDispatched`, `DecisionVerified`, `DecisionFailed`, `DecisionRecovered`, `GoalPlanCreated`, `FlowInferred` variants to `TraceEventType` in `src/domain/trace.rs`
- [x] T019 Add `goal_plan: Option<GoalPlan>`, `decisions: Vec<Decision>`, and `active_flow_policy: Option<FlowPolicy>` fields to the session model in `src/domain/session.rs`
- [x] T020 [P] Write unit tests for Decision model validation and status transitions in `tests/unit/decision_model.rs` and register in `tests/unit.rs`
- [x] T021 [P] Write unit tests for GoalPlan model validation and status transitions in `tests/unit/goal_plan_model.rs` and register in `tests/unit.rs`
- [x] T022 [P] Write unit tests for FlowPolicy model and stage constraint enforcement in `tests/unit/flow_policy_model.rs` and register in `tests/unit.rs`
- [x] T023 [P] Write unit tests for ToolResult model validation in `tests/unit/tool_result_model.rs` and register in `tests/unit.rs`

**Checkpoint**: Foundation ready — all domain models in place, validated by unit tests

---

## Phase 3: User Story 1 — Bounded Decision Loop (Priority: P1) 🎯 MVP

**Goal**: Implement the observe→decide→act→verify→update execution loop

**Independent Test**: Run `boundline run` on a session with a recorded goal and verify
the engine produces a sequence of typed, inspectable decision objects in the trace

### Tests for User Story 1

- [x] T024 [P] [US1] Contract test for the decision loop interface: loop produces decisions, enforces step limits, and terminates in explicit terminal state in `tests/contract/decision_loop_contract.rs` and register in `tests/contract.rs`
- [x] T025 [P] [US1] Unit test for observe phase: workspace state and evidence collection in `tests/unit/decision_loop.rs` and register in `tests/unit.rs`
- [x] T026 [P] [US1] Unit test for decide phase: next-action selection from evidence and plan in `tests/unit/decision_loop.rs`
- [x] T027 [P] [US1] Unit test for verify phase: tool result evaluation and status transition in `tests/unit/decision_loop.rs`
- [x] T028 [P] [US1] Unit test for exhaustion terminal state when max_steps is reached in `tests/unit/decision_loop.rs`

### Implementation for User Story 1

- [x] T029 [US1] Implement observe phase in `src/orchestrator/decision_loop.rs`: collect workspace file state, last decision result, accumulated evidence, and session context
- [x] T030 [US1] Implement decide phase in `src/orchestrator/decision_loop.rs`: select next DecisionType and target from observed state, plan tasks, and evidence; create Decision object with Pending status
- [x] T031 [US1] Implement act phase in `src/orchestrator/decision_loop.rs`: dispatch Decision through agent or tool adapter, transition status to Dispatched, capture ToolResult
- [x] T032 [US1] Implement verify phase in `src/orchestrator/decision_loop.rs`: evaluate ToolResult against expected_outcome, transition Decision to Verified or Failed
- [x] T033 [US1] Implement update phase in `src/orchestrator/decision_loop.rs`: persist Decision in session, update context with new evidence, advance plan position
- [x] T034 [US1] Implement bounded loop coordinator in `src/orchestrator/decision_loop.rs`: call observe→decide→act→verify→update in a loop, check step limits, produce explicit terminal state (Success, Failure, Exhausted, NoActionableState)
- [x] T035 [US1] Emit trace events for each decision phase (DecisionCreated, DecisionDispatched, DecisionVerified, DecisionFailed, DecisionRecovered) in `src/orchestrator/decision_loop.rs`
- [x] T036 [US1] Implement recovery path: when verify fails, select Fix or Replan decision referencing failure evidence, transition failed Decision to Recovered status in `src/orchestrator/decision_loop.rs`

---

## Phase 4: User Story 2 — Goal-Derived Planning (Priority: P2)

**Goal**: Derive initial bounded task draft from goal, workspace state, and context

**Independent Test**: Run `boundline plan` and verify it produces a GoalPlan with tasks
referencing real workspace files, without requiring `boundline init` or execution profile

### Tests for User Story 2

- [x] T037 [P] [US2] Contract test for goal planner: given goal text and workspace path, produces GoalPlan with non-empty tasks in `tests/contract/goal_plan_contract.rs` and register in `tests/contract.rs`
- [x] T038 [P] [US2] Unit test for workspace signal collection in `tests/unit/goal_planner.rs` and register in `tests/unit.rs`
- [x] T039 [P] [US2] Unit test for task derivation from goal text and signals in `tests/unit/goal_planner.rs`
- [x] T040 [P] [US2] Unit test for error when goal is missing in `tests/unit/goal_planner.rs`

### Implementation for User Story 2

- [x] T041 [US2] Implement workspace signal collector in `src/orchestrator/goal_planner.rs`: enumerate file tree (bounded depth 4), detect language from manifests, check `.boundline/config.toml` and `.canon/` presence
- [x] T042 [US2] Implement task derivation in `src/orchestrator/goal_planner.rs`: parse goal text, match against workspace signals, generate ordered PlannedTask list with targets, expected outcomes, and decision type hints
- [x] T043 [US2] Implement Canon artifact scanning in `src/orchestrator/goal_planner.rs`: if `.canon/` exists, scan for governed artifacts and include as source_evidence in GoalPlan
- [x] T044 [US2] Integrate goal planner into `boundline plan` command in `src/cli/session.rs`: when session has recorded goal and no execution profile, call goal planner, persist GoalPlan in session state, show plan summary to user
- [x] T045 [US2] Handle error path in `src/cli/session.rs`: if `boundline plan` is called without a recorded goal, return explicit error message

---

## Phase 5: User Story 3 — Inferred Flow with Lightweight Confirmation (Priority: P3)

**Goal**: Propose inferred flow from goal text during `boundline plan`

**Independent Test**: Record a goal with "fix" keyword, run `boundline plan`, and verify
bug-fix flow is proposed with confirmation prompt

### Tests for User Story 3

- [x] T046 [P] [US3] Unit test for flow inference from bug-fix keywords in `tests/unit/flow_inference.rs` and register in `tests/unit.rs`
- [x] T047 [P] [US3] Unit test for flow inference from change keywords in `tests/unit/flow_inference.rs`
- [x] T048 [P] [US3] Unit test for flow inference from delivery keywords in `tests/unit/flow_inference.rs`
- [x] T049 [P] [US3] Unit test for no-flow fallback when no keywords match in `tests/unit/flow_inference.rs`

### Implementation for User Story 3

- [x] T050 [US3] Implement flow inference engine in `src/orchestrator/flow_inference.rs`: match goal text against keyword patterns (fix/bug/broken→bug-fix, add/implement/feature→change, deliver/release/ship→delivery), return InferredFlow with confidence_reason
- [x] T051 [US3] Integrate flow inference into `boundline plan` in `src/cli/session.rs`: after goal plan creation, run inference, show proposed flow to user with confirmation prompt
- [x] T052 [US3] Implement `--flow` override flag in `boundline plan` in `src/cli/session.rs`: skip inference and use explicit flow
- [x] T053 [US3] Implement `--no-flow` flag in `boundline plan` in `src/cli/session.rs`: skip inference and run without flow constraints

---

## Phase 6: User Story 4 — Flow as Decision Policy (Priority: P4)

**Goal**: Constrain decision types based on active flow stage

**Independent Test**: Run with bug-fix flow and verify only Analyze decisions in
investigate stage, only Code/Fix in implement stage, only Test/Replan in verify stage

### Tests for User Story 4

- [x] T054 [P] [US4] Unit test: bug-fix flow investigate stage only allows Analyze decisions in `tests/unit/flow_policy_model.rs`
- [x] T055 [P] [US4] Unit test: bug-fix flow implement stage allows Code and Fix in `tests/unit/flow_policy_model.rs`
- [x] T056 [P] [US4] Unit test: stage transition requires all decisions verified in `tests/unit/flow_policy_model.rs`
- [x] T057 [P] [US4] Unit test: decision rejected when type not allowed by current stage in `tests/unit/flow_policy_model.rs`

### Implementation for User Story 4

- [x] T058 [US4] Integrate FlowPolicy into decision loop decide phase in `src/orchestrator/decision_loop.rs`: before creating a Decision, check if active FlowPolicy allows the selected DecisionType for the current stage
- [x] T059 [US4] Implement stage transition logic in `src/orchestrator/decision_loop.rs`: when all decisions in current stage are verified, advance to next stage, emit StageTransitioned trace event
- [x] T060 [US4] Implement terminal flow completion in `src/orchestrator/decision_loop.rs`: when final stage completes, set session to success terminal state

---

## Phase 7: User Story 5 — Tool-Driven Execution (Priority: P5)

**Goal**: Ground the decision loop in concrete tool adapter invocations

**Independent Test**: Run with a Code decision and verify tool adapter writes
the file, runs verification, and feeds output as evidence for next decision

### Tests for User Story 5

- [x] T061 [P] [US5] Unit test: tool adapter produces structured ToolResult from StepExecutionResult in `tests/unit/tool_result_model.rs`
- [x] T062 [P] [US5] Unit test: ToolResult is persisted as evidence in next decision in `tests/unit/decision_loop.rs`
- [x] T063 [P] [US5] Unit test: failed tool invocation (non-zero exit) feeds failure evidence in `tests/unit/decision_loop.rs`

### Implementation for User Story 5

- [x] T064 [US5] Extend tool dispatch in `src/orchestrator/decision_loop.rs` act phase: convert adapter output to ToolResult, attach to Decision
- [x] T065 [US5] Implement ToolResult→EvidenceRef conversion in `src/domain/decision.rs`: create ToolOutput evidence from completed Decision
- [x] T066 [US5] Implement file-read tool dispatch: read target file and return content as ToolResult in `src/adapters/tool.rs`
- [x] T067 [US5] Implement file-write tool dispatch: write or patch target file and return diff as ToolResult in `src/adapters/tool.rs`
- [x] T068 [US5] Implement command-execution tool dispatch: run validation command, capture stdout/stderr/exit_code as ToolResult in `src/adapters/tool.rs`

---

## Phase 8: User Story 6 — Fixture Compatibility (Priority: P6)

**Goal**: Preserve fixture execution path as fallback, route correctly

**Independent Test**: Verify `boundline run` with `.boundline/execution.json` uses fixture
path; `boundline run` with recorded goal uses decision loop

### Tests for User Story 6

- [x] T069 [P] [US6] Integration test: `boundline run` with execution profile uses fixture path in `tests/integration/fixture_compat_flow.rs` and register in `tests/integration.rs`
- [x] T070 [P] [US6] Integration test: `boundline run` with recorded goal uses decision loop in `tests/integration/session_native_flow.rs` and register in `tests/integration.rs`
- [x] T071 [P] [US6] Integration test: `boundline run` with both goal and profile uses decision loop unless `--profile` is explicit in `tests/integration/session_native_flow.rs`

### Implementation for User Story 6

- [x] T072 [US6] Implement routing predicate in `src/cli/run.rs`: check session for goal_plan, check for execution profile, check for `--profile` flag, route to decision_loop or fixture accordingly
- [x] T073 [US6] Implement routing predicate in `src/orchestrator/session_runtime.rs`: integrate decision loop entry when session has goal_plan
- [x] T074 [US6] Demote fixture.rs: extract reusable workspace mutation primitives (file write, command execute) into helper functions callable by the decision loop's tool adapters in `src/fixture.rs`
- [x] T075 [US6] Ensure existing fixture tests pass without modification by verifying fixture routing with execution profile in `tests/integration/fixture_compat_flow.rs`

---

## Phase 9: End-to-End Integration

**Purpose**: Full session-native flow validation

- [x] T076 Integration test: full `goal → plan → run → inspect` without `init` in `tests/integration/session_native_flow.rs`
- [x] T077 Integration test: decision loop with flow inference, flow policy constraints, and stage transitions in `tests/integration/session_native_flow.rs`
- [x] T078 Integration test: decision loop recovery path — verification failure triggers fix decision in `tests/integration/session_native_flow.rs`
- [x] T079 Integration test: exhaustion terminal state at step limit in `tests/integration/session_native_flow.rs`

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, coverage, formatting, and release hygiene

- [x] T080 Update `ROADMAP.md`: move Priority 1 items to completed status under v0.13.0, remove the companion review reference from Priority 1 since the work is done
- [x] T081 Update `README.md`: document the session-native flow (`goal → plan → run → inspect`) as the primary usage path, push `init` to optional/advanced section
- [x] T082 Update `docs/getting-started.md`: rewrite getting started around goal capture and plan instead of init templates
- [x] T083 Update `docs/configuration.md`: document new `--flow` and `--no-flow` flags, document GoalPlan interaction with config
- [x] T084 Update `CONTRIBUTING.md`: add new domain modules and test files to the contributor map
- [x] T085 Ensure all new modules have adequate test coverage — run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and verify new code paths are covered
- [x] T086 Run `cargo fmt --all` to format all source files
- [x] T087 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix all warnings
- [x] T088 Run full validation suite: `cargo nextest run --workspace --all-features` — all tests pass
- [x] T089 Run `cargo deny check licenses advisories bans sources` — no new violations

---

## Dependencies

```text
T001 ──→ T002..T008 (setup before stubs)
T002..T008 ──→ T009..T023 (stubs before foundation)
T009..T023 ──→ T024..T036 (foundation before US1)
T024..T036 ──→ T037..T045 (US1 before US2 — loop needs plan)
T037..T045 ──→ T046..T053 (US2 before US3 — plan needs inference)
T046..T053 ──→ T054..T060 (US3 before US4 — inference before policy)
T054..T060 ──→ T061..T068 (US4 before US5 — policy before tools)
T061..T068 ──→ T069..T075 (US5 before US6 — tools before compat)
T069..T075 ──→ T076..T079 (US6 before e2e)
T076..T079 ──→ T080..T089 (e2e before polish)
```

## Parallel Execution Opportunities

- **Phase 1**: T002–T008 are all independent stub creations
- **Phase 2**: T010, T012, T014, T015, T018, T020–T023 can run in parallel
- **Phase 3–8**: Test tasks within each phase can run in parallel
- **Phase 10**: T080–T084 (docs) can run in parallel with each other

## Implementation Strategy

1. **MVP**: Phase 1 + Phase 2 + Phase 3 (US1) delivers the core decision loop — this alone shifts the product from static replay to bounded observe→decide→act→verify
2. **Incremental**: Each subsequent phase adds one user story that builds on the previous
3. **Final**: Phase 9 validates the integrated flow, Phase 10 cleans up docs and tooling

## Summary

| Metric                         | Value |
| ------------------------------ | ----- |
| Total tasks                    | 89    |
| Phase 1 (Setup)                | 8     |
| Phase 2 (Foundation)           | 15    |
| Phase 3 (US1 — Decision Loop)  | 13    |
| Phase 4 (US2 — Goal Planning)  | 9     |
| Phase 5 (US3 — Flow Inference) | 8     |
| Phase 6 (US4 — Flow Policy)    | 7     |
| Phase 7 (US5 — Tool-Driven)    | 8     |
| Phase 8 (US6 — Fixture Compat) | 7     |
| Phase 9 (E2E Integration)      | 4     |
| Phase 10 (Polish)              | 10    |
| Parallelizable tasks           | 38    |
| MVP scope                      | T001–T036 (36 tasks) |
