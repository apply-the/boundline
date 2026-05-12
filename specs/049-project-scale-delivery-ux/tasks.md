# Tasks: Boundline Project-Scale Delivery UX

**Input**: Design documents from `/specs/049-project-scale-delivery-ux/`  
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/project-scale-delivery-contract.md](./contracts/project-scale-delivery-contract.md), [quickstart.md](./quickstart.md)

**Tests**: This feature changes runtime routing, governance surfaces, state projection, and docs. Test tasks are required before implementation work in each user story.

## Phase 1: Setup

**Purpose**: Establish version, catalog, fixtures, and validation baseline before feature work.

- [X] T001 Improve Boundline version from `0.49.1` to `0.50.0` across `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `assistant/plugin-metadata.json`, `assistant/catalog/model-catalog.toml`, `README.md`, `CHANGELOG.md`, and active distribution metadata under `distribution/winget/manifests/a/ApplyThe/Boundline/0.50.0/`.
- [X] T002 [P] Refresh provider model catalog evidence against public OpenAI, Anthropic, Google Gemini, and GitHub Copilot docs, then record the applied delta or no-change result in `assistant/catalog/model-catalog.toml` and `specs/049-project-scale-delivery-ux/research.md`.
- [X] T003 [P] Add project-scale delivery fixtures for uninitialized workspaces, Canon capability snapshots, high-risk stages, low-risk stages, and stale session state in `tests/fixtures/project_scale_delivery/`.
- [X] T004 [P] Add helper assertions for CLI/chat state parity, exact next-command guidance, and blocked-state summaries in `tests/support/project_scale_assertions.rs`.

---

## Phase 2: Foundational

**Purpose**: Add shared domain and state primitives required by every user story.

- [X] T005 [P] Add failing unit tests for the full Canon governed stage catalog in `tests/unit/governed_stage_catalog.rs`.
- [X] T006 [P] Add failing unit tests for project-scale path selection and bounded stage/work-unit decomposition in `tests/unit/project_scale_path_policy.rs`.
- [X] T007 [P] Add failing unit tests for voting trigger classification and skip rules in `tests/unit/voting_boundary_policy.rs`.
- [X] T008 Define governed stage catalog entries for every Canon `0.45.0` mode in `src/domain/governance.rs`.
- [X] T009 Define project-scale initiative, bounded stage, and bounded work unit state models in `src/domain/workflow.rs` and `src/domain/session.rs`.
- [X] T010 Define voting decision, reviewer finding, adjudication, and blocking-state models in `src/domain/review.rs` and `src/domain/session.rs`.
- [X] T011 Extend trace and checkpoint references for governed packet refs, voting refs, stage transitions, and context updates in `src/domain/trace.rs` and `src/orchestrator/review_trace.rs`.
- [X] T012 Extend CLI summary rendering for governance state, voting state, trace refs, checkpoint refs, and exact next commands in `src/cli/output.rs`.

**Checkpoint**: Shared stage, governance, voting, state, and output primitives are ready for story work.

---

## Phase 3: User Story 1 - Global Assistant Bootstrap (Priority: P1)

**Goal**: Make Boundline discoverable from supported host chat surfaces before repo-local initialization.

**Independent Test**: In a workspace with no `.boundline/`, global commands report readiness, avoid chat-history state inference, and provide exact CLI commands or execute the CLI where supported.

### Tests for User Story 1

- [X] T013 [P] [US1] Add contract tests for `boundline assistant install --host <host> --scope user` support and fallback behavior in `tests/contract/global_assistant_install_contract.rs`.
- [X] T014 [P] [US1] Add integration tests for `/boundline:init`, `/boundline:doctor`, `/boundline:status`, and `/boundline:continue` behavior in uninitialized workspaces in `tests/integration/global_assistant_bootstrap.rs`.
- [X] T015 [P] [US1] Add integration tests proving `/boundline:continue` does not infer state from chat history when `.boundline/session.json` is absent in `tests/integration/global_assistant_continue_no_state.rs`.

### Implementation for User Story 1

- [X] T016 [US1] Add `boundline assistant install --host <host> --scope user` command parsing and validation in `src/cli.rs` and `src/cli/assistant_assets.rs`.
- [X] T017 [US1] Add global assistant package metadata and command assets for Claude, Codex, Cursor, Copilot-style prompt packs, and Gemini-style fallbacks under `assistant/global/`.
- [X] T018 [US1] Extend diagnostics for Boundline install state, Canon pairing state, workspace readiness, and next repair/init command in `src/cli/diagnostics.rs`.
- [X] T019 [US1] Update repo-local assistant package generation to distinguish global and repo-local packages in `src/cli/init.rs` and `src/cli/assistant_assets.rs`.
- [X] T020 [US1] Document global package support, unsupported-host fallbacks, and bootstrap commands in `docs/guides/assistant-plugin-packages.md`, `docs/getting-started.md`, and `assistant/README.md`.

**Checkpoint**: User Story 1 is independently testable in an uninitialized workspace.

---

## Phase 4: User Story 2 - Idea-To-Code Delivery Path (Priority: P1)

**Goal**: Let Boundline propose and pilot broad initiatives as bounded stages and work units.

**Independent Test**: A broad idea produces a staged path, asks for confirmation on material transitions, stops on insufficient context, and does not claim one unchecked run can complete the initiative.

### Tests for User Story 2

- [X] T021 [P] [US2] Add integration tests for idea-to-code path proposal from a broad brief in `tests/integration/project_scale_idea_to_code.rs`.
- [X] T022 [P] [US2] Add integration tests for insufficient-context stop behavior and clarification next action in `tests/integration/project_scale_context_stop.rs`.
- [X] T023 [P] [US2] Add contract tests for initiative, stage, work-unit, checkpoint, validation, and trace state persistence in `tests/contract/project_scale_session_contract.rs`.

### Implementation for User Story 2

- [X] T024 [US2] Implement project-scale path proposal for discovery, requirements, system-shaping, architecture, backlog, implementation, verification, and review/pr-review in `src/orchestrator/decision_loop.rs`.
- [X] T025 [US2] Persist delivery initiative, confirmed path, active stage, bounded work unit, checkpoint refs, validation expectations, and trace refs in `src/orchestrator/session_runtime.rs`.
- [X] T026 [US2] Add stage-transition confirmation and boundary-exceeded stop behavior in `src/domain/workflow.rs` and `src/orchestrator/session_runtime.rs`.
- [X] T027 [US2] Surface project-scale path, active stage, active work unit, and next action in `src/cli/run.rs`, `src/cli/session.rs`, and `src/cli/inspect.rs`.
- [X] T028 [US2] Update assistant session workflow metadata so chat commands map to the same path, state, and next action in `assistant/commands/session-workflow.json` and `assistant/plugin-metadata.json`.

**Checkpoint**: User Stories 1 and 2 both work independently and preserve session-state authority.

---

## Phase 5: User Story 3 - Explicit Governed Stage Work (Priority: P2)

**Goal**: Route explicit governed stage work through `/boundline:govern` and a CLI equivalent for every current Canon mode.

**Independent Test**: `/boundline:govern` accepts, infers, or explicitly rejects every Canon mode based on capabilities, without manual JSON or manifest editing.

### Tests for User Story 3

- [X] T029 [P] [US3] Add contract tests for Canon `0.45.0` capability parsing and unavailable-mode rejection in `tests/contract/canon_capability_contract.rs`.
- [X] T030 [P] [US3] Add integration tests for `boundline govern --mode architecture`, `requirements`, `security-assessment`, `migration`, `supply-chain-analysis`, and `pr-review` in `tests/integration/boundline_govern_modes.rs`.
- [X] T031 [P] [US3] Add integration tests for `/boundline:govern` with no mode, inferred choices, missing input, approval-gated packet state, and incompatible Canon state in `tests/integration/boundline_govern_failures.rs`.

### Implementation for User Story 3

- [X] T032 [US3] Extend Canon capability command integration and mode validation in `src/adapters/governance_runtime.rs`.
- [X] T033 [US3] Implement the `boundline govern` CLI surface and mode inference in `src/cli.rs` and `src/cli/govern.rs`.
- [X] T034 [US3] Route governed stages through Boundline-owned stage boundaries and persist packet refs, provenance refs, approval state, readiness, and missing input in `src/orchestrator/governance.rs` and `src/orchestrator/session_runtime.rs`.
- [X] T035 [US3] Update `/boundline:govern` assistant command bindings and docs without promoting per-mode primary aliases in `assistant/codex/commands/boundline-govern.md`, `assistant/commands/session-workflow.json`, and `docs/guides/assistant-plugin-packages.md`.
- [X] T036 [US3] Surface governed stage refs and repair guidance in `status`, `next`, and `inspect` output in `src/cli/session.rs`, `src/cli/inspect.rs`, and `src/cli/output.rs`.

**Checkpoint**: User Story 3 can run or reject every requested Canon mode explicitly.

---

## Phase 6: User Story 4 - Voting At Risky Quality Boundaries (Priority: P2)

**Goal**: Trigger voting only at risky quality boundaries and project voting state through runtime summaries.

**Independent Test**: High-risk architecture, validation-exhausted implementation, PR-ready diffs, and material risk findings trigger voting; low-risk local work skips voting by default.

### Tests for User Story 4

- [X] T037 [P] [US4] Add integration tests for high-impact architecture and Type 1 decision voting triggers in `tests/integration/voting_architecture_boundary.rs`.
- [X] T038 [P] [US4] Add integration tests for validation-exhausted implementation voting and blocked continuation in `tests/integration/voting_validation_exhausted.rs`.
- [X] T039 [P] [US4] Add integration tests for PR-ready diff voting state and low-risk refactor skip behavior in `tests/integration/voting_pr_ready_and_skip.rs`.

### Implementation for User Story 4

- [X] T040 [US4] Implement risk and evidence voting trigger classification in `src/domain/review.rs`.
- [X] T041 [US4] Apply majority, weighted, reject-on-blocking, adjudication, override, and escalation outcomes in `src/orchestrator/review_trace.rs`.
- [X] T042 [US4] Persist latest voting state, reviewer findings, vote result, adjudication result, reviewed evidence packet, blocking status, and next action in `src/domain/session.rs` and `src/orchestrator/session_runtime.rs`.
- [X] T043 [US4] Render voting state and required next action in `status`, `next`, and `inspect` output in `src/cli/session.rs`, `src/cli/inspect.rs`, and `src/cli/output.rs`.
- [X] T044 [US4] Document voting triggers, skip rules, blocking behavior, and adjudication flow in `docs/review-voting.md` and `docs/architecture.md`.

**Checkpoint**: User Story 4 projects voting state consistently and keeps low-risk delivery unburdened.

---

## Phase 7: User Story 5 - Delivery Pilot Model Documentation (Priority: P3)

**Goal**: Explain project-scale delivery through bounded decomposition and the observe-decide-act-verify-update-context loop.

**Independent Test**: Docs include the Delivery Pilot Model, stop rules, project-scale example, and command mapping without implying unbounded autonomy.

### Tests for User Story 5

- [X] T045 [P] [US5] Add docs contract tests for the Delivery Pilot Model principle, loop terms, stop rules, and project-scale example in `tests/contract/delivery_model_docs_contract.rs`.
- [X] T046 [P] [US5] Add assistant docs validation for global/repo-local/CLI distinction and chat-to-CLI mapping in `tests/contract/assistant_delivery_docs_contract.rs`.

### Implementation for User Story 5

- [X] T047 [US5] Add `docs/delivery-model.md` with the Delivery Pilot Model, decomposition principle, observe-decide-act-verify-update-context loop, stop rules, and customer onboarding example.
- [X] T048 [US5] Update `README.md` with "Use Boundline from chat", "Use Boundline from CLI", and "How chat commands map to CLI/runtime state" sections.
- [X] T049 [US5] Cross-link the Delivery Pilot Model from `docs/architecture.md`, `docs/getting-started.md`, and `docs/guides/assistant-plugin-packages.md`.
- [X] T050 [US5] Update `assistant/README.md` and `assistant/prompts/starter-prompts.md` with project-scale starter prompts and explicit non-authoritative chat-state wording.

**Checkpoint**: User Story 5 is verifiable through documentation and assistant asset tests.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Validate cross-story consistency and close implementation quality gates.

- [X] T051 [P] Validate assistant plugin manifests, global package metadata, referenced paths, command names, unsupported-host claims, and version alignment with `scripts/validate-assistant-plugins.sh`.
- [X] T052 [P] Validate winget and distribution metadata no longer expose stale active-version guidance under `distribution/winget/manifests/a/ApplyThe/Boundline/` and `distribution/channel-metadata.toml`.
- [X] T053 Run the quickstart flows from `specs/049-project-scale-delivery-ux/quickstart.md` and record implementation evidence in `specs/049-project-scale-delivery-ux/validation-report.md`.
- [ ] T054 Add or adjust tests until every Rust file created or modified by this feature has at least 95% coverage, then run `cargo fmt`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, resolving all warnings/errors in the modified files.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately. `T001` must be first.
- **Foundational (Phase 2)**: Depends on Phase 1 and blocks all story implementation.
- **US1 and US2 (P1)**: Depend on Phase 2. They can proceed independently after shared primitives exist, but both are MVP-critical.
- **US3 and US4 (P2)**: Depend on Phase 2 and benefit from US2 session path/state support.
- **US5 (P3)**: Can begin after Phase 2 but should reconcile final command and state names after US1-US4 settle.
- **Final Phase**: Depends on all selected stories.

### User Story Dependencies

- **US1**: Independent bootstrap story; does not require active repo-local state.
- **US2**: Independent delivery path story; requires foundational stage/work-unit models.
- **US3**: Requires governed stage catalog and Canon capability primitives from Phase 2.
- **US4**: Requires voting models and session projection primitives from Phase 2.
- **US5**: Documents the product model and can be verified independently, but final wording must match implemented command surfaces.

### Parallel Opportunities

- T002, T003, and T004 can run in parallel after T001.
- T005, T006, and T007 can run in parallel.
- Test tasks within each user story can run in parallel.
- US1 asset work and US2 path-policy work can run in parallel after Phase 2.
- US3 Canon capability tests and US4 voting trigger tests can run in parallel after Phase 2.
- US5 docs can proceed in parallel with implementation once command names and state fields are stable.

## Parallel Execution Examples

```text
US1 parallel tests:
- T013 tests assistant install support and fallback behavior.
- T014 tests global bootstrap commands in uninitialized workspaces.
- T015 tests no-state continue behavior.

US3 parallel tests:
- T029 tests Canon capability parsing and mode rejection.
- T030 tests supported govern modes.
- T031 tests no-mode, missing-input, approval-gated, and incompatible paths.

US4 parallel tests:
- T037 tests architecture risk voting.
- T038 tests validation-exhausted voting.
- T039 tests PR-ready voting and low-risk skip behavior.
```

## Implementation Strategy

### MVP First

1. Complete Phase 1 and Phase 2.
2. Complete US1 so Boundline is discoverable from host chat before workspace init.
3. Complete US2 so broad ideas can become bounded staged paths.
4. Stop and validate uninitialized bootstrap plus idea-to-code pathing independently.

### Incremental Delivery

1. Add US3 to make `/boundline:govern` handle the full Canon mode set through Boundline.
2. Add US4 to enforce voting only at risky quality boundaries.
3. Add US5 to make the project-scale model clear in user-facing docs.
4. Finish with quickstart validation, manifest validation, clippy, tests, fmt, and 95% touched-Rust-file coverage.
