# Implementation Plan: Authority-Zoned Delivery Councils

**Branch**: `056-authority-zoned-councils` | **Date**: 2026-05-15 | **Spec**: [specs/056-authority-zoned-councils/spec.md](specs/056-authority-zoned-councils/spec.md)
**Input**: Feature specification from `/specs/056-authority-zoned-councils/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend the existing Boundline governance and review surfaces so the runtime can
consume Canon `authority-governance-v1`, resolve one effective control class
and bounded council profile per governed stage boundary, persist findings and
producer responses, and project Canon provenance plus stop semantics through the
session-native operator surfaces. The first implementation slice stays inside
the current governance adapter, review, session, trace, and CLI projection
modules, treats optional Canon metadata as inspectable provenance rather than
runtime control, bumps Boundline from `0.55.0` to `0.56.0`, and closes with
focused docs, changelog updates, tests, clippy, formatting, and modified-file
coverage validation at 95% or higher.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, and Rust standard-library collections, filesystem, path, and process APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, persisted traces under `.boundline/traces/`, optional `.boundline/execution.json` and `.boundline/config.toml`, plus optional Canon-governed packet refs consumed through the existing governance runtime boundary  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit and integration tests for governance resolution and projection, `cargo test --no-run --all-targets`, focused package tests such as `cargo test -p boundline-core ...` and `cargo test -p boundline-adapters ...` where root `src/` modules are re-exported, `cargo nextest run --workspace --all-features` when feasible, and modified-file coverage validation at 95% or higher  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI and library workspace with persisted local runtime state  
**Execution Model**: Sequential session-native stage execution where one governed boundary is evaluated at a time, one effective control class and council profile are resolved explicitly, local governance remains available when Canon is absent and governance is not required, and every governed boundary stops in an explicit terminal or waiting state rather than polling or branching in the background  
**Observability Surface**: `.boundline/session.json`, review traces, governance trace records, and the `plan`, `run`, `status`, `next`, and `inspect` CLI surfaces that must show consumed Canon contract line, resolved control class, council profile, findings, producer responses, adjudication result, stop semantics, and any optional provenance-only Canon fields  
**Performance Goals**: Developers should be able to identify the current authority posture, council profile, and next action from normal runtime surfaces in under 2 minutes, and the added governance resolution must not materially degrade bounded stage execution responsiveness  
**Constraints**: Boundline consumes only Canon `authority-governance-v1` in the first slice; required control inputs are limited to `authority_zone`, `change_class`, `intended_persona`, `approval_state`, `packet_readiness`, and `risk`; optional Canon provenance fields remain inspectable only; the slice must preserve local governance fallback when Canon is optional, keep councils bounded and explicitly triggered, avoid distributed execution or always-on debate, and avoid panic-prone runtime logic outside tests  
**Scale/Scope**: One active workspace session at a time, one governed boundary resolved at a time, a small fixed council-profile set (`none`, `light_single`, `yellow_pair`, `red_five`, `restricted_manual`), and a bounded set of touched governance, review, session, trace, documentation, and release surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature improves bounded engineering delivery by turning implicit governance posture into explicit admission-control decisions at real stage boundaries. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice prioritizes governance resolution, bounded review control, persisted findings, and operator-visible stop behavior ahead of release polish. See Summary, Constraints, and Scale/Scope.
- **PASS** Primary workflow: The main path remains the session-native `goal -> plan -> run -> status -> next -> inspect` workflow, with explicit local-governance compatibility when Canon is optional and unavailable. See Execution Model and Constraints.
- **PASS** Bounded execution: Governed evaluation starts at a stage boundary, resolves one explicit profile, and ends in explicit proceed, waiting, or stop states with no hidden background loops. See Execution Model and Scale/Scope.
- **PASS** Stateful execution: Council and governance outcomes are persisted in existing session and trace surfaces so later commands read the same explicit state rather than recomputing hidden decisions. See Storage and Observability Surface.
- **PASS** Mutable planning: Planning remains mutable because accepted findings can create remediation work and plan updates while preserving the same bounded session story. See Summary and Observability Surface.
- **PASS** Sequential-first design: The slice keeps one governed boundary active at a time and does not introduce parallel reviewer execution as a hidden runtime engine. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Canon inputs, local review policy, control-class resolution, and council outcomes remain explicit typed state and trace records rather than opaque heuristics. See Constraints and Observability Surface.
- **PASS** Observability and explicit intelligence: The runtime must project consumed contract line, control class, council profile, findings, producer responses, adjudication, and stop semantics through session-visible surfaces. See Observability Surface.
- **PASS** Catalog currency: The spec already records a 2026-05-15 provider-doc audit with a no-change result for `assistant/catalog/model-catalog.toml`; this plan carries that evidence forward and requires it in tasks and validation. See Summary and Technical Context.
- **PASS** Non-goals and external separation: The plan uses Canon only as a bounded governance input and preserves local governance fallback, avoiding external orchestration dependence, distributed execution, UI work, long-term memory, and unbounded provider abstraction. See Constraints and Summary.
- **PASS** Minimal slice: The smallest independently valuable capability is one explicit, inspectable authority-to-council resolution path that can stop unsafe work and surface the reason immediately. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/056-authority-zoned-councils/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── authority-governance-consumer-contract.md
│   └── council-projection-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── governance_runtime.rs
├── cli/
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── governance.rs
│   ├── review.rs
│   └── session.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── governance.rs
│   └── review_trace.rs
└── lib.rs

crates/
├── boundline-core/
│   └── src/
│       └── domain.rs
└── boundline-adapters/
    └── src/
        └── lib.rs

tests/
├── contract/
├── integration/
└── unit/

docs/
├── authority-zones-and-stop-semantics.md
├── council-adoption-guide.md
└── review-council-algorithms.md

assistant/
└── catalog/
    └── model-catalog.toml

README.md
CHANGELOG.md
Cargo.toml
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing governance adapter,
review-domain, session, trace, and CLI projection surfaces. This strengthens an
already-bounded delivery path and preserves the local-first runtime model, so
no new top-level runtime, service, or distributed review subsystem is
justified. Any new shared domain type added under `src/domain/` must also be
registered through `crates/boundline-core/src/domain.rs` so the member crates
continue to compile path-based imports.

## Complexity Tracking

No constitution violations are expected for this slice.
