---
description: "Recommend the next bounded Synod action"
---

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
Summarize `routing`, `execution_condition`, `continuity_authority`, `compatibility_follow_up`, `compatibility_trace_ref`, `latest_status`, `explanation`, and the CLI-reported `next_command`. Preserve `latest_trace_ref` when present so `/synod-inspect` can reuse it.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if it points to inspect or `continuity_authority: compatibility_trace` is present, route to `/synod-inspect`. Route to `/synod-start` only when the CLI reports no active session and no compatibility follow-up.
Allowed follow-up commands: `/synod-step`, `/synod-inspect`, `/synod-status`, `/synod-plan`, `/synod-start`.