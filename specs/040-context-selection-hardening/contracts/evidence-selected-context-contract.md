# Contract: Evidence-Selected Context

## Purpose

Define the observable contract for building a credible planning context from
explicit bounded evidence on the primary session-native path.

## Preconditions

- A session exists with a recorded goal.
- Any authored brief metadata already recorded in session state is available.
- The workspace or cluster scope is known.

## `boundline plan`

### Required behavior

- Collect candidate context inputs from explicit bounded evidence before using
  any tie-break heuristics.
- Prefer evidence anchors such as failing tests, validation paths, authored
  brief references, workflow targets, recent mutations, recent traces, and
  reusable Canon artifacts when they are available.
- Persist a `ContextPack` whose primary inputs explain why the selected files or
  artifacts matter to the current bounded goal.
- Surface a context summary, credibility state, and provenance lines that name
  the admitted inputs.

### Non-credible behavior

- If no bounded evidence supports a credible selection, do not claim a credible
  context pack.
- Persist the insufficient or stale context state and return an operator-facing
  explanation of what evidence is missing or contradicted.