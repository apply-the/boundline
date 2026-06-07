# Recursive Stage Refinement Profiles

**Feature**: 076-recursive-stage-refinement
**Version**: 0.76.0

## Overview

Recursive stage refinement adds bounded, inspectable refinement loops to the
`boundline plan` command. When enabled, the plan stage runs through a
`planner → critic → planner → finalizer` loop that iteratively improves the
plan. Each round produces a compact trace-linked packet. The loop stops when
the plan stops improving, the round/time budget is exhausted, or a blocker
is found.

## Quick Start

Create `.boundline/refinement-profiles.toml` in your workspace root:

```toml
[profiles.plan_refinement]
enabled = true
max_rounds = 3
max_elapsed_time_seconds = 300

[profiles.plan_refinement.roles]
planner_provider_id = "openai-gpt-5"
critic_provider_id = "openai-gpt-5"
finalizer_provider_id = "openai-gpt-5"
```

Then run `boundline plan` as usual. The refinement loop activates automatically.

## CLI Flags

| Flag | Effect |
|------|--------|
| `--refine` | Enable refinement for this run (overrides config) |
| `--no-refine` | Disable refinement for this run (overrides config) |
| `--max-rounds N` | Override the round limit for this run |

## Inspecting Refinement

- `boundline status` — shows active refinement profile and current round
- `boundline next` — suggests next action based on refinement outcome
- `boundline inspect` — shows full refinement history with round packets

## Stop Reasons

| Reason | Meaning |
|--------|---------|
| `no_material_delta` | Plan stopped improving — converged |
| `round_limit_exhausted` | Hit the round budget |
| `time_limit_exhausted` | Timed out |
| `unresolved_blocker` | Blocking finding couldn't be resolved |
| `empty_candidate` | Provider returned nothing |
| `provider_failure` | Provider failed mid-round |
| `malformed_packet` | Round packet was invalid |
| `invalid_delta` | Delta referenced non-existent artifact |
| `invalid_configuration` | Config validation failed |

## Outcome Labels

- **`finalized`**: Plan is ready for the next stage (`boundline run`)
- **`incomplete`**: Plan needs attention — review findings and re-run

## Configuration

Profiles are configured in `.boundline/refinement-profiles.toml`:

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Whether refinement is active |
| `max_rounds` | `3` | Hard round limit (must be ≥ 1) |
| `max_elapsed_time_seconds` | `300` | Hard time limit in seconds (must be > 0) |
| `roles.planner_provider_id` | — | Provider for the planner role |
| `roles.critic_provider_id` | — | Provider for the critic role |
| `roles.finalizer_provider_id` | — | Provider for the finalizer role |

## Round Packet Schema

Each refinement round produces a compact structured packet:

```json
{
  "schema_version": "1.0",
  "profile": "plan_refinement",
  "stage": "plan",
  "round": 2,
  "candidate_ref": "trace://plan-candidate-2",
  "findings": [],
  "requested_deltas": [],
  "applied_deltas": [],
  "critic_confidence": "sufficient",
  "effective_confidence": "sufficient",
  "confidence_adjustment_reason": null,
  "stop_reason": null
}
```

Packets reference artifacts by trace identifier (`trace://plan-candidate-N`)
rather than copying full content inline.

## Constraints

- **One stage only**: Refinement currently works for `plan` only
- **One profile only**: Only `plan_refinement` is supported
- **No hidden behavior**: Every round is trace-visible
- **No new dependencies**: Uses existing providers, registry, and trace store
- **No sqlite-vec**: Feature is fully functional without the vector extension
