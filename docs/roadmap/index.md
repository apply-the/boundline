# Roadmap

Welcome to the official **Boundline** Roadmap. Here we track the evolution of our *Bounded Cognitive Runtime*.

::: info Vision
The goal of Boundline is to transform non-deterministic AI iterations into predictable, traceable, and governable software delivery processes.
:::

## Recently Delivered

- `070` - Large Codebase Context Substrate: shipped the local large-codebase
  context substrate in Boundline `0.71.0`. Planning now projects fidelity
  tiers, inclusion modes, omission findings, repository-map readiness,
  digest-backed compaction, patch-safe edit guards, and freshness-bound derived
  snapshot-cache state instead of silently widening context reads.
- `069` - Plan Analysis Contract: shipped the read-only planning-analysis gate
  in Boundline `0.70.0`. After plan quality and backlog quality are ready, the
  runtime now validates cross-artifact coherence across goal outcomes,
  validation coverage, selected backlog slice sequencing, execution-handoff
  inputs, and governed producer evidence before execution is offered.
- `068` - Backlog Contract: shipped the first formal backlog-quality gate in
  Boundline `0.69.0`. After plan quality is ready, the runtime now blocks
  closure-limited Canon backlog packets, requests one clarification when a full
  packet still lacks execution-handoff evidence, and projects additive
  backlog-quality state through status, inspect, orchestration, and assistant
  surfaces.

## <i class="fa-solid fa-rocket" style="color: #38c7ff;"></i> Upcoming Features & Topics

### Context Handling & Execution
- **Large Codebase Context Substrate**: Handling long-term context limits, lazy hash references for huge files, and anchored hunks for large-file edits.
- **Sandboxed Execution & Secret Inheritance**: Local safety boundaries, path/network policies, and read-only/dry-run mutation modes.

### Provider Ecosystem
- **External Capability Provider Protocol**: A native provider contract replacing MCP, defining explicit permissions, health checks, and evidence collection.
- **Open Model Provider Support**: Treating Qwen, Gemma, and Llama models as open-weight families exposed through secure provider adapters.
- **AI Gateway & Inference Economics**: Managing route health, local vs remote transmission decisions, route budgets, and latency telemetry.

### Governance & Planning
- **Backlog Contracts**: Strict execution gates for backlog execution.
- **Evals & Runtime Observability**: Local quality layers, JSONL trace exports, deterministic scoring, and provider evaluations.
- **Recursive Stage Refinement Profiles**: Sequential planning profiles (planner -> critic -> finalizer) tightly governed by councils and stop semantics.

---

> Do you have suggestions? Open an issue on our GitHub repository and help us shape the future of Boundline!
