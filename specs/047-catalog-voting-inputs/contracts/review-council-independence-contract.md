# Contract: Review Council Independence

## Scope

The bounded review lifecycle in the fixture/runtime path that records reviewer findings, resolves a vote, and finalizes review outcomes.

## Inputs

- Configured review council in `ReviewProfile.reviewers`
- Completed reviewer findings in `latest_review_findings`
- Effective routing evidence from task-state `routing_projection`, with fallback to explicit reviewer source when needed

## Required Behavior

- Every completed reviewer counted in a vote must resolve to one effective review route.
- If two or more completed reviewers resolve to the same effective route, vote resolution must stop before producing a normal multi-review decision.
- When independence fails, the runtime must emit the explicit terminal code `non_independent_review_council`.
- Successful vote resolution must persist the effective route for each completed participant.

## Persisted Evidence

- `latest_review_participants[*].effective_route`
- `latest_review_vote_resolution.participants[*].effective_route`
- `latest_review_vote_decision` only when the council is independent and complete