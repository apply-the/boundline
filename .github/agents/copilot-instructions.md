# boundline Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-05-30

## Active Technologies
- Rust 1.96.0, edition 2024 + existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled SQLite support), existing workspace crates (`boundline-core`, `boundline-adapters`, `boundline-cli`), and one optional trusted `sqlite-vec` extension-loading path for local vector tables (065-activate-sqlite-vec)
- existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and `.boundline/context-intelligence/retrieval-index.sqlite3`, extended with a companion `.boundline/context-intelligence/manifest.json`, managed `.gitignore` entries, and vector-backed semantic tables inside the same derived SQLite store (065-activate-sqlite-vec)

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
- 065-activate-sqlite-vec: Added Rust 1.96.0, edition 2024 + existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` with bundled SQLite support), existing workspace crates (`boundline-core`, `boundline-adapters`, `boundline-cli`), and one optional trusted `sqlite-vec` extension-loading path for local vector tables

- 045-chat-first-runtime: Added Rust 1.96.0, edition 2024 + Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library terminal and filesystem APIs; no new runtime dependencies for the first slice

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
