# Contract: Session Input Observability

## Purpose

Define how the accepted human-authored brief, source provenance, clarification state, and governance intent appear through Synod's existing session, inspect, and trace surfaces.

## `status` Surface

- `status` must expose a compact input summary for the active session once human input has been captured.
- The summary must include enough information for the operator to confirm the accepted brief without reading raw JSON.
- The summary must identify when the accepted input came from direct text only, direct text plus Markdown briefs, or Markdown briefs only.
- When governance intent is present, `status` must show that governed execution was requested and whether execution is blocked or awaiting approval.
- When planning is blocked for clarification, `status` must show the latest clarification headline and the next recommended action.

## `inspect` Surface

- `inspect` must expose the ordered set of accepted input sources for the active session or trace target.
- Each source must identify its kind, human-visible name, workspace path when applicable, and accepted precedence.
- `inspect` must show whether a source was deduplicated or ignored because it duplicated an earlier accepted document.
- `inspect` must expose the normalized brief summary, any open clarification, and the accepted governance business values.
- `inspect` must not require the operator to understand internal manifest fields to interpret the active input state.

## Trace Expectations

- The execution timeline must show when human input was captured, when source resolution completed, when clarification blocked planning, and when governance intent was mapped into governed execution.
- Trace output must name offending sources when normalization fails because of missing, unreadable, unsupported, or conflicting documents.
- Trace output must keep source-order and deduplication decisions visible enough to explain why one document won precedence over another.

## Projection Rules

- Session and inspect projections must derive their input state from the same normalized brief bundle used for planning and execution.
- A resumed session must continue to show the same accepted source summary until new human input is captured successfully.
- If new human input is rejected during normalization, the previously accepted bundle remains the authoritative state.
- Governance intent projection must remain human-facing and must not expose internal stage IDs or Canon packet bindings unless the operator explicitly asks for advanced detail through existing inspection tooling.