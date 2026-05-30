# Implementation Plan: Real sqlite-vec Activation And DB Merge Strategy

**Branch**: `065-activate-sqlite-vec` | **Date**: 2026-05-30 | **Spec**: [specs/065-activate-sqlite-vec/spec.md](specs/065-activate-sqlite-vec/spec.md)

**Input**: Feature specification from `specs/065-activate-sqlite-vec/spec.md`

## Summary

Close the remaining gap between the shipped semantic-acceleration scaffold and
the intended local vector-backed retrieval runtime. The implementation keeps a
single workspace-local derived retrieval store, replaces the current JSON-only
semantic full-refresh path with a real `sqlite-vec` table plus SQL-side nearest-
neighbor selection, adds a companion manifest and explicit index lifecycle
commands, and preserves the existing authority order, fallback semantics, and
operator-facing explainability on `plan`, `status`, `next`, and `inspect`.

The slice also formalizes derived-index hygiene: incremental refresh during
routine edits, explicit rebuild boundaries when compatibility changes, managed
Git ignore rules, optional stale-marking hooks, and a bounded `boundline index`
CLI surface that remains the source of truth for status, refresh, rebuild,
clean, and doctor behavior.

The clarified branch-switch policy stays within that same lifecycle scope:
branch checkout, merge, pull-with-merge, rebase, and post-rewrite are treated
as freshness events rather than database merge events; fetch remains a no-op
because it does not change the working tree; commit hooks stay off by default;
full rebuild never runs automatically inside Git hooks; and `boundline index
doctor` must flag accidentally tracked derived DB, WAL, SHM, and manifest
sidecar files with safe untracking guidance.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled SQLite support), existing workspace crates (`boundline-core`, `boundline-adapters`, `boundline-cli`), and one optional trusted `sqlite-vec` extension-loading path for local vector tables

**Storage**: existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and `.boundline/context-intelligence/retrieval-index.sqlite3`, extended with a companion `.boundline/context-intelligence/manifest.json`, managed `.gitignore` entries, and vector-backed semantic tables inside the same derived SQLite store

**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused unit tests around index lifecycle and semantic capability state, contract tests for CLI JSON and projection shape, targeted integration tests for `plan`, `status`, `inspect`, `init`, and `doctor`, plus compile coverage through targeted workspace test builds

**Target Platform**: macOS and Linux developer workstations, plus Linux CI

**Project Type**: single Rust CLI and library workspace with one root package and supporting member crates

**Execution Model**: sequential session-native execution with one active retrieval query or index maintenance operation at a time, explicit ready or missing or stale or incompatible or degraded or corrupt terminal states, and no hidden background rebuilds

**Observability Surface**: `boundline index status --json`, `boundline index doctor --json`, `boundline init` preview and report output, `.boundline/context-intelligence/manifest.json`, persisted traces, and the `plan`, `status`, `next`, and `inspect` projections that must show semantic engine choice, fallback reason, bounded hybrid outcomes, explicit stale reasons after freshness events, and tracked-artifact recovery guidance

**Performance Goals**: routine refresh touches only changed or removed sources, status and doctor read manifest or DB metadata without a heavy rebuild, and semantic top-k selection runs inside SQLite rather than scoring every semantic row in Rust

**Constraints**: the derived index remains local, disposable, and never a Git-merge artifact; `sqlite-vec` is optional and must degrade explicitly when unavailable; the existing authority order and baseline retrieval correctness cannot change; checkout, merge, pull-with-merge, rebase, and post-rewrite only mark stale or request an explicitly configured bounded lightweight refresh; fetch does nothing; commit hooks remain off by default; no remote embeddings, second vector service, hidden fallback, panic-prone runtime logic, or heavy rebuilds inside Git hooks

**Scale/Scope**: one active workspace, one derived DB plus manifest per workspace, thousands of indexed files and chunk records, one bounded semantic query per decision point, and one index lifecycle CLI surface consumed by humans and skills

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by making the existing advanced-context runtime more reliable, more observable, and cheaper to maintain across normal workspace edits and branch changes. See Summary, Technical Context, and `research.md` Decisions 1 through 6.
- **PASS** Delivery-first scope: The plan prioritizes retrieval correctness, fallback visibility, index lifecycle safety, and Git hygiene ahead of optional hook automation or future retrieval infrastructure. See Summary, Technical Context, and `research.md` Decisions 2 through 6.
- **PASS** Bounded execution: Both retrieval and index maintenance end in explicit ready or missing or stale or incompatible or degraded or corrupt states, with bounded refresh or rebuild actions and no hidden background workers. See Technical Context, `data-model.md`, and `contracts/index-lifecycle-cli-contract.md`.
- **PASS** Stateful execution: The slice reads and writes persisted workspace-local state through the retrieval DB, companion manifest, session traces, and init hygiene surfaces rather than inventing a stateless maintenance path. See Technical Context, `data-model.md`, and `quickstart.md`.
- **PASS** Mutable planning: The feature improves how later planning and inspection steps consume fresh evidence, but it keeps replanning on the existing delivery path instead of creating a separate planning subsystem. See Summary, `research.md` Decision 6, and `contracts/advanced-context-vector-observability-contract.md`.
- **PASS** Sequential-first design: One retrieval query or index maintenance action remains active at a time, and optional Git hooks only mark stale state or request a bounded refresh path for checkout, merge, or rewrite events rather than introducing concurrent background execution or automatic rebuilds. See Technical Context, `research.md` Decision 5, and `quickstart.md`.
- **PASS** Tool-agent symmetry: The source of truth is a typed CLI surface with JSON output that skills can consume, so maintenance logic remains actionable and inspectable instead of hidden inside prompt logic. See `research.md` Decision 4 and `contracts/index-lifecycle-cli-contract.md`.
- **PASS** Required observability and no hidden intelligence: Status, inspect, doctor, and manifest output all make semantic engine choice, vector capability, fallback reasons, stale causes, and recovery actions explicit. See Technical Context, `contracts/advanced-context-vector-observability-contract.md`, and `contracts/index-lifecycle-cli-contract.md`.
- **PASS** Failure as a first-class path: The design names missing capability, failed extension loading, stale manifests, incompatible dimensions, corrupt DB state, tracked derived artifacts, and empty vector content as explicit operator-facing failure cases. See `research.md`, `data-model.md`, and `quickstart.md`.
- **PASS** External separation and strict non-goals: The slice stays local-first, keeps Canon or skills as consumers rather than owners of lifecycle logic, and excludes remote embeddings, graph stores, UI work, and distributed background maintenance. See Summary, Constraints, and `research.md` Decision 4.
- **PASS** Minimal capability slice: The smallest independently valuable closure is one real `sqlite-vec` activation path plus one manifest-driven lifecycle CLI and projection update, all layered onto the current retrieval index. See Summary and `research.md` Decision 1.
- **PASS** Catalog currency: The feature spec recorded the 2026-05-30 provider-doc audit and identified an Anthropic catalog delta unrelated to runtime correctness for this slice; this plan carries that evidence forward and reserves the adjacent catalog refresh as separate repository maintenance rather than expanding the retrieval feature scope. See the Catalog Research & Currency section in `spec.md` and `research.md`.

## Project Structure

### Documentation

```text
specs/065-activate-sqlite-vec/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── advanced-context-vector-observability-contract.md
│   └── index-lifecycle-cli-contract.md
└── tasks.md
```

### Source Code

```text
Cargo.toml
assistant/
└── catalog/
    └── model-catalog.toml

src/
├── cli/
│   ├── init/
│   ├── output.rs
│   ├── output_context.rs
│   └── probe.rs
├── domain/
│   ├── context_intelligence.rs
│   └── workspace_hygiene.rs
├── orchestrator/
│   └── context_intelligence.rs
└── lib.rs

crates/
├── boundline-core/
├── boundline-adapters/
└── boundline-cli/

tests/
├── contract/
└── integration/
```

**Structure Decision**: Keep the implementation inside the existing root
runtime and CLI surfaces that already own advanced-context indexing,
projection, and workspace hygiene. Extend the current derived SQLite index and
add a companion manifest rather than creating a second data store, a daemon, or
another top-level project surface. Use the existing workspace member crates only
when shared library boundaries or adapter integration require it.

## Complexity Tracking

No constitution violations are expected for this slice.
