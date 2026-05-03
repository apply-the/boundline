# Research: Delivery Orchestrator Core

## Decision 1: Use a synchronous sequential orchestrator core

- **Decision**: Implement the first orchestrator core as a synchronous, sequential state machine in a Rust library crate.
- **Rationale**: The feature spec explicitly prioritizes sequential execution, deterministic stop conditions, and easy debugging. A synchronous core minimizes hidden concurrency and keeps control flow inspectable.
- **Alternatives considered**:
  - Async executor core: rejected for v1 because it adds complexity before parallel execution is in scope.
  - Graph scheduler: rejected because the feature explicitly excludes parallel or graph-based scheduling.

## Decision 2: Keep active state in memory and persist traces to local files

- **Decision**: Store active task state in memory during execution and write execution traces to local append-only JSON files after each meaningful event.
- **Rationale**: The spec requires inspectable traces but excludes Canon integration and long-term cross-task memory. Local files satisfy inspectability without introducing external persistence infrastructure.
- **Alternatives considered**:
  - In-memory traces only: rejected because failures would become hard to inspect after process exit.
  - Database-backed persistence: rejected as unnecessary infrastructure for the first orchestration slice.
  - Canon-backed persistence: rejected because the spec explicitly excludes Canon integration details.

## Decision 3: Model recovery as a central task policy

- **Decision**: Manage retry and replanning through a central recovery policy that tracks task-level budgets and decides between retry, replan, and terminal failure after each non-successful step.
- **Rationale**: Recovery policy is easier to reason about when the orchestrator owns all limits and precedence rules. This keeps termination deterministic and prevents agent-specific behavior from creating hidden loops.
- **Alternatives considered**:
  - Per-agent retry rules: rejected because they fragment control and weaken inspectability.
  - Unlimited local retries: rejected because the spec requires bounded retries and bounded replanning.

## Decision 4: Use named registries with a uniform endpoint contract

- **Decision**: Route agent and tool steps through separate named registries, but require both to conform to one shared execution envelope for inputs, outputs, and failure metadata.
- **Rationale**: The orchestrator must select endpoints by name and preserve one shared view of step execution. A common envelope makes trace recording and recovery policy consistent across endpoint types.
- **Alternatives considered**:
  - Hardcoded analyzer/coder/tester branches: rejected because later flows need extensibility.
  - One combined registry with no endpoint kind distinction: rejected because agents and tools have different operational roles and should remain explicit in traces.

## Decision 5: Keep the crate split by domain, orchestration, registry, and adapters

- **Decision**: Organize the crate into `domain`, `orchestrator`, `registry`, and `adapters` modules.
- **Rationale**: This keeps pure state and policy separate from execution logic and from environment-facing adapters. The boundaries match the feature: models, loop control, lookup, and integration.
- **Alternatives considered**:
  - Single-module crate: rejected because retry, replanning, and trace logic would become hard to reason about.
  - Multi-crate workspace: rejected because the repository has no existing source tree and the feature is still a minimal v1 slice.

## Decision 6: Define contracts as markdown interface specifications

- **Decision**: Document orchestrator-facing interfaces in markdown contract files with canonical field sets, invariants, and lifecycle guarantees.
- **Rationale**: The project does not yet expose an HTTP API or CLI contract for this feature, but later flows and tests still need stable semantics. Markdown contracts are sufficient and easy to evolve while the crate surface is still forming.
- **Alternatives considered**:
  - OpenAPI or RPC schema files: rejected because this feature is not a network service.
  - Rustdoc only: rejected because the codebase does not exist yet and planning needs interface clarity before implementation.

## Decision 7: Use focused Rust dependencies instead of a workflow framework

- **Decision**: Limit initial dependencies to `serde`, `serde_json`, `thiserror`, `tracing`, and `uuid`.
- **Rationale**: The feature needs structured state, serialization, diagnostic traces, typed errors, and stable identifiers, but it does not yet need a full workflow engine or actor runtime.
- **Alternatives considered**:
  - Standard library only: rejected because structured trace serialization and stable identifiers would become unnecessarily ad hoc.
  - Workflow or actor frameworks: rejected because they would overfit a minimal sequential orchestrator.

## Decision 8: Validate behavior with deterministic fake endpoints

- **Decision**: Use unit, integration, and contract-style tests driven by fake agent and tool endpoints that return predetermined outputs and failure modes.
- **Rationale**: Deterministic fakes are the fastest way to prove sequential progression, retry behavior, replanning, and trace capture without depending on real model providers or shell tools.
- **Alternatives considered**:
  - End-to-end tests against live providers: rejected because they are too nondeterministic for core control-loop validation.
  - Unit tests only: rejected because orchestration behavior depends on interactions between planning, execution, recovery, and tracing.

## Decision 9: Use the constitution as a hard scoping gate

- **Decision**: Keep this feature inside the ratified Boundline constitution by limiting scope to sequential, stateful, inspectable delivery orchestration and deferring councils, provider complexity, long-term memory, and Canon-coupled behavior.
- **Rationale**: The constitution now defines delivery-first scope and explicit non-goals, which makes the plan easier to defend and less likely to drift into abstract framework work.
- **Alternatives considered**:
  - Expand this slice to include strategy or provider-routing work: rejected because the constitution defers those capabilities.
  - Leave scope enforcement informal: rejected because soft guidance would allow the same drift the constitution was created to prevent.
