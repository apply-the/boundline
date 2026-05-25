# Implementation Plan: Session and Assistant Fine-Tuning

**Branch**: `064-session-assistant-fine-tuning` | **Date**: 2026-05-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/064-session-assistant-fine-tuning/spec.md`

## Summary

This slice consolidates production-ready fine-tuning across three operator-facing surfaces: readable session references, streamlined local install refresh, and consistent two-button assistant routing in Copilot prompt commands. The implementation keeps existing runtime authority and safety boundaries intact while improving day-to-day usability and recoverability.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024, plus repository-managed shell and Markdown prompt assets  
**Primary Dependencies**: Existing workspace dependencies only (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`)  
**Storage**: Workspace-local `.boundline/session.json` and `.boundline/sessions/`, plus repository-managed prompt assets under `assistant/copilot/prompts/`  
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
src/cli.rs
scripts/install-local.sh
assistant/copilot/prompts/
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

## Complexity Tracking

No constitutional exceptions required for this slice.
