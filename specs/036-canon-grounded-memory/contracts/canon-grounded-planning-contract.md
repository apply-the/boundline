# Contract: Canon-Grounded Planning

## Purpose

Define how the primary session-native planning path consumes Canon-grounded
evidence as live bounded reasoning input.

## Preconditions

- A session exists with a captured goal.
- Negotiation and authored-brief clarifications are resolved enough for bounded
  planning.
- Canon-governed evidence may or may not exist; absence must remain explicit.

## `boundline plan`

### Required behavior

- Build the normal bounded planning context from workspace, session, and recent
  trace evidence.
- When relevant Canon packets, governed artifacts, artifact summaries, or
  capability signals exist, normalize them into a Canon context snapshot.
- Use that Canon context snapshot to shape proposal rationale, target selection,
  verification strategy, or bounded stop reasoning when it materially changes
  the result.
- Persist the Canon-grounded influence in the authoritative goal-plan and
  session projections.

### Fallback behavior

- If no relevant Canon-grounded evidence exists, planning must remain explicit
  about that absence and continue only with bounded non-Canon evidence.
- If Canon-grounded evidence exists but would widen scope beyond the accepted
  boundary, planning must exclude that scope or stop explicitly.

### Errors

- If Canon-grounded evidence is required for credibility but the current Canon
  context is missing, stale, or contradictory, planning must return a bounded
  clarification, refresh, or stop result rather than silently continuing.

## Observability requirements

- `plan`, `status`, `next`, and `inspect` must surface:
  - the decisive Canon context headline when one exists,
  - packet or artifact lineage when it materially influenced the proposal,
  - the compact-memory credibility state, and
  - the explicit next action when Canon grounding blocks continuation.