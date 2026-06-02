# Implementation Plan: Stack-Neutral Workspace Entry

**Branch**: `043-stack-neutral-init` | **Date**: 2026-05-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/043-stack-neutral-init/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Make workspace entry stack-neutral across the primary Boundline flow and align
`init` with the assistant and domain choices operators already make. Operators
should be able to start from an empty or non-Rust repository, initialize a
workspace by naming Claude, Copilot, Codex, or Gemini without hand-picking model
IDs, and let Boundline seed bounded technology-specific hygiene defaults only
when the selected domains or repository evidence make those defaults credible.

Technical approach: remove Rust-specific manifest checks from generic workspace
readiness, reuse the existing built-in routing catalog in
`src/domain/configuration.rs` as the single source of default assistant models,
extend `init` to auto-fill route slots from assistant-target defaults while
preserving explicit overrides, add a bounded workspace-hygiene policy module for
merge-only ignore defaults keyed by selected domain families and tool cues, and
project the new behavior through CLI output, docs, and release notes.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap` 4.x, `serde` 1.x, `serde_json` 1.x, `thiserror` 2.x, `tracing` 0.1, `uuid` 1.x, `toml` 0.8, plus Rust standard library filesystem, path, collections, and process APIs; no new runtime dependencies planned  
**Storage**: Workspace-local `.boundline/config.toml`, optional `.boundline/execution.json`, `.boundline/session.json`, `.boundline/traces/`, and repository ignore files such as `.gitignore`, `.dockerignore`, `.eslintignore`, `.prettierignore`, `.terraformignore`, and `.helmignore` when bounded hygiene defaults justify them  
**Testing**: `cargo test`, targeted integration tests for `init`, `doctor`, and native direct-run entry, unit tests for routing defaults and hygiene merge logic, contract-style assertions for config and CLI output surfaces  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Multi-crate Rust CLI with repository-managed docs and assistant command packs  
**Execution Model**: Sequential session-native pipeline and direct native bootstrap, one active bounded step at a time, no background workers  
**Observability Surface**: CLI diagnostics output, `init` summary output, `config show --scope effective`, persisted session and trace state, and inspectable on-disk hygiene files created or updated by bounded setup  
**Performance Goals**: `doctor` and `init` remain sub-second for local repository checks on representative small and medium workspaces; no added background processing or repeated repository scans beyond bounded file checks  
**Constraints**: Generic workspace readiness must stay stack-neutral; explicit overrides continue to win over defaults; hygiene seeding is merge-only and must not wipe operator-authored rules; no Canon dependence for the core slice; compatibility remains explicit and subordinate  
**Scale/Scope**: Four supported assistant families for default-model seeding, thirteen first-party domain families from spec 038, bounded ignore-default support for universal plus tool-specific patterns where repository cues justify them

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Delivery identity**: **PASS** — The feature improves real bounded delivery entry by letting operators start a session from credible non-Rust workspaces, select assistant/model defaults during workspace initialization, and carry domain selection into repository hygiene instead of stopping before execution begins.
- **Delivery-first scope**: **PASS** — The plan prioritizes execution entry, initialization, bounded setup hygiene, and validation surfaces ahead of polish. No speculative platform work or UI work is introduced.
- **Primary workflow**: **PASS** — The main operator path remains session-native: `goal -> plan -> confirm -> run -> status -> next -> inspect`. Direct native `run --goal` continues as the fast path to that same session-native route. Explicit compatibility behavior through `.boundline/execution.json` remains available but subordinate.
- **Bounded execution**: **PASS** — Start conditions: existing writable workspace plus any route-specific prerequisites already required by the native flow. Terminal conditions: initialization completes, planning continues, or the run stops explicitly because stack/domain or assistant default selection is not credible. Step and retry limits remain inherited from the existing sequential orchestrator.
- **Stateful execution**: **PASS** — Workspace-local config remains the authority for assistant runtimes, resolved route defaults, domain-template settings, and any seeded hygiene defaults that later planning or inspection must explain. Native planning and run surfaces keep using persisted session and trace state.
- **Mutable planning**: **PASS** — The feature does not remove existing replanning behavior. Instead, it makes the initial workspace/bootstrap context more credible so later plan mutation works from explicit domain and routing defaults instead of a Rust-biased prerequisite.
- **Sequential-first design**: **PASS** — All new behavior stays sequential: detect workspace, seed defaults, persist config, then continue. Hygiene generation is file-by-file, merge-only, and synchronous.
- **Tool-agent symmetry**: **PASS** — Reasoning remains explicit in model-default and hygiene-default selection, while action stays explicit through config persistence and ignore-file updates. No hidden background automation is introduced.
- **Observability and explicit intelligence**: **PASS** — `doctor`, `init`, `config show`, and existing follow-through surfaces will expose why a workspace is ready, which assistant defaults were chosen, which routes were overridden, and which hygiene defaults were applied or skipped.
- **Non-goals and external separation**: **PASS** — Canon is not required for the core capability. No councils, voting, distributed execution, long-term memory, UI/UX, or deployment pipelines are introduced. Model discovery stays repository-managed rather than depending on an external provider API.
- **Minimal slice**: **PASS** — The smallest independently valuable capability is: a writable empty or non-Rust repository can enter the native path, `init` can seed assistant-specific default routes without manual model strings, and the selected domains can drive bounded ignore-file hygiene updates.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/043-stack-neutral-init/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── init.rs
│   ├── run.rs
│   └── workspace.rs
├── domain/
│   ├── configuration.rs
│   ├── domain_templates.rs
│   └── workspace_hygiene.rs
├── adapters/
│   └── config_store.rs
├── orchestrator/
│   └── goal_planner.rs
└── lib.rs

tests/
├── integration/
├── contract/
└── unit/

tech-docs/
├── configuration.md
├── getting-started.md
└── architecture.md

assistant/
└── README.md
```

**Structure Decision**: Keep the feature inside the existing CLI/domain/docs
surfaces. Add one new domain module, `src/domain/workspace_hygiene.rs`, because
technology-specific hygiene policies need a reusable, testable home separate
from CLI prompting and file I/O. Reuse `src/domain/configuration.rs` as the
shared source of built-in routing defaults so `init`, config resolution, and
effective routing stay aligned.

## Complexity Tracking

> No Constitution Check violations. All additions stay inside the existing
> repository structure and sequential delivery model.
