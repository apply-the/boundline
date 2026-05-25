# Implementation Plan: Native Canon CLI Surface

**Branch**: `042-native-canon-cli` | **Date**: 2026-05-05 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/042-native-canon-cli/spec.md`

## Summary

Make Canon the default governed development runtime across the primary Boundline
CLI and assistant command surfaces.  Operators initialize a workspace with Canon
mode-selection preferences and model routes, provide a goal plus bounded
authored inputs (PRD, C4, backlog, repository evidence), answer clarification
questions, and let Boundline/AI assemble Canon-ready input documents for every
canonical Canon mode—without hand-editing workspace manifests or passing
`--governance canon`.  Install diagnostics verify the actual Canon governance
command surface (operations + modes) rather than version alone.

Technical approach: extend the existing `CanonMode` enum with seven missing
canonical modes, retain any legacy `pr-review` representation only as a
backward-compatible alias outside the 15-mode canonical surface, add a
workspace-local Canon preferences section to `config.toml`, expand guided
`init` to collect mode-selection behavior and model routes, default `run` to
Canon governance when the workspace is Canon-ready, enhance install diagnostics
with capability-surface verification, assemble operator inputs into Canon
request payloads transparently, and update all assistant command packs to expose
the same primary Canon-default workflow.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024
**Primary Dependencies**: `clap` 4.x, `serde` 1.x, `serde_json` 1.x, `thiserror` 2.x, `tracing` 0.1, `uuid` 1.x, `toml` 0.8; Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/config.toml`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` governed artifacts
**Testing**: `cargo test`, `cargo nextest run`, unit tests for domain logic and config serialization, integration tests for CLI dispatch and Canon runtime gating, contract tests for governance request/response schemas
**Target Platform**: macOS/Linux developer workstations, Linux CI
**Project Type**: Multi-crate Rust CLI (`boundline-cli`, `boundline-core`, `boundline-adapters`)
**Execution Model**: Sequential session-native pipeline (`start` → `goal` → `plan` → `run`) with bounded governance checkpoints per stage; single step active at a time
**Observability Surface**: Persisted execution traces in `.boundline/traces/`, CLI JSON output with governance intent, selected runtime, mode or mode sequence, mode-selection preference, approval state, blocked reason, governed artifact references, local opt-out state, and next safe action; `status`, `next`, `inspect` surfaces project the same governance lifecycle
**Performance Goals**: CLI startup ≤ 500 ms for local operations; Canon invocation bounded by the external Canon CLI response time; no background processes
**Constraints**: All Canon mode-selection and model-routing settings are workspace-local only (FR-005c); Canon is a bounded external runtime invoked through its CLI—Boundline does not embed Canon logic; no new runtime dependencies; explicit opt-out required for non-Canon governance; advanced compatibility manifests remain available but subordinate
**Scale/Scope**: 15 canonical Canon modes, plus legacy `pr-review` compatibility only if needed for existing persisted state; up to 4 assistant surfaces (Copilot, Codex, Claude, Gemini), single-workspace operator with optional cluster awareness from prior features

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Delivery identity**: **PASS** — This feature makes Canon-governed development the default for all primary operator surfaces, removing the manual manifest barrier that currently prevents most operators from reaching governed execution.  Every change directly serves bounded engineering task delivery through Canon modes (requirements → implementation → verification).

- **Delivery-first scope**: **PASS** — The plan prioritizes execution (Canon-default `run`, input assembly, governance lifecycle), orchestration (mode selection, stage progression, session state), and validation (install diagnostics, capability surface verification) ahead of any polish.  No optimization or cosmetic work is in scope.

- **Primary workflow**: **PASS** — The main operator path is session-native: `init` → `goal`/`run --goal` → `plan` → `run` → `status` → `next` → `inspect`.  The compatibility path through explicit `execution.json` manifests remains available as an advanced override (FR-012) but is not the primary entry point.

- **Bounded execution**: **PASS** — Start conditions: workspace must be Canon-ready (diagnostics pass, config present, Canon binary verified).  Terminal conditions: all governed stages complete, a required stage is blocked/rejected, operator aborts, or clarification is unresolved.  Step/retry limits: inherited from existing orchestrator bounded execution (max steps, max retries per step).  Mode-selection `auto` falls back to confirmation on low confidence rather than guessing.

- **Stateful execution**: **PASS** — Governed development intent, Canon mode-selection preference, selected mode or sequence, approval/blocked state, governed artifact references, and workspace preferences are all persisted to `.boundline/session.json` and `.boundline/config.toml`.  Every governance operation reads from and writes back to shared session state.

- **Mutable planning**: **PASS** — The goal planner already supports replanning.  Canon mode selection under `auto-confirm` and `auto` preferences allows inferred mode sequences that the operator can override.  Stage progression can be interrupted, refreshed, and resumed through the governed lifecycle.

- **Sequential-first design**: **PASS** — One governed stage active at a time.  Canon `start` → wait for result → evaluate readiness → advance or block.  No parallel stage execution or background Canon invocations.

- **Tool-agent symmetry**: **PASS** — Reasoning (mode inference, input sufficiency checks, clarification generation) and action (Canon invocation, artifact persistence, session state update) are separate, explicit steps.  Mode selection decisions are surfaced through the decision record.  Input assembly is traceable.

- **Observability and explicit intelligence**: **PASS** — Trace surfaces: governance intent, selected Canon mode, mode-selection preference source, capability snapshot, input assembly provenance, Canon request/response, approval state, packet readiness, and next safe action.  Mode-selection heuristic under `auto` preference must be exposed through a decision record with confidence and rationale.  All failure and blocked states surface through `status`/`next`/`inspect`.

- **Non-goals and external separation**: **PASS** — Boundline does not embed Canon logic; it invokes Canon through its CLI and consumes structured responses.  No councils, voting, distributed execution, long-term memory, UI/UX, or deployment pipelines.  Canon template authoring/management is explicitly out of scope.  Provider abstraction stays within the existing `GovernanceRuntime` trait boundary.

- **Minimal slice**: **PASS** — The smallest independently valuable capability: Canon-default `boundline run` on a workspace initialized for Canon, with capability-verified diagnostics and workspace-local mode-selection preference.  This alone removes the manifest barrier and the `--governance canon` requirement for the primary operator path.

## Project Structure

### Documentation (this feature)

```text
specs/042-native-canon-cli/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli-commands.md
│   └── canon-request-response.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/workspace.rs             # Shared workspace resolution for init, config,
│                                # doctor, run, and assistant-mapped commands
├── domain/
│   ├── governance.rs          # Extend CanonMode enum, add CanonModeSelectionPreference,
│   │                          # expand capability snapshot, add mode-document mapping
│   ├── configuration.rs       # Add CanonPreferences section to ConfigFile
│   ├── brief.rs               # Enhance GovernanceIntent for default fields
│   ├── distribution.rs        # Expand Canon install evaluation with surface verification
│   └── session.rs             # Project governance lifecycle in session state
├── adapters/
│   ├── governance_runtime.rs  # CanonCliRuntime: add capabilities() method,
│   │                          # enhance request packaging with input assembly
│   └── config_store.rs        # Serialize/deserialize expanded config
├── cli/
│   ├── init.rs                # Guided Canon mode-selection, assistants, model routes
│   ├── run.rs                 # Default to Canon governance when workspace Canon-ready
│   ├── config.rs              # Add set-canon subcommand, expand show/set
│   ├── diagnostics.rs         # Surface-level Canon verification (ops + modes)
│   └── session.rs             # Project governance state in status/next/inspect
├── orchestrator/
│   ├── governance.rs          # Canon-default routing and mode-selection gate
│   └── session_runtime.rs     # Input assembly into Canon request fields
└── registry/                  # No changes expected

assistant/
├── copilot/prompts/           # Update all prompts for Canon-default path;
│                              # add boundline-init, boundline-config-*,
│                              # boundline-doctor, mode-specific aliases,
│                              # and mode-specific Canon-ready input drafting
├── codex/                     # Equivalent command pack updates
├── claude/                    # Equivalent command pack updates
└── gemini/                    # Equivalent command pack updates

tests/
├── unit/                      # CanonMode expansion, config serialization,
│                              # mode-selection logic, input assembly
├── integration/               # CLI init flow, Canon-default run dispatch,
│                              # diagnostics surface verification, config commands
└── contract/                  # Governance request/response schema validation
                               # and assistant command-pack input assembly parity
```

**Structure Decision**: No new top-level directories or crates.  All changes fit
within the existing `src/domain/`, `src/adapters/`, `src/cli/`,
`src/orchestrator/`, and `assistant/` directories.  The feature extends existing
modules rather than introducing new structural complexity.

## Complexity Tracking

> No Constitution Check violations.  All changes are additive extensions to
> existing modules within the established project structure.
