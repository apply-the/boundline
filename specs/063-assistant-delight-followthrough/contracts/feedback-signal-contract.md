# Contract: Delight Feedback Signals

## Scope

This contract defines the lightweight usefulness signals introduced by S7.1 for
maintainer-facing review of the delight layer.

## Signals

| Signal | Meaning | Derived From |
| --- | --- | --- |
| `time_to_first_useful_answer` | Time from session start to the first delight response the operator treats as useful. | Session timestamps plus first recorded useful delight answer. |
| `explanation_attribution_rate` | Share of delight explanations that included explicit source attribution. | Recorded explanation attempts and attribution-complete results within the session or trace authority. |
| `next_action_acceptance_rate` | Share of proposed next actions that the operator accepted instead of overriding. | Recorded accepted and overridden next-action outcomes within the session or trace authority. |
| `latest_next_action_outcome` | Most recent operator response to a suggested next action. | Latest accepted, overridden, not-applicable, or unknown outcome. |

## Shared Invariants

- Signals MUST remain session-scoped and traceable to the same authority used by
  delight output.
- Signals MUST be inspectable through Boundline state or projections; they MUST
  not require unrelated log mining or external analytics systems.
- Missing signal data MUST be surfaced as `not-yet-recorded`, `not-applicable`,
  or equivalent explicit state rather than a silent zero.
- Signal capture MUST not introduce background jobs, hidden batching, or remote
  telemetry dependencies.

## Aggregation Rules

- `explanation_attribution_rate` is calculated from:
  - `attributed_explanations`
  - `total_explanations`
- `next_action_acceptance_rate` is calculated from:
  - `accepted_next_actions`
  - `overridden_next_actions`
- `time_to_first_useful_answer` is recorded when the first delight response
  produces a bounded next action that is later accepted without override in the
  same session; the stored command identifies the delight surface that produced
  that accepted next action.
- Rates are undefined until at least one qualifying event exists; the projection
  must disclose that state explicitly.

## Validation Rules

- A recorded first useful answer MUST include both its timestamp and the command
  or surface that produced it.
- Override reasons are only valid when the latest next-action outcome is
  `overridden`.
- Aggregated counters MUST never go negative and MUST stay internally
  consistent with the latest outcome projection.

## Projection Rules

- Status or inspect output MUST show enough detail for a maintainer to judge
  whether the delight layer is helping or producing noise.
- Signal projection MUST preserve source authority and any fallback conditions
  that reduce confidence in the result.