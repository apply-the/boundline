# Contract: Config CLI

## Purpose

Define the user-facing contract for inspecting, setting, and removing Synod
runtime and model routing configuration at global or workspace scope.

## Command Surface

### `synod config show`

```text
synod config show \
  [--workspace <path>] \
  [--scope <effective|workspace|global>]
```

- Default scope is `effective`.
- `--workspace` is required when the requested scope needs workspace-local
  values or effective precedence resolution.
- Output must show each supported route together with the source of the value.

### `synod config set`

```text
synod config set \
  [--workspace <path>] \
  --scope <workspace|global> \
  [--slot <planning|implementation|verification|review>] \
  [--reviewer <role-id>] \
  [--adjudicator] \
  --runtime <claude|codex|copilot|gemini> \
  --model <model-id>
```

- Exactly one target must be selected: a delivery `--slot`, one `--reviewer`, or
  `--adjudicator`.
- `--workspace` is required when `--scope workspace` is used.
- `--scope global` writes to the user-scoped config file.

### `synod config unset`

```text
synod config unset \
  [--workspace <path>] \
  --scope <workspace|global> \
  [--slot <planning|implementation|verification|review>] \
  [--reviewer <role-id>] \
  [--adjudicator]
```

- Unsetting a value removes it from the selected scope so lower-precedence
  values may apply again.

## Required Behavior

- Config commands MUST create the target config file if it does not yet exist.
- Config commands MUST validate runtime/model choices before persisting them.
- `config show` MUST surface resolved effective routing and the origin of each
  value.
- `config unset` MUST not delete unrelated configuration.

## Validation Rules

- Global-scope mutation must not require a workspace path.
- Workspace-scope mutation must fail if no workspace path is provided.
- Invalid role identifiers, duplicate role definitions, or missing target slots
  must fail before writing files.
- If the chosen runtime is unavailable for the requested route, the command must
  fail with an actionable explanation rather than storing a silently broken value.

## Compatibility Rules

- Config commands must preserve existing execution.json behavior; they only
  manage user-facing routing preferences.
- Workspace config may override global values selectively without copying the
  entire global file.
- Manual file editing remains possible, but CLI commands are the primary
  documented workflow.