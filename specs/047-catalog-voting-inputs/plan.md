# Implementation Plan: Catalog Currency, Independent Voting, and File-Backed Inputs

**Branch**: `047-catalog-voting-inputs` | **Date**: 2026-05-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/047-catalog-voting-inputs/spec.md`

## Summary

Refresh the bundled assistant model catalog from current public provider
documentation, teach authored-input normalization to treat a single Markdown
path or an ordered Markdown-path array as file-backed input instead of literal
goal text, and require review councils to prove distinct effective review routes
before a vote can be counted. The smallest valuable slice stays inside the
existing Rust CLI/runtime, reuses the current bundled catalog and routing
projection surfaces, and records the resulting route or failure evidence in the
same inspectable task and review state that Boundline already persists.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing workspace runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library filesystem, path, and collections APIs; no new runtime dependencies required for this slice  
**Storage**: Repository-managed bundled catalog at `assistant/catalog/model-catalog.toml`, workspace-local `.boundline/config.toml` for effective routing inputs, workspace-local `.boundline/execution.json`, task/session state in `.boundline/session.json`, and persisted traces under `.boundline/traces/`  
**Testing**: Focused unit coverage in `src/domain/brief.rs` and `src/domain/review.rs`, fixture-runtime coverage in `src/fixture.rs`, plus targeted `cargo test -p boundline-core ...`, `cargo test -p boundline-adapters ...`, `cargo test --no-run --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo fmt --check`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Multi-crate Rust CLI with shared root source files wired into `boundline-core`, `boundline-adapters`, and `boundline-cli`  
**Execution Model**: Sequential session-native lifecycle (`goal -> plan -> run -> status -> next -> inspect`) plus explicit compatibility capture/run paths; review steps remain one reviewer at a time followed by one bounded vote resolver  
**Observability Surface**: Bundled catalog metadata, authored-input provenance and deduplication labels, routing projection in task state, persisted review participants and vote resolution, explicit terminal error codes for invalid input or non-independent councils, and existing inspect/status/trace surfaces  
**Performance Goals**: Keep catalog usage offline at runtime, keep authored-input normalization interactive for up to 10 Markdown sources, and keep review-route independence checks constant-time over the bounded council size  
**Constraints**: No live provider discovery during normal CLI use, no new top-level runtime surface, no binary attachments, no unbounded review councils, no hidden retries, and no dilution of the curated route-capable catalog with non-route product families  
**Scale/Scope**: One workspace task at a time, one authored-input bundle with up to 10 Markdown sources, one bounded review council per task, and curated bundled models for Copilot, Claude, Codex, and Gemini only

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves the first delivery touchpoints that can otherwise misroute or overstate confidence: route selection, task capture, and review voting. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan changes catalog currency, authored-input normalization, and vote correctness before any UX or documentation polish. See Summary and Project Structure.
- **PASS** Primary workflow: The primary path remains session-native (`goal -> plan -> run -> status -> next -> inspect`), while compatibility behavior stays explicit in the shared capture/run input normalization path. See Technical Context and research decisions.
- **PASS** Bounded execution: The feature starts from explicit operator input or config, terminates in accepted input, explicit input-resolution failure, accepted independent vote, or explicit non-independent council failure, and preserves the current bounded review lifecycle. See Technical Context and contracts.
- **PASS** Stateful execution: Routing projection, authored-input provenance, and review participants remain persisted in existing task/session state rather than hidden transient logic. See Storage and data model.
- **PASS** Mutable planning: The plan does not change planning semantics; it improves the authored inputs that seed planning and the review evidence that gates later terminal outcomes. See Summary and research decisions.
- **PASS** Sequential-first design: Capture, normalization, per-reviewer steps, and vote resolution remain one step at a time with no background workers or concurrent councils. See Execution Model.
- **PASS** Tool-agent symmetry: The model catalog, input-source set, and review-route evidence all remain explicit structured inputs or outputs rather than opaque heuristics. See Observability Surface and contracts.
- **PASS** Observability and explicit intelligence: The feature surfaces authored-input source labels, effective reviewer routes, and explicit terminal error codes for collapsed councils. See Observability Surface and contracts.
- **PASS** Catalog currency: Public provider docs were reviewed on 2026-05-10, the bundled catalog was refreshed to include the missing mainstream route-capable models, and the applied delta is recorded in [research.md](./research.md). See also `assistant/catalog/model-catalog.toml`.
- **PASS** Non-goals and external separation: The slice does not add Canon-specific coupling, long-term memory, UI redesign, live discovery, or open-ended councils; it keeps voting bounded inside the existing review slice. See Constraints and research decisions.
- **PASS** Minimal slice: The smallest independently valuable delivery is current bundled models, path-only authored-input shortcuts that resolve into existing brief bundles, and hard rejection of collapsed review councils. See Summary and tasks.

## Project Structure

### Documentation (this feature)

```text
specs/047-catalog-voting-inputs/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── authored-input-shortcuts-contract.md
│   ├── catalog-refresh-contract.md
│   └── review-council-independence-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
assistant/
└── catalog/
    └── model-catalog.toml

src/
├── domain/
│   ├── brief.rs
│   └── review.rs
├── fixture.rs
└── domain/configuration.rs

tests/
└── ...existing contract and integration suites...
```

**Structure Decision**: Keep the feature inside the existing shared Rust source
files that are already re-exported into the workspace crates. The catalog stays
bundled under `assistant/catalog/`, authored-input shortcut handling belongs in
`src/domain/brief.rs`, review-route independence belongs in `src/domain/review.rs`
with the task-state wiring in `src/fixture.rs`, and no new crate or top-level
runtime surface is introduced.

## Complexity Tracking

No constitution violations require justification for this slice.
