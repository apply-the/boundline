# Tasks: Multi-Workspace Orchestration

**Input**: Design documents from `/specs/012-multi-workspace/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds new
CLI behavior, persisted cluster state, cluster-aware inspection, and new config
precedence semantics.

**Organization**: Tasks are grouped by user story so each slice can deliver
bounded, inspectable value independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the test harness and fixtures for clustered workflows.

- [X] T001 Wire cluster test harness registrations in /Users/rt/workspace/synod/tests/unit.rs, /Users/rt/workspace/synod/tests/contract.rs, and /Users/rt/workspace/synod/tests/integration.rs
- [X] T002 [P] Add reusable clustered workspace fixture helpers in /Users/rt/workspace/synod/tests/support/workspace_fixture.rs
- [X] T003 [P] Scaffold feature test files in /Users/rt/workspace/synod/tests/contract/cluster_cli_contract.rs, /Users/rt/workspace/synod/tests/contract/cluster_config_contract.rs, /Users/rt/workspace/synod/tests/integration/cluster_bootstrap_flow.rs, /Users/rt/workspace/synod/tests/integration/cluster_status_flow.rs, /Users/rt/workspace/synod/tests/integration/cluster_config_flow.rs, /Users/rt/workspace/synod/tests/unit/cluster_models.rs, /Users/rt/workspace/synod/tests/unit/cluster_projection.rs, and /Users/rt/workspace/synod/tests/unit/cluster_config_resolution.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared cluster models, persistence, and command wiring used by all stories.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Create shared cluster domain models and exports in /Users/rt/workspace/synod/src/domain/cluster.rs, /Users/rt/workspace/synod/src/domain.rs, and /Users/rt/workspace/synod/src/lib.rs
- [X] T005 [P] Add cluster persistence primitives in /Users/rt/workspace/synod/src/adapters/cluster_store.rs and /Users/rt/workspace/synod/src/adapters.rs
- [X] T006 [P] Extend configuration types and precedence primitives for cluster scope in /Users/rt/workspace/synod/src/domain/configuration.rs and /Users/rt/workspace/synod/src/cli/config.rs
- [X] T007 [P] Extend CLI parsing, dispatch, and rendering hooks for cluster-aware commands in /Users/rt/workspace/synod/src/cli.rs and /Users/rt/workspace/synod/src/cli/output.rs
- [X] T008 Add explicit cluster report and member-projection primitives in /Users/rt/workspace/synod/src/domain/cluster.rs and /Users/rt/workspace/synod/src/cli/cluster.rs

**Checkpoint**: Foundation ready - Synod can represent clusters, persist cluster state, parse cluster-aware commands, and resolve cluster scope in shared primitives.

---

## Phase 3: User Story 1 - Establish a Clustered Delivery Context (Priority: P1) 🎯 MVP

**Goal**: Let operators register a bounded cluster of related workspaces through an explicit Synod entry point.

**Independent Test**: Register two valid workspaces into one cluster, verify `.synod/cluster.toml` is created in the primary workspace, and confirm invalid or duplicate members fail without partial state.

### Tests for User Story 1

- [X] T009 [P] [US1] Add contract coverage for `synod cluster init` membership validation and failure behavior in /Users/rt/workspace/synod/tests/contract/cluster_cli_contract.rs
- [X] T010 [P] [US1] Add integration coverage for successful cluster bootstrap and invalid-member rejection in /Users/rt/workspace/synod/tests/integration/cluster_bootstrap_flow.rs
- [X] T011 [P] [US1] Add unit coverage for cluster validation and file persistence in /Users/rt/workspace/synod/tests/unit/cluster_models.rs and /Users/rt/workspace/synod/src/adapters/cluster_store.rs

### Implementation for User Story 1

- [X] T012 [US1] Implement cluster validation and persisted cluster file models in /Users/rt/workspace/synod/src/domain/cluster.rs and /Users/rt/workspace/synod/src/adapters/cluster_store.rs
- [X] T013 [US1] Implement `synod cluster init` command flow in /Users/rt/workspace/synod/src/cli/cluster.rs and /Users/rt/workspace/synod/src/cli.rs
- [X] T014 [US1] Render cluster init summaries and actionable validation errors in /Users/rt/workspace/synod/src/cli/output.rs

**Checkpoint**: User Story 1 is complete when Synod can bootstrap a named cluster from a primary workspace and fail cleanly on invalid members.

---

## Phase 4: User Story 2 - Inspect Cluster Status and Trace Context (Priority: P2)

**Goal**: Let operators inspect member session and trace state from one cluster-aware view.

**Independent Test**: With one cluster whose members have mixed session and trace conditions, run cluster status and cluster inspect and verify every member is classified explicitly with the right trace or missing-state summary.

### Tests for User Story 2

- [X] T015 [P] [US2] Add contract coverage for `synod cluster status` and `synod cluster inspect` in /Users/rt/workspace/synod/tests/contract/cluster_cli_contract.rs
- [X] T016 [P] [US2] Add integration coverage for mixed member state aggregation in /Users/rt/workspace/synod/tests/integration/cluster_status_flow.rs
- [X] T017 [P] [US2] Add unit coverage for cluster session projection and member classification in /Users/rt/workspace/synod/tests/unit/cluster_projection.rs

### Implementation for User Story 2

- [X] T018 [US2] Implement explicit member-session projection for cluster reports in /Users/rt/workspace/synod/src/cli/cluster.rs and /Users/rt/workspace/synod/src/domain/cluster.rs
- [X] T019 [US2] Implement cluster status and cluster inspect aggregation in /Users/rt/workspace/synod/src/cli/cluster.rs and /Users/rt/workspace/synod/src/adapters/session_store.rs
- [X] T020 [US2] Surface cluster-aware member classifications and trace references in /Users/rt/workspace/synod/src/cli/output.rs and /Users/rt/workspace/synod/src/adapters/trace_store.rs

**Checkpoint**: User Stories 1 and 2 are complete when operators can bootstrap a cluster and inspect explicit per-member state and traces from one command surface.

---

## Phase 5: User Story 3 - Apply Cluster Defaults Without Losing Local Control (Priority: P3)

**Goal**: Let operators save cluster-level defaults that sit between workspace-local and user-global configuration.

**Independent Test**: Save one cluster-level route, verify effective config for a member workspace reports cluster as the source, then add a workspace-local override and verify the workspace value wins.

### Tests for User Story 3

- [X] T021 [P] [US3] Add contract coverage for cluster-scoped `synod config show|set|unset` in /Users/rt/workspace/synod/tests/contract/cluster_config_contract.rs
- [X] T022 [P] [US3] Add integration coverage for cluster precedence and malformed cluster config handling in /Users/rt/workspace/synod/tests/integration/cluster_config_flow.rs
- [X] T023 [P] [US3] Add unit coverage for cluster-aware effective routing resolution in /Users/rt/workspace/synod/tests/unit/cluster_config_resolution.rs and /Users/rt/workspace/synod/tests/unit/config_resolution.rs

### Implementation for User Story 3

- [X] T024 [US3] Implement cluster-scoped config load/save/unset behavior in /Users/rt/workspace/synod/src/adapters/cluster_store.rs and /Users/rt/workspace/synod/src/cli/config.rs
- [X] T025 [US3] Extend effective routing resolution and config scopes for cluster precedence in /Users/rt/workspace/synod/src/domain/configuration.rs and /Users/rt/workspace/synod/src/cli.rs
- [X] T026 [US3] Render cluster-aware effective config and source attribution in /Users/rt/workspace/synod/src/cli/output.rs and /Users/rt/workspace/synod/src/cli/config.rs

**Checkpoint**: All user stories are complete when Synod can register a cluster, inspect member state, and resolve cluster-scoped defaults without breaking local overrides.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, keep the spec artifacts aligned, and run repository validation.

- [X] T027 [P] Update clustered workflow documentation in /Users/rt/workspace/synod/README.md, /Users/rt/workspace/synod/docs/getting-started.md, /Users/rt/workspace/synod/docs/configuration.md, and /Users/rt/workspace/synod/specs/012-multi-workspace/quickstart.md
- [X] T028 [P] Sync planning metadata and roadmap guidance in /Users/rt/workspace/synod/AGENTS.md and /Users/rt/workspace/synod/ROADMAP.md
- [X] T029 Run formatting, lint, and targeted/full test validation with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --all-targets`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies.
- **Phase 2: Foundational**: Depends on Setup and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Foundational and delivers the MVP cluster bootstrap.
- **Phase 4: User Story 2**: Depends on Foundational and integrates with the bootstrap artifacts from US1.
- **Phase 5: User Story 3**: Depends on Foundational and is safest once the cluster file and command surface are stable.
- **Phase 6: Polish**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational and has no dependency on later stories.
- **US2 (P2)**: Can start after Foundational but depends on the cluster bootstrap artifacts defined in US1.
- **US3 (P3)**: Can start after Foundational but depends on the cluster store and config primitives stabilized in US1 and US2.

### Within Each User Story

- Validation tasks should be written first and observed failing before implementation.
- Domain and persistence models should land before CLI mutation or rendering code.
- Output rendering should follow once the projected state model is stable.
- Each story should satisfy its independent test before the next priority is treated as complete.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005, T006, and T007 can run in parallel after T004; T008 follows once the shared models exist.
- **US1**: T009, T010, and T011 can run in parallel; T013 and T014 can overlap once cluster validation is in place.
- **US2**: T015, T016, and T017 can run in parallel; T019 and T020 can overlap once cluster session projection exists.
- **US3**: T021, T022, and T023 can run in parallel; T024 and T025 can overlap once cluster config persistence is stable.
- **Polish**: T027 and T028 can run in parallel before final validation in T029.

## Parallel Example: User Story 1

```bash
# Build bootstrap validation together:
Task: "T009 Add contract coverage for synod cluster init membership validation and failure behavior in tests/contract/cluster_cli_contract.rs"
Task: "T010 Add integration coverage for successful cluster bootstrap and invalid-member rejection in tests/integration/cluster_bootstrap_flow.rs"
Task: "T011 Add unit coverage for cluster validation and file persistence in tests/unit/cluster_models.rs"

# Then split persistence and command work:
Task: "T012 Implement cluster validation and persisted cluster file models in src/domain/cluster.rs and src/adapters/cluster_store.rs"
Task: "T013 Implement synod cluster init command flow in src/cli/cluster.rs and src/cli.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Setup.
2. Complete Foundational.
3. Complete User Story 1.
4. Validate cluster bootstrap independently.

### Incremental Delivery

1. Add cluster bootstrap.
2. Add cluster status and inspect aggregation.
3. Add cluster-scoped config precedence.
4. Finish with docs and full validation.

## Notes

- All tasks follow the required checklist format with exact file paths.
- The first slice intentionally stops short of automatic cross-repository plan generation and distributed execution.