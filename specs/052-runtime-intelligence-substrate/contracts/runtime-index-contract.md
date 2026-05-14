# Runtime Index Contract

## Purpose

The runtime index is the operator-visible summary of the bounded context currently steering planning or execution. It must be derivable from persisted session or trace state without re-running planning.

## Required Fields

The runtime index must be able to project:
- `context_summary`: concise bounded-context headline.
- `context_credibility`: one of `credible`, `stale`, or `insufficient`.
- `context_primary_inputs`: primary references or fallback selected targets.
- `context_provenance`: ordered provenance lines for local and optional Canon inputs.
- `context_staleness_reason`: present when credibility is `stale`.
- `suggested_next_command`: if credibility is not `credible`, the operator-facing next action must narrow or refresh context before free-form continuation.

## State Mapping

The runtime index stores credibility as one of three values:
- `credible`
- `stale`
- `insufficient`

Those stored values drive operator-visible behavior:
- `stale` maps to warning or refresh guidance
- `insufficient` maps to bounded stop or replan guidance
- terminal behavior is a runtime outcome emitted when an insufficient path cannot recover safely

## Provenance Rules

- Provenance lines must preserve the input kind.
- Provenance lines must preserve the operator-facing reference.
- Provenance lines must preserve the rationale for inclusion.
- Provenance lines must preserve the input `source` label when the input originated from `ContextInput`.
- Canon memory provenance may contribute additional `canon_memory_*` lines when a compacted governed packet is present.

## Canon Boundary

- Canon is optional enrichment, not a prerequisite for substrate construction.
- A credible local context pack is sufficient for runtime-index construction.
- Canon capability or memory inputs may extend provenance and target selection, but they must not replace local session ownership.
