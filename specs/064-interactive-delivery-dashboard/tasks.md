# Tasks: Interactive Delivery Dashboard

**Input**: Design documents from `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/`
**Prerequisites**: `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/plan.md`, `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/spec.md`, `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/research.md`, `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/data-model.md`, `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/contracts/`
**Tests**: Required. The feature changes runtime-visible state, action handling, release metadata, and terminal behavior.
**Coverage Target**: Modified Rust files must reach at least 95% patch coverage before completion.
**Design Guardrails**: Before editing implementation files, preserve separation of concerns, keep non-test Rust files at or below 500 lines unless they already exceed that limit, and do not make oversized files larger except for thin command wiring. Prefer small domain, adapter, render, input, and launcher modules over a generic dashboard framework.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other marked tasks in the same phase because it touches different files and has no dependency on incomplete task output.
- **[Story]**: User story label for story phases only.
- Every checklist item includes at least one exact file path.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish release metadata, workspace structure, dashboard crate boundaries, and repository-wide guardrails.

- [X] T001 Bump Boundline from `0.63.0` to `0.64.0` and Canon companion references from `0.59.0` to `0.60.0` in `/Users/rt/workspace/apply-the/boundline/Cargo.toml`, `/Users/rt/workspace/apply-the/boundline/distribution/channel-metadata.toml`, `/Users/rt/workspace/apply-the/boundline/distribution/homebrew/Formula/boundline.rb`, `/Users/rt/workspace/apply-the/boundline/assistant/global/manifest.json`, `/Users/rt/workspace/apply-the/boundline/README.md`, `/Users/rt/workspace/apply-the/boundline/ROADMAP.md`, and current-release docs under `/Users/rt/workspace/apply-the/boundline/docs/`
- [X] T002 Update release-facing dashboard narrative in `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md`, `/Users/rt/workspace/apply-the/boundline/README.md`, `/Users/rt/workspace/apply-the/boundline/docs/getting-started.md`, `/Users/rt/workspace/apply-the/boundline/docs/delivery-model.md`, `/Users/rt/workspace/apply-the/boundline/docs/architecture.md`, and `/Users/rt/workspace/apply-the/boundline/docs/configuration.md`
- [X] T003 Update the external wiki with dashboard usage, degraded behavior, release version, and Canon `0.60.0` pairing in `/Users/rt/workspace/apply-the/boundline.wiki/Home.md`, `/Users/rt/workspace/apply-the/boundline.wiki/Getting-Started.md`, `/Users/rt/workspace/apply-the/boundline.wiki/Daily-Operating-Guide.md`, `/Users/rt/workspace/apply-the/boundline.wiki/Reference.md`, `/Users/rt/workspace/apply-the/boundline.wiki/Troubleshooting.md`, and `/Users/rt/workspace/apply-the/boundline.wiki/Canon-Integration.md`
- [X] T004 Add `crates/boundline-dashboard` as a workspace member and add dashboard-local `ratatui` plus terminal backend dependencies in `/Users/rt/workspace/apply-the/boundline/Cargo.toml` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/Cargo.toml`
- [X] T005 [P] Scaffold the dashboard crate source modules in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/lib.rs`, `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/main.rs`, `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/app.rs`, `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/state.rs`, `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/render.rs`, `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/input.rs`, and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/branding.rs`
- [X] T006 [P] Create dashboard fixture directories and seed empty scenario files in `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/snapshots.json`, `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/panels.json`, and `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/actions.json`
- [X] T007 [P] Reconcile current public provider model docs against `/Users/rt/workspace/apply-the/boundline/assistant/catalog/model-catalog.toml` and record the applied delta or explicit no-change result in `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/quickstart.md`
- [X] T008 [P] Audit planned implementation file sizes and module boundaries before code edits, then keep dashboard logic out of oversized non-test files by using `/Users/rt/workspace/apply-the/boundline/src/domain/dashboard.rs`, `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`, `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs`, and the focused files under `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core contracts, test wiring, state projection boundaries, and dashboard shell required before any user story can be implemented.

**Checkpoint**: No user story work should begin until these tasks establish the shared dashboard model, adapter boundary, CLI launcher skeleton, crate entrypoint, and file-size guardrail.

- [X] T009 [P] Add reusable dashboard test fixture helpers in `/Users/rt/workspace/apply-the/boundline/tests/support/dashboard_fixture.rs`
- [X] T010 Register dashboard contract, integration, and unit test modules in `/Users/rt/workspace/apply-the/boundline/tests/contract.rs`, `/Users/rt/workspace/apply-the/boundline/tests/integration.rs`, and `/Users/rt/workspace/apply-the/boundline/tests/unit.rs`
- [X] T011 Create shared dashboard domain types for snapshots, sessions, events, panels, actions, degraded states, and brand marks in `/Users/rt/workspace/apply-the/boundline/src/domain/dashboard.rs`
- [X] T012 Export the shared dashboard domain module through `/Users/rt/workspace/apply-the/boundline/src/domain.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-core/src/domain.rs`
- [X] T013 Create the dashboard state adapter boundary that reads existing session, trace, checkpoint, finding, config, workflow, and optional governed-reference projections in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T014 Export the dashboard state adapter through `/Users/rt/workspace/apply-the/boundline/src/adapters.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-adapters/src/adapters.rs`
- [X] T015 Create the thin normal CLI dashboard launcher boundary without embedding TUI rendering logic in `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs`, `/Users/rt/workspace/apply-the/boundline/src/cli.rs`, and `/Users/rt/workspace/apply-the/boundline/crates/boundline-cli/src/cli.rs`
- [X] T016 Create the dashboard binary and library entrypoints that delegate to app/state/render/input modules in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/main.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/lib.rs`
- [X] T017 Implement a small app-state shell and terminal capability model in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/app.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/state.rs`
- [X] T018 Implement placeholder render, input, and branding module boundaries without full story behavior in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/render.rs`, `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/input.rs`, and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/branding.rs`
- [X] T019 [P] Add a non-test Rust file-size regression test for new dashboard modules in `/Users/rt/workspace/apply-the/boundline/tests/unit/dashboard_module_boundaries.rs`
- [X] T020 [P] Add baseline command contract fixtures for no-session, active-session, degraded, and unavailable-dashboard states in `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/snapshots.json`
- [X] T021 Verify foundational Rust code introduces no panic-prone control flow outside `main.rs`, including `#[cfg(test)]` modules and files under `/Users/rt/workspace/apply-the/boundline/tests/`, and verify stable dashboard serialization uses typed serde models plus named constants or enums in `/Users/rt/workspace/apply-the/boundline/src/domain/dashboard.rs`, `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`, and `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs`

---

## Phase 3: User Story 1 - Attach To Current Delivery State (Priority: P1)

**Goal**: An operator opens the dashboard in a workspace and immediately sees the current authoritative Boundline delivery state, next action, stop posture, and recent trace context.

**Independent Test**: Prepare active, empty, blocked, failed, exhausted, corrupt-session, multiple-session, stale-trace, and externally changed workspaces; compare the dashboard snapshot and first screen against normal `status` and `inspect` surfaces.

### Tests for User Story 1

- [X] T022 [P] [US1] Add snapshot schema and invalid-snapshot contract tests for active, no-session, blocked, failed, exhausted, degraded, multiple-session, and stale-trace states in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_snapshot_contract.rs`
- [X] T023 [P] [US1] Add command surface contract tests for `boundline-dashboard --snapshot-json` and `boundline dashboard` launch behavior in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_command_contract.rs`
- [X] T024 [P] [US1] Add integration tests comparing dashboard snapshots, current-session resolver output, and explicit refresh after external state changes with normal status and inspect summaries in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_snapshot_flow.rs`
- [X] T025 [P] [US1] Add integration tests for missing workspace, invalid session JSON, stale trace references, and no active session degraded states in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_degraded_state_flow.rs`

### Implementation for User Story 1

- [X] T026 [US1] Implement authoritative snapshot assembly and current-session resolver or selector for active session summary, route posture, stage, step, plan state, next action, ambiguity, and blocking reason in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T027 [US1] Implement runtime event projection for recent session and trace events without parsing human-readable output in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T028 [US1] Implement no-session, invalid-workspace, invalid-session, and trace-staleness degraded states with valid fallback commands in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T029 [US1] Implement first-screen summary, selected session identity, current stage, current step, execution condition, next action, blocking reason, and timeline rendering in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/render.rs`
- [X] T030 [US1] Implement `boundline-dashboard --workspace <path> --snapshot-json` serialization in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/main.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/app.rs`
- [X] T031 [US1] Implement the `boundline dashboard --workspace <path> --no-color` launcher and fallback messages in `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs` and `/Users/rt/workspace/apply-the/boundline/src/cli.rs`

**Checkpoint**: User Story 1 is independently usable when snapshot JSON and the first operational screen match normal command truth for ready and non-success states.

---

## Phase 4: User Story 2 - Inspect Plans, Evidence, And Findings (Priority: P2)

**Goal**: An operator can inspect the plan, selected evidence, context-pack facts, degraded context, stop rules, findings, checkpoints, dashboard diagnostics, and read-only governed references behind the current next action.

**Independent Test**: Prepare sessions with goal plans, selected evidence, context-pack reason/source/budget/authority fields, guidance or guardian findings, diagnostics, checkpoints, and optional governed references; verify each panel distinguishes available empty data from unavailable data and never mutates governed artifacts.

### Tests for User Story 2

- [X] T032 [P] [US2] Add inspection panel contract tests for goal plan, evidence, context-pack reason/source/budget/authority fields, context degradation, stop rules, findings, checkpoints, diagnostics, and governed references in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_inspection_contract.rs`
- [X] T033 [P] [US2] Add integration tests for panel data sourced from existing session, trace, context-pack, checkpoint, guidance, guardian, review, and diagnostic projections in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_inspection_flow.rs`
- [X] T034 [P] [US2] Add fixture coverage for context-pack reason/source/budget/authority, diagnostics, findings, checkpoints, unavailable governed references, and read-only governed references in `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/panels.json`

### Implementation for User Story 2

- [X] T035 [US2] Extend dashboard domain panel models and unavailable-state enums for context-pack facts, diagnostics, and governed references in `/Users/rt/workspace/apply-the/boundline/src/domain/dashboard.rs`
- [X] T036 [US2] Assemble goal plan, selected evidence, context-pack facts, context degradation, stop-rule, finding, checkpoint, and diagnostic panels from existing Boundline projections in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T037 [US2] Assemble optional governed-reference panel items as read-only facts with readiness, provenance, approval cues, and unavailable-state reasons in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T038 [US2] Render plan, context-pack, evidence, findings, checkpoint, diagnostics, and governed-reference panels with clear `none` versus `unavailable` states in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/render.rs`
- [X] T039 [US2] Implement keyboard focus and panel navigation without triggering runtime actions in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/input.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/app.rs`
- [X] T040 [US2] Add a regression check that dashboard inspection never writes under `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/panels.json`-modeled governed references or any workspace `.canon/` path in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_inspection_flow.rs`

**Checkpoint**: User Story 2 is independently testable when all inspection panels render from existing state and governed references remain optional and read-only.

---

## Phase 5: User Story 3 - Act Through Existing Runtime Boundaries (Priority: P3)

**Goal**: An operator can confirm, reject, replan, recover, launch, or continue from the dashboard while producing the same state transitions and trace evidence as normal Boundline commands.

**Independent Test**: Run each supported dashboard action and the equivalent normal command on equivalent prepared workspaces; compare resulting session state, traces, next actions, and refusal behavior.

### Tests for User Story 3

- [X] T041 [P] [US3] Add action request, result, and refusal contract tests for confirm, reject, replan, recover, launch, continue, and inspect-only actions in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_action_contract.rs`
- [X] T042 [P] [US3] Add successful dashboard action integration tests for confirm, reject with reason, replan, recover, launch, and continue flows in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_action_flow.rs`
- [X] T043 [P] [US3] Add stale revision, invalid workspace, missing context, stop-rule, approval-waiting, unsupported-action, and runtime-unavailable refusal tests in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_action_refusal_flow.rs`
- [X] T044 [P] [US3] Add action fixtures for proposed, confirmed, failed, blocked, exhausted, stale, and launchable sessions in `/Users/rt/workspace/apply-the/boundline/tests/fixtures/064-interactive-delivery-dashboard/actions.json`

### Implementation for User Story 3

- [X] T045 [US3] Implement dashboard action option, request, result, validation, and refusal models with typed action kinds in `/Users/rt/workspace/apply-the/boundline/src/domain/dashboard.rs`
- [X] T046 [US3] Implement action dispatch through existing Boundline runtime boundaries, not direct dashboard-owned file mutation, in `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs` and `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T047 [US3] Implement target session revision validation and fail-closed stale action handling in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`
- [X] T048 [US3] Implement dashboard input handling for confirm, reject reason capture, replan, recover, launch, continue, and inspect-only focus changes in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/input.rs`
- [X] T049 [US3] Refresh snapshots after applied actions, refused actions, explicit refresh requests, and detected external state changes while surfacing next valid action messages without triggering autonomous progression in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/app.rs` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/state.rs`
- [X] T050 [US3] Enforce one active dashboard action at a time with no background workers, hidden fan-out, or queued autonomous progression in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/app.rs`

**Checkpoint**: User Story 3 is independently testable when dashboard actions and normal commands produce equivalent session and trace outcomes.

---

## Phase 6: User Story 4 - Ship As One Complete Release-Aligned Feature (Priority: P4)

**Goal**: A maintainer can release the dashboard as a complete capability with aligned docs, version metadata, distribution metadata, wiki pages, assistant guidance, validation evidence, and terminal-safe branding.

**Independent Test**: Follow the updated docs on representative workspaces, run release validation, confirm degraded mode stays useful, and verify repository docs and generated feature artifacts do not use internal roadmap code names.

### Tests for User Story 4

- [X] T051 [P] [US4] Add render contract tests for interactive, compact, monochrome, degraded, no-color, and narrow-terminal modes in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_render_contract.rs`
- [X] T052 [P] [US4] Add branding contract tests for the simple colored `boundline` wordmark, plain fallback, and no image or SVG dependency in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_branding_contract.rs`
- [X] T053 [P] [US4] Add dashboard availability, dashboard-oriented diagnostics, and fallback command contract tests in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_command_contract.rs`
- [X] T054 [P] [US4] Add degraded terminal, non-interactive, too-narrow, and dashboard-unavailable integration tests in `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_degraded_flow.rs`
- [X] T055 [P] [US4] Add release documentation contract tests for Boundline `0.64.0`, Canon `0.60.0`, dashboard docs, and internal roadmap code-name absence in `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_release_docs_contract.rs`

### Implementation for User Story 4

- [X] T056 [US4] Implement the terminal-safe colored `boundline` wordmark and plain fallback in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/branding.rs`
- [X] T057 [US4] Implement compact, monochrome, degraded, no-color, narrow-terminal, and height-constrained render modes in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/render.rs`
- [X] T058 [US4] Implement dashboard-oriented diagnostics plus dashboard-unavailable and runtime-command-unavailable launcher messages with valid normal command fallbacks in `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs`
- [X] T059 [US4] Document dashboard launch, snapshot JSON, actions, panels, degraded mode, and branding boundaries in `/Users/rt/workspace/apply-the/boundline/README.md`, `/Users/rt/workspace/apply-the/boundline/docs/getting-started.md`, `/Users/rt/workspace/apply-the/boundline/docs/delivery-model.md`, `/Users/rt/workspace/apply-the/boundline/docs/architecture.md`, and `/Users/rt/workspace/apply-the/boundline/docs/release-checklist.md`
- [X] T060 [US4] Update assistant host guidance only where dashboard command surfaces affect operator workflows in `/Users/rt/workspace/apply-the/boundline/assistant/README.md`, `/Users/rt/workspace/apply-the/boundline/assistant/codex/commands/boundline-status.md`, `/Users/rt/workspace/apply-the/boundline/assistant/codex/commands/boundline-inspect.md`, `/Users/rt/workspace/apply-the/boundline/assistant/claude/commands/boundline-status.md`, and `/Users/rt/workspace/apply-the/boundline/assistant/claude/commands/boundline-inspect.md`
- [X] T061 [US4] Validate release metadata and wiki updates for the complete dashboard capability in `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md`, `/Users/rt/workspace/apply-the/boundline/ROADMAP.md`, `/Users/rt/workspace/apply-the/boundline/distribution/channel-metadata.toml`, `/Users/rt/workspace/apply-the/boundline/distribution/homebrew/Formula/boundline.rb`, and `/Users/rt/workspace/apply-the/boundline.wiki/Home.md`

**Checkpoint**: User Story 4 is complete when the feature is releasable as one coherent dashboard capability and normal command surfaces remain fully usable.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Repository-wide verification, coverage closure, lint cleanup, docs consistency, and final safety checks.

- [X] T062 Run `cargo fmt --check` from `/Users/rt/workspace/apply-the/boundline` and fix formatting in all modified Rust files under `/Users/rt/workspace/apply-the/boundline/src/`, `/Users/rt/workspace/apply-the/boundline/crates/`, and `/Users/rt/workspace/apply-the/boundline/tests/`
- [X] T063 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` from `/Users/rt/workspace/apply-the/boundline` and resolve every clippy issue in modified Rust files under `/Users/rt/workspace/apply-the/boundline/src/`, `/Users/rt/workspace/apply-the/boundline/crates/`, and `/Users/rt/workspace/apply-the/boundline/tests/`
- [X] T064 Run `cargo test` and `cargo nextest run` from `/Users/rt/workspace/apply-the/boundline` and fix failures in dashboard contract, integration, and unit tests under `/Users/rt/workspace/apply-the/boundline/tests/`
- [X] T065 Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` from `/Users/rt/workspace/apply-the/boundline`, use `/Users/rt/workspace/apply-the/boundline/scripts/common/coverage/intersect_patch_coverage.py`, and raise modified Rust file patch coverage to at least 95% in `/Users/rt/workspace/apply-the/boundline/lcov.info`
- [X] T066 Run `cargo deny check licenses advisories bans sources` from `/Users/rt/workspace/apply-the/boundline` and resolve dashboard dependency or distribution policy issues in `/Users/rt/workspace/apply-the/boundline/Cargo.toml` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/Cargo.toml`
- [X] T067 Run a final non-test Rust file-size audit from `/Users/rt/workspace/apply-the/boundline` and refactor any new or worsened over-500-line dashboard implementation file under `/Users/rt/workspace/apply-the/boundline/src/` or `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/`
- [X] T068 Validate release text has no internal roadmap code names and has aligned Boundline `0.64.0` plus Canon `0.60.0` references across `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/`, `/Users/rt/workspace/apply-the/boundline/README.md`, `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md`, `/Users/rt/workspace/apply-the/boundline/ROADMAP.md`, `/Users/rt/workspace/apply-the/boundline/docs/`, `/Users/rt/workspace/apply-the/boundline/assistant/`, `/Users/rt/workspace/apply-the/boundline/distribution/`, and `/Users/rt/workspace/apply-the/boundline.wiki/`
- [X] T069 Execute the validation scenarios in `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/quickstart.md` for active, no-session, proposed-plan, blocked, failed, exhausted, multiple-session, stale-trace, externally changed, governed-reference, diagnostics, degraded, color, and no-color dashboard states
- [X] T070 Capture final validation evidence and unresolved release caveats in `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/quickstart.md` and `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md`
- [X] T071 Validate first dashboard render under 1 second and local refresh under 1 second on representative workspaces, excluding underlying command runtime, and record the evidence in `/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately. T001 must be first because release version and Canon companion references frame all subsequent docs and release checks.
- **Foundational (Phase 2)**: Depends on Setup completion. Blocks all user story implementation.
- **User Story 1 (Phase 3)**: Depends on Foundational. Provides the MVP dashboard snapshot and first screen.
- **User Story 2 (Phase 4)**: Depends on Foundational and can proceed after the shared snapshot model exists; it should not require User Story 3 actions.
- **User Story 3 (Phase 5)**: Depends on Foundational and should reuse User Story 1 snapshots for refresh and refusal display.
- **User Story 4 (Phase 6)**: Depends on the implemented stories it documents, but render and branding tests can start once Foundational is complete.
- **Polish (Phase 7)**: Depends on all targeted user stories and release documentation updates.

### User Story Dependencies

- **US1 Attach To Current Delivery State**: MVP. No dependency on other stories after Foundational.
- **US2 Inspect Plans, Evidence, And Findings**: Depends on shared snapshot and panel model foundations; independent from mutating actions.
- **US3 Act Through Existing Runtime Boundaries**: Depends on shared snapshot/action models and should refresh through US1 projection logic.
- **US4 Ship As One Complete Release-Aligned Feature**: Depends on the implemented dashboard behavior and closes docs, distribution, wiki, assistant guidance, validation, and branding.

### Within Each User Story

- Validation tasks come before implementation tasks.
- Domain models come before adapters.
- Adapters come before CLI launcher and TUI rendering that consume them.
- Action dispatch must use existing Boundline runtime behavior before dashboard input handling is considered complete.
- Documentation and release claims must wait until the corresponding behavior and tests exist.

---

## Parallel Opportunities

- **Setup**: T005, T006, T007, and T008 can run in parallel after T001 is understood because they touch independent crate, fixture, catalog, and audit surfaces.
- **Foundational**: T009, T019, and T020 can run in parallel with module skeleton work because they touch test support, unit tests, and fixtures.
- **US1**: T022, T023, T024, and T025 can run in parallel as independent failing tests before snapshot assembly.
- **US2**: T032, T033, and T034 can run in parallel across contract tests, integration tests, and fixtures.
- **US3**: T041, T042, T043, and T044 can run in parallel across action contracts, action success flows, refusal flows, and fixtures.
- **US4**: T051, T052, T053, T054, and T055 can run in parallel across render, branding, command, degraded, and release-doc contracts.
- **Final**: T062 through T071 are ordered by feedback loop cost; run formatting and linting before coverage closure.

---

## Parallel Example: User Story 1

```bash
# Launch independent User Story 1 validation work:
Task: "T022 dashboard snapshot contract tests"
Task: "T023 dashboard command contract tests"
Task: "T024 dashboard snapshot integration flow"
Task: "T025 dashboard degraded-state integration flow"
```

## Parallel Example: User Story 2

```bash
# Launch independent User Story 2 validation and fixture work:
Task: "T032 dashboard inspection contract tests"
Task: "T033 dashboard inspection integration flow"
Task: "T034 dashboard panel fixtures"
```

## Parallel Example: User Story 3

```bash
# Launch independent User Story 3 validation and fixture work:
Task: "T041 dashboard action contract tests"
Task: "T042 dashboard action success flow"
Task: "T043 dashboard action refusal flow"
Task: "T044 dashboard action fixtures"
```

## Parallel Example: User Story 4

```bash
# Launch independent User Story 4 validation work:
Task: "T051 dashboard render contract tests"
Task: "T052 dashboard branding contract tests"
Task: "T053 dashboard command fallback tests"
Task: "T054 dashboard degraded integration flow"
Task: "T055 dashboard release docs contract tests"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 for snapshot assembly, degraded states, `--snapshot-json`, and first-screen rendering.
3. Stop and validate User Story 1 with `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_snapshot_contract.rs`, `/Users/rt/workspace/apply-the/boundline/tests/contract/dashboard_command_contract.rs`, and `/Users/rt/workspace/apply-the/boundline/tests/integration/dashboard_snapshot_flow.rs`.

### Incremental Delivery

1. Add User Story 1 to make the dashboard trustworthy as a read surface.
2. Add User Story 2 to make plans, context-pack facts, evidence, findings, diagnostics, checkpoints, and governed references inspectable.
3. Add User Story 3 to close the operator action loop through existing runtime semantics.
4. Add User Story 4 to close release alignment, degraded behavior, branding, assistant guidance, wiki, and distribution metadata.
5. Complete Phase 7 only after all target stories pass.

### Separation Of Concern Strategy

1. Keep stable data contracts in `/Users/rt/workspace/apply-the/boundline/src/domain/dashboard.rs`.
2. Keep authoritative state assembly in `/Users/rt/workspace/apply-the/boundline/src/adapters/dashboard_state.rs`.
3. Keep normal command launch and fallback behavior in `/Users/rt/workspace/apply-the/boundline/src/cli/dashboard.rs`.
4. Keep interactive rendering and input in `/Users/rt/workspace/apply-the/boundline/crates/boundline-dashboard/src/`.
5. Split modules before any non-test Rust file crosses 500 lines; tests are excluded from the line-count limit but should still stay readable.

### Release Closure

1. Version and Canon companion references align first.
2. Docs, wiki, assistant guidance, distribution metadata, and changelog reflect the same shipped capability.
3. Formatting, clippy, test suite, dependency policy, file-size guardrails, forbidden internal code-name scan, quickstart scenarios, local performance validation, and 95% modified-file patch coverage all pass before completion.
