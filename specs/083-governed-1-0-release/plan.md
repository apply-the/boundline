# Implementation Plan: Governed 1.0 Release

**Branch**: `083-governed-1-0-release` | **Date**: 2026-07-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/083-governed-1-0-release/spec.md`

## Summary

Deliver the 1.0 release train across the existing `boundline`, `canon`, and
`boundline-adapter-speckit` repositories. Boundline becomes the only
authoritative delivery control plane; Canon becomes a deterministic governance
kernel; and the Speckit integration consumes a stable, one-shot,
proposal-only adapter contract.

The implementation is split into independently reviewable milestones:
baseline and qualification, public contracts and CLI migration, honest Canon
authoring and decision memory, persistent fenced execution, crash-consistent
Git publication, adapter qualification, the cross-repository vertical slice,
and a frozen RC soak. The approved execution roadmap is normative where this
plan summarizes behavior.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024, pinned by the existing
`rust-toolchain.toml` and re-recorded by M0

**Primary Dependencies**: Existing workspace dependencies (`clap`, `serde`,
`serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled
SQLite); no new persistence service; contract crates are published from the
existing Boundline and Canon repositories

**Storage**: Existing workspace-local `.boundline/` projections plus a
user-only external state root containing clone-local repository records,
persistent session worktrees, execution journals, publication locks, recovery
state, backups, and the Canon outcome outbox

**Testing**: Rust unit, contract, integration, migration, property, and
fault-injection tests; `cargo fmt --check`; Clippy with warnings denied;
workspace tests; nextest; cargo-deny; llvm-cov; patch coverage; Linux, macOS,
and Windows qualification

**Target Platform**: Linux, macOS, and Windows on supported local filesystems
that honor the declared durability primitives

**Project Type**: Three separately versioned Rust CLI/library repositories with
two published contract crates and five generated host-integration packs

**Performance Goals**: Request replay lookup and status projection remain
interactive; journal writes and fingerprinting are bounded by admitted state;
model-assisted utility reaches at least 95% per case and declared
provider/model configuration over at least 20 runs

**Constraints**: Git is mandatory for stable mutation; one executor per
session; one publisher per local repository; no automatic merge or rebase; no
stable background adapter daemon; no authoritative dirty result after
publication; no synthetic executor, hidden fallback, placeholder, or
self-attested completion

**Scale/Scope**: Release train `0.90.0 -> 0.95.0 -> 1.0.0-rc.1 -> 1.0.0`;
nine Canon profiles; one adapter per invocation; five tested host packs;
separate local repository identity for each clone

## M1C Command-Surface Promotion Policy

M1C resolves the 0.90 command tree without creating parser-only stable
commands. `StableTargetPending` is planning and inventory state only: it is
not registered in the public parser, printed by stable help, emitted in
completion metadata, or represented as a runtime command classification.
`StableOperational` means that parser, real operational handler, fail-closed
behavior, contract tests, help, completion metadata, and documentation are
complete.

A command moves from `StableTargetPending` to `StableOperational` only when
its owning task explicitly promotes it in the same commit as the real handler,
authority and failure semantics, contract tests, command tree, help,
completions, and documentation. A placeholder or generic not-yet-implemented
response is never a stable handler.

The 0.90 StableOperational/help surface is an additive subset of the
documented final 1.0 stable target inventory. The wider 0.90 command tree also
contains explicit Preview and hidden Internal commands, so it is not itself a
subset of that stable inventory. It removes overlapping legacy lifecycle
entrypoints immediately and retains no compatibility aliases. Later 0.90/0.95
work may promote reserved commands only with their real implementation; after
the 0.95 contract freeze, no StableOperational command may be removed. Preview
commands remain outside the 1.0 compatibility promise.

## Constitution Check

*GATE: Passed before Phase 0 and re-checked after Phase 1 design.*

| Principle | Status | Evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | Boundline remains the sole delivery orchestrator and publication authority |
| No abstract agent systems | PASS | Providers and adapters execute bounded delivery steps and return proposals |
| Bounded and sequential-first execution | PASS | One executor per session, one adapter invocation, explicit deadlines, no background process |
| Stateful execution | PASS | Versioned journals, fingerprints, evidence bindings, leases, and terminal projections |
| Mutable planning | PASS | Re-admission and revalidation are explicit; no hidden rebase or self-modifying behavior |
| Required observability and no hidden intelligence | PASS | Every authority, capability, mutation, challenge, recovery, and synchronization decision has typed status and reason codes |
| Failure as a first-class path | PASS | Executor reconciliation, publication recovery, quarantine, stale-base rejection, and Canon outbox failure are normative flows |
| Separation from external systems | PASS | Boundline publication remains successful during Canon unavailability; Canon synchronization is durable follow-up work |
| Minimal capability slices | PASS | Milestones deliver independently reviewable contracts, governance, transaction, adapter, and release slices |
| Real acceptance criteria | PASS | The vertical slice exercises real Git mutation, crash boundaries, challenge tiers, recovery, and terminal evidence |
| Language rules | PASS | New stable shapes use typed serde models; no panic-prone control flow or magic domain literals |
| Catalog currency | PASS WITH REQUIRED DELTA | Official provider review found newer stable model lines; M0 must update and fixture the catalog before later implementation relies on defaults |

The roadmap spans multiple repositories because no single-repository slice can
prove the stable end-to-end authority boundary. Each milestone still has a
single accountable owner, explicit prerequisites, and an independently
testable exit review.

## Project Structure

### Documentation

```text
specs/083-governed-1-0-release/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── decisions/
│   └── m1c-command-surface-ownership.md
├── checklists/
│   └── requirements.md
└── contracts/
    ├── boundline-protocol-v1.md
    ├── canon-outcome-sync.md
    └── framework-adapter-v1.md
```

### Boundline repository

```text
crates/
├── boundline-protocol/                 # Public stable protocol crate
├── boundline-core/src/
│   ├── identity/                       # Clone/worktree identity and state-root rules
│   ├── transaction/                    # Revisions, idempotency, evidence bindings
│   ├── execution/                      # Lease, fencing, capability grant, reconciliation
│   └── publication/                    # Backup, publication, recovery, Canon outbox
├── boundline-adapters/src/
│   └── framework_v1/                   # Host side of the stable adapter contract
└── boundline-cli/src/
    ├── commands/                       # Stable primary and administrative surfaces
    └── projection/                     # CLI, RPC, and MCP semantic projections
src/
├── orchestrator/                       # Admission, run, proof, publication coordination
└── cli/                                # Root command parsing and presentation only
tests/
├── contract/
├── integration/
├── migration/
└── fault_injection/
```

Existing modules are split by responsibility as they are touched; the
implementation must not add transaction or publication logic to the existing
large CLI and session-runtime files.

### Canon repository

```text
canon-contracts/                        # Public deterministic governance contract crate
src/
├── profiles/                           # Nine stable profiles and registry
├── validation/                         # Deterministic structural and policy rules
├── decision_memory/                    # Typed graph and stale propagation
├── evidence/                           # External review and outcome projections
├── rpc/                                # One-shot JSON machine interface
└── cli/                                # Standalone stable commands
tests/
├── golden/
├── contract/
└── migration/
```

### Speckit adapter repository

```text
src/
├── protocol/                           # Dependency on boundline-protocol, no DTO copy
├── preflight/                          # Non-mutating capability and input checks
├── stages/                             # Requirements, planning, implementation proposal
└── transport/                          # Strict one-shot JSON framing
tests/
├── contract/
├── security/
└── qualification/
```

## Milestone Plan

| Milestone | Owner / exit reviewer | Duration | Prerequisites | Deliverable and blocking risks |
|---|---|---:|---|---|
| M0 Baseline and qualification | Release Owner / Independent Release Reviewer | 1 week | Clean branches in all repositories | Record commits, toolchain, lockfiles, command results, coverage, model catalog delta, platform/filesystem and host matrices. Risk: unsupported durability or sandbox enforcement. |
| M1 0.90 contracts, CLI, migration | Boundline Runtime Owner / Canon Kernel Owner | 2 weeks | M0 | Publish prerelease contract crates, implement canonical idempotency and revisions, break duplicate CLI surfaces, restore Canon CLI/RPC, add transactional bridges. Adapter fixtures update in parallel. |
| M2a Honest Canon exchange | Canon Kernel Owner / Security Reviewer | 2 weeks | M1 contracts | Typed authoring and evidence lineage, deterministic validation, governance bundle, removal of synthetic and stub surfaces. M3 foundations may start in parallel. |
| M2b Decision memory | Canon Kernel Owner / Independent Release Reviewer | 3 weeks | M2a | Typed graph, stale propagation, nine-profile corpus, durable terminal-outcome recording. Runs in parallel with M3/M4. |
| M3 Persistent execution and publication | Boundline Runtime Owner / Security Reviewer | 4 weeks | M1 | Identity, external worktrees, executor capability grants and fencing, crash reconciliation, challenge tiers, fingerprints, durable publication, recovery, clean commit outcome, Canon outbox. |
| M4 Adapter qualification | Adapter Owner / Boundline Runtime Owner | 2 weeks | M1 contract | Stable one-shot protocol, shared sandbox contract, qualified Speckit stages, preview separation. |
| M5 Vertical slice and RC | Release Owner / Independent Release Reviewer | 2 weeks | M2b, M3, M4 | Cross-repository workflow, fault matrix, host projection equivalence, migration, Canon retry, and compatibility qualification. |
| M6 Frozen 1.0 release | Release Owner / Independent Release Reviewer | 2-week soak | RC | Full release suite, only frozen-contract corrections, registry packages, signed tags, provenance, reports, and compatibility matrices. |

M2b, M3, and M4 are the only planned parallel streams. No source task may
cross their ownership boundaries without a contract change reviewed in M1.

## Release and Migration Gates

- Guarantee `0.95.x -> 1.0`.
- Provide Boundline `0.82.0 -> 0.90/0.95` and Canon
  `0.72.6 -> 0.90/0.95` bridge paths.
- Archive unsupported active pre-0.90 sessions read-only and restart them as
  newly admitted work unless an explicit fixture proves equivalence.
- Freeze line coverage at the greater of 80% or M0 and patch coverage at 90%
  before 0.95.
- Require 100% deterministic corpus and 100% fail-closed safety corpus.
- Require fresh proof for every terminal success claim.
- Require fault injection between every durable intent, effect, validation,
  and completion step.
- Freeze stable CLI, schemas, reason codes, protocols, profiles, and
  persistence semantics at `1.0.0-rc.1`.

## Complexity Tracking

| Apparent tension | Why required | Containment |
|---|---|---|
| Three repositories in one roadmap | The stable authority and adapter boundaries cannot be qualified in isolation | Separate milestone owners and published contracts; no new repository |
| Canon integration vs. external-system independence | Decision memory must record the actual outcome | Durable idempotent outbox; publication never rolls back or blocks on temporary Canon failure |
| Concurrent sessions vs. sequential-first constitution | Sessions may prepare independently, but mutation of one session and publication of one repository remain serialized | One executor per session and one publisher per local repository; no hidden fan-out |

## M2b-C0-A Contract Amendment

M2b-C is gated by a public outcome-recording contract rather than a private
wire format. T097 freezes `record_outcome` in `canon-contracts 0.91.0`; T098
makes it capability-discoverable but unavailable until T059 installs
transactional ingestion. T099 publishes and verifies that exact package before
T057-T059 begin.

The dependency chain is `T097 -> T098 -> T099 -> T057/T058/T059`. The workspace
version follows 0.91 because Canon packages inherit one workspace version. The
wire envelope stays V1 (`"1.0"`), the six 0.90 operations keep exact semantics,
and `publish` remains read-only.
