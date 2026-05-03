# Implementation Plan: Canon Governance Adapter

**Branch**: `009-canon-governance-adapter` | **Date**: 2026-04-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-canon-governance-adapter/spec.md`

## Summary

Add a stage-scoped governance layer to Boundline's existing flow-aware runtime by extending the workspace execution manifest with governance policy, introducing a local-first `GovernanceRuntime` abstraction with an optional Canon CLI adapter, and projecting governed stage state into the same session, trace, and inspect surfaces used by adaptive execution and review. The smallest shippable slice keeps Boundline as the delivery orchestrator, governs only meaningful built-in flow stages, records Canon run references and packet readiness when Canon is enabled, resolves approval only through later `status`, `step`, or `run` refreshes, blocks explicitly when governance is required but cannot proceed, and optionally lets autopilot choose among bounded compliant governance paths without bypassing approval or Canon guardrails. The release target for this slice is `0.9.0`.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and process APIs; no new runtime dependencies for the initial governance slice  
**Storage**: Workspace-local JSON session record at `<workspace>/.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, workspace execution manifest at `<workspace>/.boundline/execution.json`, and optional Canon-managed governed artifacts under `<workspace>/.canon/` when the Canon runtime is selected  
**Testing**: `cargo test --all-targets`, focused contract and integration coverage for execution manifest parsing, governance runtime selection, session projections, trace rendering, required-governance blocking, autopilot decisions, `cargo fmt --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings`  
**Target Platform**: macOS and Linux developer workstations plus Linux CI, with Canon CLI invoked only when installed and enabled for the workspace  
**Project Type**: Single Rust CLI crate with file-backed session and trace persistence plus repository-managed assistant assets  
**Execution Model**: Sequential flow-aware execution with one active stage at a time, one explicit governance decision per governed stage boundary, one local or Canon governance runtime selected for that boundary, packet reuse limited to same-stage reruns or the immediately previous stage in the same built-in flow, and bounded autopilot decisions that may continue, await approval, retry governance, escalate, or block explicitly  
**Observability Surface**: Persisted execution traces, session `status` and `next` output, `run` terminal rendering, `inspect` summaries, governance state patch fields in task context, Canon run references, governed packet readiness markers, approval state, autopilot decision rationale, candidate actions, and packet reuse provenance  
**Performance Goals**: Governance selection and local runtime execution remain interactive for built-in flows, Canon-backed governance adds only one bounded CLI round-trip per governed stage, and status or inspect rendering stays fast enough for normal CLI use  
**Constraints**: Preserve Boundline's local-first execution path, keep governance optional unless marked required, avoid direct dependency on Canon internals, use only Canon 0.18.0 modes available in the current slice, do not bypass approval-gated or recommendation-only boundaries, keep decisions explicit in traces, and avoid new background workers, distributed execution, UI work, or deployment scope  
**Scale/Scope**: One active workspace session, one governance runtime selection per governed stage, at most one Canon run per governed stage boundary, one bounded governed packet lineage per stage, and a small bounded set of autopilot choices per decision point

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature directly improves bounded engineering task delivery by governing explicit delivery stages without changing Boundline into a generic external orchestration shell. See Summary and Technical Context.
- Delivery-first scope: PASS. The plan prioritizes stage execution control, policy enforcement, packet reuse, and explicit blocking ahead of polish. See Summary, Technical Context, and Project Structure.
- Bounded execution: PASS. Governance runs happen only at explicit stage boundaries, autopilot chooses from a bounded set of compliant actions, and required-governance failures terminate explicitly instead of looping indefinitely. See Technical Context and research decisions.
- Stateful execution: PASS. Governance lifecycle state, Canon references, packet readiness, approval status, and autopilot decisions are persisted through task context and projected into session and trace surfaces. See Technical Context and data model.
- Mutable planning: PASS. The runtime may retry a governed stage, escalate to a governed verification path, or rebind a valid mode, but each mutation stays explicit and tied to stage evidence. See Summary and research decisions.
- Sequential-first design: PASS. Only one stage and one governance runtime are active at a time. There is no hidden concurrency, background governance queue, or parallel multi-stage execution. See Technical Context.
- Tool-agent symmetry: PASS. Governance decisions, Canon invocation, local fallback, approval blocking, and later coding or validation remain explicit runtime or step transitions rather than hidden internal behavior. See Project Structure.
- Observability and explicit intelligence: PASS. Runtime selection, mode binding, approval state, packet readiness, and autopilot rationale are surfaced through state patches, session views, and new trace events. See Technical Context and contracts.
- Non-goals and external separation: PASS. Canon remains optional behind a local-first runtime, so Boundline stays independently testable and executable without Canon. The plan excludes direct Canon internals, UI work, deployment, and unbounded agent autonomy. See Summary, Technical Context, and Scope Boundaries in the spec.
- Minimal slice: PASS. The smallest independently valuable increment is local-first stage governance with optional Canon-backed execution, explicit required-governance blocking, packet readiness checks, and inspectable autopilot decisions for built-in flows. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/009-canon-governance-adapter/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── canon-runtime-contract.md
│   ├── governance-execution-profile-contract.md
│   ├── local-governance-runtime-contract.md
│   ├── governance-session-contract.md
│   └── governance-trace-contract.md
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
│   ├── execution.rs
│   ├── flow.rs
│   ├── governance.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── fixture.rs
├── orchestrator/
│   ├── engine.rs
│   ├── governance.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
│   ├── canon_runtime_contract.rs
│   ├── governance_execution_profile_contract.rs
│   ├── local_governance_runtime_contract.rs
│   ├── governance_session_contract.rs
│   └── governance_trace_contract.rs
├── integration/
│   ├── canon_governance_flow.rs
│   └── governance_autopilot_flow.rs
└── unit/
    ├── canon_stage_mapping.rs
    ├── governance_policy.rs
    └── governance_runtime.rs
```

**Structure Decision**: Keep the feature inside the existing crate and extend the current flow-aware fixture runtime rather than introducing a second execution engine or a second workspace manifest. Add a new domain module for governance policy and state, add one adapter module for the local and Canon governance runtimes, add one orchestrator module to coordinate stage governance boundaries, and extend the existing session, trace, output, and inspect code paths so governance uses the same persistence and observability surfaces as adaptive execution and review. Approval refresh reuses the existing `status`, `step`, and `run` commands instead of adding a second approval command surface. This keeps the slice sequential, local-first, and independently testable when Canon is absent.

## Complexity Tracking

No constitution violations require justification for this slice.
