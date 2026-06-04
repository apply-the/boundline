# Contract: Planning Analysis Runtime Projection

## Purpose

Define the additive Boundline-owned planning-coherence contract used by CLI
status, inspect, orchestration snapshots, execution admission, traces, and
supported assistant assets.

## Evaluation Order

```text
goal quality
  -> plan quality
  -> backlog quality
  -> planning analysis
  -> execution handoff
```

Planning analysis must only run after the earlier planning gates are ready.
Each earlier gate remains authoritative for its own defect class.

## Evaluation Inputs

The initial slice may use only these runtime-owned or governed inputs:

- the active `GoalPlan`
- typed plan-quality state already derived from that plan
- the active ready backlog-quality snapshot
- the governed Canon backlog packet documents already available in the
  lifecycle snapshot, including `execution-handoff.md` when present
- typed validation strategy, expected outcomes, risks, constraints, and
  workspace signals already persisted in Boundline

The gate is strictly read-only. It must not mutate the plan, Canon packet,
workspace files, or governed evidence.

## Additive Session Projection

When planning analysis has run, session status and orchestration snapshots may
include:

```json
{
  "planning_analysis_state": "blocked",
  "planning_analysis_findings": [
    {
      "severity": "critical",
      "source": "governed_evidence",
      "code": "producer_contract_gap",
      "message": "execution handoff requires Canon-authored slice verification anchors",
      "source_refs": [
        {
          "artifact_kind": "backlog_document",
          "artifact_ref": "execution-handoff.md",
          "anchor": "slice_id=payments-mvp"
        }
      ]
    }
  ],
  "planning_analysis_coverage": {
    "success_criteria_total": 3,
    "success_criteria_covered": 2,
    "backlog_slice_total": 2,
    "backlog_slice_covered": 1,
    "validation_anchor_total": 3,
    "validation_anchor_covered": 2,
    "risk_total": 1,
    "risk_covered": 1,
    "constraint_total": 1,
    "constraint_covered": 1,
    "governed_evidence_ready": false
  }
}
```

Allowed `planning_analysis_state` values:

| Value | Meaning | Execution handoff |
|---|---|---|
| `clean` | No coherence defect remains. | May continue. |
| `findings` | Non-blocking coherence findings remain visible. | May continue. |
| `blocked` | One or more critical coherence defects remain. | Withheld. |

The fields are additive. Older snapshots may omit them entirely.

## Blocking Rules

Planning analysis must transition to `blocked` when any critical defect is
present, including:

- an uncovered required success criterion
- missing validation coverage for a required outcome
- an execution-critical contradiction between typed planning artifacts
- a missing required execution input
- a producer contract gap where Canon-owned evidence is absent

Non-critical findings may remain visible as `findings`, but they must not
silently upgrade to `clean`.

## Deduplication Rules

The runtime must report one finding per materially distinct defect. Equivalent
defects discovered through multiple artifacts may contribute multiple
`source_refs`, but they must not appear as duplicate top-level blockers unless
they require different operator action.

## Compatibility And Omission Rules

- If planning analysis has never run for a compatible older session snapshot,
  the projection fields must be omitted rather than synthesized.
- If backlog quality is not ready, planning analysis must not invent a default
  blocked state; the earlier gate remains authoritative.
- If Canon is optional or absent for the active route, the analysis must still
  run using Boundline-owned artifacts only.

## Assistant Asset Contract

Copilot, Claude, Codex, and Antigravity plan, run, status, and inspect assets
must preserve:

- `goal_quality_state`
- `plan_quality_state`
- `backlog_quality_state`
- `planning_analysis_state`
- `planning_analysis_findings`
- `planning_analysis_coverage`
- raw continuation commands
- assistant-safe continuation commands

Assistant hosts must not synthesize a direct execution route when
`planning_analysis_state` is `blocked`. They must route back to plan-stage
repair or the emitted continuation instead.

## Explicit Non-Goals

- No standalone `/boundline-analyze` CLI command.
- No LLM-backed semantic contradiction engine.
- No Canon packet mutation, schema change, or file rewrite.
- No automatic repair, replanning authoring, or task generation.
- No provider, browser, memory, council, concurrency, or background-worker
  behavior.
