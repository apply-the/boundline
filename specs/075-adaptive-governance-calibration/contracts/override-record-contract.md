# Contract: Override Record Format

**Feature**: 075-adaptive-governance-calibration
**Version**: 1.0

## Purpose

Define the format of override records written by `boundline override` and consumed by `boundline run`/`boundline continue`. Override records allow operators to bypass catch-level and rule-level blocks with explicit justification and traceability.

## Record Format (TOML)

```toml
finding_id = "f-20260606-001"
control_id = "ctrl-rust-guardian-01"
guardian_id = "rust-guardian"
requested_level = "catch"
reason = "False positive: the matched file is a doc comment, not runtime code"
operator_identity = "operator@example.com"
timestamp = "2026-06-06T12:00:00Z"
expiry = "2026-06-07T12:00:00Z"
satisfies_policy = true
```

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `finding_id` | string | yes | Unique identifier of the blocked finding |
| `control_id` | string | yes | Identifier of the control (guardian activation) being overridden |
| `guardian_id` | string | yes | Guardian that produced the finding |
| `requested_level` | enum(advisory/catch/rule) | yes | Level the operator is requesting (cannot request hook bypass) |
| `reason` | string | yes | Human-readable justification |
| `operator_identity` | string | no | Who performed the override (when available) |
| `timestamp` | string | yes | ISO 8601 when the override was written |
| `expiry` | string | no | ISO 8601 expiry if time-limited by policy |
| `satisfies_policy` | boolean | yes | Whether the override meets the configured override policy |

## CLI Contract

```
boundline override --workspace <PATH> --guardian-id <ID> --control-id <ID> --level <LEVEL> --reason <TEXT> [--expiry <ISO8601>]
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Override record written successfully |
| 1 | Invalid arguments (missing required fields, invalid level) |
| 2 | Override policy not satisfied (e.g., operator role not authorized) |
| 3 | Finding/control not found or already resolved |

## Consumption

`boundline run` and `boundline continue` MUST:

1. Read all unexpired override records from `.boundline/overrides.toml` before adjudication.
2. Match override records to blocked findings by `finding_id` or `control_id`.
3. Accept the override if `satisfies_policy = true` and the override is not expired.
4. Record the override consumption in the council decision trace.
5. Remove or mark consumed override records after adjudication.

## Trace Visibility

Override records are surfaced in `boundline inspect` output alongside the council decision. The trace event `control_level.overridden` is emitted when an override is consumed.

## Edge Cases

- **Expired override**: Treated as if absent. Block remains.
- **Override for hook-level finding**: Rejected. Hook bypass requires privileged process.
- **Duplicate override**: Latest override for the same finding/control wins.
- **Override while degraded**: Override evaluated first; if it fails, degradation rules apply.
