# Configuration in Synod 0.11.0

Synod `0.11.0` adds a user-friendly setup and routing configuration surface.

## What changed

- `synod init` bootstraps bounded workspace files under `.synod/`
- `synod config` manages runtime/model routing defaults
- routing values can be global, workspace-local, or command-specific
- review councils and adjudication can use distinct routing defaults

## Config locations

- Workspace-local: `<workspace>/.synod/config.toml`
- User-global: `$XDG_CONFIG_HOME/synod/config.toml`
- User-global fallback: `$HOME/.config/synod/config.toml`

## Resolution precedence

Synod resolves each routing slot with this order:

1. explicit CLI input
2. workspace config
3. global config
4. built-in defaults

Use `synod config show --scope effective --workspace <workspace>` to inspect
resolved values and their source.

## Init workflow

```bash
synod init --workspace <workspace> --template bug-fix
synod doctor --workspace <workspace>
```

When init would overwrite existing files, Synod shows a preview and requires
`--force` to apply destructive updates.

## Config commands

### Show

```bash
synod config show --workspace <workspace> --scope effective
synod config show --workspace <workspace> --scope workspace
synod config show --scope global
```

### Set delivery-stage routes

```bash
synod config set --scope global --slot planning --runtime codex --model gpt-5-codex
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
synod config unset --workspace <workspace> --scope workspace --reviewer safety
```

## Runtime support

Initial runtime support for routing and assistant setup:

- Claude
- Codex
- Copilot
- Gemini CLI

Gemini is currently treated as CLI-only in this slice.
