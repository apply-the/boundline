# Implementation Plan: Product Unification And Surface Closure

**Branch**: `032-workflow-surface-closure` | **Date**: 2026-05-02 | **Spec**: [specs/032-workflow-surface-closure/spec.md](specs/032-workflow-surface-closure/spec.md)
**Input**: Feature specification from `/specs/032-workflow-surface-closure/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Close the remaining product-identity gap by making named workflows first-class
assistant surfaces on the same primary session-native Boundline path, projecting
workflow routing plus assistant or model binding as explicitly as the direct
session surfaces, and keeping explicit compatibility follow-up visibly
subordinate. The slice stays inside the current CLI, session, trace,
configuration, assistant-asset, and docs surfaces, ships as `0.32.0`, and
closes with impacted docs, changelog, coverage refresh for touched Rust files,
clippy cleanup, and formatting.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/workflows.toml`, `.boundline/config.toml`, `.boundline/session.json`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json` for explicit compatibility follow-up, optional `.canon/` artifacts, and repository-managed assistant assets under `assistant/`  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted contract or integration tests for workflow surfaces and assistant assets, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state and repository-managed assistant command assets  
**Execution Model**: Sequential session-owned execution where named workflows compile onto the same primary `goal -> plan -> run -> status -> next -> inspect` path, while explicit compatibility execution remains opt-in and trace-authoritative  
**Observability Surface**: Workflow-aware CLI summaries, persisted session state, persisted traces, route-config projection, assistant-binding projection, workflow discovery output, assistant command guidance, and release docs that explain primary versus subordinate execution paths  
**Performance Goals**: Operators should identify the authoritative workflow or direct native path plus active route and assistant binding in under 2 minutes; maintainers should validate the `0.32.0` release story in under 20 minutes  
**Constraints**: No new workflow runtime, no provider-auth or provider-gateway layer, no assistant-owned orchestration, no hidden compatibility fallback, no distributed execution, no GUI surface, and no expansion of Canon beyond its existing bounded governed role; Gemini remains CLI-first in this slice  
**Scale/Scope**: One workspace or registered cluster at a time, shipped assistant surfaces for Claude, Codex, Copilot, and Gemini guidance, bounded updates to workflow CLI rendering, assistant assets, docs, and release-validation surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by letting operators enter and continue the same Boundline delivery story through workflows without losing route or next-step credibility. See Summary, Technical Context, and [specs/032-workflow-surface-closure/spec.md](specs/032-workflow-surface-closure/spec.md).
- **PASS** Delivery-first scope: The plan prioritizes execution ownership, route clarity, workflow follow-through, and inspectability ahead of release polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native, with named workflows compiling onto the same `goal -> plan -> run -> status -> next -> inspect` story; explicit compatibility remains available only as an opt-in subordinate route. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Workflow discovery, run, status, resume, and inspect keep explicit stop conditions for missing input, invalid definitions, blocked governance, assistant-binding mismatch, and terminal outcome; no new loops or retries are introduced beyond current runtime limits. See Technical Context, research, data model, and quickstart.
- **PASS** Stateful execution: Workflow identity, progress, route projection, and continuity cues remain grounded in the shared session or trace story rather than a stateless assistant surface. See Summary, data model, and contracts.
- **PASS** Mutable planning: Workflows continue to rely on the existing mutable goal-plan and bounded follow-through model rather than replacing planning with a fixed scripted runner. See Summary, research, and data model.
- **PASS** Sequential-first design: One workflow phase remains active at a time and the slice introduces no concurrency, background workers, or hidden branches. See Technical Context and research.
- **PASS** Tool-agent symmetry: Workflow guidance, route projection, and continuation remain explicit through CLI output and assistant guidance rather than hidden inside backend-specific behavior. See Summary, research, quickstart, and contracts.
- **PASS** Observability and explicit intelligence: Workflow identity, phase, route authority, assistant binding, compatibility ownership, blocked conditions, and next commands remain surfaced on the same operator-facing outputs. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: The plan does not create a new provider abstraction layer, UI surface, long-term memory, deployment work, or Canon-owned control flow. See Constraints, research, and [specs/032-workflow-surface-closure/spec.md](specs/032-workflow-surface-closure/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one workflow-first assistant and output closure that makes the primary Boundline product story explicit without inventing another runtime. See Summary and research.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/032-workflow-surface-closure/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── compatibility-product-boundary-contract.md
│   ├── workflow-assistant-surface-contract.md
│   └── workflow-routing-projection-contract.md
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
│   ├── output.rs
│   ├── session.rs
│   └── workflow.rs
├── domain/
│   ├── configuration.rs
│   ├── routing_decision.rs
│   ├── session.rs
│   └── workflow.rs
├── orchestrator/
│   └── session_runtime.rs
└── lib.rs

assistant/
├── README.md
├── claude/commands/
├── codex/commands/
├── copilot/prompts/
└── gemini/

docs/
├── configuration.md
└── getting-started.md

tests/
├── contract/
├── integration/
└── unit/

README.md
CONTRIBUTING.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing workflow CLI,
session projection, route-projection, assistant-asset, docs, and test surfaces.
The expected code changes are bounded updates to existing workflow rendering and
session-runtime validation paths, plus new assistant workflow asset files and
contract or integration coverage. No new top-level runtime or service is
justified because the feature closes product-surface ambiguity rather than
adding a second execution model.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
