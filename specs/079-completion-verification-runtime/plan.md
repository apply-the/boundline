# Implementation Plan: Boundline Completion Verification Runtime

**Branch**: `079-completion-verification-runtime` | **Date**: 2026-06-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/079-completion-verification-runtime/spec.md`

## Summary

Add a runtime-owned completion-verification gate that blocks task, stage, and run closeout until a claim-matched proof has been executed freshly in the current working state. The design stays sequential-first and additive: task claims remain the proof unit, parent scopes aggregate child readiness, proof freshness is decided by a normalized workspace content fingerprint, and status/inspect/orchestrate projections surface blocked, stale, failed, and missing-proof reasons without redefining the existing task lifecycle.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only; `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite`, `dialoguer`, `boundline-core`, `boundline-adapters`, `boundline-cli`

**Storage**: Existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, and related session artifacts; additive completion-verification records embedded in existing session, task, and trace persistence surfaces; no new external persistence backend

**Testing**: `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`

**Target Platform**: Local Boundline CLI runtime on supported developer workstations, with `status`, `inspect`, and `orchestrate` as the primary operator-visible surfaces

**Project Type**: Rust workspace with CLI, domain, and orchestrator layers; no new workspace member crate in this slice

**Performance Goals**: Proof selection and closeout gating add bounded synchronous overhead only at closeout time; normalized workspace fingerprint calculation remains practical for normal developer workspaces; blocked-state rendering remains immediate enough to keep CLI closeout interactive

**Constraints**: No new CLI command in the first slice; sequential execution only; one active proof command per claim; one blocked completion state at a time; no Canon runtime dependency; no hidden heuristics; no panic-prone control flow outside `main.rs`; additive status fields must not break existing consumers

**Scale/Scope**: One dominant claim per closeout attempt, four initial claim kinds (`tests_pass`, `bug_fixed`, `build_clean`, `migration_valid`), one deterministic proof-selection path per claim, deterministic claim-relevant documentation inclusion based on the active claim or proof rule, stage/run aggregation over required child scopes, and contract coverage for task, stage, and run projections

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | The slice blocks unsafe completion and improves bounded engineering execution reliability at task, stage, and run closeout. |
| No abstract agent systems | PASS | No new agent layer is introduced; claim derivation, proof selection, fingerprint invalidation, and projection rendering are deterministic runtime behaviors. |
| Bounded execution | PASS | Verification only runs at explicit closeout points, one proof command at a time, with blocked states and terminal outcomes surfaced explicitly. |
| Stateful execution | PASS | Claim source, proof refs, workspace fingerprints, child readiness counts, and findings persist into existing session/task/trace views. |
| Mutable planning and execution over perfect planning | PASS | The feature prefers explicit claim/proof correction loops over complex speculative planning or background verification. |
| Sequential-first design | PASS | The first slice forbids speculative parallel proof scheduling and aggregates child state synchronously. |
| Tool-agent symmetry and required observability | PASS | Proof commands, changed-path explanations, child readiness counts, and findings become visible in status, inspect, orchestrate, and traces. |
| No hidden intelligence | PASS | Claim inference confidence, operator confirmation rules, stale-proof reasons, and proof-selection precedence are explicit and contractable. |
| Strict non-goals and minimal capability slice | PASS | No new CLI command, no Canon packet generation dependency, no parent-scope proof replacement, and no speculative background watchers are included. |
| Separation from external systems | PASS | Canon consumes evidence later but does not control runtime closeout, and the feature runs locally without external governance services. |
| Catalog currency | PASS | `research.md` records a 2026-06-12 provider-doc audit and explicit no-change rationale for `assistant/catalog/model-catalog.toml`. |
| Rust language rules | PASS | Planned models are typed serde structs/enums, runtime states avoid panic-prone control flow, and constants/typed vocabularies replace magic literals. |

## Project Structure

### Documentation (this feature)

```text
specs/079-completion-verification-runtime/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── completion-verification-projection.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── task.rs                         # Existing task lifecycle state to extend with verification-owned projections
│   ├── session.rs                      # Existing session status view and persisted session-side records
│   ├── trace.rs                        # Existing evidence and trace references
│   └── completion_verification.rs      # New typed claim, proof, finding, fingerprint, and aggregation models
├── orchestrator/
│   ├── session_runtime_finalization.rs # Closeout gating for task/stage/run transitions
│   ├── session_runtime_step_execution.rs
│   ├── session_runtime_surface.rs      # Parent-scope projections and surfaced blocked reasons
│   ├── session_runtime_observability.rs
│   └── session_runtime_tests.rs
├── cli/
│   ├── inspect.rs
│   ├── inspect/projections.rs          # Inspect contract rendering
│   ├── output_orchestrate.rs           # Orchestrate response rendering
│   ├── output_session_status.rs        # Status rendering rules
│   └── output_runtime.rs               # Shared completion-verification text helpers
└── src/cli.rs                          # Command dispatch entrypoints using existing status/orchestrate/inspect commands

tests/
├── unit/
│   ├── completion_verification_model.rs
│   ├── completion_verification_fingerprint.rs
│   └── completion_verification_selection.rs
├── contract/
│   ├── completion_verification_projection_contract.rs
│   └── completion_verification_parent_scope_contract.rs
└── integration/
    ├── completion_verification_task_flow.rs
    ├── completion_verification_stale_flow.rs
    └── completion_verification_stage_run_flow.rs
```

**Structure Decision**: Keep the single Rust workspace and extend existing task/session/orchestrator/CLI surfaces instead of creating a second runtime path. Introduce one focused `completion_verification` domain module for typed models and keep rendering logic inside existing status/inspect/orchestrate output modules.

## Complexity Tracking

> No constitution violations to justify.

## Post-Design Constitution Recheck

*Re-checked after Phase 1 design outputs (`data-model.md`, `contracts/`, `quickstart.md`).*

| Principle | Post-design result | Evidence |
|-----------|-------------------|----------|
| Delivery identity | PASS | `data-model.md` keeps proof ownership at task closeout and makes parent scopes aggregate delivery readiness rather than inventing abstract governance behavior. |
| No abstract agent systems | PASS | The design uses typed records such as `CompletionClaim`, `ProofRunRecord`, and `ChildVerificationSummary`; no agent abstraction is added. |
| Bounded execution | PASS | `quickstart.md` and `contracts/completion-verification-projection.md` keep proof execution explicit, sequential, and limited to closeout time. |
| Stateful execution | PASS | Fingerprints, proof refs, findings, and child counters persist in explicit runtime records and surface in status/inspect/orchestrate projections. |
| Sequential-first | PASS | Parent-scope aggregation summarizes child readiness synchronously; no background verification or concurrent proof execution is introduced. |
| Required observability | PASS | The contract defines additive status fields, structured findings, changed-path reporting, and child summary counters for task, stage, and run scopes. |
| No hidden intelligence | PASS | Claim-source precedence, inference confidence, confirmation triggers, and stale-proof conditions are explicit in `research.md` and `data-model.md`. |
| Separation from external systems | PASS | The design emits Canon-consumable evidence refs but does not require Canon packet generation or remote approvals to function. |
| Rust language rules | PASS | The proposed model set is fully typed, additive, and compatible with zero-panic / typed-serde / named-vocabulary repository rules. |

All constitution gates pass post-design. No unresolved clarification remains in the planning packet.
