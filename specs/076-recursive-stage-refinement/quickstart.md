# Quickstart: Recursive Stage Refinement Profiles

**Feature**: 076-recursive-stage-refinement
**Date**: 2026-06-07

## What This Feature Does

Adds bounded, inspectable refinement loops to the `boundline plan` command. When enabled, the plan stage runs through a `planner → critic → planner → finalizer` loop that iteratively improves the plan. Each round produces a compact trace packet. The loop stops when the plan stops improving, the round/time budget is exhausted, or a blocker is found.

## Quick Enable

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

## CLI Overrides

- **Enable refinement** for a single run (even without config): `boundline plan --refine`
- **Disable refinement** for a single run (even with config): `boundline plan --no-refine`
- **Change round limit** for a single run: `boundline plan --max-rounds 5`

## Inspecting Refinement

After a plan run with refinement enabled:

```bash
# See the refinement history
boundline inspect

# Check current refinement state (useful mid-execution)
boundline status

# Get recommendation for next action
boundline next
```

## Understanding the Output

### Stop Reasons

| Reason | Meaning |
|--------|---------|
| `no_material_delta` | Plan stopped improving — good, converged |
| `round_limit_exhausted` | Hit the round budget — consider increasing `max_rounds` |
| `time_limit_exhausted` | Timed out — consider increasing `max_elapsed_time_seconds` |
| `unresolved_blocker` | A blocking finding couldn't be resolved in the budget |
| `empty_candidate` | Provider returned nothing — check provider health |
| `provider_failure` | Provider failed mid-round — check provider logs |

### Outcome Labels

- **`finalized`**: Plan is ready for the next stage (`boundline run`)
- **`incomplete`**: Plan needs attention — review outstanding findings and re-run

## Disabling

Remove or comment out the profile, or run `boundline plan --no-refine`:

```bash
boundline plan --no-refine
```

## Key Constraints

- **One stage only**: Refinement currently works for `plan` only.
- **One profile only**: Only `plan_refinement` is supported in this release.
- **No hidden behavior**: Every round is trace-visible. No latent ML state.
- **No new dependencies**: Uses existing providers, registry, and trace store.

## Troubleshooting

| Symptom | Check |
|---------|-------|
| "max_rounds must be >= 1, got 0" | Fix `max_rounds` in config or CLI flag |
| "max_elapsed_time_seconds must be > 0, got 0" | Fix `max_elapsed_time_seconds` in config |
| "provider 'X' not found in registry" | Verify provider ID is registered and active |
| Refinement didn't run | Check `enabled = true` in config or use `--refine` |
| Loop ran but plan didn't improve | Check `boundline inspect` for findings — address them and re-run |
