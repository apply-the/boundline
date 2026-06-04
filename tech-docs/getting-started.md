# Getting Started with Boundline

This guide is the practical companion to the README. It assumes you want the
fastest credible path from installation to one bounded session in a real
workspace.

For the project-scale model behind larger initiatives, read
[delivery-model.md](delivery-model.md).

## Quick Start

Use this when you want the shortest credible route from install to one active
session in a real repository:

```bash
boundline doctor --install
cd <workspace>
boundline init --assistant codex
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
boundline config show --workspace <workspace>
```

## 1. Install Boundline

Use the release-aligned path that matches your machine:

- macOS via Homebrew:

```bash
brew tap apply-the/boundline
brew install boundline
```

- Windows via winget after the release manifest is published:

```powershell
winget install ApplyThe.Boundline
```

- Source fallback when bundled channels are unavailable:

```bash
git clone https://github.com/apply-the/boundline.git
cd boundline
cargo install --path .
```

## 2. Verify The Install

Run install diagnostics before touching workspace state:

```bash
boundline doctor --install
```

Read the output literally:

- `boundline_version` is the CLI version you are actually running.
- `supported_canon_version` is the Canon compatibility target for this release.
- `companion_state` tells you whether the local Boundline-plus-Canon pairing is
  ready, already satisfied, blocked, or repair-needed.
- `actions` tells you the next repair or follow-up step.

The current 0.70.0 release documents Canon `0.67.0` support for the machine-facing
`canon governance start|refresh|capabilities --json` `v1` surface.

During normal planning, expect the runtime to surface `plan_quality_state`,
`backlog_quality_state`, and `planning_analysis_state` before execution is
offered. Treat those as authoritative runtime gates rather than chat guidance.

## 3. Initialize The Workspace

From the target repository:

```bash
cd <workspace>
boundline init --assistant codex
```

Use `init` when you want Boundline to bootstrap workspace-local state, route
defaults, and repo-local assistant surfaces. The authoritative session state
still lives under `.boundline/`, especially `.boundline/session.json`.

Init also creates the default repo-visible document roots `docs/project/` and
`docs/evidence/`. Use `docs/project/` for stable reusable inputs, and use
`docs/evidence/` for durable feature outputs that should remain in the
repository after a delivery slice completes. See
[project-memory-and-evidence-structure.md](project-memory-and-evidence-structure.md)
for the ownership model.

If you do not need a repo-local assistant surface yet, you can still run:

```bash
boundline init
```

If you want a local Ollama route profile, start Ollama first, pull the models
for the hardware class, and initialize with one preset:

```bash
ollama pull qwen2.5:7b
ollama pull qwen2.5-coder:7b
boundline init --ollama-profile small
```

Use `small` for Apple Silicon with 16 GB unified memory, `medium` for a local
64 GB workstation, and `large` for 96/128 GB unified-memory or workstation
hosts. The preset writes `codex:ollama/<model>` routes for planning,
implementation, verification, and review while keeping endpoint configuration
in the provider env template through `OLLAMA_BASE_URL`.

If you want local semantic expansion plus explicit derived-index lifecycle
management in the same workspace, enable it deliberately after init:

```bash
boundline config set-semantic-acceleration --scope workspace --policy local
boundline index status --workspace <workspace>
boundline index refresh --workspace <workspace>
```

If you also want Git freshness events to mark the derived index stale, rerun
init with:

```bash
boundline init --workspace <workspace> --semantic-index-hook-action mark-stale
```

## 4. Optional Provider Auth

Use provider auth when the selected runtime needs a credential that is not
already available through the environment.

```bash
boundline models auth login --provider github-copilot
boundline models auth status
```

In this slice, `models auth` supports `github-copilot`. Credentials are stored
in the user-global Boundline config area, not inside the repository, so one
successful login can be reused across multiple workspaces and assistant hosts.

Use removal when you want to clear the stored profile explicitly:

```bash
boundline models auth remove --provider github-copilot
```

## 5. Optional Readiness Probe

Use `probe` when you want a read-only answer to one question: should you
bootstrap, repair, or continue the active session path?

```bash
boundline probe
```

`probe` reports:

- whether the workspace has been initialized
- whether provider prerequisites are ready
- whether a session is already active
- the next recommended CLI step

`probe` does not mutate session state, and it is not a repo-local
`/boundline:*` command.

When the workspace uses local semantic acceleration, `probe` also surfaces
derived-index capability signals so assistants can tell the difference between
missing bootstrap, degraded vector capability, and a healthy local index.

## 5a. Optional Framework Adapter Setup

If the workspace should use one explicit framework adapter, register it after
init instead of editing `.boundline/config.toml` by hand:

```bash
boundline adapter add speckit --workspace <workspace>
boundline adapter show --workspace <workspace> --json
```

Read the adapter JSON report literally:

- `compatibility_line` is the host-owned protocol line the adapter claims
- `supported_transports` must include V1 JSON over stdin/stdout
- `declared_stage_overrides` and `declared_hook_subscriptions` tell you which
  lifecycle surfaces the adapter may own or observe
- `config_state`, `interactive_resolution`, and `value_count` tell you whether
  setup is complete and how it was resolved

The V1 adapter boundary is deliberately small: one-shot subprocess commands,
standard success or error envelopes on stdout, optional structured stderr for
trace enrichment only, and no graceful shutdown or resident daemon lifecycle.

For the shipped Speckit profile, the corrected ownership map is explicit from
the start: `goal` stays native to Boundline, `plan` maps to workflow ID
`speckit-planning`, `run` maps to workflow ID `speckit-implementation`, and
`status` plus `inspect` remain Boundline-owned visibility surfaces. The adapter
executes the split workflow assets `.specify/workflows/speckit/planning.yml`
and `.specify/workflows/speckit/implementation.yml`, but the runtime and
operator surfaces continue to report the semantic workflow IDs.

## 6. Start One Bounded Session

The primary product story is explicit:

```bash
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
boundline status
boundline inspect
```

Read the runtime output literally:

- `goal` records the current bounded objective.
- `plan` assembles context and drafts bounded work from the current repository
  evidence, then enforces the same planning-readiness gates the runtime will
  use before execution handoff.
- `run` executes the next approved step.
- `status` reports the active state, next command, and any blocked or degraded
  follow-up.
- `inspect` shows the trace-backed explanation.

When semantic acceleration is enabled, `status` and `inspect` also show
`retrieval_index_state`, `semantic_capability_state`,
`semantic_fallback_reason`, and `retrieval_recovery_guidance` so local vector
health stays explicit.

When plan quality stops progress, `plan` emits one `phase_request` instead of
guessing. Answer the question about the missing validation strategy or other
blocking plan input, then resume the same session rather than forcing `run`.

Planning and execution may stop instead of guessing. In particular, the runtime
can surface planning-gate outcomes such as `goal_quality_state`,
`plan_quality_state`, `backlog_quality_state`, and
`planning_analysis_state`. When that happens, follow the printed continuation,
clarification request, or repair action instead of forcing execution.

If you want the shortest path after init, use the fast path explicitly:

```bash
boundline run --goal "Fix the failing add test"
```

That is a convenience path, not the primary product story.

If a selected adapter is active, `status` and `inspect` also surface the stage
execution source, adapter ID, routing reason, and hook-delivery outcomes. When
an adapter blocks before claim because config is incomplete or transport
compatibility is wrong, the runtime keeps that pre-claim stop or fallback
reason explicit instead of silently converting the run into adapter ownership.

When the active adapter is Speckit, read the stage output literally. A claimed
`plan` stage must finish with a mandatory `speckit.analyze` readiness gate and
may use one initial analyze pass plus at most two remediation or analyze
re-check cycles before it returns `blocked`. A claimed `run` stage is
implementation-only and must not rerun planning commands. `status` and
`inspect` remain the host-owned way to see workflow ID, produced artifacts,
planning findings, remediation counters, implementation validation refs, and
hook-delivery outcomes.

## When Canon Matters

Canon is optional. Most local delivery sessions can stay on the normal
session-native path.

Use Canon options during init only when governed delivery is expected:

```bash
boundline init \
  --assistant codex \
  --canon-mode-selection auto-confirm \
  --risk medium \
  --zone engineering \
  --owner platform
```

If the Canon surface is unavailable or incompatible, init and install
diagnostics keep that boundary explicit instead of silently downgrading to a
different governed path.

## Next Pages

- [README.md](../README.md) for the shortest entry path
- [architecture.md](architecture.md) for runtime, assistant, and Canon
  boundaries
- [configuration.md](configuration.md) for config precedence and auth/profile
  scope
- [guides/init-and-update.md](guides/init-and-update.md) for the full
  bootstrap and refresh workflow
