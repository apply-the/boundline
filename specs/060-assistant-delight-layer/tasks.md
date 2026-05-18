# Implementation Tasks: S7 Assistant Delight Layer

**Feature Branch**: `060-assistant-delight-layer`  
**Feature Name**: S7 Assistant Delight Layer  
**Date**: 2026-05-17  
**Total Tasks**: 32  
**MVP Scope**: User Story 1 (`why`, `risk`, `evidence`, `next-best`) on the
active session-native runtime

---

## Overview

This feature implements the active S7 roadmap slice in Boundline itself. It
adds assistant-facing cognitive affordance commands, CLI-backed inspect lenses,
zero-config advisory fallbacks, capability disclosure, and package metadata
updates while consuming Canon-governed input only through Canon's
`057-s7-delight-provider` contract.

**Key Deliverables**:
- Runtime-backed S7 assistant commands across existing Boundline host packages
- CLI-backed and inspect-backed explanation lenses for risk, assumptions,
  evidence, blockers, and next
- Explicit fallback and degradation disclosure when Canon or advanced-context
  inputs are unavailable
- Cross-repo consumer alignment with Canon 057 provider semantics
- Coverage and docs updates for the real S7 runtime slice

---

## Phase 0: Feature Identity & Alignment

**Purpose**: Keep 060 focused on the Boundline implementation rather than the
Canon-side contract definition.

- [x] T001 Rename the active Boundline branch and feature directory to `060-assistant-delight-layer` and update the core feature titles in `specs/060-assistant-delight-layer/spec.md`, `specs/060-assistant-delight-layer/plan.md`, and `specs/060-assistant-delight-layer/tasks.md`
- [x] T002 Update `specs/060-assistant-delight-layer/research.md`, `specs/060-assistant-delight-layer/data-model.md`, `specs/060-assistant-delight-layer/quickstart.md`, and the local support references so they describe the Boundline runtime implementation and Canon 057 dependency instead of a standalone contract-delivery feature
- [x] T003 Create `specs/060-assistant-delight-layer/validation-report.md` for Boundline-side runtime evidence, Canon alignment notes, and final release checks

---

## Phase 1: Shared Validation Scaffolding

**Purpose**: Add the failing tests and validation surfaces that define the S7 runtime behavior before implementation.

- [x] T004 Add failing command-definition assertions for `/boundline:why`, `/boundline:risk`, `/boundline:evidence`, and `/boundline:next-best` in `tests/contract/assistant_command_definition_contract.rs` and `tests/contract/assistant_command_pack_contract.rs`
- [x] T005 [P] Add failing source-attribution, fallback-disclosure, and next-safe-action assertions in `tests/contract/host_command_output_contract.rs` and `tests/contract/trace_summary_contract.rs`
- [x] T006 [P] Add failing integration coverage for partial-setup advisory answers and Canon-aware explanation flow in `tests/integration/host_trace_runtime_flow.rs` and `tests/integration/cli_trace_inspection.rs`

**Checkpoint**: The command registry, output contracts, and runtime flow tests fail for the missing S7 behavior.

---

## Phase 2: User Story 1 - Fast Explanations On Real Runtime State (Priority: P1) 🎯 MVP

**Goal**: Ship runtime-backed `why`, `risk`, `evidence`, and `next-best`
commands that read the active Boundline session and traces, remain useful on
partial setup, and disclose Canon gaps explicitly.

**Independent Test**: The new command assets and CLI-backed inspect output make
`why`, `risk`, `evidence`, and `next-best` usable against the active session,
including missing-Canon and blocked-state scenarios.

### Validation for User Story 1 (MANDATORY)

- [x] T007 [P] [US1] Add failing unit coverage for `why` and `risk` summary rendering in `tests/unit/cli_output.rs` and `tests/unit/session_cli_runtime.rs`
- [x] T008 [US1] Record the Boundline↔Canon source-bucket rules and fallback wording in `specs/060-assistant-delight-layer/validation-report.md`

### Implementation for User Story 1

- [x] T009 [US1] Add `/boundline:why`, `/boundline:risk`, `/boundline:evidence`, and `/boundline:next-best` definitions to `assistant/commands/session-workflow.json` and `assistant/plugin-metadata.json`
- [x] T010 [P] [US1] Create host prompts for the MVP commands in `assistant/claude/commands/boundline-why.md`, `assistant/codex/commands/boundline-why.md`, `assistant/copilot/prompts/boundline-why.prompt.md`, and the matching `risk`, `evidence`, and `next-best` files beside them
- [x] T011 [US1] Extend runtime-backed explanation rendering in `src/cli/inspect.rs` and `src/cli/output.rs` so `why`, `risk`, `evidence`, and `next-best` can be produced from active session and trace authority
- [x] T012 [P] [US1] Add explicit Canon-gap, missing-evidence, confidence, and blocked-state disclosure text in `src/cli/output.rs` and `src/cli/diagnostics.rs`
- [x] T013 [US1] Update `assistant/README.md` and `specs/060-assistant-delight-layer/quickstart.md` with the MVP command flow and zero-config fallback examples
- [x] T014 [US1] Capture passing MVP evidence in `specs/060-assistant-delight-layer/validation-report.md`

**Checkpoint**: S7 delivers a first useful answer path on the active session-native runtime.

---

## Phase 3: User Story 2 - Deep Cognitive Affordances Without Hidden Magic (Priority: P2)

**Goal**: Add assumptions, hidden-impact, challenge, and explain-plan surfaces
that stay source-attributed, use advanced-context only when available, and keep
governance visible.

**Independent Test**: Operators can inspect assumptions, hidden impact,
challenge output, and human-facing plan explanations from runtime-backed views
without losing fallback disclosure.

### Validation for User Story 2 (MANDATORY)

- [x] T015 [P] [US2] Add failing render and grouping coverage for assumptions, hidden-impact, challenge, and explain-plan in `tests/unit/cli_output.rs` and `tests/unit/context_intelligence_projection.rs`
- [x] T016 [US2] Add failing integration coverage for advanced-context-present and advanced-context-missing reasoning flows in `tests/integration/context_intelligence_flow.rs` and `tests/integration/context_intelligence_semantic_fallback.rs`

### Implementation for User Story 2

- [x] T017 [US2] Add `/boundline:assumptions`, `/boundline:hidden-impact`, `/boundline:challenge`, and `/boundline:explain-plan` to `assistant/commands/session-workflow.json` and `assistant/plugin-metadata.json`
- [x] T018 [P] [US2] Create host prompts for those commands in `assistant/claude/commands/boundline-assumptions.md`, `assistant/codex/commands/boundline-assumptions.md`, `assistant/copilot/prompts/boundline-assumptions.prompt.md`, and the matching `hidden-impact`, `challenge`, and `explain-plan` files beside them
- [x] T019 [US2] Extend `inspect` and operator-facing rendering in `src/cli/inspect.rs` and `src/cli/output.rs` to support assumptions, risk, evidence, blockers, next, hidden impact, challenge, and explain-plan lenses
- [x] T020 [P] [US2] Integrate advanced-context-aware fallback and governance-safe challenge output in `src/cli/inspect.rs`, `src/cli/output.rs`, and `src/cli/run.rs`
- [x] T021 [US2] Document the new inspect lenses and challenge semantics in `assistant/README.md` and `specs/060-assistant-delight-layer/validation-report.md`

**Checkpoint**: S7 exposes the roadmap's deeper cognitive affordances with visible fallback behavior.

---

## Phase 4: User Story 3 - Compact Assistant Surfaces And Context Diagnosis (Priority: P3)

**Goal**: Ship `doctor-context`, contextual command visibility, and host-package
alignment so S7 stays compact and actionable across assistant hosts.

**Independent Test**: The package metadata, bootstrap assets, and runtime-backed
doctor flow stay compact, contextual, and useful on incomplete setup.

### Validation for User Story 3 (MANDATORY)

- [x] T022 [P] [US3] Add failing tests for contextual command visibility and doctor-context support in `tests/unit/assistant_assets.rs` and `tests/contract/global_assistant_install_contract.rs`
- [x] T023 [US3] Add failing integration coverage for doctor-context and bootstrap fallback in `tests/integration/distribution_doctor_flow.rs` and `tests/integration/global_assistant_bootstrap.rs`

### Implementation for User Story 3

- [x] T024 [US3] Add `/boundline:doctor-context` plus contextual visibility metadata to `assistant/commands/session-workflow.json`, `assistant/plugin-metadata.json`, and `assistant/global/manifest.json`
- [x] T025 [P] [US3] Implement doctor-context diagnostics and actionable fix commands in `src/cli/diagnostics.rs`, `src/cli/output.rs`, and `src/cli/assistant_assets.rs`
- [x] T026 [US3] Update package docs and prompt-pack summaries to keep the default palette compact in `assistant/README.md`, `assistant/global/claude/README.md`, `assistant/global/codex/README.md`, `assistant/global/copilot/README.md`, `assistant/global/cursor/README.md`, `assistant/global/gemini/README.md`, and `assistant/prompts/copilot-command-pack.md`
- [x] T027 [US3] Capture package-surface and doctor-context evidence in `specs/060-assistant-delight-layer/validation-report.md`

**Checkpoint**: The S7 command surface is useful without becoming noisy.

---

## Final Phase: Canon Alignment, Docs, And Release Gates

**Purpose**: Close the feature with cross-repo validation and the usual Boundline quality gates.

- [x] T028 Add or extend consumer-side Canon 057 alignment assertions in `tests/contract/canon_runtime_contract.rs` and record the allowed Canon input classes and degradation semantics in `specs/060-assistant-delight-layer/validation-report.md`
- [x] T029 Update `README.md`, `ROADMAP.md`, `CHANGELOG.md`, and any remaining stale 060 path or contract-only references so repository docs describe the S7 assistant delight implementation and the Canon 057 dependency accurately
- [x] T030 Perform the independent Canon↔Boundline review after Canon 057 task `T031` completes and capture the evidence in `specs/060-assistant-delight-layer/validation-report.md`
- [x] T031 Review cyclomatic complexity and file length for modified Rust files, especially `src/cli/output.rs`, `src/cli/inspect.rs`, `src/cli/assistant_assets.rs`, and any added S7 test files; refactor hotspots without widening scope
- [x] T032 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`, raise modified Rust files to at least 95% line coverage, run `cargo fmt`, and append two candidate commit messages to `specs/060-assistant-delight-layer/validation-report.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 0** starts immediately and establishes the correct feature identity.
- **Phase 1** defines the failing tests and validation scaffolding for the real
  S7 runtime behavior.
- **Phase 2 (US1)** is the MVP and depends on the shared failing tests.
- **Phase 3 (US2)** depends on US1 because deeper commands build on the same
  output and source-attribution surfaces.
- **Phase 4 (US3)** depends on US1 for package registration and on the existing
  diagnostic flow for contextual setup checks.
- **Final Phase** depends on all chosen user stories and on Canon 057 reaching
  its review checkpoint for the independent cross-repo review.

### User Story Dependencies

- **US1 (P1)**: no story dependency beyond the shared validation scaffold.
- **US2 (P2)**: depends on US1's core explanation and output surfaces.
- **US3 (P3)**: depends on US1's command registration and output vocabulary.

### Within Each User Story

- Add failing tests before touching runtime or package implementation.
- Update assistant command metadata before host-specific prompt assets.
- Extend CLI/inspect rendering before documenting the command externally.
- Record validation evidence before moving to the next story.

---

## Parallel Opportunities

- `T005` and `T006` can run in parallel after `T004` because they validate
  different layers.
- In US1, `T010` and `T012` can run in parallel after `T009`.
- In US2, `T018` and `T020` can run in parallel after `T017`.
- In US3, `T025` and `T026` can run in parallel after `T024`.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Finish Phase 0 and Phase 1.
2. Deliver US1 as the first shippable S7 slice.
3. Verify that a partial-setup workspace still gets a useful `why` or `risk`
   answer.
4. Stop for review before layering advanced hidden-impact and package-context
   work.

### Incremental Delivery

1. Ship US1 for first-value explanations.
2. Add US2 for deeper cognition and inspect lenses.
3. Add US3 for contextual package polish and doctor-context.
4. Finish with Canon review, docs closeout, complexity review, and release
   gates.

### Cross-Repo Alignment Rule

Boundline 060 remains the consumer-side implementation. Canon 057 remains the
provider-side contract. If the Boundline implementation needs a new Canon input
class or different degradation semantics, Canon 057 must be amended first or in
lockstep, and T030 stays blocked until that review lands.
