# Command: /synod-run

Shared guidance: `assistant/README.md`

## Intent
Resume the active Synod session until it reaches a terminal outcome.

## Required Context
- `workspace_ref`
- Active session state; do not ask for a new goal when one is already captured

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin synod -- run --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, ask only for missing workspace context and then provide this exact copyable command:

`cargo run --bin synod -- run --workspace <workspace>`

Wait for the user to paste the output before continuing.

## Output Interpretation
Summarize `terminal_status`, `terminal_reason`, `trace`, and `next_command`. Preserve the returned trace reference for later `/synod-inspect` use.

## Next-Step Routing
Prefer the CLI-reported `next_command`; when inspection is needed, route to `/synod-inspect`.
Allowed follow-up commands: `/synod-inspect`, `/synod-status`, `/synod-next`, `/synod-run`, `/synod-plan`, `/synod-start`.