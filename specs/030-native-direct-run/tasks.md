# Tasks: Native Direct Run

**Input**: Design documents from `/specs/030-native-direct-run/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes the
primary `run` route, direct session bootstrap behavior, compatibility routing,
session safety, and assistant-facing guidance. Coverage refresh for modified or
created Rust files is part of release closeout, and touched-Rust coverage must
remain above 95%.

**Organization**: Tasks are grouped by user story so native direct-run
bootstrapping, explicit compatibility preservation, and release closeout can be
implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.30.0` release boundary and register the focused
test surfaces for direct native run

- [X] T001 Bump crate version to `0.30.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [X] T002 Reuse the existing runtime-routing and direct-run test harness surfaces in `/Users/rt/workspace/boundline/tests/contract/runtime_routing_contract.rs`, `/Users/rt/workspace/boundline/tests/contract/compatibility_continuity_contract.rs`, `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_compat.rs`, `/Users/rt/workspace/boundline/tests/integration/fixture_compat_flow.rs`, `/Users/rt/workspace/boundline/tests/integration/session_compatibility_continuity.rs`, and `/Users/rt/workspace/boundline/tests/unit/coverage_additional.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared route-choice and diagnostics behavior required by every
story in this slice

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Add explicit direct-run route selection inputs to `/Users/rt/workspace/boundline/src/cli.rs` and `/Users/rt/workspace/boundline/src/cli/run.rs`
- [X] T004 [P] Add native direct-run bootstrap helpers that can create safe executable session state in `/Users/rt/workspace/boundline/src/cli/run.rs` and `/Users/rt/workspace/boundline/src/cli/session.rs`
- [X] T005 [P] Split native-ready versus compatibility-ready diagnostics in `/Users/rt/workspace/boundline/src/cli/diagnostics.rs`
- [X] T006 [P] Add foundational unit coverage for run validation and route selection changes in `/Users/rt/workspace/boundline/tests/unit/coverage_additional.rs` and `/Users/rt/workspace/boundline/tests/unit/cli_output.rs`

**Checkpoint**: Direct run can choose between native bootstrap and explicit compatibility, and diagnostics no longer force native direct run through the execution-profile gate.

---

## Phase 3: User Story 1 - Run A Goal Natively In One Command (Priority: P1) 🎯 MVP

**Goal**: Let operators use direct `run --goal` as a native session bootstrap
entry that reaches real workspace mutation and trace-backed follow-up in one
command.

**Independent Test**: Run `boundline run --workspace <workspace> --goal "Fix the failing add test"` in a representative Rust workspace with no active session and confirm native routing, changed files, validation, decisions, and persisted follow-up through `status` and `inspect`.

### Tests for User Story 1

- [X] T007 [P] [US1] Add contract and integration coverage for native direct-run routing in `/Users/rt/workspace/boundline/tests/contract/runtime_routing_contract.rs` and `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_compat.rs`
- [X] T008 [P] [US1] Add follow-up continuity coverage for direct native run in `/Users/rt/workspace/boundline/tests/integration/session_compatibility_continuity.rs` and `/Users/rt/workspace/boundline/tests/integration/fixture_compat_flow.rs`

### Implementation for User Story 1

- [X] T009 [US1] Bootstrap native session state for direct `run --goal` in `/Users/rt/workspace/boundline/src/cli/run.rs`
- [X] T010 [US1] Make direct native run produce an executable route by confirming inferred flows or choosing no-flow planning in `/Users/rt/workspace/boundline/src/cli/run.rs` and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`
- [X] T011 [US1] Reuse the persisted native session story on direct run output and later follow-up surfaces in `/Users/rt/workspace/boundline/src/cli/run.rs`, `/Users/rt/workspace/boundline/src/cli/session.rs`, and `/Users/rt/workspace/boundline/src/domain/session.rs`

**Checkpoint**: Direct `run --goal` uses the native goal-plan path by default and leaves `status`, `next`, and `inspect` aligned with the persisted native session.

---

## Phase 4: User Story 2 - Keep Compatibility Explicit And Session-Safe (Priority: P2)

**Goal**: Preserve the explicit compatibility route and prevent direct native
run from silently overwriting meaningful active session state.

**Independent Test**: Run direct `run --goal` against a workspace with active
session state and against one where compatibility is explicitly requested, then
verify that the operator gets either an explicit safety stop or explicit
compatibility-owned output.

### Tests for User Story 2

- [X] T012 [P] [US2] Add contract coverage for explicit compatibility opt-in and active-session protection in `/Users/rt/workspace/boundline/tests/contract/runtime_routing_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/compatibility_continuity_contract.rs`
- [X] T013 [P] [US2] Add integration coverage for direct-run session-protection and explicit compatibility behavior in `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_compat.rs` and `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`

### Implementation for User Story 2

- [X] T014 [US2] Add an explicit compatibility opt-in surface for `run` in `/Users/rt/workspace/boundline/src/cli.rs` and `/Users/rt/workspace/boundline/src/cli/run.rs`
- [X] T015 [US2] Block destructive direct-run bootstrap when meaningful active session state already exists in `/Users/rt/workspace/boundline/src/cli/run.rs` and `/Users/rt/workspace/boundline/src/adapters/session_store.rs`
- [X] T016 [US2] Keep compatibility ownership and diagnostics explicit after the route default changes in `/Users/rt/workspace/boundline/src/cli/run.rs`, `/Users/rt/workspace/boundline/src/cli/diagnostics.rs`, and `/Users/rt/workspace/boundline/src/cli/session.rs`

**Checkpoint**: Direct run is native-first, compatibility is deliberate, and active native session state is never overwritten silently.

---

## Phase 5: User Story 3 - Ship Native Direct Run As 0.30.0 (Priority: P3)

**Goal**: Ship runtime behavior, assistant guidance, docs, version metadata,
and release validation as one coherent `0.30.0` native direct-run story.

**Independent Test**: Follow the updated direct-run docs and assistant guidance
on a representative workspace, then confirm runtime output and release
validation match the native-first product story.

### Tests for User Story 3

- [X] T017 [P] [US3] Extend assistant and command-surface contract coverage for native direct run and explicit compatibility opt-in in `/Users/rt/workspace/boundline/tests/contract/assistant_command_definition_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/assistant_session_continuity_contract.rs`

### Implementation for User Story 3

- [X] T018 [US3] Update the native direct-run operator story and release notes in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/configuration.md`, `/Users/rt/workspace/boundline/docs/getting-started.md`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/ROADMAP.md`, and `/Users/rt/workspace/boundline/CHANGELOG.md`
- [X] T019 [US3] Update assistant guidance and generated agent context for native direct run and explicit compatibility opt-in in `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-status.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-next.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-inspect.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-status.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-next.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-inspect.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-status.prompt.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-next.prompt.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-inspect.prompt.md`, and `/Users/rt/workspace/boundline/AGENTS.md`

**Checkpoint**: Maintainers and assistants describe one coherent `0.30.0` direct-run-native story, with compatibility clearly subordinate and explicit.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout

- [X] T020 Run focused coverage for modified or created Rust files, refresh `/Users/rt/workspace/boundline/lcov.info`, verify touched-Rust coverage remains above 95%, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `/Users/rt/workspace/boundline/src/` and `/Users/rt/workspace/boundline/tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the runtime behavior delivered by US1.
- User Story 3 depends on Foundational and should reconcile with the settled runtime story from US1 and US2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the route-default behavior delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Shared route-choice and diagnostics behavior comes before output wording and docs.
- Runtime behavior should settle before assistant guidance and release docs are finalized.
- Compatibility ownership and session safety must remain explicit before story sign-off.

### Parallel Opportunities

- T004, T005, and T006 can run in parallel after T003.
- Test tasks within each user story marked `[P]` can run in parallel.
- T018 and T019 can run in parallel once runtime behavior is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract and integration coverage for native direct-run routing in tests/contract/runtime_routing_contract.rs and tests/integration/runtime_refoundation_compat.rs"
Task: "Add follow-up continuity coverage for direct native run in tests/integration/session_compatibility_continuity.rs and tests/integration/fixture_compat_flow.rs"

# Launch direct native-run implementation work together after route selection exists:
Task: "Bootstrap native session state for direct run in src/cli/run.rs"
Task: "Make direct native run produce an executable route by confirming inferred flows or choosing no-flow planning in src/cli/run.rs and src/orchestrator/session_runtime.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that direct `run --goal` now enters the native route and leaves native follow-up state behind.

### Incremental Delivery

1. Reserve `0.30.0` and register the focused direct-run routing test surfaces.
2. Add explicit route-choice inputs plus native-ready diagnostics.
3. Bootstrap native session state from direct `run --goal` and make it executable without pending flow-confirmation dead ends.
4. Preserve explicit compatibility execution and block unsafe session overwrite.
5. Ship docs, assistant guidance, roadmap, contributor guidance, changelog, and refreshed agent context.
6. Close with coverage refresh, touched-file coverage verification, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.30.0` as the first task.
- T018 intentionally includes impacted docs and changelog updates as one release-guidance task.
- T020 intentionally reserves the final coverage, touched-file coverage check, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.