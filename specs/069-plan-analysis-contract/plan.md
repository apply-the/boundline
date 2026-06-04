# Implementation Plan: Plan Analysis Contract

**Branch**: `069-plan-analysis-contract` | **Date**: 2026-06-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/069-plan-analysis-contract/spec.md`

## Summary

Complete the existing `planning_analysis` scaffolding as the final
Boundline-owned planning gate before execution. Reuse the typed runtime
projection already present in `GoalPlan`, widen it from its current
backlog-only checks into a deterministic end-to-end coherence audit across the
goal, plan outcomes, validation strategy, backlog packet, risks, constraints,
execution readiness, and governed Canon evidence, and keep the feature
strictly read-only. Close the slice as Boundline `0.70.0` with aligned docs,
assistant assets, release metadata, an explicit Canon `0.67.0` compatibility
story, a recorded provider-catalog no-change audit, and at least 95%
changed-file coverage for touched Rust implementation files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only;
`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`,
`rusqlite`, `dialoguer`, `boundline-core`, `boundline-adapters`,
`boundline-cli`

**Storage**: Existing workspace-local session and trace files plus governed
stage artifacts; additive planning-analysis projections only, no new
persistence surface

**Testing**: `cargo test --test unit`, `cargo test --test contract`, focused
planning runtime integration tests such as `cargo test --test integration
host_session_runtime_flow::`, `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`,
and `scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime on supported developer workstations;
assistant assets for Copilot, Claude, Codex, and Antigravity remain thin
projections over the same CLI/runtime contract

**Project Type**: Rust workspace with CLI, runtime, assistant assets,
distribution metadata, and repository-managed documentation

**Performance Goals**: Keep planning-analysis evaluation bounded to one
deterministic in-memory assessment per planning or execution-admission
decision, add no network or model dependency, and deduplicate materially
identical findings before they reach operators or hosts

**Constraints**: Read-only only; no Canon packet mutation or schema change; no
new CLI command; preserve the gate order of `goal quality -> plan quality ->
backlog quality -> planning analysis -> execution handoff`; preserve older
session snapshot compatibility; do not introduce heuristic semantic
contradiction inference beyond explicit deterministic signals already owned by
Boundline or Canon; do not run `boundline` CLI commands against this
repository root; all changed implementation files require at least 95%
changed-file coverage

**Scale/Scope**: One coherence gate over the active goal-derived plan and
ready backlog packet, additive session/orchestration/assistant projections,
focused repair routing, user docs, tech docs, release `0.70.0` closure, and
continued Canon `0.67.0` compatibility

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | The slice controls execution admission for real engineering work by rejecting incoherent planning states before implementation starts. |
| Bounded execution | PASS | One deterministic assessment runs per planning decision, asks at most one follow-up through existing continuation routes, and introduces no background process. |
| Stateful execution | PASS | Assessment state, findings, coverage, and withheld handoff remain persisted and trace-visible through the existing session model. |
| Mutable planning and execution over perfect planning | PASS | The gate diagnoses repairable coherence gaps without rewriting the plan; the operator fixes the same session and resumes. |
| Sequential-first design | PASS | Planning analysis remains the final planning gate after goal, plan, and backlog quality, with no parallel branches. |
| Tool-agent symmetry and required observability | PASS | CLI, status, inspect, orchestration, traces, and assistant assets project the same runtime-owned decision and continuation boundary. |
| No hidden intelligence | PASS | Checks are limited to explicit deterministic invariants over typed Boundline state and governed Canon evidence; no silent inference engine or chat-only synthesis is introduced. |
| Strict non-goals and minimal capability slice | PASS | No standalone analysis command, no Canon-side schema work, no automated repair, and no speculative semantic parser are added in this slice. |
| Real acceptance criteria and failure-first behavior | PASS | The spec and quickstart cover blocked, warning-only, producer-gap, compatibility, and assistant-parity scenarios in isolated temporary workspaces. |
| Separation from external systems | PASS | Canon stays a read-only governed producer; Boundline owns the coherence decision and remains independently testable. |
| Catalog currency | PASS | `research.md` records a 2026-06-04 public-provider audit and explicit no-change rationale for `assistant/catalog/model-catalog.toml`. |
| Rust language rules | PASS | The implementation remains typed, additive, traceable, zero-panic, zero-warning, and modular, with extraction expected from the oversized current `goal_plan.rs` planning-analysis logic. |

## Project Structure

### Documentation (this feature)

```text
specs/069-plan-analysis-contract/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── planning-analysis-runtime-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── goal_plan.rs
│   ├── governance.rs
│   └── session.rs
├── orchestrator/
│   ├── session_runtime.rs
│   └── session_runtime_planning_runtime.rs
└── cli/
    ├── inspect/projections.rs
    ├── output_orchestrate.rs
    ├── output_session_status.rs
    └── session.rs

assistant/
├── antigravity/commands/
│   ├── boundline-inspect.md
│   ├── boundline-plan.md
│   ├── boundline-run.md
│   └── boundline-status.md
├── claude/commands/
│   ├── boundline-inspect.md
│   ├── boundline-plan.md
│   ├── boundline-run.md
│   └── boundline-status.md
├── codex/commands/
│   ├── boundline-inspect.md
│   ├── boundline-plan.md
│   ├── boundline-run.md
│   └── boundline-status.md
├── copilot/prompts/
│   ├── boundline-inspect.prompt.md
│   ├── boundline-plan.prompt.md
│   ├── boundline-run.prompt.md
│   └── boundline-status.prompt.md
└── catalog/model-catalog.toml

tests/
├── unit/
│   ├── cli_output.rs
│   ├── goal_plan_model.rs
│   └── session_cli_runtime.rs
├── contract/
│   ├── assistant_command_definition_contract.rs
│   ├── host_command_output_contract.rs
│   └── planning_gate_pipeline_contract.rs
└── integration/
    └── host_session_runtime_flow.rs

docs/
├── runtime/
│   ├── inspect.md
│   ├── plan.md
│   ├── phase-requests.md
│   └── status.md
│   └── trace.md
└── guide/
    └── common-workflows.md

tech-docs/
├── architecture.md
├── configuration.md
└── getting-started.md

distribution/
├── channel-metadata.toml
├── homebrew/Formula/boundline.rb
└── winget/manifests/a/ApplyThe/Boundline/0.70.0/
```

**Structure Decision**: Keep the current single Rust workspace and treat plan
analysis as an audit-and-complete slice over behavior that already exists in
the runtime and assistant surfaces. The implementation should extract new
planning-analysis-specific helpers out of the existing monolithic
`src/domain/goal_plan.rs` path where needed, but must keep one runtime-owned
analysis contract rather than creating a second validator or an external
planning service.

## Phase 0 Research Conclusions

- Reuse the existing typed `PlanningAnalysisProjection` surface in
  `src/domain/goal_plan.rs`, but widen it from its current
  backlog-unmapped-items and missing-expected-outcomes checks into a real
  coherence gate.
- Keep the planning gate order deterministic: `goal quality -> plan quality ->
  backlog quality -> planning analysis -> execution handoff`.
- Limit the first release to deterministic coherence checks over typed
  Boundline fields and governed Canon evidence already available in the active
  session. Do not introduce an LLM-backed or free-form contradiction engine.
- Treat contradiction detection as explicit typed-artifact conflict checking
  only, including plan-versus-backlog and risk-or-constraint mismatches that
  can be grounded in current runtime fields or governed packet evidence.
- Treat missing Canon-owned data as a producer contract gap that blocks
  execution, not as a heuristic fallback or synthesized field.
- Preserve risk and constraint coverage as first-class planning-analysis
  signals whenever the active plan or governed packet already exposes them.
- Preserve additive backward compatibility: older snapshots may omit
  planning-analysis fields entirely; newer sessions persist them only after the
  gate runs.
- Keep the provider-catalog audit explicit even though this feature does not
  require a bundled model-family delta.

## Phase 1 Design Outputs

- [research.md](research.md) records the provider-catalog no-change audit, the
  deterministic coherence boundary, and rejected alternatives.
- [data-model.md](data-model.md) defines persisted planning-analysis state,
  finding classes, coverage summaries, and source attribution.
- [planning-analysis-runtime-contract.md](contracts/planning-analysis-runtime-contract.md)
  defines additive projections, evaluation order, blocked versus warning-only
  semantics, producer-gap behavior, and assistant-safe routing.
- [quickstart.md](quickstart.md) defines isolated runtime validation scenarios,
  targeted contract checks, and release-quality commands.

## Post-Design Constitution Recheck

The design remains compliant after Phase 1. It adds no dependency, command,
network call, external model invocation, Canon schema work, background
processing, or hidden fallback behavior. The coherence gate is deliberately
bounded: it uses explicit runtime-visible signals to decide whether planning is
credible enough for execution, and it leaves broader semantic review or
automated repair to later roadmap slices.
