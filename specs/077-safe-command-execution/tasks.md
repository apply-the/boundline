# Tasks: Safe Command Execution and Evidence Capture

**Input**: Design documents from `specs/077-safe-command-execution/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/execution-safety.md

**Tests**: Unit tests per module + contract test for CLI behavior + integration test for full pipeline. Coverage target: ≥95% for all modified or created Rust files.

**Organization**: Tasks grouped by setup, user story, integration, polish, and release verification.

---

## Phase 1: Setup — Domain Types & Module Scaffolding

- [x] T001 [P] Create `crates/boundline-core/src/execution/mod.rs` with module declarations for classifier, policy, evidence, redaction, dry_run, mutation, hooks
- [x] T002 [P] Add `pub mod execution;` to `crates/boundline-core/src/lib.rs`
- [x] T003 [P] Define `CommandIntent` enum in `crates/boundline-core/src/execution/classifier.rs` (read, test, mutate, install, deploy, unknown) with serde derives
- [x] T004 [P] Define `ExecutionMode` enum (allow, dry-run, no-mutation, require-approval, deny) with serde derives
- [x] T005 [P] Define `DryRunStatus` enum (native_dry_run_executed, read_only_executed, plan_only, unsupported_for_safe_dry_run)
- [x] T006 [P] Define `EvidencePacket`, `ExecutionTiming`, `ArtifactManifest`, `ArtifactEntry`, `MutationBoundary`, `ModifiedFile` structs in `crates/boundline-core/src/execution/evidence.rs` per data-model.md (all must derive `Debug, Clone, Serialize, Deserialize, PartialEq`)
- [x] T007 [P] Define `PolicyDecision`, `SecretPattern`, `AllowlistRule` structs in their respective modules
- [x] T008 [P] Define `ExecutionPolicy`, `PolicyDefaults`, `PolicyEntry`, `CommandOverride` structs in `crates/boundline-core/src/execution/policy.rs` per data-model.md

**Depends on**: Nothing (parallel)

**Verification**: `cargo check -p boundline-core`

---

## Phase 2: User Story 1 — Classify and Dry-Run a Command (P1) 🎯 MVP

- [x] T009 [US1] Implement command whitelist (50+ common commands mapped to intents) in `crates/boundline-core/src/execution/classifier.rs`
- [x] T010 [US1] Implement argument heuristic refinement (safety flags downgrade, risk flags escalate) in `classifier.rs`
- [x] T011 [US1] Implement unknown command default-to-mutate logic in `classifier.rs`
- [x] T012 [US1] Implement native dry-run mapping table (cargo check, terraform plan, npm --dry-run, etc.) in `crates/boundline-core/src/execution/dry_run.rs`
- [x] T013 [US1] Implement `classify_dry_run()` function returning `DryRunStatus` based on known-safe-mappings vs plan-only
- [x] T014 [US1] Implement `classify_and_dry_run()` integration function in `classifier.rs` — classify intent then dry-run
- [x] T015 [US1] Add unit tests for classifier: 15 known commands, 5 unknown commands, 5 argument refinement cases in `crates/boundline-core/src/execution/classifier.rs` #[cfg(test)]
- [x] T016 [US1] Add unit tests for dry-run: native_dry_run_executed (cargo check, terraform plan), plan_only (rm -rf), read_only_executed (ls, cat), unsupported (unknown binary) in `crates/boundline-core/src/execution/dry_run.rs`

**Depends on**: Phase 1

**Verification**: `cargo test -p boundline-core -- classifier dry_run`

---

## Phase 3: User Story 2 — Capture Structured Evidence (P1)

- [x] T017 [US2] Implement shell command execution wrapper with stdout/stderr pipe capture in `crates/boundline-adapters/src/shell.rs` (extend existing shell adapter)
- [x] T018 [US2] Implement `EvidencePacket` builder with timing capture (wall-clock, started_at, finished_at) in `crates/boundline-core/src/execution/evidence.rs`
- [x] T019 [US2] Implement trace ID generation (`{ISO8601}-{sha256_hex12}`) in `evidence.rs`
- [x] T020 [US2] Implement stdout/stderr truncation with 1MB cap and `[TRUNCATED]` marker in `evidence.rs`
- [x] T021 [US2] Implement evidence persistence to `.boundline/traces/<trace-id>.json` in `evidence.rs`
- [x] T022 [US2] Implement artifact manifest capture (file listing pre/post execution with stat comparison) in `evidence.rs`
- [x] T023 [US2] Add unit tests for evidence: successful command, failing command, empty output, truncated output, artifact manifest in `crates/boundline-core/src/execution/evidence.rs`
- [x] T024 [US2] Add integration test: full `boundline exec "echo hello"` → verify trace file exists with correct shape in `tests/integration/exec_command_integration.rs`

**Depends on**: Phase 1 (T017 can start with Phase 2)

**Verification**: `cargo test -p boundline-core -- evidence && cargo test --test exec_command_integration`

---

## Phase 4: User Story 3 — Redact Secrets (P2)

- [x] T025 [US3] Implement built-in regex patterns (GitHub token, AWS key, JWT) as const defaults in `crates/boundline-core/src/execution/redaction.rs`
- [x] T026 [US3] Implement `.boundline/redaction.toml` loader (patterns + allowlist) in `redaction.rs`
- [x] T027 [US3] Implement `redact_output(stdout, stderr, patterns) -> (redacted_stdout, redacted_stderr, audit)` in `redaction.rs`
- [x] T028 [US3] Implement allowlist rule evaluation (skip patterns matching allowlisted path+regex combos) in `redaction.rs`
- [x] T029 [US3] Implement redaction audit metadata generation (pattern ID, match count) in `redaction.rs`
- [x] T030 [US3] Wire redaction into evidence capture pipeline (redact before persist) in `evidence.rs`
- [x] T031 [US3] Add unit tests: matched secret → redacted, no secret → verbatim, multiple secrets → all redacted, allowlisted secret → preserved, deterministic output in `crates/boundline-core/src/execution/redaction.rs`

**Depends on**: Phase 3 (evidence)

**Verification**: `cargo test -p boundline-core -- redaction`

---

## Phase 5: User Story 4 — Track Mutation Boundaries (P2)

- [x] T032 [US4] Implement pre-execution file snapshot (walk workspace, record paths+hashes) in `crates/boundline-core/src/execution/mutation.rs`
- [x] T033 [US4] Implement post-execution file diff (compare pre/post, classify created/modified/deleted) in `mutation.rs`
- [x] T034 [US4] Implement 10K entry cap with truncation marker and total observed count in `mutation.rs`
- [x] T035 [US4] Implement incomplete mutation boundary on filesystem error (mark complete=false, record error) in `mutation.rs`
- [x] T036 [US4] Wire mutation boundary into evidence packet in `evidence.rs`
- [x] T037 [US4] Add unit tests: file created, modified, deleted, no changes (read-only), truncated (>10K), filesystem error in `crates/boundline-core/src/execution/mutation.rs`

**Depends on**: Phase 3 (evidence)

**Verification**: `cargo test -p boundline-core -- mutation`

---

## Phase 6: User Story 5 — Governance Hooks (P3)

- [x] T038 [US5] Implement governance hook model: trigger_intents, trigger_zones, action (block/require-approval/log) in `crates/boundline-core/src/execution/hooks.rs`
- [x] T039 [US5] Implement hook evaluation after intent classification + policy resolution in `hooks.rs`
- [x] T040 [US5] Implement require-approval flow: block execution, generate approval request, record decision in evidence packet
- [x] T041 [US5] Implement hook event logging to `.boundline/traces/governance/` in `hooks.rs`
- [x] T042 [US5] Add unit tests: deploy intent triggers block, red-zone mutate triggers block, approval recorded in trace, unknown intent passes through hooks in `crates/boundline-core/src/execution/hooks.rs`

**Depends on**: Phase 2 (classifier), Phase 4 (redaction), Phase 5 (mutation)

**Verification**: `cargo test -p boundline-core -- hooks`

---

## Phase 7: Execution Policy Matrix

- [x] T043 [P] Implement `.boundline/execution-policy.toml` loader with Intent × Zone matrix parsing in `crates/boundline-core/src/execution/policy.rs`
- [x] T044 [P] Implement command override parsing and application in `policy.rs`
- [x] T045 [P] Implement policy resolution pipeline: classify → overrides → matrix → safety flags → final mode in `policy.rs`
- [x] T046 [P] Implement default policy generation (write reasonable defaults on first use) in `policy.rs`
- [x] T047 [P] Implement `PolicyDecision` recording (matched entry, override, escalations, rationale) in `policy.rs`
- [x] T048 Add unit tests: allow, deny, dry-run, require-approval per zone; missing policy defaults; override precedence; safety escalation in `crates/boundline-core/src/execution/policy.rs`

**Depends on**: Phase 1 (types)

**Verification**: `cargo test -p boundline-core -- policy`

---

## Phase 8: CLI Integration — `boundline exec`

- [x] T049 [US1-US5] Implement `boundline exec` CLI command with `--dry-run`, `--no-mutation`, `--classify-only`, `--zone`, `--json` flags in `src/cli/exec.rs`
- [x] T050 [US1-US5] Wire CLI → classifier → policy → dry-run → shell execution → evidence → redaction → persistence pipeline in `exec.rs`
- [x] T051 [US1-US5] Implement `--classify-only` output (intent + policy decision, no execution)
- [x] T052 [US1-US5] Implement `--json` output (full EvidencePacket to stdout)
- [x] T053 [US1-US5] Register `exec` subcommand in Boundline CLI command tree in `src/cli.rs`
- [x] T054 Add CLI contract test: `boundline exec --help`, `boundline exec --classify-only "rm -rf"`, exit codes per contract in `src/cli/exec.rs` #[cfg(test)]

**Depends on**: Phase 2-7

**Verification**: `cargo test --test execution_safety_contract && cargo run -- exec --help`

---

## Phase 9: Evidence Limits Configuration

- [x] T055 Implement `.boundline/evidence-limits.toml` loader with configurable caps in `crates/boundline-core/src/execution/evidence.rs`
- [x] T056 Add unit test: custom limits override defaults in `evidence.rs`

**Depends on**: Phase 3

---

## Phase 10: Polish & Edge Cases

- [x] T057 Handle SIGKILL/timeout: capture partial output, record signal in evidence in `evidence.rs`
- [x] T058 Handle empty stdout/stderr gracefully in `evidence.rs`
- [x] T059 Ensure concurrent execution produces independent traces (no shared mutable state) in `evidence.rs`
- [x] T060 Add edge case tests: SIGKILL simulation, empty output, concurrent traces in respective test modules

**Depends on**: Phase 3, Phase 5

---

## Phase 11: Integration & Full Pipeline Test

- [x] T061 Full pipeline integration test: `boundline exec --dry-run "rm test.txt"` → verify no deletion, dry_run_status=plan_only, evidence packet correct in `tests/integration/exec_command_integration.rs`
- [x] T062 Full pipeline integration test: `boundline exec "echo secret: gh_token_abc123"` → verify redacted in trace in `tests/integration/exec_command_integration.rs`
- [x] T063 Full pipeline integration test: mutation boundary after `boundline exec "echo data > file.txt"` in `tests/integration/exec_command_integration.rs`

**Depends on**: Phase 8, Phase 10

---

## Phase 12: Roadmap & Docs Synchronization

- [x] T064 Copy roadmap seed to spec folder: `roadmap/features/13-safe-command-execution.md` → `specs/077-safe-command-execution/feat-safe-command-execution.md`
- [x] T066 Update `roadmap/features/README.md` sequencing table: mark S13 as "In Progress (spec 077)" from "Next"
- [x] T069 Update `CHANGELOG.md` with entry for this version

**Depends on**: Phase 11

---

## Final Phase: Release, Quality, And Verification

- [x] T070 Update Cargo.toml version according to Boundline versioning policy: `0.76.0` → `0.77.0` in `Cargo.toml`
- [x] T072 Run `cargo fmt`
- [x] T073 Run `cargo clippy --workspace --all-targets -- -D warnings` and fix all warnings
- [x] T074 Run `cargo test` and verify tests pass (780+ pass)
- [x] T076 Run `scripts/check-no-local-paths.sh`
- [x] T077 Run `scripts/check-rust-no-panic.sh`

---

## Task Summary

| Phase | Tasks | Story/FR |
|-------|-------|----------|
| Phase 1: Setup | T001-T008 | Domain types |
| Phase 2: US1 Classify & Dry-Run | T009-T016 | FR-001, FR-002, FR-002a |
| Phase 3: US2 Evidence | T017-T024 | FR-003, FR-004, FR-008 |
| Phase 4: US3 Redaction | T025-T031 | FR-005, FR-010 |
| Phase 5: US4 Mutation | T032-T037 | FR-006 |
| Phase 6: US5 Hooks | T038-T042 | FR-009 |
| Phase 7: Policy Matrix | T043-T048 | FR-007 |
| Phase 8: CLI Integration | T049-T054 | FR-011, FR-012 |
| Phase 9: Limits Config | T055-T056 | FR-003, FR-006 |
| Phase 10: Polish | T057-T060 | Edge cases |
| Phase 11: Integration | T061-T063 | SC-001 to SC-005 |
| Phase 12: Docs Sync | T064-T069 | Wrapper Rule 2-3 |
| Final Phase | T070-T078 | Wrapper Rule 1, 4 |

**Total**: 78 tasks

### Wrapper Rule Compliance

| Rule | Task IDs |
|------|----------|
| Rule 1: Cargo Version Bump | T070 |
| Rule 2: Docs & Roadmap Sync | T064-T069 |
| Rule 3: Docs Version Sync | T071 |
| Rule 4: Quality, Coverage, Clippy, Fmt | T072-T078 |
