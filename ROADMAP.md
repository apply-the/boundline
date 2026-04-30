# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.18.0

Synod now has its core session-native orchestration baseline plus the first
bounded workflow layer in place:

- session-native orchestration remains the primary operator path
- `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` now project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- workflow definitions live in workspace-local `.synod/workflows.toml` and remain bounded to existing Synod phases instead of becoming a generic workflow DSL
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- Canon is still integrated as a bounded stage-boundary governance runtime, including verify-stage `security-assessment`

## Next Priority: Deepen Bounded Workflow Follow-Through

The next slice should build on `0.18.0` by making more of the bounded
workflow-phase story executable without widening Synod into a general-purpose
workflow engine.

The next spec direction should explicitly deliver three things:

- bounded execution semantics for `review` and `govern` phases from the workflow surface
- stronger assistant ergonomics around named workflow discovery and invocation without forking the primary CLI story
- clearer operator guidance for authored workflow registries, including reusable examples and migration notes

### Priority rationale

- this keeps workflow support bounded and useful without reopening the generic workflow-engine problem
- it deepens the operator value of the new workflow surface instead of leaving `review` and `govern` as declaration-only phases
- it improves assistant and contributor ergonomics now that `.synod/workflows.toml` is a real product surface

### Delivered in 0.18.0

- `synod workflow run <name>` starts a named workflow against the existing session-native runtime
- `synod workflow status`, `resume`, and `inspect` expose workflow identity, current phase, routing, `execution_condition`, and next-command guidance
- workflow progress persists in `.synod/session.json` and reuses `.synod/traces/`
- unsupported bounded-semantics are rejected explicitly instead of silently widening the workflow model

Representative workflow registry shape:

```toml
[workflow.default]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "inspect"]
allow_review = true
allow_governance = true

[workflow.default.output]
next_command = true
routing_summary = true
execution_condition = true
```

### Secondary follow-up directions

- deepen Canon governance beyond the first `security-assessment` slice with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- migrate more review and compatibility configuration onto session-native summaries without losing bounded manifest support
- expand multi-workspace cluster support into full cross-repository execution planning and mutation
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