# Implementation Plan: Evals And Runtime Observability

**Branch**: `072-evals-runtime-observability` | **Date**: 2026-06-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/072-evals-runtime-observability/spec.md`

## Summary

Add a local eval suite, structured runtime event vocabulary, trace compaction policy with five retention classes, JSONL export, and runtime metrics to Boundline. The feature stays read-only over existing `.boundline/traces/` storage, introduces no new CLI command surface beyond `boundline trace compact`, and keeps evals runnable both locally and in CI with per-eval pass/fail summaries. Compaction follows deterministic class-based rules with conservative tiebreaking and hard survival guarantees for decisions and rejection reasons.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only; `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite`, `dialoguer`, `boundline-core`, `boundline-adapters`, `boundline-cli`

**Storage**: Existing workspace-local `.boundline/traces/` for compaction input; additive structured event log under `.boundline/traces/events.jsonl` for JSONL export; eval fixtures and results under a new `.boundline/evals/` directory; no new persistence backend

**Testing**: `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime on supported developer workstations; JSONL export feeds external dashboards; CI-compatible eval runner

**Project Type**: Rust workspace with CLI, runtime, and repository-managed documentation; this feature adds an eval runner and trace compaction command but no new workspace member crate

**Performance Goals**: Compaction classification completes in a single bounded in-memory pass for traces up to 50k items; each structured event emitted before the next runtime phase begins; eval suite completes within 5 minutes for a 50k-item session; no network or model dependency introduced

**Constraints**: Read-only compaction over existing trace storage; no retention policy configuration file in this slice; no automatic or background compaction; per-event-type `schema_version` with additive-field compatibility within major versions; oversized traces (>50k items) require explicit operator confirmation, chunked processing, or actionable failure; no sensitive data in exported events; all changed implementation files require at least 95% changed-file coverage

**Scale/Scope**: One eval suite covering 7 eval dimensions, one trace compaction command, one JSONL export surface, structured event emission for 6+ event types, runtime metrics recording for 11+ metric fields, operator-facing CLI projections, and aligned release metadata for the Boundline release that ships this feature

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | Evals, trace hygiene, and observability directly improve execution reliability by preventing blind regressions and protecting forensic evidence. |
| No abstract agent systems | PASS | No new agent or reasoning framework introduced; eval fixtures are deterministic test cases, compaction is a deterministic classification pass. |
| Bounded execution | PASS | Compaction completes in one bounded pass (50k item limit with oversized-trace guard); evals produce pass/fail results within a 5-minute CI budget. |
| Stateful execution | PASS | Compaction emits trace-visible events; eval results persist as machine-readable summaries; structured events write to `.boundline/traces/events.jsonl`. |
| Mutable planning and execution over perfect planning | PASS | Eval suite starts with 7 curated dimensions and expects iterative expansion; compaction uses conservative defaults to avoid over-engineering classification. |
| Sequential-first design | PASS | Compaction runs synchronously on explicit command; evals run sequentially; no background workers or concurrent passes. |
| Tool-agent symmetry and required observability | PASS | Structured events, JSONL export, compaction events, and metrics all produce inspectable output; eval summaries include per-eval status with source attribution. |
| No hidden intelligence | PASS | All checks are deterministic: compaction classification is rule-based, evals compare expected vs actual outcomes, metrics are counters not inferences. |
| Strict non-goals and minimal capability slice | PASS | No retention policy config file, no automated repair, no dashboard implementation, no eval corpus expansion beyond curated fixtures. |
| Separation from external systems | PASS | JSONL export is the only external contract surface; dashboards are external consumers; Canon interaction is limited to trace-level evidence refs already present. |
| Catalog currency | PASS | Provider catalog audit recorded as no-change (no model-family delta needed for this feature). |
| Rust language rules | PASS | Zero-panic, zero-warning, typed serde models for event shapes, named constants for retention classes and metric keys, modular extraction from existing monoliths. |

## Project Structure

### Documentation (this feature)

```text
specs/072-evals-runtime-observability/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── event-schema-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── evals.rs              # Eval runner, fixture loader, pass/fail summary types
│   ├── trace_compaction.rs   # Compaction policy, classification, tiebreaking
│   └── observability.rs      # Structured event types, metrics snapshots, JSONL export
├── cli/
│   ├── evals.rs               # `boundline evals run` CLI surface
│   ├── trace_compact.rs       # `boundline trace compact` CLI surface
│   ├── trace_export.rs        # `boundline trace export --format jsonl` CLI surface
│   └── output/                # Human-readable and JSON projections for eval/compaction output
│       ├── evals_output.rs
│       └── compaction_output.rs
└── orchestrator/
    └── session_runtime_observability.rs  # Event emission hooks during runtime phases

tests/
├── unit/
│   ├── evals_model.rs
│   ├── trace_compaction_model.rs
│   └── observability_model.rs
├── contract/
│   ├── event_schema_contract.rs
│   ├── eval_output_contract.rs
│   └── compaction_event_contract.rs
└── integration/
    ├── eval_runner_flow.rs
    └── trace_compaction_flow.rs

docs/
├── runtime/
│   ├── evals.md
│   ├── trace-compaction.md
│   └── observability.md
└── guide/
    └── common-workflows.md
```

**Structure Decision**: Keep the single Rust workspace with new domain modules for evals, trace compaction, and observability. Each module owns its typed models, algorithm, and serialization. CLI modules provide the command surface and output projections. No new workspace member crate is introduced — this feature extends `boundline-core` (domain) and `boundline-cli` (presentation).

## Complexity Tracking

> No constitution violations to justify.

## Post-Design Constitution Recheck

*Re-checked after Phase 1 design outputs (data-model.md, contracts/, quickstart.md).*

| Principle | Post-design result | Evidence |
|-----------|-------------------|----------|
| Delivery identity | PASS | Evals protect delivery quality; compaction protects forensic evidence; observability makes delivery inspectable. |
| No abstract agent systems | PASS | All entities are typed records, deterministic classifications, or test fixtures — no agent abstraction introduced. |
| Bounded execution | PASS | `data-model.md` defines bounded compaction (50k items, single pass), bounded evals (7 dimensions, 5 min CI budget). |
| Stateful execution | PASS | `StructuredRuntimeEvent`, `CompactionEvent`, `EvalResult`, and `RuntimeMetrics` all persist to disk; JSONL provides stream replay. |
| Sequential-first | PASS | Compaction is synchronous; evals run sequentially; event emission is phase-synchronous. |
| Tool-agent symmetry + required observability | PASS | `contracts/event-schema-contract.md` defines 7+ event types; `quickstart.md` covers 5 observable scenarios. |
| No hidden intelligence | PASS | Research confirms deterministic classification table, no LLM-assisted compaction; eval comparisons are explicit expected vs actual. |
| Separation from external systems | PASS | JSONL is the only external contract; Canon evidence refs are opaque; dashboards are external consumers. |
| Rust language rules | PASS | Data model uses typed enums (`EventType`, `RetentionClass`, `EvalDimension`, `EvalStatus`), named constants for versions and metric keys. |

All 12 constitution gates pass post-design. The design adds no dependency, network call, background process, hidden inference, or Canon schema change.
