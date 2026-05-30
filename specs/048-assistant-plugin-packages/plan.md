# Implementation Plan: Assistant Plugin Packages

**Branch**: `048-assistant-plugin-packages` | **Date**: 2026-05-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/048-assistant-plugin-packages/spec.md`

## Summary

Add repository-local host package surfaces that make Boundline discoverable from Claude Code, Codex, Cursor, and Copilot-style prompt packs while preserving the session-native CLI/runtime as the source of truth. The implementation upgrades Boundline to `0.49.0` first, adds shared plugin metadata and command definitions, creates host package manifests and command bindings, documents the chat-to-CLI mapping, and validates JSON, required fields, paths, command coverage, version alignment, and prohibited positioning through focused Rust tests plus a shell wrapper.

## Technical Context

**Language/Version**: Rust 1.96.0 workspace, edition 2024, plus JSON, Markdown, Bash, and SVG repository assets  
**Primary Dependencies**: Existing workspace dependencies only (`serde_json` and `toml` already available for validation); no new external crates planned  
**Storage**: Repository files only: hidden host package folders, shared assistant metadata under `assistant/`, docs under `docs/`, validation script under `scripts/`, and Spec Kit artifacts under `specs/048-assistant-plugin-packages/`  
**Testing**: `cargo test --test assistant_plugin_packages`, `bash scripts/validate-assistant-plugins.sh`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`, and touched-file coverage with `cargo llvm-cov`  
**Target Platform**: Local developer repositories and CI on macOS/Linux/Windows-friendly metadata; validation script targets Unix-like shells used by the repo scripts  
**Project Type**: Rust CLI workspace with repository-managed assistant command packs and local runtime state under `.boundline/`  
**Execution Model**: Sequential package discovery and command guidance; host commands route users into `goal -> plan -> run -> status -> inspect` and recovery via CLI-reported state/next command  
**Observability Surface**: Host manifests, shared command definition JSON, starter prompts, docs, README mapping table, validation test output, and `validation-report.md` closeout evidence  
**Performance Goals**: Package validation should remain a lightweight test target and should not invoke live Boundline sessions or provider APIs  
**Constraints**: Version bump is the first implementation step; `.boundline/session.json` remains authoritative; no divergent host behavior; no invented Copilot plugin format; no runtime redesign; no user hand-edited manifests for normal operation; final touched-Rust-file coverage must be at least 95%  
**Scale/Scope**: One feature slice with three installable host package folders, one Copilot prompt-pack metadata surface, shared metadata and command definitions, two new assistant command assets (`recover`, `govern`) per command pack, one validation module, one test target, one validation script, docs, README, and release/version metadata updates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature improves delivery by making chat surfaces enter the same bounded Boundline runtime instead of using ad hoc CLI snippets. See Summary.
- **PASS** Delivery-first scope: Work is package discovery, command mapping, validation, and docs before any polish. No runtime UX or unrelated provider work is introduced.
- **PASS** Primary workflow: The primary operator path remains session-native (`goal -> plan -> run -> status -> next -> inspect`). Copilot is an explicit prompt-pack compatibility surface.
- **PASS** Bounded execution: Host commands do not run loops; they call or guide one CLI command at a time and preserve CLI-reported terminal, blocked, failed, exhausted, or clarification-required states.
- **PASS** Stateful execution: `.boundline/session.json` remains authoritative, and commands must report current runtime state and next action from the CLI output.
- **PASS** Mutable planning: Chat packaging preserves `plan`, `plan --confirm`, `run`, status, and inspect surfaces without adding hidden planning.
- **PASS** Sequential-first design: Host packages expose sequential commands and do not introduce background workers, parallelism, or hidden fan-out.
- **PASS** Tool-agent symmetry: Each command binding records a concrete CLI surface or explicit chat-only guidance, so reasoning and action stay visible.
- **PASS** Observability and explicit intelligence: Validation outputs, shared JSON command definitions, docs, and README tables make package claims inspectable.
- **PASS** Catalog currency: Public provider docs were checked on 2026-05-11; feature `047-catalog-voting-inputs` already refreshed the catalog and this slice records a no-entry-change result in [research.md](./research.md).
- **PASS** Non-goals and external separation: Canon is conditional downstream governance only; host packages do not depend on Canon to function and do not expose Canon as the default delivery flow.
- **PASS** Minimal slice: The smallest independently valuable capability is installable/discoverable host packaging plus validation. Runtime redesign, UI, deployment, and provider-routing work remain out of scope.

## Project Structure

### Documentation (this feature)

```text
specs/048-assistant-plugin-packages/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── coherence-review.md
├── validation-report.md
├── contracts/
│   └── assistant-plugin-package-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
CHANGELOG.md
ROADMAP.md
README.md
AGENTS.md
.claude-plugin/
├── manifest.json
└── commands.json
.codex-plugin/
└── plugin.json
.cursor-plugin/
├── manifest.json
└── commands.json
.copilot-prompts/
├── README.md
└── pack.json
assistant/
├── README.md
├── plugin-metadata.json
├── assets/
│   ├── boundline-plugin-icon.svg
│   └── boundline-plugin-logo.svg
├── commands/
│   └── session-workflow.json
└── prompts/
    ├── starter-prompts.md
    └── copilot-command-pack.md
assistant/claude/commands/
├── boundline-govern.md
└── boundline-recover.md
assistant/codex/commands/
├── boundline-govern.md
└── boundline-recover.md
assistant/copilot/prompts/
├── boundline-govern.prompt.md
└── boundline-recover.prompt.md
docs/guides/
└── assistant-plugin-packages.md
distribution/
├── channel-metadata.toml
├── homebrew/Formula/boundline.rb
└── winget/manifests/a/ApplyThe/Boundline/0.49.0/
scripts/
└── validate-assistant-plugins.sh
src/
├── assistant_plugin_validation.rs
└── lib.rs
tests/
└── assistant_plugin_packages.rs
```

**Structure Decision**: Keep host package folders at the repository root because the requested package shape names those folders directly. Keep shared metadata, command definitions, prompts, and assets under `assistant/` so host manifests reference one Boundline-owned source rather than duplicating behavior. Add one small Rust validation module because validation must be unit-testable, reusable from the dedicated test target, and measurable for touched-file coverage.

## Complexity Tracking

No constitution deviations are required for this feature.
