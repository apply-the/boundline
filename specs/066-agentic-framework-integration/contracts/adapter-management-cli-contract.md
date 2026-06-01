# Contract: Adapter Management CLI

## Purpose

Define the host-owned CLI surface for explicit adapter registration, inspection,
and removal, plus the stable persisted shape stored in `.boundline/config.toml`.
This contract belongs to the Boundline repo. `boundline init` may call the same
underlying service, but it must not invent a separate persistence shape.

## Command Family

### 1. `boundline adapter add`

**Known profile form**:

```bash
boundline adapter add speckit --workspace <workspace>
```

**Custom form**:

```bash
boundline adapter add custom --workspace <workspace> --id <adapter-id> --command <binary>
```

**Supported options**:

- `--workspace <path>`: target workspace; defaults to the nearest initialized
  workspace using the same resolution rules as other Boundline commands
- `--command <binary-or-path>`: override the default binary for a known profile
  or provide the required binary for a custom adapter
- `--arg <value>`: append a fixed adapter argument; may be repeated
- `--set key=value`: seed one or more adapter field values non-interactively;
  may be repeated
- `--non-interactive`: forbid guided prompts and fail if required values or
  adapter discovery are unresolved
- `--json`: emit a machine-readable result in addition to or instead of the
  human summary

**Behavior**:

1. Resolve the profile (`speckit` or `custom`).
2. Determine the adapter command from `--command`, known-profile defaults, or
   bounded PATH discovery.
3. Invoke the adapter `describe` command to obtain capabilities and the config
   schema.
4. Collect or validate required config values.
5. Invoke the adapter `preflight` command with the proposed config values.
6. Persist the adapter selection only when the preflight result is `ready`.

**Success guarantees**:

- the active adapter selection is explicit and operator-controlled
- `.boundline/config.toml` is the authoritative persisted source of truth
- known-profile setup may prefill defaults, but it must not skip validation

**Failure guarantees**:

- a missing binary, malformed manifest, incompatible protocol line, or missing
  required field blocks activation
- `--non-interactive` may never prompt implicitly
- local executable discovery may suggest a command, but it must not activate the
  adapter without a completed `add` action

### 2. `boundline adapter show`

**Form**:

```bash
boundline adapter show --workspace <workspace>
boundline adapter show --workspace <workspace> --json
```

**Behavior**:

- when no adapter is selected, the command reports that built-in behavior is the
  active default
- when an adapter is selected, the command reports adapter ID, display name,
  command, compatibility line, last validated config state, supported
  transports, declared stages, and declared hooks
- the JSON form includes the same stable fields plus machine-readable config and
  compatibility status

**Redaction rules**:

- secret adapter values must be redacted in human-readable output
- secret adapter values may appear in persisted config when required by the
  chosen storage design, but the `show` output must never print them verbatim

### 3. `boundline adapter remove`

**Form**:

```bash
boundline adapter remove --workspace <workspace>
```

**Behavior**:

- removes the active adapter selection and stored adapter-specific values from
  `.boundline/config.toml`
- does not mutate unrelated routing or Canon preferences
- restores built-in behavior as the effective lifecycle source for subsequent
  runs

## Persisted Config Shape

The persisted shape extends `.boundline/config.toml` with an optional top-level
adapter block. If the block is absent, built-in behavior remains active.

```toml
[adapter]
selection_mode = "known_profile"
adapter_id = "speckit"
display_name = "Speckit"
command = "boundline-adapter-speckit"
compatibility_line = "framework-adapter-v1"

[[adapter.values]]
key = "template_repo"
value_kind = "path"
path_value = "../boundline-framework-template"

[[adapter.values]]
key = "adapter_repo"
value_kind = "path"
path_value = "../boundline-adapter-speckit"
```

## JSON Result Shape

### `adapter add --json` success

```json
{
  "status": "ready",
  "adapter_id": "speckit",
  "selection_mode": "known_profile",
  "command": "boundline-adapter-speckit",
  "compatibility_line": "framework-adapter-v1",
  "supported_transports": [
    {
      "transport": "stdio",
      "encoding": "json",
      "request_channel": "stdin",
      "response_channel": "stdout"
    }
  ],
  "declared_stage_overrides": ["plan", "run"],
  "declared_hook_subscriptions": ["stage_completed", "stage_failed"],
  "config_state": "complete"
}
```

### `adapter show --json` when an adapter is selected

```json
{
  "status": "ready",
  "adapter_id": "speckit",
  "selection_mode": "known_profile",
  "command": "boundline-adapter-speckit",
  "compatibility_line": "framework-adapter-v1",
  "supported_transports": [
    {
      "transport": "stdio",
      "encoding": "json",
      "request_channel": "stdin",
      "response_channel": "stdout"
    }
  ],
  "declared_stage_overrides": ["plan", "run"],
  "declared_hook_subscriptions": ["stage_completed", "stage_failed"],
  "config_state": "complete"
}
```

### `adapter show --json` when no adapter is selected

```json
{
  "status": "built_in_default",
  "execution_source": "built_in"
}
```

### `adapter add --json` failure

```json
{
  "status": "blocked",
  "adapter_id": "speckit",
  "reason": "missing_required_config",
  "missing_fields": ["template_repo"],
  "recovery": "boundline adapter add speckit --workspace <workspace>"
}
```

## Invariants

- At most one adapter selection may exist per workspace.
- Removing the adapter selection restores built-in behavior without requiring
  any extra cleanup step.
- Known profile `speckit` maps to adapter ID `speckit`, binary
  `boundline-adapter-speckit`, and registration command
  `boundline adapter add speckit`.
- `boundline init` may expose adapter setup, but it must write the same adapter
  block and surface the same failure states as `boundline adapter add`.