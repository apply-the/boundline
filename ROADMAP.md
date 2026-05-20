# Boundline Roadmap

Canon is downstream from Boundline in this roadmap: Boundline thinks, decides,
orchestrates, and executes, while Canon governs meaningful flow stages and
persists structured artifacts that Boundline can reuse for reasoning.

Delivered release history belongs in [CHANGELOG.md](CHANGELOG.md). This file is
for current direction, future feature sequencing, and product boundaries.

## Current Status: v0.64.0

Boundline is currently shipping the `0.64.0` interactive delivery dashboard
line while keeping the CLI and session-native runtime authoritative.

### Delivered in 0.64.0

- a separate `boundline-dashboard` workspace component with local terminal UI
  dependencies isolated from the normal CLI path
- typed dashboard snapshots over existing session, trace, checkpoint, finding,
  and governed-reference projections
- first-screen state, timeline, actions, panels, diagnostics, and degraded
  fallbacks that point back to normal Boundline commands
- terminal-safe `boundline` wordmark branding without SVG, raster, or wide ANSI
  banner dependencies

## Objective

Evolve Boundline into a system capable of taking a problem and transforming it
into working code with bounded execution, inspectable reasoning, and multi-role
quality control.

## Current Baseline

Boundline already has the primary delivery substrate in place:

- session-native orchestration and trace-backed status surfaces
- bounded planning, execution, review, governance, recovery, and inspection
- Canon-aware governed delivery without making Canon the runtime owner
- local context intelligence with SQLite and FTS5 retrieval plus semantic
  acceleration
- guidance catalog packs, guardian findings, authority-zoned councils, adaptive
  governance, and reasoning-profile support
- assistant command surfaces across supported hosts
- release-aligned distribution metadata and install diagnostics

Future roadmap items should extend this baseline rather than re-describe shipped
capabilities as new features.

## Forward Roadmap

The repo-local `roadmap/` folder carries the active forward-looking drafts:

- [Next Boundline Roadmap](roadmap/Next%20-%20forward-roadmap.md)
  absorbs the next forward-looking work into Boundline. It prioritizes large
  codebase context hardening, external capability providers including open-model
  adapters, evals and runtime observability, Boundline help-next, guidance
  activation hardening, council and adaptive-governance hardening, sandboxed
  execution, MCP adapters, AI gateway economics, browser validation providers,
  and trace-linked memory hygiene.
- [TD-001: Complexity Hotspots And Structural Debt](roadmap/TD-001-complexity-hotspots.md)
  remains the active structural-debt watchlist for oversized Rust files and long
  functions that should be reduced during future feature slices.

## Sequencing Rule

1. Boundline must deliver visible runtime trust before more platform abstraction
   work.
2. The dashboard remains an operator shell over the existing CLI/runtime, not a
   second product.
3. Large-codebase handling, provider permissions, and evals must precede
   stronger autonomy.
4. Canon must prove value inside the real delivery loop, not beside it.
5. MCP, browser automation, and AI gateway work happen as adapter or scale
   layers after Boundline-owned permissions and trace semantics are stable.

## Product Boundary

Canon-exclusive roadmap work is intentionally not listed as a Boundline feature.
Canon mode templates, Canon packet-quality validation, Canon `help-next`, Canon
MCP server implementation, and Canon project-memory promotion rules belong in
Canon. Boundline consumes those outputs only through stable metadata contracts
such as readiness state, evidence refs, lineage refs, approval state, and
project-memory promotion status.

## Architecture

```text
User / Copilot / Claude / Codex / Cursor / Gemini
        |
        v
    Boundline
  - Orchestrator
  - Flows
  - Agents
  - Execution
  - Review
  - Adaptive governance
        |
        v
     Canon
  - governed stage docs
  - artifact persistence
  - reusable project knowledge
```

## In One Sentence

Boundline takes a problem and transforms it into working code by orchestrating
bounded execution itself while using Canon to govern stage outputs and provide
reusable documentation.
