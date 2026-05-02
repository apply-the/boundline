# Contract: Continuity Evidence Precedence

## Purpose

Define how Synod chooses between persisted session evidence and the latest
authoritative trace when projecting guided follow-through.

## Contract

- One evidence source must win whenever both persisted session state and trace
  evidence could explain the next bounded action.
- When the native session remains authoritative, persisted session continuity
  may drive follow-through guidance.
- When explicit compatibility follow-up owns the latest authoritative state, the
  latest authoritative trace must drive follow-through guidance instead.
- The winning evidence source must remain visible whenever it materially affects
  the recommended next action or stop condition.

## Required Visible Outcomes

- Operators can tell whether the guidance came from current session state or
  the authoritative trace.
- Operators do not need to infer precedence from missing fields or from raw JSON
  artifacts.
- Session reloads and inspect-only follow-up preserve a coherent continuity
  story instead of mixing stale and fresh evidence silently.

## Boundary Conditions

- This contract does not permit silent merging of contradictory evidence.
- This contract does not create a new persistence surface outside the existing
  session and trace model.
- This contract does not allow compatibility evidence to masquerade as native
  session authority.