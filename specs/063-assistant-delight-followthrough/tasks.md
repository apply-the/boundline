# Tasks: S7.1 Assistant Delight Follow-Through

**Input**: Design documents from `/specs/063-assistant-delight-followthrough/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are required for this feature. In addition to story-specific contract and integration coverage, the final validation pass MUST refresh `lcov.info`, keep created or modified Rust files at or above 95% coverage, clear all `cargo clippy --workspace --all-targets --all-features -- -D warnings` findings, and preserve the repository rules against magic strings, duplication, and oversized functions or files.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`US1`, `US2`, `US3`)
- Every task below follows the required checklist format and includes exact file paths

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Release alignment, version preparation, and shared validation setup before code changes begin.

- [ ] T001 Bump workspace and distribution version surfaces in `Cargo.toml`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.62.0/ApplyThe.Boundline.yaml`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.62.0/ApplyThe.Boundline.installer.yaml`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.62.0/ApplyThe.Boundline.locale.en-US.yaml`
- [ ] T002 Re-check current provider docs against `assistant/catalog/model-catalog.toml` and record the explicit no-change or delta result in `specs/063-assistant-delight-followthrough/research.md`
- [ ] T003 [P] Prepare shared S7.1 validation notes in `specs/063-assistant-delight-followthrough/quickstart.md` and shared representative fixtures under `tests/fixtures/063-assistant-delight-followthrough/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared projection, fixture, and assistant-workflow primitives that MUST be complete before any user story work starts.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Create representative session and trace fixtures in `tests/fixtures/063-assistant-delight-followthrough/session.json` and `tests/fixtures/063-assistant-delight-followthrough/trace.json`
- [ ] T005 [P] Extend shared projection state and typed serialization in `src/domain/session.rs` and `src/domain/trace.rs` for inspect closures and delight feedback signals
- [ ] T006 [P] Extend shared reasoning disclosure helpers and named constants in `src/domain/reasoning.rs` and `src/cli/output.rs` to avoid new magic strings in S7.1 projections
- [ ] T007 [P] Align shared assistant workflow and host-support metadata in `assistant/commands/session-workflow.json` and `assistant/global/manifest.json` before story-specific host changes

**Checkpoint**: Foundation ready. User story work can begin with shared projection state, fixtures, and assistant metadata in place.

---

## Phase 3: User Story 1 - Reasoning-Profile-Aware Explanations (Priority: P1) 🎯 MVP

**Goal**: Make delight explanation surfaces disclose the active reasoning profile, why it was chosen, what it changed, and what fallback path was used when advanced reasoning is absent or degraded.

**Independent Test**: Activate a representative session with and without reasoning-profile support, run the explanation surfaces, and verify that the output discloses the active profile, selection rationale, contribution, and fallback wording without relying on chat history.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add contract coverage for reasoning-profile-aware delight fields in `tests/contract/s7_delight_projection_contract.rs`
- [ ] T009 [P] [US1] Add integration coverage for active reasoning-profile explanations in `tests/integration/s7_reasoning_profile_explanations.rs`
- [ ] T010 [P] [US1] Add integration coverage for degraded fallback explanations in `tests/integration/s7_reasoning_profile_explanations_degraded.rs`

### Implementation for User Story 1

- [ ] T011 [P] [US1] Extend profile-aware disclosure builders in `src/cli/output.rs` and `src/domain/reasoning.rs`
- [ ] T012 [US1] Surface active-profile contribution and fallback disclosure for `why`, `risk`, `evidence`, `challenge`, `hidden-impact`, and `explain-plan` in `src/cli/output.rs`
- [ ] T013 [US1] Align explanation command-pack guidance in `assistant/claude/commands/boundline-why.md`, `assistant/claude/commands/boundline-challenge.md`, `assistant/claude/commands/boundline-explain-plan.md`, `assistant/codex/commands/boundline-why.md`, `assistant/codex/commands/boundline-challenge.md`, `assistant/codex/commands/boundline-explain-plan.md`, `assistant/copilot/prompts/boundline-why.prompt.md`, `assistant/copilot/prompts/boundline-challenge.prompt.md`, and `assistant/copilot/prompts/boundline-explain-plan.prompt.md`

**Checkpoint**: User Story 1 is independently functional and testable through profile-aware explanation output and explicit fallback behavior.

---

## Phase 4: User Story 2 - Human-Facing Inspect Closure (Priority: P2)

**Goal**: Close the remaining inspect surfaces so operators can inspect context, council, and timeline directly from the authoritative session and trace state without reading raw payloads.

**Independent Test**: Run inspect on representative sessions that include normal progress, degradation, and review activity, then verify that `context`, `council`, and `timeline` views are human-facing, source-attributed, and preserve terminal or fallback semantics.

### Tests for User Story 2

- [ ] T014 [P] [US2] Add contract coverage for `inspect context`, `inspect council`, and `inspect timeline` output in `tests/contract/s7_inspect_closure_contract.rs`
- [ ] T015 [P] [US2] Add integration coverage for inspect context and inspect council views in `tests/integration/s7_inspect_closure_views.rs`
- [ ] T016 [P] [US2] Add integration coverage for blocked or degraded timeline output in `tests/integration/s7_inspect_closure_timeline.rs`

### Implementation for User Story 2

- [ ] T017 [P] [US2] Extend flattened trace inputs for context, council, and timeline closures in `src/domain/trace.rs` and `src/cli/inspect.rs`
- [ ] T018 [US2] Implement human-facing inspect context and inspect council render paths in `src/cli/inspect.rs` and `src/cli/output.rs`
- [ ] T019 [US2] Implement inspect timeline ordering for decision, review, governance, steps, and recovery in `src/cli/inspect.rs` and `src/cli/output.rs`
- [ ] T020 [US2] Preserve missing-state, terminal-state, and corrective guidance for new inspect closures in `src/cli/inspect.rs` and `tests/integration/s7_inspect_closure_timeline.rs`

**Checkpoint**: User Story 2 is independently functional and testable through the new inspect closure views.

---

## Phase 5: User Story 3 - Host Parity And Feedback Signals (Priority: P3)

**Goal**: Make Cursor and Gemini support modes explicit and expose lightweight delight usefulness signals without introducing a new telemetry system or a second runtime.

**Independent Test**: Review the generated host assets and run the feedback-signal scenarios to confirm that each host has a clear parity or fallback path and that session-scoped usefulness signals are inspectable through Boundline state.

### Tests for User Story 3

- [ ] T021 [P] [US3] Add contract coverage for host support modes in `tests/contract/assistant_command_pack_contract.rs` and `tests/contract/s7_assistant_host_parity_contract.rs`
- [ ] T022 [P] [US3] Add integration coverage for delight usefulness signal capture and projection in `tests/integration/s7_delight_feedback_signals.rs`
- [ ] T023 [P] [US3] Add integration coverage for Cursor and Gemini parity or fallback paths in `tests/integration/s7_host_parity_paths.rs`

### Implementation for User Story 3

- [ ] T024 [P] [US3] Add session-scoped delight feedback counters and validation in `src/domain/session.rs` and `src/domain/trace.rs`
- [ ] T025 [US3] Surface delight usefulness signal summaries through status and inspect output in `src/cli/output.rs` and `src/cli/inspect.rs`
- [ ] T026 [P] [US3] Encode Cursor and Gemini support-mode decisions in `assistant/global/manifest.json`, `assistant/global/cursor/README.md`, `assistant/global/gemini/README.md`, `assistant/gemini/README.md`, and `assistant/plugin-metadata.json`
- [ ] T027 [US3] Align repo-local assistant guidance with the chosen support modes in `assistant/README.md`, `assistant/commands/session-workflow.json`, `assistant/claude/commands/boundline-explain-plan.md`, `assistant/codex/commands/boundline-explain-plan.md`, and `assistant/copilot/prompts/boundline-explain-plan.prompt.md`

**Checkpoint**: User Story 3 is independently functional and testable through explicit host parity or fallback state plus inspectable usefulness signals.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release-facing docs, wiki alignment, coverage enforcement, and code-quality hardening across all stories.

- [ ] T028 [P] Update release-facing docs in `README.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/getting-started.md`, `docs/architecture.md`, and `docs/release-checklist.md`
- [ ] T029 [P] Update additional operator docs in `docs/reasoning-profile-algorithms.md`, `docs/runtime-confidence-and-calibration.md`, `docs/session-native-orchestrator-review.md`, `docs/degradation-and-escalation.md`, and `assistant/README.md`
- [ ] T030 [P] Update affected wiki pages in `../boundline.wiki/Home.md`, `../boundline.wiki/Assistant-Integrations.md`, `../boundline.wiki/Getting-Started.md`, `../boundline.wiki/Quick-Start.md`, `../boundline.wiki/Daily-Operating-Guide.md`, `../boundline.wiki/Troubleshooting.md`, `../boundline.wiki/Architecture-And-Decisions.md`, `../boundline.wiki/Canon-Integration.md`, `../boundline.wiki/Core-Concepts.md`, and `../boundline.wiki/Ubiquitous-Language.md`
- [ ] T031 Refactor touched Rust modules in `src/cli/output.rs`, `src/cli/inspect.rs`, `src/domain/session.rs`, `src/domain/trace.rs`, and `src/domain/reasoning.rs` to keep files and functions bounded and to remove duplication plus magic strings
- [ ] T032 Refresh focused coverage artifacts in `lcov.info` and `results.txt` using `tests/contract/*.rs`, `tests/integration/*.rs`, and `scripts/common/coverage/intersect_patch_coverage.py` so created or modified Rust files stay at or above 95% coverage
- [ ] T033 Run final quality gates against `Cargo.toml`, `clippy_output.txt`, `lcov.info`, and the touched Rust files with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, and targeted suites; clear all warnings and errors before sign-off

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup** has no dependencies and starts immediately.
- **Phase 2: Foundational** depends on Phase 1 and blocks all story work.
- **Phase 3: US1** depends on Phase 2.
- **Phase 4: US2** depends on Phase 2 and can proceed after the shared projection contracts are stable.
- **Phase 5: US3** depends on Phase 2 and can proceed after shared assistant metadata and session state are stable.
- **Phase 6: Polish** depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: starts after Foundational and is the recommended MVP slice.
- **US2 (P2)**: starts after Foundational; it reuses shared trace projections but remains independently testable from US1.
- **US3 (P3)**: starts after Foundational; it reuses shared session or assistant metadata but remains independently testable from US1 and US2.

### Recommended Completion Order

1. Phase 1 → Phase 2
2. User Story 1 (MVP)
3. User Story 2
4. User Story 3
5. Phase 6 polish, documentation, wiki, coverage, and final quality gates

---

## Parallel Opportunities

- `T003`, `T005`, `T006`, and `T007` can run in parallel after `T001` and `T002`.
- Within **US1**, `T008`, `T009`, and `T010` can run in parallel; `T011` can start once the shared models are stable.
- Within **US2**, `T014`, `T015`, and `T016` can run in parallel; `T017` can proceed in parallel with the contract work.
- Within **US3**, `T021`, `T022`, and `T023` can run in parallel; `T024` and `T026` can also proceed in parallel because they touch different surfaces.
- In **Phase 6**, `T028`, `T029`, and `T030` can run in parallel after the implementation stories are complete.

### Parallel Example: User Story 1

```bash
# Start the US1 validation work together:
T008 tests/contract/s7_delight_projection_contract.rs
T009 tests/integration/s7_reasoning_profile_explanations.rs
T010 tests/integration/s7_reasoning_profile_explanations_degraded.rs

# Then split the implementation work by surface:
T011 src/cli/output.rs src/domain/reasoning.rs
T013 assistant/claude/commands/boundline-why.md assistant/codex/commands/boundline-why.md assistant/copilot/prompts/boundline-why.prompt.md
```

### Parallel Example: User Story 2

```bash
# Start the inspect closure validation work together:
T014 tests/contract/s7_inspect_closure_contract.rs
T015 tests/integration/s7_inspect_closure_views.rs
T016 tests/integration/s7_inspect_closure_timeline.rs

# Then split data-shape and rendering work:
T017 src/domain/trace.rs src/cli/inspect.rs
T018 src/cli/inspect.rs src/cli/output.rs
```

### Parallel Example: User Story 3

```bash
# Start host and feedback validation together:
T021 tests/contract/assistant_command_pack_contract.rs tests/contract/s7_assistant_host_parity_contract.rs
T022 tests/integration/s7_delight_feedback_signals.rs
T023 tests/integration/s7_host_parity_paths.rs

# Then split runtime state from host-surface work:
T024 src/domain/session.rs src/domain/trace.rs
T026 assistant/global/manifest.json assistant/global/cursor/README.md assistant/global/gemini/README.md assistant/gemini/README.md assistant/plugin-metadata.json
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate US1 independently before moving on.

### Incremental Delivery

1. Setup + Foundational create the shared release and projection baseline.
2. US1 adds profile-aware explanation disclosure.
3. US2 closes inspect context, council, and timeline.
4. US3 adds explicit host parity or fallback state plus delight usefulness signals.
5. Phase 6 aligns release docs, roadmap, wiki, changelog, README, coverage, and quality gates.

### Notes

- The first implementation task is the requested version bump (`T001`).
- Documentation, roadmap, wiki, changelog, README, and additional doc updates are intentionally deferred to the final phase (`T028`-`T030`).
- Coverage enforcement is explicit in `T032`: modified or created Rust files must reach at least 95% coverage before sign-off.
- Code-quality enforcement is explicit in `T031` and `T033`: no new duplication, no new magic strings, no oversized functions or files, and no clippy warnings or errors.