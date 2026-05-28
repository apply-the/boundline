# Configuration in Boundline 0.64.0

This page covers the operator-facing configuration surface. Keep one rule in
mind: configuration declares defaults and policy; the runtime still owns
session state, planning gates, traces, and follow-through.

## What Config Is For

Use configuration to control:

- routing defaults
- assistant bindings
- runtime capability profiles
- slot effort policies
- Canon workspace preferences
- domain-template defaults
- project-memory roots
- workflow registry references

Use runtime commands to observe or act on session state:

- `boundline goal`
- `boundline plan`
- `boundline run`
- `boundline status`
- `boundline next`
- `boundline inspect`
- `boundline probe`

## Config Locations

- Workspace-local: `<workspace>/.boundline/config.toml`
- Cluster-scoped: `<primary-workspace>/.boundline/cluster.toml`
- User-global: `$XDG_CONFIG_HOME/boundline/config.toml`
- User-global fallback: `$HOME/.config/boundline/config.toml`

When you already run Boundline inside the target repository, you can usually
omit `--workspace`: the CLI prefers the nearest initialized `.boundline/`
ancestor, then the nearest `.git/` root, and only then falls back to the
current working directory.

## Resolution Precedence

Boundline resolves most runtime values in this order:

1. explicit CLI input
2. workspace config
3. cluster config
4. user-global config
5. built-in defaults

Inspect the effective result with:

```bash
boundline config show --scope effective
```

## What Stays Runtime-Owned

These surfaces are not configuration keys:

- `.boundline/session.json`
- `.boundline/traces/`
- planning gates such as `goal_quality_state`, `plan_quality_state`,
  `backlog_quality_state`, and `planning_analysis_state`
- host handoffs such as `phase_request`, `assistant_resume_command`, and
  `assistant_next_command`
- read-only readiness output from `boundline probe`

If those fields change, the runtime decided something from current evidence.

## Workspace Bootstrap

`boundline init` creates or updates the workspace-facing config surface.

Common examples:

```bash
boundline init --assistant codex
```

```bash
boundline init \
  --assistant copilot \
  --route planning=copilot:gpt-4o \
  --route implementation=codex:o3
```

`init` can also:

- seed domain defaults
- write Canon preferences
- scaffold IDE metadata
- export repo-local reference docs

For the full bootstrap and refresh flow, see
[guides/init-and-update.md](guides/init-and-update.md).

## Provider Auth Profiles

Provider auth is adjacent to config but separate from it.

```bash
boundline models auth login --provider github-copilot
boundline models auth status
boundline models auth remove --provider github-copilot
```

- Auth profiles are user-scoped.
- They live beside the user-global config in `auth-profiles.json`.
- They are not written into `.boundline/config.toml` or `.boundline/session.json`.
- In the current public slice, the login surface supports `github-copilot`.

Treat these profiles as operator credentials, not repository configuration.

## Routing And Assistant Setup

Assistant package setup and slot routing are related but different:

- `boundline init --assistant <host>` generates repo-local assistant surfaces.
- `boundline config` decides which runtime or model owns planning,
  implementation, verification, review, and other slots.

Use `config show --scope effective` when a route behaves differently from what
you expected.

## Canon Workspace Preferences

Canon defaults are workspace-local when governed delivery is expected.

```bash
boundline init \
  --canon-mode-selection auto-confirm \
  --risk medium \
  --zone engineering \
  --owner platform
```

You can also inspect or adjust Canon defaults directly:

```bash
boundline config show --scope workspace
boundline config set-canon --workspace . --mode-selection auto-confirm
```

The current release documents Canon `0.61.0` support for the machine-facing
`canon governance start|refresh|capabilities --json` `v1` surface.

## Workflow Registry Boundaries

Authored workflows stay intentionally narrow. They describe one bounded route,
not a hidden workflow engine.

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

The supported authored shape has no branching, loops, fan-out, fan-in, hidden background progression, or Canon-owned workflow control. Workflows provide named entrypoints over the same runtime; governance still remains explicit and route-driven.

## Advanced Context And Semantic Acceleration

Two optional policy surfaces remain explicit:

```toml
[routing.advanced_context]
retrieval_mode = "local"
remote_policy = "local_only"

[routing.semantic_acceleration]
policy = "disabled"
```

- `advanced_context` controls the baseline local retrieval path.
- `semantic_acceleration` is an additive opt-in for local semantic expansion.
- Neither surface replaces the normal routing precedence rules.

Inspect both through:

```bash
boundline config show --scope effective
```

## Safe Defaults

Use these defaults unless there is a clear reason not to:

- keep normal local delivery on the session-native path
- use `run --compatibility` only when the manifest-backed route is intentional
- treat blocked context, failed validation, and blocked governance as real stop
  conditions
- prefer workspace overrides for local engineering rules
- prefer Canon only when governed standards or governed project memory are
  intentionally part of the delivery path