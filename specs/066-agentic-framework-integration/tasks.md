# Tasks: Agentic Framework Integration

**Input**: Design documents from `/specs/066-agentic-framework-integration/`

**Prerequisites**: `plan.md` (required), `spec.md` (required), `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: This feature requires focused unit, contract, integration, and sibling-repo smoke coverage because the host repo, the reusable template repo, and the concrete Speckit adapter repo must stay compatible on the same protocol line, standard response envelope, declared V1 transport set, and stderr trace-only rules.

**Organization**: Tasks are grouped by user story so each story can be delivered and validated independently once the shared protocol and host plumbing are in place.

**Refinement Note**: Checked tasks capture the baseline host, template, and Speckit work that still applies after the V1 contract clarification. Follow-up tasks T042-T047 absorbed the refined success or error envelope, `describe.supported_transports`, stderr trace-only handling, and the explicit one-shot stdio-only scope without reopening already honest completions. The 2026-06-01 packet correction reopens the semantic plan or run slice: T050-T054 now track the authoritative stage map, the mandatory analyze gate, bounded remediation loops, corrected workflow-definition assets, and Boundline-owned visibility expectations before release closure can resume.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel after dependencies are satisfied
- **[Story]**: Which user story the task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Descriptions include concrete file paths in this repo and, where required, explicit sibling-repo paths such as `../boundline-framework-template/...` and `../boundline-adapter-speckit/...`

## Phase 1: Setup (Cross-Repo Bootstrap)

**Purpose**: Create the missing sibling-repo skeletons and the shared test harness support this feature needs before protocol work begins.

- [X] T001 [P] Bootstrap the reusable template repository structure in `../boundline-framework-template/Cargo.toml`, `../boundline-framework-template/src/main.rs`, `../boundline-framework-template/src/lib.rs`, `../boundline-framework-template/tests/contract.rs`, `../boundline-framework-template/README.md`, and `../boundline-framework-template/.gitignore`.
- [X] T002 [P] Bootstrap the Speckit adapter repository crate and harness in `../boundline-adapter-speckit/Cargo.toml`, `../boundline-adapter-speckit/src/main.rs`, `../boundline-adapter-speckit/src/lib.rs`, `../boundline-adapter-speckit/tests/contract.rs`, and `../boundline-adapter-speckit/.gitignore`.
- [X] T003 Add shared temp-workspace and sibling-repo fixture helpers in `tests/support/framework_adapter.rs` and extend the validation walkthrough in `specs/066-agentic-framework-integration/quickstart.md`.

**Checkpoint**: The host repo can reference concrete sibling-repo file targets and test helpers without relying on ad hoc local setup.

---

## Phase 2: Foundational (Blocking Protocol And Host Prerequisites)

**Purpose**: Establish the shared protocol line, host persistence model, exchange-classification rules, and subprocess bridge that every user story depends on.

**⚠️ CRITICAL**: No new protocol-sensitive user story work should start until this phase, including T042, is complete.

- [X] T004 Add the `framework-adapter-v1` protocol models, command payloads, and compatibility constants in `crates/boundline-adapters/src/adapters.rs`, `crates/boundline-adapters/src/orchestrator.rs`, and `crates/boundline-adapters/src/lib.rs`.
- [X] T005 Add golden protocol fixtures and serde round-trip helpers in `crates/boundline-adapters/src/fixture.rs`, `tests/contract/framework_adapter_protocol_contract.rs`, and `tests/contract.rs`.
- [X] T006 Add adapter-selection, config, capability-snapshot, routing, and audit domain records in `src/domain/configuration.rs`, `src/domain/execution.rs`, `src/domain/session.rs`, `src/domain/trace.rs`, and `src/domain.rs`.
- [X] T007 Add adapter registry and workspace persistence plumbing in `src/registry/agent_registry.rs`, `src/registry.rs`, `src/adapters/config_store.rs`, and `src/adapters/audit_store.rs`.
- [X] T008 Add bounded subprocess-host abstractions for `describe`, `preflight`, `execute-stage`, and `emit-hook` in `src/adapters/agent.rs`, `src/adapters.rs`, and `src/orchestrator/session_runtime_runtime_support.rs`.
- [X] T009 Add foundational protocol and persistence coverage in `tests/unit/config_store_additional.rs`, `tests/unit/config_resolution.rs`, `tests/unit/runtime_routing.rs`, `tests/contract/framework_adapter_protocol_contract.rs`, and `tests/contract.rs`.
- [X] T042 Add follow-up host exchange classification for standard success or error envelopes, enforce `describe.supported_transports` so only V1 JSON over stdin/stdout activates, block unsupported long-running transport declarations before stage claim, and keep structured stderr trace-only in `src/adapters/agent.rs`, `src/adapters/audit_store.rs`, `src/domain/execution.rs`, `src/domain/trace.rs`, `src/orchestrator/session_runtime_runtime_support.rs`, `tests/contract/framework_adapter_protocol_contract.rs`, `tests/unit/runtime_routing.rs`, and `tests/contract.rs`.

**Checkpoint**: The shared contract is typed, bounded to one-shot V1 JSON over stdin/stdout, envelope-classified, and host-callable; user-story work can proceed without reopening protocol design.

---

## Phase 3: User Story 1 - Run With Safe Default and Optional Framework Adapter (Priority: P1) 🎯 MVP

**Goal**: Preserve the built-in Canon-aware path when no adapter is selected while enabling an explicitly registered Speckit adapter to run declared stages successfully.

**Independent Test**: In a temp workspace, one lifecycle run with no adapter configured completes on built-in behavior, one lifecycle run after `boundline adapter add speckit` completes with the adapter active on its declared stages, and an adapter that advertises only unsupported transports is blocked before stage claim.

### Tests for User Story 1

- [X] T010 [P] [US1] Add adapter-management CLI contract coverage for built-in-default, known-profile activation, duplicate-registration rejection, single-active-adapter enforcement, and removal in `tests/contract/adapter_management_contract.rs` and `tests/contract.rs`.
- [X] T011 [P] [US1] Add temp-workspace integration coverage for no-adapter runs with a discoverable `boundline-adapter-speckit` binary on `PATH`, unavailable-binary and preflight-failure fallback before stage claim, and successful `boundline adapter add speckit` activation in `tests/integration/framework_adapter_activation.rs` and `tests/integration.rs`.
- [X] T012 [P] [US1] Add template-repo smoke coverage for the reusable scaffold's `describe` and `preflight` happy paths in `../boundline-framework-template/tests/contract.rs` and `../boundline-framework-template/README.md`.
- [X] T043 [P] [US1] Add CLI, activation-flow, and sibling-repo contract coverage for `supported_transports` visibility, standard envelope emission across `describe`, `preflight`, `execute-stage`, and `emit-hook`, and pre-claim blocking when an adapter advertises only unsupported transports in `tests/contract/adapter_management_contract.rs`, `tests/integration/framework_adapter_activation.rs`, `tests/integration.rs`, `../boundline-framework-template/tests/contract.rs`, `../boundline-adapter-speckit/tests/contract.rs`, and the sibling repo READMEs.

### Implementation for User Story 1

- [X] T013 [US1] Implement `boundline adapter add|show|remove` and `init` registration hooks plus duplicate-registration rejection and one-active-adapter validation in `src/cli/adapter.rs`, `src/cli/init.rs`, `src/cli/config.rs`, `src/cli/output_runtime.rs`, `src/cli/output_session_status.rs`, `src/cli.rs`, and `crates/boundline-cli/src/cli.rs`.
- [X] T014 [US1] Implement known-profile resolution, PATH discovery hints, explicit no-auto-enable behavior, and built-in-default status output for `speckit` in `src/registry/agent_registry.rs`, `src/cli/output_host.rs`, `src/cli/output_runtime.rs`, and `src/cli/output_session_status.rs`.
- [X] T015 [US1] Implement host-side activation, actionable unavailable-binary and preflight-failure feedback, fallback before stage claim, and built-in execution preservation in `src/orchestrator/engine.rs`, `src/orchestrator/session_runtime_surface.rs`, `src/orchestrator/session_runtime_native_execution.rs`, and `src/orchestrator/session_runtime_runtime_support.rs`.
- [X] T016 [P] [US1] Implement the reusable template scaffold for `describe`, `preflight`, `execute-stage`, and `emit-hook` in `../boundline-framework-template/Cargo.toml`, `../boundline-framework-template/src/main.rs`, `../boundline-framework-template/src/lib.rs`, and `../boundline-framework-template/README.md`.
- [X] T017 [P] [US1] Implement the known Speckit profile's initial binary, manifest, and successful claimed-stage path in `../boundline-adapter-speckit/Cargo.toml`, `../boundline-adapter-speckit/src/main.rs`, `../boundline-adapter-speckit/src/lib.rs`, `../boundline-adapter-speckit/src/profile.rs`, and `../boundline-adapter-speckit/README.md`.
- [X] T018 [US1] Wire cross-repo smoke validation for the Boundline host, template repo, and Speckit repo in `tests/integration/framework_adapter_activation.rs`, `tests/support/framework_adapter.rs`, `../boundline-framework-template/tests/contract.rs`, and `../boundline-adapter-speckit/tests/contract.rs`.
- [X] T044 [P] [US1] Update the template and Speckit implementations to emit the standard success or error envelope for every stdio command, declare the V1 stdio JSON transport in `describe`, and keep structured stderr optional in `../boundline-framework-template/src/lib.rs`, `../boundline-framework-template/src/main.rs`, `../boundline-adapter-speckit/src/lib.rs`, `../boundline-adapter-speckit/src/main.rs`, and `../boundline-adapter-speckit/src/profile.rs`.
- [X] T047 [US1] Surface validated `supported_transports`, the V1 JSON-over-stdin/stdout-only compatibility gate, and blocked unsupported-transport feedback for `boundline adapter show`, `init`, and status output in `src/cli/adapter.rs`, `src/cli/init.rs`, `src/cli/output_host.rs`, `src/cli/output_runtime.rs`, `src/cli/output_session_status.rs`, and `crates/boundline-cli/src/cli.rs`.

**Checkpoint**: User Story 1 is independently releasable and proves the MVP: safe default behavior plus one explicit known adapter path.

---

## Phase 4: User Story 2 - Selective Stage Overrides (Priority: P2)

**Goal**: Let adapters declare only the stages and hooks they own, enforce the corrected `goal` or `plan` or `run` ownership map, and preserve strict failure semantics once a stage has been claimed.

**Independent Test**: Register Speckit with a subset of stage overrides and hook subscriptions, verify `goal` stays native, `plan` runs the full planning lifecycle with mandatory analyze gating and bounded remediation loops, `run` executes implementation only, `status` and `inspect` remain Boundline-owned visibility surfaces, confirm a post-claim adapter failure stops the run and requires operator intervention, and distinguish protocol-error or transport-failure exchanges from in-band domain `failed` outcomes.

### Tests for User Story 2

- [X] T019 [P] [US2] Add contract coverage for declared-stage filtering, hook subscriptions, and invalid-manifest rejection in `tests/contract/framework_adapter_protocol_contract.rs`, `tests/contract/runtime_routing_contract.rs`, and `tests/contract.rs`.
- [X] T020 [P] [US2] Add integration coverage for partial stage interception, hook delivery, and post-claim failure stop semantics in `tests/integration/framework_adapter_override_flow.rs` and `tests/integration.rs`.
- [X] T021 [P] [US2] Add Speckit repo override and hook tests for claimed-stage success and failure outcomes in `../boundline-adapter-speckit/tests/override_flow.rs` and `../boundline-adapter-speckit/README.md`.
- [X] T045 [P] [US2] Add contract and integration coverage that distinguishes protocol-error envelopes and transport failures from in-band domain `failed` outcomes, and confirms stderr never changes claimed-stage ownership or result classification in `tests/contract/framework_adapter_protocol_contract.rs`, `tests/contract/runtime_routing_contract.rs`, `tests/integration/framework_adapter_override_flow.rs`, and `tests/integration.rs`.

### Implementation for User Story 2

- [X] T022 [US2] Implement stage and hook catalog validation plus routing-decision records in `src/domain/execution.rs`, `src/domain/trace.rs`, `src/orchestrator/session_runtime_execution_core.rs`, `src/orchestrator/session_runtime_surface.rs`, and `src/orchestrator/session_runtime_flow_trace.rs`.
- [X] T023 [US2] Implement hook dispatch, claimed-stage ownership tracking, and intervention-required failure handling in `src/orchestrator/session_runtime_native_execution.rs`, `src/orchestrator/session_runtime_step_execution.rs`, `src/orchestrator/session_runtime_finalization.rs`, and `src/adapters/audit_store.rs`.
- [X] T024 [P] [US2] Extend the template scaffold with partial-override helpers and hook observer examples in `../boundline-framework-template/src/lib.rs`, `../boundline-framework-template/src/main.rs`, `../boundline-framework-template/tests/contract.rs`, and `../boundline-framework-template/README.md`.
- [X] T025 [P] [US2] Implement Speckit selective stage ownership, hook subscribers, and claimed-stage failure responses in `../boundline-adapter-speckit/src/stages.rs`, `../boundline-adapter-speckit/src/hooks.rs`, `../boundline-adapter-speckit/src/profile.rs`, and `../boundline-adapter-speckit/tests/override_flow.rs`.
- [X] T026 [US2] Surface adapter stage ownership and hook outcomes in operator-visible status and inspect output in `src/cli/output_runtime.rs`, `src/cli/output_session_status.rs`, `src/cli/output_trace_summary.rs`, and `src/cli/inspect.rs`.
- [X] T046 [US2] Surface protocol-error versus transport-failure classifications in runtime audit, inspect, and status output while keeping structured stderr as trace-only enrichment in `src/adapters/audit_store.rs`, `src/orchestrator/session_runtime_execution_core.rs`, `src/orchestrator/session_runtime_flow_trace.rs`, `src/cli/output_runtime.rs`, `src/cli/output_session_status.rs`, `src/cli/output_trace_summary.rs`, and `src/cli/inspect.rs`.
- [X] T048 [P] [US2] Add contract and integration coverage proving that when an adapter declares `plan` or `run` and preflight succeeds, native stage completion is not treated as the completed outcome before the adapter returns, successful adapter responses are persisted as the authoritative stage outcomes, and blocked adapter responses leave the host stage blocked and incomplete in `tests/contract/runtime_routing_contract.rs`, `tests/integration/framework_adapter_activation.rs`, `tests/integration/framework_adapter_override_flow.rs`, and `tests/integration.rs`.
- [X] T049 [US2] Update host stage routing so declared `plan` and `run` overrides execute as the authoritative stage path, with host-owned context assembly allowed before invocation, no built-in completion before adapter return, successful adapter responses persisted as the completed stage outcomes, and claimed-stage blocked responses recorded as blocked and incomplete in `src/orchestrator/session_runtime_execution_core.rs`, `src/orchestrator/session_runtime_surface.rs`, `src/orchestrator/session_runtime_runtime_support.rs`, `src/domain/execution.rs`, and `src/adapters/audit_store.rs`.
- [X] T050 [P] [US2] Add corrected host and sibling validation proving `execute-stage(plan)` reports workflow ID `speckit-planning`, runs the required planning command sequence with mandatory `speckit.analyze`, returns planning findings plus remediation counters, and that `execute-stage(run)` reports workflow ID `speckit-implementation`, invokes implementation-only behavior, and never reruns planning commands in `tests/contract/runtime_routing_contract.rs`, `tests/integration/framework_adapter_override_flow.rs`, `tests/integration.rs`, `../boundline-adapter-speckit/tests/override_flow.rs`, and `../boundline-adapter-speckit/tests/contract.rs`.
- [X] T051 [US2] Replace the current Speckit stage bridge with the corrected mapping so `execute-stage(plan)` invokes real Speckit planning rather than bootstrap-only checks, `execute-stage(run)` invokes implementation only, and both stages return the required workflow IDs, response fields, and authoritative artifact refs in `.specify/workflows/speckit/planning.yml`, `.specify/workflows/speckit/implementation.yml`, `../boundline-adapter-speckit/src/main.rs`, `../boundline-adapter-speckit/src/profile.rs`, `../boundline-adapter-speckit/src/stages.rs`, `../boundline-adapter-speckit/src/hooks.rs`, and `../boundline-adapter-speckit/src/config.rs`.
- [X] T052 [P] [US2] Add workflow-definition asset coverage for the corrected Speckit planning and implementation entrypoints, including workflow IDs, workflow entry commands, and minimum artifact classes, while keeping the adapter bridge authoritative for clarify or analyze or remediation loops and run-stage validation or status capture in `.specify/workflows/speckit/planning.yml`, `.specify/workflows/speckit/implementation.yml`, `specs/066-agentic-framework-integration/contracts/framework-adapter-stdio-contract.md`, and `../boundline-adapter-speckit/tests/contract.rs`.
- [X] T053 [US2] Implement the mandatory `speckit.analyze` planning-readiness gate, bounded remediation handling, and the explicit one-initial-analyze plus two remediation or analyze re-check execution limit for claimed `plan` attempts in `../boundline-adapter-speckit/src/stages.rs`, `../boundline-adapter-speckit/src/profile.rs`, `src/orchestrator/session_runtime_planning_runtime.rs`, `src/orchestrator/session_runtime_runtime_support.rs`, and `src/domain/execution.rs`.
- [X] T054 [P] [US2] Surface workflow IDs, planning findings summaries, remediation loop counts, implementation validation refs, and Boundline-owned status or inspect visibility for claimed stages in `src/cli/output_runtime.rs`, `src/cli/output_session_status.rs`, `src/cli/output_trace_summary.rs`, `src/cli/inspect.rs`, `tests/unit/session_cli_runtime.rs`, and `specs/066-agentic-framework-integration/quickstart.md`.

**Checkpoint**: User Story 2 is independently testable with partial overrides, observable hook delivery, and strict post-claim stop semantics.

---

## Phase 5: User Story 3 - Guided Adapter Configuration (Priority: P3)

**Goal**: Guide operators through required adapter configuration, persist the resulting selection, and fail deterministically in non-interactive mode when required fields are missing.

**Independent Test**: Run interactive `boundline adapter add speckit` and custom-adapter setup in a temp workspace, confirm required values are collected and persisted, then rerun non-interactively with a required field removed and verify the host fails before adapter execution with actionable recovery text.

### Tests for User Story 3

- [X] T027 [P] [US3] Add contract coverage for interactive config collection, guided-setup cancellation atomicity, non-interactive missing-field failure, and JSON redaction in `tests/contract/adapter_management_contract.rs`, `tests/contract/config_cli_contract.rs`, and `tests/contract.rs`.
- [X] T028 [P] [US3] Add integration coverage for guided `speckit` setup, first-time custom-adapter registration that ends in a successful preflight or runnable path without manual config edits, interrupted setup that leaves persisted adapter state unchanged, and non-interactive blocking behavior in `tests/integration/framework_adapter_config_flow.rs` and `tests/integration.rs`.
- [X] T029 [P] [US3] Add Speckit repo config-schema and preflight validation coverage in `../boundline-adapter-speckit/tests/config_flow.rs` and `../boundline-adapter-speckit/README.md`.

### Implementation for User Story 3

- [X] T030 [US3] Implement adapter config-schema persistence, redaction, revalidation, and atomic write semantics for interrupted setup in `src/adapters/config_store.rs`, `src/domain/configuration.rs`, `src/cli/config.rs`, and `src/cli/output_runtime.rs`.
- [X] T031 [US3] Implement guided prompt collection, explicit cancel or exit handling with resume guidance, and non-interactive fast-fail behavior in `src/cli/adapter.rs`, `src/cli/init.rs`, `src/cli/workspace.rs`, and `src/orchestrator/session_runtime_surface.rs`.
- [X] T032 [P] [US3] Add known-profile defaults, custom-adapter registration, and `config show` projection updates in `src/registry/agent_registry.rs`, `src/cli/output_host.rs`, `src/cli/output_session_status.rs`, and `tests/unit/session_cli_runtime.rs`.
- [X] T033 [P] [US3] Implement Speckit required-field schema, normalized config handling, and setup guidance in `../boundline-adapter-speckit/src/profile.rs`, `../boundline-adapter-speckit/src/config.rs`, `../boundline-adapter-speckit/tests/config_flow.rs`, and `../boundline-adapter-speckit/README.md`.
- [X] T034 [P] [US3] Refresh the template repo's custom-adapter setup docs and preflight examples in `../boundline-framework-template/README.md`, `../boundline-framework-template/src/lib.rs`, and `../boundline-framework-template/tests/contract.rs`.

**Checkpoint**: User Story 3 is independently testable with interactive setup, persisted config, redacted status output, and deterministic non-interactive failure.

---

## Phase 6: Polish & Cross-Cutting Release Closure

**Purpose**: Close the feature with versioning, docs, wiki, roadmap, and release evidence updates that span the host repo and the sibling repos once the corrected plan or run semantics are implemented and revalidated.

- [X] T035 [P] Update host operator docs and validation guidance for the corrected stage map, workflow IDs `speckit-planning` and `speckit-implementation`, mandatory analyze gating, bounded remediation loops, standard response-envelope semantics, optional structured stderr, and the V1 one-shot stdio scope in `README.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `tech-docs/architecture.md`, and `specs/066-agentic-framework-integration/quickstart.md`.
- [X] T036 [P] Update host wiki pages for corrected adapter stage ownership, planning-readiness gating, implementation-only `run`, and runtime inspection in `../boundline.wiki/Home.md`, `../boundline.wiki/Getting-Started.md`, `../boundline.wiki/Configuration-Reference.md`, `../boundline.wiki/Architecture-And-Decisions.md`, and `../boundline.wiki/Reference.md`.
- [X] T037 Update host release metadata and changelog for the corrected adapter semantics only after the reopened plan or run slice has been implemented and revalidated in `Cargo.toml`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, and `CHANGELOG.md`.
- [X] T038 [P] Update template repo release docs, released `boundline-adapters` dependency pins, and compatibility notes for the corrected workflow-definition assets, standard envelope, declared V1 stdio JSON transport, optional structured stderr as trace-only enrichment, and no graceful-shutdown expectation in `../boundline-framework-template/README.md`, `../boundline-framework-template/Cargo.toml`, and `../boundline-framework-template/CHANGELOG.md`.
- [X] T039 [P] Update Speckit repo docs and release notes for the corrected `plan` and `run` semantics, workflow IDs, analyze gate, bounded remediation loops, and implementation-only `run` behavior in `../boundline-adapter-speckit/README.md`, `../boundline-adapter-speckit/Cargo.toml`, and `../boundline-adapter-speckit/CHANGELOG.md`.
- [X] T040 Update completed feature tracking and roadmap status only after the corrected semantics land and the post-correction analysis gate passes in `roadmap/features/02-agentic-framework-integration.md`, `roadmap/Next - forward-roadmap.md`, and `specs/066-agentic-framework-integration/tasks.md`.
- [X] T041 Run corrected cross-repo validation, rerun `/speckit.analyze` against the active packet, capture a passing post-correction analysis report plus compatibility evidence for the final protocol line, and record the outcome in `specs/066-agentic-framework-integration/research.md`, `specs/066-agentic-framework-integration/quickstart.md`, `CHANGELOG.md`, `../boundline-framework-template/README.md`, and `../boundline-adapter-speckit/README.md`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup** creates the missing sibling-repo structure and shared test support.
- **Phase 2: Foundational** depends on Phase 1 and blocks all story work.
- **Phase 3: User Story 1** depends on Phase 2 and is the MVP release slice.
- **Phase 4: User Story 2** depends on the Phase 2 protocol work and the baseline activation path from User Story 1.
- **Phase 5: User Story 3** depends on the Phase 2 protocol work and the adapter-management surface introduced in User Story 1.
- **Phase 6: Polish** depends on the stories that will ship in the release.

### User Story Dependencies

- **US1 (P1)**: First releasable increment; no user-story dependency beyond the foundational phase.
- **US2 (P2)**: Builds on US1's activation and runtime-ownership path, but remains independently testable through declared-stage and hook fixtures.
- **US3 (P3)**: Builds on US1's registration surface, but remains independently testable through guided setup and non-interactive failure scenarios.

### Within Each User Story

- Write the listed tests before the corresponding implementation tasks and confirm they fail for the expected missing behavior.
- Land host protocol and persistence changes before sibling-repo behavior that depends on them.
- Keep sibling template and Speckit work aligned to the released `boundline-adapters` protocol line rather than local path dependencies.
- Finish each story with the story-specific validation task before moving to the next priority.

---

## Parallel Opportunities

### User Story 1

- Run T010, T011, T012, and T043 together after Phase 2 completes.
- Run T016, T017, T044, and T047 together after T013, T014, and T015 establish the host-owned activation path.

### User Story 2

- Run T019, T020, and T021 together once the US1 baseline is stable.
- Run T024 and T025 together after T022 defines the host-owned stage and hook catalog behavior.

### User Story 3

- Run T027, T028, and T029 together after the adapter-management contract is stable.
- Run T033 and T034 together after T030 and T031 lock the host-side config schema behavior.

### Polish Phase

- Run T035, T036, T038, and T039 in parallel after the shipping stories are complete.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate the no-adapter path and `boundline adapter add speckit` path before expanding scope.

### Incremental Delivery

1. Ship the shared protocol and host activation path first.
2. Add selective stage overrides and hook ownership semantics in US2.
3. Add guided config collection and non-interactive failure semantics in US3.
4. Finish with release coordination, documentation, wiki updates, and roadmap closure.

### Parallel Team Strategy

1. One stream owns Phase 1 and Phase 2 host-contract work.
2. After US1 host activation is stable, a second stream can build the template repo while another builds the Speckit repo.
3. Docs and release closure can start in parallel once the shipping stories are functionally complete.

---

## Notes

- The template repo is currently an empty Git repository, so every `../boundline-framework-template/...` path above is a concrete bootstrap target rather than an existing implementation file.
- The Speckit repo currently has only top-level docs and license files, so paths such as `../boundline-adapter-speckit/src/profile.rs` and `../boundline-adapter-speckit/tests/override_flow.rs` are intentional new-file targets.
- Only the host wiki repo is currently identifiable in the local workspace (`../boundline.wiki/`), so wiki closure tasks are concrete for Boundline and README or changelog tasks carry the sibling-repo documentation closure.