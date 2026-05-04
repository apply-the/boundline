# Implementation Plan: Context Selection Hardening

**Branch**: `040-context-selection-hardening` | **Date**: 2026-05-03 | **Spec**: [/Users/rt/workspace/synod/specs/040-context-selection-hardening/spec.md](/Users/rt/workspace/synod/specs/040-context-selection-hardening/spec.md)
**Input**: Feature specification from `/specs/040-context-selection-hardening/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Replace the current keyword-first workspace file selection in the goal planner
with bounded, evidence-driven context selection. Make each selected input carry
an explicit evidence anchor and rationale, preserve the same context story
through goal-plan, session, trace, `status`, `next`, and `inspect`, stop
planning explicitly when only weak ambient evidence exists, and ship the slice
as `0.40.0` with README layering improvements, roadmap cleanup, changelog,
version bump, coverage above 95% for modified Rust files, `cargo fmt`, and
clean `clippy`.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/config.toml`, optional `.boundline/workflows.toml`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and repository-managed docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential session-native planning plus execution where one bounded context pack is authoritative at a time, explicit compatibility remains subordinate, and no hidden background selection runs outside the existing command surface  
**Observability Surface**: Persisted goal-plan/session state, trace summaries under `.boundline/traces/`, CLI summaries on `plan`, `run`, `status`, `next`, and `inspect`, plus README, docs, roadmap, changelog, and assistant guidance that explain the same context-provenance story  
**Performance Goals**: Operators should identify selected inputs and why they were admitted from normal CLI output in under 2 minutes; the planner should stay within existing bounded workspace scan limits and top-N target selection constraints; maintainers should complete release validation for the slice in under 20 minutes  
**Constraints**: No checkpoint or rewind work in this slice, no generalized semantic index, no background repository crawler, no Canon contract expansion, no new UI, no distributed execution, and no silent keyword-only fallback to a credible context pack  
**Scale/Scope**: One workspace or registered cluster at a time, bounded by existing session/run limits and current planner scan depth, with no more than the existing bounded number of primary context files admitted by one plan revision

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by making planning inputs causal, inspectable, and less likely to target the wrong code. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes planning correctness, failure handling, observability, and release validation ahead of polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `start -> capture -> plan -> run -> status -> next -> inspect`; explicit compatibility follow-up remains available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Planning starts from an existing captured goal, ends in explicit credible/stale/insufficient context state, and stays under current scan and run limits. See Technical Context, research, and contracts.
- **PASS** Stateful execution: Context-pack selection, provenance, and stop reasons remain persisted in goal-plan/session state and traces. See Summary, data model, and contracts.
- **PASS** Mutable planning: The planner can rebuild or supersede context on repeated planning runs while keeping the latest context story explicit. See Summary, research, and data model.
- **PASS** Sequential-first design: The slice keeps one authoritative context pack per plan revision and introduces no concurrency or hidden fan-out. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Evidence gathering, selection, stop conditions, and output projection remain explicit planner/runtime actions rather than hidden heuristics. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: File-level provenance, credibility state, evidence anchors, and recovery cues are surfaced through traces and existing CLI summaries. See Technical Context, data model, quickstart, and contracts.
- **PASS** Non-goals and external separation: Canon remains optional bounded evidence only, and the slice does not introduce councils, long-term memory, UI work, or deployment systems. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is replacing heuristic-first context admission with explicit-evidence context selection on the current planning path, plus the docs and release closure needed to ship it coherently. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/040-context-selection-hardening/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── evidence-selected-context-contract.md
│   ├── insufficient-context-contract.md
│   └── provenance-output-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
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
│   ├── session_command_contract.rs
│   └── trace_summary_contract.rs
├── integration/
│   ├── session_native_flow.rs
│   ├── retry_and_replan.rs
│   └── cli_trace_inspection.rs
└── unit/
    ├── goal_planner.rs
    ├── goal_plan_model.rs
    ├── cli_output.rs
    └── session_model.rs

README.md
docs/
assistant/
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing goal planner,
goal-plan/session/trace models, CLI projection surfaces, and release docs. No
new top-level runtime or storage surface is needed because the feature hardens
the current planning path instead of introducing a second planner or a separate
indexing service.

## Complexity Tracking

No constitution violations are expected for this slice.
