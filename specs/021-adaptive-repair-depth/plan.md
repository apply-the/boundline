# Implementation Plan: Adaptive Repair Depth

**Branch**: `021-adaptive-repair-depth` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/synod/specs/021-adaptive-repair-depth/spec.md](/Users/rt/workspace/synod/specs/021-adaptive-repair-depth/spec.md)
**Input**: Feature specification from `/specs/021-adaptive-repair-depth/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Deepen Synod's bounded adaptive delivery story by letting adaptive replans use validation failure evidence to choose a more credible next repair candidate while preserving explicit workspace-slice, selection-headline, and attempt-lineage evidence across `run`, `status`, `next`, and `inspect`. The slice stays intentionally narrow: adaptive execution remains an explicit compatibility path backed by `.synod/execution.json`, the heuristics stay local and bounded, and the release closes as `0.21.0` with updated docs, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.synod/execution.json`, `.synod/session.json`, `.synod/traces/`, and release-aligned repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted `cargo test` suites for adaptive unit, integration, and contract coverage, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, plus repository-standard `cargo nextest run --workspace --all-features` and `cargo deny check licenses advisories bans sources` during release closeout  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state  
**Execution Model**: Sequential explicit compatibility execution inside the broader session runtime, with one adaptive attempt active at a time, bounded replanning after validation failure, and explicit route reporting when workflows, review, or governance also exist in the workspace  
**Observability Surface**: Persisted task context state, execution traces, `run`, `status`, `next`, and `inspect`, adaptive workspace-slice summaries, selection headlines, attempt lineage, validation records, and explicit routing plus execution-condition output  
**Performance Goals**: Representative adaptive replans should choose a materially different bounded candidate within one normal CLI round-trip after validation failure; developers should identify the latest adaptive selection reason and route state from CLI output in under 2 minutes  
**Constraints**: Adaptive execution remains an explicit compatibility path for this slice; no hidden background exploration, no unbounded candidate search, no workflow-owned or Canon-owned adaptive control flow, no new top-level runtime surface, and crate version must bump to `0.21.0` with docs and changelog updates  
**Scale/Scope**: One active adaptive run, one selected workspace slice at a time, bounded `read_targets`, validation-guided candidate ranking within local repository context, and release updates limited to touched runtime, docs, and test surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: Explain how this feature directly improves bounded engineering task delivery.
- Delivery-first scope: Confirm execution, orchestration, decomposition, or validation work is prioritized ahead of optimization or polish.
- Primary workflow: State whether the main operator path is session-native (`start -> capture -> plan -> run -> status -> next -> inspect`) and identify any explicit compatibility path that remains available.
- Bounded execution: Identify explicit start conditions, terminal conditions, and max step or retry limits.
- Stateful execution: Describe shared task context, read and write points, and justify any stateless segment.
- Mutable planning: Describe initial planning plus replanning, step insertion, or replacement behavior.
- Sequential-first design: Confirm one-step-at-a-time execution or justify why the constitution allows an exception.
- Tool-agent symmetry: Show how reasoning and action remain explicit in the execution model.
- Observability and explicit intelligence: List trace surfaces, visible decisions, failure signals, and any heuristic behavior that must be exposed.
- Non-goals and external separation: Confirm the plan does not depend on Canon behavior beyond bounded governance/evidence boundaries or reintroduce deferred scope such as councils or voting outside an explicitly reprioritized, bounded review slice, provider abstraction complexity beyond the approved slice, long-term memory, UI/UX, or deployment pipelines.
- Minimal slice: Explain the smallest independently valuable capability delivered by this plan.

- **PASS** Delivery identity: The slice directly improves bounded delivery execution by choosing better adaptive repair candidates after validation failure instead of widening Synod into a generic agent system. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/synod/specs/021-adaptive-repair-depth/spec.md).
- **PASS** Delivery-first scope: The work prioritizes adaptive execution quality, explicit replanning, and inspectability before polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native; adaptive repair stays an explicit compatibility path and the plan keeps that relationship visible when workflows, review, or governance also exist. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Adaptive replans still respect configured selected-target, generated-attempt, retry, and replan limits, and terminate explicitly when no credible candidate remains. See Technical Context, data model, and quickstart.
- **PASS** Stateful execution: Validation guidance, selected workspace slice, selection headline, attempt lineage, and validation record remain persisted in task context, session summaries, and traces. See Summary, data model, and contracts.
- **PASS** Mutable planning: The slice deepens existing replan behavior by making candidate ranking responsive to validation evidence while keeping plan mutation explicit and traceable. See Summary, research, and data model.
- **PASS** Sequential-first design: One adaptive attempt remains active at a time with explicit validation, replanning, and terminal stop conditions. See Technical Context, research, and [spec.md](/Users/rt/workspace/synod/specs/021-adaptive-repair-depth/spec.md).
- **PASS** Tool-agent symmetry: Adaptive analysis, code mutation, validation, and replanning remain visible as explicit steps rather than hidden heuristics. See Summary, research, quickstart, and contracts.
- **PASS** Observability and explicit intelligence: Validation-guided selection reasons, workspace slices, attempt lineage, routing, and terminal outcomes remain visible in CLI and trace surfaces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The plan does not move adaptive control to workflows or Canon, does not add provider abstraction or long-term memory, and keeps UI and deployment work out of scope. See Constraints, research, and [spec.md](/Users/rt/workspace/synod/specs/021-adaptive-repair-depth/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is validation-guided adaptive candidate selection on the existing compatibility path with explicit route and inspection guidance. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/021-adaptive-repair-depth/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── adaptive-command-surface-contract.md
│   ├── adaptive-selection-evidence-contract.md
│   └── adaptive-route-guidance-contract.md
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
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── execution.rs
│   └── session.rs
├── fixture.rs
└── lib.rs

tests/
├── contract/
├── integration/
├── support/
└── unit/

assistant/
└── README.md

docs/
├── adaptive-execution.md
├── configuration.md
└── getting-started.md
```

**Structure Decision**: Keep the slice inside the existing adaptive execution domain, fixture planner, CLI projection, docs, and test-fixture surfaces. No new top-level runtime or workflow surface is justified because the feature deepens candidate ranking and observability on the existing compatibility path rather than introducing a new execution mode.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
