# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.20.0

Synod now has its core session-native orchestration baseline, bounded workflow
follow-through, and a deeper governed-stage slice in place:

- session-native orchestration remains the primary operator path
- `workflow list`, `workflow run`, `workflow status`, `workflow resume`, and `workflow inspect` now project named workflow state onto the same session, route, trace, and `execution_condition` surfaces
- bounded `review` and `govern` phases are now executable from the workflow surface, stopping in explicit paused, blocked, failed, or completed states instead of remaining declaration-only blockers
- workflow definitions live in workspace-local `.synod/workflows.toml`, can ship optional discovery metadata, and remain bounded to existing Synod phases instead of becoming a generic workflow DSL
- direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- Canon is still integrated as a bounded stage-boundary governance runtime, now including governed `bug-fix:investigate` on the primary route plus later verify-stage `security-assessment` reuse

## Next Priority: Broaden Adaptive Repair Depth

The next slice should build on `0.20.0` by broadening adaptive repair depth
without widening Synod into a general-purpose workflow engine or letting Canon
take orchestration ownership.

The next spec direction should explicitly deliver three things:

- stronger adaptive heuristics beyond deterministic local repair patterns
- the same session-native and workflow-aware route story when adaptive retries, reviews, and governance all coexist
- continued bounded orchestration ownership inside Synod without hidden background progression

### Priority rationale

- this deepens real delivery value on the existing primary route instead of adding another surface for operators to reason about
- it builds on the now-stable governed-stage and workflow projections instead of reopening orchestration ownership questions
- it keeps Synod authoritative for orchestration while making bounded retries more credible on real repository work

### Delivered in 0.20.0

- govern `bug-fix:investigate` on the primary session-native route while preserving later governed verify reuse
- refresh approval state and blocked guidance through `status`, `next`, `inspect`, and the workflow-aware surfaces
- keep inspect summaries usable for paused or blocked governance traces instead of failing on non-terminal evidence

### Delivered in 0.19.0

- `synod workflow list` exposes named workflows, phase chains, summary text, and invocation guidance from `.synod/workflows.toml`
- `synod workflow run <name>` and `resume` can now carry bounded `review` and `govern` phases to explicit paused, blocked, failed, or completed outcomes on the existing session-native route
- workflow progress persists in `.synod/session.json` and reuses `.synod/traces/` while direct session-native commands and explicit compatibility routing remain available when no named workflow is invoked
- authored workflow registries now have clearer bounded guidance through optional `summary` and `recommended_when` metadata plus updated operator and assistant docs

Representative workflow registry shape:

```toml
[workflow.governed-delivery]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "review", "govern", "inspect"]
allow_review = true
allow_governance = true
summary = "bounded delivery path with review and governance before completion"
recommended_when = "the task needs explicit review and governance evidence"

[workflow.governed-delivery.when]
review = "review_triggered"
governance = "governance_required"

[workflow.governed-delivery.output]
next_command = true
routing_summary = true
execution_condition = true
```

### Secondary follow-up directions

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