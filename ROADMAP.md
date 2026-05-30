# Boundline Roadmap

Canon is downstream from Boundline in this roadmap: Boundline thinks, decides,
orchestrates, and executes, while Canon governs meaningful flow stages and
persists structured artifacts that Boundline can reuse for reasoning.

Delivered release history belongs in [CHANGELOG.md](CHANGELOG.md). This file is
for current direction, future feature sequencing, and product boundaries.

## Current Status: v0.65.0

Boundline currently ships the session-native CLI/runtime plus the sqlite-vec
derived-index lifecycle surface: bounded local semantic retrieval, explicit
manifest-backed `index` maintenance commands, hook-aware stale detection, and
diagnostic recovery guidance all remain owned by the CLI and persisted
workspace state. There is still no separate terminal UI product line on the
forward roadmap.

### Delivered in 0.65.0

- sqlite-vec-backed local semantic retrieval over the single derived SQLite
  store is now active, with explicit fallback and bounded authority order
  preserved on normal runtime surfaces
- `boundline index status|refresh|rebuild|clean|doctor` now provides manifest-
  backed lifecycle control, incremental refresh, and tracked-artifact or
  corruption diagnosis
- derived-index hygiene now includes managed manifest plus WAL/SHM ignore rules,
  optional stale-mark Git hooks, and probe or diagnostics visibility into index
  health and hook state

### Delivered in 0.64.0

- session-native orchestration remains the primary delivery surface
- assistant command packs stay aligned with the CLI and trace-backed status model
- governed delivery, inspection, and distribution workflows ship on the current workspace version

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
2. Operator surfaces must remain thin shells over the existing CLI/runtime, not
  second products or parallel orchestration engines.
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
