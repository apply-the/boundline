# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.16.0

In addition to all v0.15.0 capabilities, the v0.16.0 release ships the full
`016-session-native-surface-unification` feature:

- **One Coherent Session View**: `run`, `status`, `next`, and `inspect` now
  project the same route explanation, `execution_condition`, decision summary,
  and next-step guidance for the primary session-native path.
- **Unified Optional Mode Projections**: review, adaptive execution, and
  governance state appear as bounded additions to the same session-owned summary
  instead of fragmenting the product story into separate runtime modes.
- **Explicit Compatibility Path**: direct manifest-backed `synod run --goal`
  remains supported and visibly labeled as compatibility behavior, while a ready
  session-native plan stays authoritative unless compatibility is requested
  deliberately.
- **Canon Compatibility Target Updated**: the documented supported Canon CLI
  target is now `0.24.0` while Canon remains bounded to governance and evidence
  overlays rather than the per-action control plane.

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