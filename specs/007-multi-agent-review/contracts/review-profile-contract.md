# Contract: Review Profile Extension

## Purpose

Define the workspace-local review contract that extends the existing execution profile.

## Location

- Embedded under `<workspace>/.synod/execution.json`

## Preferred JSON shape

```json
{
  "review": {
    "triggers": ["validation_failed", "pr_ready"],
    "reviewers": [
      {"reviewer_id": "safety", "role": "Safety", "source": "gpt", "weight": 2},
      {"reviewer_id": "maintainability", "role": "Maintainability", "source": "claude", "weight": 1}
    ],
    "vote_rule": {
      "strategy": "weighted",
      "reject_on_blocking": true
    },
    "adjudication": {
      "enabled": true,
      "reviewer_id": "lead"
    },
    "scenarios": [
      {
        "trigger": "validation_failed",
        "findings": [
          {"reviewer_id": "safety", "disposition": "block", "summary": "Validation still fails"},
          {"reviewer_id": "maintainability", "disposition": "concern", "summary": "Retry after correction"}
        ],
        "adjudication_finding": {
          "reviewer_id": "lead",
          "disposition": "block",
          "summary": "Reject until validation passes"
        }
      }
    ]
  }
}
```

## Required behavior

- `triggers` MUST contain at least one explicit review trigger.
- `reviewers` MUST contain at least two reviewers.
- Every reviewer `reviewer_id` MUST be unique.
- Every reviewer `weight` MUST be greater than zero.
- `vote_rule.strategy` MUST be one of `majority` or `weighted`.
- `scenarios` MUST contain at most one scenario per trigger.
- Every finding MUST reference a configured reviewer.
- `adjudication.reviewer_id` MUST be present when `adjudication.enabled = true`.
- `adjudication.reviewer_id` MUST be different from the configured council reviewers in the initial slice.
- Review configuration MAY be omitted entirely; when absent, the runtime MUST skip the review phase cleanly.
- Later duplicate triggers for the same task stage MUST be recorded and ignored rather than creating a second review phase.

## Trigger vocabulary

Supported trigger values for the initial slice:

- `validation_failed`
- `high_risk_change`
- `pr_ready`

## Decision vocabulary

Supported finding dispositions:

- `approve`
- `concern`
- `block`

Supported terminal review outcomes:

- `accepted`
- `rejected`
- `escalated`
- `failed`
