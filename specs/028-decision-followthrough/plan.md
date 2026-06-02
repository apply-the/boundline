# Implementation Plan: Decision Continuity And Guided Follow-Through

**Branch**: `028-decision-followthrough` | **Date**: 2026-05-01 | **Spec**: [specs/028-decision-followthrough/spec.md](specs/028-decision-followthrough/spec.md)
**Input**: Feature specification from `/specs/028-decision-followthrough/spec.md`

**Note**: This plan keeps the 028 slice inside the current session-native and
explicit compatibility operator story. The feature does not widen orchestration;
it makes the next bounded action and its supporting evidence more explicit on
the existing read-side surfaces.

## Summary

Project one explicit follow-through story through `status`, `next`, and
`inspect` by combining persisted session continuity with authoritative trace
evidence, so operators can see what Boundline should do next and why that next
step is credible. The slice stays inside the current CLI, session, trace, and
assistant-guidance surfaces, preserves explicit native versus compatibility
authority, and closes as `0.28.0` with version bump, impacted docs, changelog,
coverage refresh for touched Rust files, clippy cleanup, and formatting.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.boundline/workflows.toml`, optional cluster state under `.boundline/cluster.toml`, and repository-managed assistant assets under `assistant/`  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit/contract/integration tests for follow-through guidance and continuity authority, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and focused regression checks for touched CLI and domain slices  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state plus repository-managed assistant command packs  
**Execution Model**: Sequential session-native orchestration with explicit compatibility follow-up; the slice reuses persisted session and trace evidence to guide one next bounded action without adding concurrency or a new runtime  
**Observability Surface**: Persisted session status, authoritative trace summaries, CLI `status`, `next`, and `inspect` output, continuity authority cues, decision and recovery projection, assistant command-pack guidance, and release docs that explain the guided follow-through story  
**Performance Goals**: Operators should identify the next bounded action and its winning evidence source from runtime output in under 2 minutes; maintainers should validate the `0.28.0` release story in under 20 minutes  
**Constraints**: No new background workers, no provider gateway, no hidden control plane, no silent evidence precedence, no compatibility-authority confusion, no distributed orchestration, and no UI work  
**Scale/Scope**: One workspace or one registered cluster at a time, representative retry/replan/governance/inspect-only follow-up scenarios, and bounded updates to existing CLI, session, trace, assistant, and test surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice improves bounded engineering-task delivery by making the next bounded action and its supporting evidence visible on the existing follow-up surfaces instead of forcing manual reconstruction from raw state. See Summary, Technical Context, and [specs/028-decision-followthrough/spec.md](specs/028-decision-followthrough/spec.md).
- **PASS** Delivery-first scope: The plan centers on execution follow-through, continuity evidence, and stop-condition clarity first; release polish remains a closeout phase rather than the feature core. See Summary and Technical Context.
- **PASS** Primary workflow: Session-native remains the default operator path, while explicit compatibility follow-up stays visibly separate and authoritative only when the latest trace owns continuity. See Summary, Technical Context, research, and quickstart.
- **PASS** Bounded execution: The feature reuses the current sequential runtime, existing run limits, and explicit stop conditions instead of introducing loops or silent recovery. See Technical Context, research, and quickstart.
- **PASS** Stateful execution: Guided follow-through is driven by persisted session records and authoritative trace evidence so later commands can explain why one next action or stop condition is currently credible. See Summary, Technical Context, research, and data model.
- **PASS** Mutable planning: The slice explains existing retry, replanning, and inspect-only follow-up more clearly; it does not replace planning or invent a new mutation model. See Summary, research, and data model.
- **PASS** Sequential-first design: One step remains active at a time, and no concurrency or background control flow is introduced. See Technical Context and research.
- **PASS** Tool-agent symmetry: The plan keeps reasoning and action explicit by surfacing decision continuity, recovery evidence, and follow-up commands on the same bounded operator surfaces. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Evidence precedence, continuity authority, failure signals, and guided next-action explanations are all explicit on CLI and trace surfaces instead of remaining hidden inside session transitions. See Technical Context, research, contracts, and quickstart.
- **PASS** Non-goals and external separation: The slice avoids provider gateways, new control planes, UI work, deployment work, long-term memory, and Canon-owned orchestration. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is one guided follow-through story that reuses existing session and trace evidence on `status`, `next`, and `inspect`. See Summary and research.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/028-decision-followthrough/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── guided-next-action-surface-contract.md
│   ├── continuity-evidence-precedence-contract.md
│   └── compatibility-followthrough-boundary-contract.md
└── tasks.md
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Keep the structure minimal, delivery-focused, and sequential-
  first. Do not introduce extra top-level projects or UI/runtime surfaces unless
  the Constitution Check explicitly justifies them.
-->

```text
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── session.rs
│   ├── trace.rs
│   └── follow_through.rs
├── orchestrator/
│   └── session_runtime.rs
└── fixture.rs

assistant/
├── claude/commands/
├── codex/commands/
├── copilot/prompts/
└── gemini/

tests/
├── contract/
├── integration/
└── unit/

tech-docs/
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

**Structure Decision**: Keep the slice inside the existing session, trace, CLI
rendering, runtime follow-up, assistant-pack, and test surfaces. The only new
source module expected is a small follow-through projection type so decision
continuity can be reused across session and trace summaries without widening the
orchestration framework. No new top-level runtime or service is justified because
the feature clarifies existing follow-up ownership rather than creating a second
execution surface.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
