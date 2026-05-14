# Implementation Plan: Expert Pack Selection

**Branch**: `053-expert-pack-selection` | **Date**: 2026-05-14 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/053-expert-pack-selection/spec.md](/Users/rt/workspace/apply-the/boundline/specs/053-expert-pack-selection/spec.md)
**Input**: Feature specification from `/specs/053-expert-pack-selection/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a built-in expert-pack catalog and deterministic selection outcome to the
existing domain-template and goal-planning surfaces so Boundline can recommend
runtime roles before planning, preserve selected and rejected provenance in the
goal plan, and project the result through session-native inspection surfaces.
The first implementation slice stays inside existing domain, configuration,
goal-planner, and CLI projection surfaces, bumps Boundline from `0.52.0` to
`0.53.0`, and closes with focused tests, clippy, formatting, and 95% coverage
on all modified files.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `serde`, `serde_json`, `thiserror`, `toml`, `uuid`, and Rust standard-library collections, filesystem, and path types; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/` config and session state, persisted goal-plan data, local and cluster routing config stores, local repository cues, and optional Canon-governed expertise artifacts discovered through compatible publication and lineage surfaces  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit and integration tests for domain selection and session projection, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features` when feasible, and modified-file coverage validation at 95% or higher  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI and library workspace with persisted local runtime state  
**Execution Model**: Sequential session-native planning where expert-pack selection runs once for the bounded workspace or target before planning continues, emits an explicit selected or none-selected outcome, and never introduces background workers or hidden retrieval loops  
**Observability Surface**: Goal-plan state, trace payloads, and `status`, `next`, `inspect`, and existing config or session summaries that must distinguish local inputs from optional Canon expertise inputs  
**Performance Goals**: Maintainers should be able to identify why an expert pack or runtime role was selected or rejected from normal runtime surfaces in under 5 minutes, and selection must not materially degrade bounded planning responsiveness  
**Constraints**: Canon input remains optional; no external expert-pack installation, no councils or voting, no provider-routing expansion, no distributed execution, no long-term memory, and no panic-based runtime error handling outside tests  
**Scale/Scope**: One active workspace or bounded target at a time, one deterministic expert-pack selection outcome per planning attempt, and a small built-in catalog tied to current domain-template and reviewer-role capabilities

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded engineering delivery by making runtime role selection explicit before planning instead of leaving expertise implicit. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice prioritizes built-in catalog definition, deterministic selection, rejection handling, and inspectability before any future pack distribution or polish. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains session-native `start -> capture -> plan -> run -> status -> next -> inspect`, with Canon expertise input available only as an explicit optional enrichment path. See Execution Model and Constraints.
- **PASS** Bounded execution: Expert-pack selection starts when planning has a bounded workspace or target, ends with explicit selected or none-selected state, and does not add unbounded retries or hidden loops. See Execution Model and Scale/Scope.
- **PASS** Stateful execution: Selection outcome, provenance, and rejection reasons are persisted in existing goal-plan and trace surfaces rather than recomputed opaquely for every read-side command. See Storage and Observability Surface.
- **PASS** Mutable planning: Selection may be recomputed when the bounded target, effective config, or supporting context changes, but the active planning path reuses the persisted outcome until a bounded replan occurs. See Summary and Execution Model.
- **PASS** Sequential-first design: One expert-pack selection path remains active per bounded planning step and no parallel role-selection engine is introduced. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Domain cues, reviewer-role routes, Canon support, and rejection reasons remain explicit in state and CLI projection instead of hidden inside routing heuristics. See Observability Surface and Constraints.
- **PASS** Observability and explicit intelligence: Session and trace surfaces must expose selected packs, rejected candidates, recommended runtime roles, and Canon-versus-local provenance. See Observability Surface and Summary.
- **PASS** Catalog currency: Current OpenAI, Anthropic, and Google provider model docs were checked in the feature spec and produced a no-change result for `assistant/catalog/model-catalog.toml`. See the Catalog Research & Currency section in the spec.
- **PASS** Non-goals and external separation: The plan does not depend on Canon runtime control flow and does not reintroduce deferred scope such as external pack installation, councils, voting, UI work, deployment pipelines, or long-term memory. See Constraints and Summary.
- **PASS** Minimal slice: The smallest independently valuable capability is one built-in expert-pack catalog plus one deterministic, inspectable selection outcome that planning can trust. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/053-expert-pack-selection/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── expert-pack-selection-contract.md
│   └── expert-selection-trace-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── config.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── domain_templates.rs
│   └── goal_plan.rs
├── orchestrator/
│   └── goal_planner.rs
└── lib.rs

tests/
├── integration/
└── unit/

docs/
├── architecture.md
└── configuration.md

assistant/
└── catalog/
    └── model-catalog.toml

README.md
CHANGELOG.md
Cargo.toml
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing domain-template,
effective-configuration, goal-planning, and session-projection surfaces. The
feature strengthens the current bounded orchestration path by adding explicit
expert-pack semantics and trace projection, so no new top-level runtime, pack
registry, or UI surface is justified.

## Complexity Tracking

No constitution violations are expected for this slice.
