# Contract: Cluster Config CLI

## Purpose

Define the user-facing contract for saving and resolving cluster-scoped routing
defaults between workspace-local and user-global configuration.

## Command Surface

### `synod config show`

```text
synod config show \
  [--workspace <member-workspace>] \
  [--cluster <primary-workspace>] \
  [--scope <effective|workspace|cluster|global>]
```

- `--scope cluster` reads only cluster-scoped defaults from the primary
  workspace cluster file.
- `--scope effective` may combine workspace-local, cluster, global, and built-in
  values.

### `synod config set`

```text
synod config set \
  [--workspace <member-workspace>] \
  [--cluster <primary-workspace>] \
  --scope <workspace|cluster|global> \
  [--slot <planning|implementation|verification|review>] \
  [--reviewer <role-id>] \
  [--adjudicator] \
  --runtime <claude|codex|copilot|gemini> \
  --model <model-id>
```

- `--cluster` is required when `--scope cluster` is selected.
- Cluster-scoped writes persist inside `.synod/cluster.toml` in the primary
  workspace.

### `synod config unset`

```text
synod config unset \
  [--workspace <member-workspace>] \
  [--cluster <primary-workspace>] \
  --scope <workspace|cluster|global> \
  [--slot <planning|implementation|verification|review>] \
  [--reviewer <role-id>] \
  [--adjudicator]
```

- Unsetting a cluster-scoped value removes only that cluster-scoped entry.

## Required Behavior

- Effective resolution MUST use the precedence `CLI > workspace > cluster > global > built-in`.
- Effective config output MUST expose the source of every resolved value.
- Cluster-scoped mutations MUST preserve cluster membership metadata.
- Workspace-scoped mutations MUST remain local to the requested member workspace.

## Validation Rules

- `--scope cluster` must fail if no `--cluster` primary workspace is provided.
- Cluster-scoped reads and writes must fail if the referenced cluster file is
  missing or malformed.
- Invalid role identifiers, duplicate role definitions, or missing target slots
  must fail before any write occurs.
- Cluster-scoped config may be read even if a member workspace has no local
  config file.

## Compatibility Rules

- Global and workspace config file formats remain valid after adding cluster
  scope.
- Single-workspace `config show|set|unset` behavior remains unchanged when no
  cluster scope is requested.