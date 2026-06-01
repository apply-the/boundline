# boundline Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-05-31

## Active Technologies
- Rust 1.96.0, edition 2024 + existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled SQLite support), existing workspace crates (`boundline-core`, `boundline-adapters`, `boundline-cli`), and one optional trusted `sqlite-vec` extension-loading path for local vector tables (065-activate-sqlite-vec)
- existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and `.boundline/context-intelligence/retrieval-index.sqlite3`, extended with a companion `.boundline/context-intelligence/manifest.json`, managed `.gitignore` entries, and vector-backed semantic tables inside the same derived SQLite store (065-activate-sqlite-vec)
- Rust 1.96.0, edition 2024 across the Boundline workspace; + existing workspace crates and dependencies (066-agentic-framework-integration)
- workspace-local `.boundline/config.toml`, `.boundline/session.json`, (066-agentic-framework-integration)
- Rust 1.96.0, edition 2024 across the Boundline workspace, the sibling template repo, and the sibling Speckit adapter repo for the initial compatibility line + existing workspace crates and dependencies (`clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, `boundline-core`, `boundline-adapters`, `boundline-cli`) plus a shared framework-adapter protocol surface owned by `boundline-adapters` and consumed by sibling repos through versioned git-tag dependencies rather than committed path-based copies (066-agentic-framework-integration)
- workspace-local `.boundline/config.toml`, `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, and `.boundline/workflows.toml`, extended with an optional adapter selection block and adapter audit fields, while the sibling template and Speckit repos persist only their own Cargo manifests, README docs, and protocol fixtures (066-agentic-framework-integration)

- Rust 1.96.0, edition 2024 + Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library terminal and filesystem APIs; no new runtime dependencies for the first slice (045-chat-first-runtime)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.96.0, edition 2024: Follow standard conventions

## Recent Changes
- 066-agentic-framework-integration: Added Rust 1.96.0, edition 2024 across the Boundline workspace, the sibling template repo, and the sibling Speckit adapter repo for the initial compatibility line + existing workspace crates and dependencies (`clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, `boundline-core`, `boundline-adapters`, `boundline-cli`) plus a shared framework-adapter protocol surface owned by `boundline-adapters` and consumed by sibling repos through versioned git-tag dependencies rather than committed path-based copies
- 066-agentic-framework-integration: Added Rust 1.96.0, edition 2024 across the Boundline workspace; + existing workspace crates and dependencies
- 065-activate-sqlite-vec: Added Rust 1.96.0, edition 2024 + existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled SQLite support), existing workspace crates (`boundline-core`, `boundline-adapters`, `boundline-cli`), and one optional trusted `sqlite-vec` extension-loading path for local vector tables


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
