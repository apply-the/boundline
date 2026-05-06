# Tasks: Native Canon CLI Surface

**Input**: Design documents from `/specs/042-native-canon-cli/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are included for all user stories.  This feature
changes governance surfaces, CLI commands, config commands, install diagnostics,
and assistant command packs—all requiring executable validation with coverage for
route explanation, mode selection, approval state, Canon surface verification,
input assembly, and CLI-visible governance lifecycle surfaces.

**Organization**: Tasks are grouped by user story so each slice can deliver
bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Default Boundline layout**: `src/`, `tests/` at repository root
- Multi-crate structure: `boundline-cli`, `boundline-core`, `boundline-adapters` under `crates/`; shared source at `src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: CanonMode expansion and shared workspace resolution that all user stories depend on

- [x] T001 Extend the `CanonMode` enum in `src/domain/governance.rs` with seven new variants: `SystemShaping`, `Refactor`, `Review`, `Incident`, `SystemAssessment`, `Migration`, `SupplyChainAnalysis`; update `Display` and `FromStr` impls to use kebab-case serialization (`system-shaping`, `refactor`, `review`, `incident`, `system-assessment`, `migration`, `supply-chain-analysis`); update serde Serialize/Deserialize to match; retain `PrReview` for backward compatibility
- [x] T002 Add a canonical mode-to-document mapping function `canon_mode_primary_document(mode: &CanonMode) -> &str` in `src/domain/governance.rs` returning the primary document name for each of the 15 canonical modes (e.g., `CanonMode::Requirements` → `"requirements.md"`, `CanonMode::SupplyChainAnalysis` → `"supply-chain-analysis.md"`)
- [x] T003 Update `supported_canon_modes_for_stage()` in `src/domain/governance.rs` to include new modes in the stage-to-mode mapping tables where applicable
- [x] T004 Create `src/cli/workspace.rs` implementing the shared workspace resolution function `resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf>` with the spec-required upward search: explicit `--workspace` → upward search for `.boundline/` parent → nearest `.git/` root → CWD fallback; add module declaration in `src/cli.rs`
- [x] T005 [P] Add the `CANONICAL_MODES` constant (array of the 15 canonical `CanonMode` values, excluding legacy `PrReview`) in `src/domain/governance.rs` for use in diagnostics and capabilities verification

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain types, config schema, and session lifecycle structures that MUST be complete before any user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Add `CanonModeSelectionPreference` enum (`Manual`, `AutoConfirm`, `Auto`) with serde serialization as `"manual"`, `"auto-confirm"`, `"auto"` in `src/domain/governance.rs`
- [x] T007 Add `CanonPreferences` struct (`mode_selection: CanonModeSelectionPreference`, `default_risk: Option<String>`, `default_zone: Option<String>`, `default_owner: Option<String>`, `default_system_context: Option<String>`) with serde derives in `src/domain/configuration.rs`
- [x] T008 Extend `ConfigFile` in `src/domain/configuration.rs` with `canon: Option<CanonPreferences>` field; update serde and default impls
- [x] T009 Add `CanonSurfaceVerification` struct (`canon_path: PathBuf`, `version_compatible: bool`, `operations_verified: bool`, `missing_operations: Vec<String>`, `modes_verified: bool`, `missing_modes: Vec<CanonMode>`, `unsupported_modes: Vec<String>`, `capability_snapshot: Option<CanonCapabilitySnapshot>`, `ready: bool`, `repair_actions: Vec<String>`) with serde derives in `src/domain/distribution.rs`
- [x] T010 Extend `CanonInstallStatus` in `src/domain/distribution.rs` with `surface_verification: Option<CanonSurfaceVerification>` field
- [x] T011 [P] Extend `GovernanceIntent` in `src/domain/brief.rs` with `explicit_mode: Option<CanonMode>` and `explicit_no_canon: bool` fields; update default impls and serde
- [x] T012 Add `GovernedDocumentRef` struct (`stage_key: String`, `canon_mode: CanonMode`, `packet_ref: String`, `document_path: Option<String>`, `readiness: PacketReadiness`) with serde derives in `src/domain/governance.rs`
- [x] T013 Add `GovernedSessionLifecycle` struct (`governance_runtime: GovernanceRuntimeKind`, `explicit_opt_out: bool`, `mode_selection_preference: CanonModeSelectionPreference`, `selected_mode: Option<CanonMode>`, `selected_mode_sequence: Vec<CanonMode>`, `current_stage_index: usize`, `stage_records: Vec<GovernedStageRecord>`, `accumulated_context: Vec<GovernedDocumentRef>`, `terminal_reason: Option<String>`) with serde derives in `src/domain/governance.rs`
- [x] T014 Extend `ActiveSessionRecord` in `src/domain/session.rs` with `governance_lifecycle: Option<GovernedSessionLifecycle>` field; update serde and default impls
- [x] T015 Update `FileConfigStore` in `src/adapters/config_store.rs` to serialize/deserialize the expanded `ConfigFile` with the `[canon]` TOML section; verify round-trip for configs with and without canon preferences
- [x] T016 Migrate all CLI commands that accept `--workspace` (`init`, `run`, `status`, `next`, `inspect`, `config`, `doctor`, `capture`, `plan`) to use the shared `resolve_workspace()` from `src/cli/workspace.rs` instead of their local implementations
- [x] T017 Implement `verify_canon_surface()` in `src/domain/distribution.rs` as shared foundation, taking a `CanonCapabilitySnapshot` and returning `CanonSurfaceVerification`: verify operations include `"start"` and `"refresh"`, verify `supported_modes` contains all 15 `CANONICAL_MODES`, compute `missing_modes`, `missing_operations`, `unsupported_modes`, set `ready`, and generate `repair_actions`
- [x] T018 Extend `evaluate_canon_install()` in `src/domain/distribution.rs` to call `query_canon_capabilities()` after version check passes, call `verify_canon_surface()`, and attach the result as `surface_verification` on `CanonInstallStatus`

**Checkpoint**: Foundation ready — all domain types, config schema, session lifecycle, workspace resolution, and shared Canon surface verification in place

---

## Phase 3: User Story 1 — Bootstrap Canon-Default Governed Work From Scratch (Priority: P1) 🎯 MVP

**Goal**: Operators can run `boundline run --goal "<goal>"` on a Canon-ready workspace and reach Canon-governed execution by default, without `--governance canon` or editing manifests

**Independent Test**: In a clean workspace with Canon preferences in config and a verified Canon surface, run native direct run with a goal and verify session enters Canon governance with the correct mode-selection preference

### Tests for User Story 1

- [x] T019 [P] [US1] Unit test in `tests/unit/` verifying `CanonModeSelectionPreference` serde round-trip for all three variants (`manual`, `auto-confirm`, `auto`)
- [x] T020 [P] [US1] Unit test in `tests/unit/` verifying `CanonPreferences` TOML serialization round-trip including defaults
- [x] T021 [P] [US1] Unit test in `tests/unit/` verifying `GovernedSessionLifecycle` JSON serialization round-trip including all fields
- [x] T022 [P] [US1] Unit test in `tests/unit/` verifying `GovernanceIntent` with `explicit_mode` and `explicit_no_canon` fields
- [x] T023 [P] [US1] Integration test in `tests/integration/` verifying `boundline run --goal "<goal>"` on a workspace with `[canon]` config defaults to Canon governance runtime (mock Canon CLI returning `governed_ready`)
- [x] T024 [P] [US1] Integration test in `tests/integration/` verifying `boundline run --no-canon --goal "<goal>"` on a Canon-ready workspace falls back to local governance and projects opt-out in session state
- [x] T025 [P] [US1] Integration test in `tests/integration/` verifying `boundline run --goal "<goal>"` on a workspace without `[canon]` config uses local governance (backward compatibility)
- [x] T026 [P] [US1] Integration test in `tests/integration/` verifying `boundline run --goal "<goal>"` on a workspace with `[canon]` config but incomplete Canon surface stops before governed execution, reports `repair_actions`, and suggests `boundline doctor --install`

### Implementation for User Story 1

- [x] T027 [US1] Add `--mode <canon-mode>` to the `DeveloperCommand::Run` clap definition in `src/cli.rs`, accepting any valid `CanonMode` string; wire the parsed value into `src/cli/run.rs`
- [x] T028 [US1] Add `--no-canon` to the `DeveloperCommand::Run` clap definition in `src/cli.rs` and handle it in `src/cli/run.rs` as an alias for `--governance local`
- [x] T029 [US1] Implement Canon-default governance resolution in `src/cli/run.rs`: after workspace resolution, load workspace config; if `config.canon` is present and `evaluate_canon_install().surface_verification.ready` is true, set governance runtime to Canon; if `surface_verification.ready` is false, stop before execution with an explicit error containing `repair_actions` and a `boundline doctor --install` hint; if `--no-canon` or `--governance local`, override to Local; otherwise default to Local for backward compatibility
- [x] T030 [US1] Wire `GovernanceIntent.explicit_mode` and `GovernanceIntent.explicit_no_canon` through the capture/plan/run pipeline: populate from CLI flags in `src/cli.rs`/`src/cli/run.rs`, propagate through `execute_native_direct_run()` into session state
- [x] T031 [US1] Implement governance default application in `src/orchestrator/governance.rs`: when `GovernanceIntent.explicit_no_canon` is false and workspace has `CanonPreferences`, apply `default_risk`, `default_zone`, `default_owner` from config as fallbacks for any fields not supplied by the operator (FR-005a)
- [x] T032 [US1] Create `GovernedSessionLifecycle` in `src/orchestrator/session_runtime.rs` when a Canon-governed run starts: set `governance_runtime = Canon`, populate `mode_selection_preference` from workspace config, set `explicit_opt_out` from intent; persist to session record
- [x] T033 [US1] Extend session `status`/`next`/`inspect` output in `src/cli/session.rs` to project governance lifecycle fields: `governance_runtime`, `explicit_opt_out`, `mode_selection_preference`, `selected_mode`, `lifecycle_state`, `approval_state`, `next_action`
- [x] T034 [US1] Implement mode-selection gate in `src/orchestrator/governance.rs`: when `CanonModeSelectionPreference::Manual` and no explicit mode → return `PendingSelection` error with message "Canon mode-selection is manual; specify --mode <mode>"; when `AutoConfirm` and no explicit mode → infer mode from evidence and return confirmation prompt; when `Auto` → infer mode and proceed or fall back to confirmation

**Checkpoint**: User Story 1 complete — `boundline run` defaults to Canon on Canon-ready workspaces; opt-out is explicit; mode-selection preference is enforced

---

## Phase 4: User Story 2 — Move From Ingested Evidence To Canon-Ready Inputs (Priority: P2)

**Goal**: Operators supply briefs, PRD, architecture docs, and answers to clarification questions; Boundline assembles Canon-ready `input_documents` and `bounded_context` transparently

**Independent Test**: Supply a goal plus multiple `--brief` paths including a PRD and architecture doc, verify the assembled Canon governance request contains correctly typed `input_documents` and `bounded_context.reused_packets` from any prior governed stage

### Tests for User Story 2

- [x] T035 [P] [US2] Unit test in `tests/unit/` verifying `governance_input_documents()` maps operator briefs to Canon `input_documents` with correct `kind` tags (`stage-brief`, `authored-brief`)
- [x] T036 [P] [US2] Unit test in `tests/unit/` verifying clarification answers are assembled as `input_documents` with `kind = "clarification-answer"`
- [x] T037 [P] [US2] Unit test in `tests/unit/` verifying `bounded_governance_context()` includes `reused_packets` from `GovernedSessionLifecycle.accumulated_context` for multi-stage journeys
- [x] T038 [P] [US2] Integration test in `tests/integration/` verifying `boundline run --goal "<goal>" --brief docs/prd.md --brief docs/arch.md` assembles a Canon governance start request with the correct `input_documents` array and `bounded_context` fields (mock Canon CLI)
- [x] T039 [P] [US2] Integration test in `tests/integration/` verifying that when Canon returns `incomplete` with `missing_sections`, Boundline surfaces a clarification prompt to the operator rather than failing silently

### Implementation for User Story 2

- [x] T040 [US2] Extend `governance_input_documents()` in `src/orchestrator/governance.rs` to include clarification answers from `ClarificationRecord` as additional `GovernanceInputDocument` entries with `kind = "clarification-answer"`
- [x] T041 [US2] Extend `bounded_governance_context()` in `src/orchestrator/governance.rs` to populate `reused_packets` from `GovernedSessionLifecycle.accumulated_context` when the session has prior governed stage results (FR-020)
- [x] T042 [US2] Implement governed document forwarding: after a Canon `start` returns `governed_ready`, create a `GovernedDocumentRef` from the response packet and append it to `GovernedSessionLifecycle.accumulated_context`; persist updated session
- [x] T043 [US2] Handle Canon `incomplete` and `pending_selection` responses in `src/orchestrator/governance.rs`: extract `missing_sections` or unresolved mode choice from the response and surface as a `ClarificationRequired` state in the session with targeted prompt text (FR-019)
- [x] T044 [US2] Handle Canon `awaiting_approval` response: persist approval state in `GovernedSessionLifecycle`, set session status to show approval-pending with next action "run `boundline run` to refresh" or "wait for approval"; project through `status`/`next` (FR-021)
- [x] T045 [US2] Implement governed session refresh: when `boundline run` is invoked on a session with `awaiting_approval` lifecycle state, invoke Canon `refresh` (not `start`) with the existing `run_ref`, update approval state and packet readiness from the response

**Checkpoint**: User Story 2 complete — operator inputs are transparently assembled into Canon requests; multi-stage forwarding and non-success states are handled

---

## Phase 5: User Story 3 — Keep CLI And Assistant Surfaces Aligned (Priority: P3)

**Goal**: All assistant command packs expose the same Canon-default workflow as the CLI; no assistant tells the operator to edit manifests manually

**Independent Test**: Compare the CLI path `boundline run --goal` with the assistant command pack instructions for `/boundline-run` across Copilot, Codex, Claude, and Gemini; verify both surfaces reference the same governance fields, mode-selection, and follow-through guidance

### Tests for User Story 3

- [x] T046 [P] [US3] Contract test in `tests/contract/` verifying that each Copilot prompt file references `boundline run` (not `--governance canon`), includes Canon-default behavior, and does not instruct manual manifest editing
- [x] T047 [P] [US3] Contract test in `tests/contract/` verifying assistant command packs map `/boundline-run requirements` and `/boundline-requirements` to the same canonical CLI invocation (`boundline run --mode requirements`) and require the same governance lifecycle output fields as the CLI (`governance_runtime`, `mode_selection_preference`, `selected_mode`, `approval_state`, `next_action`)

### Implementation for User Story 3

- [x] T048 [P] [US3] Create `assistant/copilot/prompts/boundline-init.prompt.md` for the `/boundline-init` command mapping to `boundline init` with guided Canon mode-selection, assistant, and model route collection
- [x] T049 [P] [US3] Create `assistant/copilot/prompts/boundline-doctor.prompt.md` for the `/boundline-doctor` command mapping to `boundline doctor --install` with Canon surface verification output interpretation
- [x] T050 [P] [US3] Create `assistant/copilot/prompts/boundline-config-show.prompt.md` for the `/boundline-config-show` command mapping to `boundline config show`
- [x] T051 [P] [US3] Create `assistant/copilot/prompts/boundline-config-set-canon.prompt.md` for the `/boundline-config-set-canon` command mapping to `boundline config set-canon --mode-selection <preference>`
- [x] T052 [P] [US3] Create `assistant/copilot/prompts/boundline-capture.prompt.md` for the `/boundline-capture` command mapping to `boundline capture --goal --brief`
- [x] T053 [US3] Update `assistant/copilot/prompts/boundline-run.prompt.md` to document Canon-default behavior, `--mode`, `--no-canon`, governance field flags, and mode-selection preference semantics; remove any language suggesting manual manifest editing as the primary path
- [x] T054 [US3] Update `assistant/copilot/prompts/boundline-status.prompt.md` to include governance lifecycle output interpretation: `governance_runtime`, `mode_selection_preference`, `selected_mode`, `approval_state`, `blocked_reason`, `next_action`
- [x] T055 [US3] Update `assistant/copilot/prompts/boundline-next.prompt.md` and `assistant/copilot/prompts/boundline-inspect.prompt.md` to include governance lifecycle output fields and Canon-specific follow-through guidance
- [x] T056 [P] [US3] Create mode-specific alias prompt files in `assistant/copilot/prompts/` for the 15 canonical modes: `boundline-requirements.prompt.md`, `boundline-discovery.prompt.md`, `boundline-system-shaping.prompt.md`, `boundline-architecture.prompt.md`, `boundline-backlog.prompt.md`, `boundline-change.prompt.md`, `boundline-implementation.prompt.md`, `boundline-refactor.prompt.md`, `boundline-review.prompt.md`, `boundline-verification.prompt.md`, `boundline-incident.prompt.md`, `boundline-security-assessment.prompt.md`, `boundline-system-assessment.prompt.md`, `boundline-migration.prompt.md`, `boundline-supply-chain-analysis.prompt.md`; each maps to `boundline run --mode <mode>`
- [x] T057 [US3] Update `assistant/codex/` command files to mirror the same Canon-default workflow, new commands, and mode aliases as Copilot prompts
- [x] T058 [US3] Update `assistant/claude/` command files to mirror the same Canon-default workflow, new commands, and mode aliases as Copilot prompts
- [x] T059 [US3] Update `assistant/gemini/` to add equivalent command documentation for Canon-default workflow and mode aliases

**Checkpoint**: User Story 3 complete — all assistant surfaces expose the same Canon-default path as the CLI

---

## Phase 6: User Story 4 — Configure Canon Autonomy And Models During Init (Priority: P4)

**Goal**: Guided `boundline init` collects Canon mode-selection preference and model routes; config commands allow inspection and mutation

**Independent Test**: Run guided `boundline init`, choose `auto-confirm` and model routes, verify `config show` reports the same settings; change mode-selection via `config set-canon`, verify the new value is used

### Tests for User Story 4

- [x] T060 [P] [US4] Integration test in `tests/integration/` verifying `boundline init --canon-mode-selection auto-confirm --assistant copilot --route planning=copilot:gpt-4o` writes correct `[canon]` and `[routing]` sections to `.boundline/config.toml`
- [x] T061 [P] [US4] Integration test in `tests/integration/` verifying `boundline config show` on an initialized workspace reports Canon preferences and model routes
- [x] T062 [P] [US4] Integration test in `tests/integration/` verifying `boundline config set-canon --mode-selection auto` updates the config file and subsequent `config show` reflects the change
- [x] T063 [P] [US4] Automated guided-init test in `tests/integration/` or a pure unit test around the prompt collector verifying TTY answers (`auto-confirm`, assistant surface, model route) produce the same `[canon]` and `[routing]` config as the non-interactive flags

### Implementation for User Story 4

- [x] T064 [US4] Extend `boundline init` CLI args in `src/cli.rs` and execution handling in `src/cli/init.rs` with `--canon-mode-selection <manual|auto-confirm|auto>`, `--risk <risk>`, `--zone <zone>`, `--owner <owner>` flags
- [x] T065 [US4] Implement guided init flow in `src/cli/init.rs`: when `--canon-mode-selection` is not provided and stdin is a TTY, prompt the operator for Canon mode-selection preference (display choices with descriptions), assistant surfaces, and model routes; when not a TTY, allow explicit flags for scripted setup
- [x] T066 [US4] Wire init to write `CanonPreferences` to workspace `.boundline/config.toml` via `FileConfigStore`: merge with existing config if present, create `[canon]` section with chosen mode-selection and governance defaults
- [x] T067 [US4] Add `set-canon` to the `ConfigSubcommand` clap enum in `src/cli.rs` and implement execution in `src/cli/config.rs`, accepting `--mode-selection <manual|auto-confirm|auto>` and `--workspace`; load existing config, update `canon.mode_selection`, persist
- [x] T068 [US4] Extend `boundline config show` execution in `src/cli/config.rs` to display `[canon]` section including `mode_selection`, `default_risk`, `default_zone`, `default_owner` when present

**Checkpoint**: User Story 4 complete — init collects Canon preferences, config commands inspect and mutate them

---

## Phase 7: User Story 5 — Verify The Real Canon Surface Before Work Starts (Priority: P5)

**Goal**: Install diagnostics verify Canon governance operations and canonical modes, not just version; report authoritative binary path and repair guidance

**Independent Test**: With a Canon binary whose version matches but whose capabilities response is missing a mode, verify `boundline doctor --install` marks the install as not ready and reports the missing mode plus repair guidance

### Tests for User Story 5

- [x] T069 [P] [US5] Unit test in `tests/unit/` verifying `CanonSurfaceVerification` correctly marks `ready = false` when `missing_operations` is non-empty
- [x] T070 [P] [US5] Unit test in `tests/unit/` verifying `CanonSurfaceVerification` correctly marks `ready = false` when `missing_modes` is non-empty and `ready = true` when all 15 modes and both operations are present
- [x] T071 [P] [US5] Integration test in `tests/integration/` verifying `boundline doctor --install` with a mock Canon binary that reports correct version but missing `governance start` operation → reports `canon_governance_surface` check as failed with repair guidance
- [x] T072 [P] [US5] Integration test in `tests/integration/` verifying `boundline doctor --install` with a full-featured mock Canon binary → all canon checks pass and `canon_path` is reported

### Implementation for User Story 5

- [x] T073 [US5] Add mock Canon capability fixtures in `tests/fixtures/` for full surface, missing operation, and missing mode cases; reuse them across `doctor --install` tests and runtime gating tests
- [x] T074 [US5] Harden `CanonSurfaceVerification.repair_actions` in `src/domain/distribution.rs` so missing operations, missing modes, incompatible version, and missing capability snapshots each produce actionable, CLI-safe repair guidance
- [x] T075 [US5] Extend `boundline doctor --install` in `src/cli/diagnostics.rs` to add three new diagnostic checks: `canon_governance_surface` (operations verified), `canon_modes` (all canonical modes present), and `canon_path` (authoritative binary path); populate from `CanonInstallStatus.surface_verification`

**Checkpoint**: User Story 5 complete — install diagnostics verify the real Canon governance surface; incompatible binaries are rejected before governed work starts

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, coverage, assistant README, and cross-story integration hardening

- [x] T076 [P] Update `docs/getting-started.md` to document Canon-default workspace setup, guided init, and the primary governed development workflow
- [x] T077 [P] Update `docs/configuration.md` to document the `[canon]` config section, mode-selection preferences, and `config set-canon` command
- [x] T078 [P] Update `assistant/README.md` to list all new assistant commands (`/boundline-init`, `/boundline-doctor`, `/boundline-config-show`, `/boundline-config-set-canon`, `/boundline-capture`, mode aliases) and document the Canon-default path as primary
- [x] T079 [P] Update `CHANGELOG.md` with feature 042 entry covering Canon-default governance, CanonMode expansion, config commands, install diagnostics surface verification, and assistant command parity
- [x] T080 [P] Add unit tests in `tests/unit/` for `resolve_workspace()` covering: explicit path, `.boundline/` upward search, git root fallback, CWD fallback, and ambiguous-workspace error
- [x] T081 [P] Add unit tests in `tests/unit/` for all seven new `CanonMode` variants: `Display`, `FromStr` round-trip, serde round-trip, and `canon_mode_primary_document()` mapping
- [x] T082 [P] Add integration test in `tests/integration/` for end-to-end multi-stage governed journey: first mode produces `governed_ready` → document ref accumulated → second mode receives `reused_packets` from prior stage (mock Canon CLI)
- [x] T083 Run `cargo fmt --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` to verify all changes pass formatting and lint
- [x] T084 Run `cargo nextest run` to verify all tests pass
- [x] T085 Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` to refresh coverage artifacts
- [x] T086 Run `specs/042-native-canon-cli/quickstart.md` validation: execute the documented commands against a mock workspace and verify each step produces expected output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (Phase 1) completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (Phase 2) — MVP target
- **User Story 2 (Phase 4)**: Depends on Foundational (Phase 2); integrates with US1 session state but is independently testable
- **User Story 3 (Phase 5)**: Depends on Foundational (Phase 2); references CLI commands from US1/US2 but assistant files are independently authorable
- **User Story 4 (Phase 6)**: Depends on Foundational (Phase 2); uses config types from Phase 2; can run in parallel with US1
- **User Story 5 (Phase 7)**: Depends on Foundational (Phase 2); uses shared Canon surface verification from Phase 2; can run in parallel with US1 because runtime gating is already covered by US1
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational — no dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational — uses `GovernedSessionLifecycle` from US1 for multi-stage forwarding, but testable independently with fixture data
- **User Story 3 (P3)**: Can start after Foundational — references CLI command shapes from US1 but assistant file authoring is independent
- **User Story 4 (P4)**: Can start after Foundational — uses `CanonPreferences` config types, fully independent from other stories
- **User Story 5 (P5)**: Can start after Foundational — uses shared `CanonSurfaceVerification` and `evaluate_canon_install()` behavior, fully independent from other stories

### Within Each User Story

- Tests MUST fail before implementation when the spec requires executable behavior
- Domain types before orchestrator logic
- Orchestrator logic before CLI wiring
- CLI wiring before session projection
- Trace and failure-handling coverage before story sign-off
- Story complete before moving to next priority

### Parallel Opportunities

- Setup tasks T001–T003 are sequential (enum depends on prior); T004 and T005 are parallel with T003
- Foundational tasks T009 and T010 are sequential because both touch `src/domain/distribution.rs`; T011 remains parallel with them because it touches `src/domain/brief.rs`
- US1 tests T019–T026 are all parallel (different test files)
- US3 tasks T048–T052 and T056 are parallel (independent prompt files)
- US4 and US5 can run in parallel with US1 after Foundational completes
- All polish tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all US1 tests together (all [P]):
T019: Unit test CanonModeSelectionPreference serde
T020: Unit test CanonPreferences TOML round-trip
T021: Unit test GovernedSessionLifecycle JSON round-trip
T022: Unit test GovernanceIntent new fields
T023: Integration test Canon-default run
T024: Integration test --no-canon opt-out
T025: Integration test backward-compatible local governance
T026: Integration test incomplete Canon surface blocks run with repair guidance

# After tests exist and fail, implement in order:
T027 → T028 → T029 → T030 → T031 → T032 → T033 → T034
```

## Parallel Example: User Story 3

```bash
# Launch all new assistant file creation together (all [P]):
T048: boundline-init.prompt.md
T049: boundline-doctor.prompt.md
T050: boundline-config-show.prompt.md
T051: boundline-config-set-canon.prompt.md
T052: boundline-capture.prompt.md
T056: 15 mode-specific alias prompt files

# Then update existing files sequentially:
T053 → T054 → T055 → T057 → T058 → T059
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (CanonMode expansion, workspace resolution)
2. Complete Phase 2: Foundational (domain types, config schema, session lifecycle)
3. Complete Phase 3: User Story 1 (Canon-default run, opt-out, mode-selection gate)
4. **STOP and VALIDATE**: Test US1 independently — `boundline run --goal` on Canon-ready workspace
5. Deploy/demo if ready — operators can bootstrap Canon-governed work without manifests

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 5 → Test independently → Install diagnostics verified (high value, low risk)
4. Add User Story 4 → Test independently → Config commands available
5. Add User Story 2 → Test independently → Input assembly and multi-stage forwarding
6. Add User Story 3 → Test independently → Assistant parity
7. Polish → Coverage, docs, quickstart validation

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (MVP Canon-default run)
   - Developer B: User Story 5 (Install diagnostics) + User Story 4 (Config commands)
   - Developer C: User Story 3 (Assistant command packs)
3. After US1 is done:
   - Developer A: User Story 2 (Input assembly, multi-stage forwarding)
4. Polish phase: all developers
