# Implementation Plan: Runtime Intelligence Substrate

**Branch**: `052-runtime-intelligence-substrate` | **Date**: 2026-05-14 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/spec.md](/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/spec.md)
**Input**: Feature specification from `/specs/052-runtime-intelligence-substrate/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Turn the existing context-pack and planning groundwork into an explicit local
runtime substrate that can build deterministic context packs, persist runtime
indexes and credibility state, project the result through session and trace
surfaces, and consume Canon artifacts only as optional enrichment. The first
implementation slice stays inside existing goal-planning and session-runtime
surfaces, bumps Boundline from `0.51.1` to `0.52.0`, and closes with clippy,
formatting, focused tests, and coverage on modified files.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, collections, and synchronization-free runtime primitives; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/` session and runtime state, persisted traces under `.boundline/traces/`, existing goal-plan state, repo-visible project context such as `project.boundline.toml`, local repository files, and optional `.canon/` or repo-visible Canon artifacts as enrichment only  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit and integration tests for `goal_planner`, `session_runtime`, and CLI projection, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and file-scoped coverage validation for modified files  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI and library workspace with workspace-local persisted execution state  
**Execution Model**: Sequential session-native flow where substrate construction completes before planning starts or continues, yields explicit credibility state, and never introduces background workers, hidden retrieval loops, or parallel branch execution  
**Observability Surface**: Goal-plan state, session runtime state, trace projection, and `status`, `next`, `inspect`, and related CLI summaries that must distinguish local repository context from optional Canon enrichment  
**Performance Goals**: Operators should be able to identify why a context pack was built or rejected from standard runtime surfaces in under 5 minutes, and substrate construction must not materially degrade current bounded planning responsiveness  
**Constraints**: Canon enrichment must remain optional; no councils, voting, adaptive governance, provider-routing expansion, distributed execution, or long-term memory systems; no panic-based runtime error handling outside tests; preserve sequential-first bounded delivery semantics  
**Scale/Scope**: One active workspace or registered cluster at a time, one active substrate build per planning step, and one inspectable context pack per bounded delivery attempt

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded engineering delivery by making planning depend on an explicit, inspectable context substrate instead of ambient repository assumptions. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice prioritizes context credibility, stop behavior, runtime state, and inspectability before optimization or polish. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`, with Canon enrichment available only as an explicit compatibility input. See Summary, Execution Model, and Constraints.
- **PASS** Bounded execution: Substrate construction runs as a bounded sequential step, yields explicit credible or non-credible outcomes, and stops or replans before execution when context is not credible. See Execution Model, Performance Goals, and Constraints.
- **PASS** Stateful execution: Runtime indexes, context-pack credibility, and traceable provenance are written into existing Boundline runtime and plan surfaces rather than recomputed opaquely. See Storage, Observability Surface, and Summary.
- **PASS** Mutable planning: The substrate feeds existing planning and replanning behavior, allowing step insertion or replacement when context changes without introducing a second planner. See Summary and Execution Model.
- **PASS** Sequential-first design: One substrate path remains active at a time and no background worker or hidden parallel retrieval loop is introduced. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Context selection, Canon enrichment, failure signals, and terminal conditions remain explicit in runtime artifacts and CLI summaries rather than hidden in adapters. See Observability Surface and Constraints.
- **PASS** Observability and explicit intelligence: Trace projection and session-native surfaces must explain selected inputs, missing evidence, credibility outcome, and Canon-versus-local provenance. See Observability Surface and Summary.
- **PASS** Catalog currency: Current OpenAI, Anthropic, and Google provider model docs were checked in the feature spec and produced a no-change result for `assistant/catalog/model-catalog.toml`. See the Catalog Research & Currency section in the spec.
- **PASS** Non-goals and external separation: The slice does not require Canon control flow and does not introduce councils, voting, adaptive governance, UI work, deployment pipelines, or long-term memory. See Constraints and Summary.
- **PASS** Minimal slice: The smallest independently valuable capability is one deterministic, inspectable context substrate that blocks non-credible planning. See Summary.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/052-runtime-intelligence-substrate/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── runtime-index-contract.md
│   └── substrate-trace-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── output.rs
│   ├── session.rs
│   └── workflow.rs
├── domain/
│   ├── goal_plan.rs
│   ├── project_memory.rs
│   └── session.rs
├── orchestrator/
│   ├── flow_inference.rs
│   ├── goal_planner.rs
│   ├── governance.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
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

**Structure Decision**: Keep the slice inside the existing Boundline planning,
session runtime, CLI projection, and runtime-state surfaces. The work is an
extension of already-present `ContextPack` and planning behavior in
`src/domain/goal_plan.rs` and `src/orchestrator/goal_planner.rs`, plus session
and CLI projection updates. No new top-level runtime or storage layer is
justified because the feature strengthens the current bounded orchestration
path rather than creating a second one.

## Complexity Tracking

No constitution violations are expected for this slice.
