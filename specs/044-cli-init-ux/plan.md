# Implementation Plan: Guided CLI UX And Clearer Messaging

**Branch**: `044-cli-init-ux` | **Date**: 2026-05-07 | **Spec**: [specs/044-cli-init-ux/spec.md](specs/044-cli-init-ux/spec.md)
**Input**: Feature specification from `/specs/044-cli-init-ux/spec.md`

## Summary

Improve the first-run Boundline operator experience by making `boundline init`
and adjacent `doctor` messaging self-sufficient: interactive prompts must show
valid choices, route syntax, blank/default behavior, and recovery guidance
without forcing users into external docs. The implementation will keep the
existing non-interactive CLI stable while adding richer guided prompt text,
more human-readable validation errors, post-init summaries that explain seeded
and explicit routes, safer overwrite messaging, and capability-aware formatting
that remains plain-text safe for CI and redirected output.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library terminal and filesystem APIs  
**Storage**: Workspace-local files under `.boundline/`, repository-local assistant asset files under `assistant/`, and stdout/stderr CLI summaries  
**Testing**: `cargo test`, focused integration tests in `tests/integration.rs`, contract tests in `tests/contract.rs`, plus final `cargo clippy --workspace --all-targets --all-features -- -D warnings`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Multi-crate Rust CLI with source-root shared modules reused by `crates/boundline-cli` and `crates/boundline-core`  
**Execution Model**: Sequential CLI command execution with one operator-visible step at a time; guided `init` only reads input synchronously from the current TTY  
**Observability Surface**: `boundline init` summary lines, `boundline doctor` diagnostics output, clap help text, integration/contract tests, and existing workspace-local config plus compatibility execution file output  
**Performance Goals**: Keep first-run CLI feedback immediate; prompt rendering, validation, and summary assembly must remain negligible compared with filesystem writes  
**Constraints**: No full-screen TUI; preserve automation-safe non-interactive flags and exit codes; keep output grep-friendly; do not depend on Canon behavior; keep assistant scaffolding bounded to the repository root; bump the release to `0.44.0` with aligned docs  
**Scale/Scope**: One bounded CLI UX slice touching the init/doctor/help path, related docs, semantic summaries, version metadata, and regression tests for first-run flows

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: This feature improves the first operator step that gates bounded delivery work; if init and doctor are opaque, users do not reach the execution path at all. See Summary and Technical Context.
- **PASS** Delivery-first scope: The work stays on the execution entrypoint (`init`, `doctor`, route guidance, overwrite safety) rather than decorative branding or general CLI polish. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains session-native (`goal -> plan -> run -> status -> next -> inspect`), while `.boundline/execution.json` stays the explicit compatibility/bootstrap path surfaced by `init`. See Summary and Constraints.
- **PASS** Bounded execution: Start conditions are an operator invoking `init` or `doctor`; terminal conditions are success, explicit preview, or actionable validation failure; retries stay bounded to explicit user reruns. See Technical Context and quickstart scenarios.
- **PASS** Stateful execution: `init` reads workspace state plus requested/guided inputs, writes `.boundline` config and assistant setup state, and reports the resulting effective route/assistant summary. See Technical Context and data model.
- **PASS** Mutable planning: This slice does not change orchestrator replanning semantics; it only clarifies how init chooses, seeds, and reports the initial route state that later planning reads. See Summary and research decisions.
- **PASS** Sequential-first design: All interactive and validation behavior remains one prompt or one CLI invocation at a time with no background work. See Technical Context.
- **PASS** Tool-agent symmetry: The feature keeps explicit operator answers, explicit validation, and explicit follow-up commands rather than hidden fallback logic. See research decisions and contract.
- **PASS** Observability and explicit intelligence: Prompt defaults, seeded routes, overwrite previews, rich/plain summaries, and corrective errors remain visible on CLI surfaces and in tests. See Observability Surface and contract.
- **PASS** Non-goals and external separation: No Canon dependency, no councils, no background workers, no deployment pipeline work, and no new UI runtime surface beyond the existing CLI. This is bounded delivery-surface clarity, not a separate UI product. See Constraints and Scope Boundaries in spec.
- **PASS** Minimal slice: The smallest independently valuable capability is a first-run operator completing init and understanding the result from the CLI alone, with tests guarding the route/help/error surfaces. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/044-cli-init-ux/
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
│   ├── assistant_assets.rs
│   ├── diagnostics.rs
│   └── init.rs
├── domain/
└── lib.rs

crates/
└── boundline-cli/
    └── src/cli.rs

tests/
├── contract.rs
├── integration.rs
└── integration/
    └── init_bootstrap_flow.rs
```

**Structure Decision**: Keep the slice inside the existing CLI surfaces and test
aggregators. Add at most one focused helper module for embedded assistant assets
and keep all UX behavior wired through the existing `init` and `doctor` modules
instead of introducing new top-level crates or runtime surfaces.

## Complexity Tracking

No constitution violations require special justification for this slice.
