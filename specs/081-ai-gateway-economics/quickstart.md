# Quickstart: AI Gateway And Inference Economics

**Feature Branch**: `081-ai-gateway-economics` | **Date**: 2026-06-18

## Prerequisites

- Rust 1.96.0+ with edition 2024
- Existing Boundline workspace with `.boundline/` directory
- At least one configured AI provider (via `boundline provider add`)

## 1. Configure a Session Budget

```bash
# Set a $10.00 USD budget for the current workspace
boundline config set-budget --currency USD --limit 10.00

# Verify
boundline status
# Output includes:
#   Budget: $10.00 USD
#   Spent: $0.00 | Reserved: $0.00 | Remaining: $10.00
```

## 2. Configure Pricing Snapshots

Create a pricing snapshot file (`pricing.toml`):

```toml
[snapshot]
id = "2026-06-18-v1"
schema_version = 1
effective = "2026-06-18T00:00:00Z"
source = "operator-created"

[[snapshot.entries]]
provider_id = "openai"
model_id = "gpt-4o"
input_price_per_1k = "0.00250"
output_price_per_1k = "0.01000"
native_currency = "USD"

[[snapshot.entries]]
provider_id = "anthropic"
model_id = "claude-sonnet-4-20250514"
input_price_per_1k = "0.00300"
output_price_per_1k = "0.01500"
native_currency = "USD"
```

Activate it:

```bash
boundline provider set-pricing --snapshot pricing.toml
```

## 3. Run Governed Work with Budget Enforcement

```bash
# Interactive run — budget enforced, approval prompts appear when needed
boundline run --goal "Refactor the authentication module"

# Non-interactive run — blocks on unknown cost unless pre-authorized
boundline run --goal "Run integration tests" --non-interactive

# Resume with budget state preserved
boundline run --resume
```

## 4. Approve Spend Exceptions (Interactive)

When a call requires approval:

```
> boundline run --goal "Audit security configuration"
...
[budget] Unknown cost detected for provider=openai model=gpt-4o
[budget] Budget State: approval_required
[budget] Remaining known budget: $5.80 USD
[budget] Required approver role: session_owner (low-risk, non-egress)

Approve this call? [y/N/skip/show]: y
Reason: Security audit requires latest model
[budget] Approval recorded: single_call, expires in 60 minutes
```

## 5. Inspect Budget and Cost History

```bash
# Budget overview
boundline inspect cost --summary

# Detailed call history
boundline inspect cost --calls

# Check snapshot staleness
boundline inspect cost --snapshots
```

## 6. Verify Routing Behavior

```bash
# Show which routes are active and their health
boundline provider status --verbose

# Show route selection for a hypothetical task
boundline inspect route --task-class implementation --authority-zone yellow
```

## Key Behaviors

| Scenario | Behavior |
|----------|----------|
| Budget not configured | Economics disabled; no enforcement |
| Call within budget | Reservation recorded, call proceeds |
| Call exceeds budget (low-risk, non-egress) | Paused; session owner may approve |
| Call exceeds budget (red-zone) | Paused; governance approver required |
| Unknown cost (non-interactive) | Blocked unless pre-authorized |
| Exact provider cost arrives late | Reconciled; original approval preserved |
| Snapshot stale | Estimate marked stale_estimate; call not auto-blocked |
| New snapshot activated mid-session | Future reservations only; history unchanged |
