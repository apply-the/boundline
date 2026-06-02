# Installation And Setup

Boundline setup has four layers: CLI installation, workspace initialization,
optional provider auth, and optional assistant package surfaces.

## CLI Installation

Use a release channel when you want the supported product path:

```bash
brew tap apply-the/boundline
brew install boundline
boundline doctor --install
```

On Windows:

```powershell
winget install ApplyThe.Boundline
boundline doctor --install
```

For local development, unreleased validation, or when release channels are not
available:

```bash
git clone https://github.com/apply-the/boundline.git
cd boundline
cargo install --path .
boundline doctor --install
```

`boundline doctor --install` verifies the installed Boundline executable and
the documented Canon pairing for the current release line.

## Workspace Initialization

Workspace setup is local to the repository you operate on:

```bash
cd <workspace>
boundline init --assistant codex
```

Use `init` when you want Boundline to create local state and optional setup
surfaces. Depending on flags, it can write:

- `.boundline/config.toml`
- `.boundline/session.json` during active sessions
- `.boundline/traces/`
- repo-local assistant package folders such as `.codex-plugin/` or
  `.claude-plugin/`

If you only need the runtime state and not a host package yet, `boundline init`
without `--assistant` is still valid.

## Optional Provider Auth

Use provider auth when a selected runtime needs a stored credential:

```bash
boundline models auth login --provider github-copilot
boundline models auth status
```

These credentials are user-scoped. They are stored outside the repository so a
single login can be reused across multiple workspaces.

Use removal when you want to clear a stored profile explicitly:

```bash
boundline models auth remove --provider github-copilot
```

## Optional Readiness Probe

Use `probe` as a read-only setup check before the first bounded session:

```bash
boundline probe
```

If probe reports bootstrap is still required, return to `init`. If it reports a
repair-needed state, follow the printed action. If it reports the session path
is ready, continue with `goal`, `plan`, and `run`.

## Global Assistant Package Setup

Global assistant packages are user-scoped and available before workspace init
when the host supports them:

```bash
boundline assistant install --host codex --scope user
boundline assistant install --host claude --scope user
boundline assistant install --host cursor --scope user
```

Global commands are intentionally limited to readiness and bootstrap surfaces
such as `/boundline:init`, `/boundline:doctor`, `/boundline:help`,
`/boundline:status`, and `/boundline:continue`.

## Repository-Local Assistant Setup

Repository-local packages are generated into a workspace by `boundline init
--assistant <host>`.

Typical package folders:

- Claude Code: `.claude-plugin/`
- Codex: `.codex-plugin/`
- Cursor: `.cursor-plugin/`
- Copilot prompt environments: `.copilot-prompts/` plus `.github/prompts/`

The CLI remains authoritative even when commands are exposed through a chat
host.

## Canon-Default Setup

Use Canon options during init only when governed delivery is expected:

```bash
boundline init \
  --assistant codex \
  --canon-mode-selection auto-confirm \
  --risk medium \
  --zone engineering \
  --owner platform
```

The current release documents Canon `0.61.0` support for the machine-facing
`canon governance start|refresh|capabilities --json` `v1` surface.

## Troubleshooting Setup Failures

Use the printed command from `doctor`, `probe`, or `status` first. Common setup
failures include:

- blocked install diagnostics
- workspace not writable
- assistant package generated but not registered by the host
- missing provider authentication
- trying to use repo-local commands before `init`

See [[Troubleshooting]] for recovery paths.