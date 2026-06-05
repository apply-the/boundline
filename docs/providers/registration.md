# Provider Registration

Registration is explicit. Discovery alone does not trust or activate a
provider.

## Basic Flow

Register a provider:

```bash
boundline provider add demo --workspace <workspace> --command python3 --arg scripts/provider.py
```

Inspect the current workspace selection:

```bash
boundline provider show --workspace <workspace> --json
```

Run a readiness check:

```bash
boundline provider health --workspace <workspace>
```

Remove a registration:

```bash
boundline provider remove demo --workspace <workspace>
```

## Setup Requirements

Providers may declare setup requirements such as:

- missing config refs
- missing secret handles
- missing local binaries
- missing environment prerequisites

If setup is incomplete, Boundline persists the registration but does not
promote it to the active provider. If another provider is already active, that
previous active provider remains authoritative until the replacement completes
activation cleanly.

## Operator Rule

Treat `provider show --json` as the stable inspection surface for:

- provider ID
- activation state
- declared capability IDs
- setup requirements
- bounded summary
