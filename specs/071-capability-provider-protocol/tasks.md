# Tasks: External Capability Provider Protocol

**Input**: Design documents from `/specs/071-capability-provider-protocol/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/external-capability-provider-runtime-contract.md`,
`quickstart.md`

**Tests**: Test tasks are required. Add or refine focused regressions first,
confirm the relevant assertion fails before changing implementation, then close
the regression with the smallest coherent runtime change.

**Organization**: Tasks are grouped by user story so provider registration and
activation, bounded provider execution, and inspectable provider state can be
implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and has no
  dependency on incomplete work
- **[Story]**: Maps a task to a user story for traceability
- Every task includes repository-relative file paths

## Phase 1: Setup

**Purpose**: Lock the release boundary, reconfirm the provider-catalog
no-change result and Canon compatibility posture, and establish failing
release-surface regressions before runtime edits.

- [ ] T001 Reconfirm the 2026-06-05 official-provider no-change audit and the Canon `0.67.0` compatibility assumption in `specs/071-capability-provider-protocol/research.md`; if the slice still depends on Canon-owned schema changes or concrete provider implementations, stop and record the blocking gap before any runtime edits
- [ ] T002 [P] Add or refine release-surface and compatibility regressions for Boundline `0.72.0` and Canon `0.67.0` in `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, `tests/contract/distribution_release_surface_contract.rs`, and `tests/assistant_plugin_packages.rs`
- [ ] T003 [P] Add or refine assistant and host projection regressions for provider registration and failure-state continuity in `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_session_continuity_contract.rs`, and `tests/contract/host_command_output_contract.rs`

---

## Phase 2: Foundational

**Purpose**: Make the missing provider-protocol behavior fail in the smallest
high-signal places before user-story implementation work begins.

**Critical**: Complete this phase before user-story implementation.

- [ ] T004 [P] Add focused failing provider domain regressions for registration identity, activation-state transitions, permission-envelope validation, and fail-closed metadata conflicts in `tests/unit/capability_provider_model.rs` and `tests/unit/config_resolution.rs`
- [ ] T005 [P] Add focused failing runtime regressions for setup-requirement blocking, readiness blocking, prepare blocking, explicit permission admission failure, and non-authoritative patch-proposal handling in `tests/unit/session_cli_runtime.rs`, `tests/contract/capability_provider_protocol_contract.rs`, and `tests/integration/capability_provider_activation_flow.rs`
- [ ] T006 [P] Add focused failing projection regressions for provider status, capability identity, validation disposition, accepted and rejected evidence refs, limitations, and failure-class visibility in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: The feature now fails for the required provider protocol while
preserving backward-compatibility expectations.

---

## Phase 3: User Story 1 - Register And Activate A Provider Safely (Priority: P1) 🎯 MVP

**Goal**: Let operators explicitly register, inspect, and activate providers
without treating discovery as trust.

**Independent Test**: In an isolated temporary workspace, register one
provider, activate it successfully, interrupt activation of a replacement
provider, and confirm the previous active configuration remains authoritative.

### Implementation

- [ ] T007 [US1] Add typed provider-registration, transport-descriptor, setup-requirement, activation-state, and setup-result domain models plus stable constants in `src/domain/capability_provider.rs`
- [ ] T008 [US1] Extend configuration and auth-handle persistence for provider registrations and activation metadata in `src/domain/configuration.rs`, `src/adapters/config_store.rs`, and `src/adapters/auth_profile_store.rs`
- [ ] T009 [US1] Implement CLI command parsing and report surfaces for provider add, show, remove, and health, including missing setup-requirement projection before activation completes, in `src/cli.rs`, `src/cli/provider.rs`, and `src/cli/output_runtime.rs`
- [ ] T010 [US1] Implement transport-neutral `capabilities` and `health` admission flows plus atomic activation semantics and dry-run setup validation in `src/adapters/capability_provider_runtime/command.rs`, `src/adapters/capability_provider_runtime/http.rs`, and `src/orchestrator/capability_provider_runtime.rs`
- [ ] T011 [US1] Persist provider registration and activation projections into session-visible runtime state in `src/domain/session.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/session.rs`
- [ ] T012 [US1] Run the focused US1 regression set in `tests/unit/capability_provider_model.rs`, `tests/unit/config_resolution.rs`, `tests/integration/capability_provider_activation_flow.rs`, and `tests/contract/capability_provider_protocol_contract.rs`

**Checkpoint**: Provider discovery is no longer confused with activation, and
operators can safely register and activate providers with atomic rollback.

---

## Phase 4: User Story 2 - Execute Through A Permission-Scoped Provider (Priority: P2)

**Goal**: Route one bounded capability request through an activated provider
with explicit permissions, evidence collection, and non-authoritative output.

**Independent Test**: In an isolated fixture, execute one provider-backed
request, confirm the explicit permission envelope and expected outputs are
sent, and verify that unsupported patch proposals or missing evidence are
rejected without mutating Boundline-owned state.

### Implementation

- [ ] T013 [US2] Add typed prepare, execute, collect-evidence, permission-envelope, and validation-disposition models in `src/domain/capability_provider.rs` and supporting projection helpers in `src/domain/trace.rs`
- [ ] T014 [US2] Implement transport-neutral `prepare`, `execute`, and `collect_evidence` request/response handling in `src/adapters/capability_provider_runtime/command.rs`, `src/adapters/capability_provider_runtime/http.rs`, and `src/adapters/capability_provider_runtime.rs`
- [ ] T015 [US2] Integrate provider-backed admission, execution, and validation disposition into runtime orchestration in `src/orchestrator/capability_provider_runtime.rs`, `src/orchestrator/session_runtime.rs`, and `src/orchestrator/session_runtime_planning_runtime.rs`
- [ ] T016 [US2] Enforce explicit least-privilege permission admission, fail-closed metadata conflicts, and proposal-only patch semantics in `src/domain/capability_provider.rs`, `src/orchestrator/capability_provider_runtime.rs`, and `src/orchestrator/session_runtime_surface.rs`
- [ ] T017 [US2] Run the focused US2 regression set in `tests/unit/session_cli_runtime.rs`, `tests/contract/capability_provider_protocol_contract.rs`, and `tests/integration/capability_provider_execution_flow.rs`

**Checkpoint**: Provider-backed execution is bounded, permission-scoped, and
cannot directly mutate Boundline-owned state without validation.

---

## Phase 5: User Story 3 - Inspect Provider State, Limits, And Evidence (Priority: P3)

**Goal**: Surface provider identity, capability, permissions, evidence,
limitations, failure class, and validation outcome through standard runtime
and assistant projections.

**Independent Test**: In an isolated fixture, compare one accepted and one
blocked provider-backed request and confirm that `status`, `inspect`, host
JSON, and assistant assets all report the same provider state and failure
classification.

### Implementation

- [ ] T018 [P] [US3] Add additive provider projection, failure-class, and accepted-versus-rejected evidence summary fields to session and trace records in `src/domain/session.rs` and `src/domain/trace.rs`
- [ ] T019 [P] [US3] Render provider registration state, capability identity, validation disposition, accepted and rejected evidence refs, and limitations in `src/cli/output_host.rs`, `src/cli/output_runtime.rs`, `src/cli/output_session_status.rs`, and `src/cli/inspect/projections.rs`
- [ ] T020 [P] [US3] Align assistant plan, run, status, and inspect assets with the provider-protocol runtime contract in `assistant/antigravity/commands/boundline-plan.md`, `assistant/antigravity/commands/boundline-run.md`, `assistant/antigravity/commands/boundline-status.md`, `assistant/antigravity/commands/boundline-inspect.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, `assistant/claude/commands/boundline-inspect.md`, `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-inspect.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, and `assistant/copilot/prompts/boundline-inspect.prompt.md`
- [ ] T021 [US3] Implement specialized-profile overlay precedence without weakening generic runtime policy in `src/domain/capability_provider.rs`, `src/orchestrator/capability_provider_runtime.rs`, and `src/domain/framework_adapter.rs`
- [ ] T022 [US3] Run the focused US3 regression set in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_session_continuity_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`, explicitly validating accepted-versus-rejected evidence visibility and provider failure-class continuity

**Checkpoint**: Operators and assistant surfaces can inspect provider-backed
execution without opening raw provider payloads or losing failure clarity.

---

## Phase 6: Release, Documentation, and Quality Closure

**Purpose**: Close versioning, docs, roadmap, assistant assets, and quality
gates for release `0.72.0`.

- [ ] T023 [P] Refresh the provider-catalog audit result and explicit no-change rationale in `assistant/catalog/model-catalog.toml` and `specs/071-capability-provider-protocol/research.md`
- [ ] T024 [P] Document the provider protocol, explicit registration, activation, permissions, fail-closed conflict rule, and evidence handling in `README.md`, `docs/providers/overview.md`, `docs/providers/protocol.md`, `docs/providers/registration.md`, `docs/providers/troubleshooting.md`, `docs/runtime/run.md`, `docs/runtime/status.md`, `docs/runtime/inspect.md`, and `docs/runtime/trace.md`
- [ ] T025 [P] Update operator and architecture guidance for provider storage, auth handles, setup requirements, CLI usage, and Canon separation in `tech-docs/architecture.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `docs/reference/cli.md`, `docs/reference/configuration.md`, and `docs/guide/common-workflows.md`
- [ ] T026 [P] Bump release metadata to `0.72.0` and keep Canon `0.67.0` compatibility wording aligned in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `assistant/plugin-metadata.json`, `assistant/global/manifest.json`, `src/domain/distribution.rs`, and `tests/contract/canon_reasoning_posture_contract.rs`
- [ ] T027 [P] Add the `0.72.0` WinGet release manifests under `distribution/winget/manifests/a/ApplyThe/Boundline/0.72.0/`
- [ ] T028 [P] Record the delivered roadmap slice and release summary in `CHANGELOG.md`, `docs/roadmap/index.md`, `roadmap/Next - forward-roadmap.md`, `roadmap/features/README.md`, and `roadmap/joint-roadmap-graph.md`
- [ ] T029 Run `cargo fmt` and verify formatting with `cargo fmt --check`
- [ ] T030 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix every reported issue
- [ ] T031 Run focused tests with `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration capability_provider_activation_flow::`, `cargo test --test integration capability_provider_execution_flow::`, and `cargo test --test integration host_session_runtime_flow::`
- [ ] T032 Run assistant and release-surface regressions in `tests/assistant_plugin_packages.rs`, `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_session_continuity_contract.rs`, `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, and `tests/contract/distribution_release_surface_contract.rs`
- [ ] T033 Run the full regression suite with `cargo test` and resolve any failures
- [ ] T034 Generate `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- [ ] T035 Build an explicit repository-relative implementation-file list, run `scripts/common/coverage/intersect_patch_coverage.py` against every touched Rust implementation file, and add tests until changed-file coverage is at least 95 percent
- [ ] T036 Validate the isolated scenarios in `specs/071-capability-provider-protocol/quickstart.md`, including the under-30-second operator explanation check, without running Boundline CLI commands against the repository root

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Setup and blocks all story work.
- **User Story 1 (Phase 3)**: Depends on Foundational and is the MVP.
- **User Story 2 (Phase 4)**: Depends on the typed provider registration and
  activation contract from US1.
- **User Story 3 (Phase 5)**: Depends on the runtime-backed provider execution
  and validation disposition contract from US2.
- **Release and Quality Closure (Phase 6)**: Depends on all selected stories.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational and has no dependency on
  later stories.
- **User Story 2 (P2)**: Can start after US1 stabilizes because admission and
  execution depend on active provider registration and capability metadata.
- **User Story 3 (P3)**: Can start after US2 stabilizes because inspectability
  must project final execution and validation state.

### Parallel Opportunities

- T002 and T003 can run in parallel.
- T004, T005, and T006 can run in parallel.
- T018, T019, and T020 can run in parallel after US2 stabilizes.
- T023 through T028 can run in parallel after runtime behavior stabilizes.

---

## Parallel Example: User Story 1

```bash
# Launch the core failing registration and activation regressions together:
Task: "Add focused failing provider domain regressions in tests/unit/capability_provider_model.rs and tests/unit/config_resolution.rs"
Task: "Add focused failing runtime regressions in tests/unit/session_cli_runtime.rs, tests/contract/capability_provider_protocol_contract.rs, and tests/integration/capability_provider_activation_flow.rs"
Task: "Add focused failing projection regressions in tests/unit/cli_output.rs, tests/contract/host_command_output_contract.rs, and tests/integration/host_session_runtime_flow.rs"
```

## Parallel Example: User Story 3

```bash
# Launch projection and assistant-surface work in parallel after the execution contract lands:
Task: "Add additive provider projection fields in src/domain/session.rs and src/domain/trace.rs"
Task: "Render provider state in src/cli/output_host.rs, src/cli/output_runtime.rs, src/cli/output_session_status.rs, and src/cli/inspect/projections.rs"
Task: "Align assistant assets in assistant/antigravity/, assistant/claude/, assistant/codex/, and assistant/copilot/"
```

---

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational regressions.
2. Complete US1 provider registration and activation behavior.
3. Validate one discoverable-but-inactive provider, one successful activation,
   and one interrupted replacement activation in isolated fixtures.
4. Proceed to bounded execution only after activation semantics are stable.

### Incremental Delivery

1. Finish Setup + Foundational so the missing provider contract is failing in
   focused tests.
2. Add US1 and validate explicit registration and activation.
3. Add US2 and validate prepare, execute, collect-evidence, and validation
   disposition.
4. Add US3 and validate runtime, host, and assistant projections.
5. Close docs, release metadata, and coverage only after the runtime contract
   is stable.

### Quality Rule

Do not treat formatting, clippy, tests, docs, assistant assets, release
metadata, or changed-file coverage as deferred cleanup. The feature is complete
only when `cargo fmt --check`, strict clippy, the regression suite, and at
least 95 percent changed-file coverage pass while the release and Canon
compatibility surfaces remain aligned.

---

## Notes

- `[P]` tasks target different files and can be executed in parallel.
- `[US1]`, `[US2]`, and `[US3]` labels preserve traceability from spec to
  implementation.
- Every user story remains independently testable once its phase completes.
- Provider output must remain non-authoritative throughout implementation.
