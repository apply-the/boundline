# Contract: Provenance Output Projection

## Purpose

Define the observable CLI and trace contract for projecting hardened context
selection results.

## Surfaces

- `boundline plan`
- `boundline run`
- `boundline status`
- `boundline next`
- `boundline inspect`
- persisted session and trace summaries

## Required behavior

- Each surface must expose the authoritative context summary and credibility
  state when a goal plan or trace contains them.
- Context provenance must be rendered as explicit human-readable lines naming
  the selected input and why it was admitted.
- Primary context inputs must remain visible without reading raw JSON.
- Compatibility follow-up surfaces must preserve explicit compatibility
  authority while reusing the same provenance vocabulary.

## Consistency rules

- The same authoritative context pack must drive the summary shown on status,
  next, and inspect.
- Trace rendering must not invent provenance lines that were not present in the
  persisted plan or trace inputs.
- When the context is stale or insufficient, the surfaced output must include
  the stop or staleness reason if one exists.