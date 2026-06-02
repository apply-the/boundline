# Implementation Plan: Guidance Catalog And Guardian Rule Packs

**Branch**: `055-guidance-catalog-packs` | **Date**: 2026-05-15 | **Spec**: [specs/055-guidance-catalog-packs/spec.md](specs/055-guidance-catalog-packs/spec.md)
**Input**: Feature specification from `/specs/055-guidance-catalog-packs/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a typed guidance-catalog and guardian-rule-pack layer so Boundline can discover pack manifests, validate catalog manifests and indexes, classify entries by canonical pillars plus authority metadata, standardize guidance-strength and guardian-disposition vocabulary for catalog-backed content, and expose loaded, skipped, and invalid entries through existing runtime traces and inspect surfaces. The first slice keeps guardian execution inside S2.1, introduces typed catalog models and validation helpers, updates the runtime consumer surfaces needed to accept the catalog vocabulary, adds repo-managed reference packs plus workspace-local pack discovery, documents Canon-promotion compatibility without implementing Canon publication, and closes with focused tests, formatting, clippy-clean output, and 95% coverage on modified or new Rust files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, and Rust standard-library filesystem, path, collections, and process APIs; no new runtime dependencies planned for this slice  
**Storage**: Repository-managed reference catalog packs under `assistant/packs/`, workspace-local catalog packs under `.boundline/packs/`, existing workspace-local `.boundline/session.json` and `.boundline/traces/`, and optional Canon-governed repo-visible standards consumed through existing project-memory and governed-artifact surfaces  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests for catalog parsing, validation, resolution projection, and CLI visibility, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features` when feasible, and modified-file coverage validation at 95% or higher  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI and library workspace with repo-managed assistant assets and persisted local runtime state  
**Execution Model**: Sequential catalog discovery and validation before plan-time or phase-time guidance resolution; every pack load ends with a loaded, skipped, warning, or error outcome, with no background scanning or hidden retries  
**Observability Surface**: Persisted goal-plan state, execution traces, validation findings, and `status`, `next`, and `inspect` projection surfaces that must expose loaded packs, skipped packs, authority source metadata, index decisions, and invalid entry findings  
**Performance Goals**: Maintainers should be able to identify why a catalog entry loaded, skipped, failed validation, or lost precedence from normal runtime surfaces in under 5 minutes, and catalog validation must not materially degrade a normal planning CLI round-trip  
**Constraints**: S2.1 remains the execution owner for guidance and guardians, but this slice may update the runtime consumer models needed to accept canonical catalog strength and disposition vocabulary; no new routing slots, no publishing registry, no remote package service, no hidden fallback behavior, no panic-based runtime errors outside tests, and the feature must include the required provider-doc audit plus final validation for docs, format, clippy, tests, and 95% modified-file coverage  
**Scale/Scope**: One active workspace or bounded target at a time, a bounded set of installed catalog packs per workspace, and an initial reference pack limited to core clean-code, language, framework, testing, architecture, security, domain, and operations metadata plus schemas and examples

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded engineering delivery by making guidance content packages explicit, typed, inspectable, and safe to resolve before planning or bounded execution. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice prioritizes pack discovery, validation, trace visibility, and runtime-consumable metadata ahead of distribution polish or packaging automation. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`, with catalog packs feeding the same runtime surfaces rather than introducing a parallel workflow. See Execution Model and Observability Surface.
- **PASS** Bounded execution: Catalog discovery and validation run at explicit planning or phase-entry points with clear loaded, skipped, warning, or error terminal states and no hidden background work. See Execution Model and Constraints.
- **PASS** Stateful execution: Loaded pack metadata, validation findings, and precedence outcomes are persisted in existing session and trace surfaces rather than recomputed opaquely on every read-side command. See Storage and Observability Surface.
- **PASS** Mutable planning: A replan may incorporate newly added or removed catalog packs while preserving the previously persisted load story until an explicit bounded replan occurs. See Summary and Execution Model.
- **PASS** Sequential-first design: One catalog discovery path and one ordered validation pass remain active per bounded planning or phase-entry point. See Execution Model and Scale/Scope.
- **PASS** Tool-agent symmetry: Validation and runtime consumption remain explicit through typed models, contract tests, and CLI projection rather than hidden markdown conventions. See Summary and Observability Surface.
- **PASS** Observability and explicit intelligence: Session and trace surfaces expose loaded packs, skipped entries, authority source metadata, validation findings, and precedence decisions. See Observability Surface and Summary.
- **PASS** Catalog currency: The feature keeps provider-model routing out of functional scope, but still reuses the repository-required provider-doc audit pattern and records an explicit no-change rationale in `research.md`. See Constraints and Phase 0 notes in `tasks.md`.
- **PASS** Non-goals and external separation: The plan does not depend on Canon runtime control flow and does not introduce councils, voting, new routing slots, distributed execution, UI work, or deployment pipelines. See Constraints and Summary.
- **PASS** Minimal slice: The smallest independently valuable capability is one typed catalog-manifest plus index validation and trace-visible resolution story that S2.1 can safely consume. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/055-guidance-catalog-packs/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── catalog-manifest-contract.md
│   ├── guidance-index-contract.md
│   └── guardian-index-contract.md
├── reference/
└── tasks.md
```

### Source Code (repository root)

```text
assistant/
└── packs/

src/
├── cli/
│   ├── inspect.rs
│   └── output.rs
├── domain/
│   ├── goal_plan.rs
│   ├── guidance.rs
│   ├── guidance_catalog.rs
│   └── trace.rs
├── orchestrator/
│   ├── guidance_catalog_runtime.rs
│   ├── guidance_runtime.rs
│   └── session_runtime.rs
├── domain.rs
├── orchestrator.rs
└── lib.rs

tests/
├── contract/
├── integration/
└── unit/

tech-docs/
├── architecture.md
├── configuration.md
└── getting-started.md

README.md
CHANGELOG.md
Cargo.toml
```

**Structure Decision**: Keep the slice inside the existing guidance runtime, trace, and CLI projection surfaces. Introduce one small typed domain model for catalog manifests and entries plus one orchestrator helper for discovery and validation, update the existing guidance-domain consumer where the canonical catalog vocabulary crosses into runtime findings, and store built-in reference packs under `assistant/packs/` so the feature remains local-first and independently testable without inventing a registry service or new top-level runtime.

## Complexity Tracking

No constitution violations are expected for this slice.