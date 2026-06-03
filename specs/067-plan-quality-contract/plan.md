# Implementation Plan: Plan Quality Contract

**Branch**: `067-plan-quality-contract` | **Date**: 2026-06-02 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/067-plan-quality-contract/spec.md`

## Summary

Ship the first formal planning-readiness slice as Boundline-owned runtime
behavior. Reuse the existing `GoalPlan`, `PlanQualityAssessment`,
`SessionStatusView`, orchestration `phase_request`, trace, and assistant-asset
surfaces; audit and complete the current scaffolding so a missing verification
strategy blocks execution handoff with one focused question,
while low-impact defaults remain visible assumptions. Close the slice as
release `0.67.0` with aligned metadata, operator docs, clippy compliance, and at
least 95% patch coverage for changed or created implementation files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only; no new runtime dependency is planned

**Storage**: Existing workspace-local session and trace files, extended with additive plan-quality fields and trace-visible projections

**Testing**: `cargo test --test unit`, `cargo test --test contract`,
`cargo test --test integration human_input_capture_flow::`, focused
planning-gate tests, `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and
`scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime on supported developer workstations;
assistant assets for Copilot, Claude, Codex, and Antigravity remain thin
projections over the same CLI/runtime contract

**Project Type**: Rust workspace with CLI, runtime, assistant assets,
distribution metadata, and repository-managed documentation

**Performance Goals**: Plan-quality evaluation remains bounded to one
in-memory assessment per planning or execution-admission decision and adds no
external I/O; blocked flows emit exactly one operator question per handoff

**Constraints**: Reuse current session-native planning; preserve older session
snapshot compatibility; no new CLI subcommand; no file-first Speckit runtime;
no Canon-owned control flow; sequential question handling only; all changed
implementation files require at least 95% patch coverage

**Scale/Scope**: One readiness gate over an active plan, three additive state
values, concise findings and assumptions, one-question recovery, four
supported assistant planning assets, user docs, tech docs, and release closure

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | The gate prevents unvalidated implementation handoff and directly improves working-code delivery reliability. |
| Bounded execution | PASS | Evaluation is single-pass; recovery emits exactly one `phase_request` and waits for explicit operator input. |
| Stateful execution | PASS | Assessment, findings, assumptions, recovery transitions, and traces remain session-visible. |
| Mutable planning and execution over perfect planning | PASS | Missing validation strategy requests focused input and resumes the same plan; the feature does not require exhaustive upfront analysis. |
| Sequential-first design | PASS | One gate and one question are active at a time; no parallel or background processing is added. |
| Tool-agent symmetry and required observability | PASS | CLI, status, orchestration snapshots, inspect, traces, and assistant assets project the same runtime decision. |
| No hidden intelligence | PASS | Stable findings, accepted assumptions, gate order, and recovery routing are surfaced explicitly. |
| Strict non-goals and minimal capability slice | PASS | Backlog quality, planning analysis, providers, sandboxing, memory, councils, and recursive refinement remain outside this slice. |
| Real acceptance criteria and failure-first behavior | PASS | The spec covers ready, blocked, compatibility, and recovered planning paths in isolated temporary workspaces. |
| Separation from external systems | PASS | Canon packets may inform planning, but Boundline remains independently testable and owns admission control. |
| Catalog currency | PASS | `research.md` records the 2026-06-02 public-doc refresh and the local duplicate cleanup required in the bundled catalog. |
| Rust language rules | PASS | The plan requires typed serde shapes, named constants, explicit errors, docs, structured tracing, formatting, and clippy closure. |

## Project Structure

### Documentation (this feature)

```text
specs/067-plan-quality-contract/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── plan-quality-runtime-contract.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   ├── goal_plan.rs
│   └── session.rs
├── orchestrator/
│   ├── session_runtime.rs
│   ├── session_runtime_native_goal_plan.rs
│   └── session_runtime_planning_runtime.rs
└── cli/
    ├── session.rs
    ├── output_orchestrate.rs
    └── output_session_status.rs

assistant/
├── antigravity/commands/boundline-plan.md
├── claude/commands/boundline-plan.md
├── codex/commands/boundline-plan.md
├── copilot/prompts/boundline-plan.prompt.md
└── catalog/model-catalog.toml

tests/
├── unit/
│   ├── goal_plan_model.rs
│   ├── session_cli_runtime.rs
│   ├── session_record.rs
│   └── cli_output.rs
├── contract/
│   ├── assistant_command_definition_contract.rs
│   ├── host_command_output_contract.rs
│   └── planning_gate_pipeline_contract.rs
└── integration/
    └── human_input_capture_flow.rs

docs/
├── runtime/plan.md
├── runtime/phase-requests.md
├── guide/common-workflows.md
└── roadmap/index.md

tech-docs/
├── architecture.md
├── configuration.md
├── getting-started.md
└── host-orchestration-contract.md

distribution/
├── channel-metadata.toml
├── homebrew/Formula/boundline.rb
└── winget/manifests/a/ApplyThe/Boundline/0.67.0/
```

**Structure Decision**: Keep the current single Rust workspace and reuse the
existing planning-quality scaffolding. The implementation phase starts with a
gap audit against the spec, then changes only files needed to close observed
behavior, documentation, catalog hygiene, release metadata, and test coverage.

## Phase 0 Research Conclusions

- Reuse `GoalPlan::assess_plan_quality()` and its typed additive
  `PlanQualityAssessment`; do not add a second planning validator.
- Keep plan-quality evaluation ahead of backlog quality and planning-analysis
  admission checks so the first actionable planning defect is deterministic.
- Preserve `phase_request` as the one-question recovery contract; assistant
  assets must not synthesize execution from chat-only assumptions.
- Treat `routing_policy_summary` omission as a visible accepted assumption,
  while missing planning rationale or verification strategy remains
  recoverable clarification and non-credible context remains blocked.
- Refresh public model documentation in the feature packet. No new model family
  is required, but the duplicate `opus-4.8` entry in the bundled catalog must
  be removed and catalog metadata refreshed during implementation.

## Phase 1 Design Outputs

- [research.md](research.md) records reuse decisions, public model-catalog
  evidence, and rejected alternatives.
- [data-model.md](data-model.md) defines persisted assessment, finding,
  assumption, and clarification-request behavior.
- [plan-quality-runtime-contract.md](contracts/plan-quality-runtime-contract.md)
  defines additive session output, evaluation ordering, and assistant-safe
  recovery semantics.
- [quickstart.md](quickstart.md) defines isolated validation scenarios and
  release-quality commands.

## Post-Design Constitution Recheck

The design remains compliant after Phase 1. It adds no dependency, command,
provider abstraction, external runtime dependency, concurrency, hidden
fallback, or second planning engine. The implementation packet must preserve
the first-slice boundary and must not fold roadmap seeds 04 or 05 into release
`0.67.0`.
