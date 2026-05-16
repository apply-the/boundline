# Adaptive Governance Projection Contract

## Purpose

Define the minimum session, trace, and CLI projection that Boundline must
surface after resolving adaptive governance for one boundary.

## Required Projection Fields

Boundline must preserve these runtime-owned fields through `plan`, `run`,
`status`, `next`, and `inspect` when adaptive governance is active:

- `latest_governance_runtime_state`
- `latest_governance_rollout_profile`
- `latest_governance_confidence_level`
- `latest_governance_trust_state`
- `latest_governance_degradation_mode`
- `latest_governance_escalation_target`
- `latest_governance_override_summary`
- `latest_governance_contract_lines`
- `latest_governance_reason`
- `latest_governance_next_action`

## Projection Rules

- Projection must show the required Canon baseline contract line separately from any optional adaptive companion contract line.
- Projection must preserve `adaptive_contract_line: unavailable` when the required baseline remains compatible but no compatible companion metadata is available.
- Projection must explain whether the current posture is advisory, catch, rule, or hook.
- Projection must show whether the boundary continued normally, degraded, escalated, waited, or stopped.
- Projection must preserve why confidence or trust changed rather than exposing only the resulting state label.
- Projection must remain readable from compact surfaces such as `status` and `next`, while `inspect` preserves the fuller rationale and supporting evidence references.

## Degradation And Escalation Rules

- Any degradation outcome must show both the degradation mode and the mapped S3 stop semantics.
- Any escalation outcome must show the trigger and the next required authority target.
- If governance is blocked because a required Canon contract is unavailable, the projection must show that compatibility failure explicitly instead of collapsing it into a generic error.

## Explicit Exclusions

Projection does not require the runtime to expose hidden heuristics or opaque
scores without explanation. If a confidence or trust field appears, its visible
rationale must also appear through the same session or trace story.