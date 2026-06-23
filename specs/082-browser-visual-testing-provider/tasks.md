# Tasks: Browser And Visual Testing Provider

**Input**: Design documents from `/specs/082-browser-visual-testing-provider/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/browser-provider-protocol.md, quickstart.md

**Tests**: Included — all tasks include corresponding test coverage per Rust project conventions and the constitution's requirement for failure-path testing (Principle XV).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

All paths are relative to the workspace root. The project uses a Cargo workspace with three member crates:
- `crates/boundline-core/` — domain types and traits
- `crates/boundline-adapters/` — persistence and I/O adapters
- `crates/boundline-cli/` — CLI presentation layer

Tests live under `tests/` at the workspace root.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Module skeleton and provider protocol contract wire-up

- [x] T001 Create `src/domain/browser_provider.rs` with module-level doc comment, empty placeholder enums for FindingKind, StepStatus, ArtifactKind, and RetentionClass
- [x] T002 [P] Declare `pub mod browser_provider` in `crates/boundline-core/src/domain.rs`
- [x] T003 [P] Register `browser` capability kind variant in `src/domain/capability_provider.rs` (new `CapabilityKind::Browser` with stable serialized identifier matching `browser-provider-v1`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core domain types and message schemas that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Define `StepStatus` enum (Completed, Failed, TimedOut, ProviderError, Cancelled, QueueTimeout, QueueFull) with `Serialize`/`Deserialize` in `src/domain/browser_provider.rs`
- [x] T005 [P] Define `FindingSeverity` enum (Blocking, Warning, Info) and `FindingKind` enum (12 variants: ConsoleError, AccessibilityViolation, VisualDiffDetected, NetworkAccessViolation, PageLoadTimeout, BrowserReadinessTimeout, BaselineCreated, ScriptStepFailed, AccessibilityScanFailed, BrowserConcurrencyTimeout, BrowserQueueFull, CancelledBeforeStart) in `src/domain/browser_provider.rs`
- [x] T006 [P] Define `ArtifactKind` enum (Screenshot, ConsoleLog, NetworkLog, DomSnapshot, AccessibilityOutput, EvidencePacket, DiffImage) and `RetentionClass` enum (RequiredEvidence, Diagnostic, Verbose, Ephemeral) in `src/domain/browser_provider.rs`
- [x] T007 [P] Define `ArtifactReference` struct with all fields from data-model.md §5 (kind, relative_path, content_hash, media_type, byte_size, created_at, retention_class, validation_run_id) in `src/domain/browser_provider.rs`
- [x] T008 Define `BrowserFinding` struct with all fields from data-model.md §6 (kind, severity, message, evidence_refs, retryability, confirmed_intermittent) in `src/domain/browser_provider.rs`
- [x] T009 [P] Define `RetryabilityHint` struct and `RetryabilityLevel`/`RetryabilityCategory` enums from data-model.md §7 in `src/domain/browser_provider.rs`
- [x] T010 [P] Define `StepTiming` struct with all fields from data-model.md §8 (queue_wait_ms, navigation_ms, readiness_wait_ms, script_execution_ms, accessibility_ms, total_ms) in `src/domain/browser_provider.rs`
- [x] T011 [P] Define `LocatorType` enum (CssSelector, TestId, AccessibleRole, Text) and `LocatorState` enum (Attached, Visible, Hidden, Detached) in `src/domain/browser_provider.rs`
- [x] T012 [P] Define `ReadinessLocator` struct with all fields from data-model.md §2 (locator_type, locator_value, expected_state, timeout_seconds, stabilization_delay_ms) in `src/domain/browser_provider.rs`
- [x] T013 [P] Define `BrowserAction` enum (Navigate, Click, Type, Wait, Screenshot) with payload structs from data-model.md §3 in `src/domain/browser_provider.rs`
- [x] T014 Define `BrowserEvidencePacket` struct with all fields from data-model.md §4 (validation_run_id, provider_id, status, started_at, completed_at, page_title, http_status, artifacts, findings, timing, capabilities_active, schema_version) in `src/domain/browser_provider.rs`
- [x] T015 [P] Define `BrowserValidationStep` struct with all fields from data-model.md §1 (validation_run_id, url, readiness, interaction_script, accessibility_enabled, dom_inspection_enabled, baseline_ref, timeouts, network_allowlist, artifact_dir, session_id) in `src/domain/browser_provider.rs`
- [x] T016 Define `ValidationTimeouts` struct with all fields from data-model.md §9 (page_load_seconds, readiness_seconds, script_step_seconds, execution_seconds) in `src/domain/browser_provider.rs`
- [x] T017 Implement JSON request serialization for `BrowserValidationStep` — must produce the exact schema from contracts/browser-provider-protocol.md request section
- [x] T018 Implement JSON response deserialization into `BrowserEvidencePacket` — must accept the exact schema from contracts/browser-provider-protocol.md response section
- [x] T019 Write unit tests for all domain type serialization round-trips (request → JSON → request, JSON → response → struct) covering all enum variants in `tests/unit/browser_provider_types.rs`

**Checkpoint**: All domain types defined and serialization validated. Provider dispatch can now be built on top.

---

## Phase 3: User Story 1 - Basic Browser Validation Step (Priority: P1) 🎯 MVP

**Goal**: Invoke browser provider via JSON stdio, capture screenshot + console errors for a single URL, produce evidence packet, and surface findings in trace/inspect output.

**Independent Test**: Register a mock browser provider, run `boundline validate browser --url http://localhost:3000`, verify evidence packet written to session-scoped artifact directory (screenshot, console.json, evidence.json), and verify findings appear in `boundline inspect browser` output.

### Implementation for User Story 1

- [x] T020 [P] [US1] Create `crates/boundline-adapters/src/browser_provider_runtime.rs` with module-level doc comment
- [x] T021 [P] [US1] Create `crates/boundline-adapters/src/browser_artifact_store.rs` with module-level doc comment
- [x] T022 [US1] Implement `BrowserProviderRuntime` struct in `crates/boundline-adapters/src/browser_provider_runtime.rs` — spawns provider subprocess via `std::process::Command`, reads startup handshake (JSON line from stdout per contract), validates protocol line and schema version, and returns provider capabilities
- [x] T023 [US1] Implement `BrowserProviderRuntime::dispatch()` in `crates/boundline-adapters/src/browser_provider_runtime.rs` — serializes `BrowserValidationStep` to JSON, writes to provider stdin, reads one JSON response line from provider stdout, deserializes into `BrowserEvidencePacket`, captures stderr on failure for `provider_error` status
- [x] T024 [US1] Implement `BrowserArtifactStore` struct in `crates/boundline-adapters/src/browser_artifact_store.rs` — creates session-scoped artifact directory `.boundline/sessions/<id>/browser/<run_id>/` with subdirectories for screenshots, logs, DOM, accessibility
- [x] T025 [US1] Implement `BrowserArtifactStore::write_artifact()` in `crates/boundline-adapters/src/browser_artifact_store.rs` — writes artifact bytes to disk, computes SHA-256 content hash, creates `ArtifactReference`, returns reference
- [x] T026 [US1] Implement `BrowserArtifactStore::write_evidence_packet()` in `crates/boundline-adapters/src/browser_artifact_store.rs` — writes normalized evidence packet JSON to `evidence.json`, records it as a `required_evidence` artifact
- [x] T027 [US1] Implement finding normalization in `BrowserProviderRuntime` — maps provider response `findings` array into Boundline structured findings. For each `BrowserFinding`, produce a finding record with category, severity, message, and artifact refs. Retryability hints are preserved as advisory metadata.
- [x] T028 [US1] Register wire-up in `crates/boundline-adapters/src/adapters.rs` — add `pub mod browser_provider_runtime` and `pub mod browser_artifact_store`
- [x] T029 [US1] Add `BrowserValidationCompleted` trace event type to `src/domain/observability.rs` with a payload schema matching the evidence packet (schema_version=1, provider_id, status, artifact_count, finding_count, total_ms)
- [x] T030 [US1] Emit `BrowserValidationCompleted` trace event after each completed provider dispatch in `BrowserProviderRuntime::dispatch()`
- [x] T031 [US1] Add additive browser validation run reference field (`browser_validation_runs: Vec<BrowserValidationRunRef>`) to `ActiveSessionRecord` in `src/domain/session.rs` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
- [x] T031a [US1] Implement session-scoped artifact path construction in `BrowserProviderRuntime` — derive `.boundline/sessions/<session_id>/browser/<run_id>/` from the session identifier and validation run identifier, and pass it as `artifact_dir` in every provider request
- [x] T031b [US1] Implement console log serialization in `BrowserArtifactStore` — write structured console entries (severity, message, source location) as JSON to `console.json` in the artifact directory and record the artifact reference with kind `ConsoleLog` (FR-011)
- [x] T031c [US1] Implement network permission policy enforcement in `BrowserProviderRuntime` — validate the `network_allowlist` field in the request before dispatch. Provide the allowlist to the provider in the JSON request payload. When the response includes `network_access_violation` findings, normalize them and surface in trace/inspect output (FR-008)
- [x] T031d [US1] Implement artifact size limit check in `BrowserArtifactStore` — before writing any artifact, check its byte size against a configurable maximum (default 50 MB for screenshots, 10 MB for logs). Reject oversized artifacts with an `artifact_size_exceeded` finding and record the rejection in the evidence packet without blocking other artifact writes.
- [x] T032 [US1] Create `crates/boundline-cli/src/cli/validate_browser.rs` — implement `boundline validate browser` subcommand that accepts `--url`, `--readiness-selector`, `--readiness-state`, `--readiness-timeout` args, builds `BrowserValidationStep`, dispatches via `BrowserProviderRuntime`, writes artifacts, updates session record, and renders findings to terminal
- [x] T033 [US1] Wire `boundline validate browser` subcommand into `crates/boundline-cli/src/cli.rs`
- [x] T034 [US1] Create `crates/boundline-cli/src/cli/inspect_browser.rs` — implement `boundline inspect browser` subcommand with `--run`, `--artifacts`, `--findings` flags. Reads evidence packet from artifact directory, renders summary/findings/artifacts to terminal
- [x] T035 [US1] Wire `boundline inspect browser` subcommand into `crates/boundline-cli/src/cli.rs`
- [x] T036 [US1] Write unit tests for `BrowserProviderRuntime` dispatch — success path (valid response), failure paths (provider binary not found, startup timeout, malformed JSON response, provider error status, missing evidence packet) in `tests/unit/browser_provider_types.rs`
- [x] T037 [US1] Write unit tests for `BrowserArtifactStore` — directory creation, artifact write + hash, evidence packet write, retention class assignment in `tests/unit/browser_provider_types.rs`
- [x] T038 [US1] Write contract test for provider startup handshake — validate that a mock provider emitting the correct handshake JSON is accepted, and a provider emitting malformed/missing handshake is rejected in `tests/contract/browser_provider_protocol.rs`
- [x] T039 [US1] Write contract test for request/response schema — serialize a `BrowserValidationStep`, validate against contract schema; deserialize a valid `BrowserEvidencePacket`, validate all required fields present in `tests/contract/browser_provider_protocol.rs`
- [x] T040 [US1] Write integration test for end-to-end basic validation — configure a mock browser provider (shell script emitting predefined evidence JSON), run `boundline validate browser --url http://example.com`, verify evidence.json written, verify findings rendered, verify trace event emitted in `tests/integration/browser_provider_cli.rs`
- [x] T041 [US1] Write integration test for provider failure — configure a mock provider that returns `status: failed`, verify error finding rendered, verify stderr captured, verify session not corrupted in `tests/integration/browser_provider_cli.rs`
- [x] T041a [US1] Write edge case test for malformed JSON from provider — mock provider emits non-JSON or truncated JSON on stdout; verify `provider_error` status, verify stderr captured as diagnostic, verify session not corrupted and no partial evidence packet written in `tests/integration/browser_provider_cli.rs`
- [x] T041b [US1] Write edge case test for file-download-on-load scenario — mock provider simulating a page that triggers an automatic file download on load; verify the provider does not hang, captures the download event as a diagnostic finding, and completes the step without crashing in `tests/integration/browser_provider_cli.rs`

**Checkpoint**: US1 is fully functional — single-URL screenshot + console capture with evidence packet, trace event, and CLI inspection.

---

## Phase 4: User Story 2 - DOM Inspection And Accessibility Checks (Priority: P2)

**Goal**: Extend the browser validation step with optional DOM snapshot capture and accessibility audit (axe-core or equivalent). Findings include rule identifiers, impact levels, and element selectors.

**Independent Test**: Run `boundline validate browser --url http://localhost:3000 --accessibility --dom-inspection`, verify accessibility findings (or explicit "zero violations") in evidence packet, and verify DOM snapshot artifact in the artifact directory.

### Implementation for User Story 2

- [x] T042 [P] [US2] Add `dom_inspection_enabled: bool` and `dom_root_selector: Option<String>`, `dom_max_depth: Option<u32>` fields to `BrowserValidationStep` in `src/domain/browser_provider.rs` (fields already in data model, ensure serialization)
- [x] T043 [P] [US2] Add `AccessibilityViolation` sub-struct with fields (rule_id, impact, element_selector, description) to `BrowserEvidencePacket` findings context in `src/domain/browser_provider.rs`
- [x] T044 [US2] Extend `BrowserProviderRuntime::dispatch()` in `crates/boundline-adapters/src/browser_provider_runtime.rs` — pass `accessibility` and `dom_inspection` flags in the JSON request payload
- [x] T045 [US2] Implement accessibility finding normalization in `BrowserProviderRuntime` — when response contains accessibility findings, map each to a `BrowserFinding` with kind `AccessibilityViolation` and preserve rule_id, impact, element_selector. When response reports zero violations, add an info finding with `accessibility_scan_passed`.
- [x] T046 [US2] Add `--accessibility` and `--dom-inspection` flags to `boundline validate browser` in `crates/boundline-cli/src/cli/validate_browser.rs`
- [x] T047 [US2] Extend `boundline inspect browser` to render accessibility findings with rule_id, impact level, and element selector in `crates/boundline-cli/src/cli/inspect_browser.rs`
- [x] T048 [US2] Write unit test for accessibility finding normalization — valid findings, zero-violations case, accessibility scan failure (timeout, injection failure → FR-015 `accessibility_scan_failed` finding) in `tests/unit/browser_provider_types.rs`
- [x] T049 [US2] Write integration test for accessibility audit — mock provider returning accessibility findings, verify all violations rendered with correct rule_id/impact/selector in `tests/integration/browser_provider_cli.rs`

**Checkpoint**: US1 + US2 both functional — basic validation + accessibility/DOM inspection.

---

## Phase 5: User Story 3 - Scripted Interactions And Visual Diff (Priority: P3)

**Goal**: Execute scripted browser interaction sequences (navigate, click, type, wait, screenshot) and compare screenshots against stored baselines for visual regression detection.

**Independent Test**: Create a scripted interaction JSON file, run `boundline validate browser --url http://localhost:3000 --script steps.json --baseline dashboard-v1`, verify step-by-step screenshots captured, verify visual diff finding (or baseline_created) in evidence packet.

### Implementation for User Story 3

- [x] T050 [P] [US3] Define `InteractionScript` wrapper struct (Vec<BrowserAction>) with `Serialize` in `src/domain/browser_provider.rs` (BrowserAction already defined in T013)
- [x] T051 [P] [US3] Add `baseline_ref: Option<String>` serialization to `BrowserValidationStep` JSON request
- [x] T052 [US3] Extend `BrowserProviderRuntime::dispatch()` in `crates/boundline-adapters/src/browser_provider_runtime.rs` — serialize interaction_script and baseline_ref into the JSON request payload
- [x] T053 [US3] Implement visual diff finding normalization in `BrowserProviderRuntime` — when response contains `VisualDiffDetected` findings, preserve diff percentage and diff image artifact reference. When response indicates baseline was created, produce `BaselineCreated` finding.
- [x] T054 [US3] Implement script step failure normalization in `BrowserProviderRuntime` — when response findings include `ScriptStepFailed`, preserve step index, failure reason, and diagnostic screenshot reference
- [x] T055 [US3] Add `--script` and `--baseline` flags to `boundline validate browser` in `crates/boundline-cli/src/cli/validate_browser.rs` — `--script` accepts a path to a JSON interaction script file
- [x] T056 [US3] Extend `boundline inspect browser` to render interaction script results (per-step screenshots, step durations, step failures, visual diff results) in `crates/boundline-cli/src/cli/inspect_browser.rs`
- [x] T057 [US3] Write unit test for script step failure normalization — mock response with step_index=2, failure="element not found", verify finding rendered correctly in `tests/unit/browser_provider_types.rs`
- [x] T058 [US3] Write unit test for visual diff normalization — mock response with diff_percentage=12.5, diff_image_ref, verify VisualDiffDetected finding; mock baseline_created response, verify BaselineCreated finding in `tests/unit/browser_provider_types.rs`
- [x] T059 [US3] Write integration test for interaction script — mock provider returning per-step screenshots, verify all steps captured, verify script_step_failed on failing step in `tests/integration/browser_provider_cli.rs`
- [x] T060 [US3] Write integration test for visual diff — first run creates baseline, second run with baseline detects diff, third identical run passes within tolerance in `tests/integration/browser_provider_cli.rs`

**Checkpoint**: All three user stories functional — budget enforcement, route selection, spend exception approval, and pricing snapshot lifecycle.

---

## Phase 6: Concurrency, Retryability, And Readiness (Cross-Cutting)

**Purpose**: Queue semantics, retryability hints, and readiness locators are shared across all user stories but were specified in detail during clarification. Implement them as cross-cutting additions.

- [x] T061 Implement provider startup handshake parsing in `BrowserProviderRuntime` — read one JSON line from stdout, validate `protocol: "browser-provider-v1"` and `schema_version: 1`, extract capabilities map and concurrency state
- [x] T062 Implement concurrency awareness in `BrowserProviderRuntime` — after handshake, store `max_concurrency`, `active`, `queue_depth`, `max_queue`. Expose `is_at_capacity()` method that checks whether queue is full
- [x] T063 Implement queue management awareness — before dispatch, check `is_at_capacity()` and reject immediately with `BrowserQueueFull` finding if queue is full. Provider owns the actual queue; Boundline is an informed client.
- [x] T064 Implement retryability hint normalization in `BrowserProviderRuntime` — when a finding carries a `retryability` field, preserve level, category, reason, and timing_context. Do not modify finding disposition.
- [x] T065 Implement readiness locator serialization — ensure `ReadinessLocator` is correctly serialized into the JSON request (locator type, value, state, timeout, stabilization delay)
- [x] T066 Implement readiness timeout normalization — when response contains `BrowserReadinessTimeout` finding, map to finding with diagnostic screenshot ref and timing context. Distinguish from `page_load_timeout`.
- [x] T067 Write unit test for readiness locator serialization — CSS selector, test_id, accessible role, text locator types; all four states in `tests/unit/browser_provider_types.rs`
- [x] T068 Write unit test for retryability hint normalization — verify level "likely" with category "network_transient" preserved, verify non-retryable application findings (selector never appears, JS exception) carry `not_indicated` or no hint in `tests/unit/browser_provider_types.rs`
- [x] T069 Write contract test for concurrency state in handshake — mock provider handshake with concurrency block, verify Boundline parses max_concurrency and active correctly in `tests/contract/browser_provider_protocol.rs`
- [x] T070 Write integration test for queue-full scenario — configure mock provider returning queue_full status, verify `BrowserQueueFull` finding, verify request not dispatched to provider in `tests/integration/browser_provider_cli.rs`

---

## Phase 7: Integration & Observability

**Purpose**: Trace event wiring, provider health checks, artifact lifecycle integration

- [x] T071 Wire `BrowserValidationCompleted` trace event emission into `browser_provider_runtime.rs` — emit after every completed dispatch (success, failure, timeout, queue-full)
- [x] T072 Implement provider health check integration — extend `boundline provider status` to query the browser provider's handshake and display capabilities, concurrency state, and queue depth in `crates/boundline-cli/src/cli/provider.rs`
- [x] T073 Implement artifact lifecycle integration — ensure browser artifacts follow session archive/retention policy. When a session is archived, browser artifacts are compacted or retained per their `retention_class`. Wire into existing session archive path.
- [x] T074 Implement artifact cleanup guard — prevent deletion of browser artifacts when a durable verification, governance, or audit record still references them. Produce `artifact_unavailable` finding if a proof record points to a missing artifact.
- [x] T075 Add `boundline provider health browser-playwright` subcommand — display active/blocked state, capabilities, concurrency (active/max, queue_depth/max_queue), last handshake timestamp
- [x] T076 Write contract test for `BrowserValidationCompleted` trace event payload schema — validate required fields (provider_id, status, artifact_count, finding_count, total_ms) present and correct types in `tests/contract/browser_provider_protocol.rs`
- [x] T077 Write integration test for trace event emission — dispatch a validation, verify `BrowserValidationCompleted` event present in trace with correct artifact_count and finding_count in `tests/integration/browser_provider_cli.rs`

---

## Phase 8: Roadmap Conversion & Docs Synchronization

**Purpose**: Convert the roadmap seed into a spec artifact, remove duplication, and synchronize cross-repo documentation per boundline wrapper Rule 2.

- [x] T078 Copy `roadmap/features/21-browser-and-visual-testing-provider.md` to `specs/082-browser-visual-testing-provider/feat-browser-and-visual-testing-provider.md` using the `feat-<slug>.md` convention
- [x] T079 Remove the original roadmap seed `roadmap/features/21-browser-and-visual-testing-provider.md` per move-on-conversion semantics
- [x] T080 Update `roadmap/Next - forward-roadmap.md` to point feature 21 to `specs/082-browser-visual-testing-provider/spec.md` and mark status "In Spec" (supersedes T086)
- [x] T080a [P] Review `docs/` and `tech-docs/` markdown files for stale references to the old roadmap seed or missing browser provider documentation; update `docs/configuration.md` with the new `[providers.<id>]` browser provider section and `boundline validate browser` subcommand
- [x] T081 Update `CHANGELOG.md` with browser provider entry under Unreleased changes, referencing this feature branch
- [x] T082 [P] Update `AGENTS.md` active technologies section with browser provider context

**Checkpoint**: Roadmap seed converted, no duplicate source-of-truth, cross-repo docs synchronized.

---

## Final Phase: Release, Quality, And Verification

**Purpose**: Version bump, format, lint, test, coverage, and release readiness per boundline wrapper Rules 1-4. This phase gates merge.

- [x] T083 Update workspace version in `Cargo.toml` from `0.81.0` to `0.82.0` per boundline versioning policy (feature increment: 0.x.y → 0.x+1.0)
- [x] T084 Run `./scripts/update-docs-versions.sh` to synchronize version references across `docs/`, `tech-docs/`, and `README.md`
- [x] T085 Run `./scripts/sync-distribution-metadata.sh` to update Homebrew formula and Winget manifests from the bumped Cargo.toml version
- [x] T086 Run `cargo fmt` on all modified and new Rust files
- [x] T087 Run `scripts/clippy.sh` (`cargo clippy --workspace --all-targets --all-features -- -D warnings`) and fix all warnings
- [x] T088 Run `scripts/test.sh` (`cargo nextest run --workspace --all-features`) and fix all failing tests
- [x] T089 Run `scripts/coverage.sh` and confirm at least 95% line coverage for every modified or created Rust file. If any file falls below 95%, add targeted tests or justify exclusion explicitly.
- [x] T090 Run `scripts/check-no-local-paths.sh` and verify no local filesystem paths are committed
- [x] T091 Run `scripts/check-rust-no-panic.sh` and verify no new `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, or assert-family macros outside `main.rs`
- [x] T092 Run `scripts/validate-assistant-plugins.sh` if any assistant plugin metadata was touched by this feature
- [x] T093 Validate quickstart.md scenarios end-to-end — run each quickstart command sequence in a temp fixture workspace and verify output matches expected behavior
- [x] T094 Verify that `cargo llvm-cov --workspace --all-features` produces usable lcov.info with no coverage regressions on existing code
- [x] T095 Final review: confirm all 26 Functional Requirements, 8 Success Criteria, 10 edge cases, and 3 user stories are addressed by at least one passing test or explicit deferral note

**Completion Gate**: All quality scripts pass, coverage ≥ 95%, version bumped, docs synchronized, roadmap seed converted.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001 for module file) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational — No dependencies on other stories
- **User Story 2 (Phase 4)**: Depends on Foundational + US1 provider runtime (T022-T023) — adds accessibility/DOM flags to existing dispatch
- **User Story 3 (Phase 5)**: Depends on Foundational + US1 provider runtime — adds interaction scripts and visual diff to existing dispatch
- **Concurrency & Cross-Cutting (Phase 6)**: Depends on US1 provider runtime (T022-T023) + Foundational types
- **Integration & Observability (Phase 7)**: Depends on US1 + US2 + US3 completion for full finding surface
- **Roadmap Conversion (Phase 8)**: Depends on US3 completion — can run in parallel with Phase 6-7
- **Release, Quality, And Verification (Final Phase)**: Depends on all prior phases — final gate

### Within Each User Story

- Domain types before adapter logic
- Adapter runtime before CLI commands
- CLI dispatch before CLI inspect
- Core implementation before tests
- Story complete with tests passing before moving to next priority

### Parallel Opportunities

- T002, T003 can run in parallel (different files)
- T004-T016 can be parallelized within Foundational (same file, but independent type blocks)
- T020, T021 can run in parallel (different adapter files)
- T034, T035 can run in parallel with T032, T033 (inspect vs validate CLI, different files)
- T036, T037, T038, T039 can run in parallel (different test files)
- T042, T043 can run in parallel (same file, additive fields)
- T050, T051 can run in parallel
- T061-T070 (Phase 6) can run in parallel with T042-T049 (Phase 4) and T050-T060 (Phase 5) — cross-cutting vs story-specific
- T078-T082 (Phase 8: Roadmap Conversion) can run in parallel with T071-T077 (Phase 7)
- T083-T095 (Final Phase: Quality) quality scripts can run in parallel after all implementation complete

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T019)
3. Complete Phase 3: User Story 1 (T020-T041)
4. **STOP and VALIDATE**: Register mock provider, run single URL validation, verify evidence packet and findings
5. Deploy/demo MVP — basic browser validation delivers immediate value

### Incremental Delivery

1. Setup + Foundational → Domain types ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP: single-URL screenshot + console)
3. Add User Story 2 → Test independently → Deploy/Demo (accessibility + DOM inspection)
4. Add User Story 3 → Test independently → Deploy/Demo (interaction scripts + visual diff)
5. Phase 6 (Concurrency/Retryability) → Cross-cutting polish
6. Phase 7 (Integration & Observability) → Trace events, health checks, artifact lifecycle
7. Phase 8 (Roadmap Conversion & Docs Sync) → Seed converted, cross-repo docs synchronized
8. Final Phase (Release, Quality, And Verification) → All quality gates pass → Merge ready

---

## Notes

- [P] tasks = different files or independent validation commands, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- The reference browser provider binary is a SEPARATE project — the Boundline workspace has zero dependency on Playwright or any browser automation library
- All new types outside `main.rs` and `#[cfg(test)]` MUST avoid `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, and assert-family macros per constitution Language Rules
- All stable serialization shapes MUST use typed structs/enums with serde derives per constitution Language Rules
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- NEVER run `boundline` CLI against the repository root — use a temp fixture workspace
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` after every code change
- The Final Phase (Release, Quality, And Verification) is the merge gate — all T083-T095 must pass before merging
- T078-T080 handle roadmap seed conversion: the original seed at `roadmap/features/21-browser-and-visual-testing-provider.md` is copied to the spec folder as `feat-browser-and-visual-testing-provider.md` and removed from roadmap per move-on-conversion semantics
- T083 bumps `Cargo.toml` from 0.81.0 to 0.82.0 per boundline versioning policy (feature increment)
