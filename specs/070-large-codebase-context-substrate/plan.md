# Implementation Plan: Large Codebase Context Substrate

**Branch**: `070-large-codebase-context-substrate` | **Date**: 2026-06-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/070-large-codebase-context-substrate/spec.md`

## Summary

Add a Boundline-owned large-codebase context substrate that keeps context packs
safe, inspectable, and deterministic when repositories or artifacts become too
large for naive full reads. The first slice should extend the existing local
context-intelligence, project-index, goal-planning, and session projection
surfaces with explicit fidelity tiers, inclusion modes, omission reasons,
critical-context blocking, repository-map-assisted search-before-read,
digest-backed compaction, patch-safe edit guidance, and a derived
snapshot-cache boundary that remains explicitly separate from memory. Close the
slice as Boundline `0.72.5`, keep Canon compatibility guidance at `0.67.0`,
record a provider-catalog audit, and require at least 95% changed-file coverage
for touched Rust implementation files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only;
`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`,
`rusqlite`, `dialoguer`, `boundline-core`, `boundline-adapters`,
`boundline-cli`; reuse the existing local SQLite + FTS5 retrieval substrate and
the optional `sqlite-vec` acceleration path when present, but do not introduce
new runtime dependencies in this slice

**Storage**: Existing workspace-local `.boundline/session.json`,
`.boundline/traces/`, `.boundline/config.toml`, `.boundline/execution.json`,
`.boundline/workflows.toml`, and `.boundline/context-intelligence/` derived
index files, extended additively with context-substrate projections, omission
findings, repository-map metadata, and snapshot-cache freshness state only

**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets
--all-features -- -D warnings`, targeted `cargo test --test unit`,
`cargo test --test contract`, focused large-context integration tests under
`tests/integration/`, `cargo llvm-cov --workspace --all-features --lcov
--output-path lcov.info`, and
`scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime on supported developer workstations and
CI; assistant assets for Copilot, Claude, Codex, and Antigravity remain thin
projections over the same runtime-owned contract

**Project Type**: Rust workspace with CLI, runtime, derived local indexing,
assistant assets, documentation, and distribution metadata

**Performance Goals**: Refuse or downgrade unsafe oversized full reads in 100%
of validated cases, block on critical-context omission in 100% of validated
cases, keep initial context-pack selection within 10 seconds for at least 95%
of maintained large-repository fixture runs, and explain inclusion or omission
decisions from `status` or `inspect` fast enough for an operator to diagnose
the pack without reading raw files

**Constraints**: Local-only and deterministic; no remote retrieval service; no
new background crawler; no Canon schema mutation; no memory promotion; no full
repository semantic parser; no hidden heuristics; no new CLI command required
for the initial slice; preserve sequential planning/runtime control flow; do
not run `boundline` CLI commands against this repository root; all changed Rust
implementation files must stay above 95% changed-file coverage

**Scale/Scope**: One active workspace at a time, one authoritative bounded
context pack per planning step, one derived repository map and snapshot-cache
state per workspace, additive session and trace projections, assistant/runtime
surface updates, docs, release metadata, and quality closure for Boundline
`0.72.5`

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | The slice improves real engineering delivery by preventing unsafe or silently lossy context selection before planning and execution continue. |
| Bounded execution | PASS | Context selection remains one bounded deterministic build per planning step with explicit refusal, downgrade, or blocked outcomes; no background worker or unbounded scan is introduced. |
| Stateful execution | PASS | Context fidelity, inclusion decisions, omission findings, repository-map freshness, and cache freshness remain persisted in Boundline-owned runtime/session state and traces. |
| Mutable planning and execution over perfect planning | PASS | The substrate blocks or narrows unsafe context while preserving the same bounded repair and replanning paths instead of attempting exhaustive repository understanding. |
| Sequential-first design | PASS | One authoritative context pack remains active at a time; repository discovery and compaction happen inside the existing sequential planning/runtime path. |
| Tool-agent symmetry and required observability | PASS | `status`, `inspect`, traces, diagnostics, and assistant assets project the same included and omitted context decisions with source attribution and repair guidance. |
| No hidden intelligence | PASS | Ranking, tiering, omission, freshness, and compaction must be explainable through explicit local signals; no LLM scoring or silent fallback path is introduced. |
| Strict non-goals and minimal capability slice | PASS | The slice does not add reviewed memory, provider-owned retrieval, Canon-owned indexing, or a generalized semantic contradiction engine. It stays on local context selection and derived cache boundaries only. |
| Real acceptance criteria and failure-first behavior | PASS | The spec and quickstart cover unsafe large reads, critical omission blocking, stale cache invalidation, and compacted-artifact explanation using isolated fixtures. |
| Separation from external systems | PASS | Canon remains optional enrichment and provider surfaces remain unchanged; the core context substrate remains independently testable from local workspace state. |
| Catalog currency | PASS | `research.md` records a 2026-06-05 provider-doc audit and explicit no-change rationale for `assistant/catalog/model-catalog.toml`. |
| Rust language rules | PASS | The design stays additive, typed, modular, zero-panic outside `main.rs`, and should prefer helper extraction over large decision functions when implementation lands. |

## Project Structure

### Documentation (this feature)

```text
specs/070-large-codebase-context-substrate/
├── spec.md
├── feat-large-codebase-context-substrate.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── large-codebase-context-runtime-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── context_intelligence.rs
│   ├── goal_plan.rs
│   ├── governance.rs
│   ├── project_index.rs
│   ├── project_memory.rs
│   └── session.rs
├── orchestrator/
│   ├── context_intelligence.rs
│   ├── goal_planner.rs
│   ├── session_runtime.rs
│   └── session_runtime_planning_context.rs
└── cli/
    ├── diagnostics.rs
    ├── inspect/projections.rs
    ├── output_context.rs
    ├── output_orchestrate.rs
    ├── output_runtime.rs
    ├── output_session_status.rs
    └── session.rs

tests/
├── unit/
│   ├── goal_plan_model.rs
│   ├── context_intelligence_model.rs
│   └── cli_output.rs
├── contract/
│   ├── host_command_output_contract.rs
│   └── assistant_command_definition_contract.rs
└── integration/
    ├── context_intelligence_semantic_inspect.rs
    ├── context_intelligence_semantic_flow.rs
    └── host_session_runtime_flow.rs

assistant/
├── antigravity/commands/
├── claude/commands/
├── codex/commands/
├── copilot/prompts/
└── catalog/model-catalog.toml

docs/
├── runtime/
│   ├── inspect.md
│   ├── plan.md
│   ├── run.md
│   └── status.md
├── architecture/
│   ├── context-intelligence.md
│   └── runtime-model.md
└── reference/
    ├── cli.md
    └── file-layout.md

tech-docs/
├── architecture.md
├── configuration.md
└── getting-started.md

distribution/
├── channel-metadata.toml
├── homebrew/Formula/boundline.rb
└── winget/manifests/a/ApplyThe/Boundline/0.72.5/
```

**Structure Decision**: Keep the feature inside the existing single Rust
workspace and extend current context-planning surfaces instead of creating a
second retrieval or memory subsystem. `src/domain/context_intelligence.rs`,
`src/orchestrator/context_intelligence.rs`, `src/domain/project_index.rs`, and
`src/orchestrator/goal_planner.rs` already own the closest primitives for
repository discovery, derived local indexing, and context-pack construction, so
the implementation should refine those surfaces rather than introducing a new
top-level engine.

## Phase 0 Research Conclusions

- Reuse the existing local context-intelligence and goal-planning stack rather
  than adding a separate large-codebase service.
- Represent fidelity tier, inclusion mode, omission reasons, compaction state,
  and snapshot-cache freshness as typed Boundline-owned runtime data.
- Keep repository discovery deterministic and search-first: use local path,
  symbol, import/export, test, trace, and Canon-relation signals before any
  oversized full read is attempted.
- Treat the repository navigation map as a compact derived aid for selection,
  not as a full authoritative semantic database.
- Keep digest-backed compaction and patch-safe editing explicit and traceable;
  large artifacts may be summarized or referenced by digest, but operators must
  be able to resolve the original source.
- Define the persistent snapshot cache as derived, local, disposable,
  rebuildable, and non-authoritative; it must never be confused with reviewed
  memory or Canon-governed truth.
- Keep provider-model routing unchanged for this slice; the required provider
  audit produced a no-change result.

## Phase 1 Design Outputs

- [research.md](research.md) records the provider-catalog audit, the local-only
  substrate decisions, the cache-versus-memory boundary, and rejected
  alternatives.
- [data-model.md](data-model.md) defines context candidates, context-pack
  entries, omission findings, repository-map snapshots, digest-backed
  references, snapshot-cache entries, and patch-safe edit attempts.
- [large-codebase-context-runtime-contract.md](contracts/large-codebase-context-runtime-contract.md)
  defines additive runtime projection, state vocabularies, blocking rules,
  cache freshness, diagnostics, and assistant-surface obligations.
- [quickstart.md](quickstart.md) defines isolated validation scenarios for
  unsafe full reads, critical omission blocking, inspectable inclusion and
  omission, stale cache invalidation, and digest-backed compaction.

## Post-Design Constitution Recheck

The design remains compliant after Phase 1. It introduces no remote retrieval,
no background indexing loop, no memory promotion, no Canon-owned control flow,
and no hidden model-based ranking. The substrate is deliberately bounded: it
uses local signals to keep context packs safe and explainable in large
repositories while leaving reviewed memory, broader provider integration, and
future external capability protocols to later roadmap slices.
