# Tasks: Governed 1.0 Release

**Input**: Design documents in `specs/083-governed-1-0-release/`

**Repositories**: `boundline`, `canon`, `boundline-adapter-speckit`

**Execution rule**: Complete M0 and M1 before story work. After M1, Canon
decision memory, Boundline transaction work, and adapter qualification may
proceed in parallel. All Rust behavior tasks use test-first implementation.

## Phase 1: Setup and M0 Baseline

- [x] T001 Record exact commit, branch, toolchain, target, lockfile digest, and filesystem data for all three repositories in `specs/083-governed-1-0-release/baseline.md`
- [x] T002 Run every required repository command without assuming success and record command, exit status, duration, and artifact references in `specs/083-governed-1-0-release/baseline.md`
- [x] T003 [P] Measure Boundline workspace and patch coverage and record the greater-of-80%-or-baseline line threshold plus the 90% patch threshold in `specs/083-governed-1-0-release/qualification-matrix.md`
- [x] T004 [P] Define Linux, macOS, Windows, local-filesystem, durability, Git-feature, and fail-closed qualification rows in `specs/083-governed-1-0-release/qualification-matrix.md`
- [x] T005 [P] Record the exact local Claude, Codex, Copilot, Cursor, and Antigravity inventory plus an owned fail-closed M5 disposition for every unqualified host in `specs/083-governed-1-0-release/host-compatibility.md`; T076 owns final semantic-projection fixtures
- [x] T006 Update current provider models and retirement fixtures from the official evidence in `boundline:assistant/catalog/model-catalog.toml` and its catalog tests without changing user-pinned routes
- [x] T007 Add a named, expiring coverage and platform waiver schema plus approval policy to `specs/083-governed-1-0-release/qualification-matrix.md`
- [x] T008 Review M0 evidence and block M1 only for an unreproducible build, unexplained repository state, impossible durability boundary, unenforceable security boundary, or equivalent foundational-assumption failure in `specs/083-governed-1-0-release/baseline.md`

---

## Phase 2: Foundational M1 Contracts, CLI, and Migration

**Blocking prerequisite for every user story**

- [x] T009 Create typed public envelope, revision, lineage, evidence, projection, and reason-code contract tests in `boundline:crates/boundline-protocol/tests/protocol_v1.rs`
- [x] T010 Implement the public protocol crate without internal persistence records in `boundline:crates/boundline-protocol/src/lib.rs`
- [ ] T011 Add idempotency tests proving lookup-before-revision, same-digest replay, different-digest conflict, and new-request revision validation in `boundline:crates/boundline-core/tests/idempotency.rs`
- [ ] T012 Implement contract-line-and-operation-scoped canonical idempotency records in `boundline:crates/boundline-core/src/transaction/idempotency.rs`
- [ ] T013 Add CLI snapshot tests for the frozen Boundline primary/admin command tree and removed duplicate commands in `boundline:tests/contract/cli_090.rs`
- [ ] T014 Replace the Boundline command tree with the 0.90 stable surface and explicit preview classifications in `boundline:src/cli.rs` and `boundline:crates/boundline-cli/src/cli.rs`
- [x] T015 [P] Create Canon contract-crate tests for packets, profiles, deterministic evidence, decision-memory projections, and one-shot operations in `canon:crates/canon-contracts/tests/contracts.rs`
- [x] T016 [P] Implement the `canon-contracts` crate and nine-profile registry types in `canon:crates/canon-contracts/src/lib.rs`
- [ ] T017 Add fixture-driven transactional migration tests for Boundline 0.82.0 and Canon 0.72.6, including forced termination and unsupported active-session archival, in `boundline:tests/migration/bridge_090.rs` and `canon:tests/migration/bridge_090.rs`
- [ ] T018 Implement backup-first, idempotent, inspectable, atomic 0.90 bridge migrators and conversion reports in `boundline:crates/boundline-core/src/migration/mod.rs` and `canon:crates/canon-engine/src/policy/migration.rs`
- [ ] T019 Set prerelease workspace versions to the roadmap’s explicit `0.90.0` train point in `boundline:Cargo.toml` and `canon:Cargo.toml`; after the `boundline-protocol` prerelease package exists, replace the adapter’s owned Boundline 0.66 bridge and update its supported prerelease range in `boundline-adapter-speckit:Cargo.toml`
- [ ] T020 Publish and verify immutable prerelease package fixtures plus matching signed-tag provenance metadata in `boundline:tech-docs/release-checklist.md` and `canon:CHANGELOG.md`

**Checkpoint**: Public contract lines, CLI break, and bridge migration are
reviewed and frozen for M2–M4 implementation.

---

## Phase 3: User Story 1 — Publish a Governed Change Safely

**Goal**: Execute one admitted change in an isolated persistent worktree,
invalidate stale proof, and publish exactly one verified commit.

**Independent test**: A fixture session advances the admitted target branch
once and leaves the authoritative worktree clean and equal to the candidate
commit.

- [ ] T021 [P] [US1] Add clone-vs-linked-worktree repository identity tests in `boundline:crates/boundline-core/tests/repository_identity.rs`
- [ ] T022 [US1] Implement clone-local repository and authoritative-worktree identity with role markers in `boundline:crates/boundline-core/src/identity/repository.rs`
- [ ] T023 [P] [US1] Add fingerprint fixtures for tracked/untracked files, index, rename, delete, executable bit, symlink, binary, Unicode collision, exclusions, and schema upgrades in `boundline:crates/boundline-core/tests/fingerprint.rs`
- [ ] T024 [US1] Implement versioned product fingerprints and explicit ignored/cache policy in `boundline:crates/boundline-core/src/transaction/fingerprint.rs`
- [ ] T025 [P] [US1] Add evidence-binding tests for exact revision/diff/fingerprint/claims/lineage and formatter/codegen staleness in `boundline:crates/boundline-core/tests/evidence_binding.rs`
- [ ] T026 [US1] Implement approval, proof, risk, and verification freshness propagation in `boundline:crates/boundline-core/src/transaction/evidence.rs`
- [ ] T027 [P] [US1] Add session execution lease tests for concurrent admission and superseded fencing-token writes in `boundline:crates/boundline-core/tests/execution_lease.rs`
- [ ] T028 [US1] Implement exclusive per-session execution leases and monotonic fencing checks in `boundline:crates/boundline-core/src/execution/lease.rs`
- [ ] T029 [P] [US1] Add capability-grant rejection tests for path, command, environment, secret, network, process, resource, output, Git-ref, and state-root escapes in `boundline:crates/boundline-core/tests/capability_grant.rs`
- [ ] T030 [US1] Implement the shared provider/tool/adapter capability grant and fail-closed admission service in `boundline:crates/boundline-core/src/execution/capability.rs`
- [ ] T031 [P] [US1] Add persistent external worktree retention and cleanup-state tests in `boundline:tests/integration/session_worktree.rs`
- [ ] T032 [US1] Implement external managed worktree creation, lease metadata, role validation, resume, abort, and cleanup in `boundline:crates/boundline-core/src/execution/worktree.rs`
- [ ] T033 [P] [US1] Add Tier 0–3 challenge tests, same-lineage override tests, and shared-conversation rejection tests in `boundline:crates/boundline-core/tests/challenge_policy.rs`
- [ ] T034 [US1] Implement the frozen Tier 0–3 challenge matrix and named override records in `boundline:crates/boundline-core/src/transaction/challenge.rs`
- [ ] T035 [P] [US1] Add candidate-commit, per-path precondition, CAS, stale-base, and clean-final-worktree tests in `boundline:tests/integration/publication.rs`
- [ ] T036 [US1] Implement candidate preparation, backup validation, per-path replacement, index update, target-ref CAS, and final fingerprint verification in `boundline:crates/boundline-core/src/publication/transaction.rs`
- [ ] T037 [US1] Add `run`, `approve`, `status`, `inspect`, `session abort`, and `session cleanup` projections for execution, proof freshness, challenge, and publication in `boundline:crates/boundline-cli/src/projection/session.rs`

---

## Phase 4: User Story 2 — Resume or Recover Without Guessing

**Goal**: Reconcile executor and publication crashes without overwriting
unexplained state.

**Independent test**: Forced termination at every declared boundary completes,
restores, or preserves state without destructive action.

- [ ] T038 [P] [US2] Add Unix process-group and Windows Job Object lifecycle tests, including a child that survives the Boundline parent, in `boundline:crates/boundline-core/tests/process_containment.rs`
- [ ] T039 [US2] Implement cooperative cancellation, escalation, descendant termination confirmation, and worktree blocking in `boundline:crates/boundline-core/src/execution/process.rs`
- [ ] T040 [P] [US2] Add executor intent/no-delta/uncommitted-candidate reconciliation tests around every journal boundary in `boundline:tests/fault_injection/executor.rs`
- [ ] T041 [US2] Implement durable `ExecutorInFlightRecord` transitions and candidate revalidation admission in `boundline:crates/boundline-core/src/execution/reconciliation.rs`
- [ ] T042 [P] [US2] Add state-root ownership, permission, canonicalization, symlink, traversal, redaction, and role-confusion tests in `boundline:crates/boundline-core/tests/state_root_security.rs`
- [ ] T043 [US2] Implement protected state-root initialization and redacted trace boundaries in `boundline:crates/boundline-core/src/identity/state_root.rs`
- [ ] T044 [P] [US2] Add forced-termination tests before publication start, during each file/index/ref effect, and after durable completion in `boundline:tests/fault_injection/publication.rs`
- [ ] T045 [US2] Implement crash-consistent SQLite transactions, full synchronization, platform flush adapters, publication fencing, and durable recovery records in `boundline:crates/boundline-core/src/publication/journal.rs`
- [ ] T046 [P] [US2] Add complete, restore, unknown-change, mixed-change, completed-lock, and forced-abandon recovery tests in `boundline:tests/integration/recovery.rs`
- [ ] T047 [US2] Implement recovery inspection, completion, restoration, quarantine, and non-destructive abandonment in `boundline:crates/boundline-core/src/publication/recovery.rs`
- [ ] T048 [US2] Implement `recover inspect|complete|restore|abandon` presentation and stable reason codes in `boundline:crates/boundline-cli/src/commands/recover.rs`

---

## Phase 5: User Story 3 — Govern Intent and Record the Actual Outcome

**Goal**: Make Canon deterministic and record exactly the terminal outcome
produced by Boundline.

**Independent test**: Canon authorizes from typed deterministic inputs,
Boundline publishes while Canon is unavailable, and later synchronization
creates exactly one decision-memory event.

- [ ] T049 [P] [US3] Add golden tests for exactly nine stable profiles and preview/removal of implementation mode in `canon:tests/golden/profile_registry.rs`
- [ ] T050 [US3] Implement the nine-profile registry and change-vs-implementation boundary in `canon:crates/canon-engine/src/modes/mod.rs`
- [ ] T051 [P] [US3] Add tests proving external semantic evidence lineage and rejecting self-attested or internally executed semantic review in `canon:tests/contract/external_verification.rs`
- [ ] T052 [US3] Replace synthetic Copilot verification, reviewer stubs, MCP stubs, and unimplemented stable verify behavior with deterministic validation or explicit unsupported status in `canon:crates/canon-adapters/src/reviewer.rs`, `canon:crates/canon-adapters/src/mcp_stdio.rs`, and `canon:crates/canon-cli/src/commands/verify.rs`
- [ ] T053 [P] [US3] Add structural, cross-packet, authority, required-evidence, and stale-propagation corpus tests in `canon:tests/golden/deterministic_governance.rs`
- [ ] T054 [US3] Implement typed governance bundles, deterministic validators, decision-memory graph, and stale propagation in `canon:crates/canon-engine/src/decision_memory/mod.rs`
- [ ] T055 [P] [US3] Add CLI and one-shot RPC contract tests for all stable Canon commands and operations in `canon:tests/contract/stable_surface.rs`
- [ ] T056 [US3] Implement standalone Canon CLI commands and exactly-one-request JSON RPC dispatch in `canon:crates/canon-cli/src/commands.rs` and `canon:crates/canon-cli/src/rpc.rs`
- [ ] T057 [P] [US3] Add Canon outcome outbox tests for retry, duplicate delivery, digest conflict, permanent rejection, evidence retention, and authorized archival in `boundline:tests/integration/canon_outcome_sync.rs`
- [ ] T058 [US3] Implement the durable outcome outbox and status projection in `boundline:crates/boundline-core/src/publication/canon_outbox.rs`
- [ ] T059 [US3] Implement transactional outcome ingestion and decision-memory revision responses in `canon:crates/canon-engine/src/decision_memory/outcome.rs`

---

## Phase 6: User Story 4 — Delegate Through a Bounded Adapter

**Goal**: Qualify a one-shot proposal-only adapter without authority or
capability escape.

**Independent test**: Stable operations succeed under admitted capabilities
and every security/authority escape fails closed.

- [ ] T060 [P] [US4] Add `FrameworkAdapterV1` framing, operation, failure, capability, and authority contract tests in `boundline:crates/boundline-protocol/tests/framework_adapter_v1.rs`
- [ ] T061 [US4] Implement stable `describe`, `preflight`, `execute_stage`, and `emit_hook` protocol types in `boundline:crates/boundline-protocol/src/framework_adapter.rs`
- [ ] T062 [P] [US4] Add host subprocess tests for strict stdout framing, separate stderr, deadlines, output limits, process-tree termination, and no background work in `boundline:crates/boundline-adapters/tests/framework_v1.rs`
- [ ] T063 [US4] Replace the legacy adapter host path with the shared capability-bound one-shot runtime in `boundline:crates/boundline-adapters/src/framework_v1/runtime.rs`
- [ ] T064 [P] [US4] Replace duplicated DTO fixtures with `boundline-protocol` compatibility tests in `boundline-adapter-speckit:tests/contract.rs`
- [ ] T065 [US4] Depend on the immutable protocol package and implement strict request/response transport in `boundline-adapter-speckit:Cargo.toml` and `boundline-adapter-speckit:src/transport.rs`
- [ ] T066 [P] [US4] Add qualification tests for stable requirements, planning, and implementation-proposal stages plus visibly preview clarification, checklist, and task decomposition in `boundline-adapter-speckit:tests/qualification.rs`
- [ ] T067 [US4] Implement the qualified stable-stage handlers and preview capability declarations in `boundline-adapter-speckit:src/stages.rs` and `boundline-adapter-speckit:src/profile.rs`
- [ ] T068 [US4] Add generated adapter status, inspect, trace, and five-host projection fixtures in `boundline:tests/contract/adapter_projections.rs`

---

## Phase 7: User Story 5 — Upgrade and Release with Verifiable Compatibility

**Goal**: Qualify migrations, platforms, hosts, release gates, and the frozen
vertical slice.

**Independent test**: Supported migrations and the end-to-end workflow pass
the frozen deterministic, safety, utility, platform, host, and recovery
matrices.

- [ ] T069 [P] [US5] Add direct 0.95.x-to-1.0 migration fixtures and read-only historical archive fixtures in `boundline:tests/migration/release_100.rs` and `canon:tests/migration/release_100.rs`
- [ ] T070 [US5] Implement and document the single-source 0.95.x-to-1.0 transactional migrators in `boundline:crates/boundline-core/src/migration/release_100.rs` and `canon:crates/canon-engine/src/policy/release_100.rs`
- [ ] T071 [P] [US5] Add Git/filesystem matrix fixtures for symlinks, modes, gitlinks, LFS, sparse checkout, detached HEAD, case/Unicode collisions, Windows locks, separate volumes, large files, and binaries in `boundline:tests/integration/git_filesystem_matrix.rs`
- [ ] T072 [US5] Implement explicit support-or-fail-closed qualification checks in `boundline:crates/boundline-core/src/publication/compatibility.rs`
- [ ] T073 [P] [US5] Add the complete cross-repository vertical-slice harness and normalized projection comparator in `boundline:tests/integration/release_vertical_slice.rs`
- [ ] T074 [US5] Exercise Tier 2/3 challenge, `no_change`, stale approval, concurrent executor, surviving child, stale second publisher, all publication crash phases, proof invalidation, and final Canon outcome in `boundline:tests/integration/release_vertical_slice.rs`
- [ ] T075 [P] [US5] Add at least 20 fixed-fixture runs per benchmark case/configuration and a per-case 95% gate in `boundline:evals/1.0/model-assisted-utility.toml`
- [ ] T076 [US5] Generate and validate Claude, Codex, Copilot, Cursor, and Antigravity compatibility packs against the frozen host matrix in `boundline:assistant/` and `specs/083-governed-1-0-release/host-compatibility.md`
- [ ] T077 [US5] Set the three repositories to `0.95.0`, freeze coverage/waiver/benchmark/host matrices, and publish migration reports in each repository’s `Cargo.toml` and `CHANGELOG.md`
- [ ] T078 [US5] Set the three repositories to `1.0.0-rc.1`, freeze stable interfaces and persistence semantics, and begin the minimum two-week soak in each repository’s `Cargo.toml` and release checklist

---

## Final Phase: Release, Quality, And Verification

- [ ] T079 Verify the roadmap snapshot in `specs/083-governed-1-0-release/feat-governed-1-0-release.md` is the sole active normative seed and update `roadmap/Next - forward-roadmap.md`, `roadmap/joint-roadmap-graph.md`, and `docs/roadmap/index.md`
- [ ] T080 Update stable CLI, migration, transaction, recovery, adapter, Canon, and compatibility documentation in `README.md`, `CHANGELOG.md`, `docs/`, and `tech-docs/`
- [ ] T081 Update final release versions to `1.0.0` in `boundline:Cargo.toml`, `canon:Cargo.toml`, and `boundline-adapter-speckit:Cargo.toml`
- [ ] T082 Run `boundline:scripts/update-docs-versions.sh` after the final Boundline version update and verify all generated version references
- [ ] T083 Run `boundline:scripts/update-plugin-manifests.sh` and `boundline:scripts/validate-assistant-plugins.sh` for all five host packs
- [ ] T084 Run `boundline:scripts/sync-distribution-metadata.sh` and verify Homebrew and Winget release metadata
- [ ] T085 Run `cargo fmt` in all three repositories and verify `cargo fmt --check`
- [ ] T086 Run `boundline:scripts/clippy.sh` plus equivalent workspace Clippy commands in Canon and the adapter and fix every warning
- [ ] T087 Run `boundline:scripts/test.sh` plus complete workspace/all-feature tests and nextest in Canon and the adapter and fix every failure
- [ ] T088 Run `boundline:scripts/coverage.sh` and repository-equivalent coverage checks and confirm at least 95% coverage for every modified or created Rust file
- [ ] T089 Run the 90% patch-coverage gate using `boundline:scripts/common/coverage/intersect_patch_coverage.py` and resolve or explicitly approve every expiring waiver
- [ ] T090 Run `boundline:scripts/check-no-local-paths.sh` and equivalent path scans in Canon and the adapter
- [ ] T091 Run `boundline:scripts/check-rust-no-panic.sh` and equivalent no-panic audits in Canon and the adapter
- [ ] T092 Run `cargo deny check licenses advisories bans sources` in all three repositories and resolve every release-blocking finding
- [ ] T093 Run the complete Linux, macOS, Windows, filesystem, migration, fault-injection, deterministic, rejection, model-utility, and host-compatibility matrices and attach immutable results to `specs/083-governed-1-0-release/release-evidence.md`
- [ ] T094 Publish registry packages, signed source tags, provenance, compatibility matrices, migration reports, and release notes only after independent release review is recorded in `specs/083-governed-1-0-release/release-evidence.md`

## Dependencies

```text
Phase 1 (M0)
  -> Phase 2 (M1)
      -> US1 publication ───────────────┐
      -> US2 recovery (after US1 core) ─┤
      -> US3 Canon ─────────────────────┼-> US5 vertical slice -> Final release
      -> US4 adapter ───────────────────┘
```

- US2 depends on US1 identity, fingerprint, lease, worktree, and publication
  transaction foundations.
- US3 and US4 can run in parallel with US1 after M1.
- US5 depends on complete US1–US4 contract and integration gates.
- Final release work begins only after `1.0.0-rc.1` completes its soak without
  a frozen-contract change.

## Parallel Execution Examples

### After M1

```text
Canon owner:    T049-T059
Boundline owner: T021-T048
Adapter owner:  T060-T068
```

### Within Boundline transaction work

```text
Identity/fingerprint fixtures: T021, T023
Evidence/challenge fixtures:   T025, T033
Capability/process fixtures:   T029, T038
```

Each parallel test task must merge before the corresponding implementation
task is considered complete.

## Implementation Strategy

1. Establish M0 truth before changing runtime behavior.
2. Freeze the public 0.90 contract and migration boundary.
3. Deliver the smallest end-to-end P1 slice: one deterministic executor,
   one Tier 1 proof, one verified commit, and complete recovery.
4. Add Canon decision memory and adapter qualification against frozen
   contracts.
5. Expand to Tier 2/3, all platforms, hosts, migrations, and fault points.
6. Freeze at RC and accept only corrections needed to satisfy the frozen
   contract.
