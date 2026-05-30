# Implementation Plan: Domain Agent Templates

**Branch**: `038-domain-agent-templates` | **Date**: 2026-05-03 | **Spec**: [specs/038-domain-agent-templates/spec.md](specs/038-domain-agent-templates/spec.md)
**Input**: Feature specification from `/specs/038-domain-agent-templates/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend Boundline's existing configuration, context assembly, and read-side CLI
surfaces so bounded tasks can inherit a first-party domain-template catalog,
layer shared and workspace-specific standards with explicit precedence, bind
optional or required external context inputs, and reuse Canon-governed
artifacts as supporting evidence without ceding template ownership. Keep the
main operator path session-native, block planning explicitly when no credible
domain guidance exists for the bounded task, surface the applied domain story
through `init`, `config show`, `plan`, `run`, `status`, `next`, and `inspect`,
and ship the slice as `0.38.0` with release closure and >95% coverage for
modified Rust files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/config.toml`, cluster-local `.boundline/cluster.toml`, user-global config at `$XDG_CONFIG_HOME/boundline/config.toml` or `$HOME/.config/boundline/config.toml`, persisted session and trace state under `.boundline/session.json` and `.boundline/traces/`, optional `.boundline/execution.json`, and repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with file-backed configuration, session, and trace state  
**Execution Model**: Sequential session-native planning plus bounded observe -> decide -> act -> verify execution where domain guidance is selected from effective config and bounded task evidence, and planning stops explicitly when domain context is insufficient  
**Observability Surface**: Persisted scoped config, goal-plan context packs, task-context state when present, execution traces under `.boundline/traces/`, CLI summaries on `init`, `config show`, `plan`, `run`, `status`, `next`, and `inspect`, plus updated docs and assistant guidance  
**Performance Goals**: Operators should identify the active domain family, standards precedence, and supporting external inputs from normal CLI output in under 2 minutes; `init` should seed the relevant workspace domain selections in under 15 minutes on representative repositories; release validation for the slice should complete in under 20 minutes  
**Constraints**: No new orchestration runtime, no background workers, no hidden concurrency, no Canon-owned template selection, no requirement to execute every possible external context provider directly, no third-party template marketplace, and compatibility follow-up remains subordinate and trace-authoritative  
**Scale/Scope**: One workspace or registered cluster at a time, a first-party catalog covering the declared major language and framework families, bounded per-task domain selection from existing targets and workspace evidence, and scoped standards plus external bindings resolved from global, cluster, and workspace config

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by letting Boundline apply the correct domain guidance, local standards, and supporting context before and during execution. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes init/config persistence, context assembly, planning gates, and execution/inspection surfaces ahead of documentation polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`; explicit compatibility follow-up stays available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Planning stops explicitly when no credible domain guidance or required supporting input exists; execution stays inside existing step and retry limits with no background activity. See Summary, Technical Context, contracts, and quickstart.
- **PASS** Stateful execution: Effective domain config, applied domain context, optional governed artifacts, and external input status are persisted through existing scoped config, goal-plan, task-context, and trace state rather than hidden process state. See Summary, data model, and contracts.
- **PASS** Mutable planning: Domain guidance is derived from current bounded context and can change on replan or later task-target changes without inventing a second planning system. See Summary, Technical Context, and research.
- **PASS** Sequential-first design: Domain selection, context assembly, planning, execution, and inspection remain one-step-at-a-time state transitions with no fan-out or background MCP orchestration. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Domain templates, governed artifacts, and external bindings are explicit inputs to bounded decisions rather than hidden prompt state, and resulting reasoning remains visible through existing execution surfaces. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Active domain families, precedence sources, governed artifacts, external-context status, and blocked-domain reasons are surfaced through config, goal-plan projection, and traces. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: Canon remains optional and downstream, external context providers stay supporting inputs rather than control-flow owners, and the plan does not reintroduce councils, distributed execution, long-term memory, UI work, or deployment pipelines. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is one end-to-end domain-template system that can be configured, applied, inspected, and updated without introducing a new runtime or marketplace. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/038-domain-agent-templates/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── domain-template-configuration-surface-contract.md
│   ├── applied-domain-context-contract.md
│   └── external-context-binding-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── config.rs
│   ├── init.rs
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── goal_plan.rs
│   ├── task_context.rs
│   └── domain_templates.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
├── integration/
└── unit/

README.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
assistant/
docs/
AGENTS.md
```

**Structure Decision**: Keep the slice inside existing config, planning,
execution, and read-side surfaces. Add one dedicated domain-template module for
the first-party catalog and applied-domain entities, but keep persistence in the
existing scoped config files and keep domain projection attached to existing
goal-plan, task-context, and trace state. No new top-level runtime or storage
plane is needed.

## Complexity Tracking

No constitution violations are expected for this slice.
