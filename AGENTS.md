# synod Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-05-02

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
- Rust 1.95.0, edition 2024 + `clap` 4.x, `serde` 1.x, `serde_json` 1.x, `thiserror` 2.x, `tracing` 0.1, `uuid` 1.x, `toml` 0.8 (013-session-native-orchestrator)
- workspace-local JSON files (`.synod/session.json`, `.synod/traces/`, `.synod/execution.json`, `.synod/config.toml`) (013-session-native-orchestrator)
- Rust 1.95.0, edition 2024 + `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, Rust standard library filesystem and process APIs (014-native-loop-integration)
- Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts (014-native-loop-integration)
- Rust 1.95.0, edition 2024 + `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs (015-runtime-refoundation)
- Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout (015-runtime-refoundation)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice (016-session-native-surface-unification)
- Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, and updated repository docs and assistant assets (016-session-native-surface-unification)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; external Canon CLI compatibility target updated to `0.25.0`; no new runtime dependencies planned for the first slice (017-canon-governance-expansion)
- Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, and repository docs plus assistant assets (017-canon-governance-expansion)
- Workspace-local `.synod/workflows.toml`, `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout (018-workflow-layer)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice (019-workflow-follow-through)
- Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, and release-aligned repository docs plus assistant assets (020-governed-stage-depth)
- Workspace-local `.synod/execution.json`, `.synod/session.json`, `.synod/traces/`, and release-aligned repository docs plus assistant assets (021-adaptive-repair-depth)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and path APIs; no new runtime dependencies planned for this slice (022-session-compatibility-continuity)
- Existing workspace-local `.synod/session.json` and `.synod/traces/` remain authoritative; no new persistence surface is planned unless research proves existing state cannot express continuity safely (022-session-compatibility-continuity)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice (024-unify-route-summaries)
- Workspace-local `.synod/session.json`, `.synod/traces/`, `.synod/execution.json`, `.synod/config.toml`, optional `.synod/workflows.toml`, and release-aligned repository docs plus assistant assets (024-unify-route-summaries)
- Workspace-local `.synod/cluster.toml`, `.synod/session.json`, `.synod/traces/`, `.synod/execution.json`, `.synod/config.toml`, optional `.synod/workflows.toml`, and release-aligned repository docs plus assistant assets (025-multi-workspace-delivery)
- Workspace-local `.synod/session.json`, `.synod/traces/`, task-context state embedded in persisted session tasks, optional cluster projection in primary-workspace session state, and release-aligned repository docs plus assistant assets (026-goal-constraint-modeling)
- Workspace-local `.synod/config.toml`, `.synod/cluster.toml`, `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, and repository-managed assistant asset files under `assistant/` (027-routing-assistant-decoupling)
- Workspace-local `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json`, optional `.synod/workflows.toml`, optional cluster state under `.synod/cluster.toml`, and repository-managed assistant assets under `assistant/` (028-decision-followthrough)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and process APIs; no new runtime dependencies planned for this slice (030-native-direct-run)
- Workspace-local `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json` for explicit compatibility execution, optional `.synod/workflows.toml`, optional cluster state under `.synod/cluster.toml`, and repository-managed assistant assets under `assistant/` (030-native-direct-run)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice (031-canon-delivery-loop)
- Workspace-local `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json`, optional `.synod/workflows.toml`, optional cluster state under `.synod/cluster.toml`, optional `.canon/` governed artifacts, and updated repository docs plus assistant assets (031-canon-delivery-loop)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice (032-workflow-surface-closure)
- Workspace-local `.synod/workflows.toml`, `.synod/config.toml`, `.synod/session.json`, persisted traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json` for explicit compatibility follow-up, optional `.canon/` artifacts, and repository-managed assistant assets under `assistant/` (032-workflow-surface-closure)

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
- `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- `cargo deny check licenses advisories bans sources`

## Code Style

Rust 1.95.0, edition 2024: Follow standard conventions

## Specs rules
Crate versioning follows Semantic Versioning.
Before 1.0.0, breaking changes MAY occur in minor versions.

## Recent Changes
- 032-workflow-surface-closure: Added Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice
- 031-canon-delivery-loop: Added credible delivery-completion gating so bounded `bug-fix` and `change` work only succeeds with material diff and passed validation evidence
- 031-canon-delivery-loop: Added Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
