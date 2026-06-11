# Implementation Plan: Safe Command Execution and Evidence Capture

**Branch**: `077-safe-command-execution` | **Date**: 2026-06-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/077-safe-command-execution/spec.md`

## Summary

Add a deterministic command execution safety layer to Boundline that classifies command intent, enforces execution policy via an Intent × Zone matrix, captures structured evidence packets, redacts secrets, and supports governance hooks. No Docker required — v1 is entirely local execution safety.

## Technical Context

**Language/Version**: Rust 1.96.0, Edition 2024

**Primary Dependencies**: Existing workspace crates (`boundline-core`, `boundline-cli`, `boundline-adapters`) + `clap`, `serde`, `serde_json`, `toml`, `thiserror`, `tracing`, `uuid`, `sha2`, `regex`

**Storage**: Workspace-local files under `.boundline/`: traces JSON, execution-policy.toml, redaction.toml, evidence-limits.toml

**Testing**: `cargo test`, `cargo nextest run`, `assert_cmd` for CLI integration

**Target Platform**: macOS, Linux, Windows — CLI binary + library

**Project Type**: CLI + library (Rust workspace, single-project structure)

**Performance Goals**: Evidence packet within 100ms of command termination; classification <5ms

**Constraints**: No Docker, no container runtime, no LLM calls for classification, no external services

**Scale/Scope**: 1 command at a time; stdout/stderr 1MB caps; mutation boundary 10K entries

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Delivery Identity | ✅ PASS | Improves delivery reliability via auditable, policy-gated execution |
| II. Delivery-First Scope | ✅ PASS | Execution safety is the most fundamental delivery concern |
| III. No Abstract Agent Systems | ✅ PASS | Deterministic rule-based classification, no agent reasoning |
| IV. Bounded Execution | ✅ PASS | 1MB caps, 10K limits, configurable timeouts |
| V. Stateful Execution | ✅ PASS | Evidence packets + mutation boundaries persist state |
| VI. Mutable Planning | ✅ PASS | Policy hot-reloadable via TOML |

## Project Structure

### Source Code

```text
crates/boundline-core/src/execution/
├── mod.rs              # Module root
├── classifier.rs       # Intent classification (whitelist + args)
├── policy.rs           # Intent × Zone matrix + overrides
├── evidence.rs         # Evidence packet capture + persistence
├── redaction.rs        # Regex-based secret redaction
├── dry_run.rs          # Deterministic dry-run tiers
├── mutation.rs         # Mutation boundary computation
└── hooks.rs            # Governance hooks

crates/boundline-cli/src/commands/
└── exec.rs             # NEW: `boundline exec` command

crates/boundline-adapters/src/
└── shell.rs            # MODIFIED: capture/redact/policy hooks

tests/
├── contract/execution_safety_contract.rs
├── integration/exec_command_integration.rs
└── unit/execution/     # Per-module unit tests
```

## Complexity Tracking

*No constitution violations — no entries needed.*
