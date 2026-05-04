# Contract: Insufficient Context

## Purpose

Define how Boundline behaves when context selection cannot justify a credible
bounded target.

## Trigger conditions

- Only weak keyword or path similarity exists.
- Available evidence is stale or contradicted by newer validation output.
- A candidate file lies outside the active workspace or cluster scope without a
  direct evidence anchor.

## Required behavior

- `boundline plan` must stop explicitly with an insufficient or stale context
  state.
- The resulting plan/session story must preserve the context summary,
  credibility, provenance collected so far, and one bounded recovery cue.
- `boundline status`, `boundline next`, and `boundline inspect` must project
  the same non-credible story instead of silently reverting to a generic
  fallback context.

## Errors

- If a previously selected input becomes invalid or scope-unsafe, Boundline must
  exclude it from the authoritative context or downgrade the pack credibility.
- If no bounded recovery action is known, Boundline must still surface the stop
  condition explicitly.