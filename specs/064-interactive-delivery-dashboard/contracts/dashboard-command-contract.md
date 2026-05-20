# Contract: Dashboard Command Surface

## Purpose

Define how operators start the dashboard and how dashboard launch failures point back to normal Boundline commands.

## Commands

### `boundline-dashboard`

Dedicated dashboard entrypoint.

```text
boundline-dashboard [--workspace <path>] [--no-color] [--snapshot-json]
```

Expected behavior:

- Without `--workspace`, resolves the workspace using the same workspace discovery rules as normal Boundline commands.
- With `--workspace`, targets that workspace explicitly.
- With `--no-color`, renders without color.
- With `--snapshot-json`, emits the current dashboard snapshot for contract and CI validation instead of starting interactive rendering.

### `boundline dashboard`

Normal command-surface entrypoint or launcher.

```text
boundline dashboard [--workspace <path>] [--no-color]
```

Expected behavior:

- Starts the dashboard when the dashboard entrypoint is available.
- If the dashboard entrypoint is unavailable, reports a clear message and valid fallback commands.
- Does not duplicate dashboard rendering or state semantics inside the normal command surface.

## Launch Outcomes

| Outcome | Meaning | Required Next Step |
|---------|---------|--------------------|
| `started` | Interactive dashboard started | None |
| `snapshot_emitted` | Snapshot JSON emitted | Consumer validates snapshot |
| `degraded` | Dashboard could not start fully | Show fallback command |
| `invalid_workspace` | Workspace discovery failed | Show init or workspace selection command |
| `dashboard_unavailable` | Dashboard entrypoint not installed or not built | Show dedicated install or command fallback guidance |

## Required Fallback Commands

Fallback output must prefer existing normal commands:

- `boundline init --workspace <path>` when workspace setup is missing.
- `boundline start --workspace <path>` when no session exists.
- `boundline status --workspace <path>` when state can be inspected normally.
- `boundline inspect --workspace <path>` when trace inspection is the best next step.
- `boundline plan --confirm --workspace <path>` when confirmation is required.
- `boundline run --workspace <path>` when execution is ready.

## Invalid Behavior

- The launcher must not silently create dashboard-only state.
- The launcher must not hide normal command fallbacks.
- The launcher must not claim governed references are required for dashboard launch.
- The launcher must not proceed with mutating actions from stale snapshots.
