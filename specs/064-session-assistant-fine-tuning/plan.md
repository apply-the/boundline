# Implementation Plan: Session, Assistant, and Audit Fine-Tuning

**Branch**: `064-session-assistant-fine-tuning` | **Date**: 2026-05-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/064-session-assistant-fine-tuning/spec.md`

## Summary

This slice consolidates production-ready fine-tuning across five operator-facing surfaces: readable session references, streamlined local install refresh, consistent two-button assistant routing in Copilot prompt commands, session audit attribution clarity, and a dedicated audit-focused inspect surface. The implementation keeps existing runtime authority and safety boundaries intact while improving day-to-day usability, explainability, and recoverability.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024, plus repository-managed shell and Markdown prompt assets  
**Primary Dependencies**: Existing workspace dependencies only (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`)  
**Storage**: Workspace-local `.boundline/session.json`, session-local storage under `.boundline/sessions/<session>/`, including `.boundline/sessions/<session>/audit/events.jsonl` and `.boundline/sessions/<session>/audit/cursor.json`, plus repository-managed prompt assets under `assistant/`  
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and representative `cargo test` coverage for touched behavior  
**Target Platform**: macOS maintainer workflow plus standard Linux CI behavior for Rust and prompt asset checks  
**Project Type**: Rust workspace CLI with assistant command-pack assets

## Constitution Check

- PASS: Session-native runtime remains authoritative; no secondary workflow engine introduced.
- PASS: Changes are bounded and operator-visible; no hidden loops or autonomous retry behavior.
- PASS: Prompt boundaries and allowed command lists are preserved.
- PASS: No Canon contract expansion is required for this slice.

## Project Structure

### Documentation (this feature)

```text
specs/064-session-assistant-fine-tuning/
├── plan.md
├── spec.md
├── tasks.md
└── checklists/
    └── requirements.md
```

### Source Code (affected surfaces)

```text
src/domain/session.rs
src/cli/session.rs
src/cli/govern.rs
src/domain/audit.rs
src/adapters/audit_store.rs
src/orchestrator/session_runtime.rs
src/cli/orchestrate.rs
src/cli/inspect.rs
src/cli/output_trace_summary.rs
src/cli.rs
scripts/install-local.sh
assistant/copilot/prompts/
assistant/*/commands/
```

## Phase Plan

### Phase 1: Session Reference Fine-Tuning

- Align session reference generation to `YYYYMMDD-NNN-slug`.
- Keep normalization constraints and deterministic date handling.
- Reconcile CLI session initialization and governance entry points with the same session reference contract.

### Phase 2: Local Install Flow Fine-Tuning

- Introduce and verify `scripts/install-local.sh` for local maintainer refresh.
- Ensure release build and destination copy path are explicit and repeatable.

### Phase 3: Prompt Routing Fine-Tuning

- Update seven Copilot prompts to standardized two-button routing.
- Preserve resume-command override behavior.
- Preserve allowed follow-up command boundaries.

### Phase 4: Validation and Alignment

- Keep tests and assertions aligned with adjusted runtime semantics.
- Run formatting, linting, and representative tests.
- Confirm prompt text coherence and no contradictory next-step guidance.

### Phase 5: Session Audit Attribution Refinement

- Extend audit actors to preserve mixed reviewer routes and participant route lists.
- Keep inspect-ready audit projections human-readable without losing structured attribution.
- Reuse the same audit vocabulary across runtime, inspect, and assistant-facing projections.

### Phase 6: Audit-First Assistant and Inspect Surfaces

- Expose explicit audit projections on orchestrate NDJSON events for assistant hosts.
- Add `inspect --audit` as a dedicated audit-focused operator surface.
- Update inspect command-pack guidance to prefer `--audit` when the user explicitly asks for audit lineage.

## Complexity Tracking

No constitutional exceptions required for this slice.
