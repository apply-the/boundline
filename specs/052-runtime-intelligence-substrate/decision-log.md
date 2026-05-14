# Decision Log: Runtime Intelligence Substrate

## D-001: Reuse The Existing ContextPack

Status: accepted

Boundline already persists a bounded `ContextPack` in the goal plan. The feature should extend that structure and its read-side helpers instead of introducing a second substrate model.

## D-002: Keep Canon Optional

Status: accepted

Canon capability and memory inputs may enrich context selection and provenance, but Boundline must remain able to plan and project context without Canon control flow.

## D-003: Use Session-Native Surfaces As The Read Side

Status: accepted

`status`, `next`, `run`, and `inspect` already expose context summary, credibility, primary inputs, and provenance. The feature should continue to deepen those surfaces rather than add a parallel UI.

## D-004: Preserve Input Source In Provenance

Status: accepted

`ContextInput` already stores `source`, but the operator-facing provenance line previously discarded it. The substrate now surfaces `source` in provenance lines so local versus Canon and scanner-derived inputs remain inspectable.

## D-005: Align The Active Canon Target To 0.51.0

Status: accepted

Boundline active compatibility, docs, fixtures, and distribution metadata now target Canon `0.51.0` to match the updated Canon release contract.

## D-006: Separate Credibility States From Behavioral Outcomes

Status: accepted

The runtime substrate stores credibility as `credible`, `stale`, or
`insufficient`. Operator-visible outcomes such as warning, refresh, replan, and
terminal stop are behaviors derived from those state values rather than extra
stored credibility states.
