---
description: "Plan a Synod workflow"
---

# Command: /synod-plan

Shared guidance: `assistant/README.md`

## Intent
Capture human-authored input and plan the active session into a resumable task.

## Required Context
- `workspace_ref`
- At least one authored input source: bounded goal text and/or workspace-relative Markdown brief path(s)

## Shell-Enabled Path
If the workspace and at least one authored input source are known, run the matching capture command exactly once, then run plan:

`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`
`cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- plan --workspace <workspace>`

Ask only for missing workspace or missing authored input. Reuse confirmed brief paths instead of asking for them again.

## Chat-Only Path
Ask only for the missing workspace or authored input, then provide the matching exact copyable capture command followed by plan:

`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"`
`cargo run --bin synod -- capture --workspace <workspace> --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>" --brief <path> [--brief <path> ...]`
`cargo run --bin synod -- plan --workspace <workspace>`

Tell the user to run them one at a time and paste the outputs before continuing.

## Output Interpretation
Summarize the captured goal or `authored_input_summary`, `authored_input_sources`, any requested governance intent, the resulting plan state, and the CLI-reported `next_command`.

## Next-Step Routing
Prefer the CLI-reported `next_command`; otherwise move to `/synod-step` or `/synod-run` once planning succeeds.
Allowed follow-up commands: `/synod-step`, `/synod-run`, `/synod-plan`, `/synod-start`.
