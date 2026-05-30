# Implementation Plan: Provider Auth, Probe Readiness, and Assistant Handoff Fine-Tuning

**Branch**: `064-session-assistant-fine-tuning` | **Date**: 2026-05-28 | **Spec**: [spec.md](./spec.md)
**Input**: Retrospective plan update based on commits `cad1675`, `9ba0b21`, and `6182711`

## Summary

This slice now consolidates four implemented capability areas: a global provider-auth lifecycle for GitHub Copilot, runtime-visible planning-gate and assistant-handoff semantics, the read-only `boundline probe` readiness surface, and cross-host assistant prompt contract closure. The implementation keeps Boundline as the authority for readiness, planning, and continuation while making authentication, preflight routing, and host guidance materially more reliable.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024, plus repository-managed Markdown assistant assets
**Primary Dependencies**: Existing workspace dependencies only (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`)
**Storage**: User-global `auth-profiles.json` under the Boundline global config directory, workspace-local `.boundline/session.json`, workspace-local `.boundline/config.toml`, optional `.boundline/execution.json`, optional `.boundline/cluster.toml`, optional `.boundline/context-intelligence/retrieval-index.sqlite3`, and repository-managed assets under `assistant/`
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted probe tests, targeted assistant contract tests, and representative host-output and planning-gate coverage
**Target Platform**: macOS maintainer workflow plus standard Rust CI behavior for CLI and contract validation
**Project Type**: Rust workspace CLI with assistant command packs and host-facing Markdown assets

## Constitution Check

- PASS: Boundline remains the runtime authority for planning gates, next actions, and workspace readiness.
- PASS: The new auth and probe surfaces are bounded and operator-visible; they do not add hidden workflow loops.
- PASS: Prompt safety boundaries remain explicit through required sections and allowed follow-up commands.
- PASS: No Canon contract expansion is required for the probe or provider-auth slices.

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
src/adapters/auth_profile_store.rs
src/adapters/github_device_flow.rs
src/adapters/provider_runtime.rs
src/adapters/provider_runtime/copilot.rs
src/cli/models_auth.rs
src/cli.rs
src/cli/orchestrate.rs
src/cli/output.rs
src/cli/output_host.rs
src/cli/output_session_status.rs
src/cli/probe.rs
src/domain/auth_profile.rs
src/domain/brief.rs
src/domain/goal_plan.rs
src/domain/governance.rs
src/domain/probe.rs
src/orchestrator/session_runtime.rs
assistant/README.md
assistant/*/commands/
assistant/copilot/prompts/
tests/contract/
```

## Phase Plan

### Phase 1: Provider Auth Foundation

- Add a versioned auth profile domain model and global JSON persistence for provider credentials.
- Implement GitHub Copilot device-flow login and wire `models auth login|status|remove` into the CLI surface.
- Teach the touched provider-runtime adapters to consult stored auth profiles alongside existing environment credentials.

### Phase 2: Planning Gates and Assistant-Safe Output Alignment

- Surface `goal_quality_state`, `plan_quality_state`, `backlog_quality_state`, and `planning_analysis_state` through session and host-facing output models.
- Keep `phase_request`, `assistant_resume_command`, and `assistant_next_command` authoritative in orchestrate and related continuation flows.
- Add contract coverage for planning-gate precedence and structured host output.

### Phase 3: Probe Preflight Surface

- Introduce typed probe domain models for workspace, session, provider, Canon, and capability state.
- Add CLI dispatch, workspace fallback resolution, and host-envelope support for `boundline probe`.
- Compute bootstrap-safe `recommended_next` and `recommended_handoffs` without mutating workspace state.

### Phase 4: Assistant Handoff and Prompt Contract Closure

- Update readiness-sensitive assets to use probe as the preflight gate and to respect bootstrap versus doctor versus session-ready outcomes.
- Close missing `Next-Step Routing`, command-link, and host-native action gaps across Copilot and non-Copilot assets.
- Keep Copilot command-URI syntax and non-Copilot `/boundline:*` syntax separated and validated by contract tests.

### Phase 5: Documentation and Validation

- Document probe as an assistant helper surface, not a repo-local assistant command.
- Run focused probe and assistant contract suites plus workspace linting.
- Confirm retrospective spec artifacts match the implemented branch state.

## Complexity Tracking

No constitutional exceptions required for this slice.
