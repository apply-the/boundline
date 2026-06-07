# Adaptive Governance Calibration — Runtime Reference

**Feature**: 075-adaptive-governance-calibration
**Version**: 0.75.0

## Overview

Adaptive governance calibration allows guardian enforcement levels to graduate from advisory (visible only) through catch and rule to hook (unconditional block), based on trust metrics accumulated across adjudicated sessions.

## Files

| File | Purpose |
|------|---------|
| `.boundline/calibration-policy.toml` | Maps guardian rules to control levels by authority zone and risk level |
| `.boundline/overrides.json` | Operator override records for bypassing catch/rule blocks |
| `.boundline/trust-records.json` | Accumulated guardian trust metrics (TP, FP, TPR, FPR) |

## Control Levels

- **Advisory**: Finding is visible but does not block execution.
- **Catch**: Finding needs attention; operator can bypass with an override.
- **Rule**: Block unless a satisfying override is provided.
- **Hook**: Unconditional block; only privileged process can bypass.

## Commands

```bash
# Adjudicate with calibration
boundline council adjudicate [--json]

# Write an override record
boundline override --guardian-id <ID> --control-id <ID> --level <LEVEL> --reason <TEXT> [--expiry <ISO8601>]
```

## Trust Evolution

1. After every council adjudication, trust counters update (TP/FP).
2. After the configured evidence window (default 5 sessions), the system evaluates promotion/demotion.
3. A guardian with TPR >= 90% and zero FPs promotes one level.
4. A guardian with FPR > 20% demotes or stays advisory.
5. Guardians correlated with incidents are locked at advisory/catch.
6. Evals below confidence threshold block promotion.

## Structured Events

- `control.level.assigned` — emitted when a control level is assigned to a guardian.
- `control.level.graduated` — emitted on promotion or demotion.
- `control.degraded` — emitted when a control is downgraded due to unavailability.
- `control.escalated` — emitted when a finding is escalated.
