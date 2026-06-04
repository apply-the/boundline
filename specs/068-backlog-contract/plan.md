# Implementation Plan: Backlog Contract

**Branch**: `068-backlog-contract` | **Date**: 2026-06-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/068-backlog-contract/spec.md`

## Summary

Ship the first formal backlog-readiness gate as Boundline-owned runtime
behavior. Reuse and audit the existing `BacklogQualityAssessment`,
planning-gate ordering, status/orchestration projections, and assistant-asset
surfaces already present in the workspace; tighten them against the Canon
`0.67.0` backlog packet contract so only a credible governed backlog can pass
from planning into execution. Close the slice as Boundline `0.69.0` with
aligned release metadata, Canon compatibility guidance, current provider-catalog
research, and at least 95% changed-file coverage for touched Rust
implementation files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only;
`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`,
`rusqlite`, `dialoguer`, `boundline-core`, `boundline-adapters`,
`boundline-cli`

**Storage**: Existing workspace-local session and trace files plus governed
stage artifacts; additive backlog-quality projections only, no new persistence
surface

**Testing**: `cargo test --test unit`, `cargo test --test contract`, focused
backlog-planning integration tests such as `cargo test --test integration
host_session_runtime_flow::`, `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`,
and `scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime on supported developer workstations;
assistant assets for Copilot, Claude, Codex, and Antigravity remain thin
projections over the same CLI/runtime contract

**Project Type**: Rust workspace with CLI, runtime, assistant assets,
distribution metadata, and repository-managed documentation

**Performance Goals**: Keep backlog-quality evaluation bounded to one
in-memory assessment per planning or execution-admission decision, add no new
network dependency, and preserve one-question recovery behavior

**Constraints**: No new CLI command; no Canon file changes; no hidden fallback
ordering; no parallel execution semantics; Canon `0.67.0` already supplies the
packet evidence this consumer slice needs, so Boundline must validate the
packet literally instead of widening scope with heuristics

**Scale/Scope**: One backlog-readiness gate, additive session/orchestration
projections, one-question recovery, plan/run assistant parity, Canon `0.67.0`
compatibility alignment, user docs, tech docs, and release `0.69.0` closure

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | The slice blocks unsafe execution handoff when governed backlog evidence is weak; it directly improves delivery reliability. |
| Bounded execution | PASS | One backlog-quality assessment runs per planning decision, emits at most one `phase_request`, and introduces no background processing. |
| Stateful execution | PASS | Assessment, findings, task count, MVP scope, unmapped items, and recovery transitions remain visible through session and trace surfaces. |
| Mutable planning and execution over perfect planning | PASS | Clarification-required backlog packets stay in the same planning session and resume after focused input or regenerated Canon output. |
| Sequential-first design | PASS | The gate stays within the existing `goal -> plan -> backlog -> analysis` ordering and asks one question at a time. |
| Tool-agent symmetry and required observability | PASS | CLI, status, orchestration, inspect, traces, and assistant assets project the same runtime-owned backlog decision. |
| No hidden intelligence | PASS | The plan requires explicit blocked versus clarification semantics and forbids silent dependency or ordering synthesis. |
| Strict non-goals and minimal capability slice | PASS | No `/boundline-tasks` command, no Canon schema changes, no planning-analysis expansion, and no execution scheduler work are added. |
| Real acceptance criteria and failure-first behavior | PASS | The spec and quickstart cover blocked, clarification-required, ready, compatibility, and assistant-parity scenarios. |
| Separation from external systems | PASS | Canon remains a governed producer only; Boundline owns validation and stays independently testable. |
| Catalog currency | PASS | `research.md` records a 2026-06-03 public-provider audit and explicit no-change rationale for `assistant/catalog/model-catalog.toml`. |
| Rust language rules | PASS | The implementation remains typed, additive, traceable, and subject to zero-panic, zero-warning, and changed-file coverage constraints. |

## Project Structure

### Documentation (this feature)

```text
specs/068-backlog-contract/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── backlog-quality-runtime-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── governance.rs
│   ├── goal_plan.rs
│   └── session.rs
├── orchestrator/
│   ├── session_runtime.rs
│   └── session_runtime_planning_runtime.rs
└── cli/
    ├── session.rs
    ├── output_session_status.rs
    └── output_orchestrate.rs

assistant/
├── antigravity/commands/
│   ├── boundline-plan.md
│   └── boundline-run.md
├── claude/commands/
│   ├── boundline-plan.md
│   └── boundline-run.md
├── codex/commands/
│   ├── boundline-plan.md
│   └── boundline-run.md
├── copilot/prompts/
│   ├── boundline-plan.prompt.md
│   └── boundline-run.prompt.md
└── catalog/model-catalog.toml

tests/
├── unit/
│   ├── governance_policy.rs
│   ├── session_cli_runtime.rs
│   └── cli_output.rs
├── contract/
│   ├── assistant_command_definition_contract.rs
│   ├── host_command_output_contract.rs
│   └── planning_gate_pipeline_contract.rs
└── integration/
    └── host_session_runtime_flow.rs

docs/
├── runtime/
│   ├── plan.md
│   ├── phase-requests.md
│   ├── status.md
│   └── inspect.md
├── guide/
│   ├── common-workflows.md
│   └── getting-started.md
└── architecture/runtime-model.md

tech-docs/
├── architecture.md
├── configuration.md
└── getting-started.md

distribution/
├── channel-metadata.toml
├── homebrew/Formula/boundline.rb
└── winget/manifests/a/ApplyThe/Boundline/0.69.0/
```

**Structure Decision**: Keep the current single Rust workspace and treat
backlog quality as an audit-and-complete slice over code that already exists in
the runtime, CLI projections, assistant assets, and docs. Start with a strict
Canon `0.67.0` packet compatibility audit before changing behavior so the
implementation stays inside the governed producer contract and does not drift
into Boundline-only heuristics.

## Phase 0 Research Conclusions

- Reuse the existing typed backlog-quality domain owner in
  `src/domain/governance.rs`; do not introduce a second validator.
- Keep the gate order deterministic: `goal quality -> plan quality -> backlog
  quality -> planning analysis -> execution handoff`.
- Validate only Canon `0.67.0` backlog evidence already present in the emitted
  packet. The Canon follow-up now supplies stable slice identity and additive
  execution-handoff evidence, so Boundline should consume that packet literally
  rather than inventing hidden heuristics.
- Treat closure-limited or risk-only Canon backlog packets as blocked, while
  structurally credible but incomplete full packets may use
  `clarification_required` and one `phase_request`.
- Ship release closure as Boundline `0.69.0`, align Canon compatibility-facing
  surfaces from `0.63.0` to `0.67.0`, and keep the public provider catalog
  audit current even when no bundled model-family delta is required.

## Phase 1 Design Outputs

- [research.md](research.md) records the Canon `0.67.0` audit boundary, current
  provider-catalog evidence, and rejected alternatives.
- [data-model.md](data-model.md) defines backlog assessment state, finding
  classes, packet evidence boundaries, and recovery semantics.
- [backlog-quality-runtime-contract.md](contracts/backlog-quality-runtime-contract.md)
  defines additive projections, evaluation order, blocked versus clarification
  semantics, and assistant-safe routing.
- [quickstart.md](quickstart.md) defines isolated runtime validation scenarios
  and release-quality commands.

## Post-Design Constitution Recheck

The design remains compliant after Phase 1. It adds no dependency, command,
provider abstraction, network dependency, hidden fallback behavior, or
concurrency. The packet audit gate preserves Canon ownership boundaries and
keeps this slice honest: Canon `0.67.0` now emits the required signals, so
Boundline implementation must validate them literally rather than widening
scope or silently lowering the bar.
