# Review Council Algorithms

Boundline `0.64.0` keeps review councils bounded and runtime-owned. Canon can
describe the authority posture of the stage, but Boundline decides how to
assemble reviewers, enforce independence, persist the council outcome, and
project the result through the session-native CLI surfaces.

This document describes the current runtime algorithm as of `0.64.0`, not an
aspirational future council system.

## 1. Intake: Structured Findings

Reviewers emit structured findings rather than untyped vote strings alone.
Each finding can preserve:

- the reviewer identity and runtime role
- severity and required action
- confidence
- evidence references
- the human-readable summary that explains the vote

That allows the runtime to distinguish a cosmetic concern from a blocking
finding that must affect stop semantics.

## 2. Independence Guard

Before the council can be treated as credible, Boundline checks whether the
required reviewer roles satisfy the expected independence posture.

Current principles:

- mandatory roles cannot collapse into one effectively non-independent voice
- the runtime records the resulting independence state explicitly
- failed independence keeps the stage from being treated as a normal approval

This guard is why the council decision is not the same thing as simple vote
tallying.

## 3. Council Assembly

The runtime resolves one council profile from the authority posture:

- `none`: no dedicated council beyond the bounded stage decision
- `light_single`: one bounded reviewer is sufficient
- `yellow_pair`: paired review is required
- `red_five`: a larger council and stronger quorum are required
- `restricted_manual`: runtime automation stops and a manual gate owns the next step

Council assembly validates the configured or discovered reviewers against that
profile. The result carries:

- the selected council profile
- quorum or mandatory-role failures when present
- the independence state
- a selection summary that can be shown to operators

## 4. Vote Resolution And Adjudication

After assembly, Boundline evaluates the findings and resolves the council vote.

The important rule is simple: unresolved blockers from mandatory roles do not
degrade into informational noise. They move the boundary into a stronger stop
posture such as `council_required`, `adjudication_required`, or `hard_stop`.

The domain already carries `ProducerResponse` and `AdjudicationOutcome` models.
The current session projection focuses first on council posture, independence,
selection summary, and stop semantics. That keeps the operator-facing runtime
truthful while the end-to-end producer-response loop continues to be wired.

## 5. Persisted Runtime Projection

When review resolution completes, Boundline persists and projects the current
council state through session-native fields including:

- `latest_review_vote_resolution`
- `latest_review_council_resolution`
- `latest_review_council_profile`
- `latest_review_independence_state`
- `latest_review_selection_summary`
- `latest_review_stop_semantics`

Those fields feed the review trace projection and the CLI `status`, `next`, and
`inspect` views.

## 6. Operator Reading Guide

Use the projected fields in this order:

1. council profile: how much review authority the stage currently needs
2. independence state: whether the council composition is credible
3. stop semantics: whether the stage can proceed, needs adjudication, or must stop
4. selection summary: which council shape or reviewer set produced the result

If the stage is blocked, these fields tell you whether the next action is to
add an independent reviewer, answer a blocking finding, obtain adjudication, or
stop automation entirely.