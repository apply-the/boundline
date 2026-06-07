# Contract: Calibration Policy File Format

**Feature**: 075-adaptive-governance-calibration
**Version**: 1.0

## Purpose

Define the TOML schema for `.boundline/calibration-policy.toml`, the versioned configuration that maps guardian rules to graduated control levels (advisory, catch, rule, hook) by authority zone and risk level.

## Schema

```toml
schema_version = "1.0"
evidence_window = 5
minimum_evidence_threshold = 3

[[entries]]
rule_id = "rust-runtime-change"
authority_zone = "green"
risk_level = "low"
default_level = "advisory"
green_level = "catch"
yellow_level = "rule"
red_level = "rule"
confidence_threshold = 0.85

[entries.override_policy]
allowed_roles = ["operator", "maintainer"]
required_evidence = ["override_reason", "test_results"]
time_limited = true
max_duration_hours = 24

[[entries]]
rule_id = "security-sensitive-change"
authority_zone = "red"
risk_level = "high"
default_level = "hook"
green_level = "rule"
yellow_level = "hook"
red_level = "hook"
confidence_threshold = 0.95

[entries.override_policy]
allowed_roles = ["security-lead"]
required_evidence = ["security_review", "threat_model_update"]
time_limited = false
```

## Required Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | string | yes | Policy schema version |
| `evidence_window` | integer | yes | Minimum adjudicated sessions before trust evaluation |
| `minimum_evidence_threshold` | integer | yes | Minimum sample size for TPR/FPR computation |
| `entries` | array | yes | Per-rule calibration entries |

### Entry Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rule_id` | string | yes | Matches a guardian rule id in `guardian-rules.toml` |
| `authority_zone` | enum(green/yellow/red) | yes | Authority zone this entry applies to |
| `risk_level` | enum(low/medium/high) | yes | Risk level this entry applies to |
| `default_level` | enum(advisory/catch/rule/hook) | yes | Level when no trust data exists |
| `green_level` | enum(advisory/catch/rule/hook) | yes | Level in green zone with trust |
| `yellow_level` | enum(advisory/catch/rule/hook) | yes | Level in yellow zone |
| `red_level` | enum(advisory/catch/rule/hook) | yes | Level in red zone |
| `confidence_threshold` | float (0.0–1.0) | yes | Minimum calibrated confidence for promotion |
| `override_policy` | table | yes | Override authorization rules |

### Override Policy Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allowed_roles` | array of strings | yes | Roles authorized to override |
| `required_evidence` | array of strings | yes | Evidence types required with override |
| `time_limited` | boolean | yes | Whether override expires |
| `max_duration_hours` | integer | if time_limited | Expiry duration in hours |

## Validation Rules

1. **Fail closed**: Invalid TOML, missing required fields, or contradictory entries → policy rejected, all guardians default to advisory.
2. **Red-zone guard**: Red zone entries with `default_level = advisory` → validation error.
3. **Level consistency**: `advisory` cannot appear as `red_level`. `hook` can only be superseded by `hook`.
4. **Confidence range**: `confidence_threshold` must be 0.0–1.0 inclusive.
5. **Evidence window**: `evidence_window` must be ≥ 1. `minimum_evidence_threshold` must be ≤ `evidence_window`.

## Compatibility

- Versioned: `schema_version` field enables future schema evolution.
- Backward-compatible additions (new optional fields) are allowed without version bump.
- Breaking changes (removed fields, changed semantics) require a `schema_version` increment and migration guidance.
