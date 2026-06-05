# boundline Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-06-05

## Active Technologies
- Rust 1.96.0, edition 2024 + existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled SQLite support), existing workspace crates (`boundline-core`, `boundline-adapters`, `boundline-cli`), and one optional trusted `sqlite-vec` extension-loading path for local vector tables (065-activate-sqlite-vec)
- existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and `.boundline/context-intelligence/retrieval-index.sqlite3`, extended with a companion `.boundline/context-intelligence/manifest.json`, managed `.gitignore` entries, and vector-backed semantic tables inside the same derived SQLite store (065-activate-sqlite-vec)
- Rust 1.96.0, edition 2024 across the Boundline workspace, the sibling template repo, and the sibling Speckit adapter repo for the initial compatibility line + existing workspace crates and dependencies (`clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, `boundline-core`, `boundline-adapters`, `boundline-cli`) plus a shared framework-adapter protocol surface owned by `boundline-adapters` and consumed by sibling repos through versioned git-tag dependencies rather than committed path-based copies (066-agentic-framework-integration)
- workspace-local `.boundline/config.toml`, `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, and `.boundline/workflows.toml`, extended with an optional adapter selection block and adapter audit fields, while the sibling template and Speckit repos persist only their own Cargo manifests, README docs, and protocol fixtures (066-agentic-framework-integration)
- Rust 1.96.0, edition 2024 + existing workspace crates and dependencies only; no new runtime dependency planned (067-plan-quality-contract)
- existing workspace-local session and trace files, extended with additive plan-quality fields and trace-visible projections (067-plan-quality-contract)
- Rust 1.96.0, edition 2024 + Existing workspace crates and dependencies only; (069-plan-analysis-contract)
- Existing workspace-local session and trace files plus governed (069-plan-analysis-contract)
- Existing workspace-local `.boundline/session.json`, (070-large-codebase-context-substrate)
- Existing workspace-local `.boundline/config.toml`, (071-capability-provider-protocol)

- Rust 1.96.0, edition 2024 + `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite`, `dialoguer`
- Workspace-local config and traces: `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, `.boundline/execution.json`, `.boundline/workflows.toml`
- Repository-managed assets: `assistant/`, `distribution/`, `tech-docs/`, `specs/`
- Local SQLite + FTS5 retrieval index for semantic context (no remote embedding services yet)

## Project Structure

```text
src/
tests/
```

## Commands

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo nextest run`
- `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- `cargo deny check licenses advisories bans sources`
- Patch-coverage helpers live under `scripts/common/coverage/`; prefer `intersect_patch_coverage.py` when the question is about uncovered diff lines rather than full-file coverage.

## Code Style

Rust 1.96.0, edition 2024: Follow standard conventions

## Specs rules
Crate versioning follows Semantic Versioning.
Before 1.0.0, breaking changes MAY occur in minor versions.

## Recent Changes
- 071-capability-provider-protocol: Added Rust 1.96.0, edition 2024 + Existing workspace crates and dependencies only;
- 070-large-codebase-context-substrate: Added Rust 1.96.0, edition 2024 + Existing workspace crates and dependencies only;
- 069-plan-analysis-contract: Added Rust 1.96.0, edition 2024 + Existing workspace crates and dependencies only;


<!-- MANUAL ADDITIONS START -->
## Rust Language Rules

- AI-visible Rust language rules live in
	`.agents/skills/boundline-shared/references/rust-language-rules.md`.
- Rust code outside `main.rs` MUST NOT introduce panic-prone control flow
	such as `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`,
	`unreachable!`, or assert-family runtime guards; this applies to
	`#[cfg(test)]` modules and files under `tests/` too, and failures must use
	explicit error propagation instead.
- Stable serialized or deserialized shapes in Rust code outside `main.rs`,
	`#[cfg(test)]`, and files under `tests/` MUST use typed `struct` or
	`enum` models with `serde` derives instead of ad hoc `serde_json::Map`
	assembly, repeated raw field-name strings, or stable `json!` object
	construction.

## Clean Code & Modularity (Strict Enforcement)

- **NO GIGANTIC FILES**: Do not dump all logic into a single massive file. If a module grows complex, extract helpers, algorithms, and state transitions into private submodules (`pub(crate)`).
- **APPLY DESIGN PATTERNS**: Do not use monolithic match statements or procedural god-functions. Extract responsibilities using appropriate design patterns (e.g. Builder, Strategy, Dependency Injection). Keep business logic strictly isolated from I/O and HTTP/CLI transport boundaries.
- **ZERO MAGIC STRINGS/NUMBERS**: You MUST NOT use magic strings or magic numbers in domain logic, protocol handling, persistence, configuration, CLI contracts, timeouts, retry limits, or serialization paths. Extract them into named `const` items or typed `enum`s/newtypes owned by the relevant module or type.
- **EXTRACT HELPERS PROACTIVELY**: Aim for <50 lines per function. If you need a comment to explain the middle of a function, extract that block into a well-named helper function.
- **NO DEAD CODE**: Remove all commented-out code, unused variables, and unreachable branches immediately. `git` remembers.
- **WHY NOT WHAT**: Documentation and comments must explain the *why*, business constraints, and invariants, not narrate the *what*.
- **COMPREHENSIVE DOCUMENTATION**: Every folder/module MUST have a module-level doc comment (e.g. `//!` in `mod.rs` or `<module_name>.rs`) explaining its purpose, and these docs must be kept up to date. Furthermore, all structs, public functions, enums, and constants MUST have clear and up-to-date doc comments (`///`).
- **LOGGING & OUTPUT BOUNDARIES**: Log at major state-transition decision points using structured `tracing` spans/events. Always include reproducible context (IDs) but NEVER log secrets, tokens, or PII. Maintain strict separation between presentation and core logic: use `println!` or `eprintln!` ONLY in presentation layers (e.g., `cli.rs`, `init.rs`). For orchestrator, core logic, and adapters, NEVER use `println!`. User-facing messages must be propagated up to the CLI layer via return values (e.g., `Result<T, Error>`).
- **CONCURRENCY**: Avoid `Arc<Mutex<T>>` lock-contention. Prefer message-passing (channels) or immutable data snapshots to share state across async boundaries.

## Pre-Commit & Code Quality

- **CLIPPY**: After any code modification or run, you MUST check and fix any `clippy` issues by running `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **CARGO FMT**: After any code modification or run, you MUST run `cargo fmt` to ensure the codebase remains correctly formatted.

## Repo Safety Rules

- NEVER save fully qualified paths in any file, but use relative paths and only related to this git repo. Other repos must be referenced by url (or if working locally by git project name).
- NEVER run `boundline` CLI commands against this repository root as a working
	workspace. Doing so writes workspace-local `.boundline/` session state,
	pollutes tracked repo history, and can dirty the developer worktree. Use a
	temporary fixture workspace, isolated temp repo, or explicit test harness
	instead.
<!-- MANUAL ADDITIONS END -->
