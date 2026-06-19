# Tasks: AI Gateway And Inference Economics

**Input**: Design documents from `/specs/081-ai-gateway-economics/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

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

**Purpose**: Project initialization, new dependency, module skeleton

- [x] T001 Add `rust_decimal` dependency to workspace `Cargo.toml` and re-export from `crates/boundline-core/Cargo.toml`
- [x] T002 [P] Create `src/domain/inference_economics.rs` with module-level doc comment and empty placeholder types
- [x] T003 Declare `pub mod inference_economics` in `crates/boundline-core/src/domain.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and configuration that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Define `Currency` enum (Usd, Eur, ...) with `Default` (Usd) and `Serialize`/`Deserialize` derives in `src/domain/inference_economics.rs`
- [x] T005 [P] Define `MonetaryAmount` newtype wrapping `Decimal` with `FromStr` for decimal-string parsing, exact arithmetic ops, and `Serialize`/`Deserialize` in `src/domain/inference_economics.rs`
- [x] T006 [P] Define `CostBasis` enum (ExactOnly, Estimated, Mixed) in `src/domain/inference_economics.rs`
- [x] T007 [P] Define `BudgetState` enum (InBounds, ApproachingLimit, ApprovalRequired, Exhausted, Disabled) in `src/domain/inference_economics.rs`
- [x] T008 [P] Define `ApprovalType` enum (UnknownCostApproval, BudgetOverride) and `ApprovalScope` enum (SingleCall, BoundedTask, BoundedSession) with `Default` (SingleCall) in `src/domain/inference_economics.rs`
- [x] T009 [P] Define `SnapshotState` enum (Current, Stale, Missing, Invalid), `ReservationCostQuality` enum (CurrentEstimate, StaleEstimate, Unknown), and `ReconciledCostQuality` enum (Exact, Estimated, Unknown, LocalZeroMarginalCost) in `src/domain/inference_economics.rs`
- [x] T010 Define `InferenceEconomicsConfig` struct with `session_budget` (Option<SessionBudgetConfig>), `staleness_threshold_days` (u32, default 30), `unknown_cost_policy` (UnknownCostPolicy enum: Block, RequireApproval, AllowWithWarning), and `approval_default_scope` (ApprovalScope) in `src/domain/inference_economics.rs`
- [x] T011 Define `SessionBudgetConfig` struct with `currency` (Currency), `limit` (Decimal) in `src/domain/inference_economics.rs`
- [x] T012 Add `[inference_economics]` section support to `ConfigFile` in `src/domain/configuration.rs` with optional `InferenceEconomicsConfig` field and `#[serde(default, skip_serializing_if = "Option::is_none")]`
- [x] T013 Implement `validate()` for `InferenceEconomicsConfig` in `src/domain/inference_economics.rs` — rejects negative limits, invalid staleness thresholds, and unknown currencies
- [x] T014 Add config serialization support in `crates/boundline-adapters/src/config_store.rs` for the new `[inference_economics]` section (handled by serde attributes on ConfigFile — existing store serializes entire ConfigFile generically)
- [x] T015 [P] Write unit tests for `MonetaryAmount` arithmetic, serialization round-trip, and `FromStr` edge cases in `tests/unit/inference_economics.rs` (in-file #[cfg(test)] module)
- [x] T016 [P] Write unit tests for `InferenceEconomicsConfig` validation (negative limits, zero limits, missing currency defaults) in `tests/unit/inference_economics.rs` (in-file #[cfg(test)] module)

**Checkpoint**: Foundation ready — all base types, config schema, and validation in place. User story implementation can now begin.

---

## Phase 3: User Story 1 - Enforce Session Budget (Priority: P1) 🎯 MVP

**Goal**: Track provider-backed inference spend, reserve budget before execution, reconcile final spend after execution, enforce budget limits, and surface budget state in session status.

**Independent Test**: Configure a session budget with `boundline config set-budget --currency USD --limit 10.00`, run provider-backed calls, and verify that `boundline status` shows known spent, reserved, remaining known budget, and that calls exceeding budget are blocked or paused for approval.

### Implementation for User Story 1

- [x] T017 [P] [US1] Define `SessionBudgetProjection` struct with all fields from data-model.md §1 in `crates/boundline-core/src/domain/inference_economics.rs`
- [x] T018 [P] [US1] Define `PricingEntry` struct and `PricingSnapshot` struct with all fields from data-model.md §2 in `crates/boundline-core/src/domain/inference_economics.rs`
- [x] T019 [P] [US1] Define `InvocationCostRecord` struct with all fields from data-model.md §3 in `crates/boundline-core/src/domain/inference_economics.rs`
- [x] T020 [US1] Define `RequiredAction` enum (ApproveUnknownCost, ApproveBudgetOverride, None) in `crates/boundline-core/src/domain/inference_economics.rs`
- [x] T021 [US1] Implement `SessionBudgetProjection::reserve(amount: Decimal, snapshot_id: &str, snapshot_age: Duration) -> Result<(), BudgetError>` — creates conservative reservation, updates reserved + remaining, transitions budget_state
- [x] T022 [US1] Implement `SessionBudgetProjection::reconcile(cost: InvocationCostRecord) -> Result<(), BudgetError>` — replaces reservation with actual cost, updates known_spent, handles unknown/estimated/exact
- [x] T023 [US1] Implement `SessionBudgetProjection::check_admission(amount: Decimal, authority_zone: CanonAuthorityZone) -> AdmissionDecision` — returns InBounds, Blocked(budget_state), or ApprovalRequired(required_role) based on authority zone
- [ ] T024 [US1] Add additive budget fields (`budget_projection: Option<SessionBudgetProjection>`, `invocation_cost_records: Vec<InvocationCostRecord>`) to `ActiveSessionRecord` in `crates/boundline-core/src/domain/session.rs` with `#[serde(default, skip_serializing_if = "...")]`
- [ ] T025 [US1] Implement `BudgetEnforcer` struct in `crates/boundline-adapters/src/budget_enforcer.rs` that wraps `SessionBudgetProjection` and provides `reserve_before_call()`, `reconcile_after_call()`, and `check_admission()` by delegating to the domain methods
- [ ] T026 [US1] Implement `PricingResolver` struct in `crates/boundline-adapters/src/pricing_resolver.rs` that loads the active `PricingSnapshot` from config, looks up `PricingEntry` by provider+model, and checks staleness against the configured threshold
- [ ] T027 [US1] Wire `BudgetEnforcer` and `PricingResolver` into the provider call path in `crates/boundline-adapters/src/provider_runtime.rs` — intercept calls before dispatch to reserve budget via `BudgetEnforcer::reserve_before_call()`, after completion to reconcile cost via `BudgetEnforcer::reconcile_after_call()`.
- [ ] T028 [US1] Add budget projection fields to `SessionStatusView` in `crates/boundline-core/src/domain/session.rs` — known_spent, reserved, remaining_known_budget, unknown_cost_call_count, budget_state
- [ ] T029 [US1] Implement `boundline config set-budget` subcommand in `crates/boundline-cli/src/cli/config.rs` — accepts `--currency` and `--limit` args, writes to `[inference_economics]` config section
- [ ] T030 [US1] Extend `boundline status` output in `crates/boundline-cli/src/cli/status.rs` to render budget projection when a budget is configured — spent, reserved, remaining, budget_state, unknown-cost call count
- [ ] T031 [US1] Implement `boundline config unset-budget` subcommand in `crates/boundline-cli/src/cli/config.rs` — removes budget config, disabling enforcement
- [ ] T032 [US1] Add session persistence for new fields in `crates/boundline-adapters/src/session_store.rs` — serialize/deserialize `budget_projection` and `invocation_cost_records`
- [x] T033 [US1] Write unit tests for `SessionBudgetProjection` state machine (reserve, reconcile exact, reconcile estimated, reconcile unknown, admission check in-bounds, admission check over-budget, admission check low-risk vs red-zone) in `tests/unit/inference_economics.rs`
- [ ] T034 [US1] Write unit tests for `BudgetEnforcer` (reservation blocks when budget exhausted, reconciliation updates totals, unknown-cost increments counter) in `tests/unit/budget_enforcer.rs`
- [ ] T035 [US1] Write unit tests for `PricingResolver` (lookup by provider+model, staleness check current vs stale, missing entry returns unknown) in `tests/unit/budget_enforcer.rs`
- [ ] T036 [US1] Write integration test for config → budget enforcement → status projection round-trip in `tests/integration/inference_economics_cli.rs`
- [ ] T037 [US1] Write integration test for budget-exhaustion scenario — configure budget, exhaust it with calls, verify last call is blocked in `tests/integration/inference_economics_cli.rs`

**Checkpoint**: US1 is fully functional — budget can be configured, enforced, projected in status, and tested end-to-end.

---

## Phase 4: User Story 2 - Route By Risk And Health (Priority: P2)

**Goal**: Route selection considers task risk, authority zone, provider health, budget state, and snapshot staleness. Low-risk work uses cheaper routes; red-zone work is never silently downgraded.

**Independent Test**: Simulate route selection inputs across low-risk/medium-risk/red-zone tasks with varying provider health and budget states, and verify chosen tiers, fallback behavior, and halt behavior when no compliant route exists.

### Implementation for User Story 2

- [ ] T038 [P] [US2] Define `InferenceRouteProfile` struct with tier (Tier0/Tier1/Tier2/Tier3), provider_id, model_id, capability_requirements, and cost_characteristics in `crates/boundline-core/src/domain/inference_economics.rs`
- [ ] T039 [P] [US2] Define `RouteTier` enum (Tier0Deterministic, Tier1Cheap, Tier2Balanced, Tier3HighCapability) in `crates/boundline-core/src/domain/inference_economics.rs`
- [ ] T040 [US2] Extend `ProviderHealthSnapshot` with optional `cost_quota_status` and `route_readiness` fields in `crates/boundline-core/src/domain/capability_provider.rs`
- [ ] T041 [US2] Implement `RouteSelector` struct in `crates/boundline-adapters/src/route_selector.rs` (new file — keep routing logic separate from budget enforcement) with method `select_route(task_class, authority_zone, budget_projection, available_routes, health_snapshots) -> RouteSelection`.
- [ ] T042 [US2] Implement route scoring logic in `RouteSelector`: lower-cost tier preferred for low-risk; capability tier preserved for red-zone; unhealthy routes excluded; stale-snapshot routes flagged but not auto-blocked
- [ ] T043 [US2] Implement fallback policy in `RouteSelector`: when preferred route unavailable, select next-compliant route; when no compliant route exists, return Blocked with explicit reason
- [ ] T044 [US2] Wire `RouteSelector` into the provider dispatch path in `crates/boundline-adapters/src/provider_runtime.rs` — route selection runs before `BudgetEnforcer::reserve_before_call()`.
- [ ] T045 [US2] Implement `boundline inspect route` subcommand in `crates/boundline-cli/src/cli/inspect_cost.rs` (or new `inspect_route.rs`) — shows route selection for hypothetical task with given class and authority zone
- [ ] T046 [US2] Extend `boundline provider status --verbose` to show route health and readiness in `crates/boundline-cli/src/cli/provider.rs`
- [ ] T047 [US2] Write unit tests for `RouteSelector` (low-risk → Tier1 selected, red-zone → Tier3 preserved, unhealthy route excluded, all routes unhealthy → blocked, stale snapshot → flagged but allowed) in `tests/unit/budget_enforcer.rs`
- [ ] T048 [US2] Write integration test for route selection end-to-end — configure routes with different tiers/health, run tasks, verify correct route chosen in `tests/integration/inference_economics_cli.rs`
- [ ] T048a [US2] Implement deterministic Tier 0 route dispatch in `RouteSelector` — when `RouteTier::Tier0Deterministic` is selected, execute locally without provider dispatch and record a zero-cost outcome with `cost_quality = local_zero_marginal_cost` in the `InvocationCostRecord` (FR-041)

**Checkpoint**: US1 + US2 both functional — budget enforcement plus risk-aware route selection.

---

## Phase 5: User Story 3 - Govern Route Changes And Spend Exceptions (Priority: P3)

**Goal**: Route policy changes require evaluation approval; spend exceptions are approved through authority-zone-based mechanism; local/private routes are supported; approval records are auditable.

**Independent Test**: Propose a route-policy change, attempt to activate without evaluation approval, then activate with approval. Configure a local route, verify zero-marginal-cost reporting. Test spend exception approval flows for session owner (low-risk) and governance approver (red-zone) scenarios.

### Implementation for User Story 3

- [ ] T049 [P] [US3] Define `SpendExceptionApprovalRecord` struct with all fields from data-model.md §4 in `crates/boundline-core/src/domain/inference_economics.rs`
- [ ] T050 [P] [US3] Define `SpendExceptionDecisionProjection` struct with all fields from data-model.md §5 in `crates/boundline-core/src/domain/inference_economics.rs`
- [ ] T051 [P] [US3] Define `ApprovalState` enum (Pending, Consumed, Expired, Revoked) and `ApproverRole` enum (SessionOwner, GovernanceApprover) in `crates/boundline-core/src/domain/inference_economics.rs`
- [ ] T052 [US3] Implement `ApprovalManager` struct in `crates/boundline-adapters/src/budget_enforcer.rs` with method `resolve_approver(authority_zone, repository_egress, session_owner_id, governance_policy) -> ApproverRole`
- [ ] T053 [US3] Implement `ApprovalManager::request_approval(call_context, required_role) -> SpendExceptionDecisionProjection` — builds the projection with all required fields for operator display
- [ ] T054 [US3] Implement `ApprovalManager::record_approval(projection, approver_identity, reason, scope) -> SpendExceptionApprovalRecord` — creates the record with timestamp, validates scope, does not create permanent exemption. For repository-egress calls, record spend exception approval and data-transmission authorization as separate fields on the record; spend approval alone MUST NOT authorize content transmission (FR-026).
- [ ] T055 [US3] Implement `ApprovalManager::consume_approval(approval_id, call_id) -> Result<(), ApprovalError>` — marks approval as consumed, rejects reuse beyond scope
- [ ] T056 [US3] Implement `ApprovalManager::expire_approvals()` — marks unconsumed approvals past expiry or session end as Expired
- [ ] T057 [US3] Add `spend_exception_approvals: Vec<SpendExceptionApprovalRecord>` field to `ActiveSessionRecord` in `crates/boundline-core/src/domain/session.rs`
- [ ] T058 [US3] Implement `boundline approve` interactive subcommand in `crates/boundline-cli/src/cli/approve.rs` — displays `SpendExceptionDecisionProjection`, prompts operator for reason and scope, records approval
- [ ] T059 [US3] Implement `boundline approve --reject` subcommand — operator rejects the pending spend exception, call remains blocked
- [ ] T060 [US3] Implement `boundline inspect cost --approvals` subcommand in `crates/boundline-cli/src/cli/inspect_cost.rs` — lists all spend exception approvals with state, scope, and consumption status
- [ ] T061 [US3] Implement `boundline inspect cost --calls` subcommand in `crates/boundline-cli/src/cli/inspect_cost.rs` — lists `InvocationCostRecord` entries with cost quality, snapshot, and approval links
- [ ] T062 [US3] Implement `boundline provider set-pricing --snapshot <file>` subcommand in `crates/boundline-cli/src/cli/provider.rs` — validates and activates a pricing snapshot from a TOML file
- [ ] T063 [US3] Implement `boundline inspect cost --snapshots` subcommand in `crates/boundline-cli/src/cli/inspect_cost.rs` — lists pricing snapshots with state (current/stale/missing/invalid) and effective timestamp
- [ ] T064 [US3] Add local-route support: when a route reports `LocalZeroMarginalCost`, bypass budget reservation but record the cost source explicitly in `InvocationCostRecord`
- [ ] T065 [US3] Add session persistence for `spend_exception_approvals` in `crates/boundline-adapters/src/session_store.rs`
- [ ] T066 [US3] Write unit tests for `ApprovalManager` (session owner approves low-risk non-egress, governance approver required for red-zone, session owner self-approval rejected for red-zone, approval consumed once, unused approval does not carry over, expired approval cannot be consumed) in `tests/unit/inference_economics.rs`
- [ ] T067 [US3] Write unit tests for `SpendExceptionApprovalRecord` lifecycle (create → pending, consume → consumed, expire → expired, revoke → revoked) in `tests/unit/inference_economics.rs`
- [ ] T068 [US3] Write integration test for full spend exception flow — configure budget, trigger over-budget call, approve as session owner, verify call proceeds and approval consumed in `tests/integration/inference_economics_cli.rs`
- [ ] T069 [US3] Write integration test for red-zone rejection — configure red-zone task, session owner attempts self-approval, verify rejection and governance approver requirement in `tests/integration/inference_economics_cli.rs`
- [ ] T069a [US3] Write integration test for unavailable approver in non-interactive execution — configure red-zone task with spend exception required, run non-interactively with no governance approver present, verify call is blocked with explicit reason rather than admitted silently (FR-027, edge case "What happens when the required approver...is unavailable during an active run?") in `tests/integration/inference_economics_cli.rs`.
- [ ] T070 [US3] Write integration test for pricing snapshot activation and staleness — activate snapshot, advance clock, verify staleness flag on reservation but not on exact provider cost in `tests/integration/inference_economics_cli.rs`
- [ ] T070a [US3] Implement route-change activation gate — before a route-policy change becomes active, verify that the required evaluation approval criteria are satisfied; keep the change inactive and surface the gating reason when approval is not met (FR-049, SC-008). Define `RouteChangeGate` type and wire into the route-policy management path in `crates/boundline-core/src/domain/inference_economics.rs`.
- [ ] T070a1 [US3] Write unit test for route-change activation gate — verify that a route-policy change without evaluation approval remains inactive and surfaces the gating reason; verify that a change with approval becomes active (FR-049, SC-008) in `tests/unit/inference_economics.rs`.
- [ ] T070b [US3] Add integration test verifying that existing model-selection defaults are preserved when inference economics is disabled or no budget is configured — run provider-backed calls without a configured budget and confirm default routing behavior is unchanged (FR-050).
- [ ] T070c [US3] Add `budget_state = Disabled` projection path — when no session budget is configured, `SessionBudgetProjection` returns `Disabled` state and all admission checks short-circuit to `InBounds` (FR-050).
- [ ] T070c1 [US3] Write unit test for disabled budget state — verify that when no budget is configured, admission always returns `InBounds`, status projection shows `Disabled`, and existing model-selection defaults are used (FR-050, SC-006) in `tests/unit/inference_economics.rs`.

**Checkpoint**: All three user stories functional — budget enforcement, route selection, spend exception approval, and pricing snapshot lifecycle.

---

## Phase 6: Integration & Observability

**Purpose**: Trace event extensions, telemetry surface wiring, contract validation

- [ ] T071 [P] Extend `ProviderCallCompleted` event payload with cost fields (cost_quality, normalized_cost, native_cost, native_currency, pricing_snapshot_id, snapshot_staleness, approval_type, approval_id) in `crates/boundline-core/src/domain/observability.rs`
- [ ] T072 [P] Add `BudgetStateChanged` event type (v1.0) to `EventType` enum and define its payload schema in `crates/boundline-core/src/domain/observability.rs`
- [ ] T073 [P] Add `SpendExceptionApprovalRecorded` event type (v1.0) to `EventType` enum and define its payload schema in `crates/boundline-core/src/domain/observability.rs`
- [ ] T074 Wire new event emissions into `BudgetEnforcer` and `ApprovalManager` in `crates/boundline-adapters/src/budget_enforcer.rs`
- [ ] T075 Add trace persistence for extended event payloads in `crates/boundline-adapters/src/trace_store.rs`
- [ ] T076 Write contract test for extended `ProviderCallCompleted` payload schema validation in `tests/contract/inference_economics_trace.rs`
- [ ] T077 Write contract test for `BudgetStateChanged` and `SpendExceptionApprovalRecorded` payload schema validation in `tests/contract/inference_economics_trace.rs`

**Checkpoint**: All trace events emit correct payloads and pass contract validation.

---

## Phase 7: Roadmap Conversion & Docs Synchronization

**Purpose**: Convert the roadmap seed into a spec artifact, remove duplication, and synchronize cross-repo documentation.

- [ ] T078 Rename `specs/081-ai-gateway-economics/spec-ai-gateway-and-inference-economics.md` to `specs/081-ai-gateway-economics/feat-ai-gateway-and-inference-economics.md` per the Boundline `feat-<slug>.md` convention. The original roadmap seed `roadmap/features/20-ai-gateway-and-inference-economics.md` is already removed; verify no stale references remain.
- [ ] T079 Verify that the original roadmap seed `roadmap/features/20-ai-gateway-and-inference-economics.md` is removed (move-on-conversion semantics); if it still exists, delete it.
- [ ] T080 Update `roadmap/Next - forward-roadmap.md` to point references for the AI Gateway feature to `specs/081-ai-gateway-economics/spec.md`
- [ ] T081 Update `specs/081-ai-gateway-economics/spec.md` Input field to reference `feat-ai-gateway-and-inference-economics.md` (renamed from `spec-ai-gateway-and-inference-economics.md`) and remove the `Initial reference` note.
- [ ] T081a [P] Review `docs/` and `tech-docs/` markdown files for stale references to the old roadmap seed or missing inference economics documentation; update `docs/configuration.md` with the new `[inference_economics]` config section schema and new CLI subcommands (`set-budget`, `unset-budget`, `approve`, `set-pricing`, `inspect cost`).
- [ ] T082 [P] Update `README.md` with inference economics feature entry if release-facing
- [ ] T083 [P] Update `CHANGELOG.md` with inference economics entry under Unreleased changes, referencing this feature branch
- [ ] T084 [P] Update `AGENTS.md` active technologies section with inference economics context

**Checkpoint**: Roadmap seed converted, no duplicate source-of-truth, cross-repo docs synchronized.

---

## Phase 8: Final Phase — Release, Quality, And Verification

**Purpose**: Version bump, format, lint, test, coverage, and release readiness. This phase gates merge.

- [ ] T085 Update workspace version in `Cargo.toml` from `0.80.0` to `0.81.0` per Boundline versioning policy (feature increment: 0.x.y → 0.x+1.0)
- [ ] T086 Run `./scripts/update-docs-versions.sh` to synchronize version references across `docs/`, `tech-docs/`, and `README.md`
- [ ] T087 Run `cargo fmt` on all modified and new Rust files
- [ ] T088 Run `scripts/clippy.sh` (`cargo clippy --workspace --all-targets --all-features -- -D warnings`) and fix all warnings
- [ ] T089 Run `scripts/test.sh` (`cargo nextest run --workspace --all-features`) and fix all failing tests
- [ ] T090 Run `scripts/coverage.sh` and confirm at least 95% line coverage for every modified or created Rust file. If any file falls below 95%, add targeted tests or justify exclusion explicitly.
- [ ] T091 Run `scripts/check-no-local-paths.sh` and verify no local filesystem paths are committed
- [ ] T092 Run `scripts/check-rust-no-panic.sh` and verify no new `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, or assert-family macros outside `main.rs`
- [ ] T093 Validate quickstart.md scenarios end-to-end — run each quickstart command sequence in a temp fixture workspace and verify output matches expected behavior
- [ ] T094 Run `scripts/sync-distribution-metadata.sh` to update Homebrew formula and Winget manifests from the bumped Cargo.toml version
- [ ] T095 Run `scripts/validate-assistant-plugins.sh` if any assistant plugin metadata was touched by this feature
- [ ] T096 Verify that `cargo llvm-cov --workspace --all-features` produces usable lcov.info with no coverage regressions on existing code
- [ ] T097 Final review: confirm all 50 Functional Requirements, 8 Success Criteria, and 12 edge cases are addressed by at least one passing test or explicit deferral note

**Completion Gate**: All quality scripts pass, coverage ≥ 95%, version bumped, docs synchronized, roadmap seed converted.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion (T001 for Decimal dependency) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational completion — No dependencies on other stories
- **User Story 2 (Phase 4)**: Depends on Foundational + US1 models (T017–T020, T024) — RouteSelector needs budget types
- **User Story 3 (Phase 5)**: Depends on Foundational + US1 budget model (T017, T024) — ApprovalManager needs SessionBudgetProjection and ActiveSessionRecord
- **Integration & Observability (Phase 6)**: Depends on US1 + US2 + US3 completion
- **Roadmap Conversion (Phase 7)**: Depends on US3 completion — can run in parallel with Phase 6
- **Release, Quality, And Verification (Phase 8)**: Depends on all prior phases — final gate

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational — MVP, independently shippable
- **User Story 2 (P2)**: Can start after US1 types are defined — depends on `SessionBudgetProjection` and `InvocationCostRecord` types; RouteSelector needs budget state for admission decisions
- **User Story 3 (P3)**: Can start after US1 types are defined — depends on `SessionBudgetProjection` for approval context; may integrate with US2 route selection

### Within Each User Story

- Types before services
- Services before adapter wiring
- Adapter wiring before CLI commands
- Core implementation before tests (write tests alongside, not after)
- Story complete with tests passing before moving to next priority

### Parallel Opportunities

- T002, T003 can run in parallel (different files)
- T004–T009 can all run in parallel (all in same new file, but different type blocks — sequential within file preferred)
- T015, T016 can run in parallel (different test functions in same file)
- T017, T018, T019 can run in parallel (three structs in same file — sequential preferred)
- T038, T039 can run in parallel
- T048 + T048a can run in parallel (different concerns within US2)
- T049, T050, T051 can run in parallel
- T070a, T070b, T070c can run in parallel (different FRs within US3); T070a1 pairs with T070a, T070c1 pairs with T070c
- T071, T072, T073 can run in parallel (different event types)
- T078–T084 (Phase 7) can run in parallel with T071–T077 (Phase 6)
- T085–T097 (Phase 8) are sequential — version bump must precede docs sync; all quality scripts can run in parallel after version bump

---

## Parallel Example: User Story 1

```bash
# After foundational types are defined (T004–T011):
# Launch type definitions in sequence (same file):
Task: "T017 Define SessionBudgetProjection struct"
Task: "T018 Define PricingEntry and PricingSnapshot structs"
Task: "T019 Define InvocationCostRecord struct"

# Then launch service implementation + config side-by-side:
Task: "T021 Implement SessionBudgetProjection::reserve()" 
Task: "T025 Implement BudgetEnforcer struct"  # depends on T021
Task: "T026 Implement PricingResolver struct"  # independent of T025

# Tests can be written alongside implementation:
Task: "T033 Write unit tests for SessionBudgetProjection"
Task: "T034 Write unit tests for BudgetEnforcer"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T016)
3. Complete Phase 3: User Story 1 (T017–T037)
4. **STOP and VALIDATE**: Configure a budget, run provider-backed calls, verify budget enforcement, status projection, and blocked outcomes
5. Deploy/demo MVP — budget enforcement delivers immediate spend control value

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP: budget enforcement!)
3. Add User Story 2 → Test independently → Deploy/Demo (route selection by risk)
4. Add User Story 3 → Test independently → Deploy/Demo (governed approvals + snapshots)
5. Phase 6 (Integration & Observability) → Trace events wired and validated
6. Phase 7 (Roadmap Conversion) → Seed converted, docs synchronized
7. Phase 8 (Release, Quality, And Verification) → All quality gates pass → Merge ready

### Parallel Team Strategy

With two developers:
1. Both complete Setup + Foundational together (T001–T016)
2. Once Foundational is done:
   - Developer A: User Story 1 (T017–T037) — budget enforcement
   - Developer B: User Story 2 (T038–T048a) + User Story 3 (T049–T070c) — routing + approvals
3. Both converge on Integration & Observability (T071–T077)
4. One developer handles Roadmap Conversion (T078–T084) while the other starts quality scripts (T085–T097)

---

## Notes

- [P] tasks = different files or independent validation commands, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- All monetary arithmetic MUST use `Decimal` — never `f64` or `f32`
- All new types outside `main.rs` and `#[cfg(test)]` MUST avoid `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, and assert-family macros per constitution Language Rules
- All stable serialization shapes MUST use typed structs/enums with serde derives per constitution Language Rules
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- NEVER run `boundline` CLI against the repository root — use a temp fixture workspace
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` after every code change
- Phase 8 (Release, Quality, And Verification) is the merge gate — all T085–T097 must pass before merging
- The version bump task (T085) increments the workspace `Cargo.toml` version from `0.80.0` to `0.81.0` per Boundline versioning policy for feature releases
- Roadmap seed `roadmap/features/20-ai-gateway-and-inference-economics.md` is converted to spec artifact; the spec folder at `specs/081-ai-gateway-economics/` is the sole source of truth after T079
