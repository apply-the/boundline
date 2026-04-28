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

### Immediate follow-up directions

- add a provider-agnostic model gateway inside Synod (no Openclaw runtime dependency), with first-class provider auth flows and capability discovery
- decouple assistant command packs from model backends so Claude/Codex/Copilot surfaces map to routing slots instead of hard-wired providers
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