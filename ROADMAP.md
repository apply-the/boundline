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

- add a human-friendly `synod init` flow with provider and model routing configuration
- deepen Canon governance with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- deepen delivery and review beyond the current bounded local execution manifests
- multi-workspace and cross-repository orchestration
- advanced goal negotiation and constraint modeling

## Next Planning Slice: Human-Friendly Init and Model Routing

### Outcome

Synod should bootstrap a workspace without asking the operator to hand-author
`.synod/execution.json`, capture which assistant runtimes are available, and
persist editable model-routing defaults that can differ by execution step and
review role.

### Initial runtime support

- Claude
- Codex
- Copilot
- Gemini CLI only for the first slice, without a dedicated Synod client adapter yet

### In scope

- a `synod init` command that scaffolds workspace-local Synod files from bounded
  templates such as `bug-fix`, `change`, and `delivery`
- optional repo setup during init for supported assistant surfaces when the
  selected runtime needs repository-local files or prompts
- persisted provider and model defaults for major stages such as planning,
  implementation, verification, review, and adjudication
- distinct default reviewer or adjudicator model selection so voting councils do
  not have to reuse the same model profile as implementation steps
- CLI commands to inspect and modify the saved configuration later instead of
  forcing manual JSON edits
- a user-scoped global config plus workspace-local override, with local values
  taking precedence when both are present

### Out of scope

- hard-coding immutable provider or model choices that the operator cannot change later
- requiring every supported runtime to ship a rich native client adapter in the
  first slice
- removing bounded execution policy from Synod; init should scaffold it, not
  replace it with an unbounded free-form mode

### Configuration precedence

```text
CLI flags or explicit command choices
        ↓
workspace-local Synod config
        ↓
user-scoped global Synod config
        ↓
built-in Synod defaults
```

### Why this slice matters next

The current CLI already accepts human-authored goals and briefs, but workspace
setup is still manifest-first and too manual. Adding init plus editable
provider and model routing closes the biggest usability gap without weakening
the bounded execution model.

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