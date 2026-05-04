# Implementation Plan: Checkpoint Rewind

**Branch**: `041-checkpoint-rewind` | **Date**: 2026-05-04 | **Spec**: [/Users/rt/workspace/synod/specs/041-checkpoint-rewind/spec.md](/Users/rt/workspace/synod/specs/041-checkpoint-rewind/spec.md)
**Input**: Feature specification from `/specs/041-checkpoint-rewind/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Ship `0.41.0` as one full checkpoint-and-rewind release by refounding Boundline
into a Rust workspace with explicit `boundline-core`, `boundline-adapters`, and
`boundline-cli` crates, adding implicit pre-mutation workspace checkpoints for
session-native `run` and `step`, exposing `checkpoint list` and
`checkpoint restore <id>`, preserving cluster authority explicitly, and
updating the docs so the quick path stays lightweight while advanced routing,
cluster, and Canon detail moves into the architecture layer.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`; no new non-standard runtime dependency is required for checkpoint persistence  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, `.boundline/cluster.toml`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and new workspace-local `.boundline/checkpoints/` manifests plus captured file payloads  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit/integration/contract tests, `cargo test --workspace --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Rust workspace with three library or binary members under one repo-root CLI surface  
**Execution Model**: Sequential session-native orchestration with one active step at a time, explicit compatibility routing remaining subordinate, and explicit checkpoint capture before mutating execution  
**Observability Surface**: Persisted session state, checkpoint manifests, append-only traces, CLI summaries on `run`, `status`, `next`, `inspect`, and repo-root docs plus assistant guidance that keep Boundline as the operational control plane and Canon as the governed companion  
**Performance Goals**: Pre-mutation checkpoint capture must stay within bounded local filesystem work for the targeted workspace slice; operators should identify restore guidance from normal CLI output in under 2 minutes; maintainers should be able to validate the release from the repo root in under 20 minutes  
**Constraints**: No Git-only dependency, no remote storage, no automatic restore on failure, no restoration outside declared workspace scope, no hidden background processes, and no change to the existing repo-root command vocabulary beyond adding the `checkpoint` command group  
**Scale/Scope**: One workspace or registered cluster at a time, bounded by existing step/retry limits and by the active delivery slice rather than by full-repository archival snapshots

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering delivery by adding reversible safety around workspace mutation and making that safety inspectable on the same delivery surfaces.
- **PASS** Delivery-first scope: Execution safety, state persistence, restore behavior, and command-surface continuity are prioritized ahead of optimization or cleanup-only refactors.
- **PASS** Primary workflow: The main operator path remains session-native `start -> capture -> plan -> run -> status -> next -> inspect`; explicit compatibility remains available but subordinate and does not own checkpoint control flow.
- **PASS** Bounded execution: Checkpoints are created only before bounded mutating `run` or `step` execution, within existing step and retry limits, and restore ends in explicit success or refusal.
- **PASS** Stateful execution: Session state, trace history, checkpoint manifests, restore records, and cluster authority all persist in workspace-owned state under `.boundline/`.
- **PASS** Mutable planning: The plan preserves existing goal-plan and replanning behavior while adding checkpoint capture around the mutating execution phase and restore events after failure.
- **PASS** Sequential-first design: Execution remains one step at a time; checkpoint capture and restore are explicit sequential operations with no hidden background worker.
- **PASS** Tool-agent symmetry: The runtime continues to expose explicit read, modify, test, ask, and replan actions while checkpoint capture and restore become explicit visible actions instead of hidden filesystem magic.
- **PASS** Observability and explicit intelligence: Checkpoint identity, restore suggestion, refusal reasons, and restore events are surfaced through normal CLI and persisted trace or checkpoint state.
- **PASS** Non-goals and external separation: The feature remains independently usable without Canon, does not add distributed orchestration, remote storage, UI work, or generalized Git rollback, and keeps Canon as an optional governed companion only.
- **PASS** Minimal slice: The smallest independently valuable capability is reversible local mutation safety on existing session-native execution plus the crate refoundation necessary to keep that safety maintainable.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/041-checkpoint-rewind/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Keep the structure minimal, delivery-focused, and sequential-
  first. Do not introduce extra top-level projects or UI/runtime surfaces unless
  the Constitution Check explicitly justifies them.
-->

```text
Cargo.toml
crates/
├── boundline-core/
│   ├── Cargo.toml
│   └── src/
│       ├── domain/
│       ├── orchestrator/
│       └── lib.rs
├── boundline-adapters/
│   ├── Cargo.toml
│   └── src/
│       ├── adapters/
│       ├── fixture.rs
│       └── lib.rs
└── boundline-cli/
  ├── Cargo.toml
  └── src/
    ├── cli/
    ├── bin/
    └── lib.rs

tests/
├── contract/
├── integration/
├── support/
└── unit/
```

**Structure Decision**: Introduce a three-member Rust workspace because the
roadmap explicitly defers the workspace refoundation into this safety slice and
because checkpoint state, session orchestration, and CLI command handling now
need clear boundaries. The added top-level `crates/` directory is justified by
the roadmap requirement to stop accumulating filesystem mutation, persistence,
and command wiring inside one crate while preserving the repo-root operator
surface.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., background worker] | [specific delivery need] | [why sequential execution is insufficient] |
