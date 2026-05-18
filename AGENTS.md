# boundline Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-05-17

## Active Technologies
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `clap` 4.x for a stable subcommand-based CLI surface (002-developer-ux-orchestrator)
- In-memory task state during execution and local file-backed traces under `<workspace>/.boundline/traces/` through the existing trace store (002-developer-ux-orchestrator)
- Rust 1.95.0, edition 2024 for the existing CLI backend plus repository-managed Markdown prompt assets for assistant command packs + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice (003-assistant-command-packs)
- Repository-stored assistant asset files under `assistant/` and existing workspace-local traces under `<workspace>/.boundline/traces/` for status and inspection backends (003-assistant-command-packs)
- Rust 1.95.0, edition 2024 for the existing CLI and orchestrator backend + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice (004-session-model-unification)
- Workspace-local JSON session record at `<workspace>/.boundline/session.json` plus the existing file-backed traces under `<workspace>/.boundline/traces/` (004-session-model-unification)
- Workspace-local JSON session record at `<workspace>/.boundline/session.json` plus persisted execution traces under `<workspace>/.boundline/traces/` (005-delivery-flows)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library process and filesystem APIs; no new runtime dependencies for the initial execution-engine slice (006-execution-engine)
- Workspace-local JSON session record at `<workspace>/.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, and workspace execution manifests under `<workspace>/.boundline/execution.json` with legacy fallback to `<workspace>/.boundline/fixture.json` (006-execution-engine)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library collections; no new runtime dependencies for the initial review slice (007-multi-agent-review)
- Workspace-local JSON session record at `<workspace>/.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, and workspace execution manifests under `<workspace>/.boundline/execution.json` extended with bounded review configuration (007-multi-agent-review)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies for the initial adaptive slice (008-adaptive-execution-engine)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and process APIs; no new runtime dependencies for the initial governance slice (009-canon-governance-adapter)
- Workspace-local JSON session record at `<workspace>/.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, workspace execution manifest at `<workspace>/.boundline/execution.json`, and optional Canon-managed governed artifacts under `<workspace>/.canon/` when the Canon runtime is selected (009-canon-governance-adapter)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies for the first human-input slice (010-human-brief-ingestion)
- Workspace-local `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, existing `<workspace>/.boundline/execution.json` with legacy fallback to `<workspace>/.boundline/fixture.json` for advanced automation, and optional Canon-managed artifacts under `<workspace>/.canon/` when governed execution is selected (010-human-brief-ingestion)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `toml` for human-editable config serialization; no additional runtime abstraction crates for the first slice (011-init-model-routing)
- Workspace-local `.boundline/execution.json`, `.boundline/session.json`, `.boundline/traces/`, new workspace-local `.boundline/config.toml`, and new user-scoped global config at `$XDG_CONFIG_HOME/boundline/config.toml` with fallback to `$HOME/.config/boundline/config.toml` on macOS/Linux developer machines (011-init-model-routing)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) plus Rust standard library path and filesystem APIs; no new runtime dependencies for the first clustered slice (012-multi-workspace)
- Workspace-local `.boundline/session.json` and `.boundline/traces/` remain authoritative per repository, existing workspace `.boundline/config.toml` and user-global config remain in place, and a new primary-workspace `.boundline/cluster.toml` stores cluster membership and cluster-scoped defaults (012-multi-workspace)
- Rust 1.95.0, edition 2024 + `clap` 4.x, `serde` 1.x, `serde_json` 1.x, `thiserror` 2.x, `tracing` 0.1, `uuid` 1.x, `toml` 0.8 (013-session-native-orchestrator)
- workspace-local JSON files (`.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, `.boundline/config.toml`) (013-session-native-orchestrator)
- Rust 1.95.0, edition 2024 + `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, Rust standard library filesystem and process APIs (014-native-loop-integration)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts (014-native-loop-integration)
- Rust 1.95.0, edition 2024 + `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs (015-runtime-refoundation)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout (015-runtime-refoundation)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice (016-session-native-surface-unification)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and updated repository docs and assistant assets (016-session-native-surface-unification)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; external Canon CLI compatibility target updated to `0.25.0`; no new runtime dependencies planned for the first slice (017-canon-governance-expansion)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and repository docs plus assistant assets (017-canon-governance-expansion)
- Workspace-local `.boundline/workflows.toml`, `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout (018-workflow-layer)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice (019-workflow-follow-through)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and release-aligned repository docs plus assistant assets (020-governed-stage-depth)
- Workspace-local `.boundline/execution.json`, `.boundline/session.json`, `.boundline/traces/`, and release-aligned repository docs plus assistant assets (021-adaptive-repair-depth)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and path APIs; no new runtime dependencies planned for this slice (022-session-compatibility-continuity)
- Existing workspace-local `.boundline/session.json` and `.boundline/traces/` remain authoritative; no new persistence surface is planned unless research proves existing state cannot express continuity safely (022-session-compatibility-continuity)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice (024-unify-route-summaries)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, and release-aligned repository docs plus assistant assets (024-unify-route-summaries)
- Workspace-local `.boundline/cluster.toml`, `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, and release-aligned repository docs plus assistant assets (025-multi-workspace-delivery)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, task-context state embedded in persisted session tasks, optional cluster projection in primary-workspace session state, and release-aligned repository docs plus assistant assets (026-goal-constraint-modeling)
- Workspace-local `.boundline/config.toml`, `.boundline/cluster.toml`, `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, and repository-managed assistant asset files under `assistant/` (027-routing-assistant-decoupling)
- Workspace-local `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.boundline/workflows.toml`, optional cluster state under `.boundline/cluster.toml`, and repository-managed assistant assets under `assistant/` (028-decision-followthrough)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and process APIs; no new runtime dependencies planned for this slice (030-native-direct-run)
- Workspace-local `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json` for explicit compatibility execution, optional `.boundline/workflows.toml`, optional cluster state under `.boundline/cluster.toml`, and repository-managed assistant assets under `assistant/` (030-native-direct-run)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice (031-canon-delivery-loop)
- Workspace-local `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.boundline/workflows.toml`, optional cluster state under `.boundline/cluster.toml`, optional `.canon/` governed artifacts, and updated repository docs plus assistant assets (031-canon-delivery-loop)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice (032-workflow-surface-closure)
- Workspace-local `.boundline/workflows.toml`, `.boundline/config.toml`, `.boundline/session.json`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json` for explicit compatibility follow-up, optional `.canon/` artifacts, and repository-managed assistant assets under `assistant/` (032-workflow-surface-closure)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, collections, and process APIs; no new runtime dependencies planned for this slice (033-context-assembly-foundation)
- Workspace-local `.boundline/session.json`, `.boundline/config.toml`, `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and updated repository docs plus assistant assets (033-context-assembly-foundation)
- Workspace-local `.boundline/session.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and repository-managed docs plus assistant assets (034-decision-driven-orchestrator)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice (035-dynamic-planning-flow)
- Workspace-local `.boundline/session.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, task-context state embedded in session tasks, optional `.canon/` governed artifacts, and repository-managed docs plus assistant assets (036-canon-grounded-memory)
- Workspace-local `.boundline/session.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, task-context state embedded in session tasks, and repository-managed docs plus assistant assets (037-bounded-delegation)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice (038-domain-agent-templates)
- Workspace-local `.boundline/config.toml`, cluster-local `.boundline/cluster.toml`, user-global config at `$XDG_CONFIG_HOME/boundline/config.toml` or `$HOME/.config/boundline/config.toml`, persisted session and trace state under `.boundline/session.json` and `.boundline/traces/`, optional `.boundline/execution.json`, and repository docs plus assistant assets (038-domain-agent-templates)
- Rust 1.95.0, edition 2024 for the CLI plus repository-managed shell scripts, YAML manifests, and GitHub Actions workflows for release packaging + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`; no new Rust runtime dependencies planned for the first slice (039-distribution-bundling)
- Existing workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, plus new repository-managed distribution metadata under `distribution/` and release automation in `.github/workflows/` (039-distribution-bundling)
- Workspace-local `.boundline/session.json`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/config.toml`, optional `.boundline/workflows.toml`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and repository-managed docs plus assistant assets (040-context-selection-hardening)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`; no new non-standard runtime dependency is required for checkpoint persistence (041-checkpoint-rewind)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, `.boundline/cluster.toml`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and new workspace-local `.boundline/checkpoints/` manifests plus captured file payloads (041-checkpoint-rewind)
- Rust 1.95.0, edition 2024 + `clap` 4.x, `serde` 1.x, `serde_json` 1.x, `thiserror` 2.x, `tracing` 0.1, `uuid` 1.x, `toml` 0.8; Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies (042-native-canon-cli)
- Workspace-local `.boundline/session.json`, `.boundline/config.toml`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` governed artifacts (042-native-canon-cli)
- Rust 1.95.0, edition 2024 + Existing runtime dependencies `clap` 4.x, `serde` 1.x, `serde_json` 1.x, `thiserror` 2.x, `tracing` 0.1, `uuid` 1.x, `toml` 0.8, plus Rust standard library filesystem, path, collections, and process APIs; no new runtime dependencies planned (043-stack-neutral-init)
- Workspace-local `.boundline/config.toml`, optional `.boundline/execution.json`, `.boundline/session.json`, `.boundline/traces/`, and repository ignore files such as `.gitignore`, `.dockerignore`, `.eslintignore`, `.prettierignore`, `.terraformignore`, and `.helmignore` when bounded hygiene defaults justify them (043-stack-neutral-init)
- Rust 1.95.0, edition 2024 + Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library terminal and filesystem APIs (044-cli-init-ux)
- Workspace-local files under `.boundline/`, repository-local assistant asset files under `assistant/`, and stdout/stderr CLI summaries (044-cli-init-ux)
- Rust 1.95.0, edition 2024 + Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) plus a terminal interaction crate for guided prompts and `indicatif` for spinner/progress feedback in the CLI crate (046-guided-init-tui)
- Existing workspace-local `.boundline/execution.json`, `.boundline/config.toml`, assistant asset files under `assistant/`, and a bundled runtime/model catalog embedded from a repository-managed asset; no new user-writable persistence surface (046-guided-init-tui)
- Rust 1.95.0, edition 2024 + Existing workspace runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library filesystem, path, and collections APIs; no new runtime dependencies required for this slice (047-catalog-voting-inputs)
- Repository-managed bundled catalog at `assistant/catalog/model-catalog.toml`, workspace-local `.boundline/config.toml` for effective routing inputs, workspace-local `.boundline/execution.json`, task/session state in `.boundline/session.json`, and persisted traces under `.boundline/traces/` (047-catalog-voting-inputs)
- Rust 1.95.0 workspace, edition 2024, plus JSON, Markdown, Bash, and SVG repository assets + Existing workspace dependencies only (`serde_json` and `toml` already available for validation); no new external crates planned (048-assistant-plugin-packages)
- Repository files only: hidden host package folders, shared assistant metadata under `assistant/`, docs under `docs/`, validation script under `scripts/`, and Spec Kit artifacts under `specs/048-assistant-plugin-packages/` (048-assistant-plugin-packages)
- Rust 1.95.0 workspace, edition 2024, plus JSON, Markdown, TOML, Bash, and assistant command assets + Existing workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`); external Canon CLI compatibility target `0.45.0`; no new runtime crates planned for the first implementation slice (049-project-scale-delivery-ux)
- Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/checkpoints/`, optional `.canon/` governed packet artifacts, repo-managed assistant package files, docs, and Spec Kit artifacts (049-project-scale-delivery-ux)
- Rust 1.95.0, Edition 2024 + `clap`, `serde`, `serde_json`, `thiserror`, (050-project-memory-delivery-integration)
- Rust 1.95.0, Edition 2024 + `clap`, `dialoguer`, `serde`, `serde_json`, (051-delivery-control-consumer)
- Rust 1.95.0, edition 2024 + Existing workspace dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `toml`, `uuid`, and Rust standard-library collections, filesystem, path, and process APIs; no new runtime dependencies planned for this slice (054-guidance-guardian-capabilities)
- Repository-managed built-in guidance and guardian assets under `assistant/`, workspace-local overrides under `.boundline/guidance/` and `.boundline/guardians/`, existing workspace-local `.boundline/session.json` and `.boundline/traces/`, and optional Canon-governed repo-visible standards discovered through existing project-memory and governed-artifact surfaces (054-guidance-guardian-capabilities)
- Rust 1.95.0, edition 2024 + Existing workspace dependencies `clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, and Rust standard-library collections, filesystem, path, and process APIs; no new runtime dependencies planned for this slice (057-adaptive-governance)
- Workspace-local `.boundline/session.json`, persisted traces under `.boundline/traces/`, optional `.boundline/execution.json` and `.boundline/config.toml`, plus Canon-governed packet metadata already consumed through the governance runtime boundary (057-adaptive-governance)
- Rust 1.95.0, edition 2024 + Existing workspace dependencies `clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, and Rust standard-library filesystem, path, collections, and process APIs, plus one embedded SQLite binding with FTS5 support for the workspace-local retrieval index; no external graph or vector service is required for the first slice (058-advanced-context-intelligence)
- Existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and Canon-promoted project-memory artifacts, plus a workspace-local retrieval index at `.boundline/context-intelligence/retrieval-index.sqlite3` for searchable document and evidence state (058-advanced-context-intelligence)
- Rust 1.95.0, edition 2024 + existing workspace runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`), existing `rusqlite` bundled SQLite support, and one optional `sqlite-vec` integration path for local vector tables; no remote embedding-provider dependency in the first slice (059-semantic-acceleration)
- existing `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and `.boundline/context-intelligence/retrieval-index.sqlite3`, extended with semantic-index metadata and vector-backed chunk tables on the same workspace-local SQLite store (059-semantic-acceleration)
- semantic acceleration is local-only and additive: preserve `semantic_policy_state`, `semantic_capability_state`, `hybrid_outcome`, candidate `match_origin`, and any `rejected_candidate:` lines on `plan`, `status`, and `inspect`; fallback to the V1 local SQLite + FTS5 path must stay explicit when semantic capability is unavailable or degraded (059-semantic-acceleration)
- Rust 1.95.0, edition 2024 (for Boundline runtime that will consume this contract); contract itself is Markdown + JSON schema for tooling + Existing Boundline stack (clap, serde, serde_json, thiserror, tracing, uuid, toml); no new runtime dependencies required for contract definition slice (060-s7-canon-contracts)
- Repository-managed Markdown documentation under `specs/060-s7-canon-contracts/contracts/` plus cross-repo reference to Canon `specs/057-s7-delight-provider/contracts/` (060-s7-canon-contracts)

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
- 060-s7-canon-contracts: Added Rust 1.95.0, edition 2024 (for Boundline runtime that will consume this contract); contract itself is Markdown + JSON schema for tooling + Existing Boundline stack (clap, serde, serde_json, thiserror, tracing, uuid, toml); no new runtime dependencies required for contract definition slice
- 059-semantic-acceleration: Added Rust 1.95.0, edition 2024 + existing workspace runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`), existing `rusqlite` bundled SQLite support, and one optional `sqlite-vec` integration path for local vector tables; no remote embedding-provider dependency in the first slice
- 059-semantic-acceleration: Advanced-context retrieval can now refresh semantic chunks on the shared local index, expand or rerank the V1 candidate set when capability is ready, and surface rejected semantic candidates plus explicit fallback reasons on the normal CLI output path


<!-- MANUAL ADDITIONS START -->
## Rust Language Rules

- AI-visible Rust language rules live in
	`.agents/skills/boundline-shared/references/rust-language-rules.md`.
- Rust code outside `main.rs`, `#[cfg(test)]`, and files under `tests/` MUST
	NOT introduce panic-prone control flow such as `unwrap`, `expect`,
	`panic!`, `todo!`, `unimplemented!`, `unreachable!`, or assert-family
	runtime guards; use explicit error propagation instead.
- Rust code outside `main.rs`, `#[cfg(test)]`, and files under `tests/` MUST
	NOT introduce magic strings or magic numbers in domain logic, protocol
	handling, persistence, configuration, CLI contracts, or serialization
	paths; use named constants or typed enums/newtypes owned by the relevant
	module or type.
- Stable serialized or deserialized shapes in Rust code outside `main.rs`,
	`#[cfg(test)]`, and files under `tests/` MUST use typed `struct` or
	`enum` models with `serde` derives instead of ad hoc `serde_json::Map`
	assembly, repeated raw field-name strings, or stable `json!` object
	construction.
<!-- MANUAL ADDITIONS END -->


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
