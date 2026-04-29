# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.15.0

In addition to all v0.14.0 capabilities, the v0.15.0 release ships the full
`015-runtime-refoundation` feature:

- **Session-Native Runtime Is Primary**: `start → capture → plan → run → status → inspect`
  is the default operator path, while `init` and `.synod/execution.json` remain
  available only as explicit compatibility/bootstrap surfaces.
- **Authoritative Bounded Task Drafts**: `plan` persists a bounded `GoalPlan`
  derived from captured input, workspace evidence, and bounded Canon context
  rather than replaying a static manifest as the primary product story.
- **Explicit Flow State**: planning now persists confirmed, proposed, skipped,
  or absent flow state so operators can confirm or skip inferred constraints
  instead of treating flow as hidden metadata.
- **Live Decision Contracts**: the runtime selects the next bounded action from
  current evidence and persists inspectable decisions with rationale,
  expected outcome, evidence inputs, action results, and timestamps.
- **Terminal Outcomes And Recovery Cues**: success, failure, exhaustion, and
  no-actionable-state outcomes are explicit in status, run, and inspect output,
  along with next-command guidance and failure evidence.
- **Compatibility Routing Is Visible**: direct manifest-backed `synod run --goal`
  remains supported, but Synod now renders that route as explicit compatibility
  behavior instead of letting it look like the primary runtime.
- **Canon Stays Bounded**: Canon remains a stage-boundary governance and
  evidence surface, not the per-action control plane for the decision loop.

### Secondary follow-up directions

- deepen Canon governance with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- migrate more review and compatibility configuration onto session-native session summaries without losing bounded manifest support
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