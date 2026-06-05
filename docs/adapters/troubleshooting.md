# Troubleshooting

Start every troubleshooting session from runtime output:

```bash
boundline status --workspace .
boundline next --workspace .
boundline inspect --workspace .
```

Use `--json` when an assistant or script needs structured fields.

## Init Problems

Run:

```bash
boundline doctor --install
boundline doctor --workspace .
```

Check:

- the `boundline` CLI is installed and available on `PATH`
- install companion state is ready or intentionally bypassed
- workspace path exists and is writable
- `.boundline/` can be created
- selected assistant package is supported
- generated package folders do not conflict with local files

## Assistant Command Not Available

First decide whether you expected a global command or repo-local command.

Global bootstrap:

```bash
boundline assistant install --host codex --scope user
```

Repo-local runtime package:

```bash
boundline init --workspace . --assistant codex
```

If the command still is not available, use the equivalent CLI command. Do not infer session state from chat history.

Remember the declared host support modes: Cursor is `copy-ready-assets` and
Gemini is `manual-fallback`. Missing package discovery on those hosts is not a
reason to invent repo-local parity in chat.

## Provider Not Authenticated

Symptoms:

- selected route cannot run
- host cannot access model
- assistant package is present but execution stops at delegation or runtime boundary

Inspect:

```bash
boundline config show --workspace . --scope effective
boundline status --workspace . --json
```

Confirm host authentication and model availability outside Boundline if needed.

If authentication or model access fails outside Boundline too, repair that first and then rerun the same Boundline command.

## Missing Routes

Run:

```bash
boundline config show --workspace . --scope effective
```

Look for route owner, source, assistant binding, capability profile, and effort policy. A package folder does not automatically mean every delivery slot is routed to that host.

## Weak Context

Planning should stop when context is weak. Repair weak context by narrowing the goal or adding a relevant brief or stronger evidence:

```bash
boundline goal --workspace . \
  --goal "Fix timeout handling in src/auth/session.rs" \
  --brief docs/incidents/session-timeout.md
boundline plan --workspace .
```

Inspect context fields:

- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance`
- `context_staleness_reason`

## Derived Index Lifecycle

When local semantic retrieval looks stale, degraded, or corrupt, inspect the
derived index directly instead of inferring from chat output:

```bash
boundline index status --workspace .
boundline index doctor --workspace .
boundline index refresh --workspace .
```

Use `status` to inspect the current lifecycle state, `doctor` to confirm
tracked-artifact, manifest, hook, or vector issues, and `refresh` to rebuild
bounded local evidence when the report recommends it. Use `boundline index
rebuild --workspace .` only when status or doctor reports an incompatible or
corrupt index that cannot be repaired incrementally.

## Failed Guardian

Inspect:

```bash
boundline inspect --workspace . --json
```

Look for:

- guardian id
- finding disposition
- evidence
- blocking outcome
- degraded or skipped checks
- suggested next action

Treat blocker and error findings as real stop conditions unless the operator explicitly changes policy.

## Trace Inspection

If output references a trace, inspect through the CLI:

```bash
boundline inspect --workspace .
```

Trace-backed fields explain planning, routing, guidance, guardian, validation, and recovery behavior. Prefer them over logs or chat summaries.

## Recovery

When a run is failed or blocked:

```bash
boundline status --workspace .
boundline next --workspace .
boundline inspect --workspace .
```

If a checkpoint restore command is reported, preserve it exactly. Use it only when you intentionally want to rewind the bounded workspace slice.

## Canon Or Governance Blocked

Check:

- Canon companion state from `doctor --install`
- workspace Canon config
- selected governance mode
- approval state
- supported contract line
- missing source artifacts
- authority stop semantics

Boundline should fail closed or stop when governed inputs are incompatible or missing. Do not bypass that by switching to chat-only assertions.

## Dashboard Degraded Or Unavailable

If `boundline-dashboard` cannot render interactively, rerun with `--snapshot-json` and compare the summary to `boundline status --workspace .` and `boundline inspect --workspace .`. Follow the fallback command printed by the dashboard before attempting another mutating action.

If `boundline dashboard` reports the dedicated entrypoint unavailable, build or install the `boundline-dashboard` binary for the same Boundline `0.72.0` release, or continue with the normal CLI commands.
