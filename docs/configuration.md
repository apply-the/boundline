# Configuration in Synod 0.24.0

Synod `0.24.0` keeps a user-friendly setup and routing configuration surface
for the session-native runtime plus explicit compatibility/bootstrap workflows.

The `0.24.0` release keeps configuration behavior stable while preserving the
same governed routing defaults across earlier `bug-fix:investigate` work,
later verify-stage `security-assessment`, workflow-aware projection of the
same bounded governance state, continuity-aware read-side follow-up, and the
broader bounded adaptive repair slice. Adaptive repair still lives in the
explicit compatibility execution manifest; there is no separate routing knob
for mutation-family selection, credibility ranking, or explicit adaptive
exhaustion. The main CLI read-side surfaces now also project material routing
facts through `route_config_projection` and keep the active `route_owner`
explicit when a workspace mixes native, workflow, governance, or compatibility
state.

## What changed

- `synod init` bootstraps an optional compatibility workspace profile and local config under `.synod/`
- `synod config` manages runtime/model routing defaults for planning, verification, review, and other bounded slots
- `synod cluster` registers bounded multi-workspace membership and aggregated inspection
- continuity between explicit compatibility traces and read-side commands is projected by the CLI surfaces, not by a new config key
- adaptive validation-guided repair remains configured in `.synod/execution.json`, not in `config.toml`
- broader adaptive mutation families and explicit exhaustion behavior are still
  configured by the execution manifest, not by new config keys
- routing values can be global, cluster-scoped, workspace-local, or command-specific
- review councils and adjudication can use distinct routing defaults
- `status`, `next`, and `inspect` now surface material workspace-local routing
	defaults when they explain the current follow-up story instead of forcing the
	operator to cross-reference `synod config show`

## Config locations

- Workspace-local: `<workspace>/.synod/config.toml`
- Cluster-scoped: `<primary-workspace>/.synod/cluster.toml`
- User-global: `$XDG_CONFIG_HOME/synod/config.toml`
- User-global fallback: `$HOME/.config/synod/config.toml`

Workflow definitions are separate from routing config:

- Workspace-local workflow registry: `<workspace>/.synod/workflows.toml`

That file declares named bounded workflows. It does not participate in runtime
or model-routing precedence; it is consumed only by `synod workflow ...`.

Optional workflow-discovery metadata lives in the same registry file:

```toml
[workflow.governed-delivery]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "review", "govern", "inspect"]
allow_review = true
allow_governance = true
summary = "bounded delivery path with review and governance before completion"
recommended_when = "the task needs explicit review and governance evidence"

[workflow.governed-delivery.when]
review = "review_triggered"
governance = "governance_required"
```

Use `synod workflow list --workspace <workspace>` to render the discovered
workflow names, phase chains, summary text, and invocation guidance. If
`summary` or `recommended_when` are omitted, Synod falls back to the workflow
name plus declared phases. The registry remains bounded: no branching, loops,
fan-out, fan-in, hidden background progression, or Canon-owned workflow control
are supported.

## Resolution precedence

Synod resolves each routing slot with this order:

1. explicit CLI input
2. workspace config
3. cluster config
4. global config
5. built-in defaults

Use `synod config show --scope effective --workspace <workspace> --cluster <primary-workspace>` to inspect
resolved values and their source.

When follow-up commands already know which route owns the current work,
Synod reuses that information and projects only the routing/config facts that
materially explain the current state. In practice, `route_config_projection`
may include workspace-local routing defaults, workflow cues, or requested
governance intent; it intentionally does not dump every possible config value.

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

You do not need `synod init` to use the primary session-native workflow. Use it when you want scaffolded compatibility defaults or assistant setup.

## Init templates

`synod init` works without a template flag. If you omit `--template`, Synod
defaults to `bug-fix`.

Available starting templates:

- `bug-fix`: small targeted repair
- `change`: bounded implementation change
- `delivery`: broader delivery update

Templates are only starting points for the generated compatibility execution profile.

- Need a different starting point later: rerun `synod init --workspace <workspace> --force --template <bug-fix|change|delivery>`
- Need another task of the same type: do not rerun init; start a new session and capture a new goal
- Need something custom: edit `<workspace>/.synod/execution.json` directly when you intentionally want compatibility behavior

`init` template and `flow` are separate concerns: `init` bootstraps the
optional compatibility profile, while `synod flow` selects the current session-native run shape.

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
