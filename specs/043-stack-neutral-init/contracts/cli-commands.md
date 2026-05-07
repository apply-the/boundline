# CLI Contract: Stack-Neutral Entry And Assistant Defaults

## `boundline doctor --workspace <workspace>`

### Input expectations

- Accepts any existing local directory as `<workspace>`.
- Must not require a language-specific manifest as a generic readiness prerequisite.

### Output expectations

- Reports workspace availability, writability, trace-store state, and execution-profile state.
- If blocked, reports only the concrete blocking reasons that prevent native entry.
- If ready, does not imply a stack choice that planning has not yet made.

## `boundline init --workspace <workspace> --assistant <runtime>`

### Input expectations

- Accepts one or more assistant targets from the supported catalog.
- Accepts explicit `--route` overrides that supersede any seeded defaults.
- May accept domain selections that later drive hygiene defaults.

### Output expectations

- Reports the initialized workspace, template, assistant runtimes, and the route defaults chosen automatically.
- Distinguishes seeded defaults from explicit overrides.
- Reports any hygiene files created or updated, or why a candidate hygiene pack was skipped.

## `boundline config show --workspace <workspace> --scope effective`

### Output expectations

- Surfaces effective routing with authoritative source markers.
- Preserves the selected assistant bindings.
- Makes it inspectable which slot values came from built-in defaults, init-time seeded defaults, or explicit overrides.
