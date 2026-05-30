# Implementation Plan: Canon Governance Expansion

**Branch**: `017-canon-governance-expansion` | **Date**: 2026-04-29 | **Spec**: [/Users/rt/workspace/boundline/specs/017-canon-governance-expansion/spec.md](/Users/rt/workspace/boundline/specs/017-canon-governance-expansion/spec.md)
**Input**: Feature specification from `/specs/017-canon-governance-expansion/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Deepen Boundline's Canon governance coverage by adding one newer governed analysis mode, `security-assessment`, to the existing stage-boundary Canon adapter while keeping Boundline's built-in `bug-fix`, `change`, and `delivery` flows unchanged. The first slice will extend Canon mode validation, stage-to-mode selection, autopilot decision building, approval refresh, packet provenance, and session-native operator surfaces so existing-system verification stages can route through governed security analysis without turning Canon into the per-action control plane. The design will preserve bounded packet reuse and leave a clear extension path for later `supply-chain-analysis` support.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; external Canon CLI compatibility target updated to `0.25.0`; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and repository docs plus assistant assets  
**Testing**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo deny check licenses advisories bans sources`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with persisted session and trace state  
**Execution Model**: Sequential session-native execution with explicit stage-boundary governance, bounded approval refresh, bounded packet reuse, and explicit terminal conditions  
**Observability Surface**: Persisted active session/task context, persisted execution traces, route-aware `run`, `status`, `next`, and `inspect` output, plus assistant-facing command-pack summaries  
**Performance Goals**: Canon mode selection and approval refresh should complete within one CLI round-trip; governed operator surfaces should render within the existing session-native latency budget for representative traces up to 500 events; no background polling or hidden asynchronous refresh behavior  
**Constraints**: Ship as crate version `0.17.0`; update the documented Canon compatibility target to `0.25.0`; keep built-in Boundline flows unchanged; do not add new top-level stage ids; preserve Canon as a bounded governance and evidence overlay; do not add full Canon mode parity; defer real `supply-chain-analysis` support, tool-availability UX, and broader operational mode coverage; keep packet reuse bounded to refs, headlines, readiness, and missing metadata  
**Scale/Scope**: One active session per workspace, first-slice governance expansion limited to the targeted existing-system verification stages plus session-visible summaries and bounded policy validation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded engineering task delivery by extending a real governed verification path for session-native work instead of adding a generic integration surface. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice prioritizes execution control, governed verification, packet readiness, approval refresh, and operator visibility before any secondary polish. See Summary and Constraints.
- **PASS** Primary workflow: `goal -> plan -> run -> status -> next -> inspect` remains the main operator path; compatibility via `.boundline/execution.json` remains available but unchanged. See Summary and Constraints.
- **PASS** Bounded execution: Governed follow-on analysis starts only at targeted stage boundaries, refreshes only on later explicit commands, and stops explicitly on blocked, failed, or approval-gated outcomes. See Technical Context and spec requirements.
- **PASS** Stateful execution: The design continues to read and write active task context, governance packet state, decision state, and trace evidence inside the existing session and trace stores. See Technical Context and Project Structure.
- **PASS** Mutable planning: The slice reuses existing stage selection, candidate-mode selection, bounded packet reuse, and explicit decision traces rather than freezing governance decisions into a static path. See Summary and Technical Context.
- **PASS** Sequential-first design: The feature remains one-step-at-a-time with command-driven refresh and no hidden concurrency or background workers. See Execution Model and Performance Goals.
- **PASS** Tool-agent symmetry: Canon mode selection, approval refresh, packet evaluation, and operator follow-up remain visible as explicit action and evaluation steps. See Observability Surface and Constraints.
- **PASS** Observability and explicit intelligence: The slice keeps selection rationale, selected Canon mode, packet provenance, readiness, approval state, blocked reason, and next action visible in session-native surfaces and traces. See Summary and Observability Surface.
- **PASS** Non-goals and external separation: Canon remains a bounded governance overlay; the plan does not depend on Canon beyond governed start/refresh semantics and does not reintroduce deferred non-goals such as new UI, provider abstraction, or distributed execution. See Constraints.
- **PASS** Minimal slice: The smallest independently valuable capability is one newer governed analysis mode (`security-assessment`) attached to existing verification stages with coherent operator visibility. See Summary and Scale/Scope.

## Project Structure

### Documentation (this feature)

```text
specs/017-canon-governance-expansion/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── governed-analysis-surface-contract.md
│   └── stage-mode-expansion-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── governance_runtime.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── flow.rs
│   ├── governance.rs
│   └── session.rs
└── orchestrator/
    ├── governance.rs
    └── session_runtime.rs

tests/
├── contract/
│   ├── canon_runtime_contract.rs
│   └── governance_session_contract.rs
├── integration/
│   └── governance_autopilot_flow.rs
└── unit/
    ├── canon_stage_mapping.rs
    ├── cli_output.rs
    ├── governance_policy.rs
    └── governance_runtime.rs
```

**Structure Decision**: Keep the work inside the existing governance, session, CLI, and test surfaces. No new top-level runtime surface is justified because the value comes from extending the current bounded stage-governance model rather than introducing a second orchestrator or a new external integration layer.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | The first slice fits the existing session-native governance surfaces without constitutional violations. |
