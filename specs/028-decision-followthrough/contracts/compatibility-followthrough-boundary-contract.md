# Contract: Compatibility Follow-Through Boundary

## Purpose

Define how guided follow-through must behave when the latest authoritative state
comes from an explicit compatibility trace rather than an active native session.

## Contract

- `status`, `next`, and `inspect` may reuse compatibility-trace evidence to
  explain the next bounded action when compatibility owns the latest
  authoritative follow-up.
- The projected guidance must keep compatibility continuity explicit through the
  existing authority vocabulary.
- Inspect-only follow-up must remain inspect-only; guided follow-through must
  not imply that the operator can resume a native session that does not exist.

## Required Visible Outcomes

- Operators can distinguish native-session continuity from compatibility-trace
  continuity.
- Compatibility guidance still explains why one follow-up action is credible.
- The recommended next command remains aligned with the real owning route.

## Boundary Conditions

- This contract does not make compatibility the default operator path.
- This contract does not allow route ownership to become ambiguous.
- This contract does not create resumability promises that the compatibility
  path cannot satisfy.