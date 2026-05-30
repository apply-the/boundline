# Implementation Plan: Guidance And Guardian Capabilities

**Branch**: `054-guidance-guardian-capabilities` | **Date**: 2026-05-14 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/054-guidance-guardian-capabilities/spec.md](/Users/rt/workspace/apply-the/boundline/specs/054-guidance-guardian-capabilities/spec.md)
**Input**: Feature specification from `/specs/054-guidance-guardian-capabilities/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a typed guidance-and-guardian capability layer to the existing session-native
runtime so Boundline can resolve engineering guidance before planning and step
execution, run deterministic or routed semantic guardians after bounded work,
emit structured findings, and project capability provenance through existing
trace, status, next, and inspect surfaces. The first implementation slice stays
inside current goal-planning, session-runtime, configuration, and CLI projection
surfaces, introduces repository-managed built-in capability assets plus
workspace-local overrides, bumps Boundline from `0.53.0` to `0.54.0`, updates
roadmap and operator docs, documents the touched code, and closes with focused
tests, clippy-clean output, formatting, and 95% coverage on all modified or new
Rust files.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, and Rust standard-library collections, filesystem, path, and process APIs; no new runtime dependencies planned for this slice  
**Storage**: Repository-managed built-in guidance and guardian assets under `assistant/`, workspace-local overrides under `.boundline/guidance/` and `.boundline/guardians/`, existing workspace-local `.boundline/session.json` and `.boundline/traces/`, and optional Canon-governed repo-visible standards discovered through existing project-memory and governed-artifact surfaces  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests for resolution, findings, routing degradation, and trace projection, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features` when feasible, and modified-file coverage validation at 95% or higher  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI and library workspace with persisted local runtime state and repo-managed assistant assets  
**Execution Model**: Sequential session-native planning and execution where guidance resolution runs before bounded planning or step execution, guardian evaluation runs after bounded work within the active lifecycle phase, and each guardian invocation ends with a finding, degraded outcome, or explicit failure without background workers or hidden retries  
**Observability Surface**: Persisted goal-plan state, execution traces, session summaries, and `status`, `next`, and `inspect` projection surfaces that must expose loaded sources, skipped sources, authority ranking, guardian timelines, degraded routing outcomes, and structured findings  
**Performance Goals**: Maintainers should be able to identify why guidance or guardian sources loaded, skipped, blocked, or degraded from normal runtime surfaces in under 5 minutes, and capability resolution plus guardian projection must not materially degrade a normal planning or step-completion CLI round-trip  
**Constraints**: Canon input remains optional; the feature must use existing runtime routing for semantic guardian invocations; no new routing slots, no councils or voting, no provider-catalog management as feature scope, no distributed execution, no hidden background work, no panic-based runtime errors outside tests, and task 1 plus final validation must include the Boundline version bump, roadmap/docs refresh, code documentation, clippy fixes, and 95% modified-file coverage  
**Scale/Scope**: One active workspace or bounded target at a time, one resolved guidance set per relevant lifecycle phase, a bounded set of guardian evaluations per phase, and an initial built-in catalog limited to core clean-code, language, framework, testing, and architecture guidance surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded engineering delivery by turning engineering standards into resolved planning inputs and explicit post-step verification outputs instead of passive documents. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice prioritizes capability resolution, guardian execution, findings, and trace projection ahead of docs polish or broader pack distribution. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`, with optional Canon standards only as bounded supporting inputs rather than a parallel runtime. See Execution Model and Constraints.
- **PASS** Bounded execution: Guidance resolution runs at bounded planning or phase-entry points, guardian execution runs with explicit per-phase limits and timeout boundaries, and every invocation ends with a success, degraded, skipped, or failure outcome. See Execution Model, Constraints, and Scale/Scope.
- **PASS** Stateful execution: Resolved guidance, skipped sources, finding summaries, and degraded outcomes are persisted in existing goal-plan and trace surfaces rather than recomputed opaquely on every read-side command. See Storage and Observability Surface.
- **PASS** Mutable planning: Planning can reuse or recompute resolved guidance when the bounded target, available sources, or effective routing changes, while the active session continues to project the persisted outcome until an explicit bounded replan occurs. See Summary and Execution Model.
- **PASS** Sequential-first design: One guidance-resolution path and one ordered guardian sequence remain active per bounded phase, with no background evaluation engine or parallel swarm. See Execution Model and Scale/Scope.
- **PASS** Tool-agent symmetry: Deterministic tools, routed semantic guardians, loaded source authority, and failure or degradation states remain explicit in persisted state and CLI projection rather than hidden inside assistant heuristics. See Observability Surface and Constraints.
- **PASS** Observability and explicit intelligence: Session and trace surfaces must expose loaded and skipped guidance sources, authority ranking, guardian order, structured findings, and route-unavailable degradation outcomes. See Observability Surface and Summary.
- **PASS** Catalog currency: Current OpenAI, Anthropic, and Google provider docs were checked during feature preparation on 2026-05-14 and produced a no-change audit recorded in `research.md`; the feature itself still keeps model catalog management out of scope and reuses existing runtime routing. See `research.md` and Constraints.
- **PASS** Non-goals and external separation: The plan does not depend on Canon runtime control flow and does not reintroduce councils, voting, new routing slots, distributed execution, long-term memory, UI work, or deployment pipelines. See Constraints and Summary.
- **PASS** Minimal slice: The smallest independently valuable capability is one typed capability model plus one bounded resolution-and-finding path that planning and session execution can trust and inspect. See Summary.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/054-guidance-guardian-capabilities/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── capability-manifest-contract.md
│   ├── guardian-finding-contract.md
│   └── guidance-guardian-trace-contract.md
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
assistant/
├── guidance/
├── guardians/
└── packs/

src/
├── cli/
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── domain_templates.rs
│   ├── goal_plan.rs
│   ├── guidance.rs
│   └── trace.rs
├── orchestrator/
│   ├── goal_planner.rs
│   ├── guidance_runtime.rs
│   └── session_runtime.rs
├── domain.rs
├── orchestrator.rs
└── lib.rs

tests/
├── contract/
├── integration/
└── unit/

docs/
├── architecture.md
├── configuration.md
├── getting-started.md
└── guides/

README.md
CHANGELOG.md
Cargo.toml
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing domain,
goal-planning, session-runtime, trace, and CLI projection surfaces. Introduce
one small typed domain model for capability manifests and findings plus one
orchestrator helper for discovery, resolution, and guardian dispatch, and keep
built-in shared assets repo-managed under `assistant/` so the feature remains
local-first and independently testable without inventing a new top-level
runtime, registry service, or plugin framework.

## Complexity Tracking

No constitution violations are expected for this slice.
