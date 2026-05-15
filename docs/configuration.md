# Configuration in Boundline 0.54.0

Boundline `0.54.0` keeps a user-friendly setup and routing configuration surface
for the session-native runtime plus explicit compatibility/bootstrap workflows.

The `0.54.0` release keeps configuration behavior stable while preserving the
same governed routing defaults across earlier `bug-fix:investigate` work,
later verify-stage `security-assessment`, workflow-aware projection of the
same bounded governance state, continuity-aware read-side follow-up, the
broader bounded adaptive repair slice, the clustered multi-workspace delivery
path, the new negotiated delivery projection, and the new domain-template
surface. Negotiation still does not
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
Runtime capability profiles and slot effort policies are the new explicit
exception: operators can now declare what each assistant family credibly
supports and how much effort each routed slot should prefer, while the runtime
continues to own the resulting delegation or stop semantics.
Domain templates extend that same explicit config surface: operators can now
declare active domain families, layered standards, and optional or required
external context bindings per scope, while the runtime still owns target
selection, credibility checks, and blocked-domain stop conditions.
Direct `run --goal` now boots the native session path without requiring a
workspace execution profile, while `run --compatibility --goal ...` remains the
manifest-backed opt-in.
Workflow discovery and follow-through now use the same primary product story as
direct native execution; compatibility remains explicit and subordinate rather
than a hidden default.
Bug-fix and change completion credibility is now stricter on that same runtime
surface, but it does not introduce a new routing slot, governance flag, or
configuration file key: Boundline simply requires a material diff plus passed
validation evidence before those bounded paths can finish successfully.
Adaptive repair still lives in the explicit compatibility execution manifest;
there is no separate routing knob for mutation-family selection, credibility
ranking, explicit adaptive exhaustion, or negotiation-state overrides.

## What changed

- `boundline init` bootstraps an optional compatibility workspace profile and local config under `.boundline/`
- `boundline init --export-docs` can also mirror stable repo-local Canon and selected assistant reference docs under `docs/boundline/`; that export is create-only by default, `--refresh` enables in-place updates, `--diff` previews changes without writing, and `--to <path>` switches the export root
- `boundline init` can also infer or accept active domain families and seed scoped domain-template defaults plus optional external context bindings
- direct `boundline run --goal` is native-first; add `--compatibility` only when the manifest-backed route is intentional
- default `boundline plan` now creates one evidence-driven proposal and `boundline plan --confirm` confirms it; planning lifecycle state is session-owned rather than config-owned
- bounded `bug-fix` and `change` completion now requires both material change evidence and passed validation on the native and governed session path
- `boundline config` manages runtime/model routing defaults, runtime capability profiles, slot effort policy, and domain-template settings for planning, implementation, verification, review, and other bounded slots
- `boundline cluster` registers bounded multi-workspace membership and aggregated inspection
- negotiated delivery modeling stays session-owned and trace-projected; there is no new negotiation-specific key in `config.toml` or `.boundline/execution.json`
- context-pack assembly and credibility projection stay session-owned and trace-projected; there is no new context-specific key in `config.toml` or `.boundline/execution.json`
- repo-visible `project.boundline.toml` can override the default Canon knowledge roots through `[docs] project_memory = "..."` and `evidence = "..."`; Boundline still defaults to `docs/project/` and `docs/evidence/` when the repo does not override them
- Canon project-memory continue, warning, and hard-stop outcomes stay runtime-owned, but `plan`, `status`, `next`, and `inspect` now project the resulting compatibility state, Canon refs, and any producer-attributed managed-block summaries discovered under `docs/evidence/`
- session-native commands can use `--cluster <primary-workspace>` to keep one authoritative primary-owned session while traversing cluster members sequentially
- continuity between explicit compatibility traces and read-side commands is projected by the CLI surfaces, not by a new config key
- adaptive validation-guided repair remains configured in `.boundline/execution.json`, not in `config.toml`
- broader adaptive mutation families and explicit exhaustion behavior are still
  configured by the execution manifest, not by new config keys
- routing values can be global, cluster-scoped, workspace-local, or command-specific
- review councils and adjudication can use distinct routing defaults
- `config show --scope effective` now exposes the resolved slot route, source,
	assistant binding, declared runtime capability summary, and declared effort
	policy for each bounded slot
- `config show --scope effective` now also exposes active domain templates,
	winning standards layers, and bound external context references with their
	source authority
- `run`, `status`, `next`, and `inspect` now surface effective routing plus
	assistant bindings, and native or compatibility traces persist the route
	snapshot used during execution
- workspace-local `.boundline/guidance/` and `.boundline/guardians/` can now
	override bundled capability packs with explicit loaded and skipped-source
	disclosure; capability precedence remains runtime-owned rather than a new
	config key
- `plan`, `run`, `status`, `next`, and `inspect` now surface selected domain
	family, winning standards source, and any required external-input blocking
	reason inside the bounded context story
- `status`, `next`, and `inspect` now also surface guided follow-through fields
	when persisted session or trace evidence can explain one concrete next
	bounded action or explicit stop condition
- when `assistant_runtimes` is non-empty or a declared runtime capability marks
	continuation or validation unsupported, native execution now stops at an
	explicit delegation boundary instead of silently falling back

## Config locations

- Workspace-local: `<workspace>/.boundline/config.toml`
- Cluster-scoped: `<primary-workspace>/.boundline/cluster.toml`
- User-global: `$XDG_CONFIG_HOME/boundline/config.toml`
- User-global fallback: `$HOME/.config/boundline/config.toml`

## Canon Workspace Preferences

Canon-default behavior is workspace-local. `boundline init` can write the
`[canon]` section directly:

```bash
boundline init \
  --workspace <workspace> \
  --canon-mode-selection auto-confirm \
  --risk medium \
  --zone engineering \
  --owner platform \
  --assistant copilot \
  --route planning=copilot:gpt-4o
```

The resulting config stores:

```toml
[canon]
mode_selection = "auto-confirm"
default_risk = "medium"
default_zone = "engineering"
default_owner = "platform"
```

Change the preference later with:

```bash
boundline config set-canon --workspace <workspace> --mode-selection auto
boundline config show --workspace <workspace> --scope workspace
```

Valid mode-selection values are `manual`, `auto-confirm`, and `auto`. The
setting controls whether `boundline run --mode <canon-mode>` is required, whether
Boundline asks before using an inferred mode, or whether it can proceed when its
mode inference is high confidence.

Workflow definitions are separate from routing config:

- Workspace-local workflow registry: `<workspace>/.boundline/workflows.toml`

That file declares named bounded workflows. It does not participate in runtime
or model-routing precedence; it is consumed only by `boundline workflow ...`.

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

Use `boundline workflow list --workspace <workspace>` to render the discovered
workflow names, phase chains, summary text, and invocation guidance. If
`summary` or `recommended_when` are omitted, Boundline falls back to the workflow
name plus declared phases. The registry remains bounded: no branching, loops,
fan-out, fan-in, hidden background progression, or Canon-owned workflow control
are supported.

## Resolution precedence

Boundline resolves each routing slot with this order:

1. explicit CLI input
2. workspace config
3. cluster config
4. global config
5. built-in defaults

Use `boundline config show --scope effective --workspace <workspace> --cluster <primary-workspace>` to inspect
resolved values and their source.

The effective view is now the operator-facing source of truth for backend
ownership: it shows the resolved route for each slot, the source that won
precedence, and the assistant binding implied by that route.

When follow-up commands already know which route owns the current work,
Boundline reuses that information and projects only the routing/config facts that
materially explain the current state. In practice, `route_config_projection`
may include persisted `effective_routing`, `assistant_bindings`,
`runtime_capabilities`, `slot_effort_policies`,
workspace-local routing defaults, workflow cues, or requested governance
intent; it intentionally does not dump every possible config value.

## Guidance And Guardian Overrides

Guidance and guardian selection is mostly runtime-owned, but two repository-local
inputs can change the capability set that `plan`, `run`, `status`, `next`, and
`inspect` report:

- `.boundline/guidance/*.md` adds high-precedence guidance overrides.
- `.boundline/guardians/*.toml` adds high-precedence guardian overrides.

These files are not a second routing layer. Boundline resolves them per lifecycle
phase alongside optional Canon guidance and bundled `assistant/packs/*.toml`, then
persists which sources loaded and which were skipped.

Semantic guardians reuse the existing effective slot for the active phase:

- planning and architecture use the planning slot
- implementation uses the implementation slot
- testing and verification use the verification slot
- review uses the review slot

If the winning route lacks validation support, Boundline records an explicit
guardian degradation instead of silently skipping the check.

## Cluster workflow

```bash
boundline cluster init \
	--workspace <primary-workspace> \
	--cluster-id delivery-a \
	--member <primary-workspace> \
	--member <secondary-workspace>

boundline cluster status --workspace <primary-workspace>
boundline cluster inspect --workspace <primary-workspace>
```

The primary workspace owns `.boundline/cluster.toml` and remains the authoritative
owner of the active clustered session in `<primary-workspace>/.boundline/session.json`.
Member workspaces keep their own `.boundline/traces/` and local `.boundline/config.toml`
files so terminal evidence stays local to the workspace that executed the
bounded handoff.

Session-native clustered delivery uses the same bounded commands through the
primary workspace:

```bash
boundline start --cluster <primary-workspace>
boundline capture --cluster <primary-workspace> --goal "Fix the failing add test"
boundline plan --cluster <primary-workspace>
boundline run --cluster <primary-workspace>
boundline status --cluster <primary-workspace>
boundline inspect --cluster <primary-workspace>
```

## Init workflow

```bash
boundline init --workspace <workspace>
boundline doctor --workspace <workspace>
```

Optional domain bootstrap:

```bash
boundline init \
	--workspace <workspace> \
	--assistant codex \
	--domain systems \
	--domain react \
	--domain-standard "react=follow the shared UI system" \
	--context-binding "react|design-system|mcp:design-system" \
	--required-context-binding "react|design-reference|design/reference.md"
```

Assistant bootstrap accepts Claude, Copilot, Codex, and Gemini. If no explicit
`--route` values are supplied, init seeds planning, implementation,
verification, and review from the selected assistant's maintained default model
catalog and reports the result in `route_setup`, including seeded slots,
explicit overrides, and `inspect_or_edit: boundline config show --workspace ...`.
Explicit routes remain authoritative for their slots, and missing slots are
still backfilled from assistant defaults. Guided init now lists supported slots
inline, explains that blank input is allowed when assistant defaults can seed
the remaining slots, and shows a valid example such as
`planning=copilot:gpt-5.4`. When a selected runtime cannot provide the missing
defaults on the current machine, init falls back to another selected available
assistant and marks the seeded line with
`fallback-from=<runtime>-unavailable`; if no selected runtime can fill the
remaining slots, init stops with an actionable error instead of persisting
broken defaults.

Domain bootstrap can also seed bounded hygiene defaults. Boundline writes
merge-only ignore entries when selected domain families or repository cues make
them credible: universal Git patterns, technology patterns for active domain
families, and tool-specific files such as `.dockerignore`, `.prettierignore`,
`.eslintignore`, `.terraformignore`, or `.helmignore` only when those tools are
present. Legacy ESLint workspaces can receive a merge-only `.eslintignore`, and
Kubernetes cues such as `kustomization.yaml` or `k8s/` append bounded
Kubernetes-related exclusions to `.gitignore`. Existing operator-authored lines
are preserved.

When init would overwrite existing files, Boundline shows a preview and requires
`--force` to apply destructive updates.

You do not need `boundline init` to use the primary session-native workflow. Use it when you want scaffolded compatibility defaults or assistant setup.

## Init templates

`boundline init` works without a template flag. If you omit `--template`, Boundline
defaults to `bug-fix`.

Available starting templates:

- `bug-fix`: small targeted repair
- `change`: bounded implementation change
- `delivery`: broader delivery update

Templates are only starting points for the generated compatibility execution profile.

- Need a different starting point later: rerun `boundline init --workspace <workspace> --force --template <bug-fix|change|delivery>`
- Need another task of the same type: do not rerun init; start a new session and capture a new goal
- Need something custom: edit `<workspace>/.boundline/execution.json` directly when you intentionally want compatibility behavior

`init` template and `flow` are separate concerns: `init` bootstraps the
optional compatibility profile, while `boundline flow` selects the current session-native run shape.

## Config commands

### Show

```bash
boundline config show --workspace <workspace> --scope effective
boundline config show --workspace <workspace> --scope workspace
boundline config show --cluster <primary-workspace> --scope cluster
boundline config show --scope global
```

### Set delivery-stage routes

```bash
boundline config set --scope global --slot planning --runtime codex --model gpt-5-codex
boundline config set --cluster <primary-workspace> --scope cluster --slot planning --runtime codex --model gpt-5-codex
boundline config set --workspace <workspace> --scope workspace --slot verification --runtime copilot --model gpt-5.4
```

### Set review-role routes

```bash
boundline config set --workspace <workspace> --scope workspace --reviewer safety --runtime claude --model sonnet-4
boundline config set --workspace <workspace> --scope workspace --adjudicator --runtime codex --model gpt-5-codex
```

### Set runtime capability profiles

```bash
boundline config set-capability --workspace <workspace> --scope workspace --runtime claude --continuation unsupported --resume unsupported --validation supported --handoff-target unsupported --escalation-context supported --notes "requires a handoff for bounded continuation"
```

### Set slot effort policy

```bash
boundline config set-effort --workspace <workspace> --scope workspace --slot implementation --level high --fallback preserve --rationale "keep implementation on the highest-effort bounded path"
```

### Set domain templates

```bash
boundline config set-domain --workspace <workspace> --scope workspace --family react --enable --standards "follow the shared UI system"
boundline config set-domain --cluster <primary-workspace> --scope cluster --family systems --enable
boundline config unset-domain --workspace <workspace> --scope workspace --family react
```

### Bind external context inputs

```bash
boundline config bind-context --workspace <workspace> --scope workspace --family react --kind design-system --reference mcp:design-system --required
boundline config bind-context --workspace <workspace> --scope workspace --family react --kind design-reference --reference design/reference.md
boundline config unbind-context --workspace <workspace> --scope workspace --family react --kind design-system --reference mcp:design-system
```

### Unset values

```bash
boundline config unset --workspace <workspace> --scope workspace --slot planning
boundline config unset --cluster <primary-workspace> --scope cluster --slot planning
boundline config unset --workspace <workspace> --scope workspace --reviewer safety
boundline config unset-domain --workspace <workspace> --scope workspace --family react
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
execution stops with an explicit delegation packet instead of an opaque
assistant-binding error.
