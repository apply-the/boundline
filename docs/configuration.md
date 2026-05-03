# Configuration in Synod 0.36.0

Synod `0.36.0` keeps a user-friendly setup and routing configuration surface
for the session-native runtime plus explicit compatibility/bootstrap workflows.

The `0.36.0` release keeps configuration behavior stable while preserving the
same governed routing defaults across earlier `bug-fix:investigate` work,
later verify-stage `security-assessment`, workflow-aware projection of the
same bounded governance state, continuity-aware read-side follow-up, the
broader bounded adaptive repair slice, the clustered multi-workspace delivery
path, and the new negotiated delivery projection. Negotiation still does not
introduce a separate config file or routing key: capture derives acceptance
boundaries from direct goals, authored briefs, and governance intent, while
`run`, `status`, `next`, and `inspect` simply project the resulting packet.
Context assembly also stays runtime-owned and inspectable rather than
configuration-driven: planning derives one bounded context pack from workspace
signals and persisted session evidence, while `run`, `status`, `next`, and
`inspect` simply project the resulting context summary, credibility, primary
inputs, provenance, and any staleness reason. The same remains true for the
Canon-grounded memory slice: capability snapshots, compact Canon memory,
governed artifact refs, and recommended next actions are runtime-owned
planning inputs rather than configuration keys. The same remains true for the
new planning lifecycle: default `plan` proposes one bounded evidence-driven
goal plan, `plan --confirm` confirms it, and neither step introduces a new
config key or routing slot.
Decision-driven selector choice is also runtime-owned rather than
configuration-driven: the native loop can choose `read`, `search`, `modify`,
`test`, `ask`, or `replan` from current evidence without introducing a new
config key or route slot.
Direct `run --goal` now boots the native session path without requiring a
workspace execution profile, while `run --compatibility --goal ...` remains the
manifest-backed opt-in.
Workflow discovery and follow-through now use the same primary product story as
direct native execution; compatibility remains explicit and subordinate rather
than a hidden default.
Bug-fix and change completion credibility is now stricter on that same runtime
surface, but it does not introduce a new routing slot, governance flag, or
configuration file key: Synod simply requires a material diff plus passed
validation evidence before those bounded paths can finish successfully.
Adaptive repair still lives in the explicit compatibility execution manifest;
there is no separate routing knob for mutation-family selection, credibility
ranking, explicit adaptive exhaustion, or negotiation-state overrides.

## What changed

- `synod init` bootstraps an optional compatibility workspace profile and local config under `.synod/`
- direct `synod run --goal` is native-first; add `--compatibility` only when the manifest-backed route is intentional
- default `synod plan` now creates one evidence-driven proposal and `synod plan --confirm` confirms it; planning lifecycle state is session-owned rather than config-owned
- bounded `bug-fix` and `change` completion now requires both material change evidence and passed validation on the native and governed session path
- `synod config` manages runtime/model routing defaults for planning, verification, review, and other bounded slots
- `synod cluster` registers bounded multi-workspace membership and aggregated inspection
- negotiated delivery modeling stays session-owned and trace-projected; there is no new negotiation-specific key in `config.toml` or `.synod/execution.json`
- context-pack assembly and credibility projection stay session-owned and trace-projected; there is no new context-specific key in `config.toml` or `.synod/execution.json`
- session-native commands can use `--cluster <primary-workspace>` to keep one authoritative primary-owned session while traversing cluster members sequentially
- continuity between explicit compatibility traces and read-side commands is projected by the CLI surfaces, not by a new config key
- adaptive validation-guided repair remains configured in `.synod/execution.json`, not in `config.toml`
- broader adaptive mutation families and explicit exhaustion behavior are still
  configured by the execution manifest, not by new config keys
- routing values can be global, cluster-scoped, workspace-local, or command-specific
- review councils and adjudication can use distinct routing defaults
- `config show --scope effective` now exposes the resolved slot route, source,
	and assistant binding for each bounded slot
- `run`, `status`, `next`, and `inspect` now surface effective routing plus
	assistant bindings, and native or compatibility traces persist the route
	snapshot used during execution
- `status`, `next`, and `inspect` now also surface guided follow-through fields
	when persisted session or trace evidence can explain one concrete next
	bounded action or explicit stop condition
- when `assistant_runtimes` is non-empty, native execution now fails explicitly
	if the active implementation or verification route requires a missing
	assistant family instead of silently falling back

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

The effective view is now the operator-facing source of truth for backend
ownership: it shows the resolved route for each slot, the source that won
precedence, and the assistant binding implied by that route.

When follow-up commands already know which route owns the current work,
Synod reuses that information and projects only the routing/config facts that
materially explain the current state. In practice, `route_config_projection`
may include persisted `effective_routing`, `assistant_bindings`,
workspace-local routing defaults, workflow cues, or requested governance
intent; it intentionally does not dump every possible config value.

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

The primary workspace owns `.synod/cluster.toml` and remains the authoritative
owner of the active clustered session in `<primary-workspace>/.synod/session.json`.
Member workspaces keep their own `.synod/traces/` and local `.synod/config.toml`
files so terminal evidence stays local to the workspace that executed the
bounded handoff.

Session-native clustered delivery uses the same bounded commands through the
primary workspace:

```bash
synod start --cluster <primary-workspace>
synod capture --cluster <primary-workspace> --goal "Fix the failing add test"
synod plan --cluster <primary-workspace>
synod run --cluster <primary-workspace>
synod status --cluster <primary-workspace>
synod inspect --cluster <primary-workspace>
```

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

Gemini is currently treated as an explicit Gemini CLI fallback in this slice.
If a workspace declares `assistant_runtimes` and the active implementation or
verification route chooses a runtime outside that capability list, native
execution stops with an explicit assistant-binding error.
