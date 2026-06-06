# Data Model: Review Councils And Role-Gated Governance

**Feature**: 074-review-councils-governance
**Date**: 2026-06-06

## Entities

### GuardianRule

A single activation rule from `.boundline/guardian-rules.toml`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Unique rule identifier |
| `stages` | `Vec<String>` | Lifecycle stages this rule applies to (plan, run, review) |
| `files` | `Vec<String>` | Glob patterns for changed files |
| `language` | `Option<String>` | Language hint (rust, python, etc.) |
| `risk` | `Option<String>` | Minimum risk classification |
| `activate` | `Vec<String>` | Guardian IDs to activate |
| `skip` | `Vec<String>` | Guardian IDs to explicitly skip |
| `mandatory` | `Vec<String>` | Guardians that cannot be skipped |

### GuardianRuleset

The loaded and validated ruleset.

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | `String` | Ruleset schema version |
| `rules` | `Vec<GuardianRule>` | Ordered list of activation rules |
| `source` | `RulesetSource` (enum) | `BuiltIn`, `File`, or `Invalid` |

### GuardianActivationPlan

The router output after evaluating all rules against the change surface.

| Field | Type | Description |
|-------|------|-------------|
| `plan_id` | `Uuid` | Unique plan identifier |
| `ruleset_source` | `RulesetSource` | Which ruleset was used |
| `matched_rules` | `Vec<String>` | Rule IDs that matched the change surface |
| `activated` | `Vec<String>` | Guardian IDs to activate |
| `skipped` | `Vec<GuardianSkipRecord>` | Guardians skipped with reasons |
| `mandatory_unavailable` | `Vec<String>` | Mandatory guardians that could not be activated |
| `escalation` | `Option<String>` | Escalation recommendation |

### GuardianSkipRecord

| Field | Type | Description |
|-------|------|-------------|
| `guardian_id` | `String` | Skipped guardian identifier |
| `reason` | `String` | Why it was skipped (no match, unavailable, optional exclusion) |
| `is_mandatory` | `bool` | Whether this guardian was mandatory |

### GuardianExecutionRecord

Emitted after each guardian invocation.

| Field | Type | Description |
|-------|------|-------------|
| `guardian_id` | `String` | Guardian identifier |
| `status` | `ExecutionStatus` (enum) | `Success`, `Failure`, `Unavailable` |
| `finding_count` | `u64` | Total findings produced |
| `blocking_count` | `u64` | Blocking findings |
| `trace_ref` | `String` | Link to the guardian's trace output |

### CouncilDecision

The adjudicated outcome from `boundline council adjudicate`.

| Field | Type | Description |
|-------|------|-------------|
| `decision_id` | `Uuid` | Unique decision identifier |
| `adjudicator` | `String` | Role or identity of the adjudicator |
| `authority_zone` | `Option<String>` | Authority zone from the council profile |
| `profile_source` | `ProfileSource` (enum) | `Configured`, `BuiltInDefault`, `Invalid` |
| `findings_reviewed` | `u64` | Total findings reviewed |
| `accepted` | `u64` | Findings accepted as valid |
| `rejected` | `u64` | Findings rejected with rationale |
| `deferred` | `u64` | Findings deferred for later review |
| `dissent` | `bool` | Whether dissent was recorded |
| `outcome` | `CouncilOutcome` (enum) | `Clean` or `Blocked` |
| `reason` | `String` | Primary reason for the outcome |

## Entity Relationships

```
GuardianRuleset 1 ──── * GuardianRule
GuardianActivationPlan 1 ──── * GuardianSkipRecord
GuardianActivationPlan 1 ──── * GuardianExecutionRecord (via guardian_id)
CouncilDecision 1 ──── 1 GuardianActivationPlan (via plan_id)
```
