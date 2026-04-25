# Implementation Plan: Assistant Command Packs

**Branch**: `003-assistant-command-packs` | **Date**: 2026-04-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-assistant-command-packs/spec.md`

## Summary

Ship repository-managed assistant command packs for Copilot, Codex, and Claude so developers can start, continue, inspect, and route Synod workflows from chat without memorizing long CLI invocations. The plan keeps the existing Rust CLI as the execution and inspection backend, documents assistant-only workflow commands for `start`, `plan`, `step`, `status`, and `next`, and validates the asset set with Rust-based contract and integration tests so assistant packs cannot drift from the CLI behavior they wrap.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024 for the existing CLI backend plus repository-managed Markdown prompt assets for assistant command packs  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice  
**Storage**: Repository-stored assistant asset files under `assistant/` and existing workspace-local traces under `<workspace>/.synod/traces/` for status and inspection backends  
**Testing**: `cargo test` with new contract, unit, and integration tests that load assistant assets, verify required command coverage and backend mappings, and exercise shell-enabled plus chat-only fallback flows against the current CLI  
**Target Platform**: Copilot, Codex, and Claude environments used on macOS and Linux developer workstations, plus Linux CI for formatting, linting, tests, and asset drift validation  
**Project Type**: Single Rust package with repository-managed assistant command assets layered over an existing local developer CLI  
**Execution Model**: Each assistant-native command either triggers one explicit synchronous CLI invocation (`doctor`, `run`, or `inspect`) or performs assistant-side routing that gathers missing context and recommends the next concrete command without hidden background execution  
**Observability Surface**: Existing human-readable CLI output, persisted JSON traces, trace inspection summaries, repository-documented command contracts, and assistant responses that must surface direct execution vs copy-paste fallback explicitly  
**Performance Goals**: First-time users can reach a runnable assistant command flow in under 2 minutes, shell-enabled command packs add no material overhead beyond the existing CLI runtime, and latest-trace status/inspection guidance remains interactive with sub-2-second turnaround excluding orchestrator step execution time  
**Constraints**: No new runtime services or assistant-specific APIs, no new orchestration engine, no background workers, no hidden assistant memory beyond explicit conversation context, keep CLI-first architecture intact, and preserve compatibility with current `doctor`, `run`, and `inspect` behavior  
**Scale/Scope**: Seven assistant commands across three environments, one active workflow context per conversation, one local run or trace inspected at a time, and repository-validated asset consistency instead of dynamic command registration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. Summary, Technical Context, and contracts keep the feature focused on bounded engineering-task delivery by making existing Synod execution workflows usable from chat rather than introducing a generic assistant platform.
- Delivery-first scope: PASS. Summary, Technical Context, research, and quickstart prioritize starting, running, inspecting, and routing real work ahead of assistant polish or platform-specific niceties.
- Bounded execution: PASS. Technical Context, research, and the fallback contract keep every direct backend action to one explicit CLI invocation with existing terminal conditions, while assistant-only commands stop after collecting context or recommending a next action.
- Stateful execution: PASS. Data model and fallback contract make workflow continuity explicit through conversation context fields such as workspace, goal, latest trace, and latest outcome instead of relying on hidden state.
- Mutable planning: PASS. Research, data model, and assistant command definition contract preserve visible retry and replanning behavior by routing `status`, `next`, and `inspect` through existing trace summaries rather than hiding plan mutations.
- Sequential-first design: PASS. Technical Context and Project Structure keep one command at a time, reuse the current sequential CLI/orchestrator backend, and avoid background workers or concurrent assistant execution.
- Tool-agent symmetry: PASS. Assistant command definitions make it explicit when the assistant is reasoning, when the CLI is acting, and when trace inspection is evaluating prior execution.
- Observability and explicit intelligence: PASS. Technical Context, contracts, and quickstart require assistant packs to expose direct execution paths, copy-paste fallbacks, latest-trace inspection, terminal status, recovery signals, and recommended next commands without silent heuristics.
- Non-goals and external separation: PASS. Summary, Technical Context, and Project Structure avoid Canon dependencies, external servers, provider abstraction work, long-term memory, UI surfaces, or deployment workflows.
- Minimal slice: PASS. Summary, research, and Project Structure deliver the smallest independently valuable slice: repository-shipped assistant command assets plus validation over the CLI that already exists.

## Project Structure

### Documentation (this feature)

```text
specs/003-assistant-command-packs/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── assistant-command-definition-contract.md
│   ├── assistant-command-pack-contract.md
│   └── assistant-fallback-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
assistant/
├── README.md
├── claude/
│   └── commands/
│       ├── synod-start.md
│       ├── synod-plan.md
│       ├── synod-step.md
│       ├── synod-run.md
│       ├── synod-status.md
│       ├── synod-next.md
│       └── synod-inspect.md
├── codex/
│   └── commands/
│       ├── synod-start.md
│       ├── synod-plan.md
│       ├── synod-step.md
│       ├── synod-run.md
│       ├── synod-status.md
│       ├── synod-next.md
│       └── synod-inspect.md
└── copilot/
    └── prompts/
        ├── synod-start.prompt.md
        ├── synod-plan.prompt.md
        ├── synod-step.prompt.md
        ├── synod-run.prompt.md
        ├── synod-status.prompt.md
        ├── synod-next.prompt.md
        └── synod-inspect.prompt.md

src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── inspect.rs
│   ├── output.rs
│   └── run.rs
├── adapters/
│   └── trace_store.rs
└── domain/
    └── trace.rs

tests/
├── contract/
│   ├── assistant_command_definition_contract.rs
│   ├── assistant_command_pack_contract.rs
│   ├── cli_command_contract.rs
│   └── trace_summary_contract.rs
├── integration/
│   ├── assistant_chat_fallback.rs
│   └── assistant_shell_enabled_flow.rs
└── unit/
    └── assistant_assets.rs
```

**Structure Decision**: Introduce one top-level `assistant/` directory because the feature's primary deliverable is a repository-shipped asset surface, not a new runtime. Keep the existing CLI and trace modules as the backend for direct execution and inspection. Place validation in Rust tests so the same quality gates that already protect the CLI also protect the assistant assets without adding a second scripting or generation pipeline.

## Complexity Tracking

No constitution violations require justification at this stage.