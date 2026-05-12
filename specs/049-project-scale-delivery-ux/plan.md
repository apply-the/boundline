# Implementation Plan: Boundline Project-Scale Delivery UX

**Branch**: `049-project-scale-delivery-ux` | **Date**: 2026-05-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/049-project-scale-delivery-ux/spec.md`

## Summary

Design the next Boundline product slice for project-scale delivery UX while preserving the existing runtime boundary: Boundline drives delivery and Canon governs packets at stage boundaries. The implementation approach is to add a user-scoped/global assistant bootstrap package model, a full Canon-mode governed stage catalog behind `/boundline:govern`, bounded project-scale path selection, risk-triggered voting state, and Delivery Pilot Model documentation. The runtime remains session-native, sequential-first, and backed by `.boundline/session.json`; assistant chat history is never authoritative.

## Technical Context

**Language/Version**: Rust 1.95.0 workspace, edition 2024, plus JSON, Markdown, TOML, Bash, and assistant command assets  
**Primary Dependencies**: Existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`); external Canon CLI compatibility target `0.45.0`; no new runtime crates planned for the first implementation slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/checkpoints/`, optional `.canon/` governed packet artifacts, repo-managed assistant package files, docs, and Spec Kit artifacts  
**Testing**: Contract/unit/integration tests for assistant bootstrap, governed stage catalog, `/boundline:govern`, voting state projection, status/next/inspect parity, docs validation, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`, and touched-Rust-file coverage with `cargo llvm-cov`  
**Target Platform**: Local developer workstations and CI on macOS/Linux/Windows-friendly metadata; assistant host packages for Claude Code, Codex, Cursor, Copilot-style prompt environments, and Gemini-style CLI/chat environments where supported  
**Project Type**: Rust CLI workspace with repository-managed assistant command packs, persisted session state, and documentation assets  
**Execution Model**: Sequential session-native delivery loop (`observe -> decide -> act -> verify -> update context`) with one active bounded stage or work unit at a time, bounded retries, explicit confirmation gates for material transitions, and no background autonomous execution  
**Observability Surface**: CLI and assistant `status`, `next`, `inspect`, traces, checkpoint refs, governed packet refs, approval state, voting state, reviewed evidence refs, validation output, and documentation quickstarts  
**Performance Goals**: Bootstrap and status-style commands should complete with one CLI state read or one diagnostic command; governed mode capability checks should use one Canon capability query; no hidden polling or long-lived host-side work  
**Constraints**: First implementation task must improve/bump Boundline version; Canon remains stage-boundary governance only; no per-mode primary commands such as `/boundline-architecture`; no claims of true global host support where a host does not provide it; no hand-edited JSON/manifests for normal operation; final task must add tests to meet at least 95% coverage for created/modified Rust files, run clippy, resolve issues, and run cargo fmt  
**Scale/Scope**: One project-scale delivery UX slice covering five user stories: global assistant bootstrap, idea-to-code pathing, explicit governed stage work, risk-triggered voting, and Delivery Pilot Model docs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature improves Boundline as a delivery orchestrator by making broad engineering work decomposable into bounded stages, with chat and CLI surfaces leading to the same runtime state. See Summary.
- **PASS** Delivery-first scope: The slice prioritizes orchestration, decomposition, validation, governance boundary routing, and status/inspect visibility before polish.
- **PASS** Primary workflow: The main operator path remains session-native (`start -> capture -> plan -> run -> status -> next -> inspect`). Global assistant bootstrap is a discovery/init path; compatibility manifests remain advanced-only.
- **PASS** Bounded execution: Each stage and work unit has entry conditions, stop conditions, confirmation gates, retry exhaustion behavior, and terminal/blocked states. No unbounded project execution is introduced.
- **PASS** Stateful execution: `.boundline/session.json` remains authoritative, and CLI/chat surfaces read the same session, trace, checkpoint, governance, and voting refs.
- **PASS** Mutable planning: Project-scale paths can insert, replace, or defer stages based on evidence, risk, Canon capability results, approval state, and validation outcomes, with the changes visible in traces.
- **PASS** Sequential-first design: The design keeps one active bounded stage or work unit at a time. Voting is a bounded review decision at selected risk boundaries, not parallel execution.
- **PASS** Tool-agent symmetry: Global commands, `/boundline:govern`, path selection, Canon capability checks, validation, voting, and inspection are explicit actions with visible inputs and outputs.
- **PASS** Observability and explicit intelligence: Stage selection, unsupported modes, approval gates, voting triggers, blocking findings, retries, terminal states, and next commands must be visible in CLI/chat summaries and traces.
- **PASS** Catalog currency: Public provider docs were checked on 2026-05-11; no model-catalog delta is required for this UX slice. Evidence and rationale are recorded in [research.md](./research.md).
- **PASS** Non-goals and external separation: Canon governs packets but does not orchestrate. Voting is explicitly scoped to risk-boundary delivery quality control, with reviewer counts/rules/triggers/terminal outcomes in scope; generic councils, provider abstraction expansion, long-term memory, UI, and deployment pipelines remain out of scope.
- **PASS** Minimal slice: The smallest independently valuable capability is project-scale UX specification plus implementable surfaces for global bootstrap, governed stage routing, voting visibility, and docs without redesigning the runtime.

## Project Structure

### Documentation (this feature)

```text
specs/049-project-scale-delivery-ux/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── coherence-review.md
├── task-consistency-review.md
├── contracts/
│   └── project-scale-delivery-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
CHANGELOG.md
README.md
AGENTS.md
assistant/
├── README.md
├── plugin-metadata.json
├── catalog/model-catalog.toml
├── commands/session-workflow.json
├── global/
│   ├── claude/
│   ├── codex/
│   ├── cursor/
│   ├── copilot/
│   └── gemini/
└── prompts/
docs/
├── architecture.md
├── delivery-model.md
├── getting-started.md
├── guides/assistant-plugin-packages.md
└── review-voting.md
distribution/
├── channel-metadata.toml
└── winget/manifests/a/ApplyThe/Boundline/
src/
├── adapters/
│   └── governance_runtime.rs
├── cli/
│   ├── assistant_assets.rs
│   ├── diagnostics.rs
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── governance.rs
│   ├── review.rs
│   ├── session.rs
│   └── workflow.rs
└── orchestrator/
    ├── decision_loop.rs
    ├── governance.rs
    ├── review_trace.rs
    └── session_runtime.rs
tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the work inside existing CLI, domain, orchestrator, governance, review, assistant, docs, and distribution surfaces. New global assistant package assets can live under `assistant/global/` because the previous repo-local host plugin packages already exist at the repository root. This avoids a second runtime while making bootstrap assets share metadata and command behavior with repo-local packages.

## Complexity Tracking

No constitution violations are required. The only elevated complexity is voting, which is explicitly scoped by the feature prompt and constitution as bounded delivery quality-control at risky stage boundaries, not as a generic multi-agent council.

## Phase 0: Research

Research decisions are captured in [research.md](./research.md). The main resolved questions are global-vs-repo-local assistant package boundaries, full Canon mode routing through one governed surface, voting trigger boundaries, Delivery Pilot Model documentation placement, Canon `0.45.0` capability validation, and model-catalog currency.

## Phase 1: Design & Contracts

Data entities, relationships, validation rules, and state transitions are captured in [data-model.md](./data-model.md). The operator-facing CLI/assistant contracts are captured in [contracts/project-scale-delivery-contract.md](./contracts/project-scale-delivery-contract.md). Manual acceptance and smoke-test flows are captured in [quickstart.md](./quickstart.md).

## Post-Design Constitution Check

- **PASS** The design still uses the existing session-native runtime and does not introduce a second orchestrator.
- **PASS** Global assistant commands bootstrap or diagnose state only; repo-local commands continue to read `.boundline/session.json` through the CLI.
- **PASS** Canon mode support is centralized in a governed stage catalog and `/boundline:govern`, with Canon invoked only at governed stage boundaries.
- **PASS** Voting remains risk/evidence-triggered, persisted, and inspectable. Low-risk stages are not burdened by default voting.
- **PASS** Docs explain decomposition and explicit stopping rules so project-scale UX does not imply unbounded autonomy.
