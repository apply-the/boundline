# synod Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-25

## Active Technologies
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `clap` 4.x for a stable subcommand-based CLI surface (002-developer-ux-orchestrator)
- In-memory task state during execution and local file-backed traces under `<workspace>/.synod/traces/` through the existing trace store (002-developer-ux-orchestrator)
- Rust 1.95.0, edition 2024 for the existing CLI backend plus repository-managed Markdown prompt assets for assistant command packs + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice (003-assistant-command-packs)
- Repository-stored assistant asset files under `assistant/` and existing workspace-local traces under `<workspace>/.synod/traces/` for status and inspection backends (003-assistant-command-packs)
- Rust 1.95.0, edition 2024 for the existing CLI and orchestrator backend + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice (004-session-model-unification)
- Workspace-local JSON session record at `<workspace>/.synod/session.json` plus the existing file-backed traces under `<workspace>/.synod/traces/` (004-session-model-unification)

- Rust 1.95.0, edition 2024 + Rust standard library plus `serde`, `serde_json`, `thiserror`, `tracing`, and `uuid` for structured state, trace serialization, error handling, instrumentation, and stable identifiers (001-delivery-orchestrator-core)

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
- `cargo deny check licenses advisories bans sources`

## Code Style

Rust 1.95.0, edition 2024: Follow standard conventions

## Specs rules
Crate versioning follows Semantic Versioning.
Before 1.0.0, breaking changes MAY occur in minor versions.

## Recent Changes
- 004-session-model-unification: Added Rust 1.95.0, edition 2024 for the existing CLI and orchestrator backend + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice
- 003-assistant-command-packs: Added Rust 1.95.0, edition 2024 for the existing CLI backend plus repository-managed Markdown prompt assets for assistant command packs + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice
- 002-developer-ux-orchestrator: Added Rust 1.95.0, edition 2024 + Existing runtime dependencies (`serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `clap` 4.x for a stable subcommand-based CLI surface


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
