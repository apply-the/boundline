---
description: "Start a new Synod workflow"
---

# Command: /synod-start

Shared guidance: `assistant/README.md`

## Intent
Initialize or reinitialize the active Synod session for a workspace.

## Required Context
- `workspace_ref`
- Broad goal only when it is already known and helps you hand off to `/synod-plan`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin synod -- start --workspace <workspace>` exactly once. If the workspace is missing, ask for it before proceeding.

## Chat-Only Path
If shell execution is unavailable, ask only for the workspace and then provide this exact copyable command:

`cargo run --bin synod -- start --workspace <workspace>`

Wait for pasted output before continuing.

## Output Interpretation
Summarize the initialized session state, confirmed `workspace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Prefer `/synod-plan` after a successful start.
Allowed follow-up commands: `/synod-plan`, `/synod-start`.
