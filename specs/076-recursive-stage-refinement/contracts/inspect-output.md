# Inspect Output Contract: Refinement State

**Feature**: 076-recursive-stage-refinement
**Version**: 1.0
**Date**: 2026-06-07

## Overview

`boundline inspect`, `boundline status`, and `boundline next` must surface refinement state including the active profile, current round, findings, stop reason, and final outcome. This contract defines the expected output shapes.

## `boundline inspect` — Refinement History

When a refinement loop has executed (or is executing), `boundline inspect` must include a refinement history section.

### Human-Readable Output (default)

```
Refinement Profile: plan_refinement
Stage: plan
Rounds: 2 (stopped)
Stop Reason: no_material_delta
Outcome: finalized

Round 1:
  Candidate: trace://plan-candidate-1
  Critic Confidence: low → effective: low
  Findings: 3 (f-001, f-002, f-003)
  Requested Deltas: 2
  Applied Deltas: 2

Round 2:
  Candidate: trace://plan-candidate-2
  Critic Confidence: sufficient → effective: sufficient
  Findings: 0
  Requested Deltas: 0
  Applied Deltas: 0
  Stop Reason: no_material_delta
```

### JSON Output (`--json`)

```json
{
  "refinement": {
    "profile": "plan_refinement",
    "stage": "plan",
    "status": "stopped",
    "outcome": "finalized",
    "rounds": [
      {
        "round": 1,
        "candidate_ref": "trace://plan-candidate-1",
        "critic_confidence": "low",
        "effective_confidence": "low",
        "confidence_adjustment_reason": null,
        "findings": ["f-001", "f-002", "f-003"],
        "requested_deltas": 2,
        "applied_deltas": 2,
        "stop_reason": null
      },
      {
        "round": 2,
        "candidate_ref": "trace://plan-candidate-2",
        "critic_confidence": "sufficient",
        "effective_confidence": "sufficient",
        "confidence_adjustment_reason": null,
        "findings": [],
        "requested_deltas": 0,
        "applied_deltas": 0,
        "stop_reason": "no_material_delta"
      }
    ]
  }
}
```

## `boundline status` — Active Refinement

When a refinement loop is mid-execution, `boundline status` must surface:

```
Active Refinement:
  Profile: plan_refinement
  Stage: plan
  Current Round: 2 of 3
  Status: running
  Next Action: continue refinement
```

When stopped:

```
Active Refinement:
  Profile: plan_refinement
  Stage: plan
  Rounds Completed: 2
  Status: stopped (no_material_delta)
  Outcome: finalized
```

## `boundline next` — Refinement Recommendation

When a refinement loop stopped with unresolved findings:

```
$ boundline next
Next: Resolve blocking findings before re-running plan stage
Unresolved: f-005 (severity: high, "Missing validation strategy for step 3")
```

When stopped with a finalized outcome:

```
$ boundline next
Next: run (plan complete, refinement converged at round 2)
```

## Contract Tests

1. **Inspect after refinement**: Running `boundline inspect` after a completed refinement loop shows profile, rounds, stop reason, and outcome.
2. **Status mid-refinement**: Running `boundline status` during an active refinement loop shows current round and next action.
3. **Next after blocked refinement**: Running `boundline next` after a refinement loop stopped with unresolved blockers recommends resolving findings.
4. **Inspect without refinement**: Running `boundline inspect` on a session where refinement was not enabled shows no refinement section.
5. **JSON output valid**: `boundline inspect --json` produces valid JSON with all refinement fields.
6. **No inline content in inspect**: The inspect output must not include full plan text; only trace references.
7. **Confidence adjustment visible**: When `critic_confidence` and `effective_confidence` differ, the inspect output must show the adjustment reason.
