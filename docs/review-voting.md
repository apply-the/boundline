# Review Voting in Boundline 0.17.0

Boundline `0.17.0` keeps the bounded multi-agent review phase on top of the session-native
runtime. Review configuration still lives inside `<workspace>/.boundline/execution.json`
under the `review` key when the explicit compatibility manifest path or a review-configured
workspace is used. Runtime/model routing for review roles is configured
through `boundline config` in global or workspace scope and resolved by precedence.

The `0.17.0` surface-unification slice keeps review behavior bounded while
making `run`, `status`, `next`, and `inspect` report review state through the
same route and `execution_condition` summary model as the rest of the session.

Review voting is not meant to gate every step. It enters at risky boundaries,
such as PR-ready, validation-failed, or explicitly high-risk changes, where a
bounded delivery phase needs structured findings or adjudication.

Large work is supported by decomposition, not by unbounded autonomy.

## What the runtime supports

- bounded review triggers: `pr_ready`, `validation_failed`, and `high_risk_change`
- two or more configured reviewers
- structured findings with `approve`, `concern`, or `block`
- vote strategies: `majority` and `weighted`
- optional adjudication through a distinct adjudicator reviewer
- persisted review evidence in `.boundline/traces/` and projected review status in `.boundline/session.json`

## Routing and role differentiation

- `reviewers[*].source` remains the runtime-facing review source field in the
  bounded execution profile.
- `boundline config` lets operators set defaults for review and per-reviewer role
  overrides without editing the manifest directly.
- adjudication can use a dedicated route that differs from both delivery stages
  and the review council defaults.
- effective resolution follows CLI > workspace config > global config > built-in defaults.

## Manifest shape

```json
{
  "review": {
    "triggers": ["pr_ready", "validation_failed"],
    "reviewers": [
      {
        "reviewer_id": "safety",
        "role": "Safety",
        "source": "gpt",
        "weight": 2
      },
      {
        "reviewer_id": "maintainability",
        "role": "Maintainability",
        "source": "claude",
        "weight": 1
      }
    ],
    "vote_rule": {
      "strategy": "weighted",
      "reject_on_blocking": true
    },
    "adjudication": {
      "enabled": true,
      "reviewer_id": "arbiter"
    },
    "scenarios": [
      {
        "trigger": "pr_ready",
        "findings": [
          {
            "reviewer_id": "safety",
            "disposition": "approve",
            "summary": "No blockers"
          },
          {
            "reviewer_id": "maintainability",
            "disposition": "concern",
            "summary": "Small cleanup still recommended"
          }
        ],
        "adjudication_finding": {
          "reviewer_id": "arbiter",
          "disposition": "approve",
          "summary": "Acceptable for merge"
        }
      }
    ]
  }
}
```

## Vote semantics

`majority` treats every reviewer as weight `1`.

`weighted` uses the configured reviewer `weight` values.

In both strategies, Boundline resolves the council with these rules:

1. if `reject_on_blocking` is `true` and any blocking finding is present, the vote is rejected immediately
2. if approvals are strictly greater than half of the total weight, the vote is accepted
3. if blocks are strictly greater than half of the total weight, the vote is rejected
4. otherwise the vote result is `needs_adjudication`

`concern` findings affect the total vote weight, but do not directly accept or reject the council.

## Adjudication behavior

When the vote result is `needs_adjudication` and `adjudication.enabled` is true,
Boundline executes one additional reviewer step for the configured adjudicator.

- the adjudicator reviewer must be distinct from the main council reviewers
- the scenario may define one `adjudication_finding`
- the initial slice runs at most one adjudication step

If adjudication is disabled and the council cannot reach acceptance or rejection,
the final review outcome becomes `escalated`.

## What users see

When review is triggered, the local CLI surfaces review evidence in four places:

- `boundline run`: review trigger, reviewer findings, vote summary, review outcome
- `boundline status`: latest review trigger, vote summary, outcome, and reviewer headline
- `boundline next`: the same projected review state plus the next suggested command
- `boundline inspect`: ordered review timeline reconstructed from persisted trace events

The persisted trace emits dedicated review events for:

- review start
- duplicate review trigger suppression
- reviewer start and completion
- vote resolution
- adjudication
- final review outcome

## Current scope

The `0.17.0` slice is intentionally bounded:

- review is manifest-driven and deterministic
- reviewers run sequentially, not concurrently
- the runtime uses local bounded execution and local trace persistence
- voting is limited to `majority` and `weighted`
- adjudication runs at most once

For an executable example, see `specs/007-multi-agent-review/quickstart.md`.