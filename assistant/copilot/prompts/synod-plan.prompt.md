---
description: "Plan a Synod workflow"
---

# Command: /synod-plan

Shared guidance: `assistant/README.md`

## Intent
Capture the goal and plan the active session into a resumable task.

## Required Context
- `workspace_ref`
- A bounded goal, or the minimum missing detail needed to capture one

## Shell-Enabled Path
If the workspace and bounded goal are known, run these commands exactly once and in order:

`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`
`cargo run --bin synod -- plan --workspace <workspace>`

If either field is missing, ask only for the missing value before running them.

## Chat-Only Path
Ask only for the missing workspace or goal, then provide these exact copyable commands in order:

`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`
`cargo run --bin synod -- plan --workspace <workspace>`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Summarize the captured goal, resulting plan state, and the CLI-reported `next_command`.

## Next-Step Routing
Prefer the CLI-reported `next_command`; otherwise move to `/synod-step` or `/synod-run` once planning succeeds.
Allowed follow-up commands: `/synod-step`, `/synod-run`, `/synod-plan`, `/synod-start`.
