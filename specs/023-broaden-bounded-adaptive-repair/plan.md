# Implementation Plan: Broaden Bounded Adaptive Repair

**Branch**: `023-broaden-bounded-adaptive-repair` | **Date**: 2026-05-01 | **Spec**: [specs/023-broaden-bounded-adaptive-repair/spec.md](specs/023-broaden-bounded-adaptive-repair/spec.md)
**Input**: Feature specification from `/specs/023-broaden-bounded-adaptive-repair/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Broaden Boundline's bounded adaptive compatibility path by adding richer deterministic mutation families, clearer candidate credibility and exhaustion semantics, and aligned read-side follow-up across `run`, `status`, `next`, and `inspect`. The slice remains intentionally bounded: adaptive execution stays manifest-backed through `.boundline/execution.json`, candidate synthesis stays deterministic and workspace-bounded, and the release closes as `0.23.0` with updated docs, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/execution.json`, `.boundline/session.json`, `.boundline/traces/`, and release-aligned repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit, integration, and contract coverage for deeper adaptive mutation families and summary projection, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, plus repository-standard `cargo nextest run --workspace --all-features` and `cargo deny check licenses advisories bans sources` during release closeout  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed execution, session, and trace state  
**Execution Model**: Sequential explicit compatibility execution inside the broader session-native runtime, with one adaptive attempt active at a time, bounded replanning after failed validation, and continuity-aware read-side follow-up when no resumable session exists  
**Observability Surface**: Persisted task context state, execution traces, `run`, `status`, `next`, `inspect`, adaptive selection headlines, workspace-slice summaries, candidate credibility and exhaustion reasons, validation records, and explicit routing plus execution-condition output  
**Performance Goals**: Representative adaptive compatibility failures that are not solvable by the current three heuristic families should produce a materially different bounded second candidate within one normal CLI replan round-trip; developers should identify the latest adaptive selection reason and exhaustion rationale from CLI output in under 2 minutes  
**Constraints**: Adaptive repair remains on the explicit compatibility path for this slice; no open-ended repository exploration, no hidden background retries, no Canon-owned adaptive planning, no workflow-owned orchestration, no new top-level runtime surface, and crate version must bump to `0.23.0` with docs and changelog updates  
**Scale/Scope**: One active adaptive run, one selected workspace slice at a time, bounded `read_targets`, bounded built-in mutation families, validation-guided candidate ranking within local repository context, and release updates limited to touched runtime, docs, and test surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: Explain how this feature directly improves bounded engineering task delivery.
- Delivery-first scope: Confirm execution, orchestration, decomposition, or validation work is prioritized ahead of optimization or polish.
- Primary workflow: State whether the main operator path is session-native (`goal -> plan -> run -> status -> next -> inspect`) and identify any explicit compatibility path that remains available.
- Bounded execution: Identify explicit start conditions, terminal conditions, and max step or retry limits.
- Stateful execution: Describe shared task context, read and write points, and justify any stateless segment.
- Mutable planning: Describe initial planning plus replanning, step insertion, or replacement behavior.
- Sequential-first design: Confirm one-step-at-a-time execution or justify why the constitution allows an exception.
- Tool-agent symmetry: Show how reasoning and action remain explicit in the execution model.
- Observability and explicit intelligence: List trace surfaces, visible decisions, failure signals, and any heuristic behavior that must be exposed.
- Non-goals and external separation: Confirm the plan does not depend on Canon behavior beyond bounded governance/evidence boundaries or reintroduce deferred scope such as councils or voting outside an explicitly reprioritized, bounded review slice, provider abstraction complexity beyond the approved slice, long-term memory, UI/UX, or deployment pipelines.
- Minimal slice: Explain the smallest independently valuable capability delivered by this plan.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

- **PASS** Delivery identity: The slice directly improves bounded code-delivery execution by making adaptive compatibility runs capable of trying more credible bounded repairs before they exhaust. See Summary, Technical Context, and [spec.md](specs/023-broaden-bounded-adaptive-repair/spec.md).
- **PASS** Delivery-first scope: The work prioritizes execution quality, replanning credibility, and explicit failure handling ahead of polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: Session-native remains the main operator path, while deeper adaptive repair stays on the explicit compatibility route and reuses the continuity-aware read-side story from `0.22.0`. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Adaptive runs keep explicit start conditions, explicit terminal conditions, and configured attempt, retry, and replan limits; no hidden background control flow is introduced. See Technical Context, data model, and quickstart.
- **PASS** Stateful execution: Candidate signatures, validation guidance, credibility evidence, workspace slice, and attempt lineage remain persisted in task context, sessions, and traces. See Summary, Technical Context, data model, and contracts.
- **PASS** Mutable planning: The slice deepens the existing adaptive replan behavior by allowing richer bounded candidate families and clearer candidate rejection semantics while keeping plan mutation inspectable. See Summary, research, and data model.
- **PASS** Sequential-first design: One adaptive attempt remains active at a time, with explicit validation, replanning, and terminal stop conditions. See Technical Context, research, and [spec.md](specs/023-broaden-bounded-adaptive-repair/spec.md).
- **PASS** Tool-agent symmetry: Adaptive analysis, candidate synthesis, validation, and replanning remain visible as explicit steps rather than hidden heuristics. See Summary, research, quickstart, and contracts.
- **PASS** Observability and explicit intelligence: Selection reasons, rejected candidates, workspace slices, exhaustion rationale, routing ownership, and terminal outcomes remain visible in CLI and trace surfaces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The plan does not move adaptive control to workflows or Canon, does not add provider abstraction or long-term memory, and keeps UI and deployment work out of scope. See Constraints, research, and [spec.md](specs/023-broaden-bounded-adaptive-repair/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is richer bounded adaptive candidate generation plus explicit credibility and exhaustion reporting on the existing compatibility path. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/023-broaden-bounded-adaptive-repair/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── adaptive-change-kind-contract.md
│   ├── adaptive-credibility-summary-contract.md
│   └── adaptive-exhaustion-follow-up-contract.md
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
│   ├── run.rs
│   └── session.rs
├── domain/
│   └── execution.rs
├── fixture.rs
├── orchestrator/
│   └── terminal.rs
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

README.md
CONTRIBUTING.md
ROADMAP.md
CHANGELOG.md
```

**Structure Decision**: Keep the slice inside the existing adaptive execution domain, fixture planner, CLI summary, docs, and test harness surfaces. No new top-level runtime or persistence area is justified because the feature deepens bounded candidate synthesis and read-side observability on the existing compatibility path rather than introducing a new execution mode.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
