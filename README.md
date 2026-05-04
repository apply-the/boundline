# Boundline

![Boundline banner](docs/images/boundline-banner.jpg)

[![CI](https://github.com/apply-the/boundline/actions/workflows/ci.yml/badge.svg)](https://github.com/apply-the/boundline/actions/workflows/ci.yml)
[![Lint](https://github.com/apply-the/boundline/actions/workflows/lint.yml/badge.svg)](https://github.com/apply-the/boundline/actions/workflows/lint.yml)
[![Vulnerabilities](https://github.com/apply-the/boundline/actions/workflows/vulnerabilities.yml/badge.svg)](https://github.com/apply-the/boundline/actions/workflows/vulnerabilities.yml)
[![Coverage](https://codecov.io/gh/apply-the/boundline/graph/badge.svg)](https://codecov.io/gh/apply-the/boundline)

**Boundline is a local CLI for taking a small engineering task from goal to code change.**
Point it at a workspace, give it a goal, let it run a bounded change, then use
`status` and `inspect` to see what happened. Canon is optional: most users can
ignore it unless they need governed stages or governed artifacts.

## Quick Path Brutale

If Boundline is already installed, this is the shortest path to doing
something useful:

```bash
cd <workspace>
boundline doctor --workspace .
boundline run --workspace . --goal "Fix the failing add test"
boundline status --workspace .
boundline inspect --workspace .
```

If you want to review the plan before execution, use the explicit flow instead:

```bash
cd <workspace>
boundline start --workspace .
boundline capture --workspace . --goal "Fix the failing add test"
boundline plan --workspace .
boundline plan --workspace . --confirm
boundline run --workspace .
boundline status --workspace .
boundline inspect --workspace .
```

Plain English version of that flow:

- `start` opens the session.
- `capture` records the goal.
- `plan` drafts the work.
- `plan --confirm` approves that draft.
- `run` executes it.
- `status` and `inspect` tell you what happened.

The primary product route stays explicit: `session-native: start a session -> capture a goal -> plan -> confirm -> run -> status -> inspect`.

Most users only need the commands above.

## Read This In Two Layers

- Quick path: this README plus [docs/getting-started.md](docs/getting-started.md)
- Advanced architecture: [docs/architecture.md](docs/architecture.md)
- Assistant-specific command packs: [assistant/README.md](assistant/README.md)

Stop here if all you need is install, verify, and run. Continue into
`docs/architecture.md` only when you need routing, workflow, cluster,
advanced execution-profile, or governance detail.

## Install

- macOS via Homebrew:

```bash
brew install https://raw.githubusercontent.com/apply-the/boundline/v0.40.0/distribution/homebrew/Formula/boundline.rb
```

- Windows via winget:

```powershell
winget install ApplyThe.Boundline
```

- Source fallback:

```bash
git clone https://github.com/apply-the/boundline.git
cd boundline
cargo install --path .
```

Then verify the install:

```bash
boundline doctor --install
```

That shows the Boundline version, the documented Canon compatibility target,
and whether the local pairing is ready, already satisfied, blocked, or needs
repair.

## Good Fit

Use Boundline when you want to:

- fix a failing test, lint error, or small bug in one repository
- make a scoped change from a short goal or a Markdown brief
- run a repo-defined workflow such as a governed delivery path
- coordinate one bounded change across a small registered cluster of repos

It is not meant to be a general deployment tool or an open-ended system for
huge refactors.

Advanced execution-profile workflows are documented outside this README.

`.boundline/execution.json` remains available as an explicit compatibility path when you intentionally need the manifest-backed route.

## Command Legend

| Command | Use it for |
| --- | --- |
| `boundline doctor --install` | Verify the installed Boundline plus Canon pairing |
| `boundline doctor --workspace <workspace>` | Check that a workspace is ready |
| `boundline run --workspace <workspace> --goal "..."` | Fastest way to do something useful |
| `boundline start` | Open or reset the active session |
| `boundline capture --goal "..."` | Save the goal or brief into the session |
| `boundline flow bug-fix|change|delivery` | Force the change type instead of inferring it |
| `boundline plan` | Generate the proposed plan |
| `boundline plan --confirm` | Approve that plan so execution can continue |
| `boundline step` | Run one step at a time |
| `boundline status` | See the current state and suggested follow-up |
| `boundline next` | Ask Boundline for the next action |
| `boundline inspect` | Read the latest trace in more detail |
| `boundline init` | Scaffold optional `.boundline` files and assistant setup |
| `boundline config` | Inspect or change routing and domain defaults |
| `boundline workflow ...` | Run a named workflow defined by the repo |
| `boundline cluster ...` | Set up or inspect a multi-repo cluster |

## Files Boundline Uses

| File | What it is |
| --- | --- |
| `.boundline/session.json` | Current session state for the default flow |
| `.boundline/traces/` | Execution history and inspectable traces |
| `.boundline/config.toml` | Local routing and domain-template settings |

## Common Examples

Run directly from a goal:

```bash
boundline run --workspace . --goal "Fix the failing add test"
```

Run from a Markdown brief:

```bash
boundline start --workspace .
boundline capture --workspace . --brief docs/brief.md
boundline plan --workspace .
boundline plan --workspace . --confirm
boundline run --workspace .
```

Run a named workflow when the repo defines one:

```bash
boundline workflow list --workspace .
boundline workflow run governed-delivery --workspace . --goal "Fix the failing add test"
```

Use a cluster when one change spans multiple repos:

```bash
boundline cluster init \
	--workspace <primary-workspace> \
	--cluster-id delivery-a \
	--member <primary-workspace> \
	--member <secondary-workspace>

boundline run --cluster <primary-workspace> --goal "Fix the failing add test"
```

## Boundline And Canon

Boundline is the main tool. Canon is a supporting governed runtime.

- Boundline owns the operator flow, session state, planning, execution, and validation.
- Canon only enters when you explicitly want governed stages, approvals, or governed artifacts.

The current release documents Canon `0.39.0` support on the
`canon governance start|refresh|capabilities --json` `v1` adapter surface.

## Read More

Keep this README short. Use the other docs only when you need more detail.

- [docs/getting-started.md](docs/getting-started.md) for a longer first-run walkthrough
- [docs/architecture.md](docs/architecture.md) for routing, governance, compatibility, and cluster details
- [docs/configuration.md](docs/configuration.md) for `init`, config precedence, and advanced execution-profile setup
- [docs/adaptive-execution.md](docs/adaptive-execution.md) for advanced adaptive manifest-backed execution
- [docs/review-voting.md](docs/review-voting.md) for review councils on the advanced manifest-backed path
- [assistant/README.md](assistant/README.md) for assistant command packs
- [CONTRIBUTING.md](CONTRIBUTING.md) for contributor workflow
- [ROADMAP.md](ROADMAP.md) for planned releases
- [CHANGELOG.md](CHANGELOG.md) for released changes

## Local Validation

Run these commands from the repository root:

If you install the repository hooks with `./scripts/install-hooks.sh`,
`pre-push` runs the same formatting, lint, test, and coverage checks used by
the blocking GitHub workflows.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```
