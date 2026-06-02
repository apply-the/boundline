# Implementation Plan: Control Graduation And Adaptive Governance

**Branch**: `057-adaptive-governance` | **Date**: 2026-05-16 | **Spec**: [specs/057-adaptive-governance/spec.md](specs/057-adaptive-governance/spec.md)
**Input**: Feature specification from `/specs/057-adaptive-governance/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend the existing Boundline governance, session, trace, and CLI surfaces so
the runtime can compute one explicit governance state and rollout profile per
governed boundary, using Canon `authority-governance-v1` as the required
semantic baseline and an optional `adaptive-governance-v1` companion contract
when present. The first implementation slice keeps confidence, trust,
degradation, escalation, override handling, and stop behavior runtime-owned in
Boundline, reuses the existing session-native workflow, and projects the full
adaptive-governance rationale through `plan`, `run`, `status`, `next`, and
`inspect` without introducing a second orchestration subsystem. Promotion from
advisory, rollout-profile changes, and resumed automation remain operator-
approved and traceable, while Canon-owned approval, readiness, lineage, and
promotion metadata stay distinct from Boundline runtime decisions.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, and Rust standard-library collections, filesystem, path, and process APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, persisted traces under `.boundline/traces/`, optional `.boundline/execution.json` and `.boundline/config.toml`, plus Canon-governed packet metadata already consumed through the governance runtime boundary  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit and integration tests for governance state resolution and projection, `cargo test --no-run --all-targets`, focused package tests such as `cargo test -p boundline-core ...`, `cargo test -p boundline-adapters ...`, and `cargo test -p boundline-cli ...`, `cargo nextest run --workspace --all-features` when feasible, and modified-file coverage validation at 95% or higher  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI and library workspace with persisted local runtime state  
**Execution Model**: Sequential session-native stage execution where one governed boundary is evaluated at a time, one runtime governance state and rollout profile are resolved explicitly, and every boundary ends in an explicit continue, degrade, escalate, wait, or stop outcome rather than hidden background processing  
**Observability Surface**: `.boundline/session.json`, governance and review traces under `.boundline/traces/`, and the `plan`, `run`, `status`, `next`, and `inspect` CLI surfaces that must show consumed Canon contract lines, runtime governance state, rollout profile, confidence rationale, trust posture, degradation or escalation outcome, and next required action  
**Performance Goals**: Operators should be able to identify the current governance state, rollout profile, confidence rationale, and next action from normal runtime surfaces in under 2 minutes, and the added governance logic must not materially degrade bounded stage execution responsiveness  
**Constraints**: `authority-governance-v1` remains the required Canon posture baseline; any `adaptive-governance-v1` companion contract stays optional and semantic; Boundline remains the runtime owner of confidence, trust, degradation, escalation, councils, and stop transitions; stronger governance recommendations, rollout-profile changes, and resumed automation require explicit operator approval plus surfaced rationale; Canon-owned approval semantics, readiness semantics, project memory, lineage, and promotion state stay distinct from local runtime control; the slice must preserve explicit local compatibility behavior when Canon is optional; no hidden concurrency, no distributed governance service, and no panic-prone runtime logic outside tests  
**Scale/Scope**: One active workspace session at a time, one governed boundary resolved at a time, four runtime governance states (`advisory`, `catch`, `rule`, `hook`), four rollout profiles (`minimal`, `guided`, `governed`, `strict`), and a bounded set of touched governance, session, trace, documentation, and CLI surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice improves bounded engineering delivery by making governance progression, degradation, and escalation explicit at real stage boundaries instead of leaving them implicit. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes runtime execution behavior, state handling, failure control, and inspectability ahead of polish or speculative expansion. See Summary, Constraints, and Scale/Scope.
- **PASS** Primary workflow: The main operator path remains the session-native `goal -> plan -> run -> status -> next -> inspect` workflow, with explicit local compatibility behavior when Canon semantics are absent and governance is not required. See Execution Model and Constraints.
- **PASS** Bounded execution: Governed evaluation starts at a stage boundary and ends in explicit continue, degrade, escalate, wait, or stop states with no hidden looping or background governance worker. See Execution Model and Scale/Scope.
- **PASS** Stateful execution: Governance-state, confidence, trust, degradation, escalation, and override outcomes are persisted in existing session and trace surfaces so later commands reuse the same explicit state. See Storage and Observability Surface.
- **PASS** Mutable planning: Degradation, escalation, and accepted follow-up obligations can create explicit remediation or human-gate next steps while preserving the same bounded session plan. See Summary and Observability Surface.
- **PASS** Sequential-first design: The slice keeps one governed boundary active at a time and does not introduce parallel councils, hidden branches, or background governance daemons. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Canon input, local evidence, runtime confidence, governance transitions, and next actions remain explicit typed state and trace records rather than opaque heuristics. See Constraints and Observability Surface.
- **PASS** Observability and explicit intelligence: The runtime must project contract consumption, governance-state changes, rollout-profile changes, confidence rationale, degradation, escalation, and next action through the same operator-visible surfaces. See Observability Surface.
- **PASS** Catalog currency: The spec records a 2026-05-16 provider-doc audit with a no-change result for `assistant/catalog/model-catalog.toml`; this plan carries that evidence forward into research and validation. See Summary and Technical Context.
- **PASS** Non-goals and external separation: Canon remains a bounded semantic input only; the plan keeps runtime control in Boundline and does not reintroduce distributed governance, provider-abstraction expansion, UI work, or deployment pipelines. See Constraints and Summary.
- **PASS** Minimal slice: The smallest independently valuable capability is one explicit, inspectable adaptive-governance path that begins in advisory mode and can degrade or escalate without silent weakening. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/057-adaptive-governance/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── adaptive-governance-consumer-contract.md
│   └── adaptive-governance-projection-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── governance_runtime.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── governance.rs
│   ├── review.rs
│   └── session.rs
├── fixture.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── governance.rs
│   ├── session_runtime.rs
│   └── review_trace.rs
└── lib.rs

crates/
├── boundline-core/
│   └── src/
│       └── domain.rs
├── boundline-adapters/
│   └── src/
│       └── lib.rs
└── boundline-cli/
    └── src/
        └── lib.rs

tests/
├── contract/
├── integration/
└── unit/

tech-docs/
├── adaptive-governance.md
├── control-graduation-model.md
├── degradation-and-escalation.md
└── runtime-confidence-and-calibration.md

assistant/
└── catalog/
    └── model-catalog.toml

README.md
CHANGELOG.md
AGENTS.md
Cargo.toml
```

**Structure Decision**: Keep the slice inside the existing governance adapter,
domain, session, trace, and CLI projection surfaces. This strengthens an
already-bounded delivery path and preserves the local-first runtime model, so
no new top-level runtime, background service, or distributed governance system
is justified. Any new shared domain type added under `src/domain/` must also be
registered through `crates/boundline-core/src/domain.rs` so the member crates
continue to compile path-based imports.

## Complexity Tracking

No constitution violations are expected for this slice.
