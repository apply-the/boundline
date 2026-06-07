# Quickstart: Adaptive Governance Calibration

**Feature**: 075-adaptive-governance-calibration
**Date**: 2026-06-06

## Prerequisites

- Boundline 0.74.0+ (with `boundline council adjudicate` available)
- A workspace initialized with `boundline init`
- Existing `.boundline/guardian-rules.toml` (or built-in defaults)

## 1. Create a Calibration Policy

Create `.boundline/calibration-policy.toml` in your workspace:

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
allowed_roles = ["operator"]
required_evidence = ["reason"]
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

## 2. Run a Governed Session

```bash
boundline run --goal "Add input validation" --workspace .
```

The council adjudicator now applies the calibration policy:
- Findings from `rust-runtime-change` in green/low: **advisory** (visible, no block)
- Findings from `security-sensitive-change` in red/high: **hook** (unconditional block)

## 3. Inspect Control-Level Decisions

```bash
boundline inspect --workspace .
```

Output includes for each guardian:
- Current control level and why
- Guardian-provided confidence score
- Effective calibrated confidence (after trust adjustment)
- Whether an override policy is available
- Any degradation state
- Terminal outcome

## 4. Bypass a Catch or Rule Block

If a `catch` or `rule` finding blocks your run:

```bash
boundline override \
  --workspace . \
  --guardian-id rust-guardian \
  --control-id ctrl-rust-guardian-01 \
  --level catch \
  --reason "False positive: matched file is documentation only"
```

Then re-run:

```bash
boundline run --goal "Add input validation" --workspace .
```

The override is consumed, and `boundline inspect` shows the override in the trace.

## 5. Watch Trust Evolve

After 5 adjudicated sessions (the default evidence window):

```bash
boundline inspect --workspace .
```

Check the guardian trust metrics:
- True positive count (findings upheld)
- False positive count (findings rejected)
- Current trust rate
- Whether promotion/demotion is pending

## 6. Understand Degradation

If a provider becomes unavailable mid-session:

```bash
boundline inspect --workspace .
```

Look for `control.degraded` events showing:
- Original level → degraded level
- Whether the degraded path is safe
- Whether a human gate is required

## Key Files

| File | Purpose |
|------|---------|
| `.boundline/calibration-policy.toml` | Control level configuration |
| `.boundline/guardian-rules.toml` | Guardian activation rules |
| `.boundline/overrides.toml` | Operator override records |
| `.boundline/traces/` | Trust metrics and trace events |

## Common Pitfalls

- **No calibration policy**: If `.boundline/calibration-policy.toml` is missing, all guardians default to `advisory` (visible, no block). Create the policy to enable graduated enforcement.
- **Red-zone advisory**: The policy validates that red-zone entries cannot default to `advisory`. Fix by setting `red_level = rule` or `hook`.
- **Insufficient trust data**: Trust rates are not computed until the minimum evidence threshold is met (default 3 adjudicated findings). No promotion or demotion occurs before this.
- **Hook bypass**: `boundline override` cannot bypass hook-level blocks. Only a privileged process (Canon-approved exception) can bypass hooks.
