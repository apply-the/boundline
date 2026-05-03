# Contract: Bounded Replan

## Purpose

Define how Synod revises a previously proposed or confirmed plan when evidence
changes.

## Preconditions

- A current goal-plan proposal exists.
- New evidence exists or the operator explicitly requests replanning.

## `synod plan --replan`

### Required behavior

- Build a fresh planning evidence bundle using the latest session and workspace
  state.
- Compare the new inference against the current authoritative proposal.
- If flow, targets, tasks, or verification strategy change materially, create a
  new proposal revision and mark the prior authoritative revision as superseded.
- Persist a replan summary describing what changed and why.
- Leave the new proposal unconfirmed until the operator explicitly confirms it.

### No-op behavior

- If the refreshed evidence does not materially change the proposal, keep the
  existing revision authoritative and report that no bounded replan was needed.

### Errors

- If no proposal exists, return a bounded operator error explaining that an
  initial plan must be generated first.
- If replanning cannot proceed because context is still insufficient, persist the
  updated insufficient-context summary and return clarification-required output.

## Inspectability requirements

- `status` and `inspect` must show:
  - the active revision,
  - the superseded revision when one exists,
  - the fields that changed, and
  - the evidence summary that triggered the replan.