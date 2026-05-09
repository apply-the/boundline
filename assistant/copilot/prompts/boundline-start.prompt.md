---
description: "Start a new Boundline workflow"
---

# Command: /boundline-start

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Initialize or reinitialize the active Boundline session for a workspace.

## Required Context
- `workspace_ref`
- Broad goal only when it is already known and helps you hand off to `/boundline-plan`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- start --workspace <workspace> --json` exactly once. If the workspace is missing, ask for it before proceeding.

## Chat-Only Path
If shell execution is unavailable, ask only for the workspace and then provide this exact copyable command:

`cargo run --bin boundline -- start --workspace <workspace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Summarize the initialized session state, confirmed `workspace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Prefer `/boundline-plan` after a successful start.
Allowed follow-up commands: `/boundline-plan`, `/boundline-start`.
