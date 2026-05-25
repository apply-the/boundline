# Implementation Plan: Governed Delivery With Canon Inside The Loop

**Branch**: `031-canon-delivery-loop` | **Date**: 2026-05-02 | **Spec**: `/Users/rt/workspace/boundline/specs/031-canon-delivery-loop/spec.md`
**Input**: Feature specification from `/Users/rt/workspace/boundline/specs/031-canon-delivery-loop/spec.md`

## Summary

Prove one real governed delivery path on the existing session-native route by
reusing Boundline's current Canon governance integration for `bug-fix` stage
framing and verify-stage evidence, then add an explicit delivery-completion
gate in `SessionRuntime` so a run can only terminate successfully when Canon is
not blocking, the workspace has a material diff, and validation evidence is
credible. Ship the slice as `0.31.0` with aligned docs, assistant guidance,
coverage, clippy, and formatting.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: existing crate dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and process APIs  
**Storage**: workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, and Canon-managed `.canon/` artifacts when governed runtime is selected  
**Testing**: `cargo test --all-targets`, focused unit/integration/contract tests, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all`  
**Target Platform**: macOS/Linux developer workstations and the existing Rust CLI test harness  
**Project Type**: single Rust library plus CLI crate  
**Execution Model**: sequential session-native task loop with bounded retries, replans, stage governance checkpoints, and persisted follow-through state  
**Observability Surface**: persisted execution traces, session view projections, CLI `run`/`status`/`next`/`inspect`, governance timeline events, review-trace events, and explicit terminal reasons  
**Performance Goals**: keep representative governed bug-fix runs within the existing bounded CLI workflow budget; add no extra background services or long-running daemons; preserve current operator feedback latency  
**Constraints**: no new orchestration owner besides Boundline, no Canon-owned session control, no silent success without material diff plus credible validation evidence, explicit compatibility route remains subordinate, modified/new Rust coverage must stay above 95%, release closeout requires version bump plus impacted docs and changelog  
**Scale/Scope**: one independently valuable governed delivery slice centered on the existing `bug-fix` flow and current CLI/read-side surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: Sections Summary and Technical Context keep the slice anchored to one real bounded code-delivery flow rather than platform abstraction.
- **PASS** Delivery-first scope: Sections Summary and Project Structure prioritize runtime gating, governed reuse, and validation before polish.
- **PASS** Primary workflow: Sections Summary and Project Structure keep the primary operator path session-native (`goal -> plan -> run -> status -> next -> inspect`); explicit compatibility remains available but subordinate.
- **PASS** Bounded execution: Sections Summary, Technical Context, and research decisions keep explicit stop conditions for governance block, pending approval, no material diff, no credible validation evidence, and existing step or replan limits.
- **PASS** Stateful execution: Sections Technical Context and data model rely on `.boundline/session.json`, task context state, and persisted traces as the authoritative shared state.
- **PASS** Mutable planning: The plan reuses the existing sequential runtime with current retry and replan behavior, only tightening terminal success conditions.
- **PASS** Sequential-first design: The runtime remains one-step-at-a-time in `SessionRuntime`; no new concurrent execution model is introduced.
- **PASS** Tool-agent symmetry: The design keeps governance, execution, validation, and resulting stop reasons explicit in task context and trace events.
- **PASS** Observability and explicit intelligence: Sections Technical Context and contracts require visible governance state, packet lineage, changed-files evidence, validation status, and terminal reasons on current CLI surfaces.
- **PASS** Non-goals and external separation: The plan does not ask Canon to own orchestration or execution; it stays inside bounded governance and evidence responsibilities.
- **PASS** Minimal slice: One governed bug-fix flow with explicit completion gating is the smallest independently valuable proof that Canon improves real delivery inside Boundline.

## Project Structure

### Documentation (this feature)

```text
specs/031-canon-delivery-loop/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
├── domain/
├── orchestrator/
├── adapters/
├── fixture.rs
└── lib.rs

tests/
├── contract/
├── integration/
├── support/
└── unit/
```

**Structure Decision**: Reuse the current CLI, domain, orchestrator, adapter,
fixture, and Rust test harness directories. The feature only needs new design
artifacts under `specs/031-canon-delivery-loop/`; it does not justify new
top-level runtime or product surfaces.

## Complexity Tracking

No constitution violations are expected for this slice.