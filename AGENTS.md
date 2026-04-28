# synod Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-28

## Active Technologies
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `clap` 4.x for a stable subcommand-based CLI surface (002-developer-ux-orchestrator)
- In-memory task state during execution and local file-backed traces under `<workspace>/.synod/traces/` through the existing trace store (002-developer-ux-orchestrator)
- Rust 1.95.0, edition 2024 for the existing CLI backend plus repository-managed Markdown prompt assets for assistant command packs + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice (003-assistant-command-packs)
- Repository-stored assistant asset files under `assistant/` and existing workspace-local traces under `<workspace>/.synod/traces/` for status and inspection backends (003-assistant-command-packs)
- Rust 1.95.0, edition 2024 for the existing CLI and orchestrator backend + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice (004-session-model-unification)
- Workspace-local JSON session record at `<workspace>/.synod/session.json` plus the existing file-backed traces under `<workspace>/.synod/traces/` (004-session-model-unification)
- Workspace-local JSON session record at `<workspace>/.synod/session.json` plus persisted execution traces under `<workspace>/.synod/traces/` (005-delivery-flows)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library process and filesystem APIs; no new runtime dependencies for the initial execution-engine slice (006-execution-engine)
- Workspace-local JSON session record at `<workspace>/.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, and workspace execution manifests under `<workspace>/.synod/execution.json` with legacy fallback to `<workspace>/.synod/fixture.json` (006-execution-engine)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library collections; no new runtime dependencies for the initial review slice (007-multi-agent-review)
- Workspace-local JSON session record at `<workspace>/.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, and workspace execution manifests under `<workspace>/.synod/execution.json` extended with bounded review configuration (007-multi-agent-review)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies for the initial adaptive slice (008-adaptive-execution-engine)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and process APIs; no new runtime dependencies for the initial governance slice (009-canon-governance-adapter)
- Workspace-local JSON session record at `<workspace>/.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, workspace execution manifest at `<workspace>/.synod/execution.json`, and optional Canon-managed governed artifacts under `<workspace>/.canon/` when the Canon runtime is selected (009-canon-governance-adapter)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies for the first human-input slice (010-human-brief-ingestion)
- Workspace-local `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, existing `<workspace>/.synod/execution.json` with legacy fallback to `<workspace>/.synod/fixture.json` for advanced automation, and optional Canon-managed artifacts under `<workspace>/.canon/` when governed execution is selected (010-human-brief-ingestion)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `toml` for human-editable config serialization; no additional runtime abstraction crates for the first slice (011-init-model-routing)
- Workspace-local `.synod/execution.json`, `.synod/session.json`, `.synod/traces/`, new workspace-local `.synod/config.toml`, and new user-scoped global config at `$XDG_CONFIG_HOME/synod/config.toml` with fallback to `$HOME/.config/synod/config.toml` on macOS/Linux developer machines (011-init-model-routing)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) plus Rust standard library path and filesystem APIs; no new runtime dependencies for the first clustered slice (012-multi-workspace)
- Workspace-local `.synod/session.json` and `.synod/traces/` remain authoritative per repository, existing workspace `.synod/config.toml` and user-global config remain in place, and a new primary-workspace `.synod/cluster.toml` stores cluster membership and cluster-scoped defaults (012-multi-workspace)

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
- 012-multi-workspace: Added Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) plus Rust standard library path and filesystem APIs; no new runtime dependencies for the first clustered slice
- 011-init-model-routing: Added Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `toml` for human-editable config serialization; no additional runtime abstraction crates for the first slice
- 010-human-brief-ingestion: Added Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies for the first human-input slice


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
