---
description: "Recommend the next bounded Synod action"
---

# Command: /synod-next

Shared guidance: `assistant/README.md`

## Intent
Recommend the next bounded Synod action after a run, failure, or inspection.

## Required Context
- `workspace_ref` or `trace_ref`
- Latest known outcome when already available

## Shell-Enabled Path
If current evidence is incomplete and a workspace is available, run `cargo run --bin synod -- inspect --workspace <workspace>` once to gather the latest trace summary. Then recommend one next command.

## Chat-Only Path
If shell execution is unavailable and evidence is missing, provide this exact copyable command:

`cargo run --bin synod -- inspect --workspace <workspace>`

Wait for pasted output and then recommend exactly one next command.

## Output Interpretation
Summarize terminal status, recovery signals, blocking issues, and the single most useful next action.

## Next-Step Routing
Allowed follow-up commands: `/synod-run`, `/synod-step`, `/synod-status`, `/synod-inspect`, `/synod-start`.