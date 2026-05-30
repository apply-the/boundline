# Implementation Plan: Reasoning Profile Closure

**Branch**: `062-reasoning-profile-closure` | **Date**: 2026-05-18 | **Spec**: [./spec.md](./spec.md)
**Input**: Feature specification from `/specs/062-reasoning-profile-closure/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Close the residual S6.1 reasoning-profile work by proving the shipped status of
the remaining concrete profiles, explicitly classifying debate as bounded
substrate and adjudication as a shared primitive, aligning all operator-visible
and release-visible claims to that decision, and clearing the release-blocking
maintainability findings in the touched session and reasoning surfaces.

**Primary requirement**: make `independent_pair_review`,
`heterogeneous_security_review`, and `bounded_reflexion` fully credible through
real session-native runtime evidence while ensuring that debate and adjudication
are no longer implied as more shipped than the runtime actually supports.

**Technical approach**:
1. Reuse the existing `061` reasoning runtime and operator surfaces instead of
  adding new workflow families, and extend them only where residual end-to-end
  evidence is missing.
2. Keep debate as bounded substrate and adjudication as a shared primitive
  rather than standalone shipped profiles.
3. Because `0.62.0` changes the published supported pair, ship one Canon
  companion publication update to `0.59.0` that realigns version windows,
  changelog notes, and contract tests without adding Canon runtime behavior.
4. Refactor the flagged session-validation and reasoning-independence helpers so
  the closure slice passes the repository maintainability gate without
  suppressions.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024 in Boundline; Markdown, TOML, and JSON repository artifacts; required Canon companion publication updates for the new supported release pair  
**Primary Dependencies**: Existing workspace dependencies only (`serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `clap`, `dialoguer`, `rusqlite` already present in workspace); no new runtime crates planned  
**Storage**: Existing `.boundline/session.json`, trace files, configuration state, spec artifacts under `specs/062-reasoning-profile-closure/`, and release-facing docs or changelog files in Boundline plus the required Canon companion publication artifacts  
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused `cargo test --test contract <filter>`, `cargo test --test integration <filter>`, and `cargo test --test unit <filter>` runs, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and the existing SonarCloud quality gate in `.github/workflows/quality.yml` for the touched cognitive-complexity findings  
**Target Platform**: macOS and Linux developer workstations plus Linux CI for Boundline and the required Canon companion validation  
**Project Type**: Rust workspace CLI and session runtime with repository-managed contract and documentation artifacts  
**Execution Model**: Sequential session-native execution with bounded reasoning-profile activation inside existing governance stages; no new background workers or hidden parallel control flow  
**Observability Surface**: `run`, `status`, `inspect`, persisted execution traces, contract docs, roadmap entries, validation reports, changelogs, and version-window contract tests  
**Performance Goals**: Profile closure must not add a second workflow hop; representative profile scenarios must still terminate within existing configured budgets; `status` and `inspect` must remain readable for representative traces up to roughly 1,000 events  
**Constraints**: No second orchestrator, no reopened Canon posture boundary, no hidden fallback from shipped-profile claims to substrate-only behavior, no suppression of maintainability issues, Canon changes limited to companion publication surfaces, and Boundline must still validate compatibility when the sibling Canon repo is absent  
**Scale/Scope**: Three residual concrete profiles, two final classification decisions, one required cross-repo release alignment pass, and two maintainability hotspots (`SessionStatusView::validate_governance` and `assess_reasoning_independence`) inside the same closure cycle

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature directly improves bounded delivery by closing the residual reasoning profiles that developers are already expected to trust in the session-native workflow. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes executable runtime evidence, bounded failure handling, release alignment, and validation before polish-only work. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains `goal -> plan -> run -> status -> next -> inspect`; compatibility validation is explicit and secondary. See Summary and Execution Model.
- **PASS** Bounded execution: Every residual profile remains governed by explicit participant, budget, interruption, degradation, and blocked-stop conditions. See Technical Context and research.
- **PASS** Stateful execution: The feature reads and writes existing session, task-context, governance, reasoning, and trace state instead of creating a stateless side path. See Technical Context and Project Structure.
- **PASS** Mutable planning: The feature closes profile behavior inside the existing replanning-capable runtime and does not freeze the workflow into a second plan carrier. See Summary and research.
- **PASS** Sequential-first design: The outer workflow remains one-step-at-a-time, and profile closure work does not introduce hidden concurrency. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Reasoning decisions, operator-visible actions, and trace events remain explicit rather than heuristic-only. See Summary and Observability Surface.
- **PASS** Observability and explicit intelligence: The plan requires aligned `run`, `status`, `inspect`, trace, roadmap, contract, and validation outputs for every shipped claim. See Summary, data-model, and contracts.
- **PASS** Catalog currency: Current OpenAI, Anthropic, and Google provider model pages were checked during spec creation and no catalog delta was required; the no-change rationale remains linked from `spec.md`. See `spec.md` Catalog Research & Currency.
- **PASS** Non-goals and external separation: Canon companion work is required only as publication alignment for the new release pair; no new Canon runtime behavior or broader non-goal scope is introduced. See Constraints and research.
- **PASS** Minimal slice: The smallest independently valuable capability is one full closure pass that makes all first-wave S6 claims honest and release-ready; a smaller slice would still leave shipped-profile claims ambiguous. See Summary and research.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/062-reasoning-profile-closure/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── profile-closure-classification-contract.md
│   └── release-alignment-contract.md
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
│   ├── governance.rs
│   ├── reasoning.rs
│   ├── session.rs
│   └── trace.rs
├── fixture.rs
└── orchestrator/
  ├── governance.rs
  ├── review_trace.rs
  └── session_runtime.rs

tests/
├── contract/
│   ├── canon_reasoning_posture_contract.rs
│   ├── reasoning_profile_contract.rs
│   └── reasoning_profile_trace_contract.rs
├── integration/
│   ├── reasoning_profile_activation.rs
│   └── reasoning_profile_degradation.rs
└── unit/
  ├── governance_policy.rs
  ├── reasoning_profile_independence.rs
  ├── reasoning_profile_trace.rs
  ├── session_model.rs
  └── workflow_session_projection.rs

docs/
├── architecture.md
├── getting-started.md
└── release-checklist.md
```

**Structure Decision**: Keep the closure work inside the existing session,
reasoning, governance, CLI, and trace surfaces so the final shipped profile set
remains part of the current runtime instead of becoming a parallel subsystem.
The feature documentation lives under the 062 spec directory, while release and
compatibility statements update the existing repository docs. Canon companion
publication work
is limited to companion version-window, changelog, and contract-publication
files rather than new runtime modules.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | The feature fits the existing session-native runtime and companion compatibility surfaces without constitution violations. |
