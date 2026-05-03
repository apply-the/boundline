# Implementation Plan: Multi-Agent Review & Voting

**Branch**: `007-multi-agent-review` | **Date**: 2026-04-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-multi-agent-review/spec.md`

## Summary

Add a bounded review phase on top of the execution engine so Boundline can run an explicit council of sequential reviewer agent steps, capture structured findings, apply majority or weighted voting, optionally perform one adjudication step, and surface the resulting review decision through run, status, next, and inspect. The minimal slice extends the existing execution manifest and task lifecycle instead of introducing a new runtime, preserving the current CLI, session store, trace store, and sequential execution model while adding review-specific task state, trace events, and developer-facing documentation. Assistants remain documentation and summary surfaces in this slice rather than runtime reviewers.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library collections; no new runtime dependencies for the initial review slice  
**Storage**: Workspace-local JSON session record at `<workspace>/.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, and workspace execution manifests under `<workspace>/.boundline/execution.json` extended with bounded review configuration  
**Testing**: `cargo test --all-targets`, contract and integration coverage for review councils, vote resolution, and inspection output, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings`  
**Target Platform**: macOS and Linux developer workstations plus Linux CI via the existing GitHub Actions workflows  
**Project Type**: Single Rust CLI crate with file-backed session and trace persistence plus repository-managed assistant assets  
**Execution Model**: Sequential task execution where a reviewable terminal delivery result may append one bounded review phase, represent each reviewer as an explicit agent step that reads deterministic findings from the manifest-backed `ReviewScenario` for the active trigger in the initial slice, tally the vote through explicit evaluation logic, optionally adjudicate once when the vote resolves to `needs_adjudication`, ignore later duplicate triggers for the same task stage, and terminate in an explicit review outcome within the existing run limits. Majority voting accepts when approval count is greater than half of completed reviewers and rejects when block count is greater than half; weighted voting applies the same threshold to summed reviewer weights. When `reject_on_blocking = true`, any completed blocking finding forces immediate rejection. Results that are neither accepted nor rejected become `needs_adjudication`. `failed` is reserved for malformed or unavailable reviewer or adjudicator output, while `escalated` is reserved for credible but unresolved review outcomes such as adjudication being disabled or exhausted  
**Observability Surface**: Persisted execution traces, session status and next-command output, inspect rendering, workspace diagnostics, structured reviewer findings, reviewer participation statuses, vote tallies, adjudication outcomes, duplicate-trigger events, and dedicated voting documentation  
**Performance Goals**: Local review runs remain interactive for small councils, review tallying adds negligible overhead compared to validation, and status or inspect rendering remains fast enough for command-line use  
**Constraints**: Reuse the existing orchestrator loop, preserve one-step-at-a-time execution, avoid hidden background councils, keep the review UX provider-agnostic, keep review behavior bounded and traceable, retain the existing execution profile fallback story, keep the crate version at 0.7.0, and add user-facing docs that explain voting and adjudication  
**Scale/Scope**: One bounded review phase per delivery run after a reviewable terminal delivery result, small councils of 2-5 reviewers, one optional adjudicator distinct from the primary council, one explicit vote resolution per review phase, and CLI-visible triggers for risky changes, failed validation, or PR-readiness review

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature adds explicit quality control for working-code delivery results and directly improves Boundline's ability to accept, reject, or escalate generated output.
- Delivery-first scope: PASS. The plan keeps execution, validation follow-up, and inspectable review decisions ahead of optimization or generalized agent-platform work.
- Bounded execution: PASS. Review starts only from explicit triggers, runs within bounded reviewer and adjudication limits, and ends in accepted, rejected, escalated, or failed terminal review states.
- Stateful execution: PASS. Reviewer findings, vote tallies, adjudication outcomes, and trigger reasons are written into task context, session projections, and traces.
- Mutable planning: PASS. The plan adds bounded review steps and one optional adjudication branch without abandoning the existing replan-capable task model.
- Sequential-first design: PASS. Review councils are modeled as conceptual groups but execute sequentially through explicit reviewer agent steps rather than parallel fan-out.
- Tool-agent symmetry: PASS. Reviewers remain explicit agents, vote tallying is explicit evaluation logic, and adjudication is represented as one visible execution step.
- Observability and explicit intelligence: PASS. Reviewer participation, structured findings, vote rules, tally outputs, duplicate-trigger handling, adjudication, and final decisions are all persisted and inspectable.
- Non-goals and external separation: PASS. This slice relies only on the explicitly reprioritized bounded review model, avoids Canon dependencies, and does not introduce a generalized provider-routing or governance platform.
- Minimal slice: PASS. The smallest independently valuable increment is a manifest-driven review council that can approve, reject, or escalate one delivery result with inspectable evidence.

## Project Structure

### Documentation (this feature)

```text
specs/007-multi-agent-review/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── review-profile-contract.md
│   ├── review-run-contract.md
│   ├── review-adjudication-contract.md
│   └── review-trace-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── execution.rs
│   ├── review.rs
│   ├── session.rs
│   ├── step.rs
│   ├── task.rs
│   └── trace.rs
├── fixture.rs
├── orchestrator/
│   ├── engine.rs
│   ├── planner.rs
│   └── session_runtime.rs
└── registry/
    ├── agent_registry.rs
    └── tool_registry.rs

tests/
├── contract/
│   ├── review_profile_contract.rs
│   ├── review_run_contract.rs
│   └── review_trace_contract.rs
├── integration/
│   ├── cli_review_run.rs
│   ├── cli_review_inspection.rs
│   └── session_review_flow.rs
├── support/
│   └── workspace_fixture.rs
└── unit/
    ├── review_profile.rs
    ├── review_voting.rs
    └── coverage_additional.rs
```

**Structure Decision**: Keep the work inside the existing crate and extend the current execution-engine surfaces instead of introducing a new review runtime. Add one new domain module for review councils, reviewer findings, participation states, vote resolution, and duplicate-trigger handling; extend `src/fixture.rs` so execution manifests can declare bounded review behavior and deterministic reviewer findings for the initial slice; wire review evidence into CLI and session runtime output; and expand unit, contract, and integration coverage to include review-specific failure handling and trace visibility.

## Complexity Tracking

No constitution violations require justification for this slice.
