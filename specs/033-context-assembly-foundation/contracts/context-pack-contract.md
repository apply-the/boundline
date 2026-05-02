# Contract: Context Pack Creation

## Goal

Planning on the primary session-native path produces one bounded context pack before a goal plan is confirmed.

## Required Outcomes

- The goal plan includes exactly one context-pack object.
- The pack has:
  - a stable identifier
  - a summary
  - an explicit credibility state
  - ordered context inputs
  - at least one primary input when credibility is `credible`
- The pack records inputs drawn from available bounded sources:
  - workspace-derived file targets
  - authored brief evidence when present
  - negotiated delivery evidence when present
  - recent trace evidence when present
  - reusable Canon artifacts when present

## Failure Contract

- If no credible bounded inputs exist, planning must not confirm the goal plan.
- The surfaced failure must identify that context credibility, not a generic runtime crash, blocked planning.
