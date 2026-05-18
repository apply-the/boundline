# Implementation Plan: Governed Reasoning Profile Contracts

**Branch**: `061-reasoning-profile-contracts` | **Date**: 2026-05-18 | **Spec**: [./spec.md](./spec.md)
**Input**: Feature specification from `/specs/061-reasoning-profile-contracts/spec.md`

## Summary

Implement the S6 contract layer by making advanced reasoning profiles a
first-class bounded runtime surface inside Boundline's existing governance and
review model while defining the Canon-side challenge-posture inputs that may
activate that surface. The feature adds typed profile vocabulary, explicit
activation and failure handling, profile-level confidence handoff, reasoning
trace events, and bilateral version-alignment tests without introducing a
second governance system or a second orchestrator.

**Primary requirement**: deliver one coherent Boundline-owned reasoning-profile
runtime that can activate bounded self-consistency, blind independent pair
review, heterogeneous review, bounded reflexion, and controlled debate through
the existing session-native workflow, while consuming Canon-owned challenge
posture only through an explicit consumer contract and supported compatibility
window.

**Technical approach**:
1. Add a typed reasoning-profile domain model that composes with existing
  governance, review, routing, and trace types instead of replacing them.
2. Reuse the current session runtime and review surfaces to record activation,
  participant topology, disagreement, adjudication, confidence contribution,
  and explicit degraded or blocked outcomes.
3. Define one Boundline consumer contract for Canon challenge posture and one
  sibling Canon provider contract, then lock the shared vocabulary and version
  window with bilateral contract tests.
4. Keep the feature independently testable in Boundline by using local posture
  fixtures and deterministic profile scenarios even when the sibling Canon
  repository is unavailable during normal unit and integration runs.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024 for Boundline and Canon runtime changes; Markdown and TOML or JSON contract artifacts for cross-repo contract surfaces  
**Primary Dependencies**: Existing Boundline workspace dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) and Canon workspace dependencies of the same family; no new runtime crates planned for the first implementation line  
**Storage**: Existing Boundline `.boundline/session.json`, `.boundline/traces/`, optional config and execution-profile surfaces, feature-local spec artifacts under `specs/061-reasoning-profile-contracts/`, sibling Canon docs under `docs/integration/`, and normal repository documentation surfaces  
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, focused unit, integration, and contract tests for reasoning-profile activation and trace projection, bilateral Boundline↔Canon compatibility tests, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS and Linux developer workstations plus Linux CI for both repositories
**Project Type**: Rust workspace CLI/runtime with persisted session and trace state plus repository-managed cross-repo contracts  
**Execution Model**: Sequential session-native execution where a governed stage may enter one bounded reasoning-profile subroutine with explicit participants, explicit budget limits, explicit failure or escalation outcomes, and no hidden concurrency or background workers  
**Observability Surface**: Existing `plan`, `run`, `status`, `next`, and `inspect` projections; persisted execution traces; review and governance timelines; bilateral contract tests; and Canon integration docs that define the supported posture vocabulary  
**Performance Goals**: Reasoning-profile activation and condition refresh complete within one operator command round-trip; `status` and `inspect` remain readable for representative traces up to 750 events; bounded debate or reflexion loops must terminate within configured limits without hidden retries  
**Constraints**: Ship Boundline as `0.61.0`; align to Canon `0.57.0`; keep existing governance semantics authoritative; do not add a second governance runtime, new top-level workflow family, hidden parallel fan-out, or Canon-owned orchestration; preserve independent Boundline testability through local fixtures; keep V1 profile counts small and explicitly bounded  
**Scale/Scope**: One active reasoning-profile execution per governed stage, 2-5 participants per profile, low single-digit branch or round counts, one confidence contribution per profile execution, and one supported Canon challenge-posture contract line in the first release

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded engineering delivery by making stronger challenge executable inside the existing session lifecycle for planning, verification, and review work. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes activation, bounded execution, explicit disagreement handling, confidence handoff, and validation before polish or documentation-only work. See Summary and Constraints.
- **PASS** Primary workflow: The primary operator path remains `start -> capture -> plan -> run -> status -> next -> inspect`; explicit compatibility and fixture-backed testing remain available but do not replace the session-native story. See Summary and Execution Model.
- **PASS** Bounded execution: Reasoning profiles start only from explicit stage activation, carry explicit participant and budget limits, and terminate in completion, degradation, blocked, interrupted, escalated, or terminal states. See Technical Context and contracts.
- **PASS** Stateful execution: Activation records, participant topology, disagreement outcomes, confidence contribution, and trace evidence are persisted in existing task, session, and trace state. See Technical Context and Project Structure.
- **PASS** Mutable planning: The plan reuses the current replanning-capable session runtime so reasoning-profile activation can refine or challenge an existing plan without freezing workflow state into a second runtime. See Summary and research.
- **PASS** Sequential-first design: Profile execution remains one-step-at-a-time at the outer workflow layer; participant reasoning is modeled as bounded sequential substeps rather than hidden parallel workers. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Participant execution, adjudication, and confidence evaluation remain explicit runtime steps and trace events instead of opaque heuristics. See Summary and Observability Surface.
- **PASS** Observability and explicit intelligence: Activation reason, Canon posture provenance, independence result, disagreement, cost, confidence contribution, and final condition are surfaced through session-native outputs and traces. See Technical Context, data-model, and contracts.
- **PASS** Catalog currency: Current OpenAI, Anthropic, and Google public model pages were checked during spec creation; no bundled catalog delta was required and the no-change rationale is recorded in the feature spec. See `spec.md` Catalog Research & Currency.
- **PASS** Non-goals and external separation: Canon supplies only a bounded posture contract; Boundline remains independently testable with local fixtures and does not delegate core control flow or create new UI, long-term memory, or deployment scope. See Summary, Constraints, and research.
- **PASS** Minimal slice: The smallest independently valuable capability is one full reasoning-profile contract layer with executable activation, inspectable outcomes, and bilateral version validation; a smaller slice would leave the Boundline↔Canon boundary undefined. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/061-reasoning-profile-contracts/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── reasoning-profile-runtime-contract.md
│   ├── canon-challenge-posture-consumer-contract.md
│   ├── reasoning-trace-contract.md
│   └── reasoning-version-alignment-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── governance.rs
│   ├── reasoning.rs
│   ├── review.rs
│   ├── session.rs
│   └── trace.rs
├── fixture.rs
└── orchestrator/
    ├── reasoning_profile.rs
    ├── review_trace.rs
    └── session_runtime.rs

tests/
├── contract/
│   ├── canon_reasoning_posture_contract.rs
│   ├── reasoning_profile_contract.rs
│   └── reasoning_profile_trace_contract.rs
├── integration/
│   ├── reasoning_profile_activation.rs
│   ├── reasoning_profile_degradation.rs
│   └── reasoning_profile_inspect.rs
└── unit/
    ├── reasoning_profile_domain.rs
    ├── reasoning_profile_selection.rs
    └── reasoning_profile_trace.rs
```

**Structure Decision**: Keep the work inside the existing session runtime,
governance, review, configuration, CLI, and trace surfaces. Add one new domain
module for typed reasoning-profile state and one focused orchestrator helper
module to keep the bounded profile logic separate from the broader session
runtime without creating a second orchestrator. The sibling Canon repository
receives matching contract docs and compatibility checks, but Boundline remains
the runtime owner and stays independently testable through local contract
fixtures.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | The feature fits the existing session-native runtime and bounded governance model without constitution violations. |
