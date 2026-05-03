# Contract: Review Adjudication and Failure Semantics

## Purpose

Define when bounded adjudication runs, how it resolves `needs_adjudication`, and how review distinguishes `escalated` from `failed`.

## Adjudication trigger contract

Adjudication runs only when all of the following are true:

- the review phase already completed reviewer collection
- vote resolution returned `needs_adjudication`
- review configuration has `adjudication.enabled = true`
- an adjudicator is configured and available

The initial slice supports exactly one adjudication step with no retries and no nested review.

## Bounded adjudication contract

Bounded adjudication means:

- one adjudicator only
- one adjudication finding only
- no retries
- no second-round council
- no additional vote after adjudication

## Decision mapping

The adjudication finding maps directly to the terminal review outcome:

- `approve` -> `accepted`
- `block` -> `rejected`
- `concern` -> `escalated`

## Failure versus escalation contract

Review ends as `failed` when:

- a required reviewer is unavailable
- the adjudicator is unavailable when adjudication is required
- reviewer or adjudicator output is malformed
- review configuration is invalid at runtime

Review ends as `escalated` when:

- the vote returns `needs_adjudication` but adjudication is disabled
- the vote returns `needs_adjudication` and the current review policy intentionally stops without a decisive accept or reject result
- the adjudicator returns `concern`

## Malformed output vocabulary

Reviewer or adjudicator output is malformed when any of the following is true:

- `reviewer_id` is missing or unknown
- `disposition` is missing or outside `approve`, `concern`, or `block`
- `summary` is empty

## Inspectability requirements

When adjudication occurs, `boundline inspect` and trace summaries MUST show:

- why adjudication was triggered
- which adjudicator ran
- the adjudication finding
- the final terminal review outcome

When review fails or escalates instead of resolving cleanly, the output MUST show which rule caused that terminal state.
