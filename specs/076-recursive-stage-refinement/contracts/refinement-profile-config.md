# Refinement Profile Configuration Contract

**Feature**: 076-recursive-stage-refinement
**Version**: 1.0
**Date**: 2026-06-07

## Overview

Refinement profiles are configured in `.boundline/refinement-profiles.toml`. This file is the primary mechanism for enabling and configuring refinement loops. CLI flags (`--refine`, `--no-refine`, `--max-rounds N`) provide per-run overrides.

## File Location

```
.boundline/refinement-profiles.toml
```

Relative to the workspace root.

## TOML Schema

```toml
# Refinement profiles configuration
# Each profile maps to a stage and defines the refinement loop parameters.

[profiles.plan_refinement]
# Whether refinement is active for the plan stage.
# CLI --no-refine overrides this to false for the current command.
enabled = true

# Hard round limit. Must be >= 1 after resolving config and CLI overrides.
# Zero values fail visibly before any refinement round starts.
# CLI --max-rounds N overrides this for the current command.
max_rounds = 3

# Hard time limit in seconds. Must be > 0.
# Zero values fail visibly before any refinement round starts.
max_elapsed_time_seconds = 300

# Provider ID mapping for refinement roles.
# Each ID must resolve through the existing provider registry.
# Missing, inactive, or unauthorized providers fail visibly before the first round.
[profiles.plan_refinement.roles]
planner_provider_id = "openai-gpt-5"
critic_provider_id = "openai-gpt-5"
finalizer_provider_id = "openai-gpt-5"
```

## Field Reference

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `profiles.<name>.enabled` | `bool` | No | `false` | — |
| `profiles.<name>.max_rounds` | `u32` | No | `3` | Must be ≥ 1; zero fails visibly |
| `profiles.<name>.max_elapsed_time_seconds` | `u64` | No | `300` | Must be > 0; zero fails visibly |
| `profiles.<name>.roles.planner_provider_id` | `String` | Only when effective `enabled=true` | — | Must resolve in registry |
| `profiles.<name>.roles.critic_provider_id` | `String` | Only when effective `enabled=true` | — | Must resolve in registry |
| `profiles.<name>.roles.finalizer_provider_id` | `String` | Only when effective `enabled=true` | — | Must resolve in registry |

## Default Values

When the file does not exist or a profile is not defined, the following built-in defaults apply:

| Field | Built-in Default |
|-------|-----------------|
| `enabled` | `false` (no refinement unless explicitly enabled) |
| `max_rounds` | `3` |
| `max_elapsed_time_seconds` | `300` |

## CLI Override Resolution

CLI flags override config values for the current command only. Resolution order:

1. Start with built-in defaults
2. Apply values from `.boundline/refinement-profiles.toml` (if present)
3. Apply CLI overrides: `--refine` sets `enabled = true`, `--no-refine` sets `enabled = false`, `--max-rounds N` sets `max_rounds = N`

The effective values and their source (config, CLI, or built-in) must be recorded in the trace.

## Validation Rules

1. **Zero max_rounds**: Fails visibly before any refinement round starts. Error message: `"max_rounds must be >= 1, got 0"`.
2. **Zero max_elapsed_time**: Fails visibly before any refinement round starts. Error message: `"max_elapsed_time_seconds must be > 0, got 0"`.
3. **Unresolved provider**: Fails visibly with the unresolved provider ID. Error message: `"provider '{id}' not found in registry"`.
4. **Inactive provider**: Fails visibly. Error message: `"provider '{id}' is registered but not active"`.
5. **Unauthorized provider**: Fails visibly. Error message: `"provider '{id}' failed permission admission"`.
6. **Duplicate provider IDs across roles**: Not an error. Role separability is a configuration concern, not a runtime restriction.

## Contract Tests

1. **Valid config**: A well-formed `.boundline/refinement-profiles.toml` with all fields loads without error.
2. **Missing file**: When the file does not exist, built-in defaults are used and no error is raised.
3. **Zero max_rounds**: Configuration with `max_rounds = 0` fails with a visible error.
4. **Zero max_elapsed_time**: Configuration with `max_elapsed_time_seconds = 0` fails with a visible error.
5. **Unresolved provider**: Configuration referencing a provider ID not in the registry fails with a visible error.
6. **CLI override precedence**: `--max-rounds 5` overrides config `max_rounds = 3`.
7. **--no-refine bypass**: `--no-refine` sets `enabled = false` regardless of config.
8. **--refine activation**: `--refine` sets `enabled = true` even when config is absent.
9. **Trace records source**: After a plan run, the trace shows whether `max_rounds` came from config, CLI, or built-in defaults.
