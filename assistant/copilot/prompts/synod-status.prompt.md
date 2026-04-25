---
description: "Summarize the latest known status of a Synod workflow"
---

# Command: /synod-status

Shared guidance: `assistant/README.md`

## Intent
Summarize the latest known status of a workflow in a workspace.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
Run `cargo run --bin synod -- inspect --workspace <workspace>` to inspect the latest trace in that workspace.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`cargo run --bin synod -- inspect --workspace <workspace>`

Then wait for pasted output.

## Output Interpretation
Summarize final status, key step results, recovery events, and the trace reference when present.

## Next-Step Routing
On success, route to `/synod-next` or `/synod-inspect` if deeper inspection is needed.
On non-success, route to `/synod-next` or `/synod-start` if readiness must be re-established.