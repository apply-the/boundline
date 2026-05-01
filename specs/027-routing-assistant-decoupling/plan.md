# Implementation Plan: Inspectable Routing And Assistant Decoupling

**Branch**: `027-routing-assistant-decoupling` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/spec.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/spec.md)
**Input**: Feature specification from `/specs/027-routing-assistant-decoupling/spec.md`

**Note**: This plan keeps the 027 slice inside the current session-native and
explicit compatibility product story. Routing remains Synod-owned; assistants
and command packs become inspectable bindings of the resolved route, not a
second orchestration authority.

## Summary

Make effective provider/model routing decisions inspectable across the existing
session-native and explicit compatibility surfaces, then bind assistant command
pack or backend selection to that same resolved routing instead of leaving the
runtime hard-wired to one assistant family. The slice stays inside the current
CLI, session, trace, config, and assistant-pack surfaces, preserves Synod as
the orchestration authority, and closes as `0.27.0` with version bump,
impacted docs, changelog, coverage refresh for touched Rust files, clippy
cleanup, and formatting.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.synod/config.toml`, `.synod/cluster.toml`, `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, and repository-managed assistant asset files under `assistant/`  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit/contract/integration coverage for routing projection and assistant binding, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and `cargo nextest run --workspace --all-features`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state plus repository-managed assistant command packs  
**Execution Model**: Sequential session-native orchestration with explicit compatibility follow-up; slot routing is resolved from CLI/workspace/cluster/global config and projected into existing run/status/next/inspect surfaces without creating a second runtime  
**Observability Surface**: `config show`, persisted session and trace summaries, CLI `run`, `status`, `next`, and `inspect` output, assistant command-pack guidance, and release docs that make route ownership plus assistant binding explicit  
**Performance Goals**: Operators should identify the active slot route, authority source, and bound assistant family from runtime output in under 2 minutes; maintainers should validate the `0.27.0` release story in under 20 minutes  
**Constraints**: No provider-auth gateway, no hidden background workers, no assistant-owned orchestration, no compatibility authority confusion, no distributed control flow, and no new runtime outside Synod's existing session and explicit compatibility surfaces  
**Scale/Scope**: One workspace or one registered cluster at a time, representative routing for planning/implementation/verification/review plus adjudication or reviewer roles when materially surfaced, and bounded updates to existing CLI, trace, config, assistant, and test surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice improves bounded engineering-task delivery by making slot routing and assistant binding visible before and after execution rather than leaving backend ownership implicit. See Summary, Technical Context, and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/spec.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/spec.md).
- **PASS** Delivery-first scope: The plan centers on execution visibility, route binding, and follow-up surfaces first; release polish remains a closeout phase rather than the feature core. See Summary and Technical Context.
- **PASS** Primary workflow: Session-native remains the default operator path, while explicit compatibility follow-up stays visibly separate and continues to use the same route-ownership vocabulary. See Summary, Technical Context, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/quickstart.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/quickstart.md).
- **PASS** Bounded execution: The feature reuses the existing sequential runtime with current terminal conditions, run limits, and compatibility follow-up boundaries instead of adding new loops or workers. See Technical Context, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/quickstart.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/quickstart.md).
- **PASS** Stateful execution: Routing decisions and assistant binding are projected through existing config, session, and trace surfaces so later commands can explain why a backend is authoritative. See Summary, Technical Context, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/data-model.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/data-model.md).
- **PASS** Mutable planning: The slice does not replace planning; it exposes which configured route owns a slot and keeps that explanation visible as planning or follow-up evolves. See Summary, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/data-model.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/data-model.md).
- **PASS** Sequential-first design: One step remains active at a time, and no concurrency or background control flow is introduced. See Technical Context and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md).
- **PASS** Tool-agent symmetry: The plan keeps reasoning and action explicit by surfacing route selection, command-pack binding, and follow-up evidence through the same bounded execution model. See Summary, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), and the contracts in [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/contracts](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/contracts).
- **PASS** Observability and explicit intelligence: Routing-decision and assistant-binding cues are explicitly projected through CLI and trace surfaces instead of remaining hidden inside adapter registration. See Technical Context, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), contracts, and quickstart.
- **PASS** Non-goals and external separation: The slice avoids provider-auth gateways, new control planes, UI work, deployment work, long-term memory, and Canon-owned orchestration. See Constraints, [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md), and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/spec.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one inspectable routing-decision story plus assistant/backend binding that follows it on existing surfaces. See Summary and [/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md](/Users/rt/workspace/synod/specs/027-routing-assistant-decoupling/research.md).

## Project Structure

### Documentation (this feature)

```text
specs/027-routing-assistant-decoupling/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── routing-decision-surface-contract.md
│   ├── assistant-binding-surface-contract.md
│   └── compatibility-routing-boundary-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── agent.rs
├── cli/
│   ├── config.rs
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── session.rs
│   ├── trace.rs
│   └── routing_decision.rs
├── orchestrator/
│   ├── engine.rs
│   └── session_runtime.rs
└── registry/
    └── agent_registry.rs

assistant/
├── claude/commands/
├── codex/commands/
├── copilot/prompts/
└── gemini/

tests/
├── contract/
├── integration/
└── unit/

docs/
├── configuration.md
└── getting-started.md

README.md
CONTRIBUTING.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing configuration,
session, trace, CLI rendering, orchestrator registry, assistant-pack, and test
surfaces. The only new source module expected is a small routing-decision
projection type so visibility can be reused across session and trace summaries
without introducing a broader framework. No new top-level runtime or service is
justified because the feature clarifies existing orchestration ownership rather
than creating a second execution surface.

## Complexity Tracking

No constitution violations are expected for this slice.
