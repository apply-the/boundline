# Tasks: Canon Governance Adapter

**Input**: Design documents from `/specs/009-canon-governance-adapter/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds new executable governance behavior, runtime selection, packet readiness rules, approval refresh semantics, autopilot decisions, and trace-visible session output.

**Organization**: Tasks are grouped by user story so each governed slice can be implemented, validated, and inspected independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the governance modules and extend the test harnesses before runtime behavior changes.

- [X] T001 Wire governance module exports and harness entries in src/adapters.rs, src/domain.rs, src/orchestrator.rs, src/lib.rs, tests/unit.rs, tests/contract.rs, and tests/integration.rs
- [X] T002 [P] Scaffold governance unit, contract, and integration test files in tests/unit/governance_policy.rs, tests/unit/governance_runtime.rs, tests/unit/canon_stage_mapping.rs, tests/contract/governance_execution_profile_contract.rs, tests/contract/local_governance_runtime_contract.rs, tests/contract/canon_runtime_contract.rs, tests/contract/governance_session_contract.rs, tests/contract/governance_trace_contract.rs, tests/integration/session_governance_flow.rs, tests/integration/canon_governance_flow.rs, and tests/integration/governance_autopilot_flow.rs
- [X] T003 [P] Extend governed workspace and packet fixture builders in tests/support/workspace_fixture.rs for local runtime, Canon runtime, packet readiness, and approval refresh scenarios

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared governance profile, state, task-context APIs, and trace primitives that every story relies on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Create governance domain types, validation rules, packet entities, and autopilot decision models in src/domain/governance.rs and extend src/domain/execution.rs
- [X] T005 [P] Implement task-context storage and retrieval APIs for GovernedStageRecord, GovernedStagePacket, PacketReuseBinding, and AutopilotDecisionRecord in src/domain/task_context.rs and src/domain/session.rs
- [X] T006 [P] Add governance trace events, summary fields, and inspect/session projection primitives in src/domain/trace.rs, src/cli/output.rs, src/cli/inspect.rs, and src/cli/session.rs
- [X] T007 [P] Create shared governance orchestration interfaces and runtime traits in src/adapters/governance_runtime.rs and src/orchestrator/governance.rs
- [X] T008 Implement governance manifest loading, stage-policy lookup, default-runtime handling, and stage-to-mode whitelist validation in src/domain/execution.rs, src/domain/flow.rs, and src/fixture.rs after the governance domain types in src/domain/governance.rs exist
- [X] T009 Implement shared governed-attempt lineage, state-patch helpers, and orchestrator persistence hooks in src/orchestrator/governance.rs, src/orchestrator/session_runtime.rs, and src/orchestrator/engine.rs

**Checkpoint**: Foundation ready - governance profiles can be loaded, persisted, traced, and projected through the existing Synod execution lifecycle.

---

## Phase 3: User Story 1 - Govern A Synod Stage Through A Local-First Runtime (Priority: P1) 🎯 MVP

**Goal**: Route a built-in stage through one explicit governance runtime, keep Synod in control of the delivery loop, and block visibly when required governance cannot proceed.

**Independent Test**: Start a governed built-in flow with Canon unavailable or disabled, verify that Synod selects the local runtime, records explicit governance state, and either continues or stops in a visible governed state without silent ungoverned fallback.

### Tests for User Story 1

- [X] T010 [P] [US1] Add contract coverage for governance profile parsing and local runtime responses in tests/contract/governance_execution_profile_contract.rs and tests/contract/local_governance_runtime_contract.rs
- [X] T011 [P] [US1] Add integration coverage for local-first stage governance and required-governance blocking in tests/integration/session_governance_flow.rs
- [X] T012 [P] [US1] Add unit coverage for governance policy validation, runtime selection, and required-governance blocking rules in tests/unit/governance_policy.rs and tests/unit/governance_runtime.rs

### Implementation for User Story 1

- [X] T013 [US1] Implement LocalGovernanceRuntime request/response flow and deterministic local packet creation in src/adapters/governance_runtime.rs and src/orchestrator/governance.rs
- [X] T014 [US1] Integrate stage-boundary governance selection, local fallback evidence, and required-governance blocking into src/fixture.rs, src/orchestrator/session_runtime.rs, and src/orchestrator/engine.rs
- [X] T015 [US1] Project local governance state, blocked reasons, and next-step guidance through src/domain/session.rs, src/cli/output.rs, src/cli/run.rs, src/cli/session.rs, and src/cli/inspect.rs

**Checkpoint**: User Story 1 is complete when a governed stage can run locally, stop explicitly when governance is required and unavailable, and expose the result through the existing CLI surfaces.

---

## Phase 4: User Story 2 - Reuse Governed Canon Packets As Bounded Stage Input (Priority: P2)

**Goal**: Open Canon-backed governed runs for meaningful stages, validate packet readiness deterministically, and reuse bounded packets in later stages without exposing unbounded Canon context.

**Independent Test**: Run a Canon-governed stage that produces a reusable packet, verify that Synod records Canon runtime state and packet provenance, and confirm that a later built-in stage consumes the bounded packet instead of rebuilding context from scratch.

### Tests for User Story 2

- [X] T016 [P] [US2] Add contract coverage for Canon runtime start/refresh responses and packet-readiness rejection in tests/contract/canon_runtime_contract.rs and tests/contract/governance_trace_contract.rs
- [X] T017 [P] [US2] Add integration coverage for Canon-governed stage execution and immediate-upstream packet reuse in tests/integration/canon_governance_flow.rs
- [X] T018 [P] [US2] Add unit coverage for Canon 0.18.0 stage-to-mode whitelist validation, packet-readiness classification, and packet reuse binding rules in tests/unit/canon_stage_mapping.rs and tests/unit/governance_runtime.rs

### Implementation for User Story 2

- [X] T019 [US2] Implement CanonCliRuntime invocation, start/refresh semantics, and Canon packet contract parsing in src/adapters/governance_runtime.rs and src/orchestrator/governance.rs
- [X] T020 [US2] Implement packet-readiness classification against the per-mode primary document expectations, reuse-binding creation, and bounded downstream packet input injection in src/domain/governance.rs, src/orchestrator/governance.rs, and src/fixture.rs
- [X] T021 [US2] Persist Canon run references, packet provenance, approval refresh state, and packet rejection outcomes in src/domain/task_context.rs, src/domain/session.rs, src/orchestrator/session_runtime.rs, and src/orchestrator/engine.rs
- [X] T022 [US2] Render Canon mode, run references, packet provenance, and governance packet rejection events in src/domain/trace.rs, src/cli/output.rs, src/cli/run.rs, src/cli/inspect.rs, and src/cli/session.rs

**Checkpoint**: User Stories 1 and 2 are complete when Canon-backed governed stages produce inspectable reusable packets and later stages consume only the bounded packet evidence defined by the contracts.

---

## Phase 5: User Story 3 - Use Autopilot To Choose A Compliant Governed Path (Priority: P3)

**Goal**: Let Synod choose among bounded compliant governance actions, refresh approval state through existing commands, and block visibly when no compliant governed path remains.

**Independent Test**: Run a governance-required stage with autopilot enabled, force a decision point with multiple compliant paths and an approval wait, and verify that Synod records the candidate actions, selected action, approval refresh, or blocked outcome without bypassing governance.

### Tests for User Story 3

- [X] T023 [P] [US3] Add contract coverage for governance session projections and autopilot decision traces in tests/contract/governance_session_contract.rs and tests/contract/governance_trace_contract.rs
- [X] T024 [P] [US3] Add integration coverage for autopilot mode selection, approval waiting, approval refresh, blocked outcomes, and explicit no-bypass behavior while approval is still pending in tests/integration/governance_autopilot_flow.rs and tests/integration/session_governance_flow.rs
- [X] T025 [P] [US3] Add unit coverage for autopilot candidate ordering, narrowed-context rules, escalation targets, escalation reuse lineage, and forbidden transitions out of `awaiting_approval` in tests/unit/governance_policy.rs and tests/unit/governance_runtime.rs

### Implementation for User Story 3

- [X] T026 [US3] Implement autopilot candidate generation, Canon mode ordering, and bounded narrowed-context selection in src/domain/governance.rs and src/orchestrator/governance.rs
- [X] T027 [US3] Implement approval-aware refresh handling for `status`, `step`, and `run` with one refresh request per command, escalation rules, and governed terminal outcomes in src/adapters/governance_runtime.rs, src/orchestrator/governance.rs, src/orchestrator/session_runtime.rs, and src/orchestrator/engine.rs
- [X] T028 [US3] Project autopilot candidates, selected mode, target stage, approval waits, blocked rationale, and refresh-driven CLI messaging through src/domain/session.rs, src/domain/trace.rs, src/cli/output.rs, src/cli/run.rs, src/cli/inspect.rs, and src/cli/session.rs

**Checkpoint**: All user stories are complete when governance-required stages can choose one compliant path, wait for approval through existing commands, and expose the entire decision record through status and inspect surfaces.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize documentation, versioning, coverage, and repository-wide validation for the governed slice.

- [X] T029 [P] Update governance documentation and examples in README.md, ROADMAP.md, AGENTS.md, assistant/README.md, and specs/009-canon-governance-adapter/quickstart.md
- [X] T030 [P] Raise governance coverage for packet edge cases, approval refresh, and autopilot lineage in tests/unit/governance_policy.rs, tests/unit/governance_runtime.rs, tests/integration/session_governance_flow.rs, tests/integration/canon_governance_flow.rs, and tests/integration/governance_autopilot_flow.rs
- [X] T031 [P] Bump crate and lockfile version to 0.9.0 in Cargo.toml and Cargo.lock
- [X] T032 Run formatting, lint, and test validation from Cargo.toml with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --all-targets`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP local-first governance slice.
- **Phase 4: User Story 2**: Depends on Phase 2 and builds on the shared governance state and runtime contracts created for US1.
- **Phase 5: User Story 3**: Depends on Phase 2 and is safest once the governed runtime state and packet lineage from US1 and US2 are stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other user stories.
- **US2 (P2)**: Starts after Foundational but depends on the governance domain model and runtime traits stabilized in US1.
- **US3 (P3)**: Starts after Foundational but depends on the governance evidence, packet lineage, and Canon refresh surfaces established in US1 and US2.

### Within Each User Story

- Contract, unit, and integration coverage should be written first and observed failing before implementation.
- Domain and orchestration changes should land before CLI rendering and session output that consume the new governance state.
- Packet readiness and reuse rules should be stable before approval-refresh and autopilot projections are finalized.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005, T006, and T007 can run in parallel after T004; T008 and T009 should follow once the governance types exist.
- **US1**: T010, T011, and T012 can run in parallel; T013 and T014 can overlap once the shared governance interfaces are stable.
- **US2**: T016, T017, and T018 can run in parallel; T019 and T020 can overlap once Canon runtime interfaces exist.
- **US3**: T023, T024, and T025 can run in parallel; T026 and T027 can overlap once Canon packet lineage and approval refresh are working.
- **Polish**: T029, T030, and T031 can run in parallel before the final validation task T032.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T010 Add contract coverage for governance profile parsing and local runtime responses in tests/contract/governance_execution_profile_contract.rs and tests/contract/local_governance_runtime_contract.rs"
Task: "T011 Add integration coverage for local-first stage governance and required-governance blocking in tests/integration/session_governance_flow.rs"
Task: "T012 Add unit coverage for governance policy validation, runtime selection, and required-governance blocking rules in tests/unit/governance_policy.rs and tests/unit/governance_runtime.rs"

# Then split runtime and orchestration work:
Task: "T013 Implement LocalGovernanceRuntime request/response flow and deterministic local packet creation in src/adapters/governance_runtime.rs and src/orchestrator/governance.rs"
Task: "T014 Integrate stage-boundary governance selection, local fallback evidence, and required-governance blocking into src/fixture.rs, src/orchestrator/session_runtime.rs, and src/orchestrator/engine.rs"
```

## Parallel Example: User Story 2

```bash
# Validate Canon runtime and reuse rules together:
Task: "T016 Add contract coverage for Canon runtime start/refresh responses and packet-readiness rejection in tests/contract/canon_runtime_contract.rs and tests/contract/governance_trace_contract.rs"
Task: "T017 Add integration coverage for Canon-governed stage execution and immediate-upstream packet reuse in tests/integration/canon_governance_flow.rs"
Task: "T018 Add unit coverage for Canon stage-to-mode validation, packet-readiness classification, and packet reuse binding rules in tests/unit/canon_stage_mapping.rs and tests/unit/governance_runtime.rs"

# Then split runtime and packet integration work:
Task: "T019 Implement CanonCliRuntime invocation, start/refresh semantics, and Canon packet contract parsing in src/adapters/governance_runtime.rs and src/orchestrator/governance.rs"
Task: "T020 Implement packet-readiness classification, reuse-binding creation, and bounded downstream packet input injection in src/domain/governance.rs, src/orchestrator/governance.rs, and src/fixture.rs"
```

## Parallel Example: User Story 3

```bash
# Validate autopilot decision behavior together:
Task: "T023 Add contract coverage for governance session projections and autopilot decision traces in tests/contract/governance_session_contract.rs and tests/contract/governance_trace_contract.rs"
Task: "T024 Add integration coverage for autopilot mode selection, approval waiting, approval refresh, and blocked outcomes in tests/integration/governance_autopilot_flow.rs and tests/integration/session_governance_flow.rs"
Task: "T025 Add unit coverage for autopilot candidate ordering, narrowed-context rules, escalation targets, and escalation reuse lineage in tests/unit/governance_policy.rs and tests/unit/governance_runtime.rs"

# Then split decision and approval integration work:
Task: "T026 Implement autopilot candidate generation, Canon mode ordering, and bounded narrowed-context selection in src/domain/governance.rs and src/orchestrator/governance.rs"
Task: "T027 Implement approval-aware refresh handling, escalation rules, and governed terminal outcomes in src/adapters/governance_runtime.rs, src/orchestrator/governance.rs, src/orchestrator/session_runtime.rs, and src/orchestrator/engine.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate one local-first governed stage and one required-governance blocked stage.
5. Confirm governance state is visible through `status`, `run`, and `inspect` before expanding into Canon reuse and autopilot.

### Incremental Delivery

1. Deliver Setup + Foundational to establish the governance profile, state model, and trace surfaces.
2. Deliver US1 to make stage-scoped governance available without requiring Canon.
3. Deliver US2 to add Canon-backed packet readiness and bounded reuse.
4. Deliver US3 to add approval-aware autopilot decisions.
5. Finish with docs, version bump to 0.9.0, and full repository validation.

### Suggested MVP Scope

- User Story 1 only.
- Keep User Stories 2 and 3 behind the shared governance foundation so the first increment already delivers a real local-first governance boundary instead of Canon-only scaffolding.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story tasks, and exact file paths.
- Governance state persistence in task context is a first-class implementation concern in this slice because session, trace, and inspect projections all depend on one explicit stored source of truth.
- The crate version moves to 0.9.0 for this slice and must stay aligned across Cargo.toml, Cargo.lock, docs, and user-facing examples.