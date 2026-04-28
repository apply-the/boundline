# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.10.0

In addition to all v0.9.0 capabilities (session, flows, execution, review,
adaptive, assistant integration, governance), the v0.10.0 release ships the
full `010-human-brief-ingestion` feature:

- **Human Brief Ingestion**: `synod capture` and `synod run` accept direct
  text, repeated `--brief <path>.md` arguments, and business-level governance
  intent, normalizing them into one inspectable authored brief bundle.
- **Multi-Source Provenance**: explicit briefs and Markdown paths referenced in
  goal text resolve in stable precedence order, deduplicate deterministically,
  and remain visible through `status`, `inspect`, and trace summaries.
- **Clarification and Governance Routing**: unbounded requests stop with one
  explicit clarification before planning, while governed runs surface blocked
  or approval-gated next actions through the existing session surfaces.

### Immediate follow-up directions

- deepen Canon governance with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- deepen delivery and review beyond the current bounded local execution manifests
- multi-workspace and cross-repository orchestration
- advanced goal negotiation and constraint modeling

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