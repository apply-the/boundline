# Quick Start

Use this page for the shortest credible path. If you want the guided version,
read [[Getting Started|Getting-Started]].

## 1. Verify The Install

```bash
boundline doctor --install
```

Do not skip this. It tells you whether the installed Boundline binary and the
documented Canon pairing are ready.

## 2. Initialize The Workspace

```bash
cd <workspace>
boundline init --assistant codex
boundline config set-semantic-acceleration --scope workspace --policy local
boundline index status --workspace .
```

That bootstraps `.boundline/` and, when requested, the repo-local assistant
surface for the selected host.

## 3. Optional Provider Auth

Use this only when the chosen runtime needs a stored provider credential:

```bash
boundline models auth login --provider github-copilot
boundline models auth status
```

These credentials are user-scoped, not repository-scoped.

## 4. Optional Readiness Probe

Use `probe` when you want a read-only answer before starting or resuming:

```bash
boundline probe
```

If `probe` says bootstrap is still required, go back to `init`. If it says
repair is needed, follow the printed action. If it says the session is ready,
continue with the normal loop.

If the workspace uses local semantic retrieval, add:

```bash
boundline index refresh --workspace .
boundline index doctor --workspace .
```

Use `refresh` to rebuild bounded local evidence and `doctor` when the manifest,
tracked-file hygiene, or vector capability looks wrong.

## 5. Run One Bounded Session

```bash
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
boundline status
boundline inspect
```

This is the primary product path.

If you want the shortest path after init, you can use:

```bash
boundline run --goal "Fix the failing add test"
```

Treat that as a fast path, not the default mental model.

For an explicit planning seed during bootstrap, use `planning=copilot:gpt-4o`:

```bash
boundline init --assistant copilot --route planning=copilot:gpt-4o
```

## What This Path Gives You

- explicit session state under `.boundline/`
- a bounded plan built from repository evidence
- read-side status and trace inspection
- explicit stop conditions when context, validation, or governance is not ready

Next step: read [[Daily Operating Guide|Daily-Operating-Guide]] when you want
the normal loop, follow-through, and recovery behavior in more detail.