---
description: "Start a new Synod workflow"
---

# Command: /synod-start

Shared guidance: `assistant/README.md`

## Intent
Establish workspace readiness and collect only the missing context needed to begin a Synod workflow.

## Required Context
- `workspace_ref`
- Broad user goal only when it is already known and helps choose the next command

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin synod -- doctor --workspace <workspace>` exactly once and summarize readiness. If the workspace is missing, ask for it before proceeding.

## Chat-Only Path
If shell execution is unavailable, ask only for the workspace and then provide this exact copyable command:

`cargo run --bin synod -- doctor --workspace <workspace>`

Wait for pasted output before continuing.

## Output Interpretation
Summarize readiness, missing prerequisites, and whether the user should continue with `/synod-plan` or `/synod-run`.

## Next-Step Routing
Allowed follow-up commands: `/synod-plan`, `/synod-run`, `/synod-start`.
