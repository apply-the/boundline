# Tasks: Evals And Runtime Observability

**Input**: Design documents from `/specs/072-evals-runtime-observability/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/event-schema-contract.md`, `quickstart.md`

**Tests**: Test tasks are required. Every new or modified Rust implementation
file must reach at least 95% changed-file coverage. Add focused regressions
first, confirm they fail, then close with the smallest coherent runtime change.

**Organization**: Tasks are grouped by user story so the eval suite, trace
compaction gate, and structured event export can be implemented and validated
independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and has no
  dependency on incomplete work
- **[Story]**: Maps a task to a user story for traceability
- Every task includes repository-relative file paths

---

## Phase 1: Setup

**Purpose**: Lock the release boundary, reconfirm the provider-catalog no-change
result, and fail fast before any domain or CLI code is written.

- [x] T001 Determine the next release version (currently 0.72.0 → 0.73.0) by reading `Cargo.toml` and confirming no version conflict with in-progress sibling specs in `specs/`
- [x] T002 [P] Reconfirm the provider-catalog no-change audit from `specs/072-evals-runtime-observability/research.md` against `assistant/catalog/model-catalog.toml`
- [x] T003 [P] Add release-surface regression references for Boundline 0.73.0 in `tests/contract/distribution_release_surface_contract.rs` and `tests/contract/canon_reasoning_posture_contract.rs`
- [x] T003a [P] Document the FR-013 enforcement mechanism in `specs/072-evals-runtime-observability/spec.md`: every AI behavior change must declare its eval path, and enforcement for this slice is through review checklist and CI gate, not automated source-code analysis. No runtime code task is required for FR-013 in this slice.

---

## Phase 2: Foundational

**Purpose**: Define every domain type, constant, and serialization shape so
no magic strings or ad hoc JSON assembly leak into implementation.

**Critical**: Complete this phase before user-story implementation.

- [x] T004 [P] Define `RetentionClass` enum (Lossless, Structured, Summary, IndexOnly, Discardable), `CompactionAction` struct, `CompactionMetrics` struct, and `TraceCompactionPolicy` type in `src/domain/trace_compaction.rs`
- [x] T005 [P] Define `EventType` enum, `EventPayload` enum with typed inner structs per variant, `StructuredRuntimeEvent` struct, and `schema_version` constants per event type in `src/domain/observability.rs`
- [x] T006 [P] Define `EvalDimension` enum, `EvalFixture` struct, `EvalResult` struct, `EvalStatus` enum (Pass/Fail), and `EvalSummary` struct in `src/domain/evals.rs`
- [x] T007 [P] Define `RuntimeMetrics` struct with all 12 metric fields from the data model in `src/domain/observability.rs`
- [x] T008 Wire the new domain modules (`evals`, `trace_compaction`, `observability`) into `src/domain/mod.rs` with `pub mod` declarations and module-level doc comments
- [x] T009 [P] Add focused failing domain regressions for `RetentionClass` classification rules, conservative tiebreaking, `EvalResult` pass/fail serialization, and `StructuredRuntimeEvent` schema_version presence in `tests/unit/trace_compaction_model.rs`, `tests/unit/evals_model.rs`, and `tests/unit/observability_model.rs`

**Checkpoint**: All domain types compile, serialization round-trips pass,
and the failing regressions confirm the contract gaps before implementation.

---

## Phase 3: User Story 1 - Run Quality Evals To Validate Behavior Changes (Priority: P1) 🎯 MVP

**Goal**: Operators can run a local eval suite covering planning quality,
context selection, guardian findings, council rejection, provider failure,
and compaction survival, and CI can consume the machine-readable summary.

**Independent Test**: In an isolated temp workspace, run `boundline evals run`
against known-good and known-broken session fixtures and confirm only the broken
session fails with per-eval attribution.

### Implementation

- [x] T010 [US1] Implement the eval fixture loader: scan `.boundline/evals/` for fixture files, deserialize `EvalFixture` entries, and validate required fields in `src/domain/evals.rs`
- [x] T011 [US1] Implement the eval runner core: iterate fixtures, execute each eval against its referenced session trace, compare expected vs actual outcome, and build `EvalResult` and `EvalSummary` in `src/domain/evals.rs`
- [x] T012 [US1] Implement the `boundline evals run` CLI command with `--json` flag for CI mode and human-readable table output for local mode in `src/cli/evals.rs`
- [x] T013 [US1] Implement human-readable eval output rendering (table with status indicators and durations) in `src/cli/output/evals_output.rs`
- [x] T014 [US1] Implement JSON eval output rendering (suite summary with per-eval result array) in `src/cli/output/evals_output.rs`
- [x] T015 [US1] Add eval fixture files for all 8 eval dimensions under `tests/fixtures/evals/` and `.boundline/evals/` (planning-quality, context-selection, critical-omission, guardian-finding, council-rejection, provider-failure, compaction-decisions, compaction-rejections)
- [x] T016 [US1] Run the focused US1 regression set and expand until `tests/unit/evals_model.rs` passes
- [x] T017 [P] [US1] Add eval output contract tests verifying JSON summary shape, suite aggregate AND logic, exit code behavior, and per-eval field completeness in `tests/contract/eval_output_contract.rs`
- [x] T018 [US1] Add eval runner integration tests in `tests/integration/eval_runner_flow.rs` covering the full flow from fixture load through summary output

- [x] T018a [US1] Implement pre-compacted fixture detection in the eval runner: before evaluating any fixture, verify that all referenced trace data is available; if trace data was compacted, fail with an actionable message identifying the missing fixture data and the compaction event that removed it, in `src/domain/evals.rs`
- [x] T018b [P] [US1] Add contract tests for pre-compacted fixture failure behavior in `tests/contract/eval_output_contract.rs`

**Checkpoint**: Eval suite runs locally and in CI, produces correct per-eval
pass/fail statuses, and fails CI exit code when any required eval fails.

---

## Phase 4: User Story 2 - Protect Critical Evidence With Trace Compaction (Priority: P2)

**Goal**: Operators can run `boundline trace compact` to reduce trace storage
while preserving accepted decisions, rejection reasons, and active stage
evidence in exact original form.

**Independent Test**: Run compaction against a trace with decisions, rejections,
and transcripts; confirm decisions/rejections survive exactly and transcripts
are replaced with lossy-marked summaries.

### Implementation

- [x] T019 [US2] Implement the `TraceCompactionPolicy` classification table: map every known trace item type to its `RetentionClass`, with item-type constants extracted to named `const` items in `src/domain/trace_compaction.rs`
- [x] T020 [US2] Implement the single-pass compaction algorithm: iterate trace items, classify each, apply retention rules, collect `CompactionAction` entries, and enforce hard survival rules (decisions, rejections, active stage evidence never destructively compacted) in `src/domain/trace_compaction.rs`
- [x] T021 [US2] Implement conservative tiebreaking: when an item type is not in the classification table, resolve to the stricter adjacent class and mark `tiebreak = true` in the `CompactionAction` in `src/domain/trace_compaction.rs`
- [x] T022 [US2] Implement oversized-trace guard: when trace exceeds 50k items or configured byte-size limit, fail with an actionable message unless operator confirms or chunked processing is selected in `src/domain/trace_compaction.rs`
- [x] T023 [US2] Implement the `boundline trace compact` CLI command in `src/cli/trace_compact.rs`
- [x] T023a [US2] Implement hard-survival-rule policy override: when the classification table would assign a non-lossless class to active stage evidence, override to lossless and record the override in the `CompactionAction` with the original classification and the override reason, in `src/domain/trace_compaction.rs`
- [x] T024 [US2] Implement human-readable compaction output rendering (class distribution table, lossy count, preserved refs) in `src/cli/output/compaction_output.rs`
- [x] T025 [US2] Implement JSON compaction output rendering in `src/cli/output/compaction_output.rs`
- [x] T026 [US2] Run and expand compaction domain regressions in `tests/unit/trace_compaction_model.rs`
- [x] T027 [P] [US2] Add compaction event contract tests verifying event shape, `schema_version`, lossy flags, tiebreak flags, and preserved refs in `tests/contract/compaction_event_contract.rs`
- [x] T028 [US2] Add trace compaction integration tests in `tests/integration/trace_compaction_flow.rs` covering the full flow from command invocation through event emission

**Checkpoint**: Compaction preserves critical evidence, classifies correctly,
handles oversized traces safely, and emits trace-visible compaction events.

---

## Phase 5: User Story 3 - Export And Visualize Runtime Observability (Priority: P3)

**Goal**: Operators and dashboards can export structured runtime events as JSONL
with per-event-type schema versions, and runtime metrics are recorded for
dashboard consumption.

**Independent Test**: Export a session's structured events as JSONL, confirm
each event has a recognized event type and `schema_version`, verify metrics
fields match runtime state.

### Implementation

- [x] T029 [US3] Implement structured event emission hooks at key runtime boundaries (planning analysis completion, guardian finding, provider call completion, phase request, route decision, context selection) in `src/orchestrator/session_runtime_observability.rs`
- [x] T030 [US3] Implement the event log writer: append `StructuredRuntimeEvent` as JSONL lines to `.boundline/traces/events.jsonl` with atomic line writes in `src/domain/observability.rs`
- [x] T031 [US3] Implement the `boundline trace export --format jsonl` CLI command with optional event-type and time-range filters in `src/cli/trace_export.rs`
- [x] T032 [US3] Emit the `trace.compacted` structured event after every compaction run with full actions list, metrics, and preserved refs via the observability hook in `src/orchestrator/session_runtime_observability.rs` (depends on T029 event emission hooks being complete)
- [x] T033 [US3] Implement sensitive-data filtering with field-level allowlists per event type: omit or redact fields named `token`, `secret`, `password`, `key`, `credential`, `authorization` in `src/domain/observability.rs`
- [x] T033a [US3] Implement JSONL event deduplication by `event_id`: when two events share the same `event_id`, only the first occurrence is exported; record deduplication count in the export summary, in `src/domain/observability.rs`
- [x] T033b [US3] Implement empty-export behavior: when a session has no structured events, produce a valid empty JSONL stream (zero lines), exit code 0, and a status message indicating no events were present, in `src/cli/trace_export.rs`
- [x] T034 [US3] Implement `RuntimeMetrics` collection: accumulate counters during runtime phases and expose via the existing session status and inspect projections in `src/domain/observability.rs` and `src/orchestrator/session_runtime_observability.rs`
- [x] T035 [US3] Wire runtime metrics into existing CLI output surfaces (`boundline status`, `boundline inspect`) as additive fields in `src/cli/output_session_status.rs` and `src/cli/inspect/projections.rs`
- [x] T036 [US3] Run and expand observability domain regressions in `tests/unit/observability_model.rs`
- [x] T037 [P] [US3] Add event schema contract tests verifying JSONL format, per-event-type `schema_version`, sensitive-data absence, event deduplication, empty-export behavior, and event-type enumeration in `tests/contract/event_schema_contract.rs`
- [x] T038 [US3] Add structured event integration tests in `tests/integration/trace_compaction_flow.rs` (extend or create `tests/integration/observability_flow.rs`) covering event emission during a full planning-to-execution session

**Checkpoint**: JSONL export produces valid versioned events, sensitive data is
filtered, metrics appear in status/inspect output, and runtime boundaries emit
the right events.

---

## Phase 6: Release, Documentation, and Quality Closure

**Purpose**: Close versioning, docs, roadmap, quality gates, and coverage for
release 0.73.0.

### Versioning & Release Metadata

- [x] T039 Bump workspace version to `0.73.0` in `Cargo.toml` and propagate to `Cargo.lock` via `cargo update --workspace` (Cargo.lock is auto-updated on next build)
- [x] T040 [P] Update release metadata to `0.73.0` in `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, and `assistant/plugin-metadata.json`
- [x] T041 [P] Add WinGet release manifests for `0.73.0` under `distribution/winget/manifests/a/ApplyThe/Boundline/0.73.0/`
- [x] T042 [P] Update `assistant/global/manifest.json` and `src/cli/init.rs` version references to `0.73.0`

### Documentation

- [x] T043 [P] Update `README.md` with the evals, trace compaction, and observability feature summary
- [x] T044 [P] Update `CHANGELOG.md` with the `0.73.0` release entry covering all three user stories
- [x] T045 [P] Update `tech-docs/architecture.md` with evals, trace compaction, and observability module descriptions
- [x] T046 [P] Update `tech-docs/configuration.md` with any new config surface (eval fixture path, compaction bounds)
- [x] T047 [P] Update `tech-docs/getting-started.md` with quickstart-style instructions for `boundline evals run` and `boundline trace compact`
- [x] T048 [P] Add runtime docs for evals, trace compaction, and observability in `docs/runtime/evals.md`, `docs/runtime/trace-compaction.md`, and `docs/runtime/observability.md`
- [x] T049 [P] Update `docs/guide/common-workflows.md` with eval and compaction workflow examples
- [x] T050 Run `scripts/update-docs-versions.sh` to synchronize version references across the docs website

### Roadmap

- [x] T051 [P] Record the delivered roadmap slice for `08-evals-and-runtime-observability` in `CHANGELOG.md`, `docs/roadmap/index.md`, and `roadmap/Next - forward-roadmap.md`
- [x] T052 [P] Mark the feature as delivered in `roadmap/features/08-evals-and-runtime-observability.md` status (note: file moved to `specs/072-evals-runtime-observability/feat-evals-and-runtime-observability.md`)

### Quality Gates

- [x] T053 Run `cargo fmt` and verify with `cargo fmt --check`
- [x] T054 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix every reported issue
- [x] T055 Run focused tests: `cargo test --test unit`, `cargo test --test contract`, and `cargo test --test integration`
- [x] T056 Run release-surface regressions in `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_release_surface_contract.rs`, and `tests/contract/distribution_metadata_contract.rs`
- [x] T057 Run the full regression suite with `cargo test` and resolve any failures
- [x] T058 Generate `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- [x] T059 Build an explicit repository-relative implementation-file list, run `scripts/common/coverage/intersect_patch_coverage.py` against every touched Rust implementation file, and add tests until changed-file coverage reaches at least 95%
- [x] T060 Validate the isolated scenarios in `specs/072-evals-runtime-observability/quickstart.md` without running Boundline CLI commands against the repository root

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Setup and blocks all story work.
- **User Story 1 (Phase 3)**: Depends on Foundational and is the MVP.
- **User Story 2 (Phase 4)**: Depends on Foundational; can start after US1
  domain types stabilize but compaction is independent of the eval runner.
- **User Story 3 (Phase 5)**: Depends on Foundational and on US2 (compaction
  events are structured events). Can partially start after US1 domain types.
- **Release and Quality Closure (Phase 6)**: Depends on all selected stories.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational; no dependency on later
  stories.
- **User Story 2 (P2)**: Can start after Foundational; the compaction domain is
  independent of the eval runner, but the `trace.compacted` event emission (T032)
  runs in Phase 5 after observability hooks are ready.
- **User Story 3 (P3)**: Can start after Foundational and US2 domain types
  stabilize, since compaction events are structured events.

### Parallel Opportunities

- T002 and T003 can run in parallel.
- T004, T005, T006, T007 can run in parallel (different domain files).
- T009 test files can run in parallel.
- T017 and T028 can run in parallel with their respective implementation phases.
- T040, T041, T042 can run in parallel.
- T043 through T049 (all docs tasks) can run in parallel.
- T051 and T052 can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Launch all domain types together:
Task: "Define EvalDimension enum, EvalFixture struct, EvalResult struct in src/domain/evals.rs"
Task: "Define RetentionClass enum, CompactionAction struct in src/domain/trace_compaction.rs"
Task: "Define EventType enum, StructuredRuntimeEvent struct in src/domain/observability.rs"
Task: "Define RuntimeMetrics struct in src/domain/observability.rs"

# After domain types compile, launch CLI and output in parallel:
Task: "Implement boundline evals run CLI command in src/cli/evals.rs"
Task: "Implement human-readable eval output in src/cli/output/evals_output.rs"
Task: "Implement JSON eval output in src/cli/output/evals_output.rs"
```

---

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational domain types.
2. Complete US1 eval suite (local + CI).
3. Validate one known-good and one known-broken scenario in an isolated temp
   workspace.
4. Proceed to US2 compaction and US3 structured events.

### Incremental Delivery

1. Finish Setup + Foundational so domain types compile.
2. Add US1 and validate the eval runner end-to-end.
3. Add US2 and validate compaction preservation rules.
4. Add US3 and validate JSONL export with schema versioning.
5. Close docs, release metadata, and coverage only after the runtime contract
   is stable.

### Quality Rule

Do not treat formatting, clippy, tests, docs, release metadata, or
changed-file coverage as deferred cleanup. Every new or modified Rust file
must reach at least 95% changed-file coverage. The feature is complete only
when `cargo fmt --check`, strict clippy, the full regression suite, and at
least 95% changed-file coverage pass while release surfaces remain aligned.

---

## Notes

- `[P]` tasks target different files and can be executed in parallel.
- `[US1]`, `[US2]`, and `[US3]` labels preserve traceability from spec to
  implementation.
- Every user story remains independently testable once its phase completes.
- Compaction must stay read-only over trace storage throughout implementation.
- No `boundline` CLI commands may be run against the repository root — use
  isolated temp workspaces for all validation.
- The `schema_version` field is a `&'static str` constant per `EventType`
  variant, not a runtime-computed string.
