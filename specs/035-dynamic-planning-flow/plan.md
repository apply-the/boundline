# Implementation Plan: Dynamic Planning And Flow Inference

**Branch**: `035-dynamic-planning-flow` | **Date**: 2026-05-02 | **Spec**: [specs/035-dynamic-planning-flow/spec.md](specs/035-dynamic-planning-flow/spec.md)
**Input**: Feature specification from `/specs/035-dynamic-planning-flow/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Replace Boundline's keyword-first flow inference and stage-static goal planning with
an evidence-driven infer -> propose -> confirm planning loop that derives flow,
targets, and verification strategy from context packs plus observed workspace
evidence. Keep the operator-facing CLI surface bounded by making `boundline plan`
produce a proposal, `boundline plan --confirm` confirm the current proposal, and
`boundline plan --replan` supersede the prior proposal with a bounded revision when
new evidence invalidates it. Preserve workflows as planning guardrails,
propagate proposal and replan state through `plan`, `run`, `status`, `next`,
and `inspect`, and ship the slice as `0.35.0` with full release closure and
coverage above 95% for modified Rust files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and repository-managed docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential session-native planning plus decision execution where one bounded planning proposal is inferred at a time, explicitly confirmed before native execution, and optionally superseded by one bounded replan revision at a time  
**Observability Surface**: Persisted goal-plan/session state, decision-oriented traces under `.boundline/traces/`, CLI summaries on `plan`, `run`, `status`, `next`, and `inspect`, plus release docs and assistant guidance that explain proposal, confirmation, and replanning state  
**Performance Goals**: Operators should recover inferred flow, selected targets, verification strategy, and confirmation state from normal CLI output in under 2 minutes; maintainers should complete release validation for the slice in under 20 minutes  
**Constraints**: No new top-level runtime, no parallel planning or hidden background inference, no long-term memory system, no provider-abstraction refoundation, no Canon contract expansion, workflows remain guardrails rather than execution owners, and explicit compatibility follow-up remains subordinate and trace-authoritative  
**Scale/Scope**: One workspace or registered cluster at a time, bounded by existing session/run limits with one authoritative proposal or replan revision per active goal

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly changes how Boundline derives and controls bounded engineering work by making planning evidence-driven and operator-confirmed. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes planning, confirmation, replanning, bounded execution, and operator follow-through ahead of optimization or polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`; explicit compatibility follow-up remains available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: The design keeps one active proposal or replan revision at a time, preserves existing session/run limits, and requires explicit confirmation or explicit stop. See Technical Context, research, and contracts.
- **PASS** Stateful execution: Planning inference, proposal confirmation state, replan lineage, and workflow guardrails remain persisted in goal-plan/session and trace state. See Summary, data model, and contracts.
- **PASS** Mutable planning: The plan explicitly supports proposal supersession and bounded replanning when new evidence invalidates the prior shape. See Summary, research, and data model.
- **PASS** Sequential-first design: Planning and replanning remain one-step-at-a-time state transitions; no concurrency or hidden fan-out is introduced. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Planning inference, confirmation, code mutation, validation, and replanning stay visible as explicit planning/runtime actions instead of hidden heuristics. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Inference rationale, proposal state, guardrails, confirmation, revision lineage, and explicit stop reasons are surfaced through traces and existing CLI summaries. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: Canon remains bounded evidence or governance input only; the slice does not introduce councils, memory systems, UI work, or distributed execution. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is replacing keyword-only planning with evidence-driven proposal plus confirmation and bounded replanning on the existing native path. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/035-dynamic-planning-flow/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── dynamic-plan-proposal-contract.md
│   ├── bounded-replan-contract.md
│   └── workflow-guardrail-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── goal_plan.rs
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   ├── flow_inference.rs
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
│   ├── goal_plan_contract.rs
│   └── runtime_refoundation_contract.rs
├── integration/
│   ├── runtime_refoundation_flow.rs
│   └── session_native_flow.rs
└── unit/
    ├── flow_confirmation.rs
    ├── flow_inference.rs
    ├── goal_planner.rs
    ├── runtime_routing.rs
    ├── session_model.rs
    └── session_record.rs

README.md
ROADMAP.md
CHANGELOG.md
assistant/
docs/
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing goal-plan model,
planner, flow inference, session runtime, session/trace projections, CLI
renderers, and release surfaces. No new top-level runtime or product surface is
needed because the feature strengthens the session-owned planning path rather
than introducing a second planner.

## Complexity Tracking

No constitution violations are expected for this slice.
