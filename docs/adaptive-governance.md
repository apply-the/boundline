# Adaptive Governance In Boundline 0.63.0

Boundline `0.63.0` can consume Canon `adaptive-governance-v1` as an additive
companion to the required `authority-governance-v1` baseline.

The companion is semantic input, not runtime control.

Boundline may use the companion to explain maturity and preserve Canon context,
but it still owns confidence, trust, degradation, escalation, councils, and
stop transitions.

## Consumed Companion Shape

When present, Boundline expects these required companion fields:

- `contract_line`
- `governance_state`
- `rollout_profile`

Optional rationale fields:

- `state_rationale`
- `profile_rationale`

Supported companion vocabulary for this slice:

- governance states: `advisory`, `catch`, `rule`, `hook`
- rollout profiles: `minimal`, `guided`, `governed`, `strict`

## Compatibility Rules

Boundline treats the companion as unavailable when:

- the object is absent
- the contract line is unsupported
- required companion fields are missing
- companion values are outside the supported vocabulary

If the current Canon stage does not require the companion, Boundline continues
with the required authority baseline and keeps companion unavailability visible.

If the current Canon stage sets `require_adaptive_companion: true`, Boundline
fails closed with `adaptive_contract_unavailable` instead of inventing a local
equivalent.

## Projection And Visibility

Companion semantics flow into compacted Canon memory and from there into
session and trace surfaces.

Typical projected lines:

- `adaptive_contract_line: adaptive-governance-v1`
- `adaptive_governance_state: advisory|catch|rule|hook`
- `adaptive_rollout_profile: minimal|guided|governed|strict`
- `adaptive_state_rationale: <text|unavailable>`
- `adaptive_profile_rationale: <text|unavailable>`

When the companion is absent on an otherwise compatible baseline, Boundline
projects:

- `adaptive_contract_line: unavailable`

That line is intentional. It keeps the absence of the companion distinct from a
compatible companion and from a blocked required-companion failure.

## What Operators Should Expect

On normal governed runs:

- `run` and `inspect` preserve separate authority and adaptive provenance lines
- `status` and `next` can continue to summarize the same compacted Canon memory
- project-memory, approval, readiness, lineage, and promotion data remain Canon
  provenance rather than local runtime authority

On blocked required-companion runs:

- the stage is blocked explicitly
- the reason code is `adaptive_contract_unavailable`
- the message explains whether the companion was missing, unavailable, or
  unsupported for that stage

## Boundary Summary

Canon owns:

- semantic posture contracts
- approval and readiness metadata
- packet lineage and promotion metadata
- companion maturity labels and rationales

Boundline owns:

- whether a stage may continue
- what governance state actually applies at runtime
- confidence and trust interpretation
- degradation and escalation outcomes
- council and stop behavior

This split is the core adaptive-governance contract boundary: Canon describes the governed
packet; Boundline decides what the runtime does next.