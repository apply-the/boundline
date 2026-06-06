# Council Output Contract

**Feature**: 074-review-councils-governance
**Date**: 2026-06-06

## Purpose

Define the output format for `boundline council adjudicate` in human-readable (default) and `--json` modes.

## Human-Readable Output

```
Council Adjudication
  Adjudicator: single-reviewer (built-in default)
  Authority zone: yellow
  Ruleset: .boundline/guardian-rules.toml (v1.0)
  Matched rules: rust-runtime-change, security-sensitive-change
  Guardians activated: rust-guardian, error-handling-guardian, traceability-guardian, security-guardian
  Guardians skipped: docs-consistency-guardian (no docs change)
  Mandatory unavailable: none

Findings reviewed: 5
  Accepted: 3
  Rejected: 1 (duplicate finding)
  Deferred: 1 (requires manual review)
  Dissent: none

Outcome: BLOCKED
Reason: security-guardian found 2 blocking findings (unauthorized file access)
```

## JSON Output (`--json`)

```json
{
  "decision_id": "uuid",
  "adjudicator": "single-reviewer",
  "authority_zone": "yellow",
  "profile_source": "built_in_default",
  "activation_plan": {
    "ruleset_source": "file",
    "matched_rules": ["rust-runtime-change"],
    "activated": ["rust-guardian", "error-handling-guardian"],
    "skipped": [{"guardian_id": "docs-consistency-guardian", "reason": "no docs change", "is_mandatory": false}],
    "mandatory_unavailable": []
  },
  "findings_reviewed": 5,
  "accepted": 3,
  "rejected": 1,
  "deferred": 1,
  "dissent": false,
  "outcome": "blocked",
  "reason": "security-guardian found 2 blocking findings"
}
```

## Structured Events

### `guardian.activation.plan.produced` (v1.0)

```json
{
  "event_type": "guardian.activation.plan.produced",
  "schema_version": "1.0",
  "payload": {
    "plan_id": "uuid",
    "ruleset_source": "file",
    "matched_rules": ["rust-runtime-change"],
    "activated": ["rust-guardian"],
    "skipped_count": 1,
    "mandatory_unavailable": []
  }
}
```

### `council.decision.produced` (v1.0)

```json
{
  "event_type": "council.decision.produced",
  "schema_version": "1.0",
  "payload": {
    "decision_id": "uuid",
    "plan_id": "uuid",
    "adjudicator": "single-reviewer",
    "outcome": "blocked",
    "blocking_finding_count": 2,
    "dissent": false
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Adjudication complete (clean or blocked) |
| 1 | Internal error (invalid ruleset, missing evidence) |
