# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.13.0

In addition to all v0.12.0 capabilities, the v0.13.0 release ships the full
`013-session-native-orchestrator` feature:

- **Session-Native Orchestration**: `start → capture → plan → run → inspect` is
  now the primary UX, pushing `init` to an optional/advanced path.
- **Goal-Derived Planning**: plans are derived from goal text, workspace state,
  collected documents, and Canon-produced artifacts via `GoalPlan` model.
- **Flow Inference**: flows are proposed automatically from goal keywords
  (bug-fix, change, delivery) with explicit `--flow` / `--no-flow` overrides.
- **Bounded Decision Loop**: observe→decide→act→verify→update execution engine
  produces typed, inspectable `Decision` objects with structured `ToolResult`
  outputs, bounded step limits, and explicit terminal states (Success, Failure,
  Exhausted, NoActionableState).
- **Flow as Policy**: `FlowPolicy` constrains decision types per flow stage
  (e.g., only Analyze in investigate stage, only Code/Fix in implement).
- **Decision Trace Events**: DecisionCreated, DecisionDispatched,
  DecisionVerified, DecisionFailed, DecisionRecovered, GoalPlanCreated,
  FlowInferred trace events provide full decision-level observability.

### Secondary follow-up directions

- deepen Canon governance with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- deepen delivery and review beyond the current bounded local execution manifests
- expand the new multi-workspace cluster slice into full cross-repository execution planning and mutation
- advanced goal negotiation and constraint modeling

To be challenged:
- add a provider-agnostic model gateway inside Synod, with first-class provider auth flows and capability discovery
- decouple assistant command packs from model backends so Claude/Codex/Copilot surfaces map to routing slots instead of hard-wired providers

## Architecture: User Through Execution

```text
User / Copilot / Claude
        ↓
      Synod
  ┌───────────────┐
  │ Orchestrator  │
  │ Flows         │
  │ Agents        │
  │ Execution     │
  │ Review        │
  │ Adaptive      │
  └───────────────┘
        ↓
     Canon
   (governed stage docs + artifact persistence)
```

## In One Sentence

Synod is a system that takes a problem and transforms it into working code, orchestrating bounded execution itself while using Canon to govern stage outputs and provide reusable documentation.