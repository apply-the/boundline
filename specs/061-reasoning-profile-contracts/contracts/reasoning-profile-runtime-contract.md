# Contract: Reasoning Profile Runtime

## Purpose

Define the Boundline-owned runtime contract for activating, projecting, and
terminating advanced reasoning profiles inside an existing session-native
workflow.

## Scope

- Applies to Boundline runtime state, session projections, and trace-backed
  inspection surfaces.
- Does not define Canon-owned challenge posture semantics.
- Does not define a new workflow family or replace existing governance stop
  semantics.

## Preferred Projection Shape

```json
{
  "reasoning_profile": {
    "profile_id": "independent_pair_review",
    "status": "active",
    "stage": "verification",
    "trigger": "canon_required_challenge",
    "activation_reason": "Canon posture requires blind review before acceptance",
    "canon_posture": {
      "contract_line": "governed_reasoning_posture_v1",
      "provenance_ref": "canon:packet:reasoning-posture-123"
    },
    "limits": {
      "max_participants": 2,
      "max_branches": 1,
      "max_debate_rounds": 0,
      "max_reflexion_revisions": 0,
      "max_calls": 2
    },
    "participants": [
      {
        "participant_id": "reviewer-a",
        "role": "blind_reviewer",
        "effective_route": "claude:sonnet-4.6",
        "provider_family": "anthropic",
        "status": "completed"
      },
      {
        "participant_id": "reviewer-b",
        "role": "blind_reviewer",
        "effective_route": "copilot:gpt-5.5",
        "provider_family": "openai",
        "status": "completed"
      }
    ],
    "independence": {
      "result": "passed",
      "summary": "distinct routes and provider families"
    },
    "outcome": {
      "outcome_kind": "adjudicated",
      "headline": "blind review converged after adjudication",
      "next_action": "boundline inspect --json"
    },
    "confidence": {
      "confidence_level": "medium",
      "summary": "agreement improved confidence but human approval remains required"
    }
  }
}
```

## Required Runtime Vocabulary

Supported `profile_id` values for the first release:

- `bounded_self_consistency`
- `independent_pair_review`
- `heterogeneous_security_review`
- `bounded_reflexion`

Supported `status` values:

- `pending`
- `active`
- `completed`
- `degraded`
- `blocked`
- `interrupted`
- `escalated`
- `failed`

Supported `outcome_kind` values:

- `converged`
- `disagreed`
- `adjudicated`
- `degraded`
- `blocked`
- `interrupted`
- `escalated`
- `failed`

Supported participant roles:

- `independent_path`
- `blind_reviewer`
- `heterogeneous_reviewer`
- `critic`
- `reviser`
- `arbiter`

## Required Behavior

- Boundline MUST activate a reasoning profile only from an existing stage inside
  the current session lifecycle.
- Boundline MUST record the selected `profile_id`, `stage`, `trigger`,
  `activation_reason`, effective limits, participant assignments, and Canon
  posture provenance when present.
- Boundline MUST keep reasoning-profile execution bounded by explicit limits.
- Boundline MUST expose degraded, blocked, interrupted, escalated, and failed
  outcomes explicitly rather than silently falling back.
- Boundline MUST preserve human interruption and override.
- Boundline MUST keep the outer workflow sequential-first, even when a profile
  models multiple participants.
- Boundline MUST surface the latest reasoning-profile summary through session
  and inspect projections when profile activity exists.
- Boundline MAY omit the `reasoning_profile` projection entirely when the active
  session has no reasoning-profile activity.

## Explicit Exclusions

- No background swarm execution
- No recursive profile spawning
- No Canon-owned runtime control
- No implicit acceptance authority derived from profile confidence alone