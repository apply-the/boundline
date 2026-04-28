# Implementation Plan: Human-Friendly Init and Model Routing

**Branch**: `011-init-model-routing` | **Date**: 2026-04-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/011-init-model-routing/spec.md`

## Summary

Add a human-friendly `synod init` entry point that scaffolds bounded workspace
files, detects supported assistant runtimes, offers repository-local assistant
setup when needed, and stores editable runtime/model routing defaults with
deterministic precedence across CLI flags, workspace-local config, user-scoped
global config, and built-in defaults. The implementation keeps the existing
manifest-driven execution profile for bounded runtime behavior, introduces a new
human-editable config layer for routing preferences and assistant setup, extends
review routing so councils and adjudicators can use distinct defaults, and makes
the effective configuration inspectable through CLI and output surfaces.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `toml` for human-editable config serialization; no additional runtime abstraction crates for the first slice  
**Storage**: Workspace-local `.synod/execution.json`, `.synod/session.json`, `.synod/traces/`, new workspace-local `.synod/config.toml`, and new user-scoped global config at `$XDG_CONFIG_HOME/synod/config.toml` with fallback to `$HOME/.config/synod/config.toml` on macOS/Linux developer machines  
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, focused contract tests for CLI and config surfaces, focused integration tests for init and precedence, and unit tests for config resolution and validation  
**Target Platform**: macOS and Linux developer workstations, Linux CI, and VS Code assistant-driven repository sessions  
**Project Type**: Single Rust CLI crate with file-backed workspace state, repository-managed assistant assets, and no background services  
**Execution Model**: Sequential CLI flow with one init or config command active at a time, deterministic config precedence resolution, explicit preview before destructive writes, and no hidden background setup workers  
**Observability Surface**: CLI init summaries, config inspection output, doctor validation feedback, persisted session and trace output for runtime use, and assistant-facing docs/prompts describing the effective routing behavior  
**Performance Goals**: Init and config inspection should feel interactive for a normal repository, keeping runtime detection, config resolution, and scaffold preview within one command round-trip and under roughly 2 seconds before any optional file writes  
**Constraints**: Preserve existing manifest-driven automation, keep bounded execution policy explicit, support only Claude, Codex, Copilot, and Gemini CLI in this slice, keep Gemini CLI as CLI-only, avoid hidden provider heuristics, require explicit confirmation before overwriting user-managed files, and keep review voting deterministic while only routing reviewer/adjudicator sources  
**Scale/Scope**: One active workspace at a time, one global config file per user, one local config file per workspace, built-in `bug-fix`, `change`, and `delivery` templates only, and a bounded set of delivery/review routing slots rather than arbitrary dynamic stage graphs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature removes the biggest remaining operator barrier to bounded delivery work by replacing hand-authored setup with a guided init and explicit routing config while keeping execution bounded. See Summary and Technical Context.
- Delivery-first scope: PASS. The plan prioritizes workspace bootstrap, routing persistence, reviewer-role differentiation, and inspection before documentation polish. See Summary and Project Structure.
- Bounded execution: PASS. Init and config flows have explicit start conditions, preview/confirmation points, and explicit blocked or aborted terminal states when runtime detection, validation, or overwrite checks fail. See Technical Context and research decisions.
- Stateful execution: PASS. Workspace and user config become explicit persisted state alongside the existing execution manifest, while runtime use continues to read/write session and trace state. See Technical Context and data model.
- Mutable planning: PASS. The feature does not remove planning mutation; it improves the setup and routing inputs that later planning and review stages consume. Existing replanning remains intact. See Summary and research decisions.
- Sequential-first design: PASS. Init, config resolution, review-role routing, and assistant setup all happen one command at a time with no new background workers or concurrency. See Technical Context.
- Tool-agent symmetry: PASS. Runtime choice, review routing, assistant setup, and config precedence are exposed as explicit CLI actions and persisted values rather than hidden automation. See contracts and quickstart.
- Observability and explicit intelligence: PASS. Effective routing inspection, previewed file writes, runtime capability reports, and traceable config sources make the setup path inspectable. See Technical Context and contracts.
- Non-goals and external separation: PASS. The plan does not depend on Canon for correctness, does not introduce distributed execution or UI work, and constrains provider/model support to a bounded approved slice. See Technical Context and Scope Boundaries.
- Minimal slice: PASS. The smallest independently valuable capability is a usable `synod init` plus editable routing config with explicit precedence and differentiated review-role defaults. See Summary.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/011-init-model-routing/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── config-cli-contract.md
│   ├── init-cli-contract.md
│   └── routing-resolution-contract.md
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
├── adapters/
│   ├── config_store.rs
│   └── session_store.rs
├── cli/
│   ├── config.rs
│   ├── diagnostics.rs
│   ├── init.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── execution.rs
│   ├── review.rs
│   └── trace.rs
├── fixture.rs
├── lib.rs
└── cli.rs

assistant/
├── README.md
├── claude/
├── codex/
├── copilot/
└── gemini/

docs/
├── getting-started.md
├── adaptive-execution.md
├── review-voting.md
└── configuration.md

tests/
├── contract/
├── integration/
├── support/
└── unit/
```

**Structure Decision**: Keep the feature inside the existing crate and assistant
asset tree. Add one new config adapter, dedicated `init` and `config` CLI
modules, and one new configuration domain model rather than spreading routing
logic across unrelated modules. Keep the execution manifest under `.synod` as
the bounded runtime contract, but separate human-editable routing preferences
into TOML config files so the operator path becomes friendlier without changing
the execution engine’s core manifest schema. The new `docs/configuration.md`
file is justified because this feature introduces a new persistent user-facing
configuration surface that cannot be explained cleanly only inside README.

## Complexity Tracking

No constitution violations require justification for this slice.
