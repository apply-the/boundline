# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.11.0

In addition to all v0.10.0 capabilities, the v0.11.0 release ships the full
`011-init-model-routing` feature:

- **Human-Friendly Init**: `synod init` scaffolds `.synod/execution.json` and
  `.synod/config.toml` from bounded templates (`bug-fix`, `change`, `delivery`),
  with explicit preview and overwrite safeguards.
- **Model Routing Configuration**: `synod config show|set|unset` manages runtime
  and model defaults for planning, implementation, verification, review, named
  reviewer roles, and adjudication.
- **Global + Local Precedence**: effective routing resolves deterministically as
  CLI input > workspace config > global config > built-in defaults, and exposes
  value source in user-facing output.
- **Runtime Surfaces**: initial support covers Claude, Codex, Copilot, and
  Gemini CLI (CLI-only in this slice).

### Priority 1: Next Spec

The next spec should be a product realignment slice, not another incremental
framework layer. It should land the primary user experience as one coherent
session-native orchestration step instead of spreading the shift across many
small specs.

Priority outcomes for that next spec:

- make `start -> capture -> plan -> run -> status -> inspect` the primary UX and push `init` out of the user's everyday mental model
- shift planning away from static init templates toward plans derived from goal, workspace state, collected documents, and Canon-produced artifacts
- propose or infer flows automatically, with explicit override only when needed
- move agent adapters toward real read/modify/test/fix model-guided loops instead of declared static change sets
- keep Canon as governance and artifact control overlay, not the center of orchestration

Companion architecture review:

- [docs/session-native-orchestrator-review.md](docs/session-native-orchestrator-review.md)

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