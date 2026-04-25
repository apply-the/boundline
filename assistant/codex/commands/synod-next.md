# Command: /synod-next

Shared guidance: `assistant/README.md`

## Intent
Ask Synod for the next recommended session command.

## Required Context
- `workspace_ref`
- Latest known outcome when already available

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin synod -- next --workspace <workspace>` exactly once and use the reported recommendation.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin synod -- next --workspace <workspace>`

Wait for pasted output and then recommend exactly one next command.

## Output Interpretation
Summarize `latest_status`, `explanation`, and the CLI-reported `next_command`. Preserve `latest_trace_ref` when present so `/synod-inspect` can reuse it.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if it points to inspect, route to `/synod-inspect`, and if the session is missing or invalid, route to `/synod-start`.
Allowed follow-up commands: `/synod-step`, `/synod-inspect`, `/synod-status`, `/synod-plan`, `/synod-start`.