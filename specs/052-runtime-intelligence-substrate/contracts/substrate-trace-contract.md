# Substrate Trace Contract

## Purpose

Trace events must carry enough substrate detail for `boundline inspect` and run summaries to reconstruct context state without re-planning.

## TaskStarted Payload

When available, the `input` payload should preserve:
- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance`
- `context_staleness_reason`
- requested governance fields such as runtime, risk, zone, and owner

## GoalPlanCreated Payload

When available, the payload should preserve the same substrate fields plus:
- goal-plan revision and state
- flow-state summary
- verification strategy
- planning rationale

## Governance Event Contribution

Governance events may extend trace reconstruction with optional Canon memory lines such as:
- `canon_memory_summary`
- `canon_memory_credibility`
- `canon_memory_compatibility`
- `canon_memory_run_ref`
- `canon_memory_packet_ref`
- `canon_memory_reason_code`
- `canon_next_action`

## Reconstruction Rules

- Read-side consumers should preserve the first bounded context summary unless a later event provides a more authoritative substrate-specific value.
- Provenance lines should be deduplicated while preserving useful order.
- Generated ids, timestamps, and trace-local metadata are not part of the determinism contract for context projection.
