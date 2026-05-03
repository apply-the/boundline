# Contract: Dynamic Plan Proposal

## Purpose

Define the observable contract for creating and confirming an evidence-driven
goal-plan proposal on the native session path.

## Preconditions

- A session exists with a captured goal.
- No blocking negotiation or authored-brief clarification remains unresolved.
- The workspace can be inspected for bounded context.

## `synod plan`

### Input

- Optional workspace or cluster selector.
- Optional flow override or `--no-flow`.
- No confirmation flag.

### Required behavior

- Build a planning evidence bundle from current session context and workspace
  evidence.
- Infer flow, targets, and verification strategy from that bundle.
- Persist a goal-plan proposal with revision `1` for a new plan or the next
  revision for a replanned plan.
- Mark the proposal as unconfirmed unless the operator explicitly requested
  otherwise.
- Surface a proposal summary that includes flow mode, targets, verification
  strategy, and evidence rationale.

### Blocking behavior

- If context credibility is insufficient, return a clarification-required result,
  persist the insufficient proposal state, and explain the missing evidence.

## `synod plan --confirm`

### Preconditions

- A current proposal exists and is not superseded.

### Required behavior

- Confirm the current proposal without mutating its inferred rationale.
- Persist the confirmation state and timestamp.
- Make native goal-plan execution eligible.

### Errors

- If no proposal exists, return a bounded operator error.
- If the current proposal is superseded or invalid, return a bounded operator
  error explaining why confirmation is not possible.

## `synod run`

### Required behavior

- Refuse native execution while the current proposal is unconfirmed.
- Explain whether the operator should confirm, replan, or capture more context.
- Use the confirmed proposal's flow, targets, and verification strategy once the
  proposal is confirmed.