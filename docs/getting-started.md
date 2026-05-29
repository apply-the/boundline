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

The current release documents Canon `0.61.0` support for the machine-facing
`canon governance start|refresh|capabilities --json` `v1` surface.

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
  evidence.
- `run` executes the next approved step.
- `status` reports the active state, next command, and any blocked or degraded
  follow-up.
- `inspect` shows the trace-backed explanation.

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