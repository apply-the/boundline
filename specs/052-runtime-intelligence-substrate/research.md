# Research: Runtime Intelligence Substrate

## Implementation Status

This document records the landed 052 slice as implemented in the current
workspace. Feature closeout is complete, and `validation-report.md` now records
the final coverage gate plus the cross-artifact review for the delivered slice.

## Current Implementation Surfaces

- `src/domain/goal_plan.rs` already carries a persisted `ContextPack` with summary, credibility, primary inputs, selected targets, and optional staleness reason.
- `src/orchestrator/goal_planner.rs` already builds bounded planning context from workspace signals, authored input, negotiation state, recent trace hints, and optional Canon capability or memory inputs.
- `src/cli/session.rs` already projects context credibility, summary, provenance, and recommended next command into the session-native `status` and `next` surfaces.
- `src/cli/inspect.rs` already summarizes trace payload fields such as `context_summary`, `context_credibility`, `context_primary_inputs`, `context_provenance`, and `context_staleness_reason`.
- `src/cli/output.rs` already renders those fields in the run and inspect summaries.

## Boundaries Confirmed During Implementation

- The substrate stays local-first: Boundline can construct a credible context pack without Canon.
- Canon remains optional enrichment: Canon capabilities and compacted memory extend context selection and provenance, but they do not replace local planning control flow.
- Session-native surfaces are the correct read-side owner for this feature because they already expose `status`, `next`, and `inspect` without introducing a second runtime UI.

## Changes Added In This Slice

- `ContextInput::provenance_line()` now carries the input `source` label so context provenance distinguishes not only the selected reference and rationale, but also where that evidence came from.
- Active Boundline compatibility surfaces now target Canon `0.51.0` instead of `0.50.0` across code, docs, fixtures, and distribution metadata.
- Active release metadata was also realigned to Boundline `0.52.0` where the current distribution surfaces still lagged on `0.51.1`.

## Credibility Vocabulary

The implemented substrate stores credibility as `credible`, `stale`, or
`insufficient`. Warning, refresh, replan, and terminal stop are behavioral
responses derived from those stored values and projected through runtime
surfaces.

## Implementation Direction

- Keep extending the existing `ContextPack` and trace projections instead of introducing a second substrate structure.
- Prefer additive projection refinements that preserve current `status`, `next`, and `inspect` behavior.
- Use trace payload contracts as the stable bridge between planning/runtime state and CLI output.
