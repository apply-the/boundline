---
description: "Execute a bounded Synod workflow"
---

# Command: /synod-run

Shared guidance: `assistant/README.md`

## Intent
Execute a bounded Synod workflow with a confirmed workspace and goal.

## Required Context
- `workspace_ref`
- Goal phrased as one bounded developer outcome

## Shell-Enabled Path
Run `cargo run --bin synod -- run --workspace <workspace> --goal "<goal>"` exactly once.

## Chat-Only Path
If shell execution is unavailable, ask only for missing context and then provide this exact copyable command:

`cargo run --bin synod -- run --workspace <workspace> --goal "<goal>"`

Wait for the user to paste the output before continuing.

## Output Interpretation
Summarize terminal status, key executed steps, recovery signals, trace location, and the most useful next command.

## Next-Step Routing
On success, prefer `/synod-status` or `/synod-inspect` when inspection is useful.
On non-success, prefer `/synod-next` or `/synod-inspect`.