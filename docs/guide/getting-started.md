# Getting Started

This page gets you from no local Boundline run to one inspected bounded
session. [[Quick Start|Quick-Start]] is the shortest path; this page is the
guided walkthrough.

## Quick Start

For the absolute shortest route, see [[Quick Start|Quick-Start]]. The minimum
sequence is:

```bash
boundline doctor --install
cd <workspace>
boundline init --assistant codex
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
boundline config show --workspace <workspace>
```

The sections below explain each step in detail.

## 1. Install Or Build Boundline

Use a release channel when you want the supported product path:

```bash
brew tap apply-the/boundline
brew install boundline
```

On Windows, use the published winget package when the release manifest is
available:

```powershell
winget install ApplyThe.Boundline
```

For source validation or unreleased branches:

```bash
git clone https://github.com/apply-the/boundline.git
cd boundline
cargo install --path .
```

## 2. Verify The Install

Run the install diagnostic command before changing a workspace:

```bash
boundline doctor --install
```

Read the grouped output literally. The important fields are:

- `boundline_version`
- `supported_canon_version`
- `companion_state`
- `actions`

The current Boundline line documents Canon `0.63.0` support for the
machine-facing `canon governance start|refresh|capabilities --json` `v1`
surface.

## 3. Initialize The Workspace

From the target repository:

```bash
cd <workspace>
boundline init --assistant codex
```

Initialize only when you want Boundline to write workspace-local state, route
defaults, assistant packages, or setup metadata.

The active session state remains `.boundline/session.json`.

Init also creates `docs/project/` and `docs/evidence/` as the default
repo-visible document roots. Use `docs/project/` for stable reusable inputs and
`docs/evidence/` for durable feature outputs. See
[[Project Memory Structure|Project-Memory-Structure]].

If you want local semantic expansion plus explicit derived-index lifecycle
management in the same workspace, enable it deliberately after init:

```bash
boundline config set-semantic-acceleration --scope workspace --policy local
boundline index status --workspace <workspace>
boundline index refresh --workspace <workspace>
```

If you also want Git freshness hooks to mark the derived index stale, rerun
init with:

```bash
boundline init --workspace <workspace> --semantic-index-hook-action mark-stale
```

## 4. Optional Provider Auth

If the chosen runtime needs a stored credential, authenticate before the first
session:

```bash
boundline models auth login --provider github-copilot
boundline models auth status
```

In the current public slice, the login surface supports `github-copilot`.
These credentials are user-scoped, so they can be reused across repositories.

## 5. Optional Readiness Probe

Use `probe` when you want a read-only readiness answer before session work:

```bash
boundline probe
```

`probe` tells you whether the workspace still needs bootstrap, needs repair, or
is ready for `goal`, `plan`, or `run`.

When local semantic acceleration is enabled, `probe` also surfaces derived-
index health and hook state so assistants can distinguish bootstrap gaps from a
degraded local vector surface.

## 5a. Optional Framework Adapter Setup

If the workspace should use one explicit framework adapter, register it after
init instead of editing `.boundline/config.toml` directly:

```bash
boundline adapter add speckit --workspace <workspace>
boundline adapter show --workspace <workspace> --json
```

Read the adapter JSON report literally:

- `compatibility_line` is the host-owned protocol line the adapter claims
- `supported_transports` must include V1 JSON over stdin/stdout
- `declared_stage_overrides` and `declared_hook_subscriptions` tell you what
	the adapter may own or observe
- `config_state`, `interactive_resolution`, and `value_count` tell you whether
	setup is complete and how it was resolved

Current public repositories for this boundary:

- [boundline-framework-template](https://github.com/apply-the/boundline-framework-template): use this as the starting scaffold for a custom compatible adapter.
- [boundline-adapter-speckit](https://github.com/apply-the/boundline-adapter-speckit): use this when the workspace should bridge into the Speckit implementation rather than the native planner or runner.

The V1 adapter boundary is intentionally narrow: one-shot subprocess commands,
standard success or error envelopes on stdout, optional structured stderr for
trace enrichment only, and no graceful shutdown or resident daemon lifecycle.

## 6. Run One Minimal Session

Use the explicit session-native flow when you want the normal operator path:

```bash
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
boundline status
boundline inspect
```

Plain English version:

- `goal` records the current bounded objective.
- `plan` assembles context and drafts bounded work.
- `run` executes the next approved step.
- `status` shows the current state and next command.
- `inspect` shows the trace-backed explanation.

When semantic acceleration is enabled, `status` and `inspect` also surface the
derived-index state, semantic capability, fallback disclosure, and recovery
guidance.

Planning and execution can stop explicitly when quality or context is not yet
credible. When you see gate fields such as `goal_quality_state`,
`plan_quality_state`, `backlog_quality_state`, or `planning_analysis_state`,
follow the printed continuation instead of forcing the run.

If you want a shorter path after init, use the fast path deliberately:

```bash
boundline run --goal "Fix the failing add test"
```

If a selected adapter is active, `status` and `inspect` also surface the stage
execution source, adapter ID, routing reason, and hook-delivery outcomes.

## 7. Check That Setup Worked

After the first run, verify:

```bash
boundline status
boundline inspect
```

Look for:

- a clear session status
- a next command or explicit terminal state
- trace location
- blocked, degraded, or clarification-required follow-up when present
- context credibility and validation posture

## Next Pages

- [[Daily Operating Guide|Daily-Operating-Guide]] for the command loop
- [[Installation And Setup|Installation-And-Setup]] for setup variants
- [[Troubleshooting]] when diagnostics, planning, or provider setup fails