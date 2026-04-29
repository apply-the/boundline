# Configuration in Synod 0.11.0

Synod `0.11.0` adds a user-friendly setup and routing configuration surface.

## What changed

- `synod init` bootstraps a starting workspace profile and local config under `.synod/`
- `synod config` manages runtime/model routing defaults
- `synod cluster` registers bounded multi-workspace membership and aggregated inspection
- routing values can be global, cluster-scoped, workspace-local, or command-specific
- review councils and adjudication can use distinct routing defaults

## Config locations

- Workspace-local: `<workspace>/.synod/config.toml`
- Cluster-scoped: `<primary-workspace>/.synod/cluster.toml`
- User-global: `$XDG_CONFIG_HOME/synod/config.toml`
- User-global fallback: `$HOME/.config/synod/config.toml`

## Resolution precedence

Synod resolves each routing slot with this order:

1. explicit CLI input
2. workspace config
3. cluster config
4. global config
5. built-in defaults

Use `synod config show --scope effective --workspace <workspace> --cluster <primary-workspace>` to inspect
resolved values and their source.

## Cluster workflow

```bash
synod cluster init \
	--workspace <primary-workspace> \
	--cluster-id delivery-a \
	--member <primary-workspace> \
	--member <secondary-workspace>

synod cluster status --workspace <primary-workspace>
synod cluster inspect --workspace <primary-workspace>
```

The primary workspace owns `.synod/cluster.toml`. Member workspaces keep their
own `.synod/session.json`, `.synod/traces/`, and local `.synod/config.toml`
files.

## Init workflow

```bash
synod init --workspace <workspace>
synod doctor --workspace <workspace>
```

When init would overwrite existing files, Synod shows a preview and requires
`--force` to apply destructive updates.

## Init templates

`synod init` works without a template flag. If you omit `--template`, Synod
defaults to `bug-fix`.

Available starting templates:

- `bug-fix`: small targeted repair
- `change`: bounded implementation change
- `delivery`: broader delivery update

Templates are only starting points for the generated execution profile.

- Need a different starting point later: rerun `synod init --workspace <workspace> --force --template <bug-fix|change|delivery>`
- Need another task of the same type: do not rerun init; start a new session and capture a new goal
- Need something custom: edit `<workspace>/.synod/execution.json` directly

`init` template and `flow` are separate concerns: `init` bootstraps the
workspace profile, while `synod flow` selects the current run shape.

## Config commands

### Show

```bash
synod config show --workspace <workspace> --scope effective
synod config show --workspace <workspace> --scope workspace
synod config show --cluster <primary-workspace> --scope cluster
synod config show --scope global
```

### Set delivery-stage routes

```bash
synod config set --scope global --slot planning --runtime codex --model gpt-5-codex
synod config set --cluster <primary-workspace> --scope cluster --slot planning --runtime codex --model gpt-5-codex
synod config set --workspace <workspace> --scope workspace --slot verification --runtime copilot --model gpt-5.4
```

### Set review-role routes

```bash
synod config set --workspace <workspace> --scope workspace --reviewer safety --runtime claude --model sonnet-4
synod config set --workspace <workspace> --scope workspace --adjudicator --runtime codex --model gpt-5-codex
```

### Unset values

```bash
synod config unset --workspace <workspace> --scope workspace --slot planning
synod config unset --cluster <primary-workspace> --scope cluster --slot planning
synod config unset --workspace <workspace> --scope workspace --reviewer safety
```

## Runtime support

Initial runtime support for routing and assistant setup:

- Claude
- Codex
- Copilot
- Gemini CLI

Gemini is currently treated as CLI-only in this slice.
