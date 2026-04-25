---
description: "Inspect a Synod trace and summarize outcome and recovery signals"
---

# Command: /synod-inspect

Shared guidance: `assistant/README.md`

## Intent
Inspect a specific or latest Synod trace from chat and summarize the outcome without requiring raw trace-file reading.

## Required Context
- `trace_ref` or `workspace_ref`

## Shell-Enabled Path
If `trace_ref` is known, run `cargo run --bin synod -- inspect --trace <trace>`. Otherwise, if `workspace_ref` is known, run `cargo run --bin synod -- inspect --workspace <workspace>`.

## Chat-Only Path
Ask only for the missing `trace_ref` or `workspace_ref`, then provide one exact copyable command:

`cargo run --bin synod -- inspect --trace <trace>`

or

`cargo run --bin synod -- inspect --workspace <workspace>`

Wait for pasted output before continuing. If trace reading fails, ask for a corrected trace reference or workspace and provide the replacement inspect command.

## Output Interpretation
Summarize terminal status, key step results, recovery signals, trace reference, and the most useful next command.

## Next-Step Routing
Allowed follow-up commands: `/synod-next`, `/synod-run`, `/synod-step`, `/synod-status`, `/synod-start`.