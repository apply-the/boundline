# Implementation Plan: Guided Init TUI and Runtime Catalog

**Branch**: `046-guided-init-tui` | **Date**: 2026-05-09 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/046-guided-init-tui/spec.md](/Users/rt/workspace/apply-the/boundline/specs/046-guided-init-tui/spec.md)
**Input**: Feature specification from `/specs/046-guided-init-tui/spec.md`

## Summary

Replace the current line-oriented guided `init` prompts with a bounded terminal
wizard that exposes visible defaults, multi-select assistant surfaces, slot-by-
slot model route editing, a bundled model catalog, and a final confirmation
summary before writing config. The first slice will also add top-level version
output, explicit non-interactive init parity, and progress feedback for time-
consuming init steps while keeping all behavior inside the existing CLI and
workspace-owned config surfaces.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) plus `dialoguer` for guided prompts and `indicatif` for spinner/progress feedback in the CLI crate  
**Storage**: Existing workspace-local `.boundline/execution.json`, `.boundline/config.toml`, repository-managed assistant asset files under `assistant/`, and a bundled catalog asset at `assistant/catalog/model-catalog.toml` compiled into the CLI; no new user-writable persistence surface  
**Testing**: Focused unit tests in `src/cli/init.rs` and adjacent CLI modules, contract tests for CLI flags and summary output under `tests/contract/`, integration tests for bootstrap flows under `tests/integration/`, `cargo test --no-run --all-targets --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo fmt --check`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Multi-crate Rust CLI with repository-managed assistant assets  
**Execution Model**: Sequential command execution with one guided step active at a time; no background workers, hidden retries, or concurrent bootstrap stages  
**Observability Surface**: CLI summaries, preview output, explicit validation errors, visible route/default decisions, bounded spinner or stable progress lines for long steps, per-surface assistant asset status, and the existing generated config and assistant assets  
**Performance Goals**: Version output remains effectively instantaneous; guided validation stays interactive; any init step lasting longer than one second surfaces progress feedback without corrupting terminal state  
**Constraints**: No remote model discovery in v1; no full-screen TUI; no raw escape-sequence leakage; keep non-interactive automation supported; preserve existing workspace output semantics and explicit `--force` preview behavior; show all four route slots during review even when some remain unset  
**Scale/Scope**: One workspace bootstrap at a time, four route slots (`planning`, `implementation`, `verification`, `review`) shown in every review pass, and the existing assistant surfaces (`claude`, `codex`, `copilot`, `gemini`)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice improves the explicit bootstrap path required before Boundline can perform bounded delivery work in a repository. A usable `init` flow is delivery-enabling setup, not generic terminal ornamentation. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan focuses on workspace bootstrap reliability, route selection, and progress visibility rather than branding or speculative interface work. See Summary and Constraints.
- **PASS** Primary workflow: The main operator workflow after bootstrap remains the existing session-native path (`goal -> plan -> run -> status -> next -> inspect`). This feature only improves the bounded setup surface that precedes that workflow. See Summary and Scope/Constraints.
- **PASS** Bounded execution: Init starts from explicit CLI invocation, terminates in confirmed write, preview stop, cancellation, validation failure, or write failure, and does not add retries or hidden background work. See Execution Model and Observability Surface.
- **PASS** Stateful execution: The wizard accumulates transient in-memory answers for one init run, then writes the existing workspace-owned config and execution files only after final confirmation. See Storage and Summary.
- **PASS** Mutable planning: This slice does not change session planning semantics; it makes bootstrap route choices editable before persistence and keeps route defaults explicit. See Summary and data model.
- **PASS** Sequential-first design: All prompt, validation, and write behavior remains one step at a time in a single CLI invocation. See Execution Model.
- **PASS** Tool-agent symmetry: The feature stays inside explicit CLI command surfaces with user-visible decisions, confirmation, and output rather than hidden heuristics. See Observability Surface and contract.
- **PASS** Observability and explicit intelligence: Default routes, bundled catalog source, custom-route warnings, summary review, and progress feedback remain visible to the operator. See Observability Surface and research decisions.
- **PASS** Non-goals and external separation: The plan does not require Canon beyond existing approval semantics, does not add provider networking, councils, voting, long-term memory, or broader runtime redesign. The UX work is limited to the explicit bootstrap command that Boundline already owns. See Constraints and spec scope boundaries.
- **PASS** Strict non-goals: The slice adds no review councils, no voting rules, no distributed agents, no deployment pipeline work, and no provider abstraction beyond the bounded init routing catalog. See Constraints and research decisions.
- **PASS** Minimal capability slices: The feature is constrained to the smallest bootstrap slice that fixes unusable prompt input while keeping version output, catalog visibility, and automation parity in the same operator journey. See Summary and research decisions.
- **PASS** Real acceptance criteria: The spec and quickstart cover concrete terminal runs, invalid input recovery, non-interactive automation, and long-running feedback scenarios. See spec acceptance scenarios and quickstart.
- **PASS** Failure as a first-class path: Cancellation, invalid custom model input, non-TTY fallback, preview-only stops, and write failures remain explicit terminal outcomes with no hidden retries. See Observability Surface, contract, and quickstart.
- **PASS** Separation from external systems: Core guided init works without Canon discovery, provider APIs, or remote catalog services. External systems only influence explicit operator choices already available in local config. See Constraints and research decisions.
- **PASS** Evolution without premature lock-in: The interaction layer is bounded behind the init command, the catalog is a bundled asset that can change without changing the stored route model, and custom model ids remain available when the curated catalog lags providers. See research decisions and data model.
- **PASS** Done means executable delivery: The slice is complete only when operators can successfully bootstrap a real workspace, recover from at least one invalid-input path, observe long-running progress, and inspect the written outputs without guesswork. See Summary, quickstart, and success criteria.

## Project Structure

### Documentation (this feature)

```text
specs/046-guided-init-tui/
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
│   ├── init.rs
│   └── assistant_assets.rs
├── domain/
│   └── configuration.rs
└── lib.rs

assistant/
├── README.md
├── catalog/
│   └── model-catalog.toml
└── ...existing assistant command assets...

tests/
├── contract/
├── integration/
├── support/
└── unit/
```

**Structure Decision**: Keep the feature inside the existing CLI bootstrap path.
Modify `src/cli.rs` for top-level version and new init flags, keep the wizard,
progress behavior, TTY fallback, assistant-pack reporting, and catalog loading
centered in `src/cli/init.rs`, embed `assistant/catalog/model-catalog.toml`
into the CLI without copying it into workspaces, reuse the existing repository-
managed assistant assets from `assistant/` when selected surfaces are scaffolded
into the target workspace, extend configuration-domain support only where route
or catalog validation needs shared types, and reuse existing integration and
contract test suites rather than introducing a new crate or top-level runtime
surface.

## Complexity Tracking

No constitution violations require special justification for this slice.
