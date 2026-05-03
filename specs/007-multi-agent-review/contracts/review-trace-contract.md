# Contract: Review Trace and Inspection Surface

## Purpose

Define the minimum trace and inspection surface for the bounded review phase.

## Required trace events

Each review phase MUST persist trace evidence for:

- `review_started`
- `review_trigger_ignored` when a duplicate trigger is recorded for the same task stage
- `reviewer_started` for each participating reviewer
- `reviewer_completed` for each completed reviewer
- `review_vote_resolved`
- `review_adjudicated` when adjudication occurs
- `review_terminal_recorded`

The trace MAY continue to include the existing delivery-stage events before or after review.

## Reviewer payload requirements

The completed payload for each reviewer MUST make the following information inspectable:

- reviewer identifier
- reviewer role
- finding disposition
- summary
- optional detail text when present
- participation status

Failed reviewer payloads MUST expose:

- reviewer identifier
- failure reason
- whether the failure caused rejection, escalation, or immediate review failure

## Vote payload requirements

The vote-resolution payload MUST expose:

- applied strategy
- participating reviewers
- participation status for configured reviewers that failed or were omitted
- counts or weights for approvals, concerns, and blocks
- preliminary decision
- whether adjudication is required

## Inspect output requirements

`boundline inspect` MUST make the following information visible after a review phase:

- the review trigger
- the participating reviewers
- the reviewer findings in execution order
- the vote method and resulting tally summary
- whether adjudication occurred
- the final review outcome

## Session status projection

When a session-backed delivery task has review evidence, `boundline status` SHOULD surface:

- the latest review trigger
- the latest review vote summary
- the latest review outcome
- the latest reviewer headline or participant summary

When no review evidence exists yet, those fields MUST be omitted cleanly.
